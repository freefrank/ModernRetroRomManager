import assert from 'node:assert/strict';
import { mkdtempSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { bumpVersion, extractReleaseNotes, getVersions, validateRelease } from './release-tools.mjs';

function fixture() {
  const root = mkdtempSync(path.join(tmpdir(), 'mrrm-release-'));
  mkdirSync(path.join(root, 'src-tauri'));
  writeFileSync(path.join(root, 'package.json'), '{\n  "version": "1.0.0",\n  "name": "fixture"\n}\n');
  writeFileSync(path.join(root, 'src-tauri/tauri.conf.json'), '{\n  "version": "1.0.0"\n}\n');
  writeFileSync(path.join(root, 'src-tauri/Cargo.toml'), '[package]\nname = "modern-retro-rom-manager"\nversion = "1.0.0"\n');
  writeFileSync(path.join(root, 'src-tauri/Cargo.lock'), '[[package]]\nname = "modern-retro-rom-manager"\nversion = "1.0.0"\n');
  writeFileSync(
    path.join(root, 'CHANGELOG.md'),
    '# Changelog / 更新日志\n\n## [Unreleased]\n\n## [1.0.0] - 2026-01-01\n\n### English\n\n#### Added\n\n- Initial release\n\n### 简体中文\n\n#### 新增\n\n- 初始版本\n',
  );
  return root;
}

test('检查四处版本号和最后一个 CHANGELOG 段落', () => {
  const root = fixture();
  assert.equal(validateRelease(root).ok, true);
  writeFileSync(path.join(root, 'src-tauri/tauri.conf.json'), '{\n  "version": "1.0.1"\n}\n');
  const result = validateRelease(root);
  assert.equal(result.ok, false);
  assert.match(result.errors.join('\n'), /tauri\.conf\.json 为 1\.0\.1/);
});

test('当前应用版本必须位于 CHANGELOG 顶部', () => {
  const root = fixture();
  const changelog = readFileSync(path.join(root, 'CHANGELOG.md'), 'utf8');
  writeFileSync(
    path.join(root, 'CHANGELOG.md'),
    changelog.replace(
      '## [1.0.0]',
      '## [1.1.0] - 2026-02-01\n\n### English\n\n- Update\n\n### 简体中文\n\n- 更新\n\n## [1.0.0]',
    ),
  );
  assert.match(validateRelease(root).errors.join('\n'), /顶部版本为 1\.1\.0，应为 1\.0\.0/);
});

test('bump 同步版本并生成必须填写的 CHANGELOG 骨架', () => {
  const root = fixture();
  bumpVersion(root, '1.1.0', '2026-02-03');
  assert.deepEqual(new Set(Object.values(getVersions(root))), new Set(['1.1.0']));
  const changelog = readFileSync(path.join(root, 'CHANGELOG.md'), 'utf8');
  assert.match(changelog, /^## \[1\.1\.0\] - 2026-02-03/m);
  assert.equal(validateRelease(root).ok, false);
  assert.match(validateRelease(root).errors.join('\n'), /待补充/);

  writeFileSync(
    path.join(root, 'CHANGELOG.md'),
    changelog.replace('To be documented', 'Completed release flow test').replace('待补充', '完成版本流程测试'),
  );
  assert.equal(validateRelease(root).ok, true);
});

test('只提取指定版本的双语 Release notes', () => {
  const root = fixture();
  const changelog = readFileSync(path.join(root, 'CHANGELOG.md'), 'utf8');
  const notes = extractReleaseNotes(changelog, '1.0.0');
  assert.match(notes, /### English/);
  assert.match(notes, /### 简体中文/);
  assert.doesNotMatch(notes, /Unreleased/);
});

test('拒绝无效版本号', () => {
  const root = fixture();
  assert.throws(() => bumpVersion(root, 'v1.2'), /无效的 SemVer/);
});
