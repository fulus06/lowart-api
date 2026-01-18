<template>
  <div class="login-container">
    <div class="login-card glass">
      <div class="header">
        <div class="logo">
          <LayoutDashboard :size="32" />
        </div>
        <h1>Lowart Admin</h1>
        <p>请输入管理员 API Key 以继续</p>
      </div>

      <form @submit.prevent="handleLogin" class="login-form">
        <div class="input-group">
          <label for="admin-key">Admin API Key</label>
          <div class="input-wrapper">
            <Lock :size="18" />
            <input 
              id="admin-key"
              v-model="keyInput" 
              type="password" 
              placeholder="sk-lowart-..." 
              required
            />
          </div>
        </div>

        <div v-if="errorMsg" class="error-msg">
          {{ errorMsg }}
        </div>

        <button type="submit" class="login-btn" :disabled="isLoading">
          <span v-if="!isLoading">登录控制台</span>
          <span v-else class="loader"></span>
        </button>
      </form>

      <div class="footer">
        <p>默认密钥通常在服务器初始化时生成</p>
      </div>
    </div>
  </div>
</template>

<script setup>
import { LayoutDashboard, Lock } from 'lucide-vue-next'

definePageMeta({
  layout: false
})

const authStore = useAuthStore()
const { login } = useApi()

const keyInput = ref('')
const isLoading = ref(false)
const errorMsg = ref('')

const handleLogin = async () => {
  if (!keyInput.value) return
  
  isLoading.value = true
  errorMsg.value = ''
  
  try {
    // Verify using the dedicated login endpoint
    const response = await login(keyInput.value)
    
    // If successful, set the key and user in store and redirect
    authStore.setAuth(keyInput.value, response.user)
    navigateTo('/')
  } catch (e) {
    authStore.logout()
    errorMsg.value = '无效的 API Key 或不是管理员账号'
    console.error('Login failed:', e)
  } finally {
    isLoading.value = false
  }
}
</script>

<style scoped>
.login-container {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 100vh;
  background: var(--bg-primary);
}

.login-card {
  width: 100%;
  max-width: 400px;
  padding: 2.5rem;
  text-align: center;
}

.header {
  margin-bottom: 2rem;
}

.logo {
  color: var(--accent-primary);
  margin-bottom: 1rem;
  display: flex;
  justify-content: center;
}

.header h1 {
  font-size: 1.75rem;
  margin-bottom: 0.5rem;
}

.header p {
  color: var(--text-secondary);
  font-size: 0.875rem;
}

.login-form {
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
  text-align: left;
}

.input-group label {
  display: block;
  font-size: 0.875rem;
  font-weight: 600;
  margin-bottom: 0.5rem;
}

.input-wrapper {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  background: var(--bg-secondary);
  border: 1px solid var(--glass-border);
  border-radius: 8px;
  padding: 0 1rem;
  transition: var(--transition);
}

.input-wrapper:focus-within {
  border-color: var(--accent-primary);
  box-shadow: 0 0 0 2px rgba(14, 165, 233, 0.2);
}

.input-wrapper input {
  flex: 1;
  background: none;
  border: none;
  height: 44px;
  color: var(--text-primary);
  outline: none;
  font-family: monospace;
}

.error-msg {
  color: var(--error);
  font-size: 0.8125rem;
  background: rgba(239, 68, 68, 0.1);
  padding: 0.5rem;
  border-radius: 6px;
  text-align: center;
}

.login-btn {
  background: var(--accent-primary);
  color: white;
  height: 44px;
  border-radius: 8px;
  font-weight: 600;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: var(--transition);
}

.login-btn:hover:not(:disabled) {
  filter: brightness(1.1);
}

.login-btn:disabled {
  opacity: 0.7;
  cursor: not-allowed;
}

.loader {
  width: 20px;
  height: 20px;
  border: 2px solid rgba(255, 255, 255, 0.3);
  border-radius: 50%;
  border-top-color: white;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.footer {
  margin-top: 2rem;
  color: var(--text-secondary);
  font-size: 0.75rem;
}
</style>
