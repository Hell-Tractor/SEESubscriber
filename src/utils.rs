use std::sync::OnceLock;

use config::Config;

pub fn config() -> &'static Config {
    static CONFIG: OnceLock<Config> = OnceLock::new();
    CONFIG.get_or_init(|| {
        Config::builder()
            .add_source(config::File::with_name("config.yaml"))
            .add_source(config::Environment::with_prefix("SEE").separator("_"))
            .build()
            .unwrap()
    })
}