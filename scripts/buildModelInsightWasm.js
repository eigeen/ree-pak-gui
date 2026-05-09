import { existsSync, lstatSync, mkdirSync, rmSync, symlinkSync } from 'node:fs'
import { homedir } from 'node:os'
import { dirname, join, relative, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { spawnSync } from 'node:child_process'

const WASM_BINDGEN_VERSION = '0.2.121'
const __dirname = dirname(fileURLToPath(import.meta.url))
const repoRoot = resolve(__dirname, '..')
const extensionDir = join(repoRoot, 'extensions/model-insight')
const generatedDir = join(extensionDir, 'pkg')
const frontendWasmDir = join(repoRoot, 'src/wasm/model-insight')
const wasmInput = join(
  extensionDir,
  'target/wasm32-unknown-unknown/release/model_insight.wasm'
)
const exeSuffix = process.platform === 'win32' ? '.exe' : ''
const frontendFiles = [
  'model_insight.js',
  'model_insight.d.ts',
  'model_insight_bg.wasm',
  'model_insight_bg.wasm.d.ts'
]

run('cargo', [
  'build',
  '--target',
  'wasm32-unknown-unknown',
  '--release',
  '--no-default-features'
], { cwd: extensionDir })

mkdirSync(generatedDir, { recursive: true })

const wasmBindgen = resolveWasmBindgen()
run(wasmBindgen, ['--target', 'web', '--out-dir', generatedDir, wasmInput])
linkFrontendWasmFiles()

function linkFrontendWasmFiles() {
  mkdirSync(frontendWasmDir, { recursive: true })

  for (const file of frontendFiles) {
    const source = join(generatedDir, file)
    const destination = join(frontendWasmDir, file)
    replaceWithSymlink(destination, source)
  }
}

function replaceWithSymlink(destination, source) {
  if (existsSync(destination)) {
    const stat = lstatSync(destination)
    if (!stat.isSymbolicLink()) {
      rmSync(destination, { force: true })
    } else {
      rmSync(destination)
    }
  }

  symlinkSync(relative(dirname(destination), source), destination)
}

function resolveWasmBindgen() {
  if (process.env.WASM_BINDGEN) {
    return process.env.WASM_BINDGEN
  }

  if (commandWorks(`wasm-bindgen${exeSuffix}`)) {
    return `wasm-bindgen${exeSuffix}`
  }

  for (const candidate of wasmPackCacheCandidates()) {
    if (existsSync(candidate)) return candidate
  }

  const installRoot = wasmPackInstallRoot()
  mkdirSync(installRoot, { recursive: true })
  run('cargo', [
    'install',
    'wasm-bindgen-cli',
    '--version',
    WASM_BINDGEN_VERSION,
    '--root',
    installRoot
  ])

  for (const candidate of wasmBindgenCandidates(installRoot)) {
    if (existsSync(candidate)) return candidate
  }

  throw new Error(`wasm-bindgen ${WASM_BINDGEN_VERSION} was installed but not found`)
}

function wasmPackCacheCandidates() {
  return wasmPackCacheRoots().flatMap((root) => wasmBindgenCandidates(wasmPackInstallRoot(root)))
}

function wasmPackCacheRoots() {
  return [
    join(homedir(), 'Library/Caches/.wasm-pack'),
    join(homedir(), '.cache/.wasm-pack'),
    join(homedir(), 'AppData/Local/.wasm-pack')
  ]
}

function wasmPackInstallRoot(root = wasmPackCacheRoots()[0]) {
  return join(root, `wasm-bindgen-cargo-install-${WASM_BINDGEN_VERSION}`)
}

function wasmBindgenCandidates(root) {
  return [
    join(root, `wasm-bindgen${exeSuffix}`),
    join(root, 'bin', `wasm-bindgen${exeSuffix}`)
  ]
}

function commandWorks(command) {
  const result = spawnSync(command, ['--version'], {
    stdio: 'ignore',
    shell: process.platform === 'win32'
  })
  return result.status === 0
}

function run(command, args, options = {}) {
  const result = spawnSync(command, args, {
    cwd: options.cwd ?? repoRoot,
    stdio: 'inherit',
    shell: process.platform === 'win32'
  })

  if (result.status !== 0) {
    throw new Error(`${command} ${args.join(' ')} failed with status ${result.status}`)
  }
}
