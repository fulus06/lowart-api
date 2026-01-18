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
          <span class="trend" :class="stats.requestsTrend >= 0 ? 'up' : 'down'">
            {{ stats.requestsTrend >= 0 ? '+' : '' }}{{ stats.requestsTrend }}% 较昨日
          </span>
        </div>
      </div>
      <div class="stat-card glass">
        <div class="icon-box purple">
          <Coins :size="20" />
        </div>
        <div class="stat-info">
          <span class="label">Tokens 消耗</span>
          <h3 class="value">{{ stats.tokens.toLocaleString() }}</h3>
          <span class="trend" :class="stats.tokensTrend >= 0 ? 'up' : 'down'">
            {{ stats.tokensTrend >= 0 ? '+' : '' }}{{ stats.tokensTrend }}% 较昨日
          </span>
        </div>
      </div>
      <div class="stat-card glass">
        <div class="icon-box green">
          <Users :size="20" />
        </div>
        <div class="stat-info">
          <span class="label">活跃用户</span>
          <h3 class="value">{{ stats.users }}</h3>
          <span class="trend">稳定</span>
        </div>
      </div>
      <div class="stat-card glass">
        <div class="icon-box orange">
          <Zap :size="20" />
        </div>
        <div class="stat-info">
          <span class="label">平均耗时</span>
          <h3 class="value">{{ stats.latency }}ms</h3>
          <span class="trend" :class="stats.latencyTrend <= 0 ? 'up' : 'down'">
            {{ stats.latencyTrend > 0 ? '+' : '' }}{{ stats.latencyTrend }}ms 
            {{ stats.latencyTrend <= 0 ? '提升' : '劣化' }}
          </span>
        </div>
      </div>
    </div>

    <div class="charts-section">
      <div class="chart-container glass">
        <div class="chart-header">
          <h3>Token 消耗趋势 (近 7 日)</h3>
        </div>
        <div class="chart-box">
          <Bar v-if="tokenChartData" :data="tokenChartData" :options="chartOptions" />
          <div v-else class="chart-placeholder">加载中...</div>
        </div>
      </div>
      <div class="model-dist glass">
        <h3>模型调用占比</h3>
        <div class="chart-box doughnut">
          <Doughnut v-if="modelChartData" :data="modelChartData" :options="doughnutOptions" />
          <div v-else class="chart-placeholder circle">加载中...</div>
        </div>
        <div class="model-list">
          <div v-for="(m, i) in models" :key="m.name" class="model-item">
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
} from 'lucide-vue-next'
import { 
  Chart as ChartJS, 
  Title, 
  Tooltip, 
  Legend, 
  BarElement, 
  CategoryScale, 
  LinearScale, 
  ArcElement 
} from 'chart.js'
import { Bar, Doughnut } from 'vue-chartjs'

ChartJS.register(Title, Tooltip, Legend, BarElement, CategoryScale, LinearScale, ArcElement)

const { getStats, getModels, getUsers } = useApi()

const stats = reactive({
  requests: 0,
  requestsTrend: 0,
  tokens: 0,
  tokensTrend: 0,
  users: 0,
  latency: 0,
  latencyTrend: 0
})

const models = ref([])
const tokenChartData = ref(null)
const modelChartData = ref(null)

const chartOptions = {
  responsive: true,
  maintainAspectRatio: false,
  plugins: { legend: { display: false } },
  scales: {
    y: { grid: { color: 'rgba(255, 255, 255, 0.05)' }, ticks: { color: 'rgba(255, 255, 255, 0.5)' } },
    x: { grid: { display: false }, ticks: { color: 'rgba(255, 255, 255, 0.5)' } }
  }
}

const doughnutOptions = {
  responsive: true,
  maintainAspectRatio: false,
  plugins: { legend: { display: false } },
  cutout: '70%'
}

const loadDashboardData = async () => {
  try {
    const [statsData, modelsData, usersData] = await Promise.all([
      getStats(),
      getModels(),
      getUsers()
    ])

    const now = new Date()
    const oneDayMs = 24 * 60 * 60 * 1000
    
    // 1. Calculate Trends (24h Window)
    const currentWindowLogs = statsData.filter(s => (now.getTime() - new Date(s.timestamp).getTime()) <= oneDayMs)
    const previousWindowLogs = statsData.filter(s => {
      const diff = now.getTime() - new Date(s.timestamp).getTime()
      return diff > oneDayMs && diff <= (2 * oneDayMs)
    })

    stats.requests = currentWindowLogs.length
    stats.tokens = currentWindowLogs.reduce((acc, curr) => acc + curr.request_tokens + curr.response_tokens, 0)
    stats.users = usersData.length
    const totalLatency = currentWindowLogs.reduce((acc, curr) => acc + curr.duration_ms, 0)
    stats.latency = currentWindowLogs.length > 0 ? Math.round(totalLatency / currentWindowLogs.length) : 0

    const prevRequests = previousWindowLogs.length
    const prevTokens = previousWindowLogs.reduce((acc, curr) => acc + curr.request_tokens + curr.response_tokens, 0)
    const prevTotalLatency = previousWindowLogs.reduce((acc, curr) => acc + curr.duration_ms, 0)
    const prevLatency = previousWindowLogs.length > 0 ? Math.round(prevTotalLatency / previousWindowLogs.length) : 0

    const calcTrend = (curr, prev) => {
      if (prev === 0) return curr > 0 ? 100 : 0
      return Math.round(((curr - prev) / prev) * 100)
    }

    stats.requestsTrend = calcTrend(stats.requests, prevRequests)
    stats.tokensTrend = calcTrend(stats.tokens, prevTokens)
    stats.latencyTrend = stats.latency - prevLatency

    // 2. Token Trend Chart (Last 7 Days)
    const days = []
    const dailyTokens = []
    for (let i = 6; i >= 0; i--) {
      const d = new Date(now.getTime() - i * oneDayMs)
      const dateStr = d.toLocaleDateString(undefined, { month: 'short', day: 'numeric' })
      days.push(dateStr)
      
      const dayStart = new Date(d.setHours(0, 0, 0, 0)).getTime()
      const dayEnd = dayStart + oneDayMs
      const dayTokens = statsData
        .filter(s => {
          const t = new Date(s.timestamp).getTime()
          return t >= dayStart && t < dayEnd
        })
        .reduce((acc, curr) => acc + curr.request_tokens + curr.response_tokens, 0)
      dailyTokens.push(dayTokens)
    }

    tokenChartData.value = {
      labels: days,
      datasets: [{
        label: 'Tokens',
        data: dailyTokens,
        backgroundColor: '#6366f1',
        borderRadius: 4
      }]
    }

    // 3. Model Distribution Chart
    const modelUsage = {}
    statsData.forEach(s => {
      modelUsage[s.model_id] = (modelUsage[s.model_id] || 0) + 1
    })

    const chartColors = ['#0ea5e9', '#6366f1', '#10b981', '#f59e0b', '#ef4444']
    const modelLabels = []
    const modelCounts = []
    const currentModels = []

    modelsData.slice(0, 5).forEach((m, i) => {
      const count = modelUsage[m.model_id] || 0
      modelLabels.push(m.model_id)
      modelCounts.push(count)
      
      const percent = statsData.length > 0 ? Math.round((count / statsData.length) * 100) : 0
      currentModels.push({
        name: m.model_id,
        percent: percent,
        color: chartColors[i % chartColors.length]
      })
    })

    models.value = currentModels
    modelChartData.value = {
      labels: modelLabels,
      datasets: [{
        data: modelCounts,
        backgroundColor: chartColors,
        borderWidth: 0,
        hoverOffset: 10
      }]
    }

  } catch (e) {
    console.error('Failed to load dashboard data:', e)
  }
}

onMounted(loadDashboardData)
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
