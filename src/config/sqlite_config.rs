use lazy_static::lazy_static;
use log::{debug, LevelFilter};
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
        "sqlite://sqlite/sqlite.db"
    ).expect("创建sqlite存储失败！");
    debug!("sqlite初始化完毕");
    rb
}

pub async fn create_table(rb: &Rbatis) {
    let sql = std::fs::read_to_string("resources/create_table.sql").unwrap();
    let raw = fast_log::LOGGER.get_level().clone();
    fast_log::LOGGER.set_level(LevelFilter::Off);
    let res = rb.exec(&sql, vec![]).await;
    if let Err(_) = res {
        debug!("创建表失败");
    }
    fast_log::LOGGER.set_level(raw);
}

