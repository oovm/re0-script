<template>
    <section class="demo">
        <div class="demo-head"><span>INTERACTIVE</span><h2>本地帧编解码</h2></div>
        <p class="muted">
            使用 <code>@yydb/yydb-client</code> 在浏览器内编码和解码一个
            Hello 帧。所有操作均在当前页面完成，不会连接数据库。
        </p>
        <label>
            request id
            <input v-model.number="requestId" type="number" min="1"/>
        </label>
        <label>
            Hello body (UTF-8，可空)
            <input v-model="bodyText"/>
        </label>
        <div class="row">
            <button type="button" class="btn btn-primary" @click="encode">Encode Hello</button>
            <button type="button" class="btn btn-ghost" @click="roundtrip">
                Encode → Decode
            </button>
        </div>
        <pre class="mono">{{ hex }}</pre>
        <pre v-if="decoded" class="mono">{{ decoded }}</pre>
    </section>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { MsgType, decodeFrame, encodeFrame } from "@yydb/yydb-client";

const requestId = ref(1);
const bodyText = ref("");
const hex = ref("");
const decoded = ref("");

function toHex(bytes: Uint8Array): string {
    return [...bytes].map((b) => b.toString(16).padStart(2, "0")).join(" ");
}

function encode() {
    const body = new TextEncoder().encode(bodyText.value);
    const bytes = encodeFrame({
        msgType: MsgType.Hello,
        flags: 0,
        requestId: requestId.value >>> 0,
        body,
    });
    hex.value = toHex(bytes);
    decoded.value = "";
}

function roundtrip() {
    encode();
    const bytes = Uint8Array.from(
        hex.value.split(/\s+/).filter(Boolean).map((part) => parseInt(part, 16)),
    );
    const frame = decodeFrame(bytes);
    decoded.value = JSON.stringify(
        {
            msgType: frame.msgType,
            flags: frame.flags,
            requestId: frame.requestId,
            body: new TextDecoder().decode(frame.body),
        },
        null,
        2,
    );
}

encode();
</script>

<style scoped>
.demo {
    margin-top: 2.5rem;
    padding: 1.4rem;
    border: 1px solid #dfe5eb;
    border-radius: 8px;
    background:#f8fafc;
}
.demo-head{display:flex;align-items:center;justify-content:space-between;gap:1rem;margin-bottom:.5rem}.demo-head>span{padding:.25rem .45rem;border-radius:3px;background:#e4f8f0;color:#187754;font:700 .55rem var(--font-mono);letter-spacing:.1em}

h2 {
    margin: 0 0 0.5rem;
    font-family: var(--font-body);
    font-size: 1.25rem;
    letter-spacing: 0;
}

.muted {
    color: var(--muted);
    margin: 0 0 0.5rem;
}

label {
    display: grid;
    gap: 0.35rem;
    margin: 0.75rem 0;
    color: var(--muted);
    font-size: 0.9rem;
}

input {
    border: 1px solid var(--line);
    border-radius: 5px;
    padding: 0.5rem 0.65rem;
    background: var(--surface);
    color: var(--ink);
}

.row {
    display: flex;
    gap: 0.6rem;
    flex-wrap: wrap;
    margin-bottom: 0.85rem;
}

.mono {
    white-space: pre-wrap;
    word-break: break-word;
    background: #0c1118;
    color: #dce5ed;
    padding: 0.85rem 1rem;
    border-radius: 6px;
    font-size: 0.85rem;
}
</style>
