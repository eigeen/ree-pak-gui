import { createHash } from 'node:crypto'
import { execFileSync } from 'node:child_process'
import { appendFile, mkdir, readFile, writeFile } from 'node:fs/promises'
import path from 'node:path'
import { parseArgs } from 'node:util'

import { zipSync } from 'fflate'

const ROOT_DIR = process.cwd()
const DEFAULT_EXE_PATH = path.join('src-tauri', 'target', 'release', 'ree-pak-rs.exe')
const DEFAULT_OUTPUT_DIR = path.join('dist', 'release')

function resolvePath(filePath) {
  return path.isAbsolute(filePath) ? filePath : path.join(ROOT_DIR, filePath)
}

function readVersionFromCargoToml() {
  const cargoTomlPath = resolvePath(path.join('src-tauri', 'Cargo.toml'))
  return readFile(cargoTomlPath, 'utf8').then((content) => {
    const versionMatch = content.match(/^version\s*=\s*"([^"]+)"/m)
    if (!versionMatch) {
      throw new Error(`Failed to read version from ${cargoTomlPath}`)
    }

    return versionMatch[1]
  })
}

function getShortCommitHash() {
  const shaFromEnv = process.env.GITHUB_SHA?.trim()
  if (shaFromEnv) {
    return shaFromEnv.slice(0, 7)
  }

  return execFileSync('git', ['rev-parse', '--short', 'HEAD'], {
    cwd: ROOT_DIR,
    encoding: 'utf8'
  }).trim()
}

function sha256Hex(buffer) {
  return createHash('sha256').update(buffer).digest('hex')
}

function formatOutputs(outputs) {
  const githubOutput = process.env.GITHUB_OUTPUT
  if (!githubOutput) {
    return Promise.resolve()
  }

  const content = Object.entries(outputs)
    .map(([key, value]) => `${key}=${value}`)
    .join('\n')

  return appendFile(githubOutput, `${content}\n`, 'utf8')
}

const { values } = parseArgs({
  options: {
    exe: { type: 'string', default: DEFAULT_EXE_PATH },
    'out-dir': { type: 'string', default: DEFAULT_OUTPUT_DIR },
    platform: { type: 'string', default: 'windows' },
    arch: { type: 'string', default: 'x86_64' },
    version: { type: 'string' },
    commit: { type: 'string' }
  },
  allowPositionals: false
})

const exePath = resolvePath(values.exe)
const outputDir = resolvePath(values['out-dir'])
const version = values.version ?? (await readVersionFromCargoToml())
const commitHash = values.commit ?? getShortCommitHash()
const platform = values.platform
const arch = values.arch

const packagedExecutableName = `ree-pak-gui_${version}_${platform}_${arch}_release_${commitHash}.exe`
const archiveName = `${packagedExecutableName}.zip`
const archivePath = path.join(outputDir, archiveName)
const manifestPath = path.join(outputDir, 'update-artifact.json')

const executableBuffer = await readFile(exePath)
const archiveBuffer = Buffer.from(
  zipSync(
    {
      [packagedExecutableName]: new Uint8Array(executableBuffer)
    },
    {
      level: 9
    }
  )
)

await mkdir(outputDir, { recursive: true })
await writeFile(archivePath, archiveBuffer)

const archiveSha256 = sha256Hex(archiveBuffer)
const manifest = {
  version,
  tag: `v${version}`,
  platform,
  arch,
  commitHash,
  executable: {
    inputPath: exePath,
    packagedName: packagedExecutableName,
    size: executableBuffer.byteLength
  },
  archive: {
    name: archiveName,
    path: archivePath,
    size: archiveBuffer.byteLength,
    sha256: archiveSha256
  }
}

await writeFile(manifestPath, `${JSON.stringify(manifest, null, 2)}\n`, 'utf8')
await formatOutputs({
  version,
  tag: manifest.tag,
  archive_name: archiveName,
  archive_path: archivePath,
  archive_sha256: archiveSha256,
  manifest_path: manifestPath,
  executable_path: exePath
})

console.log(`Prepared update archive: ${archivePath}`)
console.log(`Archive sha256: ${archiveSha256}`)
console.log(`Manifest: ${manifestPath}`)
