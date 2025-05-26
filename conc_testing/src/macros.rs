pub async fn mark_label (lab: String, rx : &mut mpsc::Receiver<bool>, tx: &mpsc::Sender<String>) {
    //wait for signal to proceed
    println!("{}", lab.clone());
    let _proceed = rx.recv().await.unwrap();
    println!("2");
    let _ = tx.send(lab).await;

}