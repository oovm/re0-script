<template>
    <nav class="sidebar-nav" aria-label="文档目录">
        <template v-for="(node, index) in docNav" :key="index">
            <RouterLink
                v-if="node.path"
                class="leaf"
                :to="docHref(node.path)"
                active-class="active"
            >
                {{ titleFor(node.title, node.path) }}
            </RouterLink>

            <div v-else-if="node.children?.length" class="group">
                <p class="group-title">{{ titleFor(node.title) }}</p>
                <div class="group-items">
                    <RouterLink
                        v-for="child in node.children"
                        :key="child.path"
                        class="leaf nested"
                        :to="docHref(child.path!)"
                        active-class="active"
                    >
                        {{ titleFor(child.title, child.path) }}
                    </RouterLink>
                </div>
            </div>
        </template>
    </nav>
</template>

<script setup lang="ts">
import { docHref, docNav } from "../content/docs";
import { pick } from "../i18n";

const english: Record<string, string> = {
    "概览": "Overview",
    "指南": "Guides",
    "嵌入与 Serve": "Embed and connect",
    "Serve 安全模型": "Local security",
    "二进制协议": "Binary protocol",
};
const titleFor = (title: string, _path?: string) => pick(title, english[title] || title);
</script>

<style scoped>
.sidebar-nav {
    display: grid;
    gap: 0.2rem;
}

.group {
    margin-top: 1.25rem;
}

.group:first-child {
    margin-top: 0;
}

.group-title {
    margin: 0 0 0.35rem;
    padding: 0 0.7rem;
    font-size: 0.66rem;
    font-weight: 700;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--muted);
}

.group-items {
    display: grid;
    gap: 0.1rem;
}

.leaf {
    display: block;
    padding: 0.55rem 0.7rem;
    border-left: 2px solid transparent;
    border-radius: 4px;
    color: var(--ink-soft);
    text-decoration: none;
    font-weight: 600;
    font-size: 0.86rem;
    line-height: 1.35;
}

.leaf.nested {
    padding-left: 0.95rem;
}

.leaf:hover,
.leaf.active {
    background: #eeebff;
    border-left-color: #6651e7;
    color: #4937b5;
}
</style>
