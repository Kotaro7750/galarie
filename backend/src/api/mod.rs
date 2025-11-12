use axum::{
    Json,
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use thiserror::Error;

pub mod search;
pub mod thumbnails;

/// Result alias for JSON payloads that map API errors automatically.
pub type ApiResult<T> = Result<Json<T>, ApiError>;

/// Result alias for JSON payloads that also customize the HTTP status code.
pub type ApiResponse<T> = Result<(StatusCode, Json<T>), ApiError>;

/// Machine-readable error codes mirrored in the OpenAPI schema.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    ValidationFailed,
    Unauthorized,
    Forbidden,
    MethodNotAllowed,
    ResourceNotFound,
    Conflict,
    TooManyRequests,
    InternalServerError,
    ServiceUnavailable,
}

impl ErrorCode {
    fn default_status(&self) -> StatusCode {
        match self {
            ErrorCode::ValidationFailed => StatusCode::BAD_REQUEST,
            ErrorCode::Unauthorized => StatusCode::UNAUTHORIZED,
            ErrorCode::Forbidden => StatusCode::FORBIDDEN,
            ErrorCode::MethodNotAllowed => StatusCode::METHOD_NOT_ALLOWED,
            ErrorCode::ResourceNotFound => StatusCode::NOT_FOUND,
            ErrorCode::Conflict => StatusCode::CONFLICT,
            ErrorCode::TooManyRequests => StatusCode::TOO_MANY_REQUESTS,
            ErrorCode::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCode::ServiceUnavailable => StatusCode::SERVICE_UNAVAILABLE,
        }
    }
}

/// Error envelope returned to HTTP clients.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorResponse {
    pub error: ErrorBody,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorBody {
    pub code: ErrorCode,
    pub message: String,
}

/// Canonical API error that converts into the shared JSON envelope.
#[derive(Debug, Error)]
#[error("{message}")]
pub struct ApiError {
    #[source]
    source: Option<anyhow::Error>,
    status: StatusCode,
    code: ErrorCode,
    message: String,
}

impl ApiError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self::with_status(code.default_status(), code, message)
    }

    pub fn with_status(status: StatusCode, code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            source: None,
            status,
            code,
            message: message.into(),
        }
    }

    fn with_source(
        status: StatusCode,
        code: ErrorCode,
        message: impl Into<String>,
        source: anyhow::Error,
    ) -> Self {
        Self {
            source: Some(source),
            status,
            code,
            message: message.into(),
        }
    }

    /// Build a validation/parameter error (HTTP 400).
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::ValidationFailed, message)
    }

    /// Build a 401 error.
    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Unauthorized, message)
    }

    /// Build a 403 error.
    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Forbidden, message)
    }

    /// Build a resource-not-found error.
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::ResourceNotFound, message)
    }

    /// Build a method-not-allowed error (HTTP 405).
    pub fn method_not_allowed(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::MethodNotAllowed, message)
    }

    /// Build a conflict error (HTTP 409).
    pub fn conflict(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Conflict, message)
    }

    /// Build a throttling error (HTTP 429).
    pub fn too_many_requests(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::TooManyRequests, message)
    }

    /// Build a service unavailable error (HTTP 503).
    pub fn service_unavailable(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::ServiceUnavailable, message)
    }

    /// Build an internal server error with a safe, client-visible message.
    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::InternalServerError, message)
    }

    /// Build an internal server error that logs the provided source.
    pub fn internal_with_source(err: impl Into<anyhow::Error>) -> Self {
        Self::with_source(
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCode::InternalServerError,
            "internal server error",
            err.into(),
        )
    }

    /// Expose the HTTP status code for logging/tests.
    pub fn status(&self) -> StatusCode {
        self.status
    }

    /// Expose the machine-readable code for logging/tests.
    pub fn code(&self) -> ErrorCode {
        self.code
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let ApiError {
            source,
            status,
            code,
            message,
        } = self;

        if matches!(
            status,
            StatusCode::INTERNAL_SERVER_ERROR | StatusCode::SERVICE_UNAVAILABLE
        ) {
            if let Some(err) = &source {
                tracing::error!(
                    error = %err,
                    code = ?code,
                    status = %status,
                    message = message.as_str(),
                    "api error (critical)"
                );
            } else {
                tracing::error!(
                    code = ?code,
                    status = %status,
                    message = message.as_str(),
                    "api error (critical)"
                );
            }
        } else {
            tracing::warn!(
                code = ?code,
                status = %status,
                message = message.as_str(),
                "api error"
            );
        }

        let payload = ErrorResponse {
            error: ErrorBody { code, message },
        };
        let mut response = (status, Json(payload)).into_response();
        response
            .extensions_mut()
            .insert(ErrorEnvelopeApplied::default());
        response
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(err: anyhow::Error) -> Self {
        Self::internal_with_source(err)
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct ErrorEnvelopeApplied;

/// Middleware that rewrites Axum default errors into the shared envelope.
pub async fn ensure_error_envelope(req: Request<Body>, next: Next) -> Response {
    let response = next.run(req).await;
    let status = response.status();

    if (status == StatusCode::METHOD_NOT_ALLOWED || status == StatusCode::NOT_FOUND)
        && response
            .extensions()
            .get::<ErrorEnvelopeApplied>()
            .is_none()
    {
        return match status {
            StatusCode::METHOD_NOT_ALLOWED => {
                ApiError::method_not_allowed("method not allowed").into_response()
            }
            StatusCode::NOT_FOUND => ApiError::not_found("route not found").into_response(),
            _ => unreachable!(),
        };
    }

    response
}

/// Fallback handler ensuring unknown routes return the API envelope.
pub async fn fallback_handler() -> ApiError {
    ApiError::not_found("route not found")
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::anyhow;
    use axum::http::StatusCode;
    use http_body_util::BodyExt;
    use serde_json::Value;

    #[tokio::test]
    async fn not_found_error_matches_contract() {
        let response = ApiError::not_found("media not found").into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let bytes = response
            .into_body()
            .collect()
            .await
            .expect("body bytes")
            .to_bytes();
        let json: Value = serde_json::from_slice(&bytes).expect("valid json");
        assert_eq!(json["error"]["code"], "RESOURCE_NOT_FOUND");
        assert_eq!(json["error"]["message"], "media not found");
    }

    #[tokio::test]
    async fn internal_with_source_masks_message() {
        let response = ApiError::internal_with_source(anyhow!("boom")).into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

        let bytes = response
            .into_body()
            .collect()
            .await
            .expect("body bytes")
            .to_bytes();
        let json: Value = serde_json::from_slice(&bytes).expect("valid json");
        assert_eq!(json["error"]["code"], "INTERNAL_SERVER_ERROR");
        assert_eq!(json["error"]["message"], "internal server error");
    }

    #[test]
    fn helper_builders_emit_expected_statuses() {
        assert_eq!(
            ApiError::bad_request("oops").status(),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            ApiError::unauthorized("nope").status(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(ApiError::forbidden("halt").status(), StatusCode::FORBIDDEN);
        assert_eq!(
            ApiError::not_found("missing").status(),
            StatusCode::NOT_FOUND
        );
        assert_eq!(ApiError::conflict("race").status(), StatusCode::CONFLICT);
        assert_eq!(
            ApiError::too_many_requests("slow down").status(),
            StatusCode::TOO_MANY_REQUESTS
        );
        assert_eq!(
            ApiError::service_unavailable("retry later").status(),
            StatusCode::SERVICE_UNAVAILABLE
        );
        assert_eq!(
            ApiError::internal("fault").code(),
            ErrorCode::InternalServerError
        );
    }
}
