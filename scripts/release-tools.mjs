import { execFileSync } from 'node:child_process';
import { readFileSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

export const VERSION_FILES = [
  'package.json',
  'src-tauri/tauri.conf.json',
  'src-tauri/Cargo.toml',
  'src-tauri/Cargo.lock',
];

const SEMVER_PATTERN = /^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/;

function read(root, file) {
  return readFileSync(path.join(root, file), 'utf8');
}

function write(root, file, contents) {
  writeFileSync(path.join(root, file), contents, 'utf8');
}

function packageVersion(root) {
  return JSON.parse(read(root, 'package.json')).version;
}

function tauriVersion(root) {
  return JSON.parse(read(root, 'src-tauri/tauri.conf.json')).version;
}

function cargoManifestVersion(root) {
  const match = read(root, 'src-tauri/Cargo.toml').match(/^\[package\][\s\S]*?^version\s*=\s*"([^"]+)"/m);
  return match?.[1];
}

function cargoLockVersion(root) {
  const match = read(root, 'src-tauri/Cargo.lock').match(
    /^name\s*=\s*"modern-retro-rom-manager"\r?\nversion\s*=\s*"([^"]+)"/m,
  );
  return match?.[1];
}

export function getVersions(root) {
  return {
    'package.json': packageVersion(root),
    'src-tauri/tauri.conf.json': tauriVersion(root),
    'src-tauri/Cargo.toml': cargoManifestVersion(root),
    'src-tauri/Cargo.lock': cargoLockVersion(root),
  };
}

export function validateRelease(root) {
  const errors = [];
  const versions = getVersions(root);
  const expected = versions['package.json'];

  if (!SEMVER_PATTERN.test(expected ?? '')) {
    errors.push(`package.json 中的版本号无效: ${expected ?? '未找到'}`);
  }

  for (const [file, version] of Object.entries(versions)) {
    if (!version) {
      errors.push(`${file} 中未找到应用版本号`);
    } else if (version !== expected) {
      errors.push(`${file} 为 ${version}，应与 package.json 的 ${expected} 一致`);
    }
  }

  const changelog = read(root, 'CHANGELOG.md');
  const firstRelease = changelog.match(/^## ([^\s]+) \(/m)?.[1];
  if (firstRelease !== expected) {
    errors.push(`CHANGELOG.md 顶部版本为 ${firstRelease ?? '未找到'}，应为 ${expected}`);
  }
  const escapedVersion = (expected ?? '').replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  const sectionMatch = changelog.match(
    new RegExp(
      `^## ${escapedVersion} \\((\\d{4}-\\d{2}-\\d{2})\\)\\s*$([\\s\\S]*?)(?=^## |(?![\\s\\S]))`,
      'm',
    ),
  );

  if (!sectionMatch) {
    errors.push(`CHANGELOG.md 缺少“## ${expected} (YYYY-MM-DD)”版本段落`);
  } else {
    const section = sectionMatch[2];
    if (!/^\s*-\s+\S+/m.test(section) || /待补充|\bTODO\b/i.test(section)) {
      errors.push(`CHANGELOG.md 的 ${expected} 版本说明仍为空或包含待补充内容`);
    }
  }

  return { ok: errors.length === 0, version: expected, versions, errors };
}

function replaceOnce(contents, pattern, replacement, file) {
  if (!pattern.test(contents)) {
    throw new Error(`无法在 ${file} 中定位版本号`);
  }
  return contents.replace(pattern, replacement);
}

export function bumpVersion(root, nextVersion, date = new Date().toISOString().slice(0, 10)) {
  if (!SEMVER_PATTERN.test(nextVersion)) {
    throw new Error(`无效的 SemVer 版本号: ${nextVersion}`);
  }

  const packageJson = read(root, 'package.json');
  write(
    root,
    'package.json',
    replaceOnce(packageJson, /("version"\s*:\s*")[^"]+(")/, `$1${nextVersion}$2`, 'package.json'),
  );

  const tauriConfig = read(root, 'src-tauri/tauri.conf.json');
  write(
    root,
    'src-tauri/tauri.conf.json',
    replaceOnce(
      tauriConfig,
      /("version"\s*:\s*")[^"]+(")/,
      `$1${nextVersion}$2`,
      'src-tauri/tauri.conf.json',
    ),
  );

  const cargoToml = read(root, 'src-tauri/Cargo.toml');
  write(
    root,
    'src-tauri/Cargo.toml',
    replaceOnce(
      cargoToml,
      /(^\[package\][\s\S]*?^version\s*=\s*")[^"]+("\s*$)/m,
      `$1${nextVersion}$2`,
      'src-tauri/Cargo.toml',
    ),
  );

  const cargoLock = read(root, 'src-tauri/Cargo.lock');
  write(
    root,
    'src-tauri/Cargo.lock',
    replaceOnce(
      cargoLock,
      /(^name\s*=\s*"modern-retro-rom-manager"\r?\nversion\s*=\s*")[^"]+("\s*$)/m,
      `$1${nextVersion}$2`,
      'src-tauri/Cargo.lock',
    ),
  );

  const changelog = read(root, 'CHANGELOG.md');
  if (!new RegExp(`^## ${nextVersion.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')} \\(`, 'm').test(changelog)) {
    const heading = '# 更新日志';
    const releaseSection = `\n\n## ${nextVersion} (${date})\n\n### 更新内容\n- 待补充\n`;
    write(root, 'CHANGELOG.md', changelog.replace(heading, `${heading}${releaseSection}`));
  }
}

export function git(root, args) {
  return execFileSync('git', args, { cwd: root, encoding: 'utf8' }).trim();
}

export function stagedReleaseErrors(root) {
  const staged = new Set(git(root, ['diff', '--cached', '--name-only']).split(/\r?\n/).filter(Boolean));
  const versionFilesStaged = VERSION_FILES.filter((file) => staged.has(file));
  if (versionFilesStaged.length === 0) return [];

  const required = [...VERSION_FILES, 'CHANGELOG.md'];
  const missing = required.filter((file) => !staged.has(file));
  const errors = missing.length > 0 ? [`版本发布提交缺少暂存文件: ${missing.join(', ')}`] : [];
  const unstaged = git(root, ['diff', '--name-only', '--', ...required])
    .split(/\r?\n/)
    .filter(Boolean);
  if (unstaged.length > 0) {
    errors.push(`发布文件仍有未暂存修改: ${unstaged.join(', ')}`);
  }
  return errors;
}

function projectRoot() {
  const scriptDir = path.dirname(fileURLToPath(import.meta.url));
  return path.resolve(scriptDir, '..');
}

function printCheck(result) {
  if (result.ok) {
    console.log(`发布检查通过: ${result.version}`);
    return;
  }
  for (const error of result.errors) console.error(`- ${error}`);
  process.exitCode = 1;
}

async function main() {
  const root = projectRoot();
  const [command = 'check', value] = process.argv.slice(2);
  if (command === 'check') {
    printCheck(validateRelease(root));
    return;
  }
  if (command === 'bump') {
    if (!value) throw new Error('用法: pnpm release:bump -- <版本号>');
    bumpVersion(root, value);
    console.log(`版本号已同步为 ${value}；请填写 CHANGELOG.md 后运行 pnpm release:check。`);
    return;
  }
  throw new Error(`未知命令: ${command}`);
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main().catch((error) => {
    console.error(error.message);
    process.exitCode = 1;
  });
}
