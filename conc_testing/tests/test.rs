use conc_testing::{controller, labelSpec, utils};

use std::sync::Arc;
use tokio::{join, spawn, sync::RwLock};
use tokio::time::{sleep, Duration};

use controller::{MainController, Nestable, ThreadController};
use utils::SharedStrings;
use tokio::sync::mpsc;

use testable::{testable, Label, Call, Spawn, NetworkCall};
// use log;

async fn echo(thing: String) -> String{
    thing
}


// async fn mark_label (lab: String, rx : &mut mpsc::Receiver<bool>, tx: &mpsc::Sender<String>) {
//     //wait for signal to proceed
//     println!("{}", lab.clone());
//     let _proceed = rx.recv().await.unwrap();
//     println!("2");
//     let _ = tx.send(lab).await;

// }

// create macros for this

#[testable]
async fn print_num_shared_write(offset: i32, data: Arc<RwLock<Vec<i32>>>) {
    data.write().await.push(offset + 1);
    sleep(Duration::from_millis(10)).await;
    data.write().await.push(offset + 2);

    Label!("label 1");
    data.write().await.push(offset + 3);
    sleep(Duration::from_millis(10)).await;
    data.write().await.push(offset + 4);
    sleep(Duration::from_millis(10)).await;
    data.write().await.push(offset + 5);

    Label!("label 2");
    data.write().await.push(offset + 6);
    sleep(Duration::from_millis(10)).await;
    data.write().await.push(offset + 7);
    sleep(Duration::from_millis(10)).await;
    data.write().await.push(offset + 8);
}


/*

To do :
Create a thread to perform tasks
Allow it to be "annotated" to take in channels OR controller objects?
When spawning the thread, pass in channels OR controller?
In the thread, call label function
In the test, send signals

*/


#[tokio::test]
async fn test_one_thread() {
    let data: Arc<RwLock<Vec<i32>>> = Arc::new(RwLock::new(vec![]));
    let tokitestThreadController = MainController::new();
    println!("Calling nest");
    
    let dc = data.clone();
    Spawn!("thread1", async {
        print_num_shared_write(tokitestThreadController, 0, dc).await;
    });

    // let tokitestThreadController = mc.nest("thread1").await;

    // spawn(async move {  // Actually seems like we need async move
    //     tokitestThreadController.label("INIT").await;
    //     println!("Spawning thread");
    //     print_num_shared_write(&tokitestThreadController, 0, dc).await;
    //     tokitestThreadController.label("END").await;
    // });

    // assert!(false);
    assert_eq!(Vec::<i32>::new(), *data.read().await);

    tokitestThreadController.run_to("thread1", "label 1").await;
    assert_eq!(vec![1,2], *data.read().await);

    tokitestThreadController.run_to("thread1", "label 2").await;
    assert_eq!(vec![1,2,3,4,5], *data.read().await);

    tokitestThreadController.run_to("thread1", "END").await;
    assert_eq!(vec![1,2,3,4,5,6,7,8], *data.read().await);
}

// #[tokio::test]
// async fn test_two_threads() {
//     let data: Arc<RwLock<Vec<i32>>> = Arc::new(RwLock::new(vec![]));
//     let mc = MainController::new();
//     println!("Calling nest");

//     let dc0 = data.clone();
//     let dc1 = data.clone();

//     let tokitestThreadController  = mc.nest("thread0").await;
//     spawn(async move {
//         tokitestThreadController.label("INIT").await;
//         println!("Spawning thread 0");
//         // Thread 0 puts 1-9 into data
//         Call!(print_num_shared_write(0, dc0)).await;
//         tokitestThreadController.label("END").await;
//     });

//     let tokitestThreadController  = mc.nest("thread1").await;
//     spawn(async move {
//         tokitestThreadController.label("INIT").await;
//         println!("Spawning thread 1");
//         // Thread 1 puts 1-9 into data
//         print_num_shared_write(tokitestThreadController,10, dc1).await;
//         tokitestThreadController.label("END").await;
//     });


//     // assert!(false);
//     assert_eq!(Vec::<i32>::new(), *data.read().await);

//     mc.run_to("thread0", "label 1").await;
//     assert_eq!(vec![1,2], *data.read().await);

//     mc.run_to("thread1", "label 2").await;
//     assert_eq!(vec![1,2,11,12,13,14,15], *data.read().await);

//     mc.run_to("thread0", "label 2").await;
//     assert_eq!(vec![1,2,11,12,13,14,15,3,4,5], *data.read().await);

//     mc.run_to("thread0", "END").await;
//     assert_eq!(vec![1,2,11,12,13,14,15,3,4,5,6,7,8], *data.read().await);

//     mc.run_to("thread1", "END").await;
//     assert_eq!(vec![1,2,11,12,13,14,15,3,4,5,6,7,8,16,17,18], *data.read().await);
// }


// #[tokio::test]
// async fn test_two_threads_join() {
//     let data: Arc<RwLock<Vec<i32>>> = Arc::new(RwLock::new(vec![]));
//     let mc = MainController::new();
//     println!("Calling nest");

//     let dc0 = data.clone();
//     let dc1 = data.clone();

//     let tokitestThreadController  = mc.nest("thread0").await;
//     spawn(async move {
//         tokitestThreadController.label("INIT").await;
//         println!("Spawning thread 0");
//         // Thread 0 puts 1-9 into data
//         print_num_shared_write(tokitestThreadController, 0, dc0).await;
//         tokitestThreadController.label("END").await;
//     });

//     let tokitestThreadController  = mc.nest("thread1").await;
//     spawn(async move {
//         tokitestThreadController.label("INIT").await;
//         println!("Spawning thread 1");
//         // Thread 1 puts 1-9 into data
//         print_num_shared_write(tokitestThreadController, 10, dc1).await;
//         tokitestThreadController.label("END").await;
//     });

//     assert_eq!(Vec::<i32>::new(), *data.read().await);

//     mc.run_to("thread0", "label 1").await;
//     assert_eq!(vec![1,2], *data.read().await);

//     join!(
//        mc.run_to("thread1", "label 2"),
//        mc.run_to("thread0", "END"),
//     );
//     // Threads run concurrently so their execution may be interleaved
//     let exp = vec![1,2,11,12,13,14,15,3,4,5,6,7,8];
//     for e in &exp {
//         assert!(data.read().await.contains(e));
//     }
//     assert_eq!(exp.len(), data.read().await.len());


//     mc.run_to("thread1", "END").await;
//     let exp = vec![1,2,11,12,13,14,15,3,4,5,6,7,8,16,17,18];
//     for e in &exp {
//         assert!(data.read().await.contains(e));
//     }
//     assert_eq!(exp.len(), data.read().await.len());
// }
