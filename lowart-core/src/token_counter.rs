use tiktoken_rs::cl100k_base;



/// Token 计算器
/// 实现原理: 基于 tiktoken-rs 封装 LLM Token 计算逻辑。支持根据模型类型选择不同的分词器。
pub struct TokenCounter;

impl TokenCounter {
    /// 计算文本的 Token 数量
    /// 示例: `let count = TokenCounter::count_tokens("Hello world");`
    pub fn count_tokens(text: &str) -> usize {
        let bpe = cl100k_base().unwrap();
        bpe.encode_with_special_tokens(text).len()
    }

    /// 根据消息列表计算 Token (OpenAI 格式)
    pub fn count_messages_tokens(messages: &serde_json::Value) -> usize {
        // 简化实现：将所有内容合并后计算
        if let Some(msg_list) = messages.as_array() {
            let mut full_text = String::new();
            for msg in msg_list {
                if let Some(content) = msg.get("content").and_then(|c| c.as_str()) {
                    full_text.push_str(content);
                }
            }
            return Self::count_tokens(&full_text);
        }
        0
    }
}
