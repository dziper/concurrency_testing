use std::sync::{Arc, RwLock};
use std::thread;

#[derive(Clone)]
pub struct SharedStrings {
    data: Arc<RwLock<Vec<String>>>,
}

impl SharedStrings {
    pub fn new() -> Self {
        SharedStrings {
            data: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn push(&self, value: String) {
        let mut vec = self.data.write().expect("Unexpected scenario");
        vec.push(value);
    }

    pub fn get_all(&self) -> Vec<String> {
        let vec = self.data.read().expect("Unexpected scenario");
        vec.clone()
    }
}

// fn main() {
//     let shared = SharedStrings::new();
//     let mut handles = vec![];

//     for i in 0..5 {
//         let shared_clone = shared.clone();
//         let handle = thread::spawn(move || {
//             shared_clone.push(format!("From thread {}", i));
//         });
//         handles.push(handle);
//     }

//     for handle in handles {
//         handle.join().unwrap();
//     }

//     println!("Final contents: {:?}", shared.get_all());
// }
