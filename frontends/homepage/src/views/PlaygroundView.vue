<template>
    <div class="playground">
        <div class="workspace">
            <aside class="rail" aria-label="Playground 导航">
                <div class="rail-section">
                    <small>WORKSPACE</small>
                    <button type="button" class="active"><span>⌘</span> Playground</button>
                </div>
                <div class="rail-section bottom">
                    <small>ENGINE</small>
                    <div class="engine" :class="{ on: !!client }">
                        <i />
                        <span>
                            Local engine
                            <b>{{ client ? "Ready" : "Not connected" }}</b>
                        </span>
                    </div>
                </div>
            </aside>

            <section class="main-pane">
                <div class="page-head">
                    <div>
                        <p>LOCAL WORKSPACE</p>
                        <h1>Database Playground</h1>
                        <span>
                            连接本机 YYDB 引擎，查看信息、确保 VOS schema、读写 durable values。
                        </span>
                    </div>
                    <div class="connection-state" :class="{ online: !!client }">
                        <span />
                        {{ client ? "Connected" : "Offline" }}
                    </div>
                </div>

                <div class="connect-panel">
                    <label>
                        <span>Endpoint</span>
                        <div class="endpoint-input">
                            <i>WS</i>
                            <input v-model="endpoint" placeholder="ws://127.0.0.1:7700/wire" />
                        </div>
                    </label>
                    <button v-if="!client" type="button" class="primary" @click="connect">
                        <i /> Connect
                    </button>
                    <button v-else type="button" class="disconnect" @click="disconnect">
                        <i /> Disconnect
                    </button>
                </div>

                <div v-if="status || error" class="notice" :class="{ error: !!error }">
                    <span>{{ error ? "!" : "✓" }}</span>
                    {{ error || status }}
                </div>

                <div v-if="!client" class="empty-state">
                    <div class="empty-graphic" aria-hidden="true">
                        <span class="db-layer l1" />
                        <span class="db-layer l2" />
                        <span class="db-layer l3">Y</span>
                    </div>
                    <h2>Connect to a local database</h2>
                    <p>Playground 只连本机 wire 端点，不托管远程库。</p>
                    <div class="steps">
                        <span><b>1</b> 启动本地引擎</span>
                        <i>→</i>
                        <span><b>2</b> 填入 endpoint</span>
                        <i>→</i>
                        <span><b>3</b> Connect</span>
                    </div>
                    <pre class="hint">yydb serve ./app.yydb --bind 127.0.0.1:7700</pre>
                </div>

                <div v-else class="dashboard">
                    <section class="panel info-card">
                        <div class="card-head">
                            <div>
                                <p>ENGINE</p>
                                <h2>Connection info</h2>
                            </div>
                            <button type="button" class="icon-button" title="Refresh" @click="refreshInfo">
                                ↻
                            </button>
                        </div>
                        <pre>{{ infoText || "No engine metadata returned." }}</pre>
                    </section>

                    <section class="panel schema-card">
                        <div class="card-head">
                            <div>
                                <p>VOS SCHEMA</p>
                                <h2>Database contract</h2>
                            </div>
                            <label class="version">
                                Version
                                <input v-model.number="schemaVersion" type="number" min="1" />
                            </label>
                        </div>
                        <div class="editor">
                            <div class="editor-bar">
                                <span>schema.vos</span>
                                <span class="syntax-ok"><i /> VOS</span>
                            </div>
                            <textarea v-model="schemaDocument" spellcheck="false" />
                        </div>
                        <div class="card-actions">
                            <button type="button" class="secondary" @click="loadSchema">Reload</button>
                            <button type="button" class="primary" @click="saveSchema">
                                Ensure schema
                            </button>
                        </div>
                    </section>

                    <section class="panel kv-card">
                        <div class="card-head">
                            <div>
                                <p>DURABLE VALUES</p>
                                <h2>Key / Value</h2>
                            </div>
                        </div>
                        <div class="kv-fields">
                            <label>
                                <span>Key</span>
                                <input v-model="kvKey" />
                            </label>
                            <label>
                                <span>Value</span>
                                <input v-model="kvValue" />
                            </label>
                        </div>
                        <div class="card-actions">
                            <button type="button" class="secondary" @click="kvGet">Get value</button>
                            <button type="button" class="primary" @click="kvPut">Put value</button>
                        </div>
                        <div class="result">
                            <span>RESULT</span>
                            <pre>{{ kvResult || "Run an operation to see its result." }}</pre>
                        </div>
                    </section>
                </div>
            </section>
        </div>
    </div>
</template>

<script setup lang="ts">
import { onBeforeUnmount, ref } from "vue";
import { Client } from "@yydb/yydb-client";

const endpoint = ref("ws://127.0.0.1:7700/wire");
const client = ref<Client | null>(null);
const status = ref("");
const error = ref("");
const infoText = ref("");
const schemaVersion = ref(1);
const schemaDocument = ref("table Project {\n  @@id: uuid,\n  @title: utf8,\n}");
const kvKey = ref("demo");
const kvValue = ref("hello");
const kvResult = ref("");

async function connect() {
    error.value = "";
    try {
        const next = await Client.connect(endpoint.value);
        client.value = next;
        status.value = `Connected · engine ${await next.serverVersion()}`;
        await refreshInfo();
        await loadSchema();
    } catch (err) {
        error.value = String(err);
        client.value = null;
    }
}

function disconnect() {
    client.value?.close();
    client.value = null;
    status.value = "Disconnected";
}

async function refreshInfo() {
    if (!client.value) {
        return;
    }
    infoText.value = await client.value.info();
}

async function loadSchema() {
    if (!client.value) {
        return;
    }
    const schema = await client.value.getSchema();
    if (!schema) {
        schemaDocument.value = "";
        return;
    }
    schemaVersion.value = schema.version;
    schemaDocument.value = schema.document;
}

async function saveSchema() {
    if (!client.value) {
        return;
    }
    error.value = "";
    try {
        await client.value.ensureSchema(schemaVersion.value, schemaDocument.value);
        status.value = "Schema ensured";
        await refreshInfo();
    } catch (err) {
        error.value = String(err);
    }
}

async function kvGet() {
    if (!client.value) {
        return;
    }
    const value = await client.value.get(kvKey.value);
    kvResult.value = value ? new TextDecoder().decode(value) : "(missing)";
}

async function kvPut() {
    if (!client.value) {
        return;
    }
    await client.value.put(kvKey.value, kvValue.value);
    kvResult.value = "Value stored";
    await refreshInfo();
}

onBeforeUnmount(() => {
    client.value?.close();
});
</script>

<style scoped>
.playground {
    --pg-bg: #080b10;
    --pg-panel: #0d121a;
    --pg-line: #25303d;
    --pg-line-soft: #293442;
    --pg-text: #e9eef5;
    --pg-mute: #7f8b99;
    --pg-mint: #3ee6a8;
    --pg-ink: #07110d;

    background: var(--pg-bg);
    color: var(--pg-text);
    min-height: calc(100vh - 4.5rem);
}

.workspace {
    display: grid;
    grid-template-columns: 220px minmax(0, 1fr);
    min-height: calc(100vh - 4.5rem);
}

.rail {
    display: flex;
    flex-direction: column;
    padding: 24px 14px;
    border-right: 1px solid #202936;
    background: #0a0e14;
}

.rail-section {
    display: grid;
    gap: 4px;
}

.rail-section small {
    margin: 0 10px 9px;
    color: #4e5a68;
    font: 700 10px var(--font-mono);
    letter-spacing: 0.1em;
}

.rail button {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px;
    border: 0;
    border-radius: 5px;
    background: transparent;
    color: #748191;
    text-align: left;
    font-size: 13px;
    cursor: pointer;
}

.rail button span {
    width: 18px;
    color: #566272;
}

.rail button.active {
    background: #151c27;
    color: #e2e8ef;
}

.rail button.active span {
    color: var(--pg-mint);
}

.rail-section.bottom {
    margin-top: auto;
}

.engine {
    display: flex;
    gap: 9px;
    align-items: center;
    padding: 10px;
}

.engine > i {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: #596574;
}

.engine.on > i {
    background: var(--pg-mint);
}

.engine span {
    display: flex;
    flex-direction: column;
    color: #a1acb9;
    font-size: 12px;
}

.engine b {
    margin-top: 2px;
    color: #566272;
    font-size: 10px;
    font-weight: 500;
}

.main-pane {
    width: min(1180px, 100%);
    padding: 40px clamp(24px, 5vw, 64px) 64px;
}

.page-head {
    display: flex;
    justify-content: space-between;
    gap: 1rem;
    align-items: start;
}

.page-head p,
.card-head p {
    margin: 0 0 8px;
    color: var(--pg-mint);
    font: 700 10px var(--font-mono);
    letter-spacing: 0.12em;
}

.page-head h1 {
    margin: 0;
    color: #f5f7fa;
    font-family: var(--font-display);
    font-variation-settings: "wdth" 95;
    font-size: clamp(1.8rem, 3vw, 2rem);
    letter-spacing: -0.03em;
}

.page-head > div > span {
    display: block;
    margin-top: 8px;
    color: var(--pg-mute);
    font-size: 14px;
    max-width: 40rem;
}

.connection-state {
    display: flex;
    align-items: center;
    gap: 7px;
    padding: 6px 10px;
    border: 1px solid #2a3441;
    border-radius: 99px;
    color: #7f8b99;
    font: 600 11px var(--font-mono);
    white-space: nowrap;
}

.connection-state span {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: #667281;
}

.connection-state.online {
    color: #a5b7b0;
}

.connection-state.online span {
    background: var(--pg-mint);
}

.connect-panel {
    display: grid;
    grid-template-columns: 1fr auto;
    gap: 12px;
    align-items: end;
    margin-top: 28px;
    padding: 18px;
    border: 1px solid var(--pg-line);
    border-radius: 8px;
    background: var(--pg-panel);
}

.connect-panel label,
.kv-fields label {
    display: grid;
    gap: 7px;
    color: #8996a5;
    font-size: 12px;
}

.endpoint-input {
    display: grid;
    grid-template-columns: auto 1fr;
    align-items: center;
    border: 1px solid var(--pg-line-soft);
    border-radius: 6px;
    background: #080c12;
    overflow: hidden;
}

.endpoint-input i {
    padding: 0 11px;
    color: var(--pg-mint);
    font: 700 10px var(--font-mono);
    font-style: normal;
}

.endpoint-input input {
    border: 0;
    border-left: 1px solid var(--pg-line-soft);
    border-radius: 0;
}

.connect-panel input,
.kv-fields input,
.version input {
    min-width: 0;
    padding: 11px 12px;
    border: 1px solid var(--pg-line-soft);
    border-radius: 6px;
    outline: none;
    background: #080c12;
    color: #dfe6ee;
    font: 500 13px var(--font-mono);
}

.connect-panel input:focus,
.kv-fields input:focus,
.version input:focus,
textarea:focus {
    border-color: var(--pg-mint);
}

.primary,
.disconnect,
.secondary,
.icon-button {
    min-height: 42px;
    padding: 0 16px;
    border: 1px solid transparent;
    border-radius: 6px;
    font-weight: 700;
    font-size: 12px;
    cursor: pointer;
}

.primary {
    background: var(--pg-mint);
    color: var(--pg-ink);
}

.primary:hover {
    background: #70edbd;
}

.primary i,
.disconnect i {
    display: inline-block;
    width: 7px;
    height: 7px;
    margin-right: 6px;
    border-radius: 50%;
    background: currentColor;
}

.disconnect {
    border-color: #523138;
    background: #21151a;
    color: #f18796;
}

.secondary,
.icon-button {
    border-color: #2a3543;
    background: #151b24;
    color: #a5b0bd;
}

.secondary:hover,
.icon-button:hover {
    border-color: #465466;
    color: #fff;
}

.notice {
    display: flex;
    gap: 9px;
    margin-top: 12px;
    padding: 11px 13px;
    border: 1px solid rgba(62, 230, 168, 0.2);
    border-radius: 6px;
    background: rgba(62, 230, 168, 0.07);
    color: #91ad9f;
    font: 500 12px var(--font-mono);
}

.notice > span {
    color: var(--pg-mint);
}

.notice.error {
    border-color: rgba(240, 100, 116, 0.25);
    background: rgba(240, 100, 116, 0.08);
    color: #df909a;
}

.notice.error > span {
    color: #f06474;
}

.empty-state {
    display: grid;
    justify-items: center;
    margin-top: 48px;
    padding: 56px 24px;
    border: 1px dashed var(--pg-line-soft);
    border-radius: 8px;
    background: #0b0f16;
    text-align: center;
}

.empty-graphic {
    position: relative;
    width: 120px;
    height: 108px;
}

.db-layer {
    position: absolute;
    left: 20px;
    width: 80px;
    height: 58px;
    display: grid;
    place-items: center;
    border: 1px solid #344151;
    border-radius: 7px;
    background: #111823;
}

.l1 {
    top: 34px;
    opacity: 0.35;
}

.l2 {
    top: 19px;
    opacity: 0.65;
}

.l3 {
    top: 4px;
    color: var(--pg-ink);
    background: var(--pg-mint);
    border-color: var(--pg-mint);
    font-weight: 800;
}

.empty-state h2 {
    margin: 18px 0 6px;
    font-size: 19px;
}

.empty-state p {
    max-width: 450px;
    margin: 0;
    color: #748191;
    font-size: 13px;
}

.steps {
    display: flex;
    align-items: center;
    gap: 14px;
    margin-top: 28px;
    color: #657180;
    font-size: 11px;
}

.steps span {
    display: flex;
    align-items: center;
    gap: 7px;
}

.steps b {
    display: grid;
    place-items: center;
    width: 20px;
    height: 20px;
    border-radius: 50%;
    background: #18202b;
    color: #9da8b5;
    font-size: 10px;
}

.steps > i {
    color: #3d4856;
    font-style: normal;
}

.hint {
    margin: 22px 0 0;
    padding: 12px 14px;
    border-radius: 6px;
    background: #080c12;
    border: 1px solid var(--pg-line-soft);
    color: #a8b4c2;
    font: 500 12px var(--font-mono);
}

.dashboard {
    display: grid;
    grid-template-columns: 0.75fr 1.25fr;
    gap: 16px;
    margin-top: 24px;
}

.panel {
    border: 1px solid var(--pg-line);
    border-radius: 8px;
    background: var(--pg-panel);
    padding: 18px;
    overflow: hidden;
}

.schema-card {
    grid-row: span 2;
}

.card-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    margin-bottom: 16px;
}

.card-head h2 {
    margin: 0;
    font-size: 15px;
}

.version {
    display: flex;
    align-items: center;
    gap: 8px;
    color: #778493;
    font-size: 11px;
}

.version input {
    width: 58px;
    padding: 7px;
}

.info-card pre,
.result pre {
    margin: 0;
    color: #7f8d9d;
    font: 500 11px/1.6 var(--font-mono);
    white-space: pre-wrap;
    word-break: break-word;
}

.editor {
    border: 1px solid var(--pg-line-soft);
    border-radius: 6px;
    background: #080c12;
    overflow: hidden;
}

.editor-bar {
    display: flex;
    justify-content: space-between;
    padding: 9px 11px;
    border-bottom: 1px solid var(--pg-line-soft);
    color: #6d7a89;
    font: 600 10px var(--font-mono);
}

.syntax-ok {
    color: #7c8b99;
}

.syntax-ok i {
    display: inline-block;
    width: 6px;
    height: 6px;
    margin-right: 5px;
    border-radius: 50%;
    background: var(--pg-mint);
}

.editor textarea {
    display: block;
    width: 100%;
    min-height: 250px;
    padding: 15px;
    border: 0;
    outline: none;
    resize: vertical;
    background: transparent;
    color: #dce4ec;
    font: 500 13px/1.7 var(--font-mono);
}

.card-actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 14px;
}

.kv-card {
    grid-column: 1;
}

.kv-fields {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 10px;
}

.result {
    margin-top: 14px;
    padding: 12px;
    border-radius: 6px;
    background: #080c12;
}

.result > span {
    display: block;
    margin-bottom: 7px;
    color: #4f5c6c;
    font: 700 9px var(--font-mono);
    letter-spacing: 0.1em;
}

@media (max-width: 850px) {
    .workspace {
        grid-template-columns: 1fr;
    }

    .rail {
        display: none;
    }

    .main-pane {
        padding: 32px 20px 48px;
    }

    .dashboard {
        grid-template-columns: 1fr;
    }

    .schema-card {
        grid-row: auto;
    }

    .kv-card {
        grid-column: auto;
    }
}

@media (max-width: 560px) {
    .page-head {
        flex-direction: column;
    }

    .connect-panel {
        grid-template-columns: 1fr;
    }

    .connect-panel button {
        width: 100%;
    }

    .steps {
        align-items: flex-start;
        flex-direction: column;
    }

    .steps > i {
        display: none;
    }

    .kv-fields {
        grid-template-columns: 1fr;
    }
}
</style>
