pub(crate) fn openai_request_is_image_generation_intent(
    requested_model: &str,
    body_json: &serde_json::Value,
) -> bool {
    openai_model_is_image_generation(requested_model)
        || body_json
            .get("model")
            .and_then(serde_json::Value::as_str)
            .is_some_and(openai_model_is_image_generation)
        || openai_tool_choice_selects_image_generation(body_json.get("tool_choice"))
}

fn openai_model_is_image_generation(model: &str) -> bool {
    model.trim().to_ascii_lowercase().starts_with("gpt-image-")
}

fn openai_tool_choice_selects_image_generation(choice: Option<&serde_json::Value>) -> bool {
    let Some(choice) = choice else {
        return false;
    };
    if let Some(value) = choice.as_str() {
        return value.trim().eq_ignore_ascii_case("image_generation");
    }
    let Some(object) = choice.as_object() else {
        return false;
    };
    object
        .get("type")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|value| value.trim().eq_ignore_ascii_case("image_generation"))
        || object
            .get("tool")
            .and_then(|value| value.get("type"))
            .and_then(serde_json::Value::as_str)
            .is_some_and(|value| value.trim().eq_ignore_ascii_case("image_generation"))
        || object
            .get("function")
            .and_then(|value| value.get("name"))
            .and_then(serde_json::Value::as_str)
            .is_some_and(|value| value.trim().eq_ignore_ascii_case("image_generation"))
}

#[cfg(test)]
mod tests {
    use super::openai_request_is_image_generation_intent;
    use serde_json::json;

    #[test]
    fn detects_openai_image_generation_intent_like_compat_proxies() {
        assert!(openai_request_is_image_generation_intent(
            "GPT-IMAGE-2",
            &json!({})
        ));
        assert!(openai_request_is_image_generation_intent(
            "gpt-5",
            &json!({"model":"gpt-image-2"})
        ));
        assert!(openai_request_is_image_generation_intent(
            "gpt-5",
            &json!({"tool_choice":{"function":{"name":"image_generation"}}})
        ));
        assert!(openai_request_is_image_generation_intent(
            "gpt-5",
            &json!({"tool_choice":{"type":"image_generation"}})
        ));
        assert!(!openai_request_is_image_generation_intent(
            "gpt-5",
            &json!({"tools":[{"type":"image_generation"}]})
        ));
        assert!(!openai_request_is_image_generation_intent(
            "gpt-5",
            &json!({"messages":[{"role":"user","content":"hello"}]})
        ));
    }
}
