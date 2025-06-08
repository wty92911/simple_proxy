use async_trait::async_trait;
use axum::http::StatusCode;
use conf::ProxyConfig;
use pingora::{http::ResponseHeader, prelude::*};
pub mod conf;

#[derive(Clone)]
pub struct SimpleProxy {
    pub(crate) config: ProxyConfig,
}

pub struct ProxyContext {
    pub(crate) config: ProxyConfig,
}

impl ProxyContext {
    pub fn new(config: ProxyConfig) -> Self {
        Self { config }
    }
}

impl SimpleProxy {
    pub fn new(config: ProxyConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &ProxyConfig {
        &self.config
    }
}

#[async_trait]
impl ProxyHttp for SimpleProxy {
    /// For this small example, we don't need context storage
    type CTX = ProxyContext;
    fn new_ctx(&self) -> Self::CTX {
        ProxyContext {
            config: self.config.clone(),
        }
    }

    async fn upstream_peer(
        &self,
        session: &mut Session,
        ctx: &mut Self::CTX,
    ) -> Result<Box<HttpPeer>> {
        let config = ctx.config.load();
        if let Some(host) = session
            .req_header()
            .headers
            .get("host")
            .and_then(|h| h.to_str().ok())
            .map(|h| h.split(':').next().unwrap_or(h))
        {
            match config.servers.get(host) {
                Some(server) => {
                    let Some(upstream) = server.choose() else {
                        return Err(Error::create(
                            ErrorType::Custom("No upstream found"),
                            ErrorSource::Upstream,
                            Some("upstream not found".into()),
                            None,
                        ));
                    };
                    let peer = HttpPeer::new(upstream, false, host.to_string());
                    Ok(Box::new(peer))
                }
                None => {
                    return Err(Error::create(
                        ErrorType::Custom("No host found"),
                        ErrorSource::Upstream,
                        Some("server not found".into()),
                        None,
                    ));
                }
            }
        } else {
            return Err(Error::create(
                ErrorType::HTTPStatus(StatusCode::BAD_REQUEST.into()),
                ErrorSource::Upstream,
                Some("host header is required".into()),
                None,
            ));
        }
    }

    async fn upstream_request_filter(
        &self,
        _session: &mut Session,
        upstream_request: &mut RequestHeader,
        _ctx: &mut ProxyContext,
    ) -> Result<()> {
        upstream_request.insert_header("user-agent", "simple-proxy-agent")?;
        Ok(())
    }

    fn upstream_response_filter(
        &self,
        _session: &mut Session,
        upstream_response: &mut ResponseHeader,
        _ctx: &mut ProxyContext,
    ) {
        upstream_response
            .insert_header("user-agent", "simple-proxy-agent")
            .unwrap();
    }
}
