<template>
  <div class="logs-page">
    <div class="filter-bar glass">
      <div class="filter-group">
        <label>用户</label>
        <select v-model="filters.user">
          <option value="">所有用户</option>
          <option v-for="u in users" :key="u" :value="u">{{ u }}</option>
        </select>
      </div>
      <div class="filter-group">
        <label>模型</label>
        <select v-model="filters.model">
          <option value="">所有模型</option>
          <option v-for="m in modelList" :key="m" :value="m">{{ m }}</option>
        </select>
      </div>
      <button class="btn secondary" @click="resetFilters">重置</button>
      <div class="search-box">
        <Search :size="16" />
        <input v-model="filters.search" type="text" placeholder="搜索请求 ID..." />
      </div>
    </div>

    <div class="table-container glass">
      <table class="log-table">
        <thead>
          <tr>
            <th>ID</th>
            <th>时间</th>
            <th>类型</th>
            <th>用户</th>
            <th>模型</th>
            <th>Token (Req/Res)</th>
            <th>耗时 (ms)</th>
            <th>操作</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="log in filteredLogs" :key="log.id">
            <td><code class="id-tag">#{{ log.id }}</code></td>
            <td class="time-cell">
              {{ new Date(log.timestamp).toLocaleTimeString() }}
              <span class="date">{{ new Date(log.timestamp).toLocaleDateString() }}</span>
            </td>
            <td>
              <span class="type-badge" :class="{ 'vendor': log.stat_type === '厂商返回响应' }">
                {{ log.stat_type }}
              </span>
            </td>
            <td>{{ log.user_id }}</td>
            <td><code>{{ log.model_id }}</code></td>
            <td>
              <div class="token-viz">
                <span>{{ log.request_tokens }} / {{ log.response_tokens }}</span>
                <div class="mini-bar">
                  <div class="req" :style="{ width: Math.min(log.request_tokens / 10, 50) + 'px' }"></div>
                  <div class="res" :style="{ width: Math.min(log.response_tokens / 10, 50) + 'px' }"></div>
                </div>
              </div>
            </td>
            <td :class="{ slow: log.duration > 2000 }">{{ log.duration }}ms</td>
            <td>
              <button class="icon-btn" title="查看详情" @click="showDetail(log)">
                <ArrowUpRight :size="14" />
              </button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- Detail Modal -->
    <Teleport to="body">
      <div v-if="selectedLog" class="modal-overlay" @click.self="selectedLog = null">
        <div class="modal-content glass">
          <div class="modal-header">
            <h3>日志详情 #{{ selectedLog.id }}</h3>
            <button class="close-btn" @click="selectedLog = null">
              <X :size="20" />
            </button>
          </div>
          <div class="modal-body">
            <pre class="json-viewer"><code>{{ JSON.stringify(selectedLog, null, 2) }}</code></pre>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<script setup>
import { Search, ArrowUpRight, X } from 'lucide-vue-next'

const { getStats } = useApi()
const logs = ref([])
const isLoading = ref(false)
const selectedLog = ref(null)

const showDetail = (log) => {
  selectedLog.value = log
}

const loadStats = async () => {
  isLoading.value = true
  try {
    const data = await getStats()
    logs.value = data.map(item => ({
      ...item,
      duration: item.duration_ms,
      status: 'Success'
    }))
  } catch (e) {
    console.error('Failed to load stats:', e)
  } finally {
    isLoading.value = false
  }
}

onMounted(loadStats)

const users = computed(() => [...new Set(logs.value.map(l => l.user_id))])
const modelList = computed(() => [...new Set(logs.value.map(l => l.model_id))])

const filters = reactive({
  user: '',
  model: '',
  search: ''
})

const filteredLogs = computed(() => {
  return logs.value.filter(l => {
    if (filters.user && l.user_id !== filters.user) return false
    if (filters.model && l.model_id !== filters.model) return false
    if (filters.search && l.id.toString() !== filters.search) return false
    return true
  })
})

const resetFilters = () => {
  filters.user = ''
  filters.model = ''
  filters.search = ''
}
</script>

<style scoped>
.filter-bar {
  display: flex;
  align-items: center;
  gap: 1.5rem;
  padding: 1rem 1.5rem;
  margin-bottom: 1.5rem;
}

.filter-group {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.filter-group label {
  font-size: 0.875rem;
  color: var(--text-secondary);
}

.filter-group select {
  background: var(--bg-primary);
  border: 1px solid var(--glass-border);
  border-radius: 6px;
  padding: 0.4rem 0.75rem;
  color: var(--text-primary);
  outline: none;
}

.search-box {
  margin-left: auto;
  display: flex;
  align-items: center;
  gap: 0.5rem;
  background: var(--bg-primary);
  padding: 0.4rem 0.75rem;
  border-radius: 6px;
  border: 1px solid var(--glass-border);
}

.search-box input {
  background: none;
  border: none;
  color: var(--text-primary);
  outline: none;
}

.log-table {
  width: 100%;
  border-collapse: collapse;
}

.log-table th {
  padding: 1rem 1.5rem;
  text-align: left;
  font-size: 0.75rem;
  color: var(--text-secondary);
  border-bottom: 1px solid var(--glass-border);
}

.log-table td {
  padding: 1rem 1.5rem;
  font-size: 0.875rem;
  border-bottom: 1px solid var(--glass-border);
}

.id-tag {
  color: var(--accent-primary);
  font-weight: 600;
}

.type-badge {
  font-size: 0.75rem;
  padding: 2px 8px;
  background: rgba(0, 0, 0, 0.05);
  color: var(--text-secondary);
  border-radius: 4px;
}

.type-badge.vendor {
  background: rgba(var(--accent-primary-rgb), 0.1);
  color: var(--accent-primary);
}

.time-cell {
  line-height: 1.2;
}

.time-cell .date {
  display: block;
  font-size: 0.75rem;
  color: var(--text-secondary);
}

.token-viz {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.mini-bar {
  display: flex;
  height: 4px;
  background: var(--bg-primary);
  border-radius: 2px;
  overflow: hidden;
  width: 100px;
}

.mini-bar .req { background: var(--accent-secondary); }
.mini-bar .res { background: var(--accent-primary); }

.slow { color: var(--warning); font-weight: 600; }

.status-indicator {
  display: inline-block;
  width: 8px;
  height: 8px;
  border-radius: 50%;
  margin-right: 0.5rem;
}

.status-indicator.success { background: var(--success); }
.status-indicator.error { background: var(--error); }

.icon-btn {
  padding: 0.4rem;
  border-radius: 4px;
}

.icon-btn:hover { background: var(--bg-secondary); }

/* Modal Styles */
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.4);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal-content {
  width: 90%;
  max-width: 600px;
  max-height: 80vh;
  display: flex;
  flex-direction: column;
  box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 10px 10px -5px rgba(0, 0, 0, 0.04);
}

.modal-header {
  padding: 1.25rem 1.5rem;
  border-bottom: 1px solid var(--glass-border);
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.modal-header h3 {
  margin: 0;
  font-size: 1.125rem;
  font-weight: 600;
}

.close-btn {
  background: none;
  border: none;
  color: var(--text-secondary);
  cursor: pointer;
  padding: 4px;
  border-radius: 4px;
  transition: var(--transition);
}

.close-btn:hover {
  background: rgba(0, 0, 0, 0.05);
  color: var(--text-primary);
}

.modal-body {
  padding: 1.5rem;
  overflow-y: auto;
}

.json-viewer {
  background: #0d1117;
  color: #e6edf3;
  padding: 1rem;
  border-radius: 8px;
  font-size: 0.875rem;
  line-height: 1.5;
  overflow-x: auto;
  border: 1px solid rgba(255, 255, 255, 0.1);
}
</style>
