# Lowart-api
[中文](./README-zh.md)
Lowart-api is a lightweight, high-performance AI API service built with Rust. It is designed to be compatible with both network-based AI vendors and local AI workflows, providing a unified interface for AI interactions.

## Key Features

- **Modular Architecture**: Built as a Cargo workspace with clean separation of concerns.
- **Multi-Model Support**: Standard adapters for OpenAI, Anthropic, and ComfyUI.
- **Dynamic Transformation**: Integrated **Rhai** scripting engine for on-the-fly request and response payload transformations.
- **Efficient Routing**: A `ModelManager` that dynamically selects adapters based on database configurations.
- **Real-time Streaming**: Full support for Server-Sent Events (SSE) across all vendors.
- **Comprehensive Statistics**: Automatic tracking of token counts, request duration, and usage frequency stored in SQLite.
- **Dual Mode Connectivity**: Supports both standard HTTP and ultra-low latency Unix Domain Socket (UDS).
- **Secure Authentication**: Middleware-based API Key verification.

## Architecture

The project is divided into several specialized crates:
- `api-server`: The core gateway handling HTTP/UDS routes and middlewares.
- `core`: The heart of the engine, managing `ModelManager`, `TokenCounter`, and `RhaiEngine`.
- `models`: Unified trait-based implementations for different AI vendors.
- `db`: Database layer using SQLite (SQLx) for persistence.
- `auth`: Identity management and authentication logic.
- `protocols`: Support for SSE, A2A, and MCP protocols.
- `utils`: Shared utilities for logging and error handling.

## Getting Started

### Prerequisites
- Rust (latest stable version)
- SQLite

### Installation
1. Clone the repository:
   ```bash
   git clone <repository_url>
   cd lowart-api
   ```
2. Initialize the database:
   The application will automatically create `lowart.db` and initialize the schema on the first run.

### Running the Server
```bash
# Start in HTTP mode (default)
cargo run -p api-server

# Or set custom listen mode
LISTEN_MODE=UDS cargo run -p api-server
```

## Usage Example

### Chat Completion (Blocking)
```bash
curl -X POST http://localhost:3000/v1/chat/completions \
  -H "Authorization: Bearer <YOUR_API_KEY>" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "gpt-4",
    "messages": [{"role": "user", "content": "Hello!"}]
  }'
```

### Chat Completion (Streaming)
```bash
curl -X POST http://localhost:3000/v1/chat/completions \
  -H "Authorization: Bearer <YOUR_API_KEY>" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "gpt-4",
    "stream": true,
    "messages": [{"role": "user", "content": "Explain Rust in detail."}]
  }'
```

## Contributing
Contributions are welcome! Please feel free to submit a Pull Request.

## License
MIT License
