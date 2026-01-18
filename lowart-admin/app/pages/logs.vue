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
          <option v-for="m in models" :key="m" :value="m">{{ m }}</option>
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
            <th>时间</th>
            <th>用户</th>
            <th>模型</th>
            <th>Token (Req/Res)</th>
            <th>耗时 (ms)</th>
            <th>状态</th>
            <th>操作</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="log in filteredLogs" :key="log.id">
            <td class="time-cell">
              {{ new Date(log.timestamp).toLocaleTimeString() }}
              <span class="date">{{ new Date(log.timestamp).toLocaleDateString() }}</span>
            </td>
            <td>{{ log.user_id }}</td>
            <td><code>{{ log.model_id }}</code></td>
            <td>
              <div class="token-viz">
                <span>{{ log.req_tokens }} / {{ log.res_tokens }}</span>
                <div class="mini-bar">
                  <div class="req" :style="{ width: (log.req_tokens / 10) + 'px' }"></div>
                  <div class="res" :style="{ width: (log.res_tokens / 10) + 'px' }"></div>
                </div>
              </div>
            </td>
            <td :class="{ slow: log.duration > 2000 }">{{ log.duration }}ms</td>
            <td>
              <span class="status-indicator" :class="log.status.toLowerCase()"></span>
              {{ log.status }}
            </td>
            <td>
              <button class="icon-btn"><ArrowUpRight :size="14" /></button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>

<script setup>
import { Search, ArrowUpRight } from 'lucide-vue-next'

const users = ['alice_dev', 'bob_research', 'system_cron']
const models = ['gpt-4o', 'claude-3-5-sonnet', 'sdxl-v1']

const filters = reactive({
  user: '',
  model: '',
  search: ''
})

const logs = ref([
  {
    id: 'req_1',
    timestamp: '2024-01-18T12:00:05Z',
    user_id: 'alice_dev',
    model_id: 'gpt-4o',
    req_tokens: 156,
    res_tokens: 842,
    duration: 1240,
    status: 'Success'
  },
  {
    id: 'req_2',
    timestamp: '2024-01-18T12:05:12Z',
    user_id: 'bob_research',
    model_id: 'sdxl-v1',
    req_tokens: 50,
    res_tokens: 0,
    duration: 4500,
    status: 'Success'
  },
  {
    id: 'req_3',
    timestamp: '2024-01-18T12:08:44Z',
    user_id: 'alice_dev',
    model_id: 'gpt-4o',
    req_tokens: 42,
    res_tokens: 12,
    duration: 150,
    status: 'Error'
  }
])

const filteredLogs = computed(() => {
  return logs.value.filter(l => {
    if (filters.user && l.user_id !== filters.user) return false
    if (filters.model && l.model_id !== filters.model) return false
    if (filters.search && !l.id.includes(filters.search)) return false
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
</style>
