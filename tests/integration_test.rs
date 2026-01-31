use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use rust_kvs::cache::ByteCache;
use rust_kvs::server::run_server;
use std::time::Duration;

async fn spawn_test_server() -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .unwrap();
    
    let addr = listener.local_addr().unwrap().to_string();
    let cache = ByteCache::new();
    tokio::spawn(
        async move {
            run_server(cache, listener, 20)
                .await
                .unwrap();
        }
    );

    tokio::time::sleep(Duration::from_millis(50)).await;
    addr
}

#[tokio::test]
async fn test_set_and_get() {
    let addr = spawn_test_server().await;
    let mut stream = TcpStream::connect(&addr).await.unwrap();

    let key = "integration_test_key";
    let val = "hello_world_value";
    let ttl_sec = 60u64;

    let mut set_cmd = Vec::new();
    set_cmd.push(0x02);

    set_cmd.extend_from_slice(&(key.len() as u32).to_be_bytes());
    set_cmd.extend_from_slice(key.as_bytes());
    set_cmd.extend_from_slice(&(val.len() as u32).to_be_bytes());
    set_cmd.extend_from_slice(val.as_bytes());
    set_cmd.extend_from_slice(&ttl_sec.to_be_bytes());
    stream.write_all(&set_cmd).await.unwrap();
    let mut response_buf = [0u8; 1];
    stream.read_exact(&mut response_buf).await.unwrap();
    assert_eq!(response_buf[0], 0x00, "Expected OK response for SET");

    let mut get_cmd = Vec::new();
    get_cmd.push(0x01);
    get_cmd.extend_from_slice(&(key.len() as u32).to_be_bytes());
    get_cmd.extend_from_slice(key.as_bytes());
    stream.write_all(&get_cmd).await.unwrap();
    let mut code_buf = [0u8; 1];
    stream.read_exact(&mut code_buf).await.unwrap();

    assert_eq!(code_buf[0], 0x02, "Expected FOUND response");
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf).await.unwrap();
    let val_len = u32::from_be_bytes(len_buf) as usize;

    assert_eq!(val_len, val.len());
    let mut val_buf = vec![0u8; val_len];
    stream.read_exact(&mut val_buf).await.unwrap();
    let received_val = String::from_utf8(val_buf).unwrap();
    assert_eq!(received_val, val);
}

#[tokio::test]
async fn test_get_missing_key() {
    let addr = spawn_test_server().await;
    let mut stream = TcpStream::connect(&addr).await.unwrap();
    let key = "ghost_key";
    let mut get_cmd = Vec::new();
    get_cmd.push(0x01);
    get_cmd.extend_from_slice(&(key.len() as u32).to_be_bytes());
    get_cmd.extend_from_slice(key.as_bytes());
    stream.write_all(&get_cmd).await.unwrap();
    let mut response_buf = [0u8; 1];
    stream.read_exact(&mut response_buf).await.unwrap();
    assert_eq!(response_buf[0], 0x01);
}