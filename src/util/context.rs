use crate::build::{
    package_tracker::package_tracker::PackageTracker, registry::registry_handler::Registry,
};
use crate::util::config::Config;

pub struct Context {
    pub config: Config,
    pub registry: Registry,
    pub tracker: PackageTracker,
}

impl Context {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config = Config::load()?;

        let registry = Registry::load_or_sync(&config.index_path, &config.registry_path);

        let tracker =
            PackageTracker::load(config.packages_path.to_str().unwrap()).unwrap_or_default();

        Ok(Self {
            config,
            registry,
            tracker,
        })
    }
}
