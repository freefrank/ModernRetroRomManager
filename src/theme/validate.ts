import { TOKEN_KEYS, EFFECT_SLOTS, EFFECT_NAMES, type ThemeManifest, type TokenKey, type EffectSlot } from "./types";

export interface ValidateResult { ok: boolean; manifest?: ThemeManifest; warnings: string[]; error?: string; }

const FORBIDDEN = /url\s*\(\s*['"]?\s*(https?:|\/\/)|expression\s*\(|javascript:|@import/i;

export function validateManifest(raw: unknown): ValidateResult {
  const warnings: string[] = [];
  const m = raw as Record<string, unknown>;
  if (!m || typeof m !== "object") return { ok: false, warnings, error: "清单不是对象" };
  if (m.schemaVersion !== 1) return { ok: false, warnings, error: "不支持的 schemaVersion" };
  if (typeof m.id !== "string" || !/^[a-z0-9][a-z0-9-]*$/.test(m.id)) return { ok: false, warnings, error: "id 非法" };
  if (typeof m.name !== "string" || !m.name) return { ok: false, warnings, error: "缺少 name" };
  if (!m.tokens || typeof m.tokens !== "object") return { ok: false, warnings, error: "缺少 tokens" };

  const tokens: Partial<Record<TokenKey, string>> = {};
  for (const [k, v] of Object.entries(m.tokens as Record<string, unknown>)) {
    if (!(TOKEN_KEYS as readonly string[]).includes(k)) { warnings.push(`未知令牌 ${k} 已忽略`); continue; }
    if (typeof v !== "string") { warnings.push(`令牌 ${k} 值非字符串,已忽略`); continue; }
    if (FORBIDDEN.test(v)) return { ok: false, warnings, error: `令牌 ${k} 含被禁止的内容` };
    tokens[k as TokenKey] = v;
  }

  const effects: ThemeManifest["effects"] = {};
  if (m.effects && typeof m.effects === "object") {
    for (const [slot, cfg] of Object.entries(m.effects as Record<string, { name?: string; opacity?: number; color?: string }>)) {
      if (!(EFFECT_SLOTS as readonly string[]).includes(slot)) { warnings.push(`未知插槽 ${slot} 已忽略`); continue; }
      const name = (EFFECT_NAMES as readonly string[]).includes(cfg?.name ?? "") ? cfg!.name! : "none";
      if (name !== cfg?.name) warnings.push(`插槽 ${slot} 引用未知效果,已回退 none`);
      effects[slot as EffectSlot] = { name: name as never, opacity: cfg?.opacity, color: cfg?.color };
    }
  }
  return { ok: true, warnings, manifest: { schemaVersion: 1, id: m.id, name: m.name, author: typeof m.author === "string" ? m.author : undefined, tokens, effects } };
}
