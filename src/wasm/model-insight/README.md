# model-insight wasm

Generated with:

```bash
bun run build:model-insight-wasm
```

The frontend loader imports `model_insight.js` from this directory at runtime.
Generated files here are symlinks to `extensions/model-insight/pkg` so wasm
development can rebuild the extension without copying artifacts by hand.
