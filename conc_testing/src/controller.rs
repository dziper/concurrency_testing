use tokio::sync::mpsc;

#[derive(Debug)]
pub struct TestController {
    id: String,
    proceed_tx: mpsc::Sender<bool>,
    label_rx: mpsc::Receiver<String>,
}

impl TestController {
    // //creates a named controller associated with a thread
    // pub fn new (id: String) -> (Self, mpsc::Receiver<bool>, mpsc::Sender<String>) {
    //     //create a channel to send "proceed signal" -- this resumes the thread operation
    //     let (proceed_tx, proceed_rx) : (mpsc::Sender<bool>, mpsc::Receiver<bool>) = mpsc::channel(1);
    //     //create a channel to receive the next label encountered in the thread
    //     let (label_tx, label_rx) : (mpsc::Sender<String>, mpsc::Receiver<String>)= mpsc::channel(1);
    //     return (TestController {
    //         id: id,
    //         proceed_tx: proceed_tx,
    //         label_rx: label_rx,
    //     }, proceed_rx, label_tx)
    // }

    pub fn new() -> Self {
        // TODO: initialize the internal map from thread_id to channels
        return TestController {}
    }

    pub fn add_thread(thread_id: String) -> (mpsc::Receiver<bool>, mpsc::Sender<String>) {
        //create a channel to send "proceed signal" -- this resumes the thread operation
        let (proceed_tx, proceed_rx) : (mpsc::Sender<bool>, mpsc::Receiver<bool>) = mpsc::channel(1);
        //create a channel to receive the next label encountered in the thread
        let (label_tx, label_rx) : (mpsc::Sender<String>, mpsc::Receiver<String>)= mpsc::channel(1);

        // add the channels to the internal map
        return proceed_rx, label_tx
    }

    pub async fn run_to(&mut self, thread_id: String, label: String){
        println!("runnto {} for thread {}", label.clone(), thread_id.clone());

        loop {

            // find the channels from the map, do the same thing
            let _ = self.proceed_tx.send(true).await;
            match self.label_rx.recv().await {
                Some(recv_label) => {
                    if recv_label == label{
                        break
                    }
                },
                None => {println!("none")},
            }
        }
    }
}