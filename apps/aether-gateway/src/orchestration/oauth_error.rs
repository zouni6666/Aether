pub(crate) fn oauth_status_may_be_invalid(status_code: u16, response_text: Option<&str>) -> bool {
    if status_code == 401 {
        return true;
    }
    if status_code != 403 {
        return false;
    }

    let Some(response_text) = response_text else {
        return false;
    };
    if let Ok(body) = serde_json::from_str::<serde_json::Value>(response_text) {
        let error_type = body
            .get("error")
            .and_then(|error| error.get("type"))
            .and_then(serde_json::Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .or_else(|| {
                body.get("type")
                    .and_then(serde_json::Value::as_str)
                    .map(str::trim)
                    .filter(|value| !value.is_empty() && !value.eq_ignore_ascii_case("error"))
            });
        if let Some(error_type) = error_type {
            return is_oauth_invalid_error_taxonomy(error_type);
        }

        let error_code = body
            .get("error")
            .and_then(|error| error.get("code"))
            .or_else(|| body.get("code"))
            .or_else(|| body.get("error").filter(|error| error.is_string()))
            .and_then(serde_json::Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty());
        if error_code.is_some_and(is_oauth_invalid_error_taxonomy) {
            return true;
        }

        return response_has_oauth_invalid_phrase(response_text);
    }

    response_has_oauth_invalid_phrase(response_text)
}

pub(crate) fn oauth_status_proves_access_token_invalid(
    status_code: u16,
    response_text: Option<&str>,
) -> bool {
    if status_code == 401 {
        return true;
    }
    if status_code != 403 {
        return false;
    }

    response_text.is_some_and(response_has_oauth_invalid_phrase)
}

fn is_oauth_invalid_error_taxonomy(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "authentication_error"
            | "invalid_authentication_token"
            | "invalid_token"
            | "oauth_token_invalid"
            | "token_invalid"
            | "token_expired"
            | "unauthenticated"
            | "biscuit_baker_service_auth_credential_error_status"
    )
}

fn response_has_oauth_invalid_phrase(response_text: &str) -> bool {
    let response_text = response_text.to_ascii_lowercase();
    if [
        "oauth_token_invalid",
        "invalid_token",
        "biscuit_baker_service_auth_credential_error_status",
    ]
    .iter()
    .any(|taxonomy| contains_ascii_taxonomy_token(&response_text, taxonomy))
    {
        return true;
    }

    [
        "oauth token is invalid",
        "oauth token is expired",
        "oauth token has expired",
        "invalid access token",
        "access token invalid",
        "access token expired",
        "expired access token",
        "authentication token has been invalidated",
        "token has been invalidated",
        "personal access token owner is inactive",
        "security token included in the request is expired",
    ]
    .iter()
    .any(|needle| response_text.contains(needle))
}

fn contains_ascii_taxonomy_token(text: &str, taxonomy: &str) -> bool {
    text.match_indices(taxonomy).any(|(start, matched)| {
        let end = start + matched.len();
        let is_identifier_byte = |byte: u8| byte.is_ascii_alphanumeric() || byte == b'_';
        let has_left_boundary = start == 0 || !is_identifier_byte(text.as_bytes()[start - 1]);
        let has_right_boundary = end == text.len() || !is_identifier_byte(text.as_bytes()[end]);
        has_left_boundary && has_right_boundary
    })
}
