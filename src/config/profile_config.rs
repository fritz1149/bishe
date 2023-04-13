use lazy_static::lazy_static;
use serde::Deserialize;
#[derive(Deserialize)]
#[derive(Debug)]
pub struct Config {
    pub info_management_address: String,
    pub flowdeploy_backend_address: String,
    pub pgsql_address: String,
}

lazy_static!{
    pub static ref CONFIG: Config = init();
}

fn init() -> Config{
    let profile = std::fs::read_to_string("resources/profile.toml").expect("配置文件\"profile.toml\"不存在，初始化失败");
    let profile: Config = toml::from_str(&profile).unwrap();
    println!("config: {:?}", profile);
    profile
}