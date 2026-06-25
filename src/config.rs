use std::fmt::Display;
use std::net::SocketAddr;
use std::str::FromStr;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Missing environment variable: {0}")]
    MissingVar(String),

    #[error("Failed to parse environment variable '{var}': {source}")]
    ParseError {
        var: String,
        #[source]
        source: anyhow::Error,
    },

    #[error("Configuration validation failed: {0}")]
    ValidationError(String),
}

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub logging: LoggingConfig,
    pub rate_limits: RateLimitsConfig,
    pub core: CoreConfig,
}

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub bind_address: SocketAddr,
    pub cors_allowed_origins: String,
}

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub db_type: DatabaseType,
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
}

#[derive(Debug, Clone)]
pub struct RateLimitsConfig {
    pub public_replenish_interval_millis: u64,
    pub public_burst_size: u32,
    pub public_strategy: RateLimitStrategy,
    pub private_replenish_interval_millis: u64,
    pub private_burst_size: u32,
}

#[derive(Debug, Clone)]
pub struct CoreConfig {
    pub enable_janitor: bool,
    pub enable_webhook_dispatcher: bool,
    pub janitor_interval_secs: u64,
    pub webhook_dispatch_interval_secs: u64,
    pub webhook_selection_limit: usize,
    pub webhook_concurrency_limit: usize,
}

impl AppConfig {
    pub fn load() -> Result<Self, ConfigError> {
        let _ = dotenvy::dotenv();

        let server = ServerConfig {
            bind_address: Self::get_env_parsed("BIND_ADDRESS")?,
            cors_allowed_origins: Self::get_env("CORS_ALLOWED_ORIGINS")?,
        };

        let database = DatabaseConfig {
            db_type: Self::get_env_parsed("DATABASE_TYPE")?,
            url: Self::get_env("DATABASE_URL")?,
            max_connections: Self::get_env_parsed_or("DATABASE_MAX_CONNECTIONS", 20)?,
        };

        let logging = LoggingConfig {
            level: Self::get_env_or("RUST_LOG", "info,necko3_core=debug".to_string()),
            format: Self::get_env_or("LOG_FORMAT", "compact".to_string()),
        };

        let rate_limits = RateLimitsConfig {
            public_replenish_interval_millis: Self::get_env_parsed_or("PUBLIC_RL_REPLENISH_INTERVAL_MS", 1000)?,
            public_burst_size: Self::get_env_parsed_or("PUBLIC_RL_BURST_SIZE", 10)?,
            public_strategy: Self::get_env_parsed_or("PUBLIC_RATE_LIMIT_STRATEGY", RateLimitStrategy::Peer)?,
            private_replenish_interval_millis: Self::get_env_parsed_or("PRIVATE_RL_REPLENISH_INTERVAL_MS", 200)?,
            private_burst_size: Self::get_env_parsed_or("PRIVATE_RL_BURST_SIZE", 50)?,
        };

        let core = CoreConfig {
            enable_janitor: Self::get_env_parsed_or("ENABLE_JANITOR", true)?,
            enable_webhook_dispatcher: Self::get_env_parsed_or("ENABLE_WEBHOOK_DISPATCHER", true)?,
            janitor_interval_secs: Self::get_env_parsed_or("JANITOR_INTERVAL_SECS", 30)?,
            webhook_dispatch_interval_secs: Self::get_env_parsed_or("WEBHOOK_DISPATCH_INTERVAL_SECS", 5)?,
            webhook_selection_limit: Self::get_env_parsed_or("WEBHOOK_SELECTION_LIMIT", 50)?,
            webhook_concurrency_limit: Self::get_env_parsed_or("WEBHOOK_CONCURRENCY_LIMIT", 100)?,
        };

        let config = AppConfig {
            server,
            database,
            logging,
            rate_limits,
            core,
        };

        config.validate()?;

        Ok(config)
    }


    fn get_env(key: &str) -> Result<String, ConfigError> {
        std::env::var(key).map_err(|_| ConfigError::MissingVar(key.to_string()))
    }

    fn get_env_or(key: &str, default: String) -> String {
        std::env::var(key).unwrap_or(default)
    }

    fn get_env_parsed<T>(key: &str) -> Result<T, ConfigError>
    where
        T: FromStr,
        T::Err: Display,
    {
        let raw = Self::get_env(key)?;
        raw.parse::<T>().map_err(|err| ConfigError::ParseError {
            var: key.to_string(),
            source: anyhow::anyhow!("{}", err),
        })
    }

    fn get_env_parsed_or<T>(key: &str, default: T) -> Result<T, ConfigError>
    where
        T: FromStr,
        T::Err: Display,
    {
        match std::env::var(key) {
            Ok(raw) => raw.parse::<T>().map_err(|err| ConfigError::ParseError {
                var: key.to_string(),
                source: anyhow::anyhow!("{}", err),
            }),
            Err(_) => Ok(default),
        }
    }

    fn validate(&self) -> Result<(), ConfigError> {
        if self.database.max_connections == 0 {
            return Err(ConfigError::ValidationError(
                "DATABASE_MAX_CONNECTIONS must be greater than 0".to_string(),
            ));
        }

        if self.rate_limits.public_burst_size == 0 || self.rate_limits.private_burst_size == 0 {
            return Err(ConfigError::ValidationError(
                "Rate limit burst sizes must be greater than 0".to_string(),
            ));
        }

        if self.core.webhook_selection_limit == 0 {
            return Err(ConfigError::ValidationError(
                "WEBHOOK_SELECTION_LIMIT must be greater than 0".to_string()
            ));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatabaseType {
    Postgres,
    Mock,
}

impl FromStr for DatabaseType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "postgres" => Ok(Self::Postgres),
            "mock" => Ok(Self::Mock),
            _ => Err(format!(
                "Invalid DATABASE_TYPE: '{}'. Supported types: postgres, mock",
                s
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RateLimitStrategy {
    Peer,
    Smart,
}

impl FromStr for RateLimitStrategy {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "peer" => Ok(Self::Peer),
            "smart" => Ok(Self::Smart),
            _ => Err(format!(
                "Invalid strategy '{}'. Allowed values: peer, smart",
                s
            )),
        }
    }
}
