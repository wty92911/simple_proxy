mod raw;
mod resolved;

use std::{path::Path, sync::Arc};

use arc_swap::ArcSwap;
pub use raw::*;
pub use resolved::*;

pub struct ProxyConfig(ArcSwap<SimpleProxyConfigResolved>);

impl ProxyConfig {
    pub fn new(config: SimpleProxyConfigResolved) -> Self {
        let config = ArcSwap::new(Arc::new(config));
        Self(config)
    }

    pub fn load(file: impl AsRef<Path>) -> anyhow::Result<Self> {
        let config = SimpleProxyConfig::new(file);
        Ok(Self::new(config.try_into()?))
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
