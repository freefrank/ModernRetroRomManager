import { describe, expect, it } from "vitest";
import { readFileSync, readdirSync } from "node:fs";
import { join, relative } from "node:path";

function sourceFiles(root: string): string[] {
  return readdirSync(root, { withFileTypes: true }).flatMap(entry => {
    const path = join(root, entry.name);
    if (entry.isDirectory()) return sourceFiles(path);
    return /\.(ts|tsx)$/.test(entry.name) && !entry.name.includes(".test.") ? [path] : [];
  });
}

describe("frontend/backend wiring", () => {
  it("registers every literal Tauri command invoked by the frontend", () => {
    const projectRoot = process.cwd();
    const backend = readFileSync(join(projectRoot, "src-tauri", "src", "lib.rs"), "utf8");
    const registered = new Set(
      [...backend.matchAll(/commands::[A-Za-z0-9_]+::([A-Za-z0-9_]+)/g)]
        .map(match => match[1]),
    );
    const invoked = sourceFiles(join(projectRoot, "src")).flatMap(file => {
      const source = readFileSync(file, "utf8");
      return [...source.matchAll(/(?:tauriInvoke(?:<[^>]+>)?|invoke(?:<[^>]+>)?)\(\s*["']([^"']+)["']/g)]
        .map(match => match[1]);
    });
    expect([...new Set(invoked)].filter(command => !registered.has(command))).toEqual([]);
  });

  it("does not render Button or IconButton controls without an action", () => {
    const projectRoot = process.cwd();
    const unbound: string[] = [];
    for (const file of sourceFiles(join(projectRoot, "src"))) {
      if (file.endsWith(join("ui", "Button.tsx"))) continue;
      const source = readFileSync(file, "utf8");
      for (const match of source.matchAll(/<(?:Button|IconButton)\b([\s\S]*?)>/g)) {
        const props = match[1];
        if (!/\bonClick\s*=/.test(props) && !/\btype\s*=\s*["']submit["']/.test(props)) {
          unbound.push(relative(projectRoot, file));
        }
      }
    }
    expect(unbound).toEqual([]);
  });
});
