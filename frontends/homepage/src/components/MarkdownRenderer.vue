<template>
    <div v-if="pending" class="loading">正在渲染文档…</div>
    <div v-else-if="error" class="error">{{ error }}</div>
    <article v-else class="markdown" v-html="html" />
</template>

<script setup lang="ts">
import { ref, watch } from "vue";
import { renderMarkdown } from "../lib/markdown";

const props = defineProps<{
    source: string;
}>();

const html = ref("");
const pending = ref(true);
const error = ref("");

async function render(source: string) {
    pending.value = true;
    error.value = "";
    try {
        html.value = await renderMarkdown(source);
    } catch (err) {
        error.value = err instanceof Error ? err.message : String(err);
        html.value = "";
    } finally {
        pending.value = false;
    }
}

watch(
    () => props.source,
    (value) => {
        void render(value);
    },
    { immediate: true },
);
</script>

<style scoped>
.loading,
.error {
    color: var(--muted);
    padding: 1rem 0;
}

.error {
    color: #9b2c2c;
}
</style>
