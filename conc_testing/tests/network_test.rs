use conc_testing::controller;
use conc_testing::label_spec;

use controller::{MainController, Nestable, ThreadController};
use label_spec::{RepeatedLabel, StringLabel};
use testable::CreateMainController;
use testable::RunTo;

use std::sync::Arc;
use tokio::{sync::RwLock};
use tokio::time::{sleep, Duration};
use testable::{testable, Label, Spawn, Call, NetworkCall, Isolate};

// Mock HTTP client result type
type HttpResult = Result<String, &'static str>;

// #[testable]
// async fn make_network_request(url: &str, results: Arc<RwLock<Vec<String>>>) {
//     Label!("before network call");
    
//     let response = NetworkCall!(mock_http_get(url)).await;
    
//     match response {
//         Ok(data) => {
//             results.write().await.push(format!("success: {}", data));
//             Label!("network success");
//         }
//         Err(err) => {
//             results.write().await.push(format!("error: {}", err));
//             Label!("network error");
//         }
//     };
    
//     Label!("after network call");
// }

// Mock HTTP function that simulates a network request
async fn mock_http_get(url: &str) -> Result<String, String> {
    sleep(Duration::from_millis(10)).await;
    Ok(format!("data from {}", url))
}

// #[tokio::test]
// async fn test_network_call_normal() {
//     let results = Arc::new(RwLock::new(Vec::<String>::new()));
    
//     CreateMainController!();
    
//     let results_clone = results.clone();
//     Spawn!("thread1", async {
//         let response = NetworkCall!(mock_http_get("hello.com")).await;

//         match response {
//             Ok(data) => {
//                 results.write().await.push(format!("success: {}", data));
//                 Label!("network success");
//             }
//             Err(err) => {
//                 results.write().await.push(format!("error: {}", err));
//                 Label!("network error");
//             }
//         };
//         // Call!(make_network_request("http://api.example.com", results_clone)).await;
//     });
    
//     // Thread should execute normally when not isolated
//     RunTo!("thread1", "network success").await;
    
//     let data = results.read().await;
//     assert_eq!(data.len(), 1);
//     assert!(data[0].contains("success"));
//     assert!(data[0].contains("data from http://api.example.com"));
    
//     RunTo!("thread1", "END").await;
// }

// #[tokio::test]
// async fn test_network_call_isolated() {
//     let results = Arc::new(RwLock::new(Vec::<String>::new()));
    
//     CreateMainController!();
    
//     let results_clone = results.clone();
//     Spawn!("thread1", async {
//         Call!(make_network_request("http://api.example.com", results_clone)).await;
//     });
    
//     // Isolate the thread before it makes the network call
//     Isolate!("thread1").await;
    
//     // Thread should get network error when isolated
//     RunTo!("thread1", "network error").await;
    
//     let data = results.read().await;
//     assert_eq!(data.len(), 1);
//     assert!(data[0].contains("error"));
//     assert!(data[0].contains("Network is dead"));
    
//     RunTo!("thread1", "END").await;
// }

// #[testable]
// async fn multiple_network_calls(base_url: &str, results: Arc<RwLock<Vec<String>>>) {
//     for i in 0..3 {
//         let url = format!("{}/endpoint{}", base_url, i);
//         Label!("before call");
        
//         let response: HttpResult = NetworkCall!(mock_http_get(&url)).await;
        
//         match response {
//             Ok(data) => results.write().await.push(format!("call {} success", i)),
//             Err(_) => results.write().await.push(format!("call {} failed", i)),
//         }
        
//         Label!("after call");
//         sleep(Duration::from_millis(5)).await;
//     }
// }

// #[tokio::test]
// async fn test_isolation_during_execution() {
//     let results = Arc::new(RwLock::new(Vec::<String>::new()));
    
//     CreateMainController!();
    
//     let results_clone = results.clone();
//     Spawn!("thread1", async {
//         Call!(multiple_network_calls("http://api.test.com", results_clone)).await;
//     });
    
//     // Let first call succeed
//     RunTo!("thread1", "after call").await;
    
//     {
//         let data = results.read().await;
//         assert_eq!(data.len(), 1);
//         assert_eq!(data[0], "call 0 success");
//     }
    
//     // Isolate thread mid-execution
//     Isolate!("thread1").await;
    
//     // Remaining calls should fail
//     RunTo!("thread1", "after call").await;
//     RunTo!("thread1", "after call").await;
    
//     {
//         let data = results.read().await;
//         assert_eq!(data.len(), 3);
//         assert_eq!(data[0], "call 0 success");
//         assert_eq!(data[1], "call 1 failed");
//         assert_eq!(data[2], "call 2 failed");
//     }
    
//     RunTo!("thread1", "END").await;
// }

// #[tokio::test]
// async fn test_multiple_threads_selective_isolation() {
//     let results1 = Arc::new(RwLock::new(Vec::<String>::new()));
//     let results2 = Arc::new(RwLock::new(Vec::<String>::new()));
    
//     CreateMainController!();
    
//     // Start two threads making network calls
//     let r1 = results1.clone();
//     Spawn!("thread1", async {
//         Call!(make_network_request("http://service1.com", r1)).await;
//     });
    
//     let r2 = results2.clone();
//     Spawn!("thread2", async {
//         Call!(make_network_request("http://service2.com", r2)).await;
//     });
    
//     // Isolate only thread1
//     Isolate!("thread1").await;
    
//     // thread1 should fail, thread2 should succeed
//     RunTo!("thread1", "network error").await;
//     RunTo!("thread2", "network success").await;
    
//     {
//         let data1 = results1.read().await;
//         let data2 = results2.read().await;
        
//         assert_eq!(data1.len(), 1);
//         assert!(data1[0].contains("Network is dead"));
        
//         assert_eq!(data2.len(), 1);
//         assert!(data2[0].contains("success"));
//         assert!(data2[0].contains("service2.com"));
//     }
    
//     RunTo!("thread1", "END").await;
//     RunTo!("thread2", "END").await;
// }

async fn fail_call() {}

#[tokio::test]
async fn test_network_call_normal() {
    let results = Arc::new(RwLock::new(Vec::<String>::new()));
    
    CreateMainController!();

    let results_clone = results.clone();
    Spawn!("thread1", async {
        // let response: Result<String, String>;
        let err_func = async || {
            Err(String::from("Network"))
        };

        let response = if tokitest_thread_controller.is_isolated().await {
           err_func()
        } else {
            mock_http_get("hello.com")
        };
        

        match response {
            Ok(data) => {
                results.write().await.push(format!("success: {}", data));
                Label!("network success");
            }
            Err(err) => {
                results.write().await.push(format!("error: {}", err));
                Label!("network error");
            }
        };
        // Call!(make_network_request("http://api.example.com", results_clone)).await;
    });
    
    // Thread should execute normally when not isolated
    RunTo!("thread1", "network success").await;
    
    let data = results_clone.read().await;
    assert_eq!(data.len(), 1);
    assert!(data[0].contains("success"));
    assert!(data[0].contains("data from hello.com"));
    
    RunTo!("thread1", "END").await;
}