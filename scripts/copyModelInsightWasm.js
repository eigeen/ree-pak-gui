import { copyFileSync, existsSync, mkdirSync } from 'node:fs'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const __dirname = dirname(fileURLToPath(import.meta.url))
const repoRoot = resolve(__dirname, '..')
const generatedDir = join(repoRoot, 'src-tauri/crates/model-insight-wasm/pkg')
const distWasmDir = join(repoRoot, 'dist/wasm/model-insight')
const wasmFiles = [
  'model_insight.js',
  'model_insight.d.ts',
  'model_insight_bg.wasm',
  'model_insight_bg.wasm.d.ts'
]

mkdirSync(distWasmDir, { recursive: true })

for (const file of wasmFiles) {
  const source = join(generatedDir, file)
  if (!existsSync(source)) {
    throw new Error(`Missing generated wasm artifact: ${source}`)
  }

  copyFileSync(source, join(distWasmDir, file))
}

console.log(`Copied model insight wasm artifacts to ${distWasmDir}`)
