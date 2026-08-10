<template>
    <footer class="footer">
        <div class="container">
            <div class="top">
                <div class="brand-block">
                    <p class="kicker">YY Database</p>
                    <p class="wordmark" aria-hidden="true">YYDB</p>
                    <p class="tag">
                        {{ pick("VOS 原生的单文件本地数据库。", "A VOS-native, single-file local database.") }}<br />
                        {{ pick("Schema 写在文件里，跟着数据走。", "Schema travels with the data.") }}
                    </p>
                </div>

                <div class="columns" aria-label="页脚导航">
                    <div class="col">
                        <h2>{{ pick("阅读", "Learn") }}</h2>
                        <RouterLink to="/d/overview">文档概览</RouterLink>
                        <RouterLink to="/d/guide/embed-and-serve">嵌入与 Serve</RouterLink>
                        <RouterLink to="/d/guide/serve-security">安全模型</RouterLink>
                        <RouterLink to="/d/guide/wire-protocol">二进制协议</RouterLink>
                        <RouterLink to="/playground">Playground</RouterLink>
                    </div>
                    <div class="col">
                        <h2>{{ pick("使用", "Use") }}</h2>
                        <a href="https://crates.io/crates/yydb" target="_blank" rel="noreferrer">
                            Rust · crates.io
                        </a>
                        <a href="https://www.npmjs.com/package/@yydb/yydb" target="_blank" rel="noreferrer">
                            Node · npm
                        </a>
                        <a
                            href="https://github.com/yy-database/yydb.rs/releases"
                            target="_blank"
                            rel="noreferrer"
                        >
                            CLI · releases
                        </a>
                        <a
                            href="https://www.npmjs.com/package/@yydb/yydb-client"
                            target="_blank"
                            rel="noreferrer"
                        >
                            Browser · client
                        </a>
                    </div>
                    <div class="col">
                        <h2>{{ pick("仓库", "Project") }}</h2>
                        <a href="https://github.com/yy-database/yydb.rs" target="_blank" rel="noreferrer">
                            GitHub
                        </a>
                        <a href="https://docs.rs/yydb" target="_blank" rel="noreferrer">docs.rs</a>
                        <a href="https://github.com/voml/vos-language" target="_blank" rel="noreferrer">
                            VOS language
                        </a>
                    </div>
                </div>
            </div>

            <div class="bottom">
                <div class="rule" aria-hidden="true">
                    <span class="tick coral"/>
                    <span class="tick mint"/>
                    <span class="tick violet"/>
                </div>
                <div class="legal">
                    <span class="license">MPL-2.0</span>
                    <span class="copy">© {{ year }} YY Database</span>
                </div>
            </div>
        </div>
    </footer>
</template>

<script setup lang="ts">
import { pick } from "../i18n";
const year = new Date().getFullYear();
</script>

<style scoped>
.footer {
    --ft-ink: #f4f2ed;
    --ft-mute: #9a978c;
    --ft-dim: #6f6d64;
    --ft-line: #3a3934;

    margin-top: auto;
    position: relative;
    overflow: hidden;
    background:
        radial-gradient(80% 90% at 0% 100%, rgba(230, 109, 87, 0.12), transparent 55%),
        radial-gradient(70% 80% at 100% 0%, rgba(112, 202, 167, 0.1), transparent 50%),
        var(--ink);
    color: var(--ft-ink);
    padding: clamp(3.5rem, 8vw, 5rem) 0 1.5rem;
}

.top {
    display: grid;
    grid-template-columns: minmax(0, 1.15fr) minmax(0, 1.35fr);
    gap: clamp(2.5rem, 6vw, 4.5rem);
    align-items: start;
}

.kicker {
    margin: 0 0 0.85rem;
    font-family: var(--font-mono);
    font-size: 0.72rem;
    letter-spacing: 0.14em;
    text-transform: uppercase;
    color: var(--mint);
}

.wordmark {
    margin: 0;
    font-family: var(--font-display);
    font-variation-settings: "wdth" 82;
    font-weight: 800;
    font-size: clamp(3.6rem, 9vw, 6.2rem);
    letter-spacing: -0.07em;
    line-height: 0.85;
    color: var(--ft-ink);
}

.tag {
    margin: 1.15rem 0 0;
    max-width: 28ch;
    font-size: 1.02rem;
    line-height: 1.55;
    color: var(--ft-mute);
}

.columns {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 1.5rem 1.75rem;
    padding-top: 0.35rem;
}

.col {
    display: grid;
    gap: 0.55rem;
    align-content: start;
}

.col h2 {
    margin: 0 0 0.35rem;
    font-family: var(--font-mono);
    font-size: 0.7rem;
    font-weight: 500;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: var(--ft-dim);
}

.col a {
    color: #d8d5cb;
    text-decoration: none;
    font-weight: 600;
    font-size: 0.95rem;
    width: fit-content;
    border-bottom: 1px solid transparent;
    transition: color 0.15s ease, border-color 0.15s ease;
}

.col a:hover {
    color: var(--mint);
    border-bottom-color: color-mix(in srgb, var(--mint) 55%, transparent);
}

.bottom {
    margin-top: clamp(2.5rem, 6vw, 3.5rem);
}

.rule {
    display: grid;
    grid-template-columns: 1.2fr 0.7fr 0.5fr;
    gap: 0.35rem;
    margin-bottom: 1.15rem;
}

.tick {
    display: block;
    height: 3px;
}

.tick.coral {
    background: var(--coral);
}

.tick.mint {
    background: var(--mint);
}

.tick.violet {
    background: var(--violet);
}

.legal {
    display: flex;
    justify-content: space-between;
    gap: 1rem;
    flex-wrap: wrap;
    font-family: var(--font-mono);
    font-size: 0.78rem;
    letter-spacing: 0.04em;
    color: var(--ft-dim);
}

.license {
    color: var(--ft-mute);
}

@media (max-width: 860px) {
    .top {
        grid-template-columns: 1fr;
        gap: 2.25rem;
    }

    .columns {
        grid-template-columns: 1fr 1fr;
    }
}

@media (max-width: 520px) {
    .columns {
        grid-template-columns: 1fr;
        gap: 1.75rem;
    }

    .rule {
        grid-template-columns: 1fr 0.6fr 0.4fr;
    }
}
</style>
