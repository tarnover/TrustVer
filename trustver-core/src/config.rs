use std::path::Path;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::version::TrustVersion;

// ---------------------------------------------------------------------------
// ConfigError
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("TOML parse error: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("TOML serialize error: {0}")]
    Serialize(#[from] toml::ser::Error),
    #[error("config file already exists")]
    AlreadyExists,
    #[error("config file not found")]
    NotFound,
}

// ---------------------------------------------------------------------------
// Custom serde for TrustVersion as a plain string
// ---------------------------------------------------------------------------

mod version_string {
    use crate::version::TrustVersion;
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(v: &TrustVersion, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&v.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<TrustVersion, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub package_name: String,
    #[serde(with = "version_string")]
    pub current_version: TrustVersion,
    #[serde(default)]
    pub strict: bool,
}

impl Config {
    /// Create a default config for a new package.
    pub fn default_with_name(name: String) -> Self {
        Config {
            package_name: name,
            current_version: "0.1.0+mix".parse().expect("hardcoded version is valid"),
            strict: false,
        }
    }

    /// Load config from a TOML file. Returns `ConfigError::NotFound` if the
    /// file does not exist.
    pub fn load(path: &Path) -> Result<Self, ConfigError> {
        if !path.exists() {
            return Err(ConfigError::NotFound);
        }
        let contents = std::fs::read_to_string(path)?;
        Self::from_toml_str(&contents)
    }

    /// Write config to a TOML file, creating or overwriting it.
    pub fn save(&self, path: &Path) -> Result<(), ConfigError> {
        let contents = self.to_toml_string()?;
        std::fs::write(path, contents)?;
        Ok(())
    }

    /// Deserialize from a TOML string.
    pub fn from_toml_str(s: &str) -> Result<Self, ConfigError> {
        let config = toml::from_str(s)?;
        Ok(config)
    }

    /// Serialize to a TOML string.
    pub fn to_toml_string(&self) -> Result<String, ConfigError> {
        let s = toml::to_string(self)?;
        Ok(s)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn config_roundtrip() {
        let config = Config {
            package_name: "mylib".to_string(),
            current_version: "1.2.3+hrai".parse().unwrap(),
            strict: false,
        };
        let toml_str = config.to_toml_string().unwrap();
        let loaded = Config::from_toml_str(&toml_str).unwrap();
        assert_eq!(loaded.package_name, "mylib");
        assert_eq!(loaded.current_version.to_string(), "1.2.3+hrai");
        assert!(!loaded.strict);
    }

    #[test]
    fn config_default_version() {
        let config = Config::default_with_name("testpkg".to_string());
        assert_eq!(config.current_version.to_string(), "0.1.0+mix");
        assert!(!config.strict);
    }

    #[test]
    fn config_load_from_file() {
        let mut f = NamedTempFile::new().unwrap();
        writeln!(f, r#"package_name = "mylib""#).unwrap();
        writeln!(f, r#"current_version = "2.0.0+h""#).unwrap();
        writeln!(f, "strict = true").unwrap();
        let config = Config::load(f.path()).unwrap();
        assert_eq!(config.package_name, "mylib");
        assert!(config.strict);
    }
}
