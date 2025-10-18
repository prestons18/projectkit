use config::{Config, ConfigError, File};
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub auth: AuthConfig,
    pub server: ServerConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AuthConfig {
    pub jwt_secret: String,
    #[serde(default = "default_token_expiry")]
    pub token_expiry_seconds: i64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
}

fn default_token_expiry() -> i64 {
    3600 // 1 hour
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    3000
}

impl AppConfig {
    /// Load configuration from a TOML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let config = Config::builder()
            .add_source(File::from(path.as_ref()))
            .build()?;
        
        config.try_deserialize()
    }

    /// Load configuration from projectkit.toml in the current directory
    pub fn load() -> Result<Self, ConfigError> {
        Self::from_file("projectkit.toml")
    }

    /// Load configuration with environment variable overrides
    /// Environment variables should be prefixed with PROJECTKIT_
    /// Example: PROJECTKIT_DATABASE_URL, PROJECTKIT_AUTH_JWT_SECRET
    /// 
    /// Returns the config and a list of environment variable overrides
    pub fn load_with_env() -> Result<(Self, Vec<String>), ConfigError> {
        // Load with environment overrides
        let config = Config::builder()
            .add_source(File::with_name("projectkit").required(false))
            .add_source(
                config::Environment::with_prefix("PROJECTKIT")
                    .separator("_")
            )
            .build()?;
        
        // Detect which values were overridden by environment
        let mut overrides = Vec::new();
        
        // Check common environment variables
        let env_vars = [
            ("PROJECTKIT_DATABASE_URL", "database.url"),
            ("PROJECTKIT_AUTH_JWT_SECRET", "auth.jwt_secret"),
            ("PROJECTKIT_AUTH_TOKEN_EXPIRY_SECONDS", "auth.token_expiry_seconds"),
            ("PROJECTKIT_SERVER_HOST", "server.host"),
            ("PROJECTKIT_SERVER_PORT", "server.port"),
        ];
        
        for (env_var, config_key) in env_vars {
            if std::env::var(env_var).is_ok() {
                overrides.push(config_key.to_string());
            }
        }
        
        let app_config = config.try_deserialize()?;
        Ok((app_config, overrides))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_values() {
        assert_eq!(default_token_expiry(), 3600);
        assert_eq!(default_host(), "0.0.0.0");
        assert_eq!(default_port(), 3000);
    }
}
