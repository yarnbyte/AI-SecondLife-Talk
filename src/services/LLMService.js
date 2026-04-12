import { API_DEFAULTS } from '../utils/constants';

export class LLMService {
    /**
     * 流式调用大模型
     * @param {string} text - 需要翻译的本文
     * @param {Array} historyContext - 上下文历史列表 [{role: 'user', content: '...'}...]
     * @param {Function} onChunk - 收到字符的回调，用于更新界面: (chunkText) => {}
     * @param {Object} config - { apiKey: string, baseUrl: string, model: string, targetLang: string }
     */
    static async translateStream(text, historyContext = [], onChunk, config) {
        const apiKey = config.apiKey;
        let baseUrl = config.baseUrl || API_DEFAULTS.BASE_URL;
        
        // 智能补全 /chat/completions，兼容有些用户只填 /v1 的情况
        if (baseUrl && !baseUrl.includes('/chat/completions')) {
            baseUrl = baseUrl.replace(/\/+$/, '') + '/chat/completions';
        }

        const model = config.model || API_DEFAULTS.MODEL;
        const targetLang = config.targetLang || 'English';
        const isSend = config.direction === 'send';

        if (!apiKey) {
            onChunk("[错误] 请先配置 API Key");
            return;
        }

        const systemPrompt = isSend 
            ? `You are an expert translator specializing in the virtual world of Second Life (SL).
You seamlessly translate the provided text to match the language predominantly used by the OTHER person in the history context.
If the context is empty or unclear, translate it to English.
Interpret SL-specific slang (e.g., TP, LM, Sim, Rez, IM, Lindens). Do NOT output notes or explanations, ONLY the translation.`
            : `You are an expert translator specializing in the virtual world of Second Life (SL).
You seamlessly translate the provided text to ${targetLang}, interpreting SL-specific slang (e.g., TP, LM, Sim, Rez, IM, Lindens).
Maintain the casual or Roleplay (RP) tone as required. Do NOT output notes or explanations, ONLY the translation.`;

        const messages = [
            {
                role: "system",
                content: systemPrompt
            },
            ...historyContext,
            {
                role: "user",
                content: text
            }
        ];

        await LLMService._doStream(messages, apiKey, baseUrl, model, onChunk);
    }

    /**
     * 通用 AI 对话流（用于底栏 AI 助手和气泡"如何回复"功能）
     * @param {string} userPrompt - 用户指令
     * @param {Array} contextMessages - 当前 tab 最近聊天记录 [{sender, text, translated}]
     * @param {Function} onChunk - 流式回调
     * @param {Object} config - { apiKey, baseUrl, model }
     * @param {string} mode - 'reply'(建议回复) | 'chat'(自由对话)
     */
    static async chatStream(userPrompt, contextMessages = [], onChunk, config, mode = 'chat') {
        const apiKey = config.apiKey;
        let baseUrl = config.baseUrl || API_DEFAULTS.BASE_URL;
        if (baseUrl && !baseUrl.includes('/chat/completions')) {
            baseUrl = baseUrl.replace(/\/+$/, '') + '/chat/completions';
        }
        const model = config.model || API_DEFAULTS.MODEL;

        if (!apiKey) {
            onChunk("[错误] 请先配置 API Key");
            return;
        }

        const contextSummary = contextMessages
            .slice(-8)
            .map(m => `${m.sender}: ${m.text}${m.translated && m.translated !== m.text ? ` (翻译: ${m.translated})` : ''}`)
            .join('\n');

        const systemPrompt = mode === 'reply'
            ? `You are a helpful assistant for a Second Life user. Based on the recent conversation context below, suggest natural, friendly reply options the user can send. Output 2-3 short reply suggestions in the language the other person is using. Format each as a numbered list. Be concise and conversational.
\nRecent conversation:\n${contextSummary}`
            : `You are a helpful AI assistant for a Second Life user. You have access to the recent conversation history below. Help the user with whatever they ask — translate, polish, suggest replies, explain context, or anything else.
\nRecent conversation:\n${contextSummary}`;

        const messages = [
            { role: 'system', content: systemPrompt },
            { role: 'user', content: userPrompt }
        ];

        await LLMService._doStream(messages, apiKey, baseUrl, model, onChunk);
    }

    /** 内部通用流式请求 */
    static async _doStream(messages, apiKey, baseUrl, model, onChunk) {
        try {
            const response = await fetch(baseUrl, {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                    "Authorization": `Bearer ${apiKey}`,
                },
                body: JSON.stringify({ model, messages, stream: true })
            });

            if (!response.ok) {
                const errorStr = await response.text();
                throw new Error(`API 请求失败: ${response.status} ${errorStr}`);
            }

            const reader = response.body.getReader();
            const decoder = new TextDecoder("utf-8");
            let done = false;

            while (!done) {
                const { value, done: readerDone } = await reader.read();
                done = readerDone;
                if (value) {
                    const chunk = decoder.decode(value, { stream: true });
                    const lines = chunk.split("\n");
                    for (const line of lines) {
                        if (line.startsWith("data: ") && line !== "data: [DONE]") {
                            try {
                                const data = JSON.parse(line.substring(6));
                                if (data.choices?.[0]?.delta?.content) {
                                    onChunk(data.choices[0].delta.content);
                                }
                            } catch (e) {
                                // JSON 解析失败可能发生，忽略
                            }
                        }
                    }
                }
            }
        } catch (error) {
            console.error("LLMService Error:", error);
            onChunk(`\n[网络错误] ${error.message}`);
        }
    }
}
