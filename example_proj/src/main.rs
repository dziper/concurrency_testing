use tokitest::{call};
use tokitest::{testable, label};

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    call!(my_async_func()).await;
}


#[testable]
pub async fn my_async_func() {
    println!("Thing 1");
    label!("label 1");
    
    println!("Thing 2");
    label!("label 2");
}