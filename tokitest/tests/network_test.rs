use std::sync::Arc;
use tokio::{sync::RwLock};
use tokio::time::{sleep, Duration};
use tokitest::{run_to, label, complete, spawn, call, heal, network_call, isolate};

#[tokitest::testable]
async fn make_network_request(url: &str, results: Arc<RwLock<Vec<String>>>) {
    label!("before network call");

    println!("Making network request to: {}", url);
    let response = network_call!(mock_http_get(url), mock_http_error_handler()).await;

    match response {
        Ok(data) => {
            results.write().await.push(format!("success: {}", data));
            label!("network success");
        }
        Err(err) => {
            results.write().await.push(format!("error: {}", err));
            label!("network error");
        }
    }

    println!("Network request completed");
    label!("after network call");
}

// Mock HTTP function that simulates a network request
async fn mock_http_get(url: &str) -> Result<String, String> {
    sleep(Duration::from_millis(10)).await;
    Ok(format!("data from {}", url))
}

async fn mock_http_error_handler() -> Result<String, String> {
    Err("Network is dead".to_string())
}

#[tokitest::test]
async fn test_network_call_normal() {
    let results = Arc::new(RwLock::new(Vec::<String>::new()));

    let results_clone = results.clone();
    spawn!("thread1", async {
        call!(make_network_request("http://api.example.com", results_clone)).await;
    });

    // Thread should execute normally when not isolated
    run_to!("thread1", "network success").await;

    let data = results.read().await;
    assert_eq!(data.len(), 1);
    assert!(data[0].contains("success"));
    assert!(data[0].contains("data from http://api.example.com"));

    complete!("thread1").await;
}

#[tokitest::test]
async fn test_network_call_error() {
    let results = Arc::new(RwLock::new(Vec::<String>::new()));

    let results_clone = results.clone();
    spawn!("thread1", async {
        call!(make_network_request("http://api.example.com", results_clone)).await;
    });

    // Thread should execute normally when not isolated
    isolate!("thread1").await;
    run_to!("thread1", "network error").await;

    let data = results.read().await;
    assert_eq!(data.len(), 1);
    assert!(data[0].contains("error"));
    assert!(data[0].contains("Network is dead"));

    complete!("thread1").await;
}

// test when isolate parent thread thread 1, spawn a child thread thread 2, and network call in thread 2 will fail
#[tokitest::test]
async fn test_network_call_isolated() {
    let results = Arc::new(RwLock::new(Vec::<String>::new()));

    let results_clone = results.clone();

    spawn!("thread1", async {

        spawn!("thread2", async {
            call!(make_network_request("http://api.example.com", results_clone)).await;
            label!("after-network-call");
        });

        label!("after-spawn-thread2");
    });

    // Isolate parent thread
    isolate!("thread1").await;

    run_to!("thread1", "after-spawn-thread2").await;

    run_to!("thread1.thread2", "after-network-call").await;

    let data = results.read().await;
    assert_eq!(data.len(), 1);
    assert!(data[0].contains("error"));
    assert!(data[0].contains("Network is dead"));
}

#[tokitest::test]
async fn test_network_call_healed() {
    let results = Arc::new(RwLock::new(Vec::<String>::new()));

    let results_clone = results.clone();

    spawn!("thread1", async {

        spawn!("thread2", async {
            call!(make_network_request("http://api.example.com", results_clone)).await;
            label!("after-network-call");
        });

        label!("after-spawn-thread2");
    });

    // Isolate parent thread
    isolate!("thread1").await;
    heal!("thread1").await;

    run_to!("thread1", "after-spawn-thread2").await;

    run_to!("thread1.thread2", "after-network-call").await;

    let data = results.read().await;
    assert_eq!(data.len(), 1);
    assert!(data[0].contains("success"));
    assert!(data[0].contains("data from http://api.example.com"));
}
