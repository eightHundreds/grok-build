//! Checks that configured `extra_body` keys reach the serialized inference request.

mod support;

use std::sync::{Arc, Mutex};

use axum::Router;
use axum::routing::post;
use tokio::net::TcpListener;
use xai_grok_sampler::SamplingClient;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn request_body_contains_extra_body_keys() {
    let captured: Arc<Mutex<Option<serde_json::Value>>> = Arc::new(Mutex::new(None));
    let sink = Arc::clone(&captured);
    let app = Router::new().route(
        "/v1/chat/completions",
        post(move |body: String| {
            let sink = Arc::clone(&sink);
            async move {
                *sink.lock().unwrap() = Some(serde_json::from_str(&body).unwrap());
                "{}"
            }
        }),
    );
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });

    let base_url = format!("http://{addr}/v1");
    let mut cfg = support::test_config(&base_url, "test-key");
    cfg.extra_body
        .insert("enable_thinking".into(), serde_json::json!(true));
    cfg.extra_body.insert(
        "custom_params".into(),
        serde_json::json!({"foo": "bar", "n": 1}),
    );
    cfg.extra_body
        .insert("tags".into(), serde_json::json!(["codex", "via-new-api"]));
    // Reserved / already-present keys must not clobber the built request.
    cfg.extra_body
        .insert("model".into(), serde_json::json!("should-not-win"));
    cfg.extra_body
        .insert("messages".into(), serde_json::json!([]));
    cfg.extra_body
        .insert("stream".into(), serde_json::json!(false));

    let client = SamplingClient::new(cfg).expect("client builds");
    support::send_one(&client).await;

    let body = captured.lock().unwrap().take().expect("request captured");
    assert_eq!(body["enable_thinking"], serde_json::json!(true));
    assert_eq!(
        body["custom_params"],
        serde_json::json!({"foo": "bar", "n": 1})
    );
    assert_eq!(body["tags"], serde_json::json!(["codex", "via-new-api"]));
    assert_eq!(
        body["model"],
        serde_json::json!("test-model"),
        "extra_body must not overwrite the reserved model field"
    );
    assert!(
        body["messages"].as_array().is_some_and(|m| !m.is_empty()),
        "extra_body must not replace messages: {body}"
    );
    assert_ne!(
        body.get("stream"),
        Some(&serde_json::json!(false)),
        "extra_body must not force stream=false onto the request"
    );
}
