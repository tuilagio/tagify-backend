use serde::Deserialize;

#[derive(Deserialize)]
pub struct Server {
    pub hostname: String,
    pub port: String
}

#[derive(Deserialize)]
pub struct MyConfig {
    pub postgres: deadpool_postgres::Config,
    pub server: Server
}

impl MyConfig {
    pub fn new(path: &str) -> Result<Self, config::ConfigError>{
        let mut settings = config::Config::default();
        settings.merge(config::File::with_name(path)).unwrap();
        settings.try_into()
    }
}