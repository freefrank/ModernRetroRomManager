import { describe, it, expect } from "vitest";
import { validateManifest } from "./validate";

const ok = { schemaVersion: 1, id: "t", name: "T", tokens: { "bg-primary": "#000" } };

describe("validateManifest", () => {
  it("接受最小合法清单", () => {
    expect(validateManifest(ok).ok).toBe(true);
  });
  it("拒绝错误 schemaVersion / 缺 id / 缺 tokens", () => {
    expect(validateManifest({ ...ok, schemaVersion: 2 }).ok).toBe(false);
    expect(validateManifest({ ...ok, id: "" }).ok).toBe(false);
    expect(validateManifest({ ...ok, tokens: undefined }).ok).toBe(false);
  });
  it("剔除未知令牌键并告警,未知效果名回退 none", () => {
    const r = validateManifest({ ...ok, tokens: { "bg-primary": "#000", evil: "x" },
      effects: { cardHover: { name: "not-exist" } } });
    expect(r.ok).toBe(true);
    expect(r.manifest!.tokens).not.toHaveProperty("evil");
    expect(r.manifest!.effects!.cardHover!.name).toBe("none");
    expect(r.warnings.length).toBeGreaterThan(0);
  });
  it("拒绝令牌值中的外部 url 与脚本向量", () => {
    expect(validateManifest({ ...ok, tokens: { "bg-primary": "url(http://x)" } }).ok).toBe(false);
    expect(validateManifest({ ...ok, tokens: { "bg-primary": "expression(a)" } }).ok).toBe(false);
  });
});
