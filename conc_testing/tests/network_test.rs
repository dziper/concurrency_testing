use conc_testing::controller;
use conc_testing::label_spec;

use controller::{MainController, Nestable, ThreadController};
use testable::CreateMainController;
use testable::RunTo;

use std::sync::Arc;
use tokio::{sync::RwLock};
use tokio::time::{sleep, Duration};
use testable::{testable, Label, Spawn, Call, NetworkCall};

// Mock HTTP client result type
type HttpResult = Result<String, String>;

#[testable]
async fn make_network_request(url: &str, results: Arc<RwLock<Vec<String>>>) {
    Label!("before network call");

    let response = NetworkCall!(mock_http_get(url), mock_http_error_handler()).await;

    match response {
        Ok(data) => {
            results.write().await.push(format!("success: {}", data));
            Label!("network success");
        }
        Err(err) => {
            results.write().await.push(format!("error: {}", err));
            Label!("network error");
        }
    }

    Label!("after network call");
}

// Mock HTTP function that simulates a network request
async fn mock_http_get(url: &str) -> Result<String, String> {
    sleep(Duration::from_millis(10)).await;
    Ok(format!("data from {}", url))
}

async fn mock_http_error_handler() -> Result<String, String> {
    Err("Network is dead".to_string())
}

#[tokio::test]
async fn test_network_call_normal() {
    let results = Arc::new(RwLock::new(Vec::<String>::new()));

    CreateMainController!();

    let results_clone = results.clone();
    Spawn!("thread1", async {
        Call!(make_network_request("http://api.example.com", results_clone)).await;
    });

    // Thread should execute normally when not isolated
    RunTo!("thread1", "network success").await;

    let data = results.read().await;
    assert_eq!(data.len(), 1);
    assert!(data[0].contains("success"));
    assert!(data[0].contains("data from http://api.example.com"));

    RunTo!("thread1", "END").await;
}