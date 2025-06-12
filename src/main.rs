use std::path::PathBuf;

use clap::{Parser, arg};
use pingora::{listeners::tls::TlsSettings, prelude::*, server::configuration::ServerConf};
use simple_proxy::SimpleProxy;
use simple_proxy::conf::ProxyConfig;
use tracing::info;

#[derive(Parser)]
struct Args {
    #[arg(short, long)]
    #[arg(default_value = "./fixtures/sample.yml")]
    config: PathBuf,
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let args = Args::parse();
    let config = ProxyConfig::load(args.config).unwrap();

    let tls_conf = config.get().global.tls.clone();

    let server_conf = ServerConf {
        ca_file: tls_conf.as_ref().and_then(|tls| tls.ca.clone()),
        ..Default::default()
    };
    let mut my_server = Server::new_with_opt_and_conf(None, server_conf);
    my_server.bootstrap();
    let sp = SimpleProxy::new(config);

    let port = sp.config().get().global.port;
    let proxy_addr = format!("0.0.0.0:{}", port);
    let mut proxy = http_proxy_service(&my_server.configuration, sp);

    match tls_conf.as_ref() {
        Some(tls) => {
            let mut tls_settings = TlsSettings::intermediate(&tls.cert, &tls.key)?;
            tls_settings.enable_h2();
            proxy.add_tls_with_settings(&proxy_addr, None, tls_settings);
            info!("proxy server started at https://{}", proxy_addr);
        }
        None => {
            proxy.add_tcp(&proxy_addr);
            info!("proxy server started at http://{}", proxy_addr);
        }
    }

    my_server.add_service(proxy);
    my_server.run_forever();
}
