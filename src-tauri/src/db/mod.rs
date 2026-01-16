pub mod models;
pub mod schema;

use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use std::path::PathBuf;
use std::sync::OnceLock;

pub type DbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

static DB_POOL: OnceLock<DbPool> = OnceLock::new();

/// 获取数据库文件路径
pub fn get_db_path(app_data_dir: &PathBuf) -> PathBuf {
    app_data_dir.join("data.db")
}

/// 初始化数据库连接池
pub fn init_db(app_data_dir: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let db_path = get_db_path(app_data_dir);
    
    // 确保目录存在
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    
    let database_url = db_path.to_string_lossy().to_string();
    let manager = ConnectionManager::<SqliteConnection>::new(&database_url);
    
    let pool = r2d2::Pool::builder()
        .max_size(5)
        .build(manager)?;
    
    // 运行迁移
    let mut conn = pool.get()?;
    run_migrations(&mut conn)?;
    
    DB_POOL.set(pool).map_err(|_| "Database pool already initialized")?;
    
    Ok(())
}

/// 获取数据库连接
pub fn get_connection() -> Result<r2d2::PooledConnection<ConnectionManager<SqliteConnection>>, r2d2::PoolError> {
    DB_POOL.get().expect("Database not initialized").get()
}

/// 运行数据库迁移
fn run_migrations(conn: &mut SqliteConnection) -> Result<(), Box<dyn std::error::Error>> {
    // 启用外键约束
    diesel::sql_query("PRAGMA foreign_keys = ON;").execute(conn)?;
    
    // 创建表结构 - 先过滤注释行，再按分号分割执行
    let migration_sql = include_str!("../migrations/001_initial.sql");
    
    // 过滤掉注释行
    let sql_without_comments: String = migration_sql
        .lines()
        .filter(|line| !line.trim().starts_with("--"))
        .collect::<Vec<_>>()
        .join("\n");
    
    // 按分号分割并执行每条语句
    for statement in sql_without_comments.split(';') {
        let statement = statement.trim();
        if !statement.is_empty() {
            diesel::sql_query(statement).execute(conn)?;
        }
    }
    
    Ok(())
}
