use futures::Stream;
use std::pin::Pin;
use serde::Serialize;


/// SSE 事件结构
#[derive(Debug, Serialize)]
pub struct SseEvent {
    pub event: Option<String>,
    pub data: String,
    pub id: Option<String>,
}

impl SseEvent {
    pub fn new(data: impl Into<String>) -> Self {
        Self {
            event: None,
            data: data.into(),
            id: None,
        }
    }

    pub fn to_string(&self) -> String {
        let mut s = String::new();
        if let Some(ref event) = self.event {
            s.push_str(&format!("event: {}\n", event));
        }
        if let Some(ref id) = self.id {
            s.push_str(&format!("id: {}\n", id));
        }
        s.push_str(&format!("data: {}\n\n", self.data));
        s
    }
}

/// SSE 数据流封装
pub type SseStream = Pin<Box<dyn Stream<Item = Result<SseEvent, crate::Error>> + Send>>;
