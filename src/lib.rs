#[allow(unused_imports)]

#[macro_use]
extern crate platform_config_derive;
pub extern crate config;

#[doc(hidden)]
pub use platform_config_derive::*;

use std::path::Path;
use config::Config;

pub struct PlatformConfigBuilder {
  config: Config
}

impl PlatformConfigBuilder {
  pub fn new() -> Self {
    Self {
      config: Config::new()
    }
  }

  pub fn with_file<P: AsRef<Path>>(self, path: P) -> Self {
    self.with(config::File::from(path.as_ref()))
  }

  pub fn with<TSource>(mut self, source: TSource) -> Self
  where
    TSource: 'static,
    TSource: config::Source + Send + Sync {
      let _ = self.config.merge(source); // TODO: error handling!
      self
    }

  pub fn build<T: From<Config>>(self) -> T {
    self.config.into()
  }
}