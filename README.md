# Lowart-api
[中文](./README-zh.md)

Lowart-api is a lightweight, enterprise-grade AI API gateway built with Rust. It provides a unified interface for managing both cloud-based AI providers (e.g., OpenAI, Anthropic) and local AI workflows (e.g., ComfyUI), featuring industrial-strength stability, dynamic governance, and comprehensive observability.

## 🌟 Key Features

### 1. High Availability & Resilience
- **Circuit Breaker**: Monitors provider health using a sliding window to automatically isolate failing models and prevent cascading failures.
- **Multi-Level Fallback**: Configure model priority chains to seamlessly switch to backup models within seconds upon primary failure, ensuring zero downtime.
- **Async Job Tracking**: Transform long-running generation tasks (e.g., ComfyUI) into background jobs with status polling and result persistence.

### 2. Governance & Agent Ecosystem
- **Deep MCP Integration**: Full support for Model Context Protocol (Stdio-based), enabling dynamic integration of tool servers written in Node.js, Python, etc.
- **Human-in-the-Loop (HITL)**: Built-in tool execution policies (Auto, Confirm, Block) with session-level support for manual authorization and replay.
- **Agent Bus (A2A)**: Asynchronous message bus for task distribution and efficient collaboration between multiple AI Agents.

### 3. Extensibility & Security
- **Rhai Scripting Engine**: Hot-reload request/response transformation logic without server restarts, making it easy to adapt to non-standard protocols.
- **Commercial-Grade Billing**:
  - **Accurate SSE Tracking**: Solves the challenge of tracking tokens for streaming outputs.
  - **Quota Management**: Token-based tiered billing and RPM (Requests Per Minute) rate limiting.
- **Hardened Security**: Provider API Keys are encrypted with AES-256-GCM. Built-in RBAC for administrative access control.

### 4. Industrial Observability
- **High-Performance Caching**: Integrated `moka` cache for millisecond responses on authentication and configuration lookups.
- **Dual-Mode Connectivity**: Support for standard TCP and ultra-low latency Unix Domain Sockets (UDS).
- **Prometheus Metrics**: Real-time exposure of model traffic, response latency, and token consumption data at the `/metrics` endpoint.

---

## 🚀 Usage Scenarios

### Scenario 1: Enterprise Unified AI Gateway
Centralize LLM access for multiple internal development teams:
- **Description**: Manage API Keys across different vendors and enforce token quotas and rate limits per department.
- **Value**: Ensure business continuity through circuit breaking and automatic failover if a provider goes down.

### Scenario 2: Advanced Tooling for Agent Systems
Empower autonomous Agents with a robust toolset:
- **Description**: Connect local databases, search plugins, or complex Python scripts via MCP.
- **Value**: Use the three-tier governance policy (HITL) to ensure sensitive operations (e.g., deletions, transfers) require manual approval.

### Scenario 3: Async Generative AI Pipeline
Handle time-consuming image or video generation tasks:
- **Description**: Connect to generative engines like ComfyUI, converting HTTP requests into async jobs where users poll for results via Job IDs.
- **Value**: Use Rhai scripts to dynamically adapt to complex ComfyUI JSON workflows.

---

## 🏗 Architecture Overview

The project is structured as a modular Cargo workspace:
- **`api-server`**: Built with Axum 0.8, handling UDS/TCP listeners, routing, and administrative middleware.
- **`lowart-core`**: The heart of the system, containing `CircuitBreaker`, `ModelManager`, `TokenCounter`, and `RhaiEngine`.
- **`models`**: Adapter layer for OpenAI, Anthropic, and ComfyUI protocol conversions.
- **`db`** Persistence layer using SQLite (SQLx) with automated migration covering the unified schema.
- **`auth`**: Identity and security layer managing API Key verification and RBAC.
- **`protocols`**: Implementation of SSE, A2A Bus, and MCP client communications.

---

## 🛠 Getting Started

### Prerequisites
- Rust (1.80+)
- SQLite

### Installation
1. **Clone the Repo**:
   ```bash
   git clone <repository_url>
   cd lowart-api
   ```
2. **Setup**:
   Database `lowart.db` is automatically created on first run using the consolidated migration `01_initial_schema.sql`.
3. **Run**:
   ```bash
   # Standard HTTP mode
   cargo run -p api-server

   # High-performance UDS mode
   LISTEN_MODE=UDS cargo run -p api-server
   ```

### Admin Operations (Example)
- **Register an MCP Tool Server**:
  ```bash
  curl -X POST http://localhost:3000/admin/mcp/register \
    -H "Authorization: Bearer <ADMIN_KEY>" \
    -d '{"name": "os-tools", "command": "python3", "args": ["tools.py"]}'
  ```

---

## 📈 Monitoring
Access `http://localhost:3000/metrics` for real-time Prometheus data:
- `http_requests_total`: Total request count.
- `gateway_tokens_total`: Token throughput per model.
- `http_request_duration_seconds`: Response latency distribution distribution.

## 🖥 Web Administrative UI (Admin UI)
We provide a modern management console built with Nuxt.js, located in the `lowart-admin` directory:
- **Features**: Visual user management, model configuration, real-time monitoring, and online chat testing.
- **Getting Started**:
  ```bash
  cd lowart-admin
  # Install dependencies on first run
  npm install
  # Start the development server
  npm run dev
  ```
- **Access**: The default address is `http://localhost:3000`.

## 📑 User Tutorial
Want to learn more about user management, model configuration, and API calling details? Please read the [User Tutorial](./docs/tutorial.md).

## 📝 License
Apache 2.0 License
