use std::sync::Arc;

use base64::Engine as _;

/// The frontdoor-normalized JSON request as both a parsed value and its exact
/// decoded bytes. This is carried only inside the local process through HTTP
/// request extensions; serialized execution contracts continue to use
/// `RequestBody::body_bytes_b64`.
#[derive(Debug, Clone)]
pub struct OriginalRequestPayload {
    body_json: Arc<serde_json::Value>,
    body_bytes: Arc<[u8]>,
}

impl OriginalRequestPayload {
    pub fn from_parsed_json(body_json: serde_json::Value, body_bytes: &[u8]) -> Self {
        Self {
            body_json: Arc::new(body_json),
            body_bytes: Arc::from(body_bytes),
        }
    }

    /// Returns the original body only when the terminal provider JSON is
    /// semantically unchanged. Object key order and whitespace are preserved by
    /// returning the captured bytes rather than serializing `provider_body`.
    pub fn body_bytes_base64_if_unchanged(
        &self,
        provider_body: &serde_json::Value,
    ) -> Option<String> {
        if self.body_bytes.is_empty() || provider_body != self.body_json.as_ref() {
            return None;
        }

        Some(base64::engine::general_purpose::STANDARD.encode(self.body_bytes.as_ref()))
    }
}

#[cfg(test)]
mod tests {
    use base64::Engine as _;
    use serde_json::json;

    use super::OriginalRequestPayload;

    #[test]
    fn preserves_exact_json_bytes_when_terminal_value_is_unchanged() {
        let raw = br#"{ "unknown": true, "model": "claude-sonnet-4" }"#;
        let parsed: serde_json::Value = serde_json::from_slice(raw).expect("request should parse");
        let payload = OriginalRequestPayload::from_parsed_json(parsed.clone(), raw);

        let encoded = payload
            .body_bytes_base64_if_unchanged(&parsed)
            .expect("unchanged body should preserve bytes");

        assert_eq!(
            base64::engine::general_purpose::STANDARD
                .decode(encoded)
                .expect("body should decode"),
            raw
        );
    }

    #[test]
    fn rejects_original_bytes_when_terminal_value_changed() {
        let raw = br#"{"model":"claude-sonnet-4","messages":[]}"#;
        let parsed: serde_json::Value = serde_json::from_slice(raw).expect("request should parse");
        let payload = OriginalRequestPayload::from_parsed_json(parsed, raw);

        assert_eq!(
            payload.body_bytes_base64_if_unchanged(&json!({
                "model": "claude-sonnet-4-5",
                "messages": []
            })),
            None
        );
    }
}
