# AI SLtalk

![AI SLtalk](src-tauri/icons/128x128@2x.png)

> 专为 **Second Life** 打造的全自动 AI 实时翻译助手，全面支持 **官方客户端 (Official Viewer)** 与 **Firestorm Viewer**。
>
> An automated, AI-powered real-time translation assistant for **Second Life**, fully compatible with both the **Official Viewer** and **Firestorm Viewer**.

*[Click here for English documentation](https://github.com/yarnbyte/AI-SecondLife-Talk?tab=readme-ov-file#features-english)*

---

## 📺 演示视频 / Demo

[![Watch the demo](https://img.youtube.com/vi/5cG9vXumbLo/maxresdefault.jpg)](https://www.youtube.com/watch?v=5cG9vXumbLo)

---

## ✨ 核心特性

| 特性 | 说明 |
|---|---|
| 🔍 **零侵入日志同步** | 直接读取 Firestorm 自动保存的聊天记录文件，无需改动游戏、无需插件 |
| 🤖 **兼容任意 OpenAI 格式 API** | 开箱支持 OpenAI、DeepSeek、阿里千问 Qwen、本地 Ollama 等模型 |
| 💬 **按频道分 Tab 展示** | 附近频道、私聊频道、群聊频道各自独立显示，一目了然 |
| 📋 **一键翻译发送** | 底部输入框输入母语，回车自动翻译并复制到剪贴板，直接粘贴进 SL 即可 |
| 🔔 **频道静音黑名单** | 对机器人/系统通知频道单独静音，不影响其他频道正常翻译 |
| 📌 **全局置顶** | 固定在游戏画面上方，随时查看翻译，不影响操作 |
| 🌐 **中/英双语界面** | 内置简体中文和英文 UI，支持自定义语言包扩展 |
| 💾 **翻译历史存档** | 所有翻译记录自动写入本地文件，随时查阅 |
| 🧠 **上下文感知翻译** | 携带对话历史上文，翻译更贴近语境，减少歧义 |

---

## 🚀 快速上手（中文）

### 第一步：配置 Firestorm

在 Firestorm 客户端中，前往：

**Preferences（首选项）→ Privacy（隐私）→ Logs & Transcripts（日志与记录）**

勾选 ✅ **Save nearby chat transcript（保存附近聊天记录）**  
勾选 ✅ **Log instant messages（记录私信）**

> ⚠️ **必须开启此选项**，AI SLtalk 才能读取到聊天记录。

---

### 第二步：准备 AI 翻译 API

你需要一个兼容 **OpenAI Chat API** 格式的服务，推荐以下选项：

| 服务 | Base URL | 特点 |
|---|---|---|
| OpenAI 官方 | `https://api.openai.com/v1` | 最稳定，需付费 |
| DeepSeek | `https://api.deepseek.com/v1` | 性价比高，中文优秀 |
| 阿里千问 Qwen | `https://dashscope.aliyuncs.com/compatible-mode/v1` | 国内访问快 |
| 本地 Ollama | `http://localhost:11434/v1` | 完全免费离线 |

你需要准备好：
- **API Key**（如 `sk-xxxxxxxxxxxxxxxx`）
- **Base URL**（如 `https://api.deepseek.com/v1`）
- **模型名称**（如 `deepseek-chat`、`gpt-4o-mini`）

---

### 第三步：软件设置

打开 AI SLtalk，点击右上角 ⚙️ **设置** 图标，填入以下信息：

1. **日志目录**：默认路径通常为 `C:\Users\<你的用户名>\AppData\Roaming\Firestorm_x64`（Firestorm 客户端）或 `C:\Users\<你的用户名>\AppData\Roaming\SecondLife`（官方客户端）。如果你自己修改过安装或日志路径，请自行导航到对应目录点击选择。
2. **SL 账号**：从下拉列表中选择你的 SL 登录账号文件夹。
3. **API Key / Base URL / 模型**：填入上一步准备好的 API 信息，填完后点击 **"测试 API"** 按钮验证是否连通。
4. **翻译目标语言**：填写你希望将对方消息翻译成的语言（如 `Simplified Chinese`）。
5. 点击 💾 **保存设置**。

---

### 第四步：启动翻译

点击顶栏的 **▶ 启动翻译** 按钮。

状态指示灯变为 🟢 **工作中** 后，软件即已开始同步 Firestorm 聊天记录：

- **附近频道** / **私聊频道** 的消息会自动按 Tab 分类显示。
- 每条消息将自动由 AI 翻译，原文与译文同时呈现。
- 在底部输入框输入你的母语，按 **回车** / 点击发送按钮，即可自动翻译为对方语言并**复制到剪贴板**，粘贴到 SL 发送即可。

---

## 🎛️ 功能详解

### 频道 Tab 管理

- 每个独立的聊天对象或群组对应一个 Tab，点击即可切换。
- 单个频道点击 🔔 铃铛图标可**静音（免打扰）**，停止该频道的翻译与通知，再次点击恢复。
- 点击 ❌ 可关闭不需要的 Tab。

### 群聊翻译

默认**关闭**群聊翻译（群聊频道不会显示 Tab）。  
如需开启，在设置中勾选 **"开启群聊同步及翻译"**，带有 `group` 字样的日志文件将被纳入同步范围。

> ⚠️ 群聊消息量大，开启后 API 消耗会显著增加，请酌情选用。

### 全局置顶

点击 📌 图标将软件固定在所有窗口最上层，方便在游戏全屏时叠加查看翻译。  
再次点击取消置顶。

### 翻译历史存档

点击右上角 📂 图标，可打开本地翻译历史记录文件夹，所有翻译结果按频道分文件保存。

---

## ❓ 常见问题 FAQ

**Q：自己发出的消息也被翻译了怎么办？**  
A：点击该频道 Tab 上的 🔔 铃铛图标，将其静音即可屏蔽这个频道的翻译。

**Q：切出游戏后，在软件上滚动很卡顿？**  
A：正常现象。当软件失去焦点时，操作系统会降低其刷新率以省电。点击一下软件界面重新获得焦点，即可恢复流畅。

**Q：群组频道不显示怎么解决？**  
A：在 ⚙️ 设置中勾选 **"开启群聊同步及翻译"**，保存后重新启动翻译即可。

**Q：翻译没反应，API 连不上？**  
A：在设置填好 API Key 和 Base URL 后，点击 **"测试 API"** 按钮，系统会返回连接是否成功及详细报错信息。

**Q：日志目录在哪里？**  
A：推荐在游戏客户端的 Preferences（首选项） → Privacy（隐私） → Logs & Transcripts（日志与记录）页面中直接查看并复制保存路径。
默认情况：Firestorm 路径为 `C:\Users\<你的用户名>\AppData\Roaming\Firestorm_x64`，官方客户端位于 `C:\Users\<你的用户名>\AppData\Roaming\SecondLife`。

---

## 🛠️ 开发者指南

本项目使用 **Tauri v2 + Vue 3 + Rust** 构建，推荐使用 [VS Code](https://code.visualstudio.com/) + Tauri / Rust Analyzer 插件开发。

**环境依赖：**
- [Node.js](https://nodejs.org/) 18+
- [Rust](https://rustup.rs/) (MSVC toolchain on Windows)
- Tauri CLI v2

```bash
# 安装依赖
npm install

# 启动开发模式
npm run tauri dev

# 构建发行版
npm run tauri build
```

---

## 📄 开源许可

本项目基于 **MIT License** 开源，欢迎 Fork、Star 和 PR。

---

## Features (English)

| Feature | Description |
|---|---|
| 🔍 **Non-intrusive Log Sync** | Reads Firestorm's auto-saved chat log files silently. No game modifications required. |
| 🤖 **Any OpenAI-compatible API** | Works with OpenAI, DeepSeek, Qwen, local Ollama, and more out of the box. |
| 💬 **Per-channel Tab View** | Nearby chat, IMs, and group chats are shown in separate tabs. |
| 📋 **One-click Translate & Send** | Type in your native language, press Enter — auto-translated and copied to clipboard. |
| 🔔 **Channel Mute / Blacklist** | Silence noisy bot/system channels individually without affecting others. |
| 📌 **Always-on-top** | Pin the window above your game view for seamless overlay use. |
| 🌐 **Bilingual UI** | Built-in Simplified Chinese and English interface. Custom language packs supported. |
| 💾 **Translation Archive** | All translated conversations are saved locally by channel. |
| 🧠 **Context-aware Translation** | Sends conversation history to the AI for more accurate, natural translations. |

## 🚀 Quick Start (English)

### Step 1: Configure Your Viewer

In your Second Life Viewer (Firestorm or Official), navigate to:

**Preferences → Privacy → Logs & Transcripts**

Check ✅ **Save nearby chat transcript**  
Check ✅ **Log instant messages**

> ⚠️ **This must be enabled** for AI SLtalk to read your chat logs.

---

### Step 2: Prepare the AI Translation API

You need a service compatible with the **OpenAI Chat API** format. Recommended options include:

| Service | Base URL | Features |
|---|---|---|
| Official OpenAI | `https://api.openai.com/v1` | Most stable, paid |
| DeepSeek | `https://api.deepseek.com/v1` | Cost-effective, excellent for Chinese |
| Alibaba Qwen | `https://dashscope.aliyuncs.com/compatible-mode/v1` | Fast access in Asia |
| Local Ollama | `http://localhost:11434/v1` | Completely free & offline |

You will need:
- **API Key** (e.g., `sk-xxxxxxxxxxxxxxxx`)
- **Base URL** (e.g., `https://api.deepseek.com/v1`)
- **Model Name** (e.g., `deepseek-chat`, `gpt-4o-mini`)

---

### Step 3: Software Settings

Open AI SLtalk, click the ⚙️ **Settings** icon in the top right, and fill in the following:

1. **Log Directory**: Normally located at `C:\Users\<YourUsername>\AppData\Roaming\Firestorm_x64` (for Firestorm Viewer) or `C:\Users\<YourUsername>\AppData\Roaming\SecondLife` (for Official Viewer). If you changed your installation or log paths, please navigate to your custom directory instead.
2. **SL Account**: Select your SL account folder from the dropdown menu.
3. **API Key / Base URL / Model**: Fill in the API information prepared in Step 2. Click the **"Test API"** button to verify the connection.
4. **Target Language**: Enter the language you want incoming messages translated into (e.g., `English`).
5. Click 💾 **Save Settings**.

---

### Step 4: Start Translator

Click the **▶ Start Translator** button in the top bar.

When the status indicator turns to 🟢 **Active**, the application has started syncing your chat logs:

- Messages from **Nearby Chat** and **Instant Messages (IMs)** will automatically appear and be categorised into tabs.
- Every message will be automatically translated by the AI, showing both the original text and the translation.
- Type your native language in the bottom input box and press **Enter** (or click send) to automatically translate it into the target language and **copy it to your clipboard**, ready to be pasted back into Second Life.

---

## 🎛️ Feature Details

### Channel Tab Management

- Each chat contact or group has an independent Tab. Click to switch between them.
- Click the 🔔 bell icon on a channel to **Mute** it. This stops translation and notifications for that channel. Click again to unmute.
- Click the ❌ icon to close unneeded tabs.

### Group Chat Translation

By default, group chat translation is **disabled** (group channels will not show tabs).  
To enable it, check **"Enable group chat translation"** in the settings. Log files containing the word `group` will then be included in the sync.

> ⚠️ Group chats have a high volume of messages. Enabling this will significantly increase API usage. Use with caution.

### Always-on-top

Click the 📌 pin icon to keep the software fixed on top of all other windows, allowing you to view translations comfortably while prioritizing your game window.  
Click it again to unpin.

### Translation Archive

Click the 📂 folder icon in the top right to open your local translation history directory. All translated results are saved and organized by channel.

---

## ❓ FAQ

**Q: My own sent messages are being translated again, what should I do?**  
A: You can mute the channel by clicking the 🔔 bell icon on the top right of the channel tab.

**Q: After switching to the game, scrolling the translator window feels laggy?**  
A: This is normal. When the software loses focus, the operating system limits its frame rate to save power. Just click anywhere on the translator window to regain focus and restore smooth 60fps scrolling.

**Q: Why are group channels not showing up?**  
A: Go to ⚙️ Settings and check **"Enable group chat translation"**, then click save and restart the translator.

**Q: Translation is not responding, can't connect to API?**  
A: After filling in the API Key and Base URL, click the **"Test API"** button. The system will return a detailed error message if the connection fails.

**Q: Where is my chat log directory located?**  
A: The default path for Firestorm is `C:\Users\<YourUsername>\AppData\Roaming\Firestorm_x64`, and for the Official Viewer it's `C:\Users\<YourUsername>\AppData\Roaming\SecondLife`. You can also find your custom paths in your viewer's Preferences → Privacy → Logs & Transcripts tab.

---

## 🛠️ Developer Guide

Built with **Tauri v2 + Vue 3 + Rust**. Recommended IDE: [VS Code](https://code.visualstudio.com/) + Tauri / Rust Analyzer extensions.

**Prerequisites:**
- [Node.js](https://nodejs.org/) 18+
- [Rust](https://rustup.rs/) (MSVC toolchain on Windows)
- Tauri CLI v2

```bash
# Install dependencies
npm install

# Start development mode
npm run tauri dev

# Build distribution
npm run tauri build
```

---

## 📄 License

This project is open-sourced under the **MIT License**. We welcome Forks, Stars, and Pull Requests.
