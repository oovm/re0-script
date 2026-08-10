# YYDB user documentation

This package contains the YYDB product homepage and the Chinese user guides. It is the documentation surface for people
evaluating or using YYDB; the database engine and client packages live alongside it in this repository.

## Site structure

- `/` — product landing
- `/d/:path` — Chinese guides (`overview`, `guide/embed-and-serve`, …)
- `/playground` — local wire Playground (same origin; needs `yydb serve`)

Built with Vue 3, Vite, vue-router, and Shiki for fenced-code highlighting.

## User guides

- [中文文档首页](https://github.com/yy-database/yydb.rs/tree/dev/frontends/homepage/documentation/zh-hans)
- [嵌入与
  `serve`](https://github.com/yy-database/yydb.rs/blob/dev/frontends/homepage/documentation/zh-hans/guide/embed-and-serve.md)
- [Serve 安全边界](https://github.com/yy-database/yydb.rs/blob/dev/frontends/homepage/documentation/zh-hans/guide/serve-security.md)
- [线协议概览](https://github.com/yy-database/yydb.rs/blob/dev/frontends/homepage/documentation/zh-hans/guide/wire-protocol.md)

## Run the docs site locally

```bash
pnpm install
pnpm --filter @yydb/yydb-client build
pnpm --filter @yydb/yydb-homepage dev
```

Open `http://localhost:5173/`.

Create a production build with:

```bash
pnpm --filter @yydb/yydb-homepage build
```

The repository's implementation and protocol notes are maintained separately in [
`yydb/documentation/`](https://github.com/yy-database/yydb.rs/tree/dev/documentation).

## Related packages

| Package                                                                                    | Use                     |
|--------------------------------------------------------------------------------------------|-------------------------|
| [`@yydb/yydb`](https://www.npmjs.com/package/@yydb/yydb)                                   | Node.js file API        |
| [`@yydb/yydb-client`](https://www.npmjs.com/package/@yydb/yydb-client)                     | Browser and wire client |
| [`@yydb/yydb-webui`](https://github.com/yy-database/yydb.rs/tree/dev/frontends/yydb-webui) | Optional standalone Playground build; prefer homepage `/playground` |
