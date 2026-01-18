<template>
  <div class="chat-page">
    <div class="chat-sidebar glass">
      <h3>配置测试</h3>
      <div class="form-group">
        <label>选择模型</label>
        <select v-model="config.model">
          <option v-for="m in models" :key="m" :value="m">{{ m }}</option>
        </select>
      </div>
      <div class="form-group">
        <label>系统提示词 (System)</label>
        <textarea v-model="config.system" placeholder="You are a helpful assistant..."></textarea>
      </div>
      <div class="settings">
        <div class="setting-row">
          <span>流式输出 (SSE)</span>
          <input v-model="config.stream" type="checkbox" />
        </div>
        <div class="setting-row">
          <span>Temperature</span>
          <input v-model="config.temperature" type="range" min="0" max="2" step="0.1" />
        </div>
      </div>
      <button class="btn secondary reset" @click="clearChat">
        <RotateCcw :size="16" /> 清空对话
      </button>
    </div>

    <div class="chat-main">
      <div class="message-list" ref="messageList">
        <div v-for="(msg, index) in messages" :key="index" class="message-wrapper" :class="msg.role">
          <div class="message glass">
            <div class="avatar">{{ msg.role[0].toUpperCase() }}</div>
            <div class="content">
              <div class="role-label">{{ msg.role === 'user' ? 'You' : 'Assistant' }}</div>
              <div class="text">{{ msg.content }}</div>
            </div>
          </div>
        </div>
        <div v-if="isTyping" class="message-wrapper assistant">
          <div class="message glass typing">
            <div class="avatar">A</div>
            <div class="content"><div class="dots"><span></span><span></span><span></span></div></div>
          </div>
        </div>
      </div>

      <div class="input-area glass">
        <textarea 
          v-model="userInput" 
          placeholder="输入消息，按 Enter 发送..." 
          @keydown.enter.prevent="sendMessage"
        ></textarea>
        <button class="send-btn" :disabled="!userInput || isTyping" @click="sendMessage">
          <Send :size="20" />
        </button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { Send, RotateCcw } from 'lucide-vue-next'

const { getModels, chat } = useApi()
const models = ref([])
const messageList = ref(null)

const config = reactive({
  model: '',
  system: '',
  stream: false,
  temperature: 0.7
})

const userInput = ref('')
const isTyping = ref(false)
const messages = ref([
  { role: 'assistant', content: '您好！我是 Lowart AI 测试助手。请选择一个模型开始测试。' }
])

const loadModels = async () => {
  try {
    const data = await getModels()
    models.value = data.map(m => m.model_id)
    if (models.value.length > 0) {
      config.model = models.value[0]
    }
  } catch (e) {
    console.error('Failed to load models:', e)
  }
}

onMounted(loadModels)

const sendMessage = async () => {
  if (!userInput.value || !config.model || isTyping.value) return

  const userMsg = userInput.value
  messages.value.push({ role: 'user', content: userMsg })
  userInput.value = ''
  isTyping.value = true

  // Scroll to bottom
  nextTick(() => {
    if (messageList.value) messageList.value.scrollTop = messageList.value.scrollHeight
  })

  try {
    const response = await chat({
      model: config.model,
      messages: [
        ...(config.system ? [{ role: 'system', content: config.system }] : []),
        ...messages.value.map(m => ({ role: m.role, content: m.content }))
      ],
      stream: false,
      temperature: config.temperature
    })

    const assistantMsg = response.choices[0].message.content
    messages.value.push({ role: 'assistant', content: assistantMsg })
  } catch (e) {
    console.error('Chat error:', e)
    messages.value.push({ role: 'assistant', content: `[错误] 无法获取模型响应: ${e.message}` })
  } finally {
    isTyping.value = false
    nextTick(() => {
      if (messageList.value) messageList.value.scrollTop = messageList.value.scrollHeight
    })
  }
}

const clearChat = () => {
  messages.value = [{ role: 'assistant', content: '对话已清空。' }]
}
</script>

<style scoped>
.chat-page {
  display: flex;
  height: calc(100vh - 160px);
  gap: 1.5rem;
}

.chat-sidebar {
  width: 280px;
  padding: 1.5rem;
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
}

.form-group label {
  display: block;
  font-size: 0.8125rem;
  color: var(--text-secondary);
  margin-bottom: 0.5rem;
}

.form-group select, .form-group textarea {
  width: 100%;
  background: var(--bg-primary);
  border: 1px solid var(--glass-border);
  border-radius: 8px;
  color: var(--text-primary);
  padding: 0.625rem;
  outline: none;
}

.form-group textarea {
  height: 100px;
  resize: none;
}

.settings {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.setting-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 0.8125rem;
}

.btn.reset {
  margin-top: auto;
  border: 1px solid var(--glass-border);
}

.chat-main {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 1rem;
  position: relative;
}

.message-list {
  flex: 1;
  overflow-y: auto;
  padding: 1rem;
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
}

.message-wrapper {
  display: flex;
  width: 100%;
}

.message-wrapper.user { justify-content: flex-end; }

.message {
  max-width: 80%;
  padding: 1rem;
  display: flex;
  gap: 1rem;
}

.message-wrapper.user .message {
  background: var(--accent-primary);
  border: none;
  color: white;
}

.avatar {
  width: 32px;
  height: 32px;
  border-radius: 8px;
  background: #f1f5f9;
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 700;
  flex-shrink: 0;
}

.message-wrapper.user .avatar { background: rgba(255, 255, 255, 0.3); }

.role-label {
  font-size: 0.75rem;
  font-weight: 700;
  margin-bottom: 0.25rem;
  opacity: 0.7;
}

.text {
  font-size: 0.9375rem;
  line-height: 1.6;
}

.input-area {
  padding: 1rem;
  display: flex;
  gap: 1rem;
  align-items: flex-end;
}

.input-area textarea {
  flex: 1;
  background: none;
  border: none;
  color: var(--text-primary);
  resize: none;
  height: 60px;
  padding: 0.5rem;
  outline: none;
}

.send-btn {
  width: 48px;
  height: 48px;
  background: var(--accent-primary);
  color: white;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: var(--transition);
}

.send-btn:disabled { opacity: 0.5; cursor: not-allowed; }
.send-btn:not(:disabled):hover { filter: brightness(1.1); transform: scale(1.05); }

/* Typing Animation */
.dots span {
  display: inline-block;
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--text-secondary);
  margin-right: 4px;
  animation: bounce 1.4s infinite ease-in-out;
}
.dots span:nth-child(1) { animation-delay: -0.32s; }
.dots span:nth-child(2) { animation-delay: -0.16s; }

@keyframes bounce {
  0%, 80%, 100% { transform: scale(0); }
  40% { transform: scale(1); }
}
</style>
