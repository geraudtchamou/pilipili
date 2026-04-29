use actix_web::{HttpResponse, http::StatusCode};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Authentication error: {0}")]
    Auth(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Insufficient funds")]
    InsufficientFunds,
    
    #[error("Invalid PIN")]
    InvalidPIN,
    
    #[error("Transaction already processed")]
    DuplicateTransaction,
    
    #[error("External service error: {0}")]
    ExternalService(String),
    
    #[error("Internal server error: {0}")]
    Internal(String),
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    pub code: u16,
}

impl actix_web::ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::Validation(_) => StatusCode::BAD_REQUEST,
            AppError::Auth(_) => StatusCode::UNAUTHORIZED,
            AppError::InvalidPIN => StatusCode::UNAUTHORIZED,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::InsufficientFunds => StatusCode::BAD_REQUEST,
            AppError::DuplicateTransaction => StatusCode::CONFLICT,
            AppError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::ExternalService(_) => StatusCode::BAD_GATEWAY,
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status = self.status_code();
        HttpResponse::build(status).json(ErrorResponse {
            error: format!("{:?}", self),
            message: self.to_string(),
            code: status.as_u16(),
        })
    }
}

pub type AppResult<T> = Result<T, AppError>;
