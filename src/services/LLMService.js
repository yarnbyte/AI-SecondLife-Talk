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
        const baseUrl = config.baseUrl || API_DEFAULTS.BASE_URL;
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

        try {
            const response = await fetch(baseUrl, {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                    "Authorization": `Bearer ${apiKey}`,
                },
                body: JSON.stringify({
                    model: model,
                    messages: messages,
                    stream: true,
                })
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
                                if (data.choices && data.choices[0].delta && data.choices[0].delta.content) {
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
