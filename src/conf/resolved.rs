use std::{collections::HashMap, path::Path};

use rand::seq::SliceRandom;

use super::{GlobalConfig, ServerConfig, SimpleProxyConfig, TlsConfig, UpstreamConfig};

#[derive(Debug, Clone)]
pub struct SimpleProxyConfigResolved {
    pub global: GlobalConfigResolved,
    pub servers: HashMap<String, ServerConfigResolved>,
}

#[derive(Debug, Clone)]
pub struct GlobalConfigResolved {
    pub port: u16,
    pub tls: Option<TlsConfigResolved>,
}

#[derive(Debug, Clone)]
pub struct TlsConfigResolved {
    pub cert: String,
    pub key: String,
    pub ca: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ServerConfigResolved {
    pub upstream: UpstreamConfigResolved,
    pub tls: bool,
}

#[derive(Debug, Clone)]
pub struct UpstreamConfigResolved {
    pub name: String,
    pub servers: Vec<String>,
}

impl TryFrom<SimpleProxyConfig> for SimpleProxyConfigResolved {
    type Error = anyhow::Error;

    fn try_from(config: SimpleProxyConfig) -> anyhow::Result<Self> {
        let global = GlobalConfigResolved::try_from(&config.global)?;

        let upstreams: HashMap<String, UpstreamConfigResolved> = config
            .upstreams
            .iter()
            .map(|upstream| {
                (
                    upstream.name.clone(),
                    UpstreamConfigResolved::from(upstream),
                )
            })
            .collect();
        let mut servers = HashMap::new();
        for server in config.servers {
            let server_resolved =
                ServerConfigResolved::try_from_with_upstreams(&server, &upstreams)?;
            for name in server.server_name {
                servers.insert(name, server_resolved.clone());
            }
        }
        Ok(Self { global, servers })
    }
}

impl TryFrom<&GlobalConfig> for GlobalConfigResolved {
    type Error = anyhow::Error;

    fn try_from(config: &GlobalConfig) -> anyhow::Result<Self> {
        let tls = match &config.tls {
            Some(tls) => Some(TlsConfigResolved::try_from(tls)?),
            None => None,
        };
        Ok(Self {
            port: config.port,
            tls,
        })
    }
}

impl TryFrom<&TlsConfig> for TlsConfigResolved {
    type Error = anyhow::Error;

    fn try_from(config: &TlsConfig) -> anyhow::Result<Self> {
        // check if the cert and key exist
        let cert_path = Path::new(&config.cert);
        if !cert_path.exists() {
            return Err(anyhow::anyhow!("cert file does not exist"));
        }
        let key_path = Path::new(&config.key);
        if !key_path.exists() {
            return Err(anyhow::anyhow!("key file does not exist"));
        }

        // check if the ca file exists
        if let Some(ca) = &config.ca {
            let ca_path = Path::new(&ca);
            if !ca_path.exists() {
                return Err(anyhow::anyhow!("ca file does not exist"));
            }
        }
        Ok(Self {
            cert: config.cert.clone(),
            key: config.key.clone(),
            ca: config.ca.clone(),
        })
    }
}

impl From<&UpstreamConfig> for UpstreamConfigResolved {
    fn from(config: &UpstreamConfig) -> Self {
        Self {
            name: config.name.clone(),
            servers: config.servers.clone(),
        }
    }
}

impl ServerConfigResolved {
    fn try_from_with_upstreams(
        config: &ServerConfig,
        upstreams: &HashMap<String, UpstreamConfigResolved>,
    ) -> anyhow::Result<Self> {
        let tls = config.tls.unwrap_or(false);
        let upstream = upstreams
            .get(&config.upstream)
            .ok_or(anyhow::anyhow!("upstream not found"))?
            .clone();
        Ok(Self { upstream, tls })
    }

    pub fn choose(&self) -> Option<&str> {
        self.upstream
            .servers
            .choose(&mut rand::thread_rng())
            .map(|s| s.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conf::raw::{SimpleProxyConfig, TlsConfig};
    use std::path::PathBuf;

    fn get_test_config_path() -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("fixtures");
        path.push("sample.yaml");
        path
    }

    #[test]
    fn test_config_resolution() -> anyhow::Result<()> {
        let config = SimpleProxyConfig::new(get_test_config_path());
        let resolved = SimpleProxyConfigResolved::try_from(config)?;

        // Test global config
        assert_eq!(resolved.global.port, 8080);
        assert!(resolved.global.tls.is_none());

        // Test servers
        assert_eq!(resolved.servers.len(), 3); // acme.com, www.acme.com, api.acme.com

        // Test web server configs
        let web_server = resolved.servers.get("acme.com").unwrap();
        assert_eq!(web_server.upstream.name, "web_servers");
        assert_eq!(
            web_server.upstream.servers,
            vec!["127.0.0.1:3001", "127.0.0.1:3002"]
        );
        assert!(!web_server.tls);

        // Test api server config
        let api_server = resolved.servers.get("api.acme.com").unwrap();
        assert_eq!(api_server.upstream.name, "api_servers");
        assert_eq!(
            api_server.upstream.servers,
            vec!["127.0.0.1:3003", "127.0.0.1:3004"]
        );
        assert!(api_server.tls);

        Ok(())
    }

    #[test]
    fn test_tls_config_validation() {
        // Test with non-existent files
        let tls_config = TlsConfig {
            cert: "non_existent.cert".to_string(),
            key: "non_existent.key".to_string(),
            ca: None,
        };

        let result = TlsConfigResolved::try_from(&tls_config);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("cert file does not exist")
        );

        // Test with non-existent key file
        let tls_config = TlsConfig {
            cert: "./fixtures/certs/sample.crt".to_string(),
            key: "./fixtures/certs/non_existent.key".to_string(),
            ca: None,
        };

        let result = TlsConfigResolved::try_from(&tls_config);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("key file does not exist")
        );

        // Test with non-existent CA file
        let tls_config = TlsConfig {
            cert: "./fixtures/certs/sample.crt".to_string(),
            key: "./fixtures/certs/sample.key".to_string(),
            ca: Some("./fixtures/certs/non_existent.ca".to_string()),
        };

        let result = TlsConfigResolved::try_from(&tls_config);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("ca file does not exist")
        );
    }

    #[test]
    fn test_upstream_resolution() -> anyhow::Result<()> {
        let config = SimpleProxyConfig::new(get_test_config_path());
        let resolved = SimpleProxyConfigResolved::try_from(config)?;

        // Test web upstream resolution
        let web_server = resolved.servers.get("acme.com").unwrap();
        assert_eq!(web_server.upstream.name, "web_servers");
        assert_eq!(web_server.upstream.servers.len(), 2);
        assert!(
            web_server
                .upstream
                .servers
                .contains(&"127.0.0.1:3001".to_string())
        );
        assert!(
            web_server
                .upstream
                .servers
                .contains(&"127.0.0.1:3002".to_string())
        );

        // Test api upstream resolution
        let api_server = resolved.servers.get("api.acme.com").unwrap();
        assert_eq!(api_server.upstream.name, "api_servers");
        assert_eq!(api_server.upstream.servers.len(), 2);
        assert!(
            api_server
                .upstream
                .servers
                .contains(&"127.0.0.1:3003".to_string())
        );
        assert!(
            api_server
                .upstream
                .servers
                .contains(&"127.0.0.1:3004".to_string())
        );

        Ok(())
    }
}
