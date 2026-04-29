use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub redis_url: String,
    pub jwt_secret: String,
    pub environment: String,
}

impl Config {
    pub fn from_env() -> Result<Self, config::ConfigError> {
        let settings = config::Config::builder()
            .add_source(config::Environment::default().separator("_"))
            .build()?;

        Ok(Self {
            host: settings.get_string("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: settings.get::<u16>("PORT").unwrap_or(8080),
            database_url: settings.get_string("DATABASE_URL")
                .expect("DATABASE_URL is required"),
            redis_url: settings.get_string("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            jwt_secret: settings.get_string("JWT_SECRET")
                .expect("JWT_SECRET is required"),
            environment: settings.get_string("ENVIRONMENT")
                .unwrap_or_else(|_| "development".to_string()),
        })
    }
}
