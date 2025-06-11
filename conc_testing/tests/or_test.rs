use conc_testing::controller;
use conc_testing::label_spec;

use controller::{Nestable, ThreadController};
use label_spec::{RepeatedLabel, StringLabel, OrLabel};
use testable::start_tokitest;
use testable::run_to;

use std::sync::Arc;
use tokio::{sync::RwLock};
use tokio::time::{sleep, Duration};
use testable::{testable, label, spawn, call};

#[testable]
async fn process_with_labels(data: Arc<RwLock<Vec<String>>>) {
    for i in 0..10 {
        if i % 2 == 0 {
            data.write().await.push(format!("even_{}", i));
            label!("even_number");
        } else {
            data.write().await.push(format!("odd_{}", i));
            label!("odd_number");
        }
        sleep(Duration::from_millis(10)).await;
    }
}

#[tokio::test]
async fn test_or_label() {
    let data = Arc::new(RwLock::new(Vec::<String>::new()));

    start_tokitest!();

    let dc = data.clone();
    spawn!("thread1", async {
        call!(process_with_labels(dc)).await;
    });

    assert_eq!(Vec::<String>::new(), *data.read().await);

    // Run to either "even_number" or "odd_number" label 5 times total
    let or_label = OrLabel::new(vec![
        StringLabel::new("even_number"),
        StringLabel::new("odd_number"),
    ]);
    
    run_to!("thread1", RepeatedLabel::new(or_label, 5)).await;
    
    // Should have processed 5 items (indices 0-4)
    assert_eq!(
        vec!["even_0", "odd_1", "even_2", "odd_3", "even_4"], 
        *data.read().await
    );

    run_to!("thread1", "END").await;
    
    // Should have all 10 items now
    assert_eq!(
        vec![
            "even_0", "odd_1", "even_2", "odd_3", "even_4",
            "odd_5", "even_6", "odd_7", "even_8", "odd_9"
        ], 
        *data.read().await
    );
}