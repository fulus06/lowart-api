<template>
  <div class="users-page">
    <div class="action-bar glass">
      <div class="search-box">
        <Search :size="18" />
        <input v-model="searchQuery" type="text" placeholder="搜索用户名或 ID..." />
      </div>
      <button class="btn primary" @click="openAddModal">
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
            <th class="hide-mobile">创建时间</th>
            <th>操作</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="user in filteredUsers" :key="user.id">
            <td>
              <div class="user-cell">
                <span class="username">
                  {{ user.username }}
                  <span v-if="user.is_admin" class="admin-badge">Admin</span>
                </span>
                <span class="user-id">{{ user.id.substring(0, 8) }}...</span>
              </div>
            </td>
            <td>
              <div class="key-cell">
                <code>{{ showKeys[user.id] ? user.api_key : '••••••••••••' }}</code>
                <div class="key-actions">
                  <button title="查看/隐藏" @click="toggleKey(user.id)">
                    <Eye v-if="!showKeys[user.id]" :size="14" />
                    <EyeOff v-else :size="14" />
                  </button>
                  <button title="复制 Key" @click="copyToClipboard(user.api_key, user.id)">
                    <Check v-if="copiedId === user.id" :size="14" class="text-success" />
                    <Copy v-else :size="14" />
                  </button>
                </div>
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
                  <div class="progress" :style="{ width: Math.min((user.token_used / user.token_quota * 100), 100) + '%' }"></div>
                </div>
                <span>{{ (user.token_used / 1000).toFixed(1) }}k / {{ (user.token_quota / 1000).toFixed(1) }}k</span>
              </div>
            </td>
            <td>{{ user.rpm_limit }}</td>
            <td class="hide-mobile">{{ new Date(user.created_at).toLocaleDateString() }}</td>
            <td>
              <div class="actions">
                <button class="icon-btn" title="修改信息" @click="openEditModal(user)">
                  <UserCog :size="16" />
                </button>
                <button class="icon-btn" title="编辑配额" @click="editQuota(user)">
                  <Settings2 :size="16" />
                </button>
                <button 
                  class="icon-btn delete" 
                  title="删除用户" 
                  :disabled="user.id === 'admin_root_id' || user.id === authStore.currentUser?.id"
                  @click="confirmDelete(user)"
                >
                  <Trash2 :size="16" />
                </button>
              </div>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- User Modal (Add / Edit Info) -->
    <div v-if="showUserModal" class="modal-overlay" @click.self="showUserModal = false">
      <div class="modal glass">
        <h3>{{ isEditing ? '编辑用户' : '添加新用户' }}</h3>
        <div class="form-group">
          <label>用户名</label>
          <input v-model="userForm.username" type="text" placeholder="唯一用户名" />
        </div>
        <div class="form-group">
          <label>API Key</label>
          <div class="input-with-action">
            <input v-model="userForm.api_key" type="text" placeholder="sk-..." />
            <button class="btn secondary sm" @click="generateKey">生成</button>
          </div>
        </div>
        <div v-if="!isEditing" class="form-group checkbox">
          <label>
            <input v-model="userForm.is_admin" type="checkbox" />
            设为管理员
          </label>
        </div>
        <div v-else class="form-group">
          <label>状态</label>
          <select v-model="userForm.status">
            <option value="Active">Active</option>
            <option value="Inactive">Inactive</option>
            <option value="Blocked">Blocked</option>
          </select>
        </div>
        <div v-if="modalError" class="error-msg">{{ modalError }}</div>
        <div class="modal-actions">
          <button class="btn secondary" @click="showUserModal = false">取消</button>
          <button class="btn primary" :disabled="isSubmitting" @click="saveUser">
            {{ isSubmitting ? '正在保存...' : '确认' }}
          </button>
        </div>
      </div>
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
  UserCog,
  Trash2,
  Copy,
  Check
} from 'lucide-vue-next'

const searchQuery = ref('')
const showKeys = reactive({})
const isLoading = ref(false)
const isSubmitting = ref(false)
const modalError = ref('')

// User Modal State
const showUserModal = ref(false)
const isEditing = ref(false)
const currentUserId = ref(null)
const userForm = reactive({
  username: '',
  api_key: '',
  is_admin: false,
  status: 'Active'
})

// Quota Modal State
const editingUser = ref(null)
const quotaForm = reactive({
  rpm_limit: 0,
  token_quota: 0
})

const { getUsers, updateQuota, createUser, updateUser, deleteUser } = useApi()
const authStore = useAuthStore()
const users = ref([])
const copiedId = ref(null)

const loadUsers = async () => {
  isLoading.value = true
  try {
    users.value = await getUsers()
  } catch (e) {
    console.error('Failed to load users:', e)
  } finally {
    isLoading.value = false
  }
}

onMounted(loadUsers)

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

const copyToClipboard = async (text, id) => {
  try {
    await navigator.clipboard.writeText(text)
    copiedId.value = id
    setTimeout(() => {
      if (copiedId.value === id) copiedId.value = null
    }, 2000)
  } catch (err) {
    console.error('Failed to copy keys:', err)
  }
}

const generateKey = () => {
  userForm.api_key = 'sk-' + Math.random().toString(36).substring(2, 15) + Math.random().toString(36).substring(2, 15)
}

const openAddModal = () => {
  isEditing.value = false
  userForm.username = ''
  userForm.api_key = ''
  userForm.is_admin = false
  userForm.status = 'Active'
  modalError.value = ''
  showUserModal.value = true
  generateKey()
}

const openEditModal = (user) => {
  isEditing.value = true
  currentUserId.value = user.id
  userForm.username = user.username
  userForm.api_key = user.api_key
  userForm.status = user.status
  modalError.value = ''
  showUserModal.value = true
}

const saveUser = async () => {
  if (!userForm.username || !userForm.api_key) {
    modalError.value = '请填写完整信息'
    return
  }

  isSubmitting.value = true
  modalError.value = ''
  try {
    if (isEditing.value) {
      await updateUser({
        user_id: currentUserId.value,
        username: userForm.username,
        api_key: userForm.api_key,
        status: userForm.status
      })
    } else {
      await createUser({
        username: userForm.username,
        api_key: userForm.api_key,
        is_admin: userForm.is_admin
      })
    }
    await loadUsers()
    showUserModal.value = false
  } catch (e) {
    if (e.status === 409) {
      modalError.value = '用户名已存在'
    } else {
      modalError.value = '保存失败: ' + (e.data || e.message)
    }
  } finally {
    isSubmitting.value = false
  }
}

const confirmDelete = async (user) => {
  if (user.id === 'admin_root_id') return
  if (confirm(`确定要删除用户 "${user.username}" 吗？此操作不可撤销。`)) {
    try {
      await deleteUser(user.id)
      await loadUsers()
    } catch (e) {
      alert('删除失败: ' + (e.data || e.message))
    }
  }
}

const editQuota = (user) => {
  editingUser.value = user
  quotaForm.rpm_limit = user.rpm_limit
  quotaForm.token_quota = user.token_quota
}

const saveQuota = async () => {
  if (!editingUser.value) return
  try {
    await updateQuota({
      user_id: editingUser.value.id,
      rpm_limit: quotaForm.rpm_limit,
      token_quota: quotaForm.token_quota
    })
    await loadUsers()
    editingUser.value = null
  } catch (e) {
    alert('更新失败: ' + (e.data || e.message))
  }
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

.user-cell .admin-badge {
  font-size: 0.625rem;
  background: var(--accent-primary);
  color: white;
  padding: 0.1rem 0.4rem;
  border-radius: 4px;
  vertical-align: middle;
  margin-left: 0.5rem;
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

.key-actions {
  display: flex;
  gap: 0.25rem;
}

.text-success {
  color: var(--success);
}

.status-tag {
  padding: 0.25rem 0.625rem;
  border-radius: 20px;
  font-size: 0.75rem;
  font-weight: 600;
}

.status-tag.active { background: rgba(16, 185, 129, 0.1); color: var(--success); }
.status-tag.inactive { background: rgba(100, 116, 139, 0.1); color: var(--text-secondary); }
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

.icon-btn:disabled {
  opacity: 0.3;
  cursor: not-allowed;
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
  width: 440px;
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

.form-group input, .form-group select {
  width: 100%;
  background: var(--bg-primary);
  border: 1px solid var(--glass-border);
  border-radius: 8px;
  padding: 0.625rem 1rem;
  color: var(--text-primary);
  outline: none;
}

.form-group input:focus, .form-group select:focus {
  border-color: var(--accent-primary);
}

.form-group.checkbox label {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  cursor: pointer;
  color: var(--text-primary);
}

.form-group.checkbox input {
  width: auto;
}

.input-with-action {
  display: flex;
  gap: 0.5rem;
}

.btn.sm {
  padding: 0.4rem 0.8rem;
  font-size: 0.75rem;
}

.error-msg {
  color: var(--error);
  font-size: 0.8125rem;
  margin-top: 1rem;
  padding: 0.5rem;
  background: rgba(239, 68, 68, 0.05);
  border-radius: 4px;
}

.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 1rem;
  margin-top: 2rem;
}

@media (max-width: 768px) {
  .hide-mobile {
    display: none;
  }
}
</style>
