use actix_web::web;

mod auth_handler;
mod wallet_handler;
mod payment_handler;
mod bill_handler;
mod agent_handler;
mod sync_handler;

pub use auth_handler::*;
pub use wallet_handler::*;
pub use payment_handler::*;
pub use bill_handler::*;
pub use agent_handler::*;
pub use sync_handler::*;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            // Auth routes
            .service(
                web::scope("/auth")
                    .route("/register", web::post().to(register))
                    .route("/verify-otp", web::post().to(verify_otp))
                    .route("/login", web::post().to(login))
            )
            // User routes
            .service(
                web::scope("/users")
                    .route("/profile", web::get().to(get_profile))
                    .route("/profile", web::put().to(update_profile))
                    .route("/pin", web::post().to(set_pin))
            )
            // Wallet routes
            .service(
                web::scope("/wallets")
                    .route("", web::get().to(get_wallet))
                    .route("/topup", web::post().to(request_topup))
                    .route("/withdraw", web::post().to(request_withdraw))
                    .route("/history", web::get().to(get_wallet_history))
            )
            // Payment routes
            .service(
                web::scope("/payments")
                    .route("/p2p", web::post().to(p2p_payment))
                    .route("/p2p/{id}", web::get().to(get_payment_status))
                    .route("/p2p/reverse", web::post().to(reverse_payment))
            )
            // Bill routes
            .service(
                web::scope("/bills")
                    .route("/providers", web::get().to(get_bill_providers))
                    .route("/validate", web::get().to(validate_bill_account))
                    .route("/pay", web::post().to(pay_bill))
                    .route("/history", web::get().to(get_bill_history))
            )
            // Agent routes
            .service(
                web::scope("/agents")
                    .route("/nearby", web::get().to(find_nearby_agents))
                    .route("/cash-in", web::post().to(record_cash_in))
                    .route("/cash-out", web::post().to(record_cash_out))
            )
            // Sync routes
            .service(
                web::scope("/sync")
                    .route("/push", web::post().to(push_sync))
                    .route("/pull", web::get().to(pull_sync))
            )
    );
}
