<script setup>
import { ref, onMounted, nextTick, computed } from 'vue';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { invoke } from '@tauri-apps/api/core';
import { TauriBridge } from './services/TauriBridge';
import { LLMService } from './services/LLMService';
import { API_DEFAULTS } from './utils/constants';
import {
  Sparkles, Activity, Play, Minus, X,
  Settings, MessageSquareDot, Send, KeyRound,
  FolderOpen, User, ChevronDown, Pin, PinOff
} from 'lucide-vue-next';

// ── 常量 ──────────────────────────────────────────────────────────
const TAB_CHAT = 'chat';
const TAB_SETTINGS = 'settings';

// ── 窗口控制 ──────────────────────────────────────────────────────
const win = getCurrentWindow();
const handleDrag     = () => win.startDragging();
const handleMinimize = () => win.minimize();
// 关闭 = 隐藏到系统托盘，从托盘右键「退出」才真正结束
const handleClose    = () => win.hide();

const isPinned = ref(false);
const togglePin = async () => {
  isPinned.value = !isPinned.value;
  await win.setAlwaysOnTop(isPinned.value);
};

// ── 页面状态 ──────────────────────────────────────────────────────
const activeTab  = ref(TAB_CHAT);
const isListening = ref(false);

// ── 设置表单 ──────────────────────────────────────────────────────
const settings = ref({
  logDir:     '',        // Firestorm 日志根目录
  account:    '',        // 用户选择的账号文件夹
  apiKey:     '',
  baseUrl:    API_DEFAULTS.BASE_URL,
  model:      API_DEFAULTS.MODEL,
  targetLang: 'Chinese',
  contextCount: 5,
});
const accountList = ref([]);
const errorMessage = ref('');

// 只要有路径和账号就能开监听，apiKey 可以后填
const settingsValid = computed(() =>
  Boolean(settings.value.logDir) &&
  Boolean(settings.value.account)
);

// ── 聊天状态 ──────────────────────────────────────────────────────
const chatHistory = ref([
  {
    time: new Date().toLocaleTimeString(),
    sender: 'System',
    text: 'Tauri translation core initialized.',
    translated: 'Tauri 翻译引擎已就绪。'
  }
]);
const inputShow = ref(false);
const draftText = ref('');
const inputRef  = ref(null);
const chatScrollRef = ref(null);

// ── 初始化 ──────────────────────────────────────────────────────
onMounted(async () => {
  // 读取持久化设置
  const saved = localStorage.getItem('sl-translator-settings');
  if (saved) {
    try { Object.assign(settings.value, JSON.parse(saved)); } catch (_) {}
  }

  // 如果没有保存的路径，自动解析 %APPDATA%\Firestorm_x64 的真实路径
  if (!settings.value.logDir) {
    try {
      const defaultDir = await invoke('get_default_log_dir');
      if (defaultDir) {
        settings.value.logDir = defaultDir;
      }
    } catch (_) {}
  }

  // 有路径就自动扫描账号列表
  if (settings.value.logDir) {
    await refreshAccounts();
  }

  // 侦听后端日志推送
  await TauriBridge.onLogUpdate(async (payload) => {
    if (!payload?.includes(': ')) return;
    const colonIdx = payload.indexOf(': ');
    const sender   = payload.slice(0, colonIdx);
    const text     = payload.slice(colonIdx + 2);
    const newItem  = { time: new Date().toLocaleTimeString(), sender, text, translated: '' };
    
    // 把对象塞入具有 Proxy 深层响应式的数组中
    chatHistory.value.push(newItem);
    // 从数组中获取已被 Proxy 劫持的响应式引用，避免直接修改原生对象导致视图不更新！
    const reactiveItem = chatHistory.value[chatHistory.value.length - 1];
    
    scrollToBottom();

    await LLMService.translateStream(text, [], (chunk) => {
      reactiveItem.translated += chunk;
    }, {
      apiKey:     settings.value.apiKey,
      baseUrl:    settings.value.baseUrl,
      model:      settings.value.model,
      targetLang: settings.value.targetLang,
    });
  });

  // 侦听全局快捷键
  await TauriBridge.onShortcutTrigger(async () => {
    inputShow.value = true;
    await nextTick();
    inputRef.value?.focus();
  });
});

// ── 设置操作 ──────────────────────────────────────────────────────
const saveSettings = () => {
  localStorage.setItem('sl-translator-settings', JSON.stringify(settings.value));
};

const browseLogDir = async () => {
  // 调用后端 open_dialog 指令
  try {
    const dir = await invoke('open_folder_dialog');
    if (dir) {
      settings.value.logDir = dir;
      await refreshAccounts();
    }
  } catch (e) {
    alert('打开文件夹失败，请手动粘贴路径。\n' + e);
  }
};

const refreshAccounts = async () => {
  if (!settings.value.logDir) return;
  try {
    accountList.value = await invoke('list_accounts', { logDir: settings.value.logDir });
  } catch (e) {
    accountList.value = [];
  }
};

// ── 监听控制 ──────────────────────────────────────────────────────
const startListening = async () => {
  errorMessage.value = '';
  if (!settings.value.logDir || !settings.value.account) {
    activeTab.value = TAB_SETTINGS;
    errorMessage.value = "请先填写正确的日志目录，并在下拉框中选择你的 SL 账号！\n如果下拉列表为空，请检查日志目录是否正确（必须是 Firestorm_x64 的根目录）。";
    return;
  }
  saveSettings();
  try {
    const watchPath = `${settings.value.logDir}\\${settings.value.account}`;
    await TauriBridge.startLogWatcher(watchPath);
    isListening.value = true;
    activeTab.value   = TAB_CHAT;
  } catch (e) {
    errorMessage.value = '引擎启动失败: ' + e;
  }
};

// ── 快捷回复 ──────────────────────────────────────────────────────
const sendMyMessage = async () => {
  const text = draftText.value.trim();
  if (!text) return;

  draftText.value  = '';
  inputShow.value  = false;

  const item = {
    time: new Date().toLocaleTimeString(),
    sender: settings.value.account || 'Me',
    text,
    translated: ''
  };
  chatHistory.value.push(item);
  const reactiveItem = chatHistory.value[chatHistory.value.length - 1];
  scrollToBottom();

  let translatedResult = '';
  await LLMService.translateStream(text, [], (chunk) => {
    translatedResult  += chunk;
    reactiveItem.translated += chunk;
  }, {
    apiKey:     settings.value.apiKey,
    baseUrl:    settings.value.baseUrl,
    model:      settings.value.model,
    targetLang: 'English',
  });

  await TauriBridge.copyToClipboard(translatedResult);
};

const closeInput = () => {
  inputShow.value = false;
  draftText.value = '';
};

// ── 工具 ──────────────────────────────────────────────────────────
const scrollToBottom = () => {
  nextTick(() => {
    if (chatScrollRef.value) {
      chatScrollRef.value.scrollTop = chatScrollRef.value.scrollHeight;
    }
  });
};
</script>

<template>
  <div class="app-root">

    <!-- ── 极光背景装饰 ────────────────────────────────────────── -->
    <div class="aurora-fx"></div>

    <!-- ── 顶栏 ──────────────────────────────────────────────── -->
    <header class="title-bar" @mousedown="handleDrag">
      <div class="brand">
        <Sparkles :size="15" class="brand-icon" />
        <span class="brand-name">AITranslate Core</span>
      </div>

      <div class="title-actions" @mousedown.stop>
        <!-- 监听状态 -->
        <div v-if="isListening" class="badge-listening">
          <Activity :size="12" class="pulse" /> 监听中
        </div>
        <button v-else class="btn-start" @click="startListening">
          <Play :size="12" /> 开启监听
        </button>

        <!-- 设置 -->
        <button
          class="ctrl-btn"
          :class="{ active: activeTab === TAB_SETTINGS }"
          title="设置"
          @click="activeTab = activeTab === TAB_SETTINGS ? TAB_CHAT : TAB_SETTINGS"
        >
          <Settings :size="14" />
        </button>

        <!-- 置顶 -->
        <button class="ctrl-btn ctrl-pin" :class="{ pinned: isPinned }" :title="isPinned ? '取消置顶' : '全局置顶'" @click="togglePin">
          <Pin v-if="!isPinned" :size="14" />
          <PinOff v-else :size="14" />
        </button>

        <!-- 最小化 -->
        <button class="ctrl-btn" title="最小化" @click="handleMinimize">
          <Minus :size="14" />
        </button>

        <!-- 关闭 -->
        <button class="ctrl-btn ctrl-close" title="关闭" @click="handleClose">
          <X :size="14" />
        </button>
      </div>
    </header>

    <!-- ── 主体区 ──────────────────────────────────────────────── -->
    <div class="main-area">

      <!-- 设置面板 -->
      <Transition name="slide-down">
        <section class="settings-panel" v-if="activeTab === TAB_SETTINGS">

          <div v-if="errorMessage" class="error-banner">
            <span style="white-space: pre-wrap;">{{ errorMessage }}</span>
          </div>

          <div class="form-section">
            <label class="form-label"><FolderOpen :size="13" /> Firestorm 日志目录</label>
            <div class="input-row">
              <input
                v-model="settings.logDir"
                class="form-input"
                placeholder="%AppData%\Firestorm_x64"
                @change="refreshAccounts"
              />
              <button class="btn-browse" @click="browseLogDir">浏览</button>
            </div>
          </div>

          <div class="form-section">
            <label class="form-label"><User :size="13" /> SL 账号</label>
            <div class="select-wrap">
              <select v-model="settings.account" class="form-select">
                <option value="" disabled>-- 选择账号文件夹 --</option>
                <option v-for="acc in accountList" :key="acc" :value="acc">{{ acc }}</option>
              </select>
              <ChevronDown :size="14" class="select-chevron" />
            </div>
            <div class="form-hint" v-if="accountList.length === 0">
              请先填写日志目录，软件会自动扫描账号列表。
            </div>
          </div>

          <div class="form-section">
            <label class="form-label"><KeyRound :size="13" /> API Key</label>
            <input
              v-model="settings.apiKey"
              type="password"
              class="form-input"
              placeholder="sk-xxxx..."
            />
          </div>

          <div class="form-section">
            <label class="form-label">Base URL</label>
            <input v-model="settings.baseUrl" class="form-input" />
          </div>

          <div class="form-section">
            <label class="form-label">模型</label>
            <input v-model="settings.model" class="form-input" placeholder="gpt-4o-mini" />
          </div>

          <div class="form-section">
            <label class="form-label">目标语言（收到消息翻译为）</label>
            <input v-model="settings.targetLang" class="form-input" placeholder="Chinese" />
          </div>

          <button class="btn-save" @click="saveSettings">💾 保存设置</button>

        </section>
      </Transition>

      <!-- API Key 快速条（开启监听后若未填会提示） -->
      <div class="apikey-bar" v-if="isListening && !settings.apiKey">
        <KeyRound :size="13" class="apikey-icon" />
        <span class="apikey-hint">填写 API Key 才能翻译：</span>
        <input
          v-model="settings.apiKey"
          type="password"
          class="apikey-input"
          placeholder="sk-xxxx..."
          @change="saveSettings"
        />
      </div>

      <!-- 聊天面板 -->
      <div class="chat-list" ref="chatScrollRef">
        <div
          v-for="(msg, i) in chatHistory"
          :key="i"
          class="message-group"
          :class="{ 'self-msg': msg.sender === (settings.account || 'Me') }"
        >
          <div class="message-meta">
            <span class="msg-sender">{{ msg.sender }}</span>
            <span class="msg-time">{{ msg.time }}</span>
          </div>
          <div class="message-bubble">
            <div class="source-text">{{ msg.text }}</div>
            <div class="translate-text" v-if="msg.translated">{{ msg.translated }}</div>
          </div>
        </div>
      </div>
    </div>

    <!-- ── 快捷输入浮层 ──────────────────────────────────────── -->
    <Transition name="fade">
      <div class="input-overlay" v-if="inputShow" @click.self="closeInput">
        <div class="terminal-box">
          <div class="terminal-header">
            <div class="th-left">
              <MessageSquareDot :size="13" /> 中 → 英（回车翻译并复制）
            </div>
            <button class="icon-btn" @click="closeInput"><X :size="14" /></button>
          </div>
          <div class="terminal-body">
            <input
              ref="inputRef"
              class="terminal-input"
              v-model="draftText"
              placeholder="输入中文..."
              @keyup.enter="sendMyMessage"
              @keyup.esc="closeInput"
            />
            <button class="btn-send" @click="sendMyMessage">
              <Send :size="14" />
            </button>
          </div>
        </div>
      </div>
    </Transition>

  </div>
</template>
