//! PS3 ROM 相关功能模块
//!
//! 包含 PARAM.SFO 解析、Boxart 生成等 PS3 特定功能

pub mod boxart;
pub mod sfo;

// 重新导出常用类型和函数
pub use boxart::{
    extract_ps3_logo, extract_ps3_logo_from_iso, generate_ps3_boxart, generate_ps3_boxart_from_iso,
};
pub use sfo::{parse_param_sfo, parse_param_sfo_from_iso};
