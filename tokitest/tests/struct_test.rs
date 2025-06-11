use std::sync::Arc;
use tokio::{sync::RwLock};
use tokio::time::{sleep, Duration};
use tokitest::prelude::*;
use tokitest::{label, spawn, call, run_to};

pub struct Worker{
    data: Arc<RwLock<Vec<i32>>>,
}

#[tokitest::testable_struct]
impl Worker {
    pub fn new(data: Arc<RwLock<Vec<i32>>>) -> Self {
        Worker {
            data: data,
        }
    }

    pub async fn print_num_shared_write_1(&self, offset: i32) {
        self.data.write().await.push(offset + 1);
        sleep(Duration::from_millis(10)).await;
        self.data.write().await.push(offset + 2);

        label!("label 1");

        self.data.write().await.push(offset + 3);
        sleep(Duration::from_millis(10)).await;
        self.data.write().await.push(offset + 4);
        sleep(Duration::from_millis(10)).await;
        self.data.write().await.push(offset + 5);

        label!("label 2");

        self.data.write().await.push(offset + 6);
        sleep(Duration::from_millis(10)).await;
        self.data.write().await.push(offset + 7);
        sleep(Duration::from_millis(10)).await;
        self.data.write().await.push(offset + 8);
    }

    pub async fn print_num_shared_write_2(&self, offset: i32) {
        self.data.write().await.push(offset + 10);
        sleep(Duration::from_millis(5)).await;
        self.data.write().await.push(offset + 11);

        label!("step A");
        self.data.write().await.push(offset + 12);
        sleep(Duration::from_millis(5)).await;
        self.data.write().await.push(offset + 13);

        label!("step B");
        self.data.write().await.push(offset + 14);
        sleep(Duration::from_millis(5)).await;
        self.data.write().await.push(offset + 15);
    }
}


#[tokitest::test]
async fn test_two_threads_struct() {
    let data: Arc<RwLock<Vec<i32>>> = Arc::new(RwLock::new(vec![]));
    println!("Calling nest");

    let obj = Arc::new(call!(Worker::new(data.clone())));

    let obj1 = obj.clone();
    spawn!("thread0", async {
        call!(obj1.print_num_shared_write_1(0)).await;
    });

    let obj2 = obj.clone();
    spawn!("thread1", async {
        call!(obj2.print_num_shared_write_1(10)).await;
    });

    // assert!(false);
    assert_eq!(Vec::<i32>::new(), *data.read().await);

    run_to!("thread0", "label 1").await;
    assert_eq!(vec![1,2], *data.read().await);

    run_to!("thread1", "label 2").await;
    assert_eq!(vec![1,2,11,12,13,14,15], *data.read().await);

    run_to!("thread0", "label 2").await;
    assert_eq!(vec![1,2,11,12,13,14,15,3,4,5], *data.read().await);

    run_to!("thread0", "END").await;
    assert_eq!(vec![1,2,11,12,13,14,15,3,4,5,6,7,8], *data.read().await);

    run_to!("thread1", "END").await;
    assert_eq!(vec![1,2,11,12,13,14,15,3,4,5,6,7,8,16,17,18], *data.read().await);
}