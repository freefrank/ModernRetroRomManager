import i18n from "i18next";
import { initReactI18next } from "react-i18next";

import zhCNCommon from "./locales/zh-CN/common.json";
import zhCNLibrary from "./locales/zh-CN/library.json";
import zhCNScraper from "./locales/zh-CN/scraper.json";
import zhCNSettings from "./locales/zh-CN/settings.json";
import zhCNCnTools from "./locales/zh-CN/cnTools.json";
import zhCNImport from "./locales/zh-CN/import.json";

import enCommon from "./locales/en/common.json";
import enLibrary from "./locales/en/library.json";
import enScraper from "./locales/en/scraper.json";
import enSettings from "./locales/en/settings.json";
import enCnTools from "./locales/en/cnTools.json";
import enImport from "./locales/en/import.json";

type JsonObject = { [key: string]: unknown };

function isPlainObject(value: unknown): value is JsonObject {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

/** 深合并多个分文件为单个 translation 对象,后者覆盖前者的同名 key */
function deepMerge(...sources: JsonObject[]): JsonObject {
  const target: JsonObject = {};
  for (const source of sources) {
    for (const key of Object.keys(source)) {
      const value = source[key];
      if (isPlainObject(value) && isPlainObject(target[key])) {
        target[key] = deepMerge(target[key], value);
      } else {
        target[key] = value;
      }
    }
  }
  return target;
}

const zhCN = deepMerge(zhCNCommon, zhCNLibrary, zhCNScraper, zhCNSettings, zhCNCnTools, zhCNImport);
const en = deepMerge(enCommon, enLibrary, enScraper, enSettings, enCnTools, enImport);

export const resources = {
  "zh-CN": { translation: zhCN },
  en: { translation: en },
} as const;

export const languages = [
  { code: "zh-CN", name: "简体中文" },
  { code: "en", name: "English" },
] as const;

i18n.use(initReactI18next).init({
  resources,
  lng: "zh-CN", // 默认语言
  fallbackLng: "en",
  interpolation: {
    escapeValue: false,
  },
});

export default i18n;
