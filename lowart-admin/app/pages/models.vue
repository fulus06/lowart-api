<template>
  <div class="models-page">
    <div class="action-bar glass">
      <div class="header-info">
        <h3>可用模型负载</h3>
        <span class="count">{{ models.length }} 个活跃模型</span>
      </div>
      <button class="btn primary" @click="showAddModal = true">
        <Plus :size="18" />
        注册新模型
      </button>
    </div>

    <div class="models-grid">
      <div v-for="model in models" :key="model.id" class="model-card glass">
        <div class="card-header">
          <div class="vendor-icon" :class="model.vendor_type.toLowerCase()">
            {{ model.vendor_type[0] }}
          </div>
          <div class="title-area">
            <h4>{{ model.title }}</h4>
            <span class="model-id">{{ model.model_id }}</span>
          </div>
          <div class="status-toggle">
            <span class="dot" :class="{ active: model.is_active }"></span>
          </div>
        </div>

        <div class="card-body">
          <div class="info-row">
            <span class="label">供应商:</span>
            <span class="value">{{ model.vendor_type }}</span>
          </div>
          <div class="info-row">
            <span class="label">Endpoint:</span>
            <span class="value truncate">{{ model.base_url }}</span>
          </div>
          <div class="info-row">
            <span class="label">成本 (/1k):</span>
            <span class="value">¥{{ (model.cost_per_1k_tokens / 100).toFixed(2) }}</span>
          </div>
        </div>

        <div class="card-footer">
          <button class="action-link" @click="configureFallback(model)">
            <GitBranch :size="14" />
            配置降级
          </button>
          <div class="main-actions">
            <button class="icon-btn"><Edit3 :size="16" /></button>
            <button class="icon-btn delete"><Trash2 :size="16" /></button>
          </div>
        </div>
      </div>
    </div>

    <!-- Fallback Configuration Modal -->
    <div v-if="configuringFallback" class="modal-overlay" @click.self="configuringFallback = null">
      <div class="modal glass large">
        <h3>降级策略配置: {{ configuringFallback.model_id }}</h3>
        <p class="subtitle">当主模型发生熔断或报错时，自动切换至以下备选方案：</p>
        
        <div class="fallback-list">
          <div v-for="(fb, index) in currentFallbacks" :key="fb.id" class="fallback-item glass">
            <div class="order">#{{ index + 1 }}</div>
            <div class="fb-info">
              <span class="fb-name">{{ fb.fallback_model_id }}</span>
              <span class="fb-condition">触发条件: {{ fb.trigger_condition }}</span>
            </div>
            <button class="icon-btn delete"><X :size="14" /></button>
          </div>
          
          <button class="add-fallback-btn">
            <Plus :size="14" />
            添加降级模型
          </button>
        </div>

        <div class="modal-actions">
          <button class="btn secondary" @click="configuringFallback = null">关闭</button>
          <button class="btn primary">保存链条</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { 
  Plus, 
  GitBranch, 
  Edit3, 
  Trash2, 
  X 
} from 'lucide-vue-next'

const { getModels } = useApi()
const models = ref([])
const isLoading = ref(false)
const configuringFallback = ref(null)
const currentFallbacks = ref([])

const loadModels = async () => {
  isLoading.value = true
  try {
    const data = await getModels()
    models.value = data
  } catch (e) {
    console.error('Failed to load models:', e)
  } finally {
    isLoading.value = false
  }
}

onMounted(loadModels)

const configureFallback = (model) => {
  configuringFallback.value = model
  // Mock fallback data for now as specific endpoint might not be ready
  currentFallbacks.value = [
    { id: 'f1', fallback_model_id: 'gpt-4-turbo', trigger_condition: 'error/timeout' },
    { id: 'f2', fallback_model_id: 'gpt-3.5-turbo', trigger_condition: 'always' }
  ]
}
</script>

<style scoped>
.action-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 1.25rem 1.5rem;
  margin-bottom: 2rem;
}

.header-info h3 { margin-bottom: 0.25rem; }
.header-info .count { font-size: 0.875rem; color: var(--text-secondary); }

.models-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
  gap: 1.5rem;
}

.model-card {
  padding: 1.5rem;
  display: flex;
  flex-direction: column;
}

.card-header {
  display: flex;
  align-items: center;
  gap: 1rem;
  margin-bottom: 1.5rem;
}

.vendor-icon {
  width: 40px;
  height: 40px;
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 700;
  font-size: 1.125rem;
}

.vendor-icon.openai { background: #10a37f; color: white; }
.vendor-icon.anthropic { background: #d97757; color: white; }
.vendor-icon.comfyui { background: #6366f1; color: white; }

.title-area h4 { font-size: 1rem; }
.title-area .model_id { font-size: 0.75rem; color: var(--text-secondary); }

.status-toggle {
  margin-left: auto;
}

.dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: var(--bg-secondary);
}

.dot.active {
  background: var(--success);
  box-shadow: 0 0 8px var(--success);
}

.card-body {
  flex: 1;
  margin-bottom: 1.5rem;
}

.info-row {
  display: flex;
  justify-content: space-between;
  font-size: 0.875rem;
  margin-bottom: 0.5rem;
}

.info-row .label { color: var(--text-secondary); }
.info-row .value.truncate {
  max-width: 180px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.card-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding-top: 1rem;
  border-top: 1px solid var(--glass-border);
}

.action-link {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  font-size: 0.75rem;
  color: var(--accent-primary);
  font-weight: 600;
}

.main-actions {
  display: flex;
  gap: 0.5rem;
}

/* Fallback Modal Specific */
.modal.large { width: 500px; }
.subtitle { font-size: 0.875rem; color: var(--text-secondary); margin-bottom: 1.5rem; }

.fallback-list {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.fallback-item {
  display: flex;
  align-items: center;
  gap: 1rem;
  padding: 0.75rem 1rem;
}

.fallback-item .order {
  font-size: 0.875rem;
  font-weight: 700;
  color: var(--accent-primary);
}

.fb-info {
  flex: 1;
}

.fb-info .fb-name { display: block; font-weight: 600; font-size: 0.875rem; }
.fb-info .fb-condition { display: block; font-size: 0.75rem; color: var(--text-secondary); }

.add-fallback-btn {
  width: 100%;
  padding: 0.75rem;
  border: 1px dashed var(--glass-border);
  border-radius: 8px;
  color: var(--text-secondary);
  font-size: 0.875rem;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  margin-top: 0.5rem;
  transition: var(--transition);
}

.add-fallback-btn:hover {
  background: rgba(0, 0, 0, 0.02);
  border-color: var(--accent-primary);
  color: var(--text-primary);
}
</style>
