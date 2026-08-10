import { Marked } from "marked";
import {
    createHighlighter,
    type Highlighter,
    type LanguageRegistration,
} from "shiki";
import { vosLanguage as sharedVos } from "@game-gpt/vos-textmate";

/** Same TextMate grammar as `vscode-vos` (`@game-gpt/vos-textmate`). */
const vosLanguage = sharedVos as LanguageRegistration;

const marked = new Marked();

let highlighterPromise: Promise<Highlighter> | null = null;

function getHighlighter(): Promise<Highlighter> {
    if (!highlighterPromise) {
        highlighterPromise = createHighlighter({
            themes: ["github-light"],
            langs: [
                vosLanguage,
                "typescript",
                "ts",
                "javascript",
                "bash",
                "shell",
                "json",
                "text",
                "rust",
            ],
        });
    }
    return highlighterPromise;
}

marked.use({
    async: true,
    async walkTokens(token) {
        if (token.type !== "code") {
            return;
        }

        const highlighter = await getHighlighter();
        const requested = token.lang?.trim() || "text";
        const aliases = new Set(["vos", "VOS"]);
        const lang = aliases.has(requested)
            ? "vos"
            : highlighter.getLoadedLanguages().includes(requested as never)
              ? requested
              : "text";

        const html = highlighter.codeToHtml(token.text, {
            lang,
            theme: "github-light",
        });

        Object.assign(token, {
            type: "html",
            block: true,
            text: html,
        });
    },
});

export async function renderMarkdown(source: string): Promise<string> {
    return (await marked.parse(source)) as string;
}
