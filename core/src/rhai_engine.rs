use rhai::{Engine, Dynamic, Scope};
use utils::{Result, anyhow};
use crate::token_counter::TokenCounter;

/// Rhai 脚本引擎封装
/// 实现原理: 使用 Rhai 提供动态格式转换能力。将 Token 计算等能力注入脚本作用域，使其可以动态干预消息流。
pub struct RhaiEngine {
    engine: Engine,
}

impl RhaiEngine {
    pub fn new() -> Self {
        let mut engine = Engine::new();

        // 注入 Token 计算能力
        engine.register_fn("count_tokens", |text: String| {
            TokenCounter::count_tokens(&text) as i64
        });

        Self { engine }
    }

    /// 执行转换脚本
    /// 示例: 将输入 payload 转换。
    /// 实现注意: Rhai 的 EvalAltResult 不满足 Sync，需要手动转换为 String 再包装。
    pub fn transform(&self, script: &str, input: serde_json::Value) -> Result<serde_json::Value> {
        let mut scope = Scope::new();
        
        // 分别转换，避免直接在非 Sync 类型上使用 ?
        let input_dynamic: Dynamic = match rhai::serde::to_dynamic(input) {
            Ok(d) => d,
            Err(e) => return Err(anyhow!("Rhai 输入转换失败: {}", e.to_string())),
        };

        scope.push("input", input_dynamic);

        let result: Dynamic = match self.engine.eval_with_scope(&mut scope, script) {
            Ok(d) => d,
            Err(e) => return Err(anyhow!("Rhai 脚本执行失败: {}", e.to_string())),
        };

        let output = match rhai::serde::from_dynamic(&result) {
            Ok(v) => v,
            Err(e) => return Err(anyhow!("Rhai 结果序列化失败: {}", e.to_string())),
        };

        Ok(output)
    }
}
