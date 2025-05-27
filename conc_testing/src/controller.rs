use std::{collections::HashMap, sync::Arc};
use tokio::{sync::{mpsc::{Sender, Receiver, channel}, RwLock}};

pub trait Nestable {
    async fn nest(&self, id: &str) -> Arc<ThreadController>;
}

#[derive(Debug)]
struct MainControllerData {
    thread_controllers: HashMap<String, Arc<ThreadController>>,
    waiting_for: HashMap<String, Sender<Arc<ThreadController>>>,
}

impl MainControllerData {
    pub fn new() -> MainControllerData {
        MainControllerData {
            thread_controllers: HashMap::new(),
            waiting_for: HashMap::new(),
        }
    }

    pub async fn add_thread(&mut self, id: &str, tc: Arc<ThreadController>) {
        self.thread_controllers.insert(id.to_string(), tc.clone());
        match self.waiting_for.get(id) {
            Some(tx) => {
                tx.send(tc.clone()).await;
                self.waiting_for.remove(id);
            },
            None => {}
        }
    }
}

#[derive(Debug)]
pub struct MainController {
    data: Arc<RwLock<MainControllerData>>
}

impl MainController {
    pub fn new() -> MainController {
        MainController { data: Arc::new(RwLock::new(MainControllerData::new())) }
    }

    pub async fn run_to(&self, id: &str, label: &str) {
        let thread_controller = self.get_thread_controller(id).await;
        thread_controller.run_to(label).await;
    }

    async fn get_thread_controller(&self, id: &str) -> Arc<ThreadController> {
        let mut data_lock = self.data.write().await;
        match data_lock.thread_controllers.get(id) {
            Some(tc) => {
                return tc.clone();
            },
            None => {
                if data_lock.waiting_for.contains_key(id) {
                    panic!("Waiting on {} twice! (Are you calling run_to() twice on same thread?)", id);
                }
                let (waiting_tx, mut waiting_rx) = channel::<Arc<ThreadController>>(1);
                data_lock.waiting_for.insert(id.to_string(), waiting_tx.clone());
                drop(data_lock);

                return waiting_rx.recv().await.unwrap();
            }
        };
    }
}

impl Nestable for MainController {
    async fn nest(&self, id: &str) -> Arc<ThreadController> {
        let tc = Arc::new(ThreadController::new(id, self.data.clone()));
        self.data.write().await.add_thread(&id, tc.clone());
        return tc;
    }
}

#[derive(Debug)]
pub struct ThreadController {
    id: String,
    proceed_chan: (Sender<bool>, RwLock<Receiver<bool>>),
    label_chan: (Sender<String>, RwLock<Receiver<String>>),
    main_controller_data: Arc<RwLock<MainControllerData>>
}

impl Nestable for ThreadController {
    async fn nest(&self, id: &str) -> Arc<ThreadController> {
        let new_id = self.id.clone() + id;
        let tc = Arc::new(ThreadController::new(&new_id, self.main_controller_data.clone()));
        self.main_controller_data.write().await.add_thread(&new_id, tc.clone());
        return tc;
    }
}

impl ThreadController {
    
    // creates a named controller associated with a thread
    pub fn new(id: &str, mc_data: Arc<RwLock<MainControllerData>>) -> ThreadController {
        //create a channel to send "proceed signal" -- this resumes the thread operation
        let proceed = channel::<bool>(1);
        //consume the next label encountered in the thread
        let label = channel::<String>(1);

        ThreadController { 
            id: id.to_string(),
            proceed_chan: (proceed.0, RwLock::new(proceed.1)),
            label_chan: (label.0, RwLock::new(label.1)),
            main_controller_data: mc_data
        }
    }

    pub async fn run_to(&self, label: &str){
        println!("runnto {} for thread {}", label.clone(), self.id.clone());

        loop {
            let _ = self.proceed_chan.0.send(true).await;
            match self.label_chan.1.write().await.recv().await {
                Some(recv_label) => {
                    if recv_label == label {
                        break
                    }
                },
                None => {
                    println!("none");
                },
            }
        }
    }

    pub async fn label(&self, label: String) {
        let _ = self.proceed_chan.1.write().await.recv().await.unwrap();
        self.label_chan.0.send(label).await;
    }
}