<template>
    <header class="header">
        <div class="container bar">
            <RouterLink class="brand" to="/" aria-label="YYDB home">
                <span class="brand-mark">Y</span>
                <span class="name">YYDB</span>
            </RouterLink>

            <button
                class="menu-toggle"
                type="button"
                :aria-expanded="open"
                aria-controls="site-nav"
                @click="open = !open"
            >
                <span class="menu-icon" :class="{ active: open }" aria-hidden="true"><i /><i /></span>
                <span class="sr-only">{{ open ? "关闭菜单" : "打开菜单" }}</span>
            </button>

            <nav id="site-nav" class="nav" :class="{ open }" @click="open = false">
                <RouterLink to="/d/overview">{{ pick("文档", "Docs") }}</RouterLink>
                <RouterLink to="/playground">Playground</RouterLink>
                <a href="https://github.com/yy-database/yydb.rs" target="_blank" rel="noreferrer">
                    GitHub
                </a>
                <label class="locale">
                    <span class="sr-only">{{ pick("选择语言", "Choose language") }}</span>
                    <select v-model="locale" :aria-label="pick('选择语言', 'Choose language')">
                        <option value="zh-CN">简体中文</option>
                        <option value="en-US">English</option>
                    </select>
                </label>
            </nav>
        </div>
    </header>
</template>

<script setup lang="ts">
import { ref, watch } from "vue";
import { useRoute } from "vue-router";
import { locale, pick } from "../i18n";

const open = ref(false);
const route = useRoute();

watch(
    () => route.fullPath,
    () => {
        open.value = false;
    },
);
</script>

<style scoped>
.header {
    position: sticky;
    top: 0;
    z-index: 20;
    background: rgba(7, 10, 15, .94);
    border-bottom: 1px solid #202936;
    backdrop-filter: blur(12px);
}

.header .brand,
.header .nav a,
.header .menu-toggle { color:#e8edf4; }
.header .brand-mark { background:#3ee6a8; color:#07110d; }
.header .nav a:hover,
.header .nav a.router-link-active { color:#3ee6a8; }

.header.over {
    position: absolute;
    inset: 0 0 auto;
    background: color-mix(in srgb, #070a0f 82%, transparent);
    border-bottom-color: rgba(255, 255, 255, 0.08);
    backdrop-filter: blur(12px);
}

.header.over .brand,
.header.over .nav a,
.header.over .menu-toggle {
    color: #e8edf4;
}

.header.over .brand-mark {
    background: #3ee6a8;
    color: #07110d;
}

.header.over .nav a:hover,
.header.over .nav a.router-link-active {
    color: #3ee6a8;
}

.bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    min-height: 4.5rem;
}

.brand {
    display: flex;
    align-items: center;
    gap: .65rem;
    text-decoration: none;
    color: var(--ink);
}

.brand-mark { display:grid; place-items:center; width:27px; height:27px; background:var(--ink); color:#fff; font:750 .72rem var(--font-display); border-radius:2px; }

.name {
    font-family: var(--font-display);
    font-variation-settings: "wdth" 90;
    font-weight: 750;
    font-size: 1.2rem;
    letter-spacing: 0;
}

.nav {
    display: flex;
    align-items: center;
    gap: 1.35rem;
}

.nav a {
    color: var(--ink-soft);
    text-decoration: none;
    font-weight: 650;
    font-size: .9rem;
}

.nav a:hover,
.nav a.router-link-active {
    color: var(--accent-deep);
}

.locale select {
    height: 34px;
    padding: 0 1.8rem 0 .65rem;
    border: 1px solid #303a49;
    border-radius: 5px;
    background: #10151d;
    color: #c8d1dc;
    font-size: .76rem;
    cursor: pointer;
}

.menu-toggle {
    display: none;
    border: 1px solid var(--line);
    background: transparent;
    width: 42px;
    height: 42px;
    align-items: center;
    justify-content: center;
    border-radius: 3px;
    padding: 0;
    color: var(--ink);
    cursor: pointer;
}
.menu-icon { position:relative; display:block; width:18px; height:14px; }.menu-icon i{position:absolute;left:0;width:18px;height:1.5px;background:currentColor;transition:.18s}.menu-icon i:first-child{top:4px}.menu-icon i:last-child{top:10px}.menu-icon.active i:first-child{top:7px;transform:rotate(45deg)}.menu-icon.active i:last-child{top:7px;transform:rotate(-45deg)}
.sr-only { position:absolute; width:1px; height:1px; padding:0; margin:-1px; overflow:hidden; clip:rect(0,0,0,0); white-space:nowrap; border:0; }

@media (max-width: 720px) {
    .menu-toggle {
        display: inline-flex;
    }

    .nav {
        display: none;
        position: absolute;
        inset: calc(4.5rem - 1px) 0 auto;
        flex-direction: column;
        align-items: stretch;
        gap: 0;
        padding: 0.5rem 1.25rem 1rem;
        background: var(--surface);
        border-bottom: 1px solid var(--line);
    }

    .header.over .nav {
        background: #0d1118;
        border-bottom-color: #26303d;
    }

    .header.over .nav a {
        border-bottom-color: #26303d;
    }

    .nav.open {
        display: flex;
    }

    .nav a {
        padding: 0.75rem 0.15rem;
        border-bottom: 1px solid var(--line);
    }

    .nav a:last-child {
        border-bottom: 0;
    }

    .locale { padding: .65rem 0; }
    .locale select { width: 100%; }

    .bar {
        position: relative;
    }
}
</style>
