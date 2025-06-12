use tokitest::{call};
use example_proj::thing::thingy;

#[cfg(feature = "tokitest")]
fn main() {}

#[cfg(not(feature = "tokitest"))]
#[tokio::main]
async fn main() {
    println!("Hello, world!");

    // call!(my_async_func()).await;

    thingy().await;
}

