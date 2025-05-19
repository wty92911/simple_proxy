mod raw;
mod resolved;

use std::sync::Arc;

use arc_swap::ArcSwap;
pub use raw::*;
pub use resolved::*;

pub struct ProxyConfig(Arc<ArcSwap<SimpleProxyConfigResolved>>);

impl ProxyConfig {
    pub fn new(config: SimpleProxyConfigResolved) -> Self {
        let config = Arc::new(ArcSwap::new(Arc::new(config)));
        Self(config)
    }

    pub fn update(&self, config: SimpleProxyConfigResolved) {
        self.0.store(Arc::new(config));
    }

    pub fn get(&self) -> Arc<SimpleProxyConfigResolved> {
        self.0.load_full()
    }
}

impl std::ops::Deref for ProxyConfig {
    type Target = ArcSwap<SimpleProxyConfigResolved>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
