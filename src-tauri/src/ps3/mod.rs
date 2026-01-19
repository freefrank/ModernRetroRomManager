//! PS3 ROM 相关功能模块
//!
//! 包含 PARAM.SFO 解析、Boxart 生成等 PS3 特定功能

pub mod sfo;
pub mod boxart;

// 重新导出常用类型和函数
pub use sfo::{parse_param_sfo, parse_param_sfo_from_iso, Ps3GameInfo};
pub use boxart::generate_ps3_boxart;
