# 在应用中使用 YYDB

YYDB 同时提供 Rust 进程内嵌入和 TypeScript 产品包。两种方式使用同一套 Rust 数据库内核与 VOS 语义。

## TypeScript / Node.js

安装一个依赖即可。`@yydb/yydb` 会为当前平台准备引擎并管理本地连接，应用不需要额外安装程序或手动启动服务。

```bash
npm install @yydb/yydb
```

```ts
import { Database } from "@yydb/yydb";

const db = await Database.open("app.yydb");

await db.ensureSchema(1, `
  table Setting {
    @@setting_id: uuid
    @key: utf8
    value: utf8
  }
`);

await db.put("theme", "dark");
await db.close();
```

## Rust

Rust 宿主可以将 YYDB 直接嵌入进程，适合需要最小调用开销、明确生命周期控制或 Rust UDF 的应用。数据库仍然保存为可独立重新打开的 `.yydb` 文件。

## 底层客户端

`@yydb/yydb-client` 面向 WebUI、浏览器开发工具和需要自行管理连接的高级集成。常规 TypeScript 应用应优先使用 `@yydb/yydb`。
