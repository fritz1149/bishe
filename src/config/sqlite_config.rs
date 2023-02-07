use lazy_static::lazy_static;
use log::LevelFilter;
use rbatis::Rbatis;
use tokio::sync::Mutex;

// 加载全局资源
lazy_static!{
    pub static ref RB: Mutex<Rbatis> = Mutex::new(
        init()
    );
}

pub fn init() -> Rbatis {
    let rb = Rbatis::new();
    rb.init(
        rbdc_sqlite::driver::SqliteDriver{},
        "sqlite://target/sqlite.db"
    ).unwrap();
    rb
}

pub async fn create_table(rb: &Rbatis) {
    let sql = std::fs::read_to_string("resources/create_table.sql").unwrap();
    let raw = fast_log::LOGGER.get_level().clone();
    fast_log::LOGGER.set_level(LevelFilter::Off);
    let _ = rb.exec(&sql, vec![]).await;
    fast_log::LOGGER.set_level(raw);
}

