<script setup>
import { ref, onMounted } from 'vue';
import { TauriBridge } from './services/TauriBridge';
import { LLMService } from './services/LLMService';
import { API_DEFAULTS } from './utils/constants';

const isListening = ref(false);
const chatHistory = ref([]); // 格式: { time: '14:22', sender: 'John', text: 'hi', translated: '你好' }
const inputShow = ref(false);
const draftText = ref('');
const apiKey = ref('');

onMounted(async () => {
    // 监听后端日志更新
    TauriBridge.onLogUpdate(async (payload) => {
        // 解析 payload，假设是 "John Resident: Hello"
        if (payload && payload.includes(": ")) {
            const [sender, text] = payload.split(": ");
            const item = { time: new Date().toLocaleTimeString(), sender, text, translated: '' };
            chatHistory.value.push(item);
            
            // 发起流式翻译
            LLMService.translateStream(text, [], (chunk) => {
                item.translated += chunk;
            }, {
                apiKey: apiKey.value,
                baseUrl: API_DEFAULTS.BASE_URL,
                model: API_DEFAULTS.MODEL,
                targetLang: 'Chinese'
            });
        }
    });

    // 监听全局快捷键触发
    TauriBridge.onShortcutTrigger(() => {
        inputShow.value = true;
        // 注意：实际应当等待 DOM 渲染后聚焦 input 框
    });
});

const startListening = async () => {
    if (!apiKey.value) {
        alert("为测试演示，请先填入 API Key！");
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
    
    // 模拟等待翻译
    LLMService.translateStream(mySource, [], (chunk) => {
        translatedResult += chunk;
    }, {
        apiKey: apiKey.value,
        baseUrl: API_DEFAULTS.BASE_URL,
        model: API_DEFAULTS.MODEL,
        targetLang: 'English'
    }).then(async () => {
        // 完成翻译后自动推入剪贴板
        await TauriBridge.copyToClipboard(translatedResult);
        chatHistory.value.push({
            time: new Date().toLocaleTimeString(),
            sender: 'Me',
            text: mySource,
            translated: translatedResult
        });
        inputShow.value = false;
        draftText.value = '';
        // 可以加上 TDesign 的 Msg 提示：已复制！
    });
};

const closeInput = () => {
    inputShow.value = false;
    draftText.value = '';
};
</script>

<template>
  <!-- 这是一个模拟无边框半透明窗体 -->
  <div class="app-container" data-tauri-drag-region>
    <div class="header" data-tauri-drag-region>
        <span>🌟 SL 无感翻译器</span>
        <button v-if="!isListening" class="start-btn" @click="startListening">启动监听</button>
        <span v-else class="status-badge">监听中...</span>
    </div>

    <!-- API Key 配置区 (测试用) -->
    <div class="config-bar" v-if="!isListening">
        <input type="password" v-model="apiKey" placeholder="输入 OpenAI API Key 以运行" />
    </div>

    <!-- 聊天记录区 -->
    <div class="chat-list" id="chat-list">
        <div class="chat-item" v-for="(msg, i) in chatHistory" :key="i">
            <div class="chat-meta">[{{msg.time}}] <strong>{{msg.sender}}</strong>:</div>
            <div class="chat-text source">{{msg.text}}</div>
            <div class="chat-text translate" v-if="msg.translated">{{msg.translated}}</div>
        </div>
    </div>

    <!-- 快捷发送区 (按Ctrl+Space唤起) -->
    <div class="input-overlay" v-if="inputShow">
        <div class="input-box">
            <div class="input-header">发消息 (回车将自动翻译并复制)</div>
            <input 
                type="text" 
                v-model="draftText" 
                @keyup.enter="sendMyMessage"
                @keyup.esc="closeInput"
                placeholder="输入中文..."
                autofocus 
            />
        </div>
    </div>
  </div>
</template>
