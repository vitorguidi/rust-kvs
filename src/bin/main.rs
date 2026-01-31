use rust_kvs::bytestore::ByteCache;
use rust_kvs::server::run_server;

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
    run_server(cache).await?;
    Ok(())
}