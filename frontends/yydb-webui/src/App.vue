<template>
    <div class="app-shell">
        <header class="topbar">
            <a class="brand" href="http://127.0.0.1:4317/"><span>Y</span><b>YYDB</b><i>Playground</i></a>
            <nav class="top-actions">
                <div class="connection-state" :class="{ online: client }"><span />{{ client ? text("已连接", "Connected") : text("离线", "Offline") }}</div>
                <a class="docs-link" href="http://127.0.0.1:4317/d/overview">{{ text("文档", "Docs") }} ↗</a>
                <label class="locale"><span class="visually-hidden">Language</span><select v-model="locale"><option value="zh-CN">简体中文</option><option value="en-US">English</option></select></label>
            </nav>
        </header>

        <main class="workspace">
            <aside class="rail">
                <div class="rail-section"><small>{{ text("工作区", "WORKSPACE") }}</small><button class="active"><span>⌘</span> Playground</button><button><span>▦</span> {{ text("数据", "Data") }}</button><button><span>⌁</span> {{ text("对象", "Objects") }}</button></div>
                <div class="rail-section bottom"><small>{{ text("引擎", "ENGINE") }}</small><div class="engine"><i /><span>{{ text("本地引擎", "Local engine") }}<b>{{ client ? text("就绪", "Ready") : text("未连接", "Not connected") }}</b></span></div></div>
            </aside>

            <section class="main-pane">
                <div class="page-head"><div><p>{{ text("本地工作区", "LOCAL WORKSPACE") }}</p><h1>{{ text("数据库 Playground", "Database Playground") }}</h1><span>{{ text("检查本地 YYDB 引擎、编辑 VOS schema，并操作持久化数据。", "Inspect a local YYDB engine, edit its VOS schema, and work with durable values.") }}</span></div></div>

                <div class="connect-panel">
                    <label><span>{{ text("连接地址", "Endpoint") }}</span><div class="endpoint-input"><i>WS</i><input v-model="endpoint" placeholder="ws://127.0.0.1:7700/wire" /></div></label>
                    <button v-if="!client" class="primary" @click="connect"><i /> {{ text("连接", "Connect") }}</button>
                    <button v-else class="disconnect" @click="disconnect"><i /> {{ text("断开", "Disconnect") }}</button>
                </div>
                <div v-if="status || error" class="notice" :class="{ error }"><span>{{ error ? "!" : "✓" }}</span>{{ error || status }}</div>

                <div class="empty-state" v-if="!client">
                    <div class="empty-graphic"><span class="db-layer l1" /><span class="db-layer l2" /><span class="db-layer l3">Y</span><i /></div>
                    <h2>{{ text("连接本地数据库", "Connect to a local database") }}</h2><p>{{ text("Playground 直接连接此设备上的 YYDB 本地端点。", "The playground communicates directly with the YYDB wire endpoint on this device.") }}</p>
                    <div class="steps"><span><b>1</b> {{ text("启动本地引擎", "Start the local engine") }}</span><i>→</i><span><b>2</b> {{ text("输入连接地址", "Enter its endpoint") }}</span><i>→</i><span><b>3</b> {{ text("连接", "Connect") }}</span></div>
                </div>

                <div v-else class="dashboard">
                    <section class="info-card">
                        <div class="card-head"><div><p>ENGINE</p><h2>Connection info</h2></div><button class="icon-button" title="Refresh" @click="refreshInfo">↻</button></div>
                        <pre>{{ infoText || "No engine metadata returned." }}</pre>
                    </section>

                    <section class="schema-card">
                        <div class="card-head"><div><p>VOS SCHEMA</p><h2>Database contract</h2></div><label class="version">Version <input v-model.number="schemaVersion" type="number" min="1" /></label></div>
                        <div class="editor"><div class="editor-bar"><span>schema.vos</span><span class="syntax-ok"><i /> VOS</span></div><textarea v-model="schemaDocument" spellcheck="false" /></div>
                        <div class="card-actions"><button class="secondary" @click="loadSchema">Reload</button><button class="primary" @click="saveSchema">Ensure schema</button></div>
                    </section>

                    <section class="kv-card">
                        <div class="card-head"><div><p>DURABLE VALUES</p><h2>Key / Value</h2></div></div>
                        <div class="kv-fields"><label><span>Key</span><input v-model="kvKey" /></label><label><span>Value</span><input v-model="kvValue" /></label></div>
                        <div class="card-actions"><button class="secondary" @click="kvGet">Get value</button><button class="primary" @click="kvPut">Put value</button></div>
                        <div class="result"><span>RESULT</span><pre>{{ kvResult || "Run an operation to see its result." }}</pre></div>
                    </section>
                </div>
            </section>
        </main>
    </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { Client } from "@yydb/yydb-client";
import { locale, text } from "./i18n";
const endpoint=ref("ws://127.0.0.1:7700/wire"),client=ref<Client|null>(null),status=ref(""),error=ref(""),infoText=ref(""),schemaVersion=ref(1),schemaDocument=ref("table Project {\n  @@id: uuid,\n  @title: utf8,\n}"),kvKey=ref("demo"),kvValue=ref("hello"),kvResult=ref("");
async function connect(){error.value="";try{const next=await Client.connect(endpoint.value);client.value=next;status.value=`Connected · engine ${await next.serverVersion()}`;await refreshInfo();await loadSchema()}catch(err){error.value=String(err);client.value=null}}
function disconnect(){client.value?.close();client.value=null;status.value="Disconnected"}
async function refreshInfo(){if(client.value)infoText.value=await client.value.info()}
async function loadSchema(){if(!client.value)return;const schema=await client.value.getSchema();if(!schema){schemaDocument.value="";return}schemaVersion.value=schema.version;schemaDocument.value=schema.document}
async function saveSchema(){if(!client.value)return;error.value="";try{await client.value.ensureSchema(schemaVersion.value,schemaDocument.value);status.value="Schema ensured";await refreshInfo()}catch(err){error.value=String(err)}}
async function kvGet(){if(!client.value)return;const value=await client.value.get(kvKey.value);kvResult.value=value?new TextDecoder().decode(value):"(missing)"}
async function kvPut(){if(!client.value)return;await client.value.put(kvKey.value,kvValue.value);kvResult.value="Value stored";await refreshInfo()}
</script>
