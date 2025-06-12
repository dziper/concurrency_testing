use std::sync::Arc;
use tokio::{join, sync::RwLock};
use tokio::time::{sleep, Duration};
use tokitest::{call, label, spawn, run_to};


#[tokitest::testable]
async fn print_num_shared_write(offset: i32, data: Arc<RwLock<Vec<i32>>>) {
    data.write().await.push(offset + 1);
    sleep(Duration::from_millis(10)).await;
    data.write().await.push(offset + 2);

    label!("label 1");
    data.write().await.push(offset + 3);
    sleep(Duration::from_millis(10)).await;
    data.write().await.push(offset + 4);
    sleep(Duration::from_millis(10)).await;
    data.write().await.push(offset + 5);

    label!("label 2");
    data.write().await.push(offset + 6);
    sleep(Duration::from_millis(10)).await;
    data.write().await.push(offset + 7);
    sleep(Duration::from_millis(10)).await;
    data.write().await.push(offset + 8);
}


#[tokitest::test]
async fn test_one_thread() {
    let data: Arc<RwLock<Vec<i32>>> = Arc::new(RwLock::new(vec![]));
    let dc = data.clone();
    spawn!("thread1", async {
        call!(print_num_shared_write(0, dc)).await;
    });

    assert_eq!(Vec::<i32>::new(), *data.read().await);

    run_to!("thread1", "label 1").await;
    assert_eq!(vec![1,2], *data.read().await);

    run_to!("thread1", "label 2").await;
    assert_eq!(vec![1,2,3,4,5], *data.read().await);

    run_to!("thread1", "END").await;
    assert_eq!(vec![1,2,3,4,5,6,7,8], *data.read().await);
}


#[tokitest::test]
async fn test_two_threads() {
    let data: Arc<RwLock<Vec<i32>>> = Arc::new(RwLock::new(vec![]));

    let dc0 = data.clone();
    let dc1 = data.clone();

    spawn!("thread0", async {
        call!(print_num_shared_write(0, dc0)).await;
    });

    spawn!("thread1", async {
        call!(print_num_shared_write(10, dc1)).await;
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


#[tokitest::test]
async fn test_two_threads_join() {
    let data: Arc<RwLock<Vec<i32>>> = Arc::new(RwLock::new(vec![]));

    let dc0 = data.clone();
    let dc1 = data.clone();

    spawn!("thread0", async {
        call!(print_num_shared_write(0, dc0)).await;
    });

    spawn!("thread1", async {
        call!(print_num_shared_write(10, dc1)).await;
    });

    assert_eq!(Vec::<i32>::new(), *data.read().await);

    run_to!("thread0", "label 1").await;
    assert_eq!(vec![1,2], *data.read().await);

    join!(
       run_to!("thread1", "label 2"),
       run_to!("thread0", "END"),
    );
    // Threads run concurrently so their execution may be interleaved
    let exp = vec![1,2,11,12,13,14,15,3,4,5,6,7,8];
    for e in &exp {
        assert!(data.read().await.contains(e));
    }
    assert_eq!(exp.len(), data.read().await.len());


    run_to!("thread1", "END").await;
    let exp = vec![1,2,11,12,13,14,15,3,4,5,6,7,8,16,17,18];
    for e in &exp {
        assert!(data.read().await.contains(e));
    }
    assert_eq!(exp.len(), data.read().await.len());
}


// #[tokio::test]
// async fn test_no_labels() {
//     // This test makes sure that when we don't pass tokitest, the label!() macros are ignored.
//     let data: Arc<RwLock<Vec<i32>>> = Arc::new(RwLock::new(vec![]));
//     start_tokitest!();

//     let dc = data.clone();
//     let h = spawn!("thread1", async {
//         call!(print_num_shared_write(0, dc)).await;
//     });

//     let _ = h.await;
//     assert_eq!(vec![1,2,3,4,5,6,7,8], *data.read().await);
// }