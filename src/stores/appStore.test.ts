import { describe, it, expect } from "vitest";
import i18n from "@/i18n";
import {
  applyInterfaceLanguage,
  isSafeAssetPath,
  normalizeInterfaceLanguage,
  sanitizeCustomCss,
} from "./appStore";

const mockConvert = (p: string) => `converted://${p}`;
const DIR = "C:/config/themes/demo";

describe("isSafeAssetPath", () => {
  it("接受普通相对路径", () => {
    expect(isSafeAssetPath("bg.png")).toBe(true);
    expect(isSafeAssetPath("img/tile.png")).toBe(true);
  });
  it("拒绝路径穿越与绝对路径", () => {
    expect(isSafeAssetPath("../evil.png")).toBe(false);
    expect(isSafeAssetPath("a/../b.png")).toBe(false);
    expect(isSafeAssetPath("/etc/passwd")).toBe(false);
    expect(isSafeAssetPath("./a.png")).toBe(false);
    expect(isSafeAssetPath("a//b.png")).toBe(false);
    expect(isSafeAssetPath("")).toBe(false);
  });
  it("拒绝反斜杠与冒号(Windows 路径/协议向量)", () => {
    expect(isSafeAssetPath("a\\b.png")).toBe(false);
    expect(isSafeAssetPath("c:/x.png")).toBe(false);
    expect(isSafeAssetPath("javascript:alert(1)")).toBe(false);
  });
});

describe("sanitizeCustomCss", () => {
  it("合法占位符替换为 convertFileSrc 结果", () => {
    const css = ".rr-card{background:url(__RR_ASSET__/bg.png)}";
    expect(sanitizeCustomCss(css, DIR, mockConvert)).toBe(
      `.rr-card{background:url("converted://${DIR}/assets/bg.png")}`,
    );
  });
  it("支持引号包裹的占位符", () => {
    const css = `.a{background:url('__RR_ASSET__/img/tile.png')}`;
    expect(sanitizeCustomCss(css, DIR, mockConvert)).toBe(
      `.a{background:url("converted://${DIR}/assets/img/tile.png")}`,
    );
  });
  it("非法路径的 url 直接置空", () => {
    expect(sanitizeCustomCss(".a{background:url(__RR_ASSET__/../evil.png)}", DIR, mockConvert)).toBe(
      ".a{background:url()}",
    );
    expect(sanitizeCustomCss(".a{background:url(__RR_ASSET__//abs.png)}", DIR, mockConvert)).toBe(
      ".a{background:url()}",
    );
  });
  it("剥离残留 @import(含 EOF 不带分号的逃逸)", () => {
    expect(sanitizeCustomCss('@import url(http://x.com/a.css); .a{color:var(--text-primary)}', DIR, mockConvert)).toBe(
      " .a{color:var(--text-primary)}",
    );
    expect(sanitizeCustomCss('.a{}\n@import "http://x.com/a.css"', DIR, mockConvert)).toBe(".a{}\n");
  });
});

describe("interface language", () => {
  it("兼容旧中文值并拒绝未知语言", () => {
    expect(normalizeInterfaceLanguage("zh")).toBe("zh-CN");
    expect(normalizeInterfaceLanguage("zh-CN")).toBe("zh-CN");
    expect(normalizeInterfaceLanguage("unknown")).toBe("zh-CN");
    expect(normalizeInterfaceLanguage("en")).toBe("en");
    expect(normalizeInterfaceLanguage("fr")).toBe("fr");
    expect(normalizeInterfaceLanguage("de")).toBe("de");
    expect(normalizeInterfaceLanguage("it")).toBe("it");
    expect(normalizeInterfaceLanguage("es")).toBe("es");
    expect(normalizeInterfaceLanguage("ru")).toBe("ru");
    expect(normalizeInterfaceLanguage("zh-TW")).toBe("zh-TW");
  });

  it("初始化时把保存语言应用到 i18next", async () => {
    await applyInterfaceLanguage("en");
    expect(i18n.resolvedLanguage).toBe("en");
    await applyInterfaceLanguage("zh");
    expect(i18n.resolvedLanguage).toBe("zh-CN");
  });
});
