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
  FolderOpen, User, ChevronDown, Pin, PinOff, BookText
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
  await invoke('toggle_topmost', { pin: isPinned.value });
};

// ── 页面状态 ──────────────────────────────────────────────────────
const activeTab  = ref(TAB_CHAT);
const isListening = ref(false);

// ── 设置表单 ──────────────────────────────────────────────────────
const settings = ref({
  logDir:     '',
  account:    '',
  apiKey:     '',
  baseUrl:    API_DEFAULTS.BASE_URL,
  model:      API_DEFAULTS.MODEL,
  recvLang:   'Simplified Chinese',
  contextCount: 5,
  translateGroup: false,
  uiLang:     'zh-CN',
  blacklist:  [],
  lslEnabled: false,
  lslPort:    7001,
});
const accountList = ref([]);
const errorMessage = ref('');

// ── 内置语言包 ────────────────────────────────────────────────────
const I18N_BUNDLES = {
  'zh-CN': {
    appTitle: "AISLtalk", listeningInfo: "监听中", startListening: "开启监听",
    viewHistory: "查看历史记录", settingTitle: "设置",
    folderLog: "Firestorm 日志目录", browseLabel: "浏览",
    slAccount: "SL 账号", slAccountDropTip: "-- 选择账号文件夹 --",
    slAccountNoDirHint: "请先填写日志目录，软件会自动扫描账号列表。",
    apiKeyLabel: "API Key", baseUrlLabel: "Base URL", modelLabel: "模型",
    recvLangConfig: "将其发来的消息，翻译为：",
    ctxCountSetting: "翻译参考上文的条数（默认 5 条，填 0 关闭）",
    groupCb: "开启群聊日志翻译 (带有 group 字样的频道)", uiLangLabel: "软件界面语言",
    saveBtn: "💾 保存设置", apiKeyFloatTip: "填写 API Key 才能翻译：",
    inputPlc: "输入你的母语...回车翻译为对方语言并复制到剪贴板"
  },
  'en-US': {
    appTitle: "AISLtalk", listeningInfo: "Listening", startListening: "Start Listening",
    viewHistory: "View History", settingTitle: "Settings",
    folderLog: "Firestorm Log Directory", browseLabel: "Browse",
    slAccount: "SL Account", slAccountDropTip: "-- Select account folder --",
    slAccountNoDirHint: "Please set the log directory first.",
    apiKeyLabel: "API Key", baseUrlLabel: "Base URL", modelLabel: "Model",
    recvLangConfig: "Translate incoming messages to:",
    ctxCountSetting: "Context lines for translation (0 = disabled):",
    groupCb: "Enable group chat translation (files containing 'group')", uiLangLabel: "UI Language",
    saveBtn: "💾 Save Settings", apiKeyFloatTip: "Enter API Key to enable translation:",
    inputPlc: "Type in your language...Enter to translate & copy to clipboard"
  }
};

const i18n = ref({ ...I18N_BUNDLES['zh-CN'] });

const applyUiLang = async () => {
  const lang = settings.value.uiLang;
  if (lang === 'custom') {
    try {
      const custom = await invoke('load_custom_i18n');
      if (custom) Object.assign(i18n.value, JSON.parse(custom));
    } catch(e) {}
  } else {
    const bundle = I18N_BUNDLES[lang] || I18N_BUNDLES['zh-CN'];
    Object.assign(i18n.value, bundle);
  }
};

// 只要有路径和账号就能开监听，apiKey 可以后填
const settingsValid = computed(() =>
  Boolean(settings.value.logDir) &&
  Boolean(settings.value.account)
);


// ── 聊天状态 ──────────────────────────────────────────────────────
const chatTabs = ref([
  {
    id: 'chat.txt',
    title: '附近',
    messages: [
      {
        time: new Date().toLocaleTimeString(),
        sender: 'System',
        text: 'Tauri translation core initialized.',
        translated: 'Tauri 翻译引擎已就绪。'
      }
    ],
    hasUnread: false
  }
]);
const activeChatTabId = ref('chat.txt');
const activeTabMessages = computed(() => {
  const tab = chatTabs.value.find(t => t.id === activeChatTabId.value);
  return tab ? tab.messages : [];
});

const visibleTabs = computed(() => {
  return chatTabs.value.filter(t => !t.hidden);
});

const persistState = () => {
  const clone = chatTabs.value.map(t => ({
    ...t,
    // 保存最近 100 条防止 JSON 爆满内存
    messages: t.messages.slice(-100)  
  }));
  invoke('save_ui_state', { state: JSON.stringify(clone) }).catch(console.error);
};

const closeTab = (tabId) => {
  const tab = chatTabs.value.find(t => t.id === tabId);
  if (tab) tab.hidden = true;
  if (activeChatTabId.value === tabId) {
    activeChatTabId.value = 'chat.txt';
  }
  persistState();
};

const switchTab = (tabId) => {
  activeChatTabId.value = tabId;
  const tab = chatTabs.value.find(t => t.id === tabId);
  if (tab) tab.hasUnread = false;
  scrollToBottom();
};

const getHistoryContext = (tabId) => {
  const tab = chatTabs.value.find(t => t.id === tabId);
  if (!tab || !settings.value.contextCount) return [];
  const start = Math.max(0, tab.messages.length - settings.value.contextCount);
  const lastN = tab.messages.slice(start);
  return lastN.map(m => ({
    role: 'system',
    content: `[Context] ${m.sender}: ${m.text || m.translated}`
  }));
};
const inputShow = ref(false);
const draftText = ref('');
const inputRef  = ref(null);
const chatScrollRef = ref(null);

// ── 黑名单管理 ────────────────────────────────────────────────────
const blacklistInput = ref('');
const addBlacklistEntry = () => {
  const name = blacklistInput.value.trim();
  if (!name) return;
  if (!settings.value.blacklist.includes(name)) {
    settings.value.blacklist.push(name);
    saveSettings();
  }
  blacklistInput.value = '';
};
const removeBlacklistEntry = (name) => {
  settings.value.blacklist = settings.value.blacklist.filter(n => n !== name);
  saveSettings();
};

// ── LSL 中继服务器 ────────────────────────────────────────────
const lslServerAddr = ref('');

const startLslServer = async () => {
  try {
    const addr = await invoke('start_lsl_server', { port: settings.value.lslPort });
    lslServerAddr.value = addr;
  } catch (e) {
    // 已在运行则展示已存地址
    if (String(e).includes('already running')) {
      lslServerAddr.value = `本机 IP:${settings.value.lslPort}`;
    }
  }
};

const stopLslServer = async () => {
  await invoke('stop_lsl_server');
  lslServerAddr.value = '';
};

const toggleLslServer = async () => {
  if (settings.value.lslEnabled) {
    await startLslServer();
  } else {
    await stopLslServer();
  }
  saveSettings();
};

const LSL_SCRIPT_TEMPLATE = (serverUrl) => `// AISLtalk Nearby Chat HUD by AISLtalk
// 将 SERVER_URL 改为你的公网 IP 地址
string SERVER_URL = "${serverUrl}";
integer LISTEN_HANDLE;

string sanitize(string s) {
    list parts = llParseString2List(s, ["\\\\"], []);
    s = llDumpList2String(parts, "\\\\\\\\");
    parts = llParseString2List(s, ["\""], []);
    s = llDumpList2String(parts, "\\\\\"");
    return s;
}

default {
    state_entry() {
        LISTEN_HANDLE = llListen(0, "", NULL_KEY, "");
        llOwnerSay("[AISLtalk HUD] 已开始监听附近频道");
    }
    listen(integer channel, string name, key id, string message) {
        if (id == llGetOwner()) return; // 不发送自己的消息
        string payload = "{\"<<"sender">>\":\"" + sanitize(name) + "\",\"<<"text">>\":\"" + sanitize(message) + "\"}"; 
        llHTTPRequest(SERVER_URL, [HTTP_METHOD, "POST", HTTP_MIMETYPE, "application/json"], payload);
    }
    http_response(key id, integer status, list meta, string body) {}
    on_rez(integer p) { llResetScript(); }
}`;

const copyLslScript = async () => {
  const url = lslServerAddr.value
    ? `http://${lslServerAddr.value}/chat`
    : `http://你的公网IP:${settings.value.lslPort}/chat`;
  const script = LSL_SCRIPT_TEMPLATE(url)
    .replace(/<<"sender">>/g, 'sender')
    .replace(/<<"text">>/g, 'text');
  await navigator.clipboard.writeText(script);
  alert('✅ LSL 脚本已复制到剪贴板，请在 LSL 编辑器中粘贴！');
};


onMounted(async () => {
  // 读取持久化设置
  const saved = localStorage.getItem('sl-translator-settings');
  if (saved) {
    try { 
      const parsed = JSON.parse(saved);
      // 兼容旧版单一 targetLang
      if (parsed.targetLang && !parsed.recvLang) {
        parsed.recvLang = parsed.targetLang;
        delete parsed.targetLang;
      }
      Object.assign(settings.value, parsed); 
    } catch (_) {}
  }

  // 启动时应用已保存的界面语言
  await applyUiLang();

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

  // 自动开启监听逻辑（当存在合法设置时）
  if (settingsValid.value) {
    startListening();
  }

  // 如果上次开启了 LSL 中继，自动重启
  if (settings.value.lslEnabled) {
    await startLslServer();
  }

  // 加载上一次保留的 UI VDOM 会话状态
  try {
    const savedState = await invoke('load_ui_state');
    if (savedState) {
      chatTabs.value = JSON.parse(savedState);
    }
  } catch(e) {}

  let lastMsgHash = '';
  let lastMsgTime = 0;

  // 侦听后端日志推送
  await TauriBridge.onLogUpdate(async (payload) => {
    const source = payload.source || 'chat.txt';
    const msgContext = payload.msg;

    if (!msgContext?.includes(': ')) return;
    const colonIdx = msgContext.indexOf(': ');
    const sender   = msgContext.slice(0, colonIdx);
    const text     = msgContext.slice(colonIdx + 2);
    
    // 基础去重（防止 Firestorm 多文件记录同一个消息）
    const msgHash = `${source}||${sender}||${text}`;
    const now = Date.now();
    if (msgHash === lastMsgHash && (now - lastMsgTime) < 2000) {
      return; 
    }
    lastMsgHash = msgHash;
    lastMsgTime = now;

    // ── 层层层屁屁屁局局尿尿尾尾尽尽尼尼尻尻尺尺 登录训练新游被拳版 尺尺尻尻尼尼尽尽尾尾尿尿局局屁屁层层
    const accountBaseName = settings.value.account ? settings.value.account.split('_')[0].toLowerCase() : '---';
    const isMySelf = sender.toLowerCase().includes(accountBaseName) || sender.toLowerCase() === 'me';
    const isSystemMessage = sender.toLowerCase() === 'second life';
    const isGroupFile = source.toLowerCase().includes('group');
    const skipGroupMessage = isGroupFile && !settings.value.translateGroup;

    // 系统上下线提示：直接丢弃，不渲染不翻译
    if (isSystemMessage) return;

    // 定位或创建 Tab
    let tabInfo = chatTabs.value.find(t => t.id === source);
    if (!tabInfo) {
      let title = source.replace(/\.txt$/i, '').replace(/\.log$/i, '');
      if (source.toLowerCase() === 'conversation.log' || source.toLowerCase() === 'chat.txt') {
        title = '附近';
      }
      tabInfo = { id: source, title, messages: [], hasUnread: true };
      chatTabs.value.push(tabInfo);
    } else {
      tabInfo.hidden = false;
    }
    
    // 标红点
    if (activeChatTabId.value !== source) {
      tabInfo.hasUnread = true;
    }

    const newItem  = { time: new Date().toLocaleTimeString(), sender, text, translated: '' };
    tabInfo.messages.push(newItem);
    const reactiveItem = tabInfo.messages[tabInfo.messages.length - 1];
    
    persistState();
    if (activeChatTabId.value === source) scrollToBottom();

    // 自己发的、群组屏蔽、黑名单用户：渲染上屏但不翻译
    const isBlacklisted = settings.value.blacklist.some(
      name => name.trim().toLowerCase() === sender.toLowerCase()
    );
    if (isMySelf || skipGroupMessage || isBlacklisted) return;

    // 组织上文（将同一个对话频道里的前面 N 句当做参考喂给AI以保持连贯！）
    const history = getHistoryContext(source);

    await LLMService.translateStream(text, history, (chunk) => {
      reactiveItem.translated += chunk;
    }, {
      apiKey:     settings.value.apiKey,
      baseUrl:    settings.value.baseUrl,
      model:      settings.value.model,
      targetLang: settings.value.recvLang,
    });
    
    // 异步保存到本地历史记录档案中
    if (reactiveItem.translated.trim()) {
      invoke('append_translation_history', {
        source,
        timestamp: reactiveItem.time,
        sender,
        text,
        translated: reactiveItem.translated
      }).catch(e => console.error("保存历史记录失败", e));
    }
    persistState();
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
  const tab = chatTabs.value.find(t => t.id === activeChatTabId.value);
  if (tab) {
    tab.messages.push(item);
  }
  const reactiveItem = tab ? tab.messages[tab.messages.length - 1] : item;
  scrollToBottom();

  const history = getHistoryContext(activeChatTabId.value);

  let translatedResult = '';
  await LLMService.translateStream(text, history, (chunk) => {
    translatedResult  += chunk;
    reactiveItem.translated += chunk;
  }, {
    apiKey:     settings.value.apiKey,
    baseUrl:    settings.value.baseUrl,
    model:      settings.value.model,
    targetLang: settings.value.sendLang,
  });

  if (translatedResult.trim()) {
    invoke('append_translation_history', {
      source: activeChatTabId.value,
      timestamp: item.time,
      sender: item.sender,
      text,
      translated: translatedResult
    }).catch(e => console.error("保存历史记录失败", e));
  }
  
  persistState();

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

const openHistoryFolder = async () => {
  await invoke('open_history_folder');
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
        <span class="brand-name">AISLtalk</span>
      </div>

      <div class="title-actions" @mousedown.stop>
        <!-- 监听状态 -->
        <div v-if="isListening" class="badge-listening">
          <Activity :size="12" class="pulse" /> 监听中
        </div>
        <button v-else class="btn-start" @click="startListening">
          <Play :size="12" /> 开启监听
        </button>

        <!-- 打开历史记录 -->
        <button class="ctrl-btn" title="查看全部历史记录" @click="openHistoryFolder">
          <BookText :size="14" />
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
            <label class="form-label">将其发来的消息，翻译为：</label>
            <input v-model="settings.recvLang" class="form-input" placeholder="Simplified Chinese / English / etc..." />
          </div>

          <div class="form-section">
            <label class="form-label">将我发出的内容，翻译为：</label>
            <input v-model="settings.sendLang" class="form-input" placeholder="English / Spanish / etc..." />
          </div>

          <div class="form-section">
            <label class="form-label">翻译参考上文的条数（默认 5 条，填 0 关闭）</label>
            <input v-model.number="settings.contextCount" type="number" class="form-input" placeholder="5" />
          </div>

          <div class="form-section">
            <label class="form-label">
              <input type="checkbox" v-model="settings.translateGroup" style="vertical-align: middle; margin-right: 5px;" />
              开启群聊日志翻译 (带有 group 字样的频道)
            </label>
          </div>

          <div class="form-section">
            <label class="form-label">软件界面语言</label>
            <div class="select-wrap">
              <select v-model="settings.uiLang" class="form-select" @change="applyUiLang">
                <option value="zh-CN">简体中文</option>
                <option value="en-US">English</option>
                <option value="custom">Custom (i18n.json)</option>
              </select>
              <ChevronDown :size="14" class="select-chevron" />
            </div>
          </div>

          <div class="form-section">
            <label class="form-label">🚫 翻译黑名单（这些用户的消息不翻译）</label>
            <div class="input-row">
              <input
                v-model="blacklistInput"
                class="form-input"
                placeholder="输入用户名..."
                @keyup.enter="addBlacklistEntry"
              />
              <button class="btn-browse" @click="addBlacklistEntry">添加</button>
            </div>
            <div class="blacklist-tags" v-if="settings.blacklist.length > 0">
              <span
                v-for="name in settings.blacklist"
                :key="name"
                class="blacklist-tag"
                @click="removeBlacklistEntry(name)"
                title="点击删除"
              >
                {{ name }} ×
              </span>
            </div>
          </div>

          <div class="form-section lsl-section">
            <label class="form-label">
              <input type="checkbox" v-model="settings.lslEnabled" style="vertical-align: middle; margin-right: 5px;" @change="toggleLslServer" />
              🛸 开启 LSL HUD 公共频道中继
            </label>

            <div v-if="settings.lslEnabled" class="lsl-config">
              <div class="input-row" style="margin-top: 8px;">
                <input v-model.number="settings.lslPort" type="number" class="form-input" style="width: 90px;" />
                <button class="btn-browse" @click="startLslServer">重启服务</button>
              </div>

              <div v-if="lslServerAddr" class="lsl-addr-box">
                <span class="lsl-addr-label">你的 HUD 地址：</span>
                <code class="lsl-addr">http://{{ lslServerAddr }}/chat</code>
              </div>

              <div class="lsl-hint">
                <p>ℹ️ 如何使用：</p>
                <ol>
                  <li>将路由器端口 <b>{{ settings.lslPort }}</b> 转发到此电脑</li>
                  <li>在 SL 中使用以下 LSL 脚本，将 SERVER_URL 改为公网地址</li>
                  <li>将脚本放入 HUD，佩戴后即可监听附近频道</li>
                </ol>
                <button class="btn-browse" style="width:100%; margin-top: 6px;" @click="copyLslScript">📝 复制 LSL 脚本到剪贴板</button>
              </div>
            </div>
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

      <!-- 选项卡展示区 -->
      <div class="chat-tab-bar" v-if="activeTab === TAB_CHAT && isListening">
        <div 
          v-for="t in visibleTabs" 
          :key="t.id" 
          class="chat-tab-item" 
          :class="{ active: activeChatTabId === t.id, unread: t.hasUnread }"
          @click="switchTab(t.id)">
          <span class="tab-title">{{ t.title }}</span>
          <button class="tab-close" @click.stop="closeTab(t.id)" v-if="t.id !== 'chat.txt' && t.id !== 'conversation.log'"><X :size="10" /></button>
        </div>
      </div>

      <!-- 聊天面板 -->
      <div class="chat-list" ref="chatScrollRef">
        <div
          v-for="(msg, i) in activeTabMessages"
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
            <div class="translate-text" v-if="msg.translated && msg.translated.trim() !== msg.text.trim()">{{ msg.translated }}</div>
          </div>
        </div>
      </div>

      <!-- 底部常驻输入栏 -->
      <div class="bottom-input-area" v-if="activeTab === TAB_CHAT && isListening">
        <input
          ref="inputRef"
          class="inline-input"
          v-model="draftText"
          placeholder="输入你的母语...回车翻译为对方语言并复制到剪贴板"
          @keyup.enter="sendMyMessage"
        />
        <button class="btn-send-inline" @click="sendMyMessage" title="翻译并复制">
          <Send :size="14" />
        </button>
      </div>

    </div>
  </div>
</template>
