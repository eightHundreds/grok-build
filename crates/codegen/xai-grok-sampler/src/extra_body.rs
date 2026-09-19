//! Merge configured `extra_body` keys into a serialized inference request.
//!
//! Extra keys are inserted at the top level only. Reserved protocol fields and
//! keys already present on the request are skipped (never overwrite the body).

use indexmap::IndexMap;
use serde_json::Value;

/// Request fields that identify the call or control streaming / tools.
/// `extra_body` must not set these even when the serialized body omitted them.
const RESERVED_BODY_KEYS: &[&str] = &[
    "model",
    "messages",
    "input",
    "tools",
    "stream",
    "stream_options",
];

pub(crate) fn is_reserved_body_key(key: &str) -> bool {
    RESERVED_BODY_KEYS
        .iter()
        .any(|reserved| key.eq_ignore_ascii_case(reserved))
}

/// Fold `extra` into `body` without replacing the object or clobbering core fields.
pub(crate) fn apply_extra_body(body: &mut Value, extra: &IndexMap<String, Value>) {
    if extra.is_empty() {
        return;
    }
    let Some(obj) = body.as_object_mut() else {
        tracing::warn!("extra_body ignored: request body is not a JSON object");
        return;
    };
    for (key, value) in extra {
        if is_reserved_body_key(key) {
            tracing::warn!(key, "extra_body key skipped: reserved request field");
            continue;
        }
        if obj.contains_key(key) {
            tracing::warn!(
                key,
                "extra_body key skipped: would overwrite an existing request field"
            );
            continue;
        }
        obj.insert(key.clone(), value.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn extra(pairs: &[(&str, Value)]) -> IndexMap<String, Value> {
        pairs
            .iter()
            .map(|(k, v)| ((*k).to_owned(), v.clone()))
            .collect()
    }

    #[test]
    fn inserts_new_top_level_keys_including_nested_values() {
        let mut body = json!({
            "model": "m",
            "messages": []
        });
        apply_extra_body(
            &mut body,
            &extra(&[
                ("enable_thinking", json!(true)),
                ("custom_params", json!({"foo": "bar", "n": 1})),
                ("tags", json!(["codex", "via-new-api"])),
            ]),
        );
        assert_eq!(body["enable_thinking"], json!(true));
        assert_eq!(body["custom_params"], json!({"foo": "bar", "n": 1}));
        assert_eq!(body["tags"], json!(["codex", "via-new-api"]));
        assert_eq!(body["model"], json!("m"));
        assert_eq!(body["messages"], json!([]));
    }

    #[test]
    fn skips_reserved_keys_even_when_absent() {
        let mut body = json!({"temperature": 0.2});
        apply_extra_body(
            &mut body,
            &extra(&[
                ("model", json!("attacker")),
                ("messages", json!([{"role": "user", "content": "x"}])),
                ("input", json!("x")),
                ("tools", json!([])),
                ("stream", json!(false)),
                ("stream_options", json!({"include_usage": false})),
                ("ok", json!(1)),
            ]),
        );
        assert_eq!(
            body,
            json!({
                "temperature": 0.2,
                "ok": 1
            })
        );
    }

    #[test]
    fn skips_reserved_keys_case_insensitively() {
        let mut body = json!({});
        apply_extra_body(&mut body, &extra(&[("Model", json!("x"))]));
        assert_eq!(body, json!({}));
    }

    #[test]
    fn skips_keys_already_present() {
        let mut body = json!({
            "temperature": 0.7,
            "user": "built-in"
        });
        apply_extra_body(
            &mut body,
            &extra(&[
                ("temperature", json!(0.1)),
                ("user", json!("from-extra")),
                ("keep_me", json!("yes")),
            ]),
        );
        assert_eq!(body["temperature"], json!(0.7));
        assert_eq!(body["user"], json!("built-in"));
        assert_eq!(body["keep_me"], json!("yes"));
    }

    #[test]
    fn empty_extra_is_a_noop() {
        let mut body = json!({"model": "m"});
        apply_extra_body(&mut body, &IndexMap::new());
        assert_eq!(body, json!({"model": "m"}));
    }

    #[test]
    fn non_object_body_is_left_alone() {
        let mut body = json!([1, 2, 3]);
        apply_extra_body(&mut body, &extra(&[("foo", json!("bar"))]));
        assert_eq!(body, json!([1, 2, 3]));
    }
}
