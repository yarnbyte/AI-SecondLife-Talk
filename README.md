# AI SLtalk (<a href="./docs/AI_SLtalk_使用教程.md">中文使用教程</a>)

![AI SLtalk](src-tauri/icons/128x128@2x.png)

A non-intrusive, automated AI translation relay and assistant for **Second Life**, built with **Tauri v2** + **Vue 3**.

## Features

- **Headless Silence:** Does not require you to paste translated messages heavily into the application manually. Just type, and AI SLtalk automatically translates using state-of-the-art LLMs (like OpenAI/DeepSeek) and copies the result flawlessly into your clipboard.
- **Two Dual-Mode Operations:**
  - **Local Game Logs:** Scans your locally stored Firestorm chat logs for complete passiveness and 0 impact on gaming networks.
  - **High-speed LSL HUD Relay:** Supports fetching instantaneous HUD local chats through Ngrok local POST channels to guarantee lightning-speed nearby conversation coverage without local storage delay!
- **Anti-Spam & Deduplication Filter:** Actively cleans up duplicate entries commonly blasted by SL game files saving mechanics.
- **Seamless System Notifications Blacklisting:** Ignore system updates or targeted nearby chatbots with one click.
- **Dual-language Embedded UI:** Interactive UI and built-in tutorials supporting both English and Simplified Chinese out-of-the-box.

## Quick Start
You can check the comprehensive tutorial (Chinese) in our documentation folder to get started with fetching AI keys, Ngrok, and LSL HUD setups:
[📖 AI_SLtalk_使用教程.md](./docs/AI_SLtalk_使用教程.md)

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
