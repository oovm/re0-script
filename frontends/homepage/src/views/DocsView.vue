<template>
    <div class="docs">
        <div class="docs-top">
            <div class="container docs-top-inner">
                <div><span class="docs-product">YYDB Docs</span><i>/</i><span>{{ localizedTitle }}</span></div>
                <a href="https://github.com/yy-database/yydb.rs" target="_blank" rel="noreferrer">{{ pick("在 GitHub 编辑", "Edit on GitHub") }} ↗</a>
            </div>
        </div>
        <div class="container frame">
            <aside class="sidebar">
                <p class="label">{{ pick("文档", "DOCUMENTATION") }}</p>
                <DocsSidebar />
                <div class="sidebar-meta"><span>YYDB</span><small>MPL-2.0</small></div>
            </aside>

            <div class="content">
                <template v-if="page">
                    <MarkdownRenderer :source="page.body" />
                    <WireDemo v-if="showWireDemo" />
                </template>
                <p v-else class="missing">未找到该文档页。</p>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { findDoc, normalizeDocPath } from "../content/docs";
import DocsSidebar from "../components/DocsSidebar.vue";
import MarkdownRenderer from "../components/MarkdownRenderer.vue";
import WireDemo from "../components/WireDemo.vue";
import { pick } from "../i18n";

const props = defineProps<{
    pathMatch?: string | string[];
}>();

const docPath = computed(() => normalizeDocPath(props.pathMatch));
const page = computed(() => findDoc(docPath.value));
const localizedTitle = computed(() => {
    const titles: Record<string, string> = {
        overview: "Overview",
        "guide/embed-and-serve": "Embed and connect",
        "guide/serve-security": "Local security",
        "guide/wire-protocol": "Binary protocol",
    };
    return pick(page.value?.title || "文档", titles[docPath.value] || "Documentation");
});
const showWireDemo = computed(() => docPath.value === "guide/wire-protocol");
</script>

<style scoped>
.docs {
    min-height: 70vh;
    padding: 0 0 5rem;
    background: #f8fafc;
}
.docs-top{border-bottom:1px solid #e1e6ec;background:#fff}.docs-top-inner{min-height:54px;display:flex;align-items:center;justify-content:space-between;gap:1rem;color:#7b8795;font-size:.76rem}.docs-top-inner>div{display:flex;align-items:center;gap:.65rem}.docs-product{color:#242c38;font-weight:700}.docs-top-inner i{color:#c2c9d1;font-style:normal}.docs-top-inner>a{color:#667383;text-decoration:none;font-weight:650}.docs-top-inner>a:hover{color:#4c38bd}

.frame {
    display: grid;
    grid-template-columns: 225px minmax(0, 1fr);
    gap: clamp(2.5rem, 6vw, 5.5rem);
    align-items: start;
    padding-top: 2.5rem;
}

.sidebar {
    position: sticky;
    top: 5.5rem;
    padding: 0;
}

.label {
    margin: 0 0 0.85rem;
    font-family: var(--font-mono);
    font-size: .62rem;
    font-weight: 700;
    letter-spacing: .12em;
    color: #8994a2;
}

.content {
    min-width: 0;
    max-width: 850px;
    background: #fff;
    border: 1px solid #e1e6ec;
    border-radius: 8px;
    padding: clamp(1.6rem,4vw,3.4rem) clamp(1.4rem,5vw,4rem) 4rem;
    box-shadow: 0 12px 36px rgba(15,23,42,.04);
}
.sidebar-meta{display:flex;justify-content:space-between;margin-top:2rem;padding:1rem .65rem 0;border-top:1px solid #e1e6ec;color:#9ba5b1;font:600 .62rem var(--font-mono)}

.missing {
    margin: 0;
    color: var(--muted);
}

@media (max-width: 860px) {
    .frame {
        grid-template-columns: 1fr;
        gap: 1.25rem;
    }

    .sidebar {
        position: static;
        top: auto;
        border-bottom: 1px solid var(--line);
        padding-bottom: 0.7rem;
        overflow-x: auto;
        scrollbar-width: none;
    }
    .sidebar::-webkit-scrollbar{display:none}
    .label,.docs-top-inner>a,.sidebar-meta{display:none}
    .sidebar :deep(.sidebar-nav){display:flex;width:max-content;gap:.25rem}
    .sidebar :deep(.group){display:contents;margin:0}
    .sidebar :deep(.group-title){display:none}
    .sidebar :deep(.group-items){display:flex;gap:.25rem}
    .sidebar :deep(.leaf){padding:.55rem .7rem;border:0;border-radius:5px;white-space:nowrap}
    .sidebar :deep(.leaf.active){box-shadow:inset 0 -2px #6651e7}
    .content{padding:1.4rem 1.2rem 3rem}.frame{padding-top:1rem}
}
</style>
