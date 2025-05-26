mod utils;
mod controller;
use controller::TestController;
use utils::SharedStrings;
use tokio::sync::mpsc;
// use log;

async fn echo(thing: String) -> String{
    thing
}


async fn mark_label (lab: String, rx : &mut mpsc::Receiver<bool>, tx: &mpsc::Sender<String>) {
    //wait for signal to proceed
    println!("{}", lab.clone());
    let _proceed = rx.recv().await.unwrap();
    println!("2");
    let _ = tx.send(lab).await;

}

// create macros for this
async fn print_num(mut rx : mpsc::Receiver<bool>, tx: mpsc::Sender<String>) {
    // LabelStart!("INIT")
    mark_label("INIT".to_string(), &mut rx, &tx);
    println!("1");
    println!("2");
    // Label!("label 1")
    mark_label("label 1".to_string(), &mut rx, &tx);
    mark_label("label 1 temp".to_string(), &mut rx, &tx);
    println!("3");
    println!("4");
    println!("5");
    // Label!("label 2")
    mark_label("label 2".to_string(), &mut rx, &tx);  // --> block here
    mark_label("label 2 temp".to_string(), &mut rx, &tx);
    println!("6");
    println!("7");
    println!("8");
    // LabelEnd!("END")
    mark_label("END".to_string(), &mut rx, &tx);
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

    let strs = SharedStrings::new();
    //should this be automated?
    let (mut controller, proceed_rx, label_tx) = TestController::new("thread1".to_string());

    //create a macro for this 
    tokio::spawn (
        print_num(proceed_rx, label_tx)
    );
    println!("spawned task");
    controller.run_to("label 2".to_string());
    println!("Thread 1 should be stopped at label 1");
    // controller.run_to("label 2".to_string());
    println!("End test");

    // let c1 = controller.clone("thread1");
    // tokio::spawn(|| {
    //     c1.label("A");
    //     c1.label("B");
    // });

    // let c2 = controller.clone("thread2");
    // tokio::spawn({
    //     c2.label("A");
    //     c2.label("B");
    //     for i in 1..2 {
    //         let ci = c2.clone(i);
    //         tokio::spawn({
    //             ci.label("D");
    //             ci.label("E");
    //         })
    //     }
    // });

    // controller.order(vec!["thread1.A", "thread2.A", "thread2.B", "thread1.B", "thread2.1.D", "thread2.2.D"]);
}