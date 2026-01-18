<template>
  <div class="users-page">
    <div class="action-bar glass">
      <div class="search-box">
        <Search :size="18" />
        <input v-model="searchQuery" type="text" placeholder="搜索用户名或 ID..." />
      </div>
      <button class="btn primary" @click="showAddModal = true">
        <UserPlus :size="18" />
        添加用户
      </button>
    </div>

    <div class="table-container glass">
      <table class="user-table">
        <thead>
          <tr>
            <th>用户</th>
            <th>API Key</th>
            <th>状态</th>
            <th>配额 (Tokens)</th>
            <th>速率 (RPM)</th>
            <th>创建时间</th>
            <th>操作</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="user in filteredUsers" :key="user.id">
            <td>
              <div class="user-cell">
                <span class="username">{{ user.username }}</span>
                <span class="user-id">{{ user.id.substring(0, 8) }}...</span>
              </div>
            </td>
            <td>
              <div class="key-cell">
                <code>{{ showKeys[user.id] ? user.api_key : '••••••••••••' }}</code>
                <button @click="toggleKey(user.id)">
                  <Eye v-if="!showKeys[user.id]" :size="14" />
                  <EyeOff v-else :size="14" />
                </button>
              </div>
            </td>
            <td>
              <span class="status-tag" :class="user.status.toLowerCase()">
                {{ user.status }}
              </span>
            </td>
            <td>
              <div class="quota-cell">
                <div class="progress-bar">
                  <div class="progress" :style="{ width: (user.token_used / user.token_quota * 100) + '%' }"></div>
                </div>
                <span>{{ (user.token_used / 1000).toFixed(1) }}k / {{ (user.token_quota / 1000).toFixed(1) }}k</span>
              </div>
            </td>
            <td>{{ user.rpm_limit }}</td>
            <td>{{ new Date(user.created_at).toLocaleDateString() }}</td>
            <td>
              <div class="actions">
                <button class="icon-btn" title="编辑配额" @click="editQuota(user)">
                  <Settings2 :size="16" />
                </button>
                <button class="icon-btn delete" title="禁用用户">
                  <UserX :size="16" />
                </button>
              </div>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- Edit Quota Modal -->
    <div v-if="editingUser" class="modal-overlay" @click.self="editingUser = null">
      <div class="modal glass">
        <h3>管理用户配额: {{ editingUser.username }}</h3>
        <div class="form-group">
          <label>RPM 限制</label>
          <input v-model="quotaForm.rpm_limit" type="number" />
        </div>
        <div class="form-group">
          <label>总 Token 配额</label>
          <input v-model="quotaForm.token_quota" type="number" />
        </div>
        <div class="modal-actions">
          <button class="btn secondary" @click="editingUser = null">取消</button>
          <button class="btn primary" @click="saveQuota">保存更改</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { 
  Search, 
  UserPlus, 
  Eye, 
  EyeOff, 
  Settings2, 
  UserX 
} from 'lucide-vue-next'

const searchQuery = ref('')
const showAddModal = ref(false)
const showKeys = reactive({})
const editingUser = ref(null)
const quotaForm = reactive({
  rpm_limit: 0,
  token_quota: 0
})

// Mock data (replace with actual fetch)
const users = ref([
  {
    id: 'user_12345678',
    username: 'alice_dev',
    api_key: 'sk-lowart-abc123xyz789',
    status: 'Active',
    token_quota: 1000000,
    token_used: 450000,
    rpm_limit: 60,
    created_at: '2024-01-10T08:00:00Z'
  },
  {
    id: 'user_88889999',
    username: 'bob_research',
    api_key: 'sk-lowart-def456uvw012',
    status: 'Blocked',
    token_quota: 500000,
    token_used: 500000,
    rpm_limit: 30,
    created_at: '2024-01-12T10:30:00Z'
  }
])

const filteredUsers = computed(() => {
  if (!searchQuery.value) return users.value
  const q = searchQuery.value.toLowerCase()
  return users.value.filter(u => 
    u.username.toLowerCase().includes(q) || u.id.toLowerCase().includes(q)
  )
})

const toggleKey = (id) => {
  showKeys[id] = !showKeys[id]
}

const editQuota = (user) => {
  editingUser.value = user
  quotaForm.rpm_limit = user.rpm_limit
  quotaForm.token_quota = user.token_quota
}

const saveQuota = async () => {
  console.log('Saving quota for', editingUser.value.id, quotaForm)
  // TODO: API Call
  editingUser.value = null
}
</script>

<style scoped>
.action-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 1rem 1.5rem;
  margin-bottom: 1.5rem;
}

.search-box {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  background: var(--bg-primary);
  padding: 0.5rem 1rem;
  border-radius: 8px;
  width: 320px;
  border: 1px solid var(--glass-border);
}

.search-box input {
  background: none;
  border: none;
  color: var(--text-primary);
  width: 100%;
  outline: none;
}

.btn {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.625rem 1.25rem;
  border-radius: 8px;
  font-weight: 600;
  transition: var(--transition);
}

.btn.primary {
  background: var(--accent-primary);
  color: white;
}

.btn.primary:hover {
  filter: brightness(1.1);
  box-shadow: 0 4px 12px rgba(56, 189, 248, 0.4);
}

.btn.secondary {
  background: var(--bg-secondary);
  color: var(--text-primary);
}

.table-container {
  overflow: hidden;
}

.user-table {
  width: 100%;
  border-collapse: collapse;
  text-align: left;
}

.user-table th {
  padding: 1.25rem 1.5rem;
  font-size: 0.75rem;
  text-transform: uppercase;
  color: var(--text-secondary);
  border-bottom: 1px solid var(--glass-border);
}

.user-table td {
  padding: 1rem 1.5rem;
  border-bottom: 1px solid var(--glass-border);
}

.user-cell .username {
  display: block;
  font-weight: 600;
}

.user-cell .user-id {
  display: block;
  font-size: 0.75rem;
  color: var(--text-secondary);
}

.key-cell {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.key-cell code {
  background: var(--bg-primary);
  padding: 0.25rem 0.5rem;
  border-radius: 4px;
  font-family: monospace;
}

.status-tag {
  padding: 0.25rem 0.625rem;
  border-radius: 20px;
  font-size: 0.75rem;
  font-weight: 600;
}

.status-tag.active { background: rgba(16, 185, 129, 0.1); color: var(--success); }
.status-tag.blocked { background: rgba(239, 68, 68, 0.1); color: var(--error); }

.quota-cell {
  width: 160px;
}

.progress-bar {
  height: 6px;
  background: var(--bg-primary);
  border-radius: 3px;
  margin-bottom: 0.5rem;
  overflow: hidden;
}

.progress {
  height: 100%;
  background: var(--accent-primary);
  border-radius: 3px;
}

.quota-cell span {
  font-size: 0.75rem;
  color: var(--text-secondary);
}

.actions {
  display: flex;
  gap: 0.5rem;
}

.icon-btn {
  padding: 0.5rem;
  border-radius: 6px;
  color: var(--text-secondary);
  transition: var(--transition);
}

.icon-btn:hover {
  background: var(--bg-secondary);
  color: var(--text-primary);
}

.icon-btn.delete:hover {
  color: var(--error);
  background: rgba(239, 68, 68, 0.1);
}

/* Modal Styles */
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  backdrop-filter: blur(4px);
}

.modal {
  width: 400px;
  padding: 2rem;
}

.modal h3 {
  margin-bottom: 1.5rem;
}

.form-group {
  margin-bottom: 1.25rem;
}

.form-group label {
  display: block;
  font-size: 0.875rem;
  margin-bottom: 0.5rem;
  color: var(--text-secondary);
}

.form-group input {
  width: 100%;
  background: var(--bg-primary);
  border: 1px solid var(--glass-border);
  border-radius: 8px;
  padding: 0.625rem 1rem;
  color: var(--text-primary);
  outline: none;
}

.form-group input:focus {
  border-color: var(--accent-primary);
}

.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 1rem;
  margin-top: 2rem;
}
</style>
