use futures::{SinkExt, StreamExt};
use tokio::sync::Semaphore;
use std::io::Result;
use std::time::Duration;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::Framed;

use crate::cache::ByteCache;
use crate::protocol::{CacheCodec, Command, Response};


pub async fn run_server(
    cache: ByteCache<String>,
    listener: TcpListener,
    max_connections: usize
) -> Result<()> {
    let limit = Arc::new(Semaphore::new(max_connections));
    loop {
        let permit = match limit.clone().acquire_owned().await {
            Ok(p) => p,
            Err(_) => {
                break;
            }
        };
        let (socket, addr) = listener.accept().await?;
        println!("Accepted connection from: {}", addr);
        let cache_handle = cache.clone();

        tokio::spawn(async move {
            let _permit = permit;
            if let Err(e) = process(socket, cache_handle).await {
                eprintln!("Error processing client {}: {:?}", addr, e);
            }
        });
    }
    Ok(())
}

pub async fn process(
    socket: TcpStream,
    cache: ByteCache<String>
) -> Result<()> {
    let mut framed = Framed::new(socket, CacheCodec);
    println!("Processing connection...");

    while let Some(message_result) = framed.next().await {
        println!("Received message: {:?}", message_result);
        match message_result {
            Err(e) => return Err(e),
            Ok(command) => {
                let response = match command {
                    Command::Ping => {
                        Response::Ok
                    }
                    Command::Get {key} => {
                        match cache.get(&key) {
                            Some(data_arc) => {
                                let bytes = bytes::Bytes::copy_from_slice(&data_arc);
                                Response::Found(bytes)
                            }
                            None => Response::NotFound,
                        }
                    }
                    Command::Set {key, value, ttl_sec} => {
                        let mut aligned = rkyv::AlignedVec::new();
                        aligned.extend_from_slice(&value);
                        let ttl = if ttl_sec > 0 {
                            Some(Duration::from_secs(ttl_sec))
                        } else {
                            None
                        };
                        cache.set(key, aligned, ttl);
                        Response::Ok
                    }
                };
                println!("Replying: {:?}", response);
                if let Err(e) = framed.send(response).await {
                    return Err(e);
                }
            }
        }
    }
    Ok(())
}