use crate::user_models::CreateUser;
use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct Server {
    pub hostname: String,
    pub port: String,
    pub key: String,
    pub threads: usize,
}

#[derive(Deserialize, Clone)]
pub struct TagifyData {
    pub path: String,
    pub google_storage_enable: bool,
    pub google_key_json: String,
    pub key_file: String,
    pub project_number: String,
}

#[derive(Deserialize, Clone)]
pub struct LetsEncrypt {
    pub port: String,
    pub path: String,
    pub domain: String,
    pub email: String,
    pub timeout: u64,
    pub activate: bool,
}

#[derive(Deserialize, Clone)]
pub struct MyConfig {
    pub postgres: deadpool_postgres::Config,
    pub server: Server,
    pub cert: LetsEncrypt,
    pub default_admin: CreateUser,
    pub default_user: CreateUser,
    pub tagify_data: TagifyData,
}

impl MyConfig {
    pub fn new(path: &str) -> Result<Self, config::ConfigError> {
        let mut settings = config::Config::default();
        settings.merge(config::File::with_name(path)).unwrap();
        settings.try_into()
    }
}
