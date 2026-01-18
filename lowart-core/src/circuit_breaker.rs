use tokio::sync::RwLock;

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// 断路器状态
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CircuitState {
    Closed,     // 闭合 (正常)
    Open,       // 开启 (熔断)
    HalfOpen,   // 半开 (探测)
}

/// 模型健康统计
struct HealthStats {
    state: CircuitState,
    failure_count: u32,
    last_failure_time: Option<Instant>,
    last_success_time: Option<Instant>,
}

impl Default for HealthStats {
    fn default() -> Self {
        Self {
            state: CircuitState::Closed,
            failure_count: 0,
            last_failure_time: None,
            last_success_time: None,
        }
    }
}

/// 智能断路器管理器
/// 实现原理: 基于简单计数与超时机制的状态机。
/// 1. 当失败次数超过阈值时，进入 Open 状态。
/// 2. 在 Open 状态维持一段时间后，自动进入 HalfOpen。
/// 3. HalfOpen 期间，第一个成功请求会将状态重置为 Closed。
pub struct CircuitBreaker {
    stats: RwLock<HashMap<String, HealthStats>>,
    failure_threshold: u32,
    reset_timeout: Duration,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u32, reset_timeout: Duration) -> Self {
        Self {
            stats: RwLock::new(HashMap::new()),
            failure_threshold,
            reset_timeout,
        }
    }

    /// 检查指定模型是否允许访问
    pub async fn is_allowed(&self, model_id: &str) -> bool {
        let mut stats_map = self.stats.write().await;
        let health = stats_map.entry(model_id.to_string()).or_default();

        match health.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                if let Some(last_fail) = health.last_failure_time {
                    if last_fail.elapsed() > self.reset_timeout {
                        // 熔断时间到，进入探测期
                        health.state = CircuitState::HalfOpen;
                        tracing::info!("模型 {} 进入半开 (Half-Open) 状态", model_id);
                        return true;
                    }
                }
                false
            }
            CircuitState::HalfOpen => true, // 允许少量探测流量
        }
    }

    /// 上报执行结果
    pub async fn report_result(&self, model_id: &str, is_success: bool) {
        let mut stats_map = self.stats.write().await;
        let health = stats_map.entry(model_id.to_string()).or_default();

        if is_success {
            if health.state == CircuitState::HalfOpen {
                health.state = CircuitState::Closed;
                health.failure_count = 0;
                tracing::info!("模型 {} 恢复正常 (Closed)", model_id);
            }
            health.last_success_time = Some(Instant::now());
        } else {
            health.failure_count += 1;
            health.last_failure_time = Some(Instant::now());

            if health.failure_count >= self.failure_threshold {
                if health.state != CircuitState::Open {
                    health.state = CircuitState::Open;
                    tracing::warn!("模型 {} 触发熔断 (Open)，当前失败数: {}", model_id, health.failure_count);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_circuit_breaker_transitions() {
        let cb = CircuitBreaker::new(3, Duration::from_millis(100));
        let model = "test-model";

        // 1. Initial State: Closed
        assert!(cb.is_allowed(model).await);

        // 2. Report 3 failures -> Open
        cb.report_result(model, false).await;
        cb.report_result(model, false).await;
        cb.report_result(model, false).await;
        assert!(!cb.is_allowed(model).await);

        // 3. Wait for reset timeout -> HalfOpen
        tokio::time::sleep(Duration::from_millis(110)).await;
        assert!(cb.is_allowed(model).await); // Enters HalfOpen

        // 4. Report success -> Closed
        cb.report_result(model, true).await;
        assert!(cb.is_allowed(model).await);
        
        // Ensure failure count is reset
        cb.report_result(model, false).await;
        assert!(cb.is_allowed(model).await);
    }

    #[tokio::test]
    async fn test_circuit_breaker_half_open_failure() {
        let cb = CircuitBreaker::new(3, Duration::from_millis(100));
        let model = "test-model";

        // Open state
        for _ in 0..3 { cb.report_result(model, false).await; }
        assert!(!cb.is_allowed(model).await);

        // HalfOpen state
        tokio::time::sleep(Duration::from_millis(110)).await;
        assert!(cb.is_allowed(model).await);

        // Success in HalfOpen -> Closed
        cb.report_result(model, true).await;
        
        // Failure now shouldn't trip it immediately
        cb.report_result(model, false).await;
        assert!(cb.is_allowed(model).await);
    }
}

