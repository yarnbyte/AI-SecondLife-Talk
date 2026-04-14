<script setup>
import { ref, watch, onMounted, nextTick, computed } from 'vue';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { invoke } from '@tauri-apps/api/core';
import { openUrl } from '@tauri-apps/plugin-opener';
import { saveWindowState, restoreStateCurrent } from '@tauri-apps/plugin-window-state';
import { TauriBridge } from './services/TauriBridge';
import { LLMService } from './services/LLMService';
import { UpdateService } from './services/UpdateService';
import { API_DEFAULTS } from './utils/constants';
import {
  Sparkles, Activity, Play, Minus, X,
  Settings, MessageSquareDot, Send, KeyRound,
  FolderOpen, User, ChevronDown, Pin, PinOff, BookText,
  Bell, BellOff, Copy, HelpCircle
} from 'lucide-vue-next';

// ── 常量 ──────────────────────────────────────────────────────────
const TAB_CHAT = 'chat';
const TAB_SETTINGS = 'settings';

// ── 窗口控制 ──────────────────────────────────────────────────────
const win = getCurrentWindow();
const handleDrag     = () => win.startDragging();
const handleMinimize = () => win.minimize();
// 关闭 = 隐藏到系统托盘，从托盘右键「退出」才真正结束
const handleClose = async () => {
  try { await saveWindowState(63); } catch (e) {}
  win.hide();
};

const isPinned = ref(false);
const togglePin = async () => {
  isPinned.value = !isPinned.value;
  await invoke('toggle_topmost', { pin: isPinned.value });
};

// ── 页面状�?──────────────────────────────────────────────────────
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
  lslPublicUrl: '',
  lslEnabled: false,
  viewerType: 'firestorm',  // 'firestorm' | 'official' | 'custom'
  bgOpacity: 0.75,          // 窗口背景透明�?
  nearbyWhitelistEnabled: false, // 附近频道白名单开�?
  nearbyWhitelist: [],           // 白名单用户名数组（附近频道）
  keywordBlocklist: [],          // 关键词屏蔽数组（含此词不翻译�?
});
const accountList = ref([]);
const errorMessage = ref('');

// ── 内置语言�?────────────────────────────────────────────────────
const I18N_BUNDLES = {
  'zh-CN': {
    appTitle: "AI.SLtalk", listeningInfo: "工作�?, startListening: "启动翻译",
    viewHistory: "查看历史记录", settingTitle: "设置",
    folderLog: "日志目录", browseLabel: "浏览",
    slAccount: "SL 账号", slAccountDropTip: "-- 选择账号文件�?--",
    slAccountNoDirHint: "请先填写日志目录，软件会自动扫描账号列表�?,
    apiKeyLabel: "API Key", baseUrlLabel: "Base URL", modelLabel: "模型",
    recvLangConfig: "将其发来的消息，翻译为：",
    ctxCountSetting: "翻译参考上文的条数（默�?5 条，�?0 关闭�?,
    groupCb: "开启群聊同步及翻译 (带有 group 字样的频�?", uiLangLabel: "软件界面语言",
    saveBtn: "💾 保存设置", apiKeyFloatTip: "填写 API Key 才能翻译�?,
    inputPlc: "输入你的母语...回车翻译为对方语言并复制到剪贴�?,
    nearbyTab: "附近", tutorialTitle: "使用教程", stopListeningTitle: "停止翻译引擎",
    testApiBtn: "测试 API", testApiTesting: "测试�?..",
    testApiNoKey: "请先填写 API Key",
    testApiConnecting: "正在连接 API...",
    testApiSuccess: "�?测试通过！API 连接正常�?,
    testApiFail: "�?测试失败：无法获取响应流，请检�?Base URL�?,
    muteChanTip: "免打�?(停止翻译此频�?",
    sendBtnTip: "翻译并复�?,
    mutedOverlay: "已对该频道静音（不再翻译），您可以点击上�?Tab 中的钓铃图标取消静音�?,
    engineStartFail: "引擎启动失败: ",
    settingsIncomplete: "请先填写正确的日志目录，并在下拉框中选择你的 SL 账号！\n如果下拉列表为空，请检查日志目录是否正确�?,
    pinTitle: "全局置顶", unpinTitle: "取消置顶",
    minimizeTitle: "最小化", closeTitle: "关闭",
    updateAvailable: "发现新版本：v{v}", updateBtn: "前往下载",
    copyOriginal: "复制原文", copyTranslation: "复制译文", suggestReply: "AI 建议回复",
    aiChatPlc: "�?AI 帮你翻译、润色、建议回复�?, aiChatSend: "发�?,
    copyResult: "复制结果", clearResult: "清除",
    refreshAccounts: "刷新账号列表",
    viewerLabel: "客户�?,
    viewerFirestorm: "Firestorm Viewer",
    viewerOfficial: "官方客户�?(Official)",
    viewerCustom: "自定义目�?,
    logDirCustomHint: "请手动填写日志目录路�?,
    windowOpacityLabel: "窗口透明�?,
    nearbyWhitelistLabel: "附近频道白名�?,
    nearbyWhitelistCb: "启用附近白名单（仅翻译白名单内用户的消息，私聊不受影响）",
    nearbyWhitelistPlc: "每行一个用户名，可从Profile复制过来",
    keywordBlocklistLabel: "关键词屏蔽（消息含以下词时不翻译�?,
    keywordBlocklistPlc: "每行一个关键词，例如：\nis offline\n離開該區�?,
    secTranslation: "翻译参数",
    secFilter: "过滤规则",
    secAppearance: "界面外观",
  },
  'en-US': {
    appTitle: "AI.SLtalk", listeningInfo: "Active", startListening: "Start Translator",
    viewHistory: "View History", settingTitle: "Settings",
    folderLog: "Log Directory", browseLabel: "Browse",
    slAccount: "SL Account", slAccountDropTip: "-- Select account folder --",
    slAccountNoDirHint: "Please set the log directory first.",
    apiKeyLabel: "API Key", baseUrlLabel: "Base URL", modelLabel: "Model",
    recvLangConfig: "Translate incoming messages to:",
    ctxCountSetting: "Context lines for translation (0 = disabled):",
    groupCb: "Enable group chat translation (files containing 'group')", uiLangLabel: "UI Language",
    saveBtn: "💾 Save Settings", apiKeyFloatTip: "Enter API Key to enable translation:",
    inputPlc: "Type in your language...Enter to translate & copy to clipboard",
    nearbyTab: "Nearby", tutorialTitle: "Tutorial", stopListeningTitle: "Stop Translator Engine",
    testApiBtn: "Test API", testApiTesting: "Testing...",
    testApiNoKey: "Please fill in the API Key first",
    testApiConnecting: "Connecting to API...",
    testApiSuccess: "�?Test passed! API connection is working.",
    testApiFail: "�?Test failed: no response stream received. Please check the Base URL.",
    muteChanTip: "Mute (stop translating this channel)",
    sendBtnTip: "Translate and copy",
    mutedOverlay: "This channel is muted. Click the bell icon above to unmute.",
    engineStartFail: "Engine failed to start: ",
    settingsIncomplete: "Please set the log directory and select your SL account first.\nIf the dropdown is empty, check that the log directory path is correct.",
    pinTitle: "Always on Top", unpinTitle: "Unpin",
    minimizeTitle: "Minimize", closeTitle: "Close",
    updateAvailable: "New version available: v{v}", updateBtn: "Download",
    copyOriginal: "Copy original", copyTranslation: "Copy translation", suggestReply: "AI suggest reply",
    aiChatPlc: "Ask AI to translate, polish, suggest a reply...", aiChatSend: "Send",
    copyResult: "Copy result", clearResult: "Clear",
    refreshAccounts: "Refresh account list",
    viewerLabel: "Viewer",
    viewerFirestorm: "Firestorm Viewer",
    viewerOfficial: "Official Second Life Viewer",
    viewerCustom: "Custom directory",
    logDirCustomHint: "Enter the log directory path manually",
    windowOpacityLabel: "Window Opacity",
    nearbyWhitelistLabel: "Nearby Chat Whitelist",
    nearbyWhitelistCb: "Enable whitelist for Nearby Chat (only translate whitelisted users; IMs unaffected)",
    nearbyWhitelistPlc: "One username per line, copy from Profile",
    keywordBlocklistLabel: "Keyword Blocklist (skip translation if message contains any keyword)",
    keywordBlocklistPlc: "One keyword per line, e.g.:\nis offline\n離開該區�?,
    secTranslation: "Translation",
    secFilter: "Filters",
    secAppearance: "Appearance",
  }
};

const i18n = ref({ ...I18N_BUNDLES['zh-CN'] });
const updateAvailable = ref(false);
const updateInfo = ref(null);

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


// ── 聊天状�?──────────────────────────────────────────────────────
const chatTabs = ref([
  {
    id: 'chat.txt',
    title: 'nearby',
    messages: [
      {
        time: new Date().toLocaleTimeString(),
        sender: 'System',
        text: 'AI SLtalk translation core initialized.',
        translated: 'AI SLtalk 翻译引擎已就绪�?
      }
    ],
    hasUnread: false
  }
]);
const activeChatTabId = ref('chat.txt');
const tabRenderLimits = ref({});
const CHAT_RENDER_LIMIT = 20;

const activeTabMessages = computed(() => {
  const tab = chatTabs.value.find(t => t.id === activeChatTabId.value);
  if (!tab) return [];
  // 如果当前数量超过限制，则截断；否则全部显�?
  const limit = tabRenderLimits.value[activeChatTabId.value] || CHAT_RENDER_LIMIT;
  return tab.messages.slice(-limit);
});

let isPaginating = false;
const handleChatScroll = async (e) => {
  if (e.target.scrollTop <= 5 && !isPaginating) {
    const tabId = activeChatTabId.value;
    const tab = chatTabs.value.find(t => t.id === tabId);
    if (!tab) return;
    
    const limit = tabRenderLimits.value[tabId] || CHAT_RENDER_LIMIT;
    if (limit < tab.messages.length) {
      isPaginating = true;
      const oldHeight = e.target.scrollHeight;
      const oldScrollTop = e.target.scrollTop;
      
      // 每次滚动到顶多加�?20 �?
      tabRenderLimits.value[tabId] = limit + 20;
      
      await nextTick();
      // 计算加入新节点后的高度差并恢复滚动位�?
      e.target.scrollTop = e.target.scrollHeight - oldHeight + oldScrollTop;
      
      // 释放锁，允许下次上拉触发
      setTimeout(() => { isPaginating = false; }, 50);
    }
  }
};

const visibleTabs = computed(() => {
  let tabs = chatTabs.value.filter(t => !t.hidden);
  if (!settings.value.translateGroup) {
    tabs = tabs.filter(t => !t.id.toLowerCase().includes('group'));
  }
  return tabs;
});

const persistState = () => {
  const clone = chatTabs.value.map(t => ({
    ...t,
    // 保存最�?100 条防�?JSON 爆满内存
    messages: t.messages.slice(-100)  
  }));
  invoke('save_ui_state', { account: settings.value.account || '', state: JSON.stringify(clone) }).catch(console.error);
};

const closeTab = (tabId) => {
  const tab = chatTabs.value.find(t => t.id === tabId);
  if (tab) tab.hidden = true;
  if (activeChatTabId.value === tabId) {
    activeChatTabId.value = 'chat.txt';
  }
  persistState();
};

const tabBarRef = ref(null);
const handleTabScroll = (e) => {
  if (tabBarRef.value) {
    // 将鼠标垂直滚�?deltaY)转换为横向滚动，实现标准鼠标顺畅切换
    tabBarRef.value.scrollLeft += e.deltaY;
  }
};

const copyContent = async (text) => {
  if (!text) return;
  try {
    await navigator.clipboard.writeText(text);
  } catch(e) {}
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

// 获取对方发出的原文消息，不包括自己发出的，用于推断对方语言
const getOtherRawMessages = (tabId) => {
  const tab = chatTabs.value.find(t => t.id === tabId);
  if (!tab) return [];
  const myName = (settings.value.account || '').trim().toLowerCase();
  const CONTEXT_LIMIT = 5;
  const otherMessages = tab.messages
    .filter(m => m.sender.trim().toLowerCase() !== myName && m.text)
    .slice(-CONTEXT_LIMIT);
  return otherMessages.map((m, i) => `${i + 1}. ${m.text}`);
};

const inputShow = ref(false);
const draftText = ref('');
const inputRef  = ref(null);
const chatScrollRef = ref(null);

// AI 助手相关状�?
const aiReplyMap = ref({});   // { 'tabId:msgIndex': { loading, text } }
const aiChat = ref({ loading: false, result: '' });
const aiChatInput = ref('');

// ── 黑名单管�?───────────────────────────────────────────────
const isTargetBlacklisted = (targetId) => {
  return settings.value.blacklist.some(n => n.toLowerCase() === targetId.toLowerCase());
};

// ── 白名�?/ 关键词屏蔽：�?computed v-model 避免回车换行被拦�?──
const nearbyWhitelistRaw = computed({
  get: () => settings.value.nearbyWhitelist.join('\n'),
  set: (v) => { settings.value.nearbyWhitelist = v.split('\n'); }
});
const keywordBlocklistRaw = computed({
  get: () => settings.value.keywordBlocklist.join('\n'),
  set: (v) => { settings.value.keywordBlocklist = v.split('\n'); }
});

const toggleBlacklist = (targetId) => {
  if (isTargetBlacklisted(targetId)) {
    settings.value.blacklist = settings.value.blacklist.filter(n => n.toLowerCase() !== targetId.toLowerCase());
  } else {
    settings.value.blacklist.push(targetId);
  }
  saveSettings();
};

// ── 设置面板手风琴折�?───────────────────────────────────────
const openSections = ref(new Set(['connection']));
const toggleSection = (key) => {
  if (openSections.value.has(key)) {
    openSections.value.delete(key);
  } else {
    openSections.value.add(key);
  }
  openSections.value = new Set(openSections.value); // 触发响应�?
};

// 监听并应用背景透明�?
watch(() => settings.value.bgOpacity, (alpha) => {
  document.documentElement.style.setProperty('--bg-opacity', alpha);
}, { immediate: true });

onMounted(async () => {
  // 恢复窗口状态并显示
  try {
    await restoreStateCurrent(63); // 63 = StateFlags.ALL
    await win.show();
  } catch (e) {
    console.error('State restore error:', e);
    await win.show(); // Fallback
  }

  // 监听移动/缩放时保存（加抖动限制，防止拖拽卡死�?
  let saveTimer = null;
  const debouncedSave = () => {
    if (saveTimer) clearTimeout(saveTimer);
    saveTimer = setTimeout(() => {
      saveWindowState(63).catch(() => {});
    }, 500);
  };
  win.onMoved(debouncedSave);
  win.onResized(debouncedSave);

  // 读取持久化设置（兼容旧版�?key 格式，自动迁移）
  const legacySaved = localStorage.getItem('sl-translator-settings');
  const globalSaved  = localStorage.getItem(GLOBAL_SETTINGS_KEY);

  if (legacySaved && !globalSaved) {
    // 迁移旧格�?
    try {
      const parsed = JSON.parse(legacySaved);
      if (parsed.targetLang && !parsed.recvLang) { parsed.recvLang = parsed.targetLang; }
      Object.assign(settings.value, parsed);
      // 写入新格�?
      saveSettings();
      localStorage.removeItem('sl-translator-settings');
    } catch (_) {}
  } else if (globalSaved) {
    // 新格式：先加载全局
    try { Object.assign(settings.value, JSON.parse(globalSaved)); } catch (_) {}
    // 再加载上次选中账号的账号级设置
    if (settings.value.account) {
      try {
        const acctSaved = localStorage.getItem(accountSettingsKey(settings.value.account));
        if (acctSaved) Object.assign(settings.value, JSON.parse(acctSaved));
      } catch (_) {}
    }
  }

  // 启动时应用已保存的界面语言
  await applyUiLang();

  // 如果没有保存的路径，自动解析 %APPDATA%\Firestorm_x64 的真实路�?
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

  // 自动开启同步逻辑（当存在合法设置时）
  if (settingsValid.value) {
    toggleListening();
  }

  // 加载上一次保留的 UI VDOM 会话状�?
  try {
    const savedState = await invoke('load_ui_state', { account: settings.value.account || '' });
    if (savedState) {
      chatTabs.value = JSON.parse(savedState);
    }
  } catch(e) {}

  let lastMsgHash = '';
  let lastMsgTime = 0;

  // 侦听后端日志推�?
  await TauriBridge.onLogUpdate(async (payload) => {
    const source = payload.source || 'chat.txt';
    const msgContext = payload.msg;

    if (!msgContext?.includes(': ')) return;
    const colonIdx = msgContext.indexOf(': ');
    const sender   = msgContext.slice(0, colonIdx);
    const text     = msgContext.slice(colonIdx + 2);
    
    // 基础去重（防�?Firestorm 多文件记录同一个消息）
    const msgHash = `${source}||${sender}||${text}`;
    const now = Date.now();
    if (msgHash === lastMsgHash && (now - lastMsgTime) < 2000) {
      return; 
    }
    lastMsgHash = msgHash;
    lastMsgTime = now;

    // ── 层层层屁屁屁局局尿尿尾尾尽尽尼尼尻尻尺尺 登录训练新游被拳�?尺尺尻尻尼尼尽尽尾尾尿尿局局屁屁层层
    const accountBaseName = settings.value.account ? settings.value.account.split('_')[0].toLowerCase() : '---';
    const isMySelf = sender.toLowerCase().includes(accountBaseName) || sender.toLowerCase() === 'me';
    const isSystemMessage = sender.toLowerCase() === 'second life';
    const isGroupFile = source.toLowerCase().includes('group');
    const skipGroupMessage = isGroupFile && !settings.value.translateGroup;

    // 系统上下线提示：直接丢弃，不渲染不翻�?
    if (isSystemMessage) return;

    // 如果未开启群聊翻译，且这是个群组日志：直接丢弃，不建 Tab 不渲�?
    if (skipGroupMessage) return;

    // 定位或创�?Tab
    let tabInfo = chatTabs.value.find(t => t.id === source);
    if (!tabInfo) {
      let title = source.replace(/\.txt$/i, '').replace(/\.log$/i, '');
      if (source.toLowerCase() === 'conversation.log' || source.toLowerCase() === 'chat.txt') {
        title = 'nearby';
      }
      tabInfo = { id: source, title, messages: [], hasUnread: true };
      chatTabs.value.push(tabInfo);
    } else {
      tabInfo.hidden = false;
    }
    
    // 标红�?
    if (activeChatTabId.value !== source) {
      tabInfo.hasUnread = true;
    }

    const newItem  = { time: new Date().toLocaleTimeString(), sender, text, translated: '' };
    tabInfo.messages.push(newItem);
    const reactiveItem = tabInfo.messages[tabInfo.messages.length - 1];
    
    persistState();
    if (activeChatTabId.value === source) scrollToBottom(true);

    // 自己发的、黑名单用户或频道：渲染上屏但不翻译
    const isBlacklisted = settings.value.blacklist.some(
      name => name.trim().toLowerCase() === sender.toLowerCase() || 
              name.trim().toLowerCase() === source.toLowerCase() || 
              name.trim().toLowerCase() === tabInfo.title.toLowerCase()
    );
    if (isMySelf || isBlacklisted) return;

    // 附近频道白名单过滤（私聊不受影响�?
    const PUBLIC_CHAT_FILES_WL = ['chat.txt', 'conversation.log'];
    const isNearbyChan = PUBLIC_CHAT_FILES_WL.includes(source.toLowerCase());
    if (isNearbyChan && settings.value.nearbyWhitelistEnabled) {
      const wl = settings.value.nearbyWhitelist;
      const senderLower = sender.toLowerCase();
      // 白名单匹配：用户名完全匹�?OR 括号格式 "Display (username)" 中的任一部分匹配
      const inWhitelist = wl.some(name => {
        const n = name.trim().toLowerCase();
        if (!n) return false;
        return senderLower === n || senderLower.includes(n);
      });
      if (!inWhitelist) return;
    }

    // 关键词屏蔽：消息原文中包含屏蔽词则不翻译
    const kbl = settings.value.keywordBlocklist;
    const textLower = text.toLowerCase();
    const hasBlockedKeyword = kbl.some(kw => {
      const k = kw.trim().toLowerCase();
      return k && textLower.includes(k);
    });
    if (hasBlockedKeyword) return;

    // 组织上文（将同一个对话频道里的前�?N 句当做参考喂给AI以保持连贯！�?
    const history = getHistoryContext(source);

    await LLMService.translateStream(text, history, (chunk) => {
      reactiveItem.translated += chunk;
    }, {
      apiKey:     settings.value.apiKey,
      baseUrl:    settings.value.baseUrl,
      model:      settings.value.model,
      targetLang: settings.value.recvLang,
    });

    // 翻译完成后滚到底部（无论翻译时用户是否滚动过�?
    if (activeChatTabId.value === source) scrollToBottom(true);

    // 公屏频道不存档，仅私聊保�?
    const PUBLIC_CHAT_FILES = ['chat.txt', 'conversation.log'];
    const isPublicChannel = PUBLIC_CHAT_FILES.includes(source.toLowerCase());

    if (!isPublicChannel && reactiveItem.translated.trim()) {
      invoke('append_translation_history', {
        account: settings.value.account || '',
        source,
        timestamp: reactiveItem.time,
        sender,
        text,
        translated: reactiveItem.translated
      }).catch(e => console.error("保存历史记录失败", e));
    }
    persistState();
  });

  // 侦听全局快捷�?
  await TauriBridge.onShortcutTrigger(async () => {
    inputShow.value = true;
    await nextTick();
    inputRef.value?.focus();
  });

  // 检查更�?
  UpdateService.checkUpdate().then(info => {
    if (info && info.hasUpdate) {
      updateAvailable.value = true;
      updateInfo.value = info;
    }
  });
});

watch([activeTab, isListening], ([newTab, newListening]) => {
  if (newTab === TAB_CHAT && newListening) {
    scrollToBottom();
  }
});

// ── 设置操作 ──────────────────────────────────────────────────────
// 全局设置键（API Key、模型等共享配置�?
const GLOBAL_SETTINGS_KEY = 'sl-translator-global';
// 账号级设置键（黑名单、接收语言等随账号变化�?
const accountSettingsKey = (account) => `sl-settings-${account}`;
// 全局字段清单
const GLOBAL_FIELDS = ['logDir', 'account', 'apiKey', 'baseUrl', 'model', 'uiLang', 'contextCount', 'translateGroup', 'viewerType', 'bgOpacity', 'nearbyWhitelistEnabled'];

const saveSettings = () => {
  const global = {};
  GLOBAL_FIELDS.forEach(k => { global[k] = settings.value[k]; });
  localStorage.setItem(GLOBAL_SETTINGS_KEY, JSON.stringify(global));

  if (settings.value.account) {
    const perAccount = {
      account:            settings.value.account,
      recvLang:           settings.value.recvLang,
      blacklist:          settings.value.blacklist,
      nearbyWhitelist:    settings.value.nearbyWhitelist,
      keywordBlocklist:   settings.value.keywordBlocklist,
    };
    localStorage.setItem(accountSettingsKey(settings.value.account), JSON.stringify(perAccount));
  }
};

// 切换账号时加载该账号的专属设�?
const loadAccountSettings = () => {
  if (!settings.value.account) return;
  try {
    const saved = localStorage.getItem(accountSettingsKey(settings.value.account));
    if (saved) {
      const parsed = JSON.parse(saved);
      // 只覆盖账号级字段，不影响 API Key 等全局字段
      settings.value.recvLang          = parsed.recvLang          ?? settings.value.recvLang;
      settings.value.blacklist          = parsed.blacklist          ?? [];
      settings.value.nearbyWhitelist    = parsed.nearbyWhitelist    ?? [];
      settings.value.keywordBlocklist   = parsed.keywordBlocklist   ?? [];
    } else {
      // 新账号：重置账号级字段为默认�?
      settings.value.blacklist        = [];
      settings.value.nearbyWhitelist  = [];
      settings.value.keywordBlocklist = [];
    }
  } catch (_) {}
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

// 切换客户端：自动填充日志目录并重新扫描账�?
const selectViewer = async (viewerType) => {
  settings.value.viewerType = viewerType;
  if (viewerType === 'custom') return;  // 自定义时不覆盖路�?
  try {
    const dir = await invoke('get_viewer_log_dir', { viewer: viewerType });
    if (dir) {
      settings.value.logDir = dir;
      await refreshAccounts();
    }
  } catch (e) {
    console.error('获取客户端路径失�?', e);
  }
};

// ── 监听控制 ──────────────────────────────────────────────────────
const testApiStatus = ref('idle');
const testApiMessage = ref('');

const testApiConnection = async () => {
  if (!settings.value.apiKey) {
    testApiStatus.value = 'error';
    testApiMessage.value = i18n.value.testApiNoKey;
    return;
  }
  
  testApiStatus.value = 'testing';
  testApiMessage.value = i18n.value.testApiConnecting;
  saveSettings();
  
  let result = '';
  await LLMService.translateStream('Test message connection please reply Hello', [], (chunk) => {
    result += chunk;
  }, {
    apiKey:     settings.value.apiKey,
    baseUrl:    settings.value.baseUrl,
    model:      settings.value.model,
    targetLang: 'English'
  });
  
  if (result.includes('[\u7f51\u7edc\u9519\u8bef]') || result.includes('[\u9519\u8bef]')) {
    testApiStatus.value = 'error';
    testApiMessage.value = '\u274c ' + result.replace('\n', '').trim();
  } else if (result.trim().length > 0) {
    testApiStatus.value = 'success';
    testApiMessage.value = i18n.value.testApiSuccess;
  } else {
    testApiStatus.value = 'error';
    testApiMessage.value = i18n.value.testApiFail;
  }
};

const toggleListening = async () => {
  errorMessage.value = '';
  
  if (isListening.value) {
    try {
      await TauriBridge.stopLogWatcher();
      isListening.value = false;
    } catch (e) {
      console.error(e);
    }
    return;
  }
  
  if (!settings.value.logDir || !settings.value.account) {
    activeTab.value = TAB_SETTINGS;
    errorMessage.value = i18n.value.settingsIncomplete;
    return;
  }
  saveSettings();
  try {
    const watchPath = `${settings.value.logDir}\\${settings.value.account}`;
    await TauriBridge.startLogWatcher(watchPath);
    isListening.value = true;
    activeTab.value   = TAB_CHAT;
  } catch (e) {
    errorMessage.value = i18n.value.engineStartFail + e;
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
  scrollToBottom(true);

  const otherMessages = getOtherRawMessages(activeChatTabId.value);
  const contextBlock = otherMessages.length > 0
    ? `这是对方发给我的最近消息（最�?5 条）\n${otherMessages.join('\n')}\n\n我想回复：`
    : ``;
  const sendText = contextBlock + text;

  let translatedResult = '';
  await LLMService.translateStream(sendText, [], (chunk) => {
    translatedResult  += chunk;
    reactiveItem.translated += chunk;
  }, {
    apiKey:     settings.value.apiKey,
    baseUrl:    settings.value.baseUrl,
    model:      settings.value.model,
    direction:  'send',
  });

  if (translatedResult.trim()) {
    invoke('append_translation_history', {
      account: settings.value.account || '',
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

// ── AI 气泡：建议回�?──────────────────────────────────────────
const getAiReplyKey = (tabId, msgIndex) => `${tabId}:${msgIndex}`;

const askHowToReply = async (msg, msgIndex) => {
  const tabId = activeChatTabId.value;
  const key = getAiReplyKey(tabId, msgIndex);
  if (aiReplyMap.value[key]?.loading) return;

  aiReplyMap.value[key] = { loading: true, text: '' };

  const tab = chatTabs.value.find(t => t.id === tabId);
  const contextMessages = tab ? tab.messages.slice(-8) : [];

  await LLMService.chatStream(
    `How should I reply to this message: "${msg.text}"`,
    contextMessages,
    (chunk) => { aiReplyMap.value[key].text += chunk; },
    { apiKey: settings.value.apiKey, baseUrl: settings.value.baseUrl, model: settings.value.model },
    'reply'
  );

  aiReplyMap.value[key].loading = false;
};

// ── AI 底栏对话助手 ────────────────────────────────────────────
const sendAiChat = async () => {
  const prompt = aiChatInput.value.trim();
  if (!prompt || aiChat.value.loading) return;

  aiChat.value = { loading: true, result: '' };
  aiChatInput.value = '';

  const tab = chatTabs.value.find(t => t.id === activeChatTabId.value);
  const contextMessages = tab ? tab.messages.slice(-8) : [];

  await LLMService.chatStream(
    prompt,
    contextMessages,
    (chunk) => { aiChat.value.result += chunk; },
    { apiKey: settings.value.apiKey, baseUrl: settings.value.baseUrl, model: settings.value.model },
    'chat'
  );

  aiChat.value.loading = false;
};

// ── 工具 ──────────────────────────────────────────────────────────
const scrollToBottom = (smooth = false) => {
  nextTick(() => {
    if (chatScrollRef.value) {
      if (smooth) {
        chatScrollRef.value.scrollTo({ top: chatScrollRef.value.scrollHeight, behavior: 'smooth' });
      } else {
        chatScrollRef.value.scrollTop = chatScrollRef.value.scrollHeight;
      }
    }
  });
};

const openHistoryFolder = async () => {
  await invoke('open_history_folder', { account: settings.value.account || '' });
};

const openTutorial = async () => {
  try {
    await openUrl('https://github.com/yarnbyte/AI-SecondLife-Talk/blob/main/README.md');
  } catch (e) {
    console.error('打开教程链接失败:', e);
  }
};
</script>

<template>
  <div class="app-root">

    <div v-if="updateAvailable" class="update-banner">
      <Sparkles :size="12" style="margin-right: 4px;" />
      <span>{{ i18n.updateAvailable.replace('{v}', updateInfo.remoteVersion) }}</span>
      <button class="btn-update" @click="UpdateService.openUpdateUrl(updateInfo.url)">{{ i18n.updateBtn }}</button>
      <button class="btn-update-close" @click="updateAvailable = false"><X :size="10" /></button>
    </div>

    <!-- ── 极光背景装饰 ────────────────────────────────────────── -->
    <div class="aurora-fx"></div>

    <!-- ── 顶栏 ──────────────────────────────────────────────── -->
    <header class="title-bar" @mousedown="handleDrag">
      <div class="brand">
        <Sparkles :size="15" class="brand-icon" />
        <span class="brand-name">{{ i18n.appTitle }}</span>
      </div>

      <div class="title-actions" @mousedown.stop>
        <!-- 同步状�?-->
        <button v-if="isListening" class="badge-listening" @click="toggleListening" style="cursor: pointer;" :title="i18n.stopListeningTitle">
          <Activity :size="12" class="pulse" /> {{ i18n.listeningInfo }}
        </button>
        <button v-else class="btn-start" @click="toggleListening">
          <Play :size="12" /> {{ i18n.startListening }}
        </button>

        <!-- 设置 -->
        <button
          class="ctrl-btn"
          :class="{ active: activeTab === TAB_SETTINGS }"
          :title="i18n.settingTitle"
          @click="activeTab = activeTab === TAB_SETTINGS ? TAB_CHAT : TAB_SETTINGS"
        >
          <Settings :size="14" />
        </button>

        <!-- 置顶 -->
        <button class="ctrl-btn ctrl-pin" :class="{ pinned: isPinned }" :title="isPinned ? i18n.unpinTitle : i18n.pinTitle" @click="togglePin">
          <Pin v-if="!isPinned" :size="14" />
          <PinOff v-else :size="14" />
        </button>

        <!-- 最小化 -->
        <button class="ctrl-btn" :title="i18n.minimizeTitle" @click="handleMinimize">
          <Minus :size="14" />
        </button>

        <!-- 关闭 -->
        <button class="ctrl-btn ctrl-close" :title="i18n.closeTitle" @click="handleClose">
          <X :size="14" />
        </button>
      </div>
    </header>

    <!-- ── 主体�?──────────────────────────────────────────────── -->
    <div class="main-area">

      <!-- 设置面板 -->
      <Transition name="slide-down">
        <section class="settings-panel" v-if="activeTab === TAB_SETTINGS">

          <div v-if="errorMessage" class="error-banner">
            <span style="white-space: pre-wrap;">{{ errorMessage }}</span>
          </div>

          <!-- �?连接 -->
          <div class="acc-group">
            <button class="acc-header" @click="toggleSection('connection')">
              <FolderOpen :size="13" />
              <span>{{ i18n.viewerLabel }} &amp; {{ i18n.slAccount }}</span>
              <span class="acc-arrow" :class="{ open: openSections.has('connection') }">�?/span>
            </button>
            <div v-show="openSections.has('connection')" class="acc-body">
              <!-- 客户�?-->
              <div class="form-section">
                <label class="form-label">
                  {{ i18n.viewerLabel }}
                  <button class="help-icon-btn" :title="i18n.tutorialTitle" @click="openTutorial"
                    style="background:none;border:none;cursor:pointer;padding:0 2px;vertical-align:middle;opacity:0.7;color:rgba(255,255,255,0.85);">
                    <HelpCircle :size="13" />
                  </button>
                </label>
                <div class="viewer-selector">
                  <button class="viewer-btn" :class="{ active: settings.viewerType === 'firestorm' }" @click="selectViewer('firestorm')">{{ i18n.viewerFirestorm }}</button>
                  <button class="viewer-btn" :class="{ active: settings.viewerType === 'official' }"   @click="selectViewer('official')">{{ i18n.viewerOfficial }}</button>
                  <button class="viewer-btn" :class="{ active: settings.viewerType === 'custom' }"     @click="selectViewer('custom')">{{ i18n.viewerCustom }}</button>
                </div>
                <div class="input-row" v-if="settings.viewerType === 'custom'" style="margin-top:6px">
                  <input v-model="settings.logDir" class="form-input" :placeholder="i18n.logDirCustomHint" @change="refreshAccounts" />
                  <button class="btn-browse" @click="browseLogDir">{{ i18n.browseLabel }}</button>
                </div>
              </div>
              <!-- 账号 -->
              <div class="form-section" style="margin-bottom:0">
                <label class="form-label"><User :size="13" /> {{ i18n.slAccount }}</label>
                <div class="input-row">
                  <div class="select-wrap" style="flex:1">
                    <select v-model="settings.account" class="form-select" @change="loadAccountSettings">
                      <option value="" disabled>{{ i18n.slAccountDropTip }}</option>
                      <option v-for="acc in accountList" :key="acc" :value="acc">{{ acc }}</option>
                    </select>
                    <ChevronDown :size="14" class="select-chevron" />
                  </div>
                  <button class="btn-browse" @click="refreshAccounts" :title="i18n.refreshAccounts">&#x21bb;</button>
                </div>
                <div class="form-hint" v-if="accountList.length === 0">{{ i18n.slAccountNoDirHint }}</div>
              </div>
            </div>
          </div>

          <!-- �?API 设置 -->
          <div class="acc-group">
            <button class="acc-header" @click="toggleSection('api')">
              <KeyRound :size="13" />
              <span>API</span>
              <span class="acc-arrow" :class="{ open: openSections.has('api') }">�?/span>
            </button>
            <div v-show="openSections.has('api')" class="acc-body">
              <div class="form-section">
                <label class="form-label">{{ i18n.apiKeyLabel }}</label>
                <input v-model="settings.apiKey" type="password" class="form-input" placeholder="sk-xxxx..." />
              </div>
              <div class="form-section">
                <label class="form-label">{{ i18n.baseUrlLabel }}</label>
                <input v-model="settings.baseUrl" class="form-input" placeholder="https://api.deepseek.com/v1" />
              </div>
              <div class="form-section" style="margin-bottom:0">
                <label class="form-label">{{ i18n.modelLabel }}</label>
                <div class="input-row">
                  <input v-model="settings.model" class="form-input" placeholder="gpt-4o-mini" />
                  <button class="btn-browse" @click="testApiConnection" :disabled="testApiStatus === 'testing'" style="min-width: 76px;">
                    {{ testApiStatus === 'testing' ? i18n.testApiTesting : i18n.testApiBtn }}
                  </button>
                </div>
                <div v-if="testApiMessage" class="form-hint" :style="{ color: testApiStatus === 'success' ? '#10b981' : (testApiStatus === 'error' ? '#ef4444' : '#fbbf24') }" style="margin-top:6px;">
                  {{ testApiMessage }}
                </div>
              </div>
            </div>
          </div>

          <!-- �?翻译参数 -->
          <div class="acc-group">
            <button class="acc-header" @click="toggleSection('translate')">
              <MessageSquareDot :size="13" />
              <span>{{ i18n.secTranslation }}</span>
              <span class="acc-arrow" :class="{ open: openSections.has('translate') }">�?/span>
            </button>
            <div v-show="openSections.has('translate')" class="acc-body">
              <div class="form-section">
                <label class="form-label">{{ i18n.recvLangConfig }}</label>
                <input v-model="settings.recvLang" class="form-input" placeholder="Simplified Chinese / English / etc..." />
              </div>
              <div class="form-section">
                <label class="form-label">{{ i18n.ctxCountSetting }}</label>
                <input v-model.number="settings.contextCount" type="number" class="form-input" placeholder="5" />
              </div>
              <div class="form-section" style="margin-bottom:0">
                <label class="form-label" style="text-transform:none;letter-spacing:0;font-weight:500;">
                  <input type="checkbox" v-model="settings.translateGroup" style="vertical-align:middle;margin-right:5px;" />
                  {{ i18n.groupCb }}
                </label>
              </div>
            </div>
          </div>

          <!-- �?过滤规则 -->
          <div class="acc-group">
            <button class="acc-header" @click="toggleSection('filter')">
              <BellOff :size="13" />
              <span>{{ i18n.secFilter }}</span>
              <span class="acc-arrow" :class="{ open: openSections.has('filter') }">�?/span>
            </button>
            <div v-show="openSections.has('filter')" class="acc-body">
              <div class="form-section">
                <label class="form-label" style="text-transform:none;letter-spacing:0;font-weight:500;">
                  <input type="checkbox" v-model="settings.nearbyWhitelistEnabled" style="vertical-align:middle;margin-right:5px;" />
                  {{ i18n.nearbyWhitelistCb }}
                </label>
                <textarea v-if="settings.nearbyWhitelistEnabled"
                  class="form-input"
                  style="margin-top:6px;resize:vertical;min-height:65px;font-size:12px;line-height:1.5;"
                  v-model="nearbyWhitelistRaw"
                  :placeholder="i18n.nearbyWhitelistPlc"
                />
              </div>
              <div class="form-section" style="margin-bottom:0">
                <label class="form-label">{{ i18n.keywordBlocklistLabel }}</label>
                <textarea class="form-input"
                  style="resize:vertical;min-height:55px;font-size:12px;line-height:1.5;"
                  v-model="keywordBlocklistRaw"
                  :placeholder="i18n.keywordBlocklistPlc"
                />
              </div>
            </div>
          </div>

          <!-- �?界面 -->
          <div class="acc-group">
            <button class="acc-header" @click="toggleSection('appearance')">
              <Settings :size="13" />
              <span>{{ i18n.secAppearance }}</span>
              <span class="acc-arrow" :class="{ open: openSections.has('appearance') }">�?/span>
            </button>
            <div v-show="openSections.has('appearance')" class="acc-body">
              <div class="form-section">
                <label class="form-label" style="display:flex;justify-content:space-between;">
                  <span>{{ i18n.windowOpacityLabel }}</span>
                  <span style="opacity:0.7;">{{ Math.round(settings.bgOpacity * 100) }}%</span>
                </label>
                <input type="range" v-model.number="settings.bgOpacity" min="0" max="1" step="0.05"
                  style="width:100%;accent-color:var(--accent);cursor:pointer;" />
              </div>
              <div class="form-section" style="margin-bottom:0">
                <label class="form-label">{{ i18n.uiLangLabel }}</label>
                <div class="select-wrap">
                  <select v-model="settings.uiLang" class="form-select" @change="applyUiLang">
                    <option value="zh-CN">简体中�?/option>
                    <option value="en-US">English</option>
                    <option value="custom">Custom (i18n.json)</option>
                  </select>
                  <ChevronDown :size="14" class="select-chevron" />
                </div>
              </div>
            </div>
          </div>

          <!-- 底部操作 -->
          <div style="display:flex;gap:8px;margin-top:12px;">
            <button class="btn-secondary" @click="openHistoryFolder" :title="i18n.viewHistory">
              <BookText :size="14" />
            </button>
            <button class="btn-secondary" @click="openTutorial" :title="i18n.tutorialTitle">
              <HelpCircle :size="14" />
            </button>
            <button class="btn-save" style="flex:1;" @click="saveSettings">{{ i18n.saveBtn }}</button>
          </div>

        </section>
      </Transition>


      <!-- API Key 快速条（开启监听后若未填会提示�?-->

      <div class="apikey-bar" v-if="isListening && !settings.apiKey">
        <KeyRound :size="13" class="apikey-icon" />
        <span class="apikey-hint">{{ i18n.apiKeyFloatTip }}</span>
        <input
          v-model="settings.apiKey"
          type="password"
          class="apikey-input"
          placeholder="sk-xxxx..."
          @change="saveSettings"
        />
      </div>

      <!-- 选项卡展示区 -->
      <div class="chat-tab-bar" v-if="activeTab === TAB_CHAT && isListening" ref="tabBarRef" @wheel.prevent="handleTabScroll">
        <div 
          v-for="t in visibleTabs" 
          :key="t.id" 
          class="chat-tab-item" 
          :class="{ active: activeChatTabId === t.id, unread: t.hasUnread }"
          @click="switchTab(t.id)">
          <span class="tab-title">{{ (t.id === 'chat.txt' || t.id === 'conversation.log') ? i18n.nearbyTab : t.title }}</span>
          <button class="tab-close tab-ctrl-btn" :title="i18n.muteChanTip" @click.stop="toggleBlacklist(t.id)">
            <BellOff v-if="isTargetBlacklisted(t.id)" :size="10" />
            <Bell v-else :size="10" />
          </button>
          <button class="tab-close" @click.stop="closeTab(t.id)" v-if="t.id !== 'chat.txt' && t.id !== 'conversation.log'"><X :size="10" /></button>
        </div>
      </div>

      <!-- 聊天面板 + 底部输入：整�?v-show，避免设置页打开时底部输入浮�?-->
      <div v-show="activeTab === TAB_CHAT" style="display:flex;flex-direction:column;flex:1;overflow:hidden;min-height:0;">

        <div class="chat-list" ref="chatScrollRef" @scroll="handleChatScroll">
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
          <div class="message-bubble group">
            <div class="source-text">{{ msg.text }}</div>
            <div class="translate-text" v-if="msg.translated && msg.translated.trim() !== msg.text.trim()">{{ msg.translated }}</div>
            <!-- 气泡操作�?-->
            <div class="bubble-actions">
              <button class="bubble-action-btn" @click="copyContent(msg.text)" :title="i18n.copyOriginal">
                <Copy :size="11" />
              </button>
              <button class="bubble-action-btn" v-if="msg.translated && msg.translated !== msg.text" @click="copyContent(msg.translated)" :title="i18n.copyTranslation">
                <MessageSquareDot :size="11" />
              </button>
              <button
                class="bubble-action-btn"
                v-if="msg.sender !== (settings.account || 'Me')"
                @click="askHowToReply(msg, i)"
                :title="i18n.suggestReply"
                :class="{ 'loading': aiReplyMap[`${activeChatTabId}:${i}`]?.loading }"
              >
                <Sparkles :size="11" />
              </button>
            </div>
          </div>
          <!-- AI 建议回复展开�?-->
          <div
            v-if="aiReplyMap[`${activeChatTabId}:${i}`]"
            class="ai-reply-box"
          >
            <div class="ai-reply-content">
              <span v-if="aiReplyMap[`${activeChatTabId}:${i}`].loading" class="ai-thinking">�?思考中�?/span>
              <span v-else>{{ aiReplyMap[`${activeChatTabId}:${i}`].text }}</span>
            </div>
            <button
              v-if="!aiReplyMap[`${activeChatTabId}:${i}`].loading && aiReplyMap[`${activeChatTabId}:${i}`].text"
              class="bubble-action-btn"
              @click="copyContent(aiReplyMap[`${activeChatTabId}:${i}`].text)"
              :title="i18n.copyResult"
            >
              <Copy :size="11" />
            </button>
          </div>
        </div>
      </div>

      <!-- 底部 AI 对话助手 -->
      <div class="ai-chat-panel" v-if="activeTab === TAB_CHAT && isListening">
        <div v-if="isTargetBlacklisted(activeChatTabId)" class="blacklist-overlay-msg">
          {{ i18n.mutedOverlay }}
        </div>
        <template v-else>
          <!-- AI 回复结果�?-->
          <div v-if="aiChat.result || aiChat.loading" class="ai-chat-result">
            <div class="ai-chat-result-text">
              <span v-if="aiChat.loading && !aiChat.result" class="ai-thinking">�?思考中�?/span>
              <span v-else>{{ aiChat.result }}</span>
            </div>
            <div class="ai-chat-result-actions">
              <button v-if="aiChat.result" class="bubble-action-btn" @click="copyContent(aiChat.result)" :title="i18n.copyResult">
                <Copy :size="11" /> {{ i18n.copyResult }}
              </button>
              <button class="bubble-action-btn danger" @click="aiChat = { loading: false, result: '' }" :title="i18n.clearResult">
                <X :size="11" /> {{ i18n.clearResult }}
              </button>
            </div>
          </div>
          <!-- 输入�?-->
          <div class="ai-chat-input-row">
            <textarea
              class="inline-input ai-chat-textarea"
              v-model="aiChatInput"
              :placeholder="i18n.aiChatPlc"
              rows="1"
              @keydown.enter.exact.prevent="sendAiChat"
              @keydown.shift.enter.exact="() => {}"
              :disabled="aiChat.loading"
            />
            <button class="btn-send-inline" @click="sendAiChat" :disabled="aiChat.loading">
              <Send :size="14" />
            </button>
          </div>
        </template>
      </div>

      </div><!-- /chat wrapper v-show -->

    </div>
  </div>
</template>
