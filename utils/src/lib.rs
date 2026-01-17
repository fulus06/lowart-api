pub mod logger;

pub use anyhow::{Error, Result, anyhow, Context};
pub use thiserror::Error as ThisError;

// 后续可在此添加自定义错误类型
