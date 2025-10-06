use std::num::ParseIntError;

use thiserror::Error;

/// Errors that can occur while building application configuration.
#[derive(Debug, Error)]
pub enum ConfigError {
    /// Required environment variable is missing.
    #[error("environment variable `{name}` is not set")]
    MissingEnvVar { name: &'static str },

    /// Environment variable contains invalid (non-UTF-8) data.
    #[error("environment variable `{name}` contains invalid UTF-8 data")]
    InvalidUnicode { name: &'static str },

    /// Provided server port could not be parsed into a valid `u16` value.
    #[error("invalid server port `{port}`: {source}")]
    InvalidServerPort {
        port: String,
        #[source]
        source: ParseIntError,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_missing_env_var_error_display() {
        let error = ConfigError::MissingEnvVar {
            name: "DATABASE_URL",
        };

        let error_message = format!("{error}");

        assert_eq!(
            error_message,
            "environment variable `DATABASE_URL` is not set"
        );
    }

    #[test]
    fn test_invalid_unicode_error_display() {
        let error = ConfigError::InvalidUnicode {
            name: "SERVER_PORT",
        };

        let error_message = format!("{error}");

        assert_eq!(
            error_message,
            "environment variable `SERVER_PORT` contains invalid UTF-8 data"
        );
    }

    #[test]
    fn test_invalid_server_port_error_display() {
        let parse_error = "not_a_number".parse::<u16>().unwrap_err();
        let error = ConfigError::InvalidServerPort {
            port: "not_a_number".to_string(),
            source: parse_error,
        };

        let error_message = format!("{error}");

        assert!(error_message.contains("invalid server port `not_a_number`"));
    }

    #[test]
    fn test_invalid_server_port_error_source() {
        use std::error::Error;

        let parse_error = "not_a_number".parse::<u16>().unwrap_err();
        let error = ConfigError::InvalidServerPort {
            port: "not_a_number".to_string(),
            source: parse_error,
        };

        assert!(error.source().is_some());
    }

    #[test]
    fn test_error_debug() {
        let error = ConfigError::MissingEnvVar {
            name: "DATABASE_URL",
        };

        let debug_output = format!("{error:?}");

        assert!(debug_output.contains("MissingEnvVar"));
        assert!(debug_output.contains("DATABASE_URL"));
    }

    #[test]
    fn test_all_error_variants_are_error_trait() {
        use std::error::Error;

        let missing_var = ConfigError::MissingEnvVar { name: "TEST" };
        let invalid_unicode = ConfigError::InvalidUnicode { name: "TEST" };
        let parse_err = "invalid".parse::<u16>().unwrap_err();
        let invalid_port = ConfigError::InvalidServerPort {
            port: "invalid".to_string(),
            source: parse_err,
        };

        // All should implement Error trait
        let _: &dyn Error = &missing_var;
        let _: &dyn Error = &invalid_unicode;
        let _: &dyn Error = &invalid_port;
    }
}
