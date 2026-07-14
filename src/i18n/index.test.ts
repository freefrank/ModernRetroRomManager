import { describe, expect, it } from "vitest";
import { languages, resources } from "./index";

function flatten(value: unknown, prefix = "", output: Record<string, string> = {}) {
  if (!value || typeof value !== "object" || Array.isArray(value)) return output;
  for (const [key, item] of Object.entries(value)) {
    const fullKey = prefix ? `${prefix}.${key}` : key;
    if (item && typeof item === "object" && !Array.isArray(item)) flatten(item, fullKey, output);
    else output[fullKey] = String(item);
  }
  return output;
}

function placeholders(value: string) {
  return [...value.matchAll(/\{\{[^}]+\}\}/g)].map((match) => match[0]).sort();
}

describe("i18n resources", () => {
  it("语言选择器包含全部已注册资源", () => {
    expect(languages.map(({ code }) => code).sort()).toEqual(Object.keys(resources).sort());
  });

  it("所有语言与英文拥有相同键和插值占位符", () => {
    const english = flatten(resources.en.translation);
    const englishKeys = Object.keys(english).sort();

    for (const [language, resource] of Object.entries(resources)) {
      const localized = flatten(resource.translation);
      expect(Object.keys(localized).sort(), `${language} translation keys`).toEqual(englishKeys);
      for (const key of englishKeys) {
        expect(placeholders(localized[key]), `${language}:${key}`).toEqual(placeholders(english[key]));
      }
    }
  });
});
