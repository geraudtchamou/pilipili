use actix_web::{dev::ServiceRequest, Error};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use jsonwebtoken::{decode, Validation, DecodingKey};
use serde::Deserialize;
use crate::models::User;
use crate::utils::AppError;

#[derive(Debug, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

pub async fn validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "secret".to_string());
    
    match decode::<Claims>(
        credentials.token(),
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    ) {
        Ok(token_data) => {
            // Get user ID from token
            let user_id = match uuid::Uuid::parse_str(&token_data.claims.sub) {
                Ok(id) => id,
                Err(_) => return Err((AppError::Auth("Invalid user ID in token".to_string()).into(), req)),
            };

            // Fetch user from database and attach to request
            let pool = req.app_data::<actix_web::web::Data<sqlx::PgPool>>();
            
            if let Some(pool) = pool {
                match sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1 AND is_active = true")
                    .bind(user_id)
                    .fetch_optional(pool.get_ref())
                    .await
                {
                    Ok(Some(user)) => {
                        req.extensions_mut().insert(user);
                        Ok(req)
                    }
                    Ok(None) => Err((AppError::Auth("User not found".to_string()).into(), req)),
                    Err(e) => Err((AppError::Database(e).into(), req)),
                }
            } else {
                Err((AppError::Internal("Database pool not available".to_string()).into(), req))
            }
        }
        Err(_) => Err((AppError::Auth("Invalid token".to_string()).into(), req)),
    }
}
