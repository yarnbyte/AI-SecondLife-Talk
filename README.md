# AI SLtalk

![AI SLtalk](src-tauri/icons/128x128@2x.png)

A non-intrusive, automated AI translation relay and assistant for **Second Life**, built with **Tauri v2** + **Vue 3**.

*Scroll down for Chinese documentation / 向下滚动查看中文使用教程*

---

## Features (特性)

- **Headless Silence:** Does not require you to paste translated messages heavily into the application manually. Just type, and AI SLtalk automatically translates using state-of-the-art LLMs (like OpenAI/DeepSeek) and copies the result flawlessly into your clipboard.
- **Two Dual-Mode Operations:**
  - **Local Game Logs:** Scans your locally stored Firestorm chat logs for complete passiveness and 0 impact on gaming networks.
  - **High-speed LSL HUD Relay:** Supports fetching instantaneous HUD local chats through Ngrok local POST channels to guarantee lightning-speed nearby conversation coverage without local storage delay!
- **Anti-Spam & Deduplication Filter:** Actively cleans up duplicate entries commonly blasted by SL game files saving mechanics.
- **Seamless System Notifications Blacklisting:** Ignore system updates or targeted nearby chatbots with one click.
- **Dual-language Embedded UI:** Interactive UI and built-in tutorials supporting both English and Simplified Chinese out-of-the-box.

---

## 快速使用指南 (中文教程)

欢迎使用 **AI SLtalk**！这款全自动翻译辅助工具可通过静默读取 Firestorm 的控制台日志，或通过 LSL 脚本直接中继来实现翻译体验。

### 1. 基础配置
1. 获取翻译引擎 API。你需要拥有一个兼容 OpenAI 格式的 API（如 OpenAI 官方、DeepSeek、千问等）：
   - **Base URL**（例如：`https://api.deepseek.com/v1`）
   - **API Key**（例如：`sk-xxxxxxx...`）
   - **模型名称/Model**（例如：`deepseek-chat` 等）
2. **填写 Firestorm 日志目录**：程序内默认路径通常为 `%AppData%\Firestorm_x64`。
3. **选择 SL 账号**：在下拉列表中选中你需要翻译的登录账号。
4. **填写 API 信息**：填入你刚才准备好的 API 数据。
5. **开启监听**：点击软件顶栏的“▶️ 开启监听”按钮。只要接收到信息（无论是周围环境发言还是他人私聊），都会自动翻译并实时的呈现在对应频道列表中；在底部输入框输入中文按回车会自动翻译并复制进剪贴板。

### 2. 公屏 LSL 脚本中继模式（搭配 ngrok）

对于多设备或追求极限极低延迟无丢包的用户，你可以使用公屏实时推流。

#### 2.1 下载并配置 ngrok
1. 去 [ngrok.com](https://ngrok.com/) 注册一个免费账号并下载对应系统的 `ngrok.exe`。
2. 打开 CMD/PowerShell 窗口，绑定鉴权：
   ```bash
   ngrok config add-authtoken 你的token码
   ```

#### 2.2 启动内网穿透
在命令提示符里输入：
```bash
ngrok http 29853
```
将屏幕上生成的 `https://xxx.ngrok.app` 地址复制下来。

#### 2.3 在 AI SLtalk 与 SL 中打通链路
1. 打开 AI SLtalk 设置页，开启 **🛸 开启 LSL HUD 公共频道中继**。
2. 将刚刚复制的 **公网网址粘贴在空框内**并保存。
3. 在 AI SLtalk 设置中点击 **📝 复制 LSL 脚本到剪贴板**。
4. 登录 SL，新建一个基础物品穿在 HUD 槽位上，往内容中新建 Script 并**将脚本代码全选覆盖粘贴**后保存。
5. 佩戴该物品，聊天内容会自动秒发至你的翻译器中。

### 3. FAQ 常见问题
**Q：为什么发送信息会被复制给别人/自己翻译一遍？**
A：可以直接在频道右上角点击 🔔 铃铛图标将其拉黑静音。

**Q：群组记录到底开不开？**
A：默认关闭群组。开启将消耗大量 API 额度，建议轻度选用。

---

## Development Setup

We recommend utilizing [VS Code](https://code.visualstudio.com/) with Tauri and Rust-analyzer extensions.

1. Ensure standard Tauri v2 system prerequisites are fulfilled (Rust, MSVC, Node.js).
2. Install dependencies:
   ```bash
   npm install
   ```
3. Run Development server:
   ```bash
   npm run tauri dev
   ```
4. Build Distribution:
   ```bash
   npm run tauri build
   ```

## License

This project is licensed under the MIT License.
