use tokio::join;
use tokio::task::JoinSet;
use tokitest::prelude::*;
use tokitest::{label, run_to, spawn_join_set};

#[tokitest::test]
async fn test_one_thread() {
    let mut set: JoinSet<i32> = JoinSet::new();
    for i in 0..5 {
        // let dc = data.clone();
        spawn_join_set!(&format!("spawned{}", i), set, async {
            // dc.write().await.push(i);
            label!("label 1");
            return i;
        });
    }

    join! {
        run_to!("spawned0", "label 1"),
        run_to!("spawned1", "label 1"),
        run_to!("spawned2", "label 1"),
        run_to!("spawned3", "END"),
        run_to!("spawned4", "END"),
    };

    if let Some(Ok(join_res)) = set.join_next().await {
        assert!(join_res == 3 || join_res == 4);
    }
    if let Some(Ok(join_res)) = set.join_next().await {
        assert!(join_res == 3 || join_res == 4);
    }

    join!(
        run_to!("spawned0", "END"),
        run_to!("spawned1", "END"),
        run_to!("spawned2", "END"),
    );

    while let Some(Ok(join_res)) = set.join_next().await {
        assert!(join_res == 0 || join_res == 1 || join_res == 2);
    }

}
