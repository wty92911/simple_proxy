use async_trait::async_trait;
use pingora::prelude::*;
pub struct SimpleProxy {}

#[async_trait]
impl ProxyHttp for SimpleProxy {
    /// For this small example, we don't need context storage
    type CTX = ();
    fn new_ctx(&self) -> Self::CTX {}

    async fn upstream_peer(&self, _session: &mut Session, _ctx: &mut ()) -> Result<Box<HttpPeer>> {
        todo!()
    }
}
