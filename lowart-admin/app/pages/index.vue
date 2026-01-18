<template>
  <div class="dashboard">
    <div class="stats-grid">
      <div class="stat-card glass">
        <div class="icon-box blue">
          <Activity :size="20" />
        </div>
        <div class="stat-info">
          <span class="label">今日请求数</span>
          <h3 class="value">{{ stats.requests }}</h3>
          <span class="trend up">+12% vs last day</span>
        </div>
      </div>
      <div class="stat-card glass">
        <div class="icon-box purple">
          <Coins :size="20" />
        </div>
        <div class="stat-info">
          <span class="label">Tokens 消耗</span>
          <h3 class="value">{{ stats.tokens.toLocaleString() }}</h3>
          <span class="trend up">+5.2% vs last day</span>
        </div>
      </div>
      <div class="stat-card glass">
        <div class="icon-box green">
          <Users :size="20" />
        </div>
        <div class="stat-info">
          <span class="label">活跃用户</span>
          <h3 class="value">{{ stats.users }}</h3>
          <span class="trend">Stable</span>
        </div>
      </div>
      <div class="stat-card glass">
        <div class="icon-box orange">
          <Zap :size="20" />
        </div>
        <div class="stat-info">
          <span class="label">平均耗时</span>
          <h3 class="value">{{ stats.latency }}ms</h3>
          <span class="trend down">-15ms improvement</span>
        </div>
      </div>
    </div>

    <div class="charts-section">
      <div class="chart-container glass">
        <div class="chart-header">
          <h3>Token 消耗趋势</h3>
          <div class="chart-actions">
            <button class="btn-item active">7 Days</button>
            <button class="btn-item">30 Days</button>
          </div>
        </div>
        <div class="chart-placeholder">
          <BarChart :size="48" class="placeholder-icon" />
          <p>Chart data being integrated with Chart.js...</p>
        </div>
      </div>
      <div class="model-dist glass">
        <h3>模型调用占比</h3>
        <div class="chart-placeholder circle">
          <PieChart :size="48" class="placeholder-icon" />
        </div>
        <div class="model-list">
          <div v-for="m in models" :key="m.name" class="model-item">
            <span class="dot" :style="{ background: m.color }"></span>
            <span class="name">{{ m.name }}</span>
            <span class="percent">{{ m.percent }}%</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { 
  Activity, 
  Coins, 
  Users, 
  Zap, 
  BarChart, 
  PieChart 
} from 'lucide-vue-next'

const stats = reactive({
  requests: 1245,
  tokens: 856000,
  users: 42,
  latency: 450
})

const models = [
  { name: 'gpt-4o', percent: 65, color: '#38bdf8' },
  { name: 'claude-3-opus', percent: 20, color: '#818cf8' },
  { name: 'sd-xl', percent: 15, color: '#10b981' }
]
</script>

<style scoped>
.stats-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
  gap: 1.5rem;
  margin-bottom: 2rem;
}

.stat-card {
  display: flex;
  align-items: center;
  gap: 1.25rem;
  padding: 1.5rem;
}

.icon-box {
  width: 48px;
  height: 48px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.icon-box.blue { background: #e0f2fe; color: #0ea5e9; }
.icon-box.purple { background: #eef2ff; color: #6366f1; }
.icon-box.green { background: #f0fdf4; color: #10b981; }
.icon-box.orange { background: #fff7ed; color: #f59e0b; }

.stat-info .label {
  display: block;
  font-size: 0.875rem;
  color: var(--text-secondary);
  margin-bottom: 0.25rem;
}

.stat-info .value {
  font-size: 1.5rem;
  margin-bottom: 0.25rem;
}

.stat-info .trend {
  font-size: 0.75rem;
  color: var(--text-secondary);
}

.trend.up { color: var(--success); }
.trend.down { color: var(--error); }

.charts-section {
  display: grid;
  grid-template-columns: 2fr 1fr;
  gap: 1.5rem;
}

.chart-container, .model-dist {
  padding: 1.5rem;
}

.chart-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 2rem;
}

.chart-actions {
  display: flex;
  gap: 0.5rem;
  background: var(--bg-primary);
  padding: 4px;
  border-radius: 8px;
}

.btn-item {
  padding: 4px 12px;
  font-size: 0.75rem;
  border-radius: 6px;
  color: var(--text-secondary);
}

.btn-item.active {
  background: var(--bg-secondary);
  color: var(--text-primary);
}

.chart-placeholder {
  height: 240px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 1rem;
  color: var(--text-secondary);
  border: 2px dashed var(--glass-border);
  border-radius: var(--border-radius);
}

.chart-placeholder.circle {
  height: 180px;
  width: 180px;
  margin: 0 auto 1.5rem;
  border-radius: 50%;
}

.model-list {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.model-item {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  font-size: 0.875rem;
}

.model-item .dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
}

.model-item .percent {
  margin-left: auto;
  font-weight: 600;
}
</style>
