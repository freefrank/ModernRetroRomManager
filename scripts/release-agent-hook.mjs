import { execFileSync } from 'node:child_process';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { stagedReleaseErrors, validateRelease } from './release-tools.mjs';

const RELEASE_PROMPT = `本次请求涉及版本、构建或发布。遵循项目发布流程：
1. 普通功能提交不要改版本号；准备发布时用 pnpm release:bump -- <版本号> 同步全部版本文件。
2. 根据该版本包含的中文 commit 更新 CHANGELOG.md，删除“待补充”，并检查 README/README.zh-CN/相关 docs 是否需要同步。
3. 依次运行测试、pnpm release:check、Windows 单文件 portable 与 installer 构建；CHANGELOG 每个版本必须包含英文和简体中文；仅在用户明确要求时 push 或打 tag。
4. 详细流程见 docs/release-process.md。`;

const DOCUMENTATION_PROMPT = `本次请求涉及项目文档。README.md 默认使用英文，并链接独立的简体中文 README.zh-CN.md；两份 README 的功能、依赖和命令保持一致。普通文档修改不 bump 版本号。用户可见行为发生变化时同步相关 docs；CHANGELOG 只在准备发布时更新。`;

function readStdin() {
  return new Promise((resolve, reject) => {
    let input = '';
    process.stdin.setEncoding('utf8');
    process.stdin.on('data', (chunk) => (input += chunk));
    process.stdin.on('end', () => {
      try {
        const normalized = input.trim().replace(/^\uFEFF/, '');
        resolve(normalized ? JSON.parse(normalized) : {});
      } catch (error) {
        reject(new Error(`无法解析 hook 输入: ${error.message}`));
      }
    });
    process.stdin.on('error', reject);
  });
}

function output(value) {
  process.stdout.write(`${JSON.stringify(value)}\n`);
}

function deny(reason) {
  output({
    hookSpecificOutput: {
      hookEventName: 'PreToolUse',
      permissionDecision: 'deny',
      permissionDecisionReason: reason,
      additionalContext: '先修正发布文件，再重新执行原命令。',
    },
  });
}

function rootFrom(input) {
  try {
    return execFileSync('git', ['rev-parse', '--show-toplevel'], {
      cwd: input.cwd || process.cwd(),
      encoding: 'utf8',
    }).trim();
  } catch {
    return path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
  }
}

function commandKind(command) {
  const normalized = String(command ?? '').replace(/\s+/g, ' ');
  const match = normalized.match(/\bgit(?:\.exe)?\b[\s\S]*?\b(commit|push|tag)\b/i);
  return match?.[1]?.toLowerCase();
}

function promptHook(input) {
  const prompt = input.prompt ?? '';
  const releaseRelated = /(发布|版本|构建|打包|artifact|release|version|build|changelog)/i.test(prompt);
  const documentationRelated = /(readme|文档|documentation|docs)/i.test(prompt);
  if (!releaseRelated && !documentationRelated) return;
  output({
    hookSpecificOutput: {
      hookEventName: 'UserPromptSubmit',
      additionalContext: [releaseRelated && RELEASE_PROMPT, documentationRelated && DOCUMENTATION_PROMPT]
        .filter(Boolean)
        .join('\n\n'),
    },
  });
}

function preToolHook(input) {
  if (input.tool_name !== 'Bash') return;
  const kind = commandKind(input.tool_input?.command);
  if (!kind) return;

  const root = rootFrom(input);
  const errors = kind === 'commit' ? stagedReleaseErrors(root) : [];
  if (kind === 'push' || kind === 'tag' || errors.length > 0) {
    const result = validateRelease(root);
    errors.push(...result.errors);
  }
  if (errors.length > 0) deny(`发布流程检查失败：\n- ${[...new Set(errors)].join('\n- ')}`);
}

const input = await readStdin();
const mode = process.argv[2];
if (mode === 'prompt') promptHook(input);
else if (mode === 'pre-tool') preToolHook(input);
else throw new Error(`未知 hook 模式: ${mode}`);
