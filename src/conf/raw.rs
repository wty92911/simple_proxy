use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
pub struct SimpleProxyConfig {
    pub global: GlobalConfig,
    pub servers: Vec<ServerConfig>,
    pub upstreams: Vec<UpstreamConfig>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GlobalConfig {
    pub port: u16,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tls: Option<TlsConfig>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TlsConfig {
    pub cert: String,
    pub key: String,
    pub ca: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ServerConfig {
    pub server_name: Vec<String>,
    pub upstream: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tls: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UpstreamConfig {
    pub name: String,
    pub servers: Vec<String>,
}

impl SimpleProxyConfig {
    pub fn new(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref();
        let config = std::fs::read_to_string(path).unwrap();
        let config: SimpleProxyConfig = serde_yaml::from_str(&config).unwrap();
        config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn get_test_config_path() -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("fixtures");
        path.push("sample.yaml");
        path
    }

    #[test]
    fn test_config_parsing() {
        let config = SimpleProxyConfig::new(get_test_config_path());

        // Test global config
        assert_eq!(config.global.port, 8080);
        assert!(config.global.tls.is_none());

        // Test servers
        assert_eq!(config.servers.len(), 2);

        // Test first server (web servers)
        let web_server = &config.servers[0];
        assert_eq!(web_server.server_name, vec!["acme.com", "www.acme.com"]);
        assert_eq!(web_server.upstream, "web_servers");

        // Test second server (api servers)
        let api_server = &config.servers[1];
        assert_eq!(api_server.server_name, vec!["api.acme.com"]);
        assert_eq!(api_server.upstream, "api_servers");

        // Test upstreams
        assert_eq!(config.upstreams.len(), 2);

        // Test web servers upstream
        let web_upstream = &config.upstreams[0];
        assert_eq!(web_upstream.name, "web_servers");
        assert_eq!(
            web_upstream.servers,
            vec!["127.0.0.1:3001", "127.0.0.1:3002"]
        );

        // Test api servers upstream
        let api_upstream = &config.upstreams[1];
        assert_eq!(api_upstream.name, "api_servers");
        assert_eq!(
            api_upstream.servers,
            vec!["127.0.0.1:3003", "127.0.0.1:3004"]
        );
    }
}
