use tokitest::{testable, label};

#[testable]
pub async fn my_async_func() {
    println!("Thing 1");
    label!("label 1");
    
    println!("Thing 2");
    label!("label 2");
}