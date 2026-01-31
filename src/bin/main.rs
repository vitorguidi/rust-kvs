use rust_kvs::{ByteCache, run_server};
use tokio::net::TcpListener;

#[tokio::main]
pub async fn main() -> std::io::Result<()> {
    let cache = ByteCache::new();
    let janitor_cache = cache.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(
            std::time::Duration::from_secs(60)
        );
        loop {
            interval.tick().await;
            janitor_cache.run_eviction().await;
        }
    });
    println!("Starting server.");
    let listener = TcpListener::bind("127.0.0.1:6379").await?;
    println!("Listening on 127.0.0.1:6379");
    run_server(cache, listener, 20).await?;
    Ok(())
}