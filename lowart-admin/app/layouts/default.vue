<template>
  <div class="app-container">
    <aside class="sidebar glass">
      <div class="logo">
        <LayoutDashboard :size="24" />
        <span>Lowart Admin</span>
      </div>
      <nav class="nav-links">
        <NuxtLink to="/" class="nav-item" :class="{ active: route.path === '/' }">
          <BarChart3 :size="20" />
          <span>仪表盘</span>
        </NuxtLink>
        <NuxtLink to="/users" class="nav-item" :class="{ active: route.path === '/users' }">
          <Users :size="20" />
          <span>用户管理</span>
        </NuxtLink>
        <NuxtLink to="/models" class="nav-item" :class="{ active: route.path === '/models' }">
          <Box :size="20" />
          <span>模型管理</span>
        </NuxtLink>
        <NuxtLink to="/logs" class="nav-item" :class="{ active: route.path === '/logs' }">
          <History :size="20" />
          <span>调用日志</span>
        </NuxtLink>
        <NuxtLink to="/chat" class="nav-item" :class="{ active: route.path === '/chat' }">
          <MessageSquare :size="20" />
          <span>聊天测试</span>
        </NuxtLink>
      </nav>
      <div class="sidebar-footer">
        <div class="user-info">
          <div class="avatar">A</div>
          <div class="details">
            <span class="name">Administrator</span>
            <span class="role">Admin Key Loaded</span>
          </div>
        </div>
      </div>
    </aside>

    <main class="main-content">
      <header class="top-header glass">
        <div class="header-left">
          <h1>{{ pageTitle }}</h1>
        </div>
        <div class="header-right">
          <div class="status-badge" :class="{ online: isOnline }">
            <span class="dot"></span>
            {{ isOnline ? 'Backend Online' : 'Backend Offline' }}
          </div>
        </div>
      </header>
      <div class="page-body">
        <slot />
      </div>
    </main>
  </div>
</template>

<script setup>
import { 
  LayoutDashboard, 
  Users, 
  Box, 
  History, 
  MessageSquare,
  BarChart3
} from 'lucide-vue-next'

const route = useRoute()
const isOnline = ref(true)

const pageTitle = computed(() => {
  const titles = {
    '/': '仪表盘概览',
    '/users': '用户管理',
    '/models': '模型管理',
    '/logs': '调用记录',
    '/chat': '模型对话测试'
  }
  return titles[route.path] || 'Unknown'
})
</script>

<style scoped>
.app-container {
  display: flex;
  min-height: 100vh;
}

.sidebar {
  width: var(--sidebar-width);
  height: 100vh;
  position: fixed;
  left: 0;
  top: 0;
  display: flex;
  flex-direction: column;
  padding: 1.5rem;
  z-index: 100;
  border-radius: 0;
  border-left: none;
  border-top: none;
  border-bottom: none;
}

.logo {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  font-size: 1.25rem;
  font-weight: 700;
  margin-bottom: 2.5rem;
  color: var(--accent-primary);
}

.nav-links {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  flex: 1;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.75rem 1rem;
  border-radius: 8px;
  color: var(--text-secondary);
  transition: var(--transition);
}

.nav-item:hover {
  background: rgba(0, 0, 0, 0.05);
  color: var(--text-primary);
}

.nav-item.active {
  background: var(--accent-primary);
  color: white;
  box-shadow: 0 4px 12px rgba(56, 189, 248, 0.3);
}

.sidebar-footer {
  padding-top: 1.5rem;
  border-top: 1px solid var(--glass-border);
}

.user-info {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.avatar {
  width: 36px;
  height: 36px;
  background: var(--accent-secondary);
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 600;
}

.details .name {
  display: block;
  font-size: 0.875rem;
  font-weight: 600;
}

.details .role {
  display: block;
  font-size: 0.75rem;
  color: var(--text-secondary);
}

.main-content {
  flex: 1;
  margin-left: var(--sidebar-width);
  padding: 1.5rem;
}

.top-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1rem 1.5rem;
  margin-bottom: 2rem;
  border-radius: var(--border-radius);
}

.header-left h1 {
  font-size: 1.5rem;
}

.status-badge {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.375rem 0.75rem;
  background: rgba(239, 68, 68, 0.1);
  color: var(--error);
  border-radius: 20px;
  font-size: 0.75rem;
  font-weight: 600;
}

.status-badge.online {
  background: rgba(16, 185, 129, 0.1);
  color: var(--success);
}

.dot {
  width: 8px;
  height: 8px;
  background: currentColor;
  border-radius: 50%;
}

.page-body {
  animation: fadeIn 0.4s ease-out;
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateY(10px); }
  to { opacity: 1; transform: translateY(0); }
}
</style>
