use axum::http;
use pingora::proxy::Session;

pub(crate) fn get_session_host_port(session: &Session) -> (&str, u16) {
    let uri = &session.req_header().uri;
    let default_port = match uri.scheme() {
        Some(scheme) if scheme.as_str() == "https" => 443,
        _ => 80,
    };

    match session.get_header(http::header::HOST) {
        Some(host) => {
            let parts: Vec<&str> = host.to_str().unwrap_or_default().split(':').collect();
            let host = parts[0];
            let port = parts[1].parse().unwrap_or(default_port);
            (host, port)
        }
        None => (
            uri.host().unwrap_or_default(),
            uri.port_u16().unwrap_or(default_port),
        ),
    }
}
