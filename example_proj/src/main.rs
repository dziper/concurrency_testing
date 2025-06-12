use tokitest::{call};
use example_proj::funcs::my_async_func;

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    call!(my_async_func()).await;
}

