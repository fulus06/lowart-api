<template>
  <div class="models-page">
    <div class="action-bar glass">
      <div class="header-info">
        <h3>可用模型负载</h3>
        <span class="count">{{ models.length }} 个活跃模型</span>
      </div>
      <button class="btn primary" @click="openAddModal">
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
            <span class="model_id">{{ model.model_id }}</span>
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
            <span class="value truncate" :title="model.base_url">{{ model.base_url }}</span>
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
            <button class="icon-btn" title="编辑模型" @click="openEditModal(model)"><Edit3 :size="16" /></button>
            <button class="icon-btn delete" title="删除模型" @click="confirmDeleteModel(model)"><Trash2 :size="16" /></button>
          </div>
        </div>
      </div>
    </div>

    <!-- Registration / Edit Modal -->
    <div v-if="showModelModal" class="modal-overlay" @click.self="showModelModal = false">
      <div class="modal glass">
        <h3>{{ isEditing ? '编辑模型配置' : '注册新模型负载' }}</h3>
        
        <div class="form-container">
          <div class="form-group">
            <label>显示名称 (Title)</label>
            <input v-model="modelForm.title" placeholder="例如: GPT-4o" />
          </div>

          <div class="form-group">
            <label>模型标识 (Model ID)</label>
            <input v-model="modelForm.model_id" placeholder="例如: gpt-4o" />
          </div>

          <div class="form-grid">
            <div class="form-group">
              <label>供应商类型</label>
              <select v-model="modelForm.vendor_type">
                <option>OpenAI</option>
                <option>Anthropic</option>
                <option>ComfyUI</option>
                <option>Custom</option>
              </select>
            </div>
            <div class="form-group">
              <label>单位成本 (1k tokens)</label>
              <input v-model.number="modelForm.cost_per_1k_tokens" type="number" step="0.01" />
            </div>
          </div>

          <div class="form-group">
            <label>Base URL</label>
            <input v-model="modelForm.base_url" placeholder="https://api.openai.com/v1" />
          </div>

          <div class="form-group">
            <label>API Key</label>
            <input v-model="modelForm.api_key" type="password" placeholder="sk-..." />
          </div>

          <div class="form-group checkbox">
            <label>
              <input v-model="modelForm.is_active" type="checkbox" />
              启用此模型负载
            </label>
          </div>

          <div v-if="errorMsg" class="error-msg">{{ errorMsg }}</div>
        </div>

        <div class="modal-actions">
          <button class="btn secondary" @click="showModelModal = false">取消</button>
          <button class="btn primary" :disabled="isSaving" @click="saveModel">
            {{ isSaving ? '提交中...' : '确认保存' }}
          </button>
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

const { getModels, createModel, updateModel, deleteModel } = useApi()
const models = ref([])
const isLoading = ref(false)
const configuringFallback = ref(null)
const currentFallbacks = ref([])

// Model Form Modal
const showModelModal = ref(false)
const isEditing = ref(false)
const isSaving = ref(false)
const errorMsg = ref('')
const modelForm = reactive({
  id: '',
  title: '',
  model_id: '',
  api_key: '',
  base_url: '',
  vendor_type: 'OpenAI',
  cost_per_1k_tokens: 0,
  is_active: true
})

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

const openAddModal = () => {
  isEditing.value = false
  errorMsg.value = ''
  Object.assign(modelForm, {
    id: '',
    title: '',
    model_id: '',
    api_key: '',
    base_url: 'https://api.openai.com/v1',
    vendor_type: 'OpenAI',
    cost_per_1k_tokens: 0.1,
    is_active: true
  })
  showModelModal.value = true
}

const openEditModal = (model) => {
  isEditing.value = true
  errorMsg.value = ''
  Object.assign(modelForm, {
    ...model,
    cost_per_1k_tokens: model.cost_per_1k_tokens / 100 // Convert back to yuan for editing
  })
  showModelModal.value = true
}

const saveModel = async () => {
  if (!modelForm.title || !modelForm.model_id) {
    errorMsg.value = '标题和模型 ID 是必填项'
    return
  }

  isSaving.value = true
  errorMsg.value = ''
  try {
    const payload = {
      ...modelForm,
      cost_per_1k_tokens: Math.round(modelForm.cost_per_1k_tokens * 100) // Convert to cents
    }

    if (isEditing.value) {
      await updateModel(payload)
    } else {
      await createModel(payload)
    }
    
    await loadModels()
    showModelModal.value = false
  } catch (e) {
    errorMsg.value = e.message || '保存失败'
  } finally {
    isSaving.value = false
  }
}

const confirmDeleteModel = async (model) => {
  if (confirm(`确定要删除模型 "${model.title}" 吗？此操作不可撤销。`)) {
    try {
      await deleteModel(model.id)
      await loadModels()
    } catch (e) {
      alert('删除失败: ' + e.message)
    }
  }
}

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

/* Registration / Edit Modal */
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
  border-radius: 16px;
  position: relative;
}

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

/* Modal Form Styles */
.form-container {
  display: flex;
  flex-direction: column;
  gap: 1.25rem;
  margin-top: 1.5rem;
}

.form-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 1rem;
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
  font-size: 0.875rem;
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

.error-msg {
  color: var(--error);
  font-size: 0.8125rem;
  margin-top: 0.5rem;
  padding: 0.5rem;
  background: rgba(239, 68, 68, 0.05);
  border-radius: 4px;
}
</style>
