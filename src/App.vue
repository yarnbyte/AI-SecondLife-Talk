<script setup>
import { ref, onMounted, nextTick } from 'vue';
import { TauriBridge } from './services/TauriBridge';
import { LLMService } from './services/LLMService';
import { API_DEFAULTS } from './utils/constants';
import { Settings2, Play, Activity, MessageSquareDot, Send, KeyRound, Sparkles, X } from 'lucide-vue-next';

const isListening = ref(false);
// 带有模拟数据的对话以测试效果
const chatHistory = ref([
  { time: '12:00', sender: 'System', text: 'Tauri translation core initialized.', translated: 'Tauri 翻译引擎已就绪。' },
]);
const inputShow = ref(false);
const draftText = ref('');
const apiKey = ref('');
const inputRef = ref(null);

onMounted(async () => {
    TauriBridge.onLogUpdate(async (payload) => {
        if (payload && payload.includes(": ")) {
            const [sender, text] = payload.split(": ");
            const item = { time: new Date().toLocaleTimeString(), sender, text, translated: '' };
            chatHistory.value.push(item);
            
            LLMService.translateStream(text, [], (chunk) => {
                item.translated += chunk;
            }, {
                apiKey: apiKey.value,
                baseUrl: API_DEFAULTS.BASE_URL,
                model: API_DEFAULTS.MODEL,
                targetLang: 'Chinese'
            });
            scrollToBottom();
        }
    });

    TauriBridge.onShortcutTrigger(async () => {
        inputShow.value = true;
        await nextTick();
        inputRef.value?.focus();
    });
});

const startListening = async () => {
    if (!apiKey.value) {
        alert("请输入有效的 API Key 以获得最佳体验");
        return;
    }
    try {
        await TauriBridge.startLogWatcher();
        isListening.value = true;
    } catch(e) {
        alert("启动失败: " + e);
    }
};

const sendMyMessage = async () => {
    if (!draftText.value.trim()) return;
    
    const mySource = draftText.value;
    let translatedResult = '';
    
    LLMService.translateStream(mySource, [], (chunk) => {
        translatedResult += chunk;
    }, {
        apiKey: apiKey.value,
        baseUrl: API_DEFAULTS.BASE_URL,
        model: API_DEFAULTS.MODEL,
        targetLang: 'English'
    }).then(async () => {
        await TauriBridge.copyToClipboard(translatedResult);
        chatHistory.value.push({
            time: new Date().toLocaleTimeString(),
            sender: 'Me',
            text: mySource,
            translated: translatedResult
        });
        inputShow.value = false;
        draftText.value = '';
        scrollToBottom();
    });
};

const scrollToBottom = () => {
    const list = document.getElementById('chat-scroll-area');
    if(list) {
        setTimeout(() => list.scrollTop = list.scrollHeight, 100);
    }
};

const closeInput = () => {
    inputShow.value = false;
    draftText.value = '';
};
</script>

<template>
  <div class="glass-app-container" data-tauri-drag-region>
    
    <!-- 极光光效装饰 -->
    <div class="aurora-fx" data-tauri-drag-region></div>

    <!-- 高级顶栏 -->
    <header class="header-nav" data-tauri-drag-region>
        <div class="brand">
            <Sparkles class="icon highlight-icon" :size="18" />
            <span class="brand-text">AITranslate Core</span>
        </div>
        <div class="actions">
            <div v-if="isListening" class="status-indicator active">
                <Activity class="icon pulse" :size="14"/> 监听中
            </div>
            <button v-else class="btn-primary" @click="startListening">
                <Play :size="14" /> 开启监听
            </button>
        </div>
    </header>

    <!-- 主对话区域 -->
    <main class="chat-list" id="chat-scroll-area">
        <!-- 如果未开启监听，引导用户填写 API -->
        <div class="auth-card" v-if="!isListening">
             <div class="auth-icon-wrap"><KeyRound :size="32" class="text-white opacity-80" /></div>
             <h3 class="auth-title">引擎待激活</h3>
             <p class="auth-desc">请输入您的 OpenAI / 模型 API Key，唤醒 AI 翻译核心。</p>
             <div class="auth-input-group">
                 <input type="password" v-model="apiKey" placeholder="sk-xxxx..." class="premium-input" />
             </div>
        </div>

        <!-- 对话流渲染 -->
        <div class="chat-wrapper" v-for="(msg, i) in chatHistory" :key="i">
            <div class="message-group" :class="{'self-msg': msg.sender === 'Me'}">
                <div class="message-meta">
                    <span class="time">{{msg.time}}</span>
                    <span class="sender">{{msg.sender}}</span>
                </div>
                <div class="message-bubble">
                    <div class="source-text">{{msg.text}}</div>
                    <div class="translate-text" v-if="msg.translated">{{msg.translated}}</div>
                </div>
            </div>
        </div>
    </main>

    <!-- 快捷调出的控制台/输入悬浮层 -->
    <Transition name="fade">
        <div class="quick-input-overlay" v-if="inputShow">
            <div class="terminal-box">
                <div class="terminal-header">
                    <div class="th-left"><MessageSquareDot :size="14" /> 快速回复中转 (输入后自动复制)</div>
                    <button class="icon-btn" @click="closeInput"><X :size="16"/></button>
                </div>
                <div class="terminal-body">
                    <input 
                        ref="inputRef"
                        type="text" 
                        class="terminal-input"
                        v-model="draftText" 
                        @keyup.enter="sendMyMessage"
                        @keyup.esc="closeInput"
                        placeholder="[中 -> 英] 开始输入..."
                    />
                    <button class="send-btn" @click="sendMyMessage">
                        <Send :size="14" />
                    </button>
                </div>
            </div>
        </div>
    </Transition>
  </div>
</template>
