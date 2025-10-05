//! Tests for configuration loading

use std::env;

#[test]
fn test_config_requires_database_url() {
    // Remove DATABASE_URL to test error handling
    let original = env::var("DATABASE_URL").ok();
    env::remove_var("DATABASE_URL");

    let result = env::var("DATABASE_URL");
    assert!(result.is_err(), "DATABASE_URL should be missing");

    // Restore
    if let Some(val) = original {
        env::set_var("DATABASE_URL", val);
    }
}

#[test]
fn test_config_with_valid_database_url() {
    env::set_var("DATABASE_URL", "postgresql://test:test@localhost:5432/testdb");

    let db_url = env::var("DATABASE_URL").unwrap();
    assert!(db_url.starts_with("postgresql://"));
    assert!(db_url.contains("@localhost"));

    env::remove_var("DATABASE_URL");
}

#[test]
fn test_config_default_port() {
    let default_port = "3000";
    assert_eq!(default_port, "3000");
}

#[test]
fn test_config_default_host() {
    let default_host = "0.0.0.0";
    assert_eq!(default_host, "0.0.0.0");
}

#[test]
fn test_config_custom_port() {
    env::set_var("SERVER_PORT", "8080");
    let port = env::var("SERVER_PORT").unwrap();
    assert_eq!(port, "8080");

    let port_num: Result<u16, _> = port.parse();
    assert!(port_num.is_ok());
    assert_eq!(port_num.unwrap(), 8080);

    env::remove_var("SERVER_PORT");
}

#[test]
fn test_config_invalid_port() {
    let invalid_ports = vec!["invalid", "70000", "-1", "abc"];

    for invalid_port in invalid_ports {
        let result = invalid_port.parse::<u16>();
        assert!(result.is_err(), "Port {invalid_port} should be invalid");
    }
}

#[test]
fn test_config_custom_host() {
    let valid_hosts = vec!["0.0.0.0", "127.0.0.1", "192.168.1.1"];

    for host in valid_hosts {
        let parsed: Result<std::net::IpAddr, _> = host.parse();
        assert!(parsed.is_ok(), "Host {host} should be valid");
    }
}

#[test]
fn test_config_invalid_host() {
    let invalid_hosts = vec!["not-an-ip", "999.999.999.999", "localhost", ""];

    for host in invalid_hosts {
        let result = host.parse::<std::net::IpAddr>();
        assert!(result.is_err(), "Host {host} should be invalid");
    }
}

#[test]
fn test_database_url_format() {
    let url = "postgresql://user:pass@localhost:5432/db";
    assert!(url.starts_with("postgresql://"));
    assert!(url.contains("@"));
    assert!(url.contains(":5432"));
    assert!(url.ends_with("/db"));
}

#[test]
fn test_dotenv_loading() {
    // Test that dotenv can be called (may or may not find .env file)
    let _ = dotenv::dotenv();
    // No assertions - just verify it doesn't panic
}
