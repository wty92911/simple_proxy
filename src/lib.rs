use async_trait::async_trait;
use axum::http::{StatusCode, header};
use bytes::Bytes;
use conf::ProxyConfig;
use pingora::{
    cache::{CacheKey, CacheMeta, NoCacheReason, RespCacheable, RespCacheable::*, key::HashBinary},
    http::ResponseHeader,
    modules::http::HttpModules,
    modules::http::compression::ResponseCompressionBuilder,
    prelude::*,
    protocols::Digest,
    protocols::http::conditional_filter,
    proxy::PurgeStatus,
};
use std::time::Duration;
use tracing::info;
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
        info!("Creating new proxy context");
        ProxyContext {
            config: self.config.clone(),
        }
    }

    async fn upstream_peer(
        &self,
        session: &mut Session,
        ctx: &mut Self::CTX,
    ) -> Result<Box<HttpPeer>> {
        info!(
            "upstream_peer, request headers: {:?}",
            session.req_header().headers
        );
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

                    // Set up TLS for upstream connection if configured
                    let peer = HttpPeer::new(upstream, server.tls, host.to_string());
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

    fn init_downstream_modules(&self, modules: &mut HttpModules) {
        info!("Initializing downstream modules");
        // Add disabled downstream compression module by default
        modules.add_module(ResponseCompressionBuilder::enable(0));
    }

    async fn request_filter(&self, session: &mut Session, _ctx: &mut Self::CTX) -> Result<bool> {
        info!(
            "request_filter, request headers: {:?}",
            session.req_header().headers
        );
        Ok(false)
    }

    async fn early_request_filter(
        &self,
        session: &mut Session,
        _ctx: &mut Self::CTX,
    ) -> Result<()> {
        info!(
            "early_request_filter, request headers: {:?}",
            session.req_header().headers
        );
        Ok(())
    }

    async fn request_body_filter(
        &self,
        _session: &mut Session,
        body: &mut Option<Bytes>,
        end_of_stream: bool,
        _ctx: &mut Self::CTX,
    ) -> Result<()> {
        info!(
            "request_body_filter, body length: {:?}, end_of_stream: {}",
            body.as_ref().map(|b| b.len()),
            end_of_stream
        );
        Ok(())
    }

    fn request_cache_filter(&self, session: &mut Session, _ctx: &mut Self::CTX) -> Result<()> {
        info!(
            "request_cache_filter, request headers: {:?}",
            session.req_header().headers
        );
        Ok(())
    }

    fn cache_key_callback(&self, session: &Session, _ctx: &mut Self::CTX) -> Result<CacheKey> {
        info!(
            "cache_key_callback, request headers: {:?}",
            session.req_header().headers
        );
        let req_header = session.req_header();
        Ok(CacheKey::default(req_header))
    }

    fn cache_miss(&self, session: &mut Session, _ctx: &mut Self::CTX) {
        info!(
            "cache_miss, request headers: {:?}",
            session.req_header().headers
        );
        session.cache.cache_miss();
    }

    async fn cache_hit_filter(
        &self,
        session: &Session,
        meta: &CacheMeta,
        _ctx: &mut Self::CTX,
    ) -> Result<bool> {
        info!(
            "cache_hit_filter, request headers: {:?}, cache meta: {:?}",
            session.req_header().headers,
            meta
        );
        Ok(false)
    }

    async fn proxy_upstream_filter(
        &self,
        session: &mut Session,
        _ctx: &mut Self::CTX,
    ) -> Result<bool> {
        info!(
            "proxy_upstream_filter, request headers: {:?}",
            session.req_header().headers
        );
        Ok(true)
    }

    fn response_cache_filter(
        &self,
        session: &Session,
        resp: &ResponseHeader,
        _ctx: &mut Self::CTX,
    ) -> Result<RespCacheable> {
        info!(
            "response_cache_filter, request headers: {:?}, response headers: {:?}",
            session.req_header().headers,
            resp.headers
        );
        Ok(Uncacheable(NoCacheReason::Custom("default")))
    }

    fn cache_vary_filter(
        &self,
        meta: &CacheMeta,
        _ctx: &mut Self::CTX,
        req: &RequestHeader,
    ) -> Option<HashBinary> {
        info!(
            "cache_vary_filter, request headers: {:?}, cache meta: {:?}",
            req.headers, meta
        );
        None
    }

    fn cache_not_modified_filter(
        &self,
        session: &Session,
        resp: &ResponseHeader,
        _ctx: &mut Self::CTX,
    ) -> Result<bool> {
        info!(
            "cache_not_modified_filter, request headers: {:?}, response headers: {:?}",
            session.req_header().headers,
            resp.headers
        );
        Ok(conditional_filter::not_modified_filter(
            session.req_header(),
            resp,
        ))
    }

    async fn upstream_request_filter(
        &self,
        session: &mut Session,
        upstream_request: &mut RequestHeader,
        _ctx: &mut Self::CTX,
    ) -> Result<()> {
        info!(
            "upstream_request_filter, request headers: {:?}, upstream request headers: {:?}",
            session.req_header().headers,
            upstream_request.headers
        );
        upstream_request.insert_header("user-agent", "simple-proxy-agent")?;
        Ok(())
    }

    fn upstream_response_filter(
        &self,
        session: &mut Session,
        upstream_response: &mut ResponseHeader,
        _ctx: &mut Self::CTX,
    ) {
        info!(
            "upstream_response_filter, request headers: {:?}, upstream response headers: {:?}",
            session.req_header().headers,
            upstream_response.headers
        );
        upstream_response
            .insert_header("user-agent", "simple-proxy-agent")
            .unwrap();
    }

    async fn response_filter(
        &self,
        session: &mut Session,
        upstream_response: &mut ResponseHeader,
        _ctx: &mut Self::CTX,
    ) -> Result<()> {
        info!(
            "response_filter, request headers: {:?}, upstream response headers: {:?}",
            session.req_header().headers,
            upstream_response.headers
        );
        Ok(())
    }

    fn upstream_response_body_filter(
        &self,
        session: &mut Session,
        body: &mut Option<Bytes>,
        end_of_stream: bool,
        _ctx: &mut Self::CTX,
    ) {
        info!(
            "upstream_response_body_filter, request headers: {:?}, body length: {:?}, end_of_stream: {}",
            session.req_header().headers,
            body.as_ref().map(|b| b.len()),
            end_of_stream
        );
    }

    fn upstream_response_trailer_filter(
        &self,
        session: &mut Session,
        upstream_trailers: &mut header::HeaderMap,
        _ctx: &mut Self::CTX,
    ) -> Result<()> {
        info!(
            "upstream_response_trailer_filter, request headers: {:?}, upstream trailers: {:?}",
            session.req_header().headers,
            upstream_trailers
        );
        Ok(())
    }

    fn response_body_filter(
        &self,
        session: &mut Session,
        body: &mut Option<Bytes>,
        end_of_stream: bool,
        _ctx: &mut Self::CTX,
    ) -> Result<Option<Duration>> {
        info!(
            "response_body_filter, request headers: {:?}, body length: {:?}, end_of_stream: {}",
            session.req_header().headers,
            body.as_ref().map(|b| b.len()),
            end_of_stream
        );
        Ok(None)
    }

    async fn response_trailer_filter(
        &self,
        session: &mut Session,
        upstream_trailers: &mut header::HeaderMap,
        _ctx: &mut Self::CTX,
    ) -> Result<Option<Bytes>> {
        info!(
            "response_trailer_filter, request headers: {:?}, upstream trailers: {:?}",
            session.req_header().headers,
            upstream_trailers
        );
        Ok(None)
    }

    async fn logging(&self, session: &mut Session, e: Option<&Error>, _ctx: &mut Self::CTX) {
        info!(
            "logging, request headers: {:?}, error: {:?}",
            session.req_header().headers,
            e
        );
    }

    fn suppress_error_log(&self, session: &Session, _ctx: &Self::CTX, error: &Error) -> bool {
        info!(
            "suppress_error_log, request headers: {:?}, error: {:?}",
            session.req_header().headers,
            error
        );
        false
    }

    fn error_while_proxy(
        &self,
        peer: &HttpPeer,
        session: &mut Session,
        e: Box<Error>,
        _ctx: &mut Self::CTX,
        client_reused: bool,
    ) -> Box<Error> {
        info!(
            "error_while_proxy, peer: {:?}, request headers: {:?}, error: {:?}, client_reused: {}",
            peer,
            session.req_header().headers,
            e,
            client_reused
        );
        let mut e = e.more_context(format!("Peer: {}", peer));
        e.retry
            .decide_reuse(client_reused && !session.as_ref().retry_buffer_truncated());
        e
    }

    fn fail_to_connect(
        &self,
        session: &mut Session,
        peer: &HttpPeer,
        _ctx: &mut Self::CTX,
        e: Box<Error>,
    ) -> Box<Error> {
        info!(
            "fail_to_connect, request headers: {:?}, peer: {:?}, error: {:?}",
            session.req_header().headers,
            peer,
            e
        );
        e
    }

    async fn fail_to_proxy(&self, session: &mut Session, e: &Error, _ctx: &mut Self::CTX) -> u16 {
        info!(
            "fail_to_proxy, request headers: {:?}, error: {:?}",
            session.req_header().headers,
            e
        );
        let server_session = session.as_mut();
        let code = match e.etype() {
            HTTPStatus(code) => *code,
            _ => {
                match e.esource() {
                    ErrorSource::Upstream => 502,
                    ErrorSource::Downstream => {
                        match e.etype() {
                            WriteError | ReadError | ConnectionClosed => {
                                /* conn already dead */
                                0
                            }
                            _ => 400,
                        }
                    }
                    ErrorSource::Internal | ErrorSource::Unset => 500,
                }
            }
        };
        if code > 0 {
            server_session.respond_error(code).await
        }
        code
    }

    fn should_serve_stale(
        &self,
        session: &mut Session,
        _ctx: &mut Self::CTX,
        error: Option<&Error>,
    ) -> bool {
        info!(
            "should_serve_stale, request headers: {:?}, error: {:?}",
            session.req_header().headers,
            error
        );
        error.is_none_or(|e| e.esource() == &ErrorSource::Upstream)
    }

    async fn connected_to_upstream(
        &self,
        session: &mut Session,
        reused: bool,
        peer: &HttpPeer,
        #[cfg(unix)] _fd: std::os::unix::io::RawFd,
        #[cfg(windows)] _sock: std::os::windows::io::RawSocket,
        _digest: Option<&Digest>,
        _ctx: &mut Self::CTX,
    ) -> Result<()> {
        info!(
            "connected_to_upstream, request headers: {:?}, reused: {}, peer: {:?}",
            session.req_header().headers,
            reused,
            peer
        );
        Ok(())
    }

    fn request_summary(&self, session: &Session, _ctx: &Self::CTX) -> String {
        info!(
            "request_summary, request headers: {:?}",
            session.req_header().headers
        );
        session.as_ref().request_summary()
    }

    fn is_purge(&self, session: &Session, _ctx: &Self::CTX) -> bool {
        info!(
            "is_purge, request headers: {:?}",
            session.req_header().headers
        );
        false
    }

    fn purge_response_filter(
        &self,
        session: &Session,
        _ctx: &mut Self::CTX,
        _purge_status: PurgeStatus,
        purge_response: &mut std::borrow::Cow<'static, ResponseHeader>,
    ) -> Result<()> {
        info!(
            "purge_response_filter, request headers: {:?}, purge_response: {:?}",
            session.req_header().headers,
            purge_response
        );
        Ok(())
    }
}
