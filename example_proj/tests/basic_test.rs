use example_proj::main::my_async_func;
use tokitest::{test, call, label, spawn, run_to};

#[tokitest::test]
async fn test_something() {
    spawn!("thread 1", async {
        call!(my_async_func()).await;
    });

    run_to!("thread 1", "label 1").await;
}