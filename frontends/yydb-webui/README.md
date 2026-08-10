# `@yydb/yydb-webui`

Local browser UI for inspecting a running YYDB server.

**Canonical surface:** the product homepage hosts Playground at
[`/playground`](../homepage) on the same Vite origin — do not run this package as a
separate day-to-day port.

```bash
pnpm --filter @yydb/yydb-homepage dev
# open http://localhost:5173/playground
```

This package remains a standalone build target if you want a static bundle next to a
private `yydb serve` endpoint. Prefer the homepage route for development and docs.

## Engine

```text
yydb serve ./app.yydb --bind 127.0.0.1:7700
```

Connect the UI to `ws://127.0.0.1:7700/wire`.

## Standalone (optional)

```bash
pnpm --filter @yydb/yydb-client build
pnpm --filter @yydb/yydb-webui dev
```

## Security

The reference server has no authentication. Keep the UI and endpoint on your machine
or a trusted private network.

## Links

- [Homepage / Playground](../homepage)
- [YYDB user guides](../homepage/documentation/zh-hans)
- [License](../../License.md)
