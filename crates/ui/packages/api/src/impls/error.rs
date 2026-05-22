use app::app_error::AppError;
use dioxus::prelude::{ServerFnError, StatusCode};

pub fn api_error(err: AppError) -> ServerFnError {
    let status = match &err {
        AppError::Validation(_) | AppError::Domain(_) => StatusCode::BAD_REQUEST,
        AppError::NotFound(_) => StatusCode::NOT_FOUND,
        AppError::Upstream(_) => StatusCode::BAD_GATEWAY,
        AppError::Database(_) | AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
    };

    ServerFnError::ServerError {
        message: err.to_string(),
        code: status.as_u16(),
        details: None,
    }
}
