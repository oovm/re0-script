<template>
    <div class="landing">
        <section class="hero">
            <div class="hero-glow" aria-hidden="true" />
            <div class="container hero-inner">
                <div class="hero-copy">
                    <p class="product-kicker"><span /> VOS-native embedded database</p>
                    <h1>The local database<br /><em>built for real data.</em></h1>
                    <p class="hero-cn">为应用而生的 VOS 原生数据库</p>
                    <p class="lede">一个可移植的 <code>.yydb</code> 文件，原生容纳结构化记录、文件与向量。嵌入进程，离线运行，不需要维护数据库服务。</p>
                    <div class="hero-actions">
                        <RouterLink class="button primary" to="/d/overview">开始使用 <span>→</span></RouterLink>
                        <a class="button secondary" href="https://github.com/yy-database/yydb.rs" target="_blank" rel="noreferrer">
                            <svg viewBox="0 0 24 24" aria-hidden="true"><path fill="currentColor" d="M12 .7a11.5 11.5 0 0 0-3.64 22.4c.58.1.79-.25.79-.56v-2.23c-3.22.7-3.9-1.37-3.9-1.37-.53-1.34-1.29-1.7-1.29-1.7-1.05-.72.08-.7.08-.7 1.16.08 1.78 1.2 1.78 1.2 1.03 1.77 2.71 1.26 3.37.96.1-.75.4-1.26.73-1.55-2.57-.29-5.27-1.28-5.27-5.69 0-1.26.45-2.28 1.19-3.09-.12-.29-.52-1.46.11-3.05 0 0 .97-.31 3.16 1.18a10.9 10.9 0 0 1 5.76 0c2.2-1.49 3.16-1.18 3.16-1.18.63 1.59.23 2.76.11 3.05.74.81 1.19 1.83 1.19 3.09 0 4.42-2.71 5.39-5.29 5.68.42.36.79 1.06.79 2.14v3.17c0 .31.21.67.8.56A11.5 11.5 0 0 0 12 .7Z" /></svg>
                            GitHub
                        </a>
                    </div>
                    <div class="install-line"><span>$</span><code>npm install @yydb/yydb</code><button type="button" title="复制安装命令" @click="copyInstall">{{ copied ? "Copied" : "Copy" }}</button></div>
                    <div class="trust-row">
                        <span><i class="live" /> Rust engine</span><span>MPL-2.0</span><span>Windows · macOS · Linux</span>
                    </div>
                </div>

                <div class="console" aria-label="YYDB 数据库控制台示意">
                    <div class="console-bar">
                        <div class="window-dots"><i /><i /><i /></div>
                        <span>studio.yydb.local</span>
                        <div class="console-status"><i /> Connected</div>
                    </div>
                    <div class="console-body">
                        <aside class="console-nav">
                            <div class="db-identity"><b>Y</b><span>studio.yydb<small>4.8 MB</small></span></div>
                            <p>DATABASE</p>
                            <a class="active"><span>▦</span> Tables <i>3</i></a>
                            <a><span>⌁</span> Objects <i>248</i></a>
                            <a><span>⌁</span> Vectors <i>12k</i></a>
                            <p>SYSTEM</p>
                            <a><span>◫</span> Schema</a>
                            <a><span>◉</span> Journal</a>
                        </aside>
                        <div class="console-main">
                            <div class="query-head"><span>Schema</span><button>Version 7 <kbd>✓</kbd></button></div>
                            <pre class="query"><code><span class="q-dim">table</span> User {
  <span class="q-string">@@user_id</span>: uuid,  @user_name: utf8,
  avatar: file?,
}</code></pre>
                            <div class="result-head"><span>User records <b>3 rows</b></span><span>Validated</span></div>
                            <div class="data-table">
                                <div class="tr th"><span>user_id <i>uuid</i></span><span>user_name <i>utf8</i></span><span>avatar <i>file?</i></span></div>
                                <div class="tr"><span>0194a6…e81</span><span>Lin</span><span class="file-value">◫ lin.webp</span></div>
                                <div class="tr"><span>0194a6…a06</span><span>Mori</span><span class="file-value">◫ mori.webp</span></div>
                                <div class="tr"><span>0194a6…bb9</span><span>Sol</span><span class="null">null</span></div>
                            </div>
                            <div class="engine-strip"><span><i /> WAL healthy</span><span>12 pages cached</span><span>objects/ 28.4 MB</span></div>
                        </div>
                    </div>
                </div>
            </div>
            <div class="container proof-bar">
                <div><small>DEPLOYMENT</small><strong>Embedded</strong><span>No server required</span></div>
                <div><small>STORAGE</small><strong>Single-file</strong><span>Portable database truth</span></div>
                <div><small>DURABILITY</small><strong>WAL + recovery</strong><span>Reopen after interruption</span></div>
                <div><small>DATA</small><strong>Rows + objects</strong><span>Files and vectors included</span></div>
            </div>
        </section>

        <section class="positioning">
            <div class="container section-intro">
                <p class="overline">WHY YYDB</p>
                <h2>SQLite 的部署形态。<br />为现代应用扩展的数据边界。</h2>
                <p>YYDB 保留本地数据库最重要的简单性，同时让 schema、二进制对象和向量共享一套生命周期。</p>
            </div>
            <div class="container value-grid">
                <article>
                    <div class="feature-icon file-icon"><span>.yydb</span></div>
                    <h3>数据库跟应用走</h3><p>复制、备份或随应用分发一个数据库文件。网络端点是可选项，不是运行前提。</p>
                    <ul><li>嵌入 Rust 进程</li><li>本地 Node.js 引擎</li><li>无需控制面</li></ul>
                </article>
                <article>
                    <div class="feature-icon wal-icon"><i /><i /><i /><i /></div>
                    <h3>写入之后，仍然可恢复</h3><p>Pager、B+Tree 和日志模式共同构成持久化路径，WAL 可以 checkpoint 回主文件。</p>
                    <ul><li>Delete / WAL journal</li><li>崩溃后重新打开</li><li>显式 checkpoint</li></ul>
                </article>
                <article>
                    <div class="feature-icon object-icon"><i /><i /><i /><i /><i /></div>
                    <h3>大对象不挤进行里</h3><p>文件和向量通过内容寻址对象引用保存。大文件分片，只读取请求覆盖的范围。</p>
                    <ul><li>Chunked CAS</li><li>Range reads</li><li>统一对象身份</li></ul>
                </article>
            </div>
        </section>

        <section class="vos">
            <div class="container vos-layout">
                <div class="vos-copy">
                    <p class="overline">VOS NATIVE</p>
                    <h2>Schema 不在数据库旁边。<br />它就在数据库里面。</h2>
                    <p>YYDB 原生保存版本化 VOS 文档，并在它成为 database truth 之前完成校验。主键、唯一约束和引用都是数据契约的一部分。</p>
                    <RouterLink to="/d/overview">了解 VOS 数据模型 <span>→</span></RouterLink>
                </div>
                <div class="schema-panel">
                    <div class="panel-bar"><span>schema.vos</span><span class="valid"><i /> Valid schema</span></div>
                    <pre><code><span class="syntax-key">table</span> <span class="syntax-type">User</span> {
  <mark>@@user_id</mark>: uuid,
  <span class="syntax-unique">@user_name</span>: utf8,
  avatar: file?,
  embedding: vector&lt;f32, 768&gt;?,
}

<span class="syntax-key">table</span> <span class="syntax-type">Post</span> {
  <mark>@@post_id</mark>: uuid,
  author: <span class="syntax-ref">&amp;User</span>,
  body: utf8,
}</code></pre>
                    <div class="schema-legend"><span><b>@@</b> Primary key</span><span><b>@</b> Unique</span><span><b>&amp;</b> Reference</span></div>
                </div>
            </div>
        </section>

        <section class="objects">
            <div class="container section-intro centered">
                <p class="overline">ONE OBJECT LAYER</p>
                <h2>Files and vectors, without another database.</h2>
                <p>同一个内容寻址对象库服务于文件、字节、向量载荷和未来的 ANN 段。</p>
            </div>
            <div class="container object-dashboard">
                <div class="object-sidebar"><span>OBJECT STORE</span><b>28.4 GB</b><small>of 64 GB local</small><div class="capacity"><i /></div><dl><dt>Hot</dt><dd>8.2 GB</dd><dt>Warm</dt><dd>12.7 GB</dd><dt>Cold</dt><dd>7.5 GB</dd></dl></div>
                <div class="object-content">
                    <div class="object-top"><span>Recent objects</span><div><button class="selected">All</button><button>Files</button><button>Vectors</button></div></div>
                    <div class="object-row head"><span>Object</span><span>Kind</span><span>Storage</span><span>Size</span><span>Status</span></div>
                    <div class="object-row"><span><i class="obj-symbol purple">F</i><b>hero-video.mp4</b><small>42 chunks</small></span><span>File</span><span>CAS</span><span>86.2 MB</span><span class="hot">● Hot</span></div>
                    <div class="object-row"><span><i class="obj-symbol cyan">V</i><b>product-embeddings</b><small>f32 · 768 dimensions</small></span><span>Vector</span><span>CAS</span><span>24.8 MB</span><span class="warm">● Warm</span></div>
                    <div class="object-row"><span><i class="obj-symbol green">B</i><b>source-archive.bin</b><small>8 chunks</small></span><span>Bytes</span><span>CAS</span><span>7.6 MB</span><span class="cold">● Cold</span></div>
                </div>
            </div>
        </section>

        <section class="get-started">
            <div class="container start-inner"><div><p class="overline">START LOCAL</p><h2>Your data deserves<br />a database of its own.</h2><p>从一个 `.yydb` 文件开始。没有账号，没有云端依赖。</p></div><div class="start-actions"><RouterLink class="button white" to="/d/overview">阅读文档 →</RouterLink><RouterLink class="button white" to="/playground">Playground →</RouterLink><a href="https://www.npmjs.com/package/@yydb/yydb" target="_blank" rel="noreferrer">View on npm ↗</a><a href="https://crates.io/crates/yydb" target="_blank" rel="noreferrer">View on crates.io ↗</a></div></div>
        </section>
    </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
const copied = ref(false);
async function copyInstall() {
    await navigator.clipboard.writeText("npm install @yydb/yydb");
    copied.value = true;
    window.setTimeout(() => (copied.value = false), 1600);
}
</script>

<style scoped>
.landing{--bg:#070a0f;--panel:#0d1118;--panel-2:#111722;--border:#202936;--text:#f6f8fb;--sub:#99a6b5;--green:#3ee6a8;--green-deep:#13b87d;--violet:#8b7cf6;--cyan:#43c8e8;overflow:hidden;background:#fff;color:#111827}.container{width:min(1180px,calc(100% - 3rem))}h1,h2,h3{font-family:var(--font-body);letter-spacing:0}.hero{position:relative;padding:8.5rem 0 0;background:var(--bg);color:var(--text);overflow:hidden}.hero::after{content:"";position:absolute;inset:0;background-image:linear-gradient(rgba(255,255,255,.025) 1px,transparent 1px),linear-gradient(90deg,rgba(255,255,255,.025) 1px,transparent 1px);background-size:56px 56px;mask-image:linear-gradient(to bottom,#000,transparent 75%);pointer-events:none}.hero-glow{position:absolute;width:620px;height:620px;right:5%;top:8%;background:radial-gradient(circle,rgba(61,230,168,.13),transparent 67%);pointer-events:none}.hero-inner{position:relative;z-index:1;display:grid;grid-template-columns:.92fr 1.08fr;gap:clamp(3rem,6vw,6rem);align-items:center;padding-bottom:6rem}.release-pill{display:inline-flex;align-items:center;gap:.65rem;padding:.38rem .65rem .38rem .4rem;border:1px solid #273142;border-radius:99px;color:#b8c2cf;text-decoration:none;font:600 .72rem var(--font-mono)}.release-pill span{padding:.25rem .45rem;background:rgba(62,230,168,.12);color:var(--green);border-radius:99px;font-size:.6rem}.release-pill i{color:var(--green);font-style:normal}.hero h1{margin:1.7rem 0 0;font-size:clamp(3.25rem,5.5vw,5.25rem);line-height:.98;font-weight:680}.hero h1 em{font-style:normal;color:var(--green)}.hero-cn{margin:1.4rem 0 0;font-weight:650;color:#d9e0e8}.lede{max-width:35rem;margin:.7rem 0 0;color:var(--sub);font-size:1.04rem;line-height:1.7}.lede code{color:#dce4ed}.hero-actions{display:flex;gap:.75rem;margin-top:1.8rem}.button{display:inline-flex;align-items:center;justify-content:center;gap:.65rem;min-height:48px;padding:0 1.2rem;border:1px solid transparent;border-radius:6px;text-decoration:none;font-weight:700;transition:.18s}.button:hover{transform:translateY(-1px)}.button.primary{background:var(--green);color:#04120d}.button.primary:hover{background:#69edbd}.button.secondary{border-color:#303a49;color:#e5ebf2;background:#10151d}.button.secondary:hover{border-color:#566173;color:#fff}.button svg{width:18px;height:18px}.install-line{display:grid;grid-template-columns:auto 1fr auto;align-items:center;max-width:430px;margin-top:1.25rem;padding:.7rem .8rem;border:1px solid #273142;border-radius:6px;background:#0c1118;font:500 .72rem var(--font-mono)}.install-line>span{color:var(--green);margin-right:.6rem}.install-line code{color:#d7dfe8;white-space:nowrap;overflow:hidden;text-overflow:ellipsis}.install-line button{border:0;background:transparent;color:#778596;font:600 .62rem var(--font-mono);cursor:pointer}.trust-row{display:flex;flex-wrap:wrap;gap:1.1rem;margin-top:1.2rem;color:#6f7d8d;font:500 .64rem var(--font-mono)}.trust-row span{display:flex;align-items:center;gap:.35rem}.live{width:6px;height:6px;border-radius:50%;background:var(--green);box-shadow:0 0 0 3px rgba(62,230,168,.12)}
.console{position:relative;border:1px solid #283241;border-radius:8px;background:var(--panel);box-shadow:0 40px 100px rgba(0,0,0,.48),0 0 0 1px rgba(255,255,255,.02);overflow:hidden}.console-bar{height:43px;display:grid;grid-template-columns:1fr auto 1fr;align-items:center;padding:0 .85rem;border-bottom:1px solid var(--border);color:#7f8c9b;font:500 .6rem var(--font-mono)}.window-dots{display:flex;gap:5px}.window-dots i{width:7px;height:7px;border-radius:50%;background:#344050}.console-status{justify-self:end;display:flex;align-items:center;gap:.4rem;color:#9eacb9}.console-status i{width:6px;height:6px;border-radius:50%;background:var(--green)}.console-body{display:grid;grid-template-columns:155px 1fr;min-height:450px}.console-nav{padding:1rem .7rem;border-right:1px solid var(--border);background:#0a0e14}.db-identity{display:flex;align-items:center;gap:.65rem;padding:.3rem .35rem 1rem}.db-identity>b{display:grid;place-items:center;width:29px;height:29px;border-radius:6px;background:var(--violet);font-size:.72rem}.db-identity>span{display:flex;flex-direction:column;font:600 .7rem var(--font-mono)}.db-identity small{margin-top:.15rem;color:#637182;font-size:.55rem}.console-nav p{margin:1rem .5rem .35rem;color:#4f5c6d;font:650 .5rem var(--font-mono);letter-spacing:.1em}.console-nav a{display:grid;grid-template-columns:18px 1fr auto;align-items:center;padding:.5rem .55rem;border-radius:4px;color:#738193;font-size:.68rem}.console-nav a.active{background:#151c27;color:#dce4ec}.console-nav a i{font-style:normal;color:#526071}.console-main{min-width:0}.query-head,.result-head{display:flex;align-items:center;justify-content:space-between;height:42px;padding:0 .9rem;border-bottom:1px solid var(--border);color:#9ca8b5;font:600 .65rem var(--font-mono)}.query-head button{padding:.35rem .6rem;border:1px solid rgba(62,230,168,.24);border-radius:4px;background:rgba(62,230,168,.1);color:var(--green);font:600 .58rem var(--font-mono)}.query-head kbd{margin-left:.4rem;color:#6c7a89}.query{height:126px;margin:0;padding:1rem 1.2rem;border-bottom:1px solid var(--border);color:#e5ebf1;background:#0b1017;font:500 .72rem/1.7 var(--font-mono);overflow:auto}.q-dim{color:var(--violet)}.q-string{color:var(--green)}.result-head b{margin-left:.4rem;padding:.2rem .4rem;background:#18202c;color:#738193;border-radius:3px;font-size:.5rem}.result-head span:last-child{color:var(--green)}.data-table{font:500 .58rem var(--font-mono)}.tr{display:grid;grid-template-columns:1.2fr .8fr 1fr;min-height:46px;align-items:center;padding:0 .9rem;border-bottom:1px solid #19212c;color:#b5c0cc}.tr.th{min-height:38px;background:#0b1016;color:#6d7a89}.tr.th span{display:flex;flex-direction:column;gap:.15rem}.tr.th i{font-style:normal;color:#465362;font-size:.48rem}.file-value{color:var(--cyan)}.null{color:#536171;font-style:italic}.engine-strip{display:flex;gap:1rem;padding:.65rem .9rem;color:#596777;font:500 .52rem var(--font-mono)}.engine-strip span:first-child{color:#9bacb8}.engine-strip i{display:inline-block;width:5px;height:5px;margin-right:.3rem;border-radius:50%;background:var(--green)}.proof-bar{position:relative;z-index:1;display:grid;grid-template-columns:repeat(4,1fr);border-top:1px solid var(--border)}.proof-bar>div{padding:1.45rem 1.5rem;border-right:1px solid var(--border)}.proof-bar>div:last-child{border-right:0}.proof-bar small{display:block;color:#586676;font:600 .52rem var(--font-mono);letter-spacing:.1em}.proof-bar strong{display:block;margin:.45rem 0 .2rem;color:#e5ebf1;font-size:.9rem}.proof-bar span{color:#6d7988;font-size:.7rem}
.positioning,.objects{padding:clamp(5.5rem,9vw,8rem) 0;background:#f7f9fb}.section-intro{display:grid;grid-template-columns:.55fr 1.6fr 1fr;gap:2rem;align-items:start}.overline{margin:.25rem 0 0;color:#667085;font:700 .64rem var(--font-mono);letter-spacing:.12em}.section-intro h2,.vos h2,.availability h2,.get-started h2{margin:0;font-size:clamp(2.35rem,4vw,4.1rem);line-height:1.07;font-weight:680}.section-intro>p:last-child{margin:0;color:#667085;line-height:1.75}.value-grid{display:grid;grid-template-columns:repeat(3,1fr);gap:1rem;margin-top:4rem}.value-grid article{padding:1.6rem;border:1px solid #e0e5eb;border-radius:8px;background:#fff;box-shadow:0 8px 24px rgba(15,23,42,.035)}.feature-icon{height:145px;margin:-.4rem -.4rem 1.6rem;border:1px solid #e7ebf0;border-radius:6px;background:#f8fafc;overflow:hidden}.file-icon{display:grid;place-items:center;background-image:radial-gradient(#d9dee6 1px,transparent 1px);background-size:14px 14px}.file-icon span{display:grid;place-items:center;width:92px;height:112px;border:1px solid #c9d1dc;border-radius:5px;background:#fff;color:#6b58d9;box-shadow:8px 8px 0 #e9e6fb;font:700 .74rem var(--font-mono)}.wal-icon{display:flex;align-items:center;justify-content:center;gap:5px}.wal-icon i{width:44px;height:68px;border:1px solid #cad2dc;background:#fff}.wal-icon i:nth-child(2){height:88px;background:#def8ed;border-color:#9be3c7}.wal-icon i:nth-child(3){height:56px}.object-icon{display:flex;align-items:center;justify-content:center;gap:4px}.object-icon i{height:28px;background:#8b7cf6}.object-icon i:nth-child(1){width:16%}.object-icon i:nth-child(2){width:23%;opacity:.85}.object-icon i:nth-child(3){width:12%;opacity:.7}.object-icon i:nth-child(4){width:28%;opacity:.5}.object-icon i:nth-child(5){width:9%;opacity:.3}.value-grid h3{margin:0 0 .6rem;font-size:1.2rem}.value-grid article>p{min-height:4.4em;margin:0;color:#667085;font-size:.9rem;line-height:1.6}.value-grid ul{margin:1.3rem 0 0;padding:1.1rem 0 0;border-top:1px solid #e8ecf1;list-style:none}.value-grid li{position:relative;padding:.25rem 0 .25rem 1.1rem;color:#475467;font-size:.8rem}.value-grid li::before{content:"✓";position:absolute;left:0;color:#18a875;font-weight:800}
.vos{padding:clamp(5.5rem,9vw,8rem) 0;background:#0b0f16;color:#f3f6f9}.vos-layout{display:grid;grid-template-columns:.9fr 1.1fr;gap:clamp(3rem,8vw,8rem);align-items:center}.vos-copy>p:not(.overline){max-width:34rem;color:#8996a5;line-height:1.75}.vos-copy>a{display:inline-flex;gap:.7rem;margin-top:1rem;color:var(--green);text-decoration:none;font-weight:700}.schema-panel{border:1px solid #26303e;border-radius:8px;background:#0e141d;box-shadow:0 26px 70px rgba(0,0,0,.28);overflow:hidden}.panel-bar{display:flex;justify-content:space-between;padding:.85rem 1rem;border-bottom:1px solid #26303e;color:#8491a0;font:600 .65rem var(--font-mono)}.valid{color:#a5b3bf}.valid i{display:inline-block;width:6px;height:6px;margin-right:.35rem;border-radius:50%;background:var(--green)}.schema-panel pre{margin:0;padding:1.7rem;color:#dce3eb;font:500 clamp(.78rem,1.3vw,.94rem)/1.85 var(--font-mono);overflow:auto}.syntax-key{color:#68778a}.syntax-type{color:#fff;font-weight:700}.schema-panel mark{color:#66e3b4;background:transparent}.syntax-unique{color:#f0c76b}.syntax-ref{color:#9b8df5}.schema-legend{display:flex;gap:1.4rem;padding:1rem 1.7rem;border-top:1px solid #26303e;color:#7d8a99;font-size:.68rem}.schema-legend b{margin-right:.3rem;color:#d8e0e8;font-family:var(--font-mono)}
.objects{background:#fff}.section-intro.centered{display:block;max-width:850px;text-align:center}.centered h2{margin-top:1rem}.centered>p:last-child{margin:1rem auto 0;max-width:40rem}.object-dashboard{display:grid;grid-template-columns:220px 1fr;margin-top:4rem;border:1px solid #dfe5eb;border-radius:8px;background:#fff;box-shadow:0 30px 80px rgba(15,23,42,.09);overflow:hidden}.object-sidebar{padding:1.5rem;border-right:1px solid #e3e8ee;background:#f8fafc}.object-sidebar>span{display:block;color:#8692a1;font:700 .55rem var(--font-mono);letter-spacing:.1em}.object-sidebar>b{display:block;margin:1.6rem 0 .2rem;font-size:1.65rem}.object-sidebar>small{color:#8a96a4}.capacity{height:6px;margin:1.4rem 0;background:#e2e8ef;border-radius:99px}.capacity i{display:block;width:44%;height:100%;background:#27c48f;border-radius:99px}.object-sidebar dl{display:grid;grid-template-columns:1fr auto;gap:.75rem;margin-top:1.5rem;font-size:.72rem}.object-sidebar dt{color:#667085}.object-sidebar dd{margin:0;font-weight:650}.object-top{display:flex;align-items:center;justify-content:space-between;padding:1rem 1.2rem;border-bottom:1px solid #e4e8ed;font-weight:700}.object-top button{padding:.35rem .65rem;border:0;border-radius:4px;background:transparent;color:#788494;font-size:.65rem}.object-top button.selected{background:#eef1f5;color:#273141}.object-row{display:grid;grid-template-columns:1.8fr 1.2fr .7fr .7fr .6fr;align-items:center;min-height:69px;padding:0 1.2rem;border-bottom:1px solid #e9edf1;color:#596579;font-size:.7rem}.object-row.head{min-height:38px;background:#fafbfc;color:#929ca8;font:600 .58rem var(--font-mono)}.object-row>span:first-child:not(.head span){display:grid;grid-template-columns:34px 1fr;grid-template-rows:auto auto;column-gap:.7rem;align-items:center}.obj-symbol{grid-row:1/3;display:grid;place-items:center;width:34px;height:34px;border-radius:5px;color:#fff;font:700 .65rem var(--font-mono)}.obj-symbol.purple{background:#7c6ee3}.obj-symbol.cyan{background:#27a9c7}.obj-symbol.green{background:#21a879}.object-row b{color:#273141}.object-row small{color:#9aa4b0;font-family:var(--font-mono)}.hot{color:#16875f}.warm{color:#9d6b12}.cold{color:#687386}
.availability{padding:clamp(5rem,8vw,7rem) 0;background:#f5f7fa}.availability-layout{display:grid;grid-template-columns:.9fr 1.1fr;gap:clamp(3rem,8vw,8rem)}.availability-layout>div:first-child>p:last-child{max-width:34rem;color:#667085;line-height:1.7}.status-list{border-top:1px solid #d6dce3}.status-list>div{display:grid;grid-template-columns:85px 1fr 1.2fr;align-items:center;min-height:68px;border-bottom:1px solid #d6dce3}.status-list span{width:max-content;padding:.25rem .4rem;border-radius:3px;font:700 .5rem var(--font-mono)}.ship{color:#087852;background:#d8f4e9}.building{color:#765e13;background:#f6edc8}.status-list b{font-size:.86rem}.status-list small{color:#7e8997}.get-started{padding:clamp(5rem,8vw,7rem) 0;background:#644ff1;color:#fff}.start-inner{display:grid;grid-template-columns:1.2fr .8fr;gap:4rem;align-items:end}.start-inner .overline{color:#c6bffc}.start-inner>div>p:last-child{color:#d5d0fa}.start-actions{display:flex;flex-direction:column;align-items:flex-start;gap:1rem}.button.white{background:#fff;color:#392b9e}.start-actions>a:not(.button){color:#e3dfff;text-decoration:none;font-weight:650;font-size:.86rem}
@media(max-width:960px){.hero-inner{grid-template-columns:1fr}.hero-copy{max-width:700px}.console{max-width:750px}.section-intro{grid-template-columns:1fr}.value-grid{grid-template-columns:1fr}.feature-icon{height:120px}.value-grid article>p{min-height:0}.vos-layout,.availability-layout{grid-template-columns:1fr}.object-dashboard{grid-template-columns:1fr}.object-sidebar{border-right:0;border-bottom:1px solid #e3e8ee}.object-sidebar dl{max-width:280px}.start-inner{grid-template-columns:1fr}.proof-bar{grid-template-columns:1fr 1fr}.proof-bar>div:nth-child(2){border-right:0}.proof-bar>div:nth-child(-n+2){border-bottom:1px solid var(--border)}}
@media(max-width:640px){.container{width:min(100% - 2rem,1180px)}.hero{padding-top:6.5rem}.hero-inner{gap:2.5rem;padding-bottom:3.5rem}.hero h1{font-size:2.85rem}.hero-cn{font-size:.9rem}.hero-actions{flex-wrap:wrap}.install-line{max-width:100%}.trust-row span:last-child{width:100%}.console-body{grid-template-columns:1fr;min-height:390px}.console-nav{display:none}.console-bar{grid-template-columns:1fr 1fr}.console-bar>span{display:none}.query{height:110px}.tr{grid-template-columns:1.15fr .7fr 1fr;padding:0 .7rem}.engine-strip span:nth-child(n+2){display:none}.proof-bar{grid-template-columns:1fr 1fr}.proof-bar>div{padding:1.1rem .6rem}.proof-bar strong{font-size:.78rem}.proof-bar span{display:none}.positioning,.objects{padding:4.5rem 0}.section-intro h2,.vos h2,.availability h2,.get-started h2{font-size:2.35rem}.value-grid{margin-top:2.5rem}.vos{padding:4.5rem 0}.vos-layout{gap:2.5rem}.schema-panel{margin-inline:-.4rem}.schema-panel pre{padding:1.1rem}.schema-legend{padding:1rem;gap:.7rem;flex-wrap:wrap}.object-dashboard{margin-top:2.5rem}.object-row{grid-template-columns:1.6fr .8fr .6fr;padding:0 .7rem}.object-row>span:nth-child(3),.object-row>span:nth-child(4){display:none}.object-row.head span:nth-child(2){display:block}.object-row.head span:nth-child(5){display:block}.object-content{overflow:hidden}.status-list>div{grid-template-columns:78px 1fr}.status-list small{display:none}.start-actions{gap:.8rem}}
.product-kicker{display:flex;align-items:center;gap:.55rem;margin:0;color:#94a1b0;font:600 .68rem var(--font-mono);text-transform:uppercase}.product-kicker span{width:7px;height:7px;border-radius:50%;background:var(--green);box-shadow:0 0 0 4px rgba(62,230,168,.1)}
</style>
