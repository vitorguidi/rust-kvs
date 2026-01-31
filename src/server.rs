use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::Framed;
use futures::{SinkExt, StreamExt};
use std::sync::Arc;
use std::time::Duration;
use std::io::Result;
use crate::bytestore::ByteCache;
use crate::network::{CacheCodec, Command, Response};

pub async fn run_server(cache: ByteCache<String>) -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379").await?;
    println!("Listening on 127.0.0.1:6379");
    loop {
        let (socket, addr) = listener.accept().await?;
        println!("Accepted connection from: {}", addr);
        let cache_handle = cache.clone();

        tokio::spawn(async move {
            if let Err(e) = process(socket, cache_handle).await {
                eprintln!("Error processing client {}: {:?}", addr, e);
            }
        });
    }
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
                if let Err(e) = framed.send(response).await {
                    return Err(e);
                }
            }
        }
    }
    Ok(())
}