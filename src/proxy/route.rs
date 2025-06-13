use std::{sync::Arc, time::Duration};

use papaya::HashMap;
use pingora_load_balancing::{Backend, LoadBalancer, health_check, selection::RoundRobin};

use crate::conf::{ServerConfigResolved, SimpleProxyConfigResolved};

const HEALTH_CHECK_FREQUENCY: Duration = Duration::from_secs(10);
#[derive(Clone)]
pub struct RouteTable(pub(crate) Arc<HashMap<String, RouteEntry>>);

impl RouteTable {
    pub fn new(config: &SimpleProxyConfigResolved) -> anyhow::Result<Self> {
        let route_table = HashMap::new();
        {
            let map = route_table.pin();
            for (name, server) in config.servers.iter() {
                map.insert(name.clone(), RouteEntry::new(server)?);
            }
        }
        Ok(Self(Arc::new(route_table)))
    }
}

impl std::ops::Deref for RouteTable {
    type Target = HashMap<String, RouteEntry>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone)]
pub struct RouteEntry {
    pub upstream: Arc<LoadBalancer<RoundRobin>>,
    pub tls: bool,
}

impl RouteEntry {
    const MAX_BACKEND_ITER: usize = 32;
    pub fn new(config: &ServerConfigResolved) -> anyhow::Result<Self> {
        let mut lb =
            LoadBalancer::try_from_iter(config.upstream.servers.iter().map(|s| s.to_string()))?;
        let hc = health_check::TcpHealthCheck::new();
        lb.set_health_check(hc);
        lb.health_check_frequency = Some(HEALTH_CHECK_FREQUENCY);

        Ok(Self {
            upstream: Arc::new(lb),
            tls: config.tls,
        })
    }

    pub(crate) fn select(&self, _host: &str) -> Option<Backend> {
        self.upstream
            .select_with(b"", Self::MAX_BACKEND_ITER, |_b, health| health)
    }
}
