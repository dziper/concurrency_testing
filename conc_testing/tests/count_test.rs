use conc_testing::controller;
use conc_testing::label_spec;

use controller::{MainController, Nestable, ThreadController};
use label_spec::{RepeatedLabel, StringLabel};
use testable::CreateMainController;
use testable::RunTo;

use std::sync::Arc;
use tokio::{sync::RwLock};
use tokio::time::{sleep, Duration};
use testable::{testable, Label, Spawn, Call};

#[testable]
async fn nested_shared_write(offset: i32, data: Arc<RwLock<Vec<i32>>>) {
	for i in 0..10 {
		data.write().await.push(offset + i);
		Label!("loop label");
		sleep(Duration::from_millis(10)).await;
	}
}

#[tokio::test]
async fn test_one_thread() {
	let data = Arc::new(RwLock::new(Vec::<i32>::new()));

	CreateMainController!();

	let dc = data.clone();
	Spawn!("thread1", async {
		Call!(nested_shared_write(0, dc)).await;
	});

	assert_eq!(Vec::<i32>::new(), *data.read().await);

	RunTo!("thread1", RepeatedLabel::new(StringLabel::new("loop label"), 5)).await;
	assert_eq!(vec![0,1,2,3,4], *data.read().await);

	RunTo!("thread1", "END").await;
	assert_eq!(vec![0,1,2,3,4,5,6,7,8,9], *data.read().await);
}

#[tokio::test]
async fn test_two_thread() {
	let data = Arc::new(RwLock::new(Vec::<i32>::new()));

	CreateMainController!();

	let dc = data.clone();
	Spawn!("thread1", async {
		Call!(nested_shared_write(0, dc)).await;
	});
	
	let dc = data.clone();
	Spawn!("thread2", async {
		Call!(nested_shared_write(10, dc)).await;
	});

	assert_eq!(Vec::<i32>::new(), *data.read().await);

	// tokitest_thread_controller.run_to_label("thread1", RepeatedLabel::new(StringLabel::new("loop label"), 5));
	RunTo!("thread1", RepeatedLabel::new(StringLabel::new("loop label"), 5)).await;
	assert_eq!(vec![0,1,2,3,4], *data.read().await);
	
	RunTo!("thread2", RepeatedLabel::new(StringLabel::new("loop label"), 3)).await;
	// tokitest_thread_controller.run_to_label("thread2", RepeatedLabel::new(StringLabel::new("loop label"), 3));
	assert_eq!(vec![0,1,2,3,4,10,11,12], *data.read().await);

	RunTo!("thread1", "END").await;
	assert_eq!(vec![0,1,2,3,4,10,11,12,5,6,7,8,9], *data.read().await);

	RunTo!("thread2", RepeatedLabel::new(StringLabel::new("loop label"), 3)).await;
	// tokitest_thread_controller.run_to_label("thread2", RepeatedLabel::new(StringLabel::new("loop label"), 3));
	assert_eq!(vec![0,1,2,3,4,10,11,12,5,6,7,8,9,13,14,15], *data.read().await);	
	
	RunTo!("thread2", "END").await;
	assert_eq!(vec![0,1,2,3,4,10,11,12,5,6,7,8,9,13,14,15,16,17,18,19], *data.read().await);	
}