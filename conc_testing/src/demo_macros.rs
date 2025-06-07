mod controller;
use std::sync::Arc;
use tokio::{join, spawn, sync::RwLock};
use tokio::time::{sleep, Duration};

use controller::{MainController, Nestable, ThreadController};

/*
Lets try to list all of the behavior we need to support in this test,
that way we can figure out what we want to macroify
*/

// Simple
#[testable]
async fn demo(offset: i32, data: Arc<RwLock<Vec<i32>>>) {
    data.write().await.push(offset + 0);
    Label!("label 1");
    data.write().await.push(offset + 1);
    Label!("label 2");
    data.write().await.push(offset + 2);
}

// After macro expansion, note NO INIT,END. This is responsibility of Spawn! not #testable
async fn demo(tc: &Arc<ThreadController>, offset: i32, data: Arc<RwLock<Vec<i32>>>) {
    data.write().await.push(offset + 0);

    tc.label("label 1").await;
    tc.label("label 1 block").await;
    data.write().await.push(offset + 1);

    tc.label("label 2").await;
    tc.label("label 2 block").await;
    data.write().await.push(offset + 2);
}

// ----

// Sync Functions should behave like Async (I think?)
#[testable]
fn sync_fn() {
    Label!("label 1");
}
// After expansion
fn sync_fn(tc: &Arc<ThreadController>) {
    tc.label("label 1").await;
    tc.label("label 1 block").await;
}

// Calling Testable Functions, propagate tc down

#[testable]
async fn async_fn() {
    Label!("label 0");
    Call!(sync_fn());
    Label!("label 2");
}

async fn async_fn(tc: &Arc<ThreadController>) {
    tc.label("label 0").await;
    tc.label("label 0 block").await;

    sync_fn(tc);    // Automatically pass in tc

    tc.label("label 2").await;
    tc.label("label 2 block").await;
}

// ----

// Spawning

#[testable]
async fn demo_spawn(offset: i32, data: &Arc<RwLock<Vec<i32>>>) {
    Spawn!("spawned", async {
        // Arbitrary Code
    });

    Spawn!("spawned2", async {
        // Arbitrary Code
    });
}

async fn demo_spawn(tc: &Arc<ThreadController>, offset: i32, data: Arc<RwLock<Vec<i32>>>) {
    let tcNew = tc.nest("spawned").await;
    // Note each tcNew is immediately moved into spawned thread. They are different objects.
    tokio::spawn(async {
        tcNew.label("INIT").await;
        let result = {
            // Arbitrary Code
        };
        tcNew.label("END").await;
        return result;
    });

    let tcNew = tc.nest("spawned2").await;
    tokio::spawn(async {
        tcNew.label("INIT").await;
        let result = {
            // Arbitrary Code
        };
        tcNew.label("END").await;
        return result;
    });
}

// ----

// Join Set

async fn demo_joinset(tc: &Arc<ThreadController>, offset: i32, data: Arc<RwLock<Vec<i32>>>) {
    let mut set: JoinSet<i32> = JoinSet::new(); 
    for i in 0..5 {
        let tcNew = tc.nest("spawned"+str(i)).await;
        set.spawn(async {
            tcNew.label("INIT");
            let result = {
                tcNew.label("label 1").await;
                tcNew.label("label 1 block").await;
                
                // Arbitary Code

                return long_computation().await;
            };  // .await?
            tcNew.label("END");
            // Note END label should come AFTER all computation!
            return result;
        });
    }

    while let Some(join_res) = set.join_next().await {
        // ...
    }
}

#[testable]
async fn demo_joinset(tc: &Arc<ThreadController>, offset: i32, data: Arc<RwLock<Vec<i32>>>) {
    let mut set: JoinSet<i32> = JoinSet::new(); 
    for i in 0..5 {
        SpawnJoinset!("spawned"+str(i), set, async {
            Label!("label 1");
            // Arbitrary Code
            return long_computation(i).await;
            // Note END label should come AFTER all computation!
        });
    }

    while let Some(join_res) = set.join_next().await {
        // ...
    }
}


// Structs

#[derive(Testable)]
struct MyTestableObj {}

impl MyTestableObj {
    async fn thing1(&self, arg1: i32) -> i32 {}
    async fn thing2(&self, arg1: i32) -> i32 {}
}


struct MyTestableObj {}

impl MyTestableObj {
    async fn thing1(&self, tc: &Arc<ThreadController>, arg1: i32) -> i32 {}
    async fn thing2(&self, tc: &Arc<ThreadController>, arg1: i32) -> i32 {}
}


// ----

struct MyPartiallyTestableObj {}
impl MyPartiallyTestableObj {
    #[testable]
    async fn thing1(&self, arg1: i32) -> i32 {}

    async fn thing2(&self, arg1: i32) -> i32 {}
}


struct MyPartiallyTestableObj {}
impl MyPartiallyTestableObj {
    async fn thing1(&self, tc: &Arc<ThreadController>, arg1: i32) -> i32 {}
    async fn thing2(&self, arg1: i32) -> i32 {}
}

// f1 = async {
//     if tc.networkDead() {
//         return Err
//     }
//     return networkcall()
// };

async fn networkcall() -> Result<()> {

}

// async fn caller (tokitestThreadController ) {
//     f1 = NetworkCall!(networkcall());

//     f2 = networkcall();

//     //some code logic

//     select! {
//         f1.await,
//         f2.await
//     }
    
// }