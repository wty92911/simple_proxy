use std::time::Duration;

use async_trait::async_trait;
#[cfg(unix)]
use pingora::server::ListenFds;
use pingora::{server::ShutdownWatch, services::Service};
use tracing::info;

use crate::proxy::route::RouteTable;

const HEALTH_CHECK_INTERVAL: Duration = Duration::from_secs(1);
pub struct HealthCheck {
    route_table: RouteTable,
}

impl HealthCheck {
    pub fn new(route_table: RouteTable) -> Self {
        Self { route_table }
    }
}

#[async_trait]
impl Service for HealthCheck {
    async fn start_service(
        &mut self,
        #[cfg(unix)] _fds: Option<ListenFds>,
        mut _shutdown: ShutdownWatch,
    ) {
        let mut ticker = tokio::time::interval(HEALTH_CHECK_INTERVAL);
        let route = self.route_table.pin_owned();
        loop {
            ticker.tick().await;
            for (host, entry) in route.iter() {
                info!("health check: {}", host);

                entry.upstream.update().await.ok();
                entry.upstream.backends().run_health_check(true).await;
            }
        }
    }

    fn name(&self) -> &str {
        "health-check"
    }

    fn threads(&self) -> Option<usize> {
        Some(1)
    }
}
