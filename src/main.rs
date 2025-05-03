use pingora::prelude::*;
use simple_proxy::SimpleProxy;
fn main() {
    tracing_subscriber::fmt::init();
    let mut my_server = Server::new(None).unwrap();
    my_server.bootstrap();
    let sp = SimpleProxy {};
    let mut proxy = http_proxy_service(&my_server.configuration, sp);
    proxy.add_tcp("127.0.0.1:6188");
    my_server.add_service(proxy);
    my_server.run_forever();
}
