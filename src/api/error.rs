use anyhow::anyhow;
use reqwest::StatusCode;

pub fn http_error(action: &str, status: StatusCode, text: &str) -> anyhow::Error {
    let trimmed = text.trim();
    let hint = status_hint(status);
    match (trimmed.is_empty(), hint) {
        (true, Some(h)) => anyhow!("{action} failed with {} — {}", status, h),
        (true, None) => anyhow!("{action} failed with {} (empty response)", status),
        (false, Some(h)) => anyhow!("{action} failed with {} — {}: {}", status, h, trimmed),
        (false, None) => anyhow!("{action} failed with {}: {}", status, trimmed),
    }
}

pub(crate) fn status_hint(status: StatusCode) -> Option<&'static str> {
    match status {
        StatusCode::NOT_FOUND => Some("not found (check the resource ID and that the endpoint exists on this Discourse version)"),
        StatusCode::FORBIDDEN => Some("forbidden (the API key's user likely lacks admin scope for this action)"),
        StatusCode::UNAUTHORIZED => Some("unauthorized (check apikey and api_username in your config)"),
        StatusCode::TOO_MANY_REQUESTS => Some("rate-limited (raise DISCOURSE_MAX_ADMIN_API_REQS_PER_MINUTE or slow the request rate)"),
        StatusCode::UNPROCESSABLE_ENTITY => Some("validation error (see details below)"),
        StatusCode::INTERNAL_SERVER_ERROR
        | StatusCode::BAD_GATEWAY
        | StatusCode::SERVICE_UNAVAILABLE
        | StatusCode::GATEWAY_TIMEOUT => Some("server error (try again; check the Discourse host is healthy)"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{http_error, status_hint};
    use reqwest::StatusCode;

    #[test]
    fn hint_maps_unauthorized_to_credentials_message() {
        let h = status_hint(StatusCode::UNAUTHORIZED).unwrap();
        assert!(h.contains("apikey"), "expected apikey hint, got {h:?}");
    }

    #[test]
    fn hint_maps_forbidden_to_scope_message() {
        let h = status_hint(StatusCode::FORBIDDEN).unwrap();
        assert!(h.contains("admin scope"), "expected admin-scope hint, got {h:?}");
    }

    #[test]
    fn hint_maps_429_to_rate_limit_message() {
        let h = status_hint(StatusCode::TOO_MANY_REQUESTS).unwrap();
        assert!(
            h.contains("DISCOURSE_MAX_ADMIN_API_REQS_PER_MINUTE"),
            "expected rate-limit hint to mention the env var, got {h:?}"
        );
    }

    #[test]
    fn hint_none_for_success_codes() {
        assert!(status_hint(StatusCode::OK).is_none());
        assert!(status_hint(StatusCode::CREATED).is_none());
    }

    #[test]
    fn hint_maps_5xx_to_server_error_message() {
        assert!(status_hint(StatusCode::INTERNAL_SERVER_ERROR).is_some());
        assert!(status_hint(StatusCode::BAD_GATEWAY).is_some());
        assert!(status_hint(StatusCode::SERVICE_UNAVAILABLE).is_some());
    }

    #[test]
    fn http_error_combines_action_status_and_hint() {
        let err = http_error("create widget", StatusCode::UNAUTHORIZED, "");
        let s = err.to_string();
        assert!(s.contains("create widget"));
        assert!(s.contains("401"));
        assert!(s.contains("apikey"));
    }

    #[test]
    fn http_error_includes_body_when_nonempty() {
        let err = http_error(
            "post stuff",
            StatusCode::UNPROCESSABLE_ENTITY,
            "{\"errors\":[\"title must be at least 15 characters\"]}",
        );
        let s = err.to_string();
        assert!(s.contains("post stuff"));
        assert!(s.contains("422"));
        assert!(s.contains("title must be at least 15 characters"));
    }

    #[test]
    fn http_error_with_unknown_status_has_no_hint_suffix() {
        let err = http_error("do thing", StatusCode::IM_A_TEAPOT, "nope");
        let s = err.to_string();
        assert!(s.contains("418"));
        assert!(s.contains("nope"));
    }
}
