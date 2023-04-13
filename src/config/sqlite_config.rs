use lazy_static::lazy_static;
use log::{debug, LevelFilter};
use rbatis::Rbatis;
use tokio::sync::Mutex;

// 加载全局资源
lazy_static!{
    pub static ref SQLITE: Mutex<Rbatis> = Mutex::new(
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

pub async fn create_table() {
    let rb = SQLITE.lock().await;
    debug!("开始建表");
    let sql = std::fs::read_to_string("sql/create_table.sql").unwrap();
    let raw = fast_log::LOGGER.get_level().clone();
    fast_log::LOGGER.set_level(LevelFilter::Off);
    let res = rb.exec(&sql, vec![]).await;
    fast_log::LOGGER.set_level(raw);
    match res {
        Ok(x) => debug!("建表结果: {}", x.to_string()),
        Err(_) => debug!("创建表失败")
    }
}