pub mod test_controller;
#[macro_use]
pub mod test_macros;

// src/test_controller.rs
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use tokio::sync::Notify;
use tokio::task_local;

task_local! {
    pub static CONTROLLER: TestController;
}

#[derive(Clone)]
pub struct TestController {
    order: Arc<Mutex<VecDeque<(String, usize)>>>,
    notify: Arc<Notify>,
}

impl TestController {
    pub fn new(order: Vec<(String, usize)>) -> Self {
        Self {
            order: Arc::new(Mutex::new(order.into())),
            notify: Arc::new(Notify::new()),
        }
    }

    pub async fn wait_for(&self, label: &str, id: usize) {
        loop {
            {
                let mut queue = self.order.lock().unwrap();
                if let Some(&(ref lbl, ref idx)) = queue.front() {
                    if lbl == label && *idx == id {
                        queue.pop_front();
                        break;
                    }
                }
            }
            self.notify.notified().await;
        }
        self.notify.notify_waiters();
    }
}

// src/test_macros.rs
#[macro_export]
macro_rules! test_label {
    ($label:expr, $id:expr) => {
        #[cfg(test)]
        $crate::test_controller::CONTROLLER.with(|ctrl| async {
            ctrl.wait_for($label, $id).await;
        }).await;
    };
}

#[cfg(test)]
use std::sync::atomic::{AtomicUsize, Ordering};
#[cfg(test)]
static TASK_COUNTER: AtomicUsize = AtomicUsize::new(1);

#[macro_export]
macro_rules! spawn_labeled {
    ($body:expr) => {{
        #[cfg(test)]
        {
            let id = $crate::test_macros::TASK_COUNTER.fetch_add(1, Ordering::SeqCst);
            tokio::spawn(async move {
                let thread_id = id;
                let run = $body(thread_id);
                run.await;
            })
        }
        #[cfg(not(test))]
        {
            tokio::spawn(async move {
                let run = $body(0);
                run.await;
            })
        }
    }};
}

// tests/interleaving_test.rs
use tokio_test_framework::{test_label, spawn_labeled, test_controller::{TestController, CONTROLLER}};

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_specific_interleaving() {
    let controller = TestController::new(vec![
        ("Label1".into(), 1),
        ("Label1".into(), 2),
        ("Label2".into(), 1),
        ("Label2".into(), 2),
    ]);

    CONTROLLER.scope(controller, async {
        let t1 = spawn_labeled!(|id| async move {
            test_label!("Label1", id);
            println!("T1 Step 1");
            test_label!("Label2", id);
            println!("T1 Step 2");
        });

        let t2 = spawn_labeled!(|id| async move {
            test_label!("Label1", id);
            println!("T2 Step 1");
            test_label!("Label2", id);
            println!("T2 Step 2");
        });

        let _ = tokio::join!(t1, t2);
    }).await;
}

t1 {
    INIT

    Label1 --> push to queue 
    Label2
    l3
    l4
    ..._

    l10

    END
}


test ()
{
    runto(l11)  // Acquires lock that no one cares about
    runto(l12)
}

t2 same as t1

in controller ()
{
    loop {
        read q_label from queue.front() if exists else continue
        if run_to_label matches q_label{
            break;
        } else if q_label before run_to_label {
            dequeue from queue
        }
    }
    resume thread
}

fn run_to(thread : MainController, name: String) {
    
    for l in thread.labels {
        l.notify()    // Release all threads (releases labels that were previously waiting)
    }
    thread.labels.push(name);
    thread.labels[name].wait() // Acquires named lock: t1.l5
    thread.labels.pop(name);
}

thread.current_label

fn init(thread) {
    thread.running.wait();
}

fn label(thread : ThreadController, name: String) {
    if !thread.labels.contains(name) {
        return 
    }
    thread.labels[name].wait()    // Stop until all labels notified for this thread.
}


struct ThreadController {
    init: Mutex
    running: Mutex
    namedLock: (String, Mutex)
}

// In Test
fn run_to(self, name: String) {

    { atomic
        if flipflop {
            flip.notify()
            namedLock.name = name
            flop.wait()
        } else {
            flop.notify()
            namedLock.name = name
            flip.wait()
        }

    }

    if first time: init.notify()

    running.wait()
}

// 1. proceed (lock whose responsibility is to resume a spawned thread t1) --> t1 thread 
// 2. lock ---> main thread

tokio spawn t1 {
    t1.init()
    1
    2
    3
    4
}


main thread --> unblocked 
t1 --> blocked on proceed

label (thread, name) {
    <- proceed
    temp <- name
}

run_to (thread, name) {
    do {
        proceed <-
        name <- temp   
    } while (temp != name)
}

INIT
1
2
3
4

main {
    // t1.label(INIT) -> blocked on proceed

    run to 2
    // main notify proceed
    // t1 unblocked
    // t1 temp <- INIT
    // t1 blocks on proceed

    // main name <- temp, name == TEMP, keep looping
    // proceed.notify, unblocks t1
    // t1 temp <- 1
    // t1 blocks on proceed

    // main name <- temp, name == 1, keep looping
    // proceed.notify, unblocks t1
    // t1 temp <- 2
    // t1 blocks on proceed

    // main name <- temp, name == 2, BREAK

    run to 4
}















run_to(name) {
    locked_name.wait() // wait for the lock
    loop until name == locked_name {
        proceed.notify()
    }
}

tokio spawn t1 {
    t1.init()
    1
    2
    3
    4
}

named_lock --> main_thread, 2
running --> t1
init --> t1 has init, t1 blocked on init --> unblock

// test.init()
t1.init() // -> t1.init.wait() x2, stuck in init

test.runto 2
// Main thread atomically sets name=2 namedLock.wait(), unblocked
// Main thread init.notify(), unblocks t1
// Main thread running.wait(), BLOCKED

// t1 is unblocked
// t1.label(1) -> 1 != namedLock.name -> noop
// t1.label(2) -> running.notify() unblocks main thread, namedLock.wait()

test.runto 4
// Main thread atomically sets name=4 namedLock.reset(), namedLock.wait(), unblocked
// Main thread running.wait(), BLOCKED


// INSIDE THREAD
fn init(self) {
    init.wait();
    running.wait();

    init.wait();    // Blocked here on init
}
fn label(self, name: String) {
    if name == namedLock.name {
        running.notify()
        namedLock.wait();   // Thread waits until namedLock is released when the NEXT run_to is called
    }
}


test () {
    
    Specify order of execution
    await all execution
    assert stuff


    await run to
    assert
    await run to
    assert
    await run to
    assert


    stop_all(t1, t2)   // t1 and t2 lock at start of tokio spawn

    run_to(t1.l5).await   // Release t1 until l5
    // All threads run to state I want
    // Test state


    run_to(t2.l3)   // Releasee t2 until l3
    // All threads run to state I want
    // Test state

    run_to(t1.l6)   // Release t1 until l6
    // All threads run to state I want
    // Test state

    run_all()       // Release all, don't care about order
    // All threads run to state I want
    // Test state

    threads.await
}

t1 {
    INIT
    1
    2
    label 0
    3
    label 1

}


t2 {
    INIT
    1
    2
    label 0
    3
    4
    label 1
}

test  () {
    run_join(t1 l0, t2 l0)  // Concurrent, don't care abt order, unblocks both threads til l0

    runto(t1, label 1)
    runto(t2, label 1)
}
/*

start test 

controller = Controller()

thread_controller = controller.clone("t1");

controller.run_to(all threads, INIT)
// Creates INIT label
// Wait on init

controller.run_to(thread_controller, "l5");
// Notify(t1.INIT)

Thread runs

// Create t1.l5
// Wait t1.l5



*/
