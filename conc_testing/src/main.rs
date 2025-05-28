mod utils;
mod controller;
use std::sync::Arc;
use tokio::{join, spawn, sync::RwLock};
use tokio::time::{sleep, Duration};

use controller::{MainController, Nestable, ThreadController};
use utils::SharedStrings;
use tokio::sync::mpsc;
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
async fn print_num(tc: Arc<ThreadController>) {
    // LabelStart!("INIT")
    tc.label("INIT").await;
    println!("1");
    println!("2");
    // Label!("label 1")
    tc.label("label 1").await;
    tc.label("label 1 block").await;
    println!("3");
    println!("4");
    println!("5");
    // Label!("label 2")
    tc.label("label 2").await;
    tc.label("label 2 block").await; // block here
    println!("6");
    println!("7");
    println!("8");
    // LabelEnd!("END")
    tc.label("END").await;
}

async fn print_num_shared_write(tc: Arc<ThreadController>, offset: i32, data: Arc<RwLock<Vec<i32>>>) {
    // LabelStart!("INIT")
    tc.label("INIT").await;
    data.write().await.push(offset + 1);
    sleep(Duration::from_millis(10)).await;
    data.write().await.push(offset + 2);

    // Label!("label 1")
    tc.label("label 1").await;
    tc.label("label 1 block").await;
    data.write().await.push(offset + 3);
    sleep(Duration::from_millis(10)).await;
    data.write().await.push(offset + 4);
    sleep(Duration::from_millis(10)).await;
    data.write().await.push(offset + 5);

    // Label!("label 2")
    tc.label("label 2").await;
    tc.label("label 2 block").await; // block here
    data.write().await.push(offset + 6);
    sleep(Duration::from_millis(10)).await;
    data.write().await.push(offset + 7);
    sleep(Duration::from_millis(10)).await;
    data.write().await.push(offset + 8);
    // LabelEnd!("END")
    tc.label("END").await;
    data.write().await.push(offset + 9);
}


/*

To do : 
Create a thread to perform tasks 
Allow it to be "annotated" to take in channels OR controller objects? 
When spawning the thread, pass in channels OR controller?
In the thread, call label function
In the test, send signals

*/

#[tokio::main]
async fn main() {
    // Calling `print_num()` does not execute the body of `print_num()`.
    //let op = print_num(); --> this is in release mode

    // This println! comes first
    println!("hello");

    // Calling `.await` on `op` starts executing `say_world`.
    // op.await;
}

#[tokio::test]
async fn test_one_thread() {
    let data: Arc<RwLock<Vec<i32>>> = Arc::new(RwLock::new(vec![]));
    let mc = MainController::new();
    println!("Calling nest");
    let tc1 = mc.nest("thread1").await;

    let dc = data.clone();
    spawn(async {
        println!("Spawning thread");
        print_num_shared_write(tc1, 0, dc).await;
    });

    // assert!(false);
    assert_eq!(Vec::<i32>::new(), *data.read().await);

    mc.run_to("thread1", "label 1").await;
    assert_eq!(vec![1,2], *data.read().await);

    mc.run_to("thread1", "label 2").await;
    assert_eq!(vec![1,2,3,4,5], *data.read().await);

    mc.run_to("thread1", "END").await;
    assert_eq!(vec![1,2,3,4,5,6,7,8,9], *data.read().await);
}

#[tokio::test]
async fn test_two_threads() {
    let data: Arc<RwLock<Vec<i32>>> = Arc::new(RwLock::new(vec![]));
    let mc = MainController::new();
    println!("Calling nest");
    
    let tc0 = mc.nest("thread0").await;
    let tc1 = mc.nest("thread1").await;

    let dc0 = data.clone();
    let dc1 = data.clone();
    
    spawn(async {
        println!("Spawning thread 0");
        // Thread 0 puts 1-9 into data
        print_num_shared_write(tc0, 0, dc0).await;
    });

    spawn(async {
        println!("Spawning thread 1");
        // Thread 1 puts 1-9 into data
        print_num_shared_write(tc1, 10, dc1).await;
    });
    

    // assert!(false);
    assert_eq!(Vec::<i32>::new(), *data.read().await);

    mc.run_to("thread0", "label 1").await;
    assert_eq!(vec![1,2], *data.read().await);

    mc.run_to("thread1", "label 2").await;
    assert_eq!(vec![1,2,11,12,13,14,15], *data.read().await);

    mc.run_to("thread0", "label 2").await;
    assert_eq!(vec![1,2,11,12,13,14,15,3,4,5], *data.read().await);

    mc.run_to("thread0", "END").await;
    assert_eq!(vec![1,2,11,12,13,14,15,3,4,5,6,7,8,9], *data.read().await);

    mc.run_to("thread1", "END").await;
    assert_eq!(vec![1,2,11,12,13,14,15,3,4,5,6,7,8,9,16,17,18,19], *data.read().await);
}


#[tokio::test]
async fn test_two_threads_join() {
    let data: Arc<RwLock<Vec<i32>>> = Arc::new(RwLock::new(vec![]));
    let mc = MainController::new();
    println!("Calling nest");
    
    let tc0 = mc.nest("thread0").await;
    let tc2 = mc.nest("thread1").await;

    let dc0 = data.clone();
    let dc2 = data.clone();
    
    spawn(async {
        println!("Spawning thread 0");
        print_num_shared_write(tc0, 0, dc0).await;
    });

    spawn(async {
        println!("Spawning thread 1");
        print_num_shared_write(tc2, 10, dc2).await;
    });
    
    assert_eq!(Vec::<i32>::new(), *data.read().await);

    mc.run_to("thread0", "label 1").await;
    assert_eq!(vec![1,2], *data.read().await);

    join!(
       mc.run_to("thread1", "label 2"),
       mc.run_to("thread0", "END"),
    );
    // Threads run concurrently so their execution may be interleaved
    let exp = vec![1,2,11,12,13,14,15,3,4,5,6,7,8,9];
    for e in &exp {
        assert!(data.read().await.contains(e));
    }
    assert_eq!(exp.len(), data.read().await.len());


    mc.run_to("thread1", "END").await;
    let exp = vec![1,2,11,12,13,14,15,3,4,5,6,7,8,9,16,17,18,19];
    for e in &exp {
        assert!(data.read().await.contains(e));
    }
    assert_eq!(exp.len(), data.read().await.len());
}
