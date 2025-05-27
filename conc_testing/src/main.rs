mod utils;
mod controller;
use std::sync::Arc;
use tokio::{spawn, sync::RwLock};

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
async fn test_something_async() {
    // let data: Arc<RwLock<Vec<_>>> = Arc::new(RwLock::new(vec![]));
    let mc = MainController::new();
    println!("Calling nest");
    let tc1 = mc.nest("thread1").await;

    spawn(async {
        println!("Spawning thread");
        print_num(tc1).await;
    });

    // assert!(false);

    println!("CALLING RUN TO 1");
    mc.run_to("thread1", "label 1").await;
    println!("RAN TO LABEL 1");

    println!("CALLING RUN TO 2");
    mc.run_to("thread1", "label 2").await;
    println!("RAN TO LABEL 2");

    println!("CALLING RUN TO END");
    mc.run_to("thread1", "END").await;
    println!("RAN TO LABEL END");

    assert!(false);
}