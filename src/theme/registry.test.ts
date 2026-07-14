import { describe, expect, it } from "vitest";
import { BUILTIN_THEMES } from "./registry";

describe("内置主题", () => {
  it("Retro 主题的标题和正文优先使用 Zpix", () => {
    const retro = BUILTIN_THEMES.find((theme) => theme.id === "retro-arcade");

    expect(retro?.tokens["font-display"]).toContain("'Zpix'");
    expect(retro?.tokens["font-body"]).toContain("'Zpix'");
  });
});
