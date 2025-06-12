use example_proj::funcs::nested_shared_write;
use std::sync::Arc;
use tokio::{sync::RwLock};
use tokitest::{RepeatedLabel, StringLabel};
use tokitest::{test, call, label, spawn, run_to, complete};

#[tokitest::test]
async fn test_one_thread() {
	let data = Arc::new(RwLock::new(Vec::<i32>::new()));

	let dc = data.clone();
	spawn!("thread1", async {
		call!(nested_shared_write(0, dc)).await;
	});

	assert_eq!(Vec::<i32>::new(), *data.read().await);

	run_to!("thread1", RepeatedLabel::new(StringLabel::new("loop label"), 5)).await;
	assert_eq!(vec![0,1,2,3,4], *data.read().await);

	complete!("thread1").await;
	assert_eq!(vec![0,1,2,3,4,5,6,7,8,9], *data.read().await);
}

#[tokitest::test]
async fn test_two_thread() {
	let data = Arc::new(RwLock::new(Vec::<i32>::new()));

	let dc = data.clone();
	spawn!("thread1", async {
		call!(nested_shared_write(0, dc)).await;
	});

	let dc = data.clone();
	spawn!("thread2", async {
		call!(nested_shared_write(10, dc)).await;
	});

	assert_eq!(Vec::<i32>::new(), *data.read().await);

	// tokitest_thread_controller.run_to_label("thread1", RepeatedLabel::new(StringLabel::new("loop label"), 5));
	run_to!("thread1", RepeatedLabel::new(StringLabel::new("loop label"), 5)).await;
	assert_eq!(vec![0,1,2,3,4], *data.read().await);

	run_to!("thread2", RepeatedLabel::new(StringLabel::new("loop label"), 3)).await;
	// tokitest_thread_controller.run_to_label("thread2", RepeatedLabel::new(StringLabel::new("loop label"), 3));
	assert_eq!(vec![0,1,2,3,4,10,11,12], *data.read().await);

	complete!("thread1").await;
	assert_eq!(vec![0,1,2,3,4,10,11,12,5,6,7,8,9], *data.read().await);

	run_to!("thread2", RepeatedLabel::new(StringLabel::new("loop label"), 3)).await;
	// tokitest_thread_controller.run_to_label("thread2", RepeatedLabel::new(StringLabel::new("loop label"), 3));
	assert_eq!(vec![0,1,2,3,4,10,11,12,5,6,7,8,9,13,14,15], *data.read().await);

	complete!("thread2").await;
	assert_eq!(vec![0,1,2,3,4,10,11,12,5,6,7,8,9,13,14,15,16,17,18,19], *data.read().await);
}