pub mod traits;
pub mod openai_api;
pub mod anthropic_api;
pub mod comfyui_api;
pub mod mock_api;

pub use traits::AiModel;
pub use openai_api::OpenAiAdapter;
pub use anthropic_api::AnthropicAdapter;
pub use comfyui_api::ComfyUiAdapter;
pub use mock_api::MockAdapter;


