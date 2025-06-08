use std::path::PathBuf;

use clap::{Parser, arg};
use pingora::prelude::*;
use simple_proxy::SimpleProxy;
use simple_proxy::conf::ProxyConfig;
use tracing::info;

#[derive(Parser)]
struct Args {
    #[arg(short, long)]
    #[arg(default_value = "config.yaml")]
    config: PathBuf,
}

fn main() {
    tracing_subscriber::fmt::init();
    let mut my_server = Server::new(None).unwrap();
    my_server.bootstrap();

    let args = Args::parse();
    let config = ProxyConfig::load(args.config).unwrap();
    let sp = SimpleProxy::new(config);

    let port = sp.config().get().global.port;
    let proxy_addr = format!("0.0.0.0:{}", port);
    let mut proxy = http_proxy_service(&my_server.configuration, sp);
    proxy.add_tcp(&proxy_addr);

    info!("proxy server started at {}", proxy_addr);
    my_server.add_service(proxy);
    my_server.run_forever();
}
