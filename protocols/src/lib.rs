pub mod sse;
pub mod a2a;
pub mod mcp;

pub use sse::{SseEvent, SseStream};

#[derive(Debug, utils::ThisError)]
pub enum Error {
    #[error("协议转换错误: {0}")]
    ProtocolError(String),
    #[error("内部错误")]
    InternalError,
}
