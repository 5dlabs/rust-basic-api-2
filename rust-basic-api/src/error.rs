use std::num::ParseIntError;

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("environment variable `{key}` is required but missing")]
    MissingEnv { key: String },
    #[error("environment variable `{key}` contained invalid unicode characters")]
    InvalidUnicode { key: String },
    #[error("environment variable `{key}` cannot be empty")]
    EmptyEnv { key: String },
    #[error("environment variable `{key}` could not be parsed: {source}")]
    InvalidValue {
        key: String,
        #[source]
        source: ParseIntError,
    },
    #[error("environment variable `{key}` must be at least {min}")]
    InvalidRange { key: String, min: u32 },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_error_display() {
        let missing_env = ConfigError::MissingEnv {
            key: "TEST_VAR".to_string(),
        };
        assert_eq!(
            missing_env.to_string(),
            "environment variable `TEST_VAR` is required but missing"
        );

        let invalid_unicode = ConfigError::InvalidUnicode {
            key: "TEST_VAR".to_string(),
        };
        assert_eq!(
            invalid_unicode.to_string(),
            "environment variable `TEST_VAR` contained invalid unicode characters"
        );

        let empty_env = ConfigError::EmptyEnv {
            key: "TEST_VAR".to_string(),
        };
        assert_eq!(
            empty_env.to_string(),
            "environment variable `TEST_VAR` cannot be empty"
        );

        let invalid_range = ConfigError::InvalidRange {
            key: "TEST_VAR".to_string(),
            min: 1,
        };
        assert_eq!(
            invalid_range.to_string(),
            "environment variable `TEST_VAR` must be at least 1"
        );
    }

    #[test]
    fn test_config_error_debug() {
        let error = ConfigError::MissingEnv {
            key: "TEST_VAR".to_string(),
        };
        let debug_output = format!("{error:?}");
        assert!(debug_output.contains("MissingEnv"));
        assert!(debug_output.contains("TEST_VAR"));
    }
}
