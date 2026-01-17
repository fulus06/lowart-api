use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// 初始化日志系统
/// 实现原理: 使用 tracing-subscriber 构建一个层级化的日志处理链，支持从环境变量 RUST_LOG 动态调整级别。
/// 示例: `utils::logger::init();`
pub fn init() {
    // 设置默认日志级别为 info，除非环境变量中已指定
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer())
        .init();

    tracing::info!("日志系统初始化完成");
}
