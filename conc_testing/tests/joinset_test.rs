use conc_testing::{controller};
use tokio::task::JoinSet;

use std::sync::Arc;
use tokio::{join, sync::RwLock};
use tokio::time::{sleep, Duration};

use controller::{MainController, Nestable, ThreadController};

use testable::{testable, Call, CreateMainController, Label, RunTo, Spawn, SpawnJoinSet};



#[tokio::test]
async fn test_one_thread() {
    CreateMainController!();

    let mut set: JoinSet<i32> = JoinSet::new(); 
    for i in 0..5 {
        // let dc = data.clone();
        SpawnJoinSet!(&format!("spawned{}", i), set, async {
            // dc.write().await.push(i);
            Label!("label 1");
            return i;
        });
    }

    join! {
        RunTo!("spawned0", "label 1"),
        RunTo!("spawned1", "label 1"),
        RunTo!("spawned2", "label 1"),
        RunTo!("spawned3", "END"),
        RunTo!("spawned4", "END"),
    };

    if let Some(Ok(join_res)) = set.join_next().await {
        assert!(join_res == 3 || join_res == 4);
    }
    if let Some(Ok(join_res)) = set.join_next().await {
        assert!(join_res == 3 || join_res == 4);
    }

    join!(
        RunTo!("spawned0", "END"),
        RunTo!("spawned1", "END"),
        RunTo!("spawned2", "END"),
    );

    while let Some(Ok(join_res)) = set.join_next().await {
        assert!(join_res == 0 || join_res == 1 || join_res == 2);
    }

}
