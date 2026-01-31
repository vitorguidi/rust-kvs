use std::time::Duration;

use tokio::time::sleep;

pub async fn do_work() {
    sleep(Duration::from_millis(200)).await;
    println!("work done");
}

#[tokio::main]
async fn main() {
    println!("starting async main");
    do_work().await;
    println!("async main finished.")
}