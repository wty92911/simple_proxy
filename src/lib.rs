use async_trait::async_trait;
use conf::ProxyConfig;
use pingora::{http::ResponseHeader, prelude::*};
use tracing::info;
pub mod conf;

pub struct SimpleProxy {}

pub struct ProxyContext {
    pub config: ProxyConfig,
}

#[async_trait]
impl ProxyHttp for SimpleProxy {
    /// For this small example, we don't need context storage
    type CTX = ();
    fn new_ctx(&self) -> Self::CTX {}

    async fn upstream_peer(&self, _session: &mut Session, _ctx: &mut ()) -> Result<Box<HttpPeer>> {
        let peer = HttpPeer::new("127.0.0.1:3000", false, "localhost1".to_string());
        info!("upstream_peer, peer: {peer:?}");
        Ok(Box::new(peer))
    }

    async fn upstream_request_filter(
        &self,
        _session: &mut Session,
        upstream_request: &mut RequestHeader,
        _ctx: &mut (),
    ) -> Result<()> {
        upstream_request.insert_header("user-agent", "simple-proxy-agent")?;
        Ok(())
    }

    fn upstream_response_filter(
        &self,
        _session: &mut Session,
        upstream_response: &mut ResponseHeader,
        _ctx: &mut (),
    ) {
        upstream_response
            .insert_header("user-agent", "simple-proxy-agent")
            .unwrap();
    }
}
