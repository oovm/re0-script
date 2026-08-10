import indexZh from "../../documentation/zh-hans/index.md?raw";
import guideEmbed from "../../documentation/zh-hans/guide/embed-and-serve.md?raw";
import guideSecurity from "../../documentation/zh-hans/guide/serve-security.md?raw";
import guideWire from "../../documentation/zh-hans/guide/wire-protocol.md?raw";

export interface DocPage {
    path: string;
    title: string;
    body: string;
}

export interface DocNavNode {
    title: string;
    path?: string;
    children?: DocNavNode[];
}

export const docs: DocPage[] = [
    { path: "overview", title: "概览", body: indexZh },
    { path: "guide/embed-and-serve", title: "嵌入与 Serve", body: guideEmbed },
    { path: "guide/serve-security", title: "Serve 安全模型", body: guideSecurity },
    { path: "guide/wire-protocol", title: "二进制协议", body: guideWire },
];

export const docNav: DocNavNode[] = [
    { title: "概览", path: "overview" },
    {
        title: "指南",
        children: [
            { title: "嵌入与 Serve", path: "guide/embed-and-serve" },
            { title: "Serve 安全模型", path: "guide/serve-security" },
            { title: "二进制协议", path: "guide/wire-protocol" },
        ],
    },
];

export const defaultDocPath = "overview";

export function normalizeDocPath(raw: string | string[] | undefined): string {
    if (raw == null) {
        return defaultDocPath;
    }
    const joined = Array.isArray(raw) ? raw.join("/") : raw;
    return joined.replace(/^\/+|\/+$/g, "") || defaultDocPath;
}

export function findDoc(path: string): DocPage | undefined {
    const normalized = normalizeDocPath(path);
    return docs.find((doc) => doc.path === normalized);
}

export function docHref(path: string): string {
    return `/d/${normalizeDocPath(path)}`;
}
