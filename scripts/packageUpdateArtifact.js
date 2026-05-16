import { createHash } from 'node:crypto'
import { execFileSync } from 'node:child_process'
import { appendFile, chmod, mkdir, mkdtemp, readFile, rm, writeFile } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import path from 'node:path'
import { parseArgs } from 'node:util'

import { zipSync } from 'fflate'

const ROOT_DIR = process.cwd()
const DEFAULT_OUTPUT_DIR = path.join('dist', 'release')
const DEFAULT_BINARY_NAME = 'ree-pak-gui'

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

  try {
    return execFileSync('jj', ['log', '-r', '@', '-T', 'commit_id.short(7)', '--no-graph'], {
      cwd: ROOT_DIR,
      encoding: 'utf8',
      stdio: ['ignore', 'pipe', 'ignore']
    }).trim()
  } catch {
    // Fall back to git for non-jj environments such as GitHub Actions.
  }

  return execFileSync('git', ['rev-parse', '--short', 'HEAD'], {
    cwd: ROOT_DIR,
    encoding: 'utf8'
  }).trim()
}

function getHostTarget() {
  const versionOutput = execFileSync('rustc', ['-vV'], {
    cwd: ROOT_DIR,
    encoding: 'utf8'
  })
  const hostMatch = versionOutput.match(/^host:\s*(.+)$/m)
  if (!hostMatch) {
    throw new Error('Failed to read host target from rustc -vV')
  }

  return hostMatch[1].trim()
}

function sha256Hex(buffer) {
  return createHash('sha256').update(buffer).digest('hex')
}

function resolveExecutableName(target, binaryName) {
  return target.includes('windows') ? `${binaryName}.exe` : binaryName
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
    exe: { type: 'string' },
    'out-dir': { type: 'string', default: DEFAULT_OUTPUT_DIR },
    target: { type: 'string' },
    'bin-name': { type: 'string', default: DEFAULT_BINARY_NAME },
    version: { type: 'string' },
    commit: { type: 'string' }
  },
  allowPositionals: false
})

const target = values.target ?? process.env.TARGET ?? getHostTarget()
const binaryName = values['bin-name']
const executableName = resolveExecutableName(target, binaryName)
const exePath = resolvePath(
  values.exe ?? path.join('src-tauri', 'target', 'release', executableName)
)
const outputDir = resolvePath(values['out-dir'])
const version = values.version ?? (await readVersionFromCargoToml())
const commitHash = values.commit ?? getShortCommitHash()

const archiveExtension = target.includes('windows') ? 'zip' : 'tar.gz'
const archiveName = `${binaryName}-v${version}-${target}.${archiveExtension}`
const archivePath = path.join(outputDir, archiveName)
const manifestPath = path.join(outputDir, `update-artifact-${target}.json`)
const binPathInArchive = executableName

const executableBuffer = await readFile(exePath)

await mkdir(outputDir, { recursive: true })

if (target.includes('windows')) {
  const archiveBuffer = Buffer.from(
    zipSync(
      {
        [binPathInArchive]: new Uint8Array(executableBuffer)
      },
      {
        level: 9
      }
    )
  )
  await writeFile(archivePath, archiveBuffer)
} else {
  const stagingDir = await mkdtemp(path.join(tmpdir(), 'ree-pak-release-'))
  const stagedExecutablePath = path.join(stagingDir, binPathInArchive)
  try {
    await writeFile(stagedExecutablePath, executableBuffer)
    await chmod(stagedExecutablePath, 0o755)
    execFileSync('tar', ['-czf', archivePath, '-C', stagingDir, binPathInArchive], {
      cwd: ROOT_DIR,
      stdio: 'inherit'
    })
  } finally {
    await rm(stagingDir, { force: true, recursive: true })
  }
}

const archiveBuffer = await readFile(archivePath)
const archiveSha256 = sha256Hex(archiveBuffer)
const manifest = {
  version,
  tag: `v${version}`,
  target,
  commitHash,
  executable: {
    inputPath: exePath,
    binPathInArchive,
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
  target,
  archive_name: archiveName,
  archive_path: archivePath,
  archive_sha256: archiveSha256,
  manifest_path: manifestPath,
  executable_path: exePath,
  bin_path_in_archive: binPathInArchive
})

console.log(`Prepared update archive: ${archivePath}`)
console.log(`Archive sha256: ${archiveSha256}`)
console.log(`Manifest: ${manifestPath}`)
