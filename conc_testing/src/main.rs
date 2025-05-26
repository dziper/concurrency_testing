mod utils;
mod controller;
use controller::TestController;
use utils::SharedStrings;

async fn echo(thing: String) -> String{
    thing
}

async fn say_world() {
    println!("world");
}

#[tokio::main]
async fn main() {
    // Calling `say_world()` does not execute the body of `say_world()`.
    let op = say_world();

    // This println! comes first
    println!("hello");

    // Calling `.await` on `op` starts executing `say_world`.
    op.await;
}

#[tokio::test]
async fn test_something_async() {
    assert!(true);

    let strs = SharedStrings::new();
    let controller = TestController::new();

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