use tokitest::{testable, label};
use std::sync::Arc;
use tokio::{sync::RwLock};
use tokio::time::{sleep, Duration};

#[testable]
pub async fn my_async_func() {
    println!("Thing 1");
    label!("label 1");
    
    println!("Thing 2");
    label!("label 2");
}

#[testable]
pub async fn nested_shared_write(offset: i32, data: Arc<RwLock<Vec<i32>>>) {
	for i in 0..10 {
		data.write().await.push(offset + i);
		label!("loop label");
		sleep(Duration::from_millis(10)).await;
	}
}

