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

import frCommon from "./locales/fr/common.json";
import frLibrary from "./locales/fr/library.json";
import frScraper from "./locales/fr/scraper.json";
import frSettings from "./locales/fr/settings.json";
import frCnTools from "./locales/fr/cnTools.json";
import frImport from "./locales/fr/import.json";

import deCommon from "./locales/de/common.json";
import deLibrary from "./locales/de/library.json";
import deScraper from "./locales/de/scraper.json";
import deSettings from "./locales/de/settings.json";
import deCnTools from "./locales/de/cnTools.json";
import deImport from "./locales/de/import.json";

import itCommon from "./locales/it/common.json";
import itLibrary from "./locales/it/library.json";
import itScraper from "./locales/it/scraper.json";
import itSettings from "./locales/it/settings.json";
import itCnTools from "./locales/it/cnTools.json";
import itImport from "./locales/it/import.json";

import esCommon from "./locales/es/common.json";
import esLibrary from "./locales/es/library.json";
import esScraper from "./locales/es/scraper.json";
import esSettings from "./locales/es/settings.json";
import esCnTools from "./locales/es/cnTools.json";
import esImport from "./locales/es/import.json";

import ruCommon from "./locales/ru/common.json";
import ruLibrary from "./locales/ru/library.json";
import ruScraper from "./locales/ru/scraper.json";
import ruSettings from "./locales/ru/settings.json";
import ruCnTools from "./locales/ru/cnTools.json";
import ruImport from "./locales/ru/import.json";

import zhTWCommon from "./locales/zh-TW/common.json";
import zhTWLibrary from "./locales/zh-TW/library.json";
import zhTWScraper from "./locales/zh-TW/scraper.json";
import zhTWSettings from "./locales/zh-TW/settings.json";
import zhTWCnTools from "./locales/zh-TW/cnTools.json";
import zhTWImport from "./locales/zh-TW/import.json";

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
const fr = deepMerge(frCommon, frLibrary, frScraper, frSettings, frCnTools, frImport);
const de = deepMerge(deCommon, deLibrary, deScraper, deSettings, deCnTools, deImport);
const it = deepMerge(itCommon, itLibrary, itScraper, itSettings, itCnTools, itImport);
const es = deepMerge(esCommon, esLibrary, esScraper, esSettings, esCnTools, esImport);
const ru = deepMerge(ruCommon, ruLibrary, ruScraper, ruSettings, ruCnTools, ruImport);
const zhTW = deepMerge(zhTWCommon, zhTWLibrary, zhTWScraper, zhTWSettings, zhTWCnTools, zhTWImport);

export const resources = {
  "zh-CN": { translation: zhCN },
  "zh-TW": { translation: zhTW },
  en: { translation: en },
  fr: { translation: fr },
  de: { translation: de },
  it: { translation: it },
  es: { translation: es },
  ru: { translation: ru },
} as const;

export type LanguageCode = keyof typeof resources;

export function isLanguageCode(value: string): value is LanguageCode {
  return Object.prototype.hasOwnProperty.call(resources, value);
}

export const languages = [
  { code: "zh-CN", name: "简体中文" },
  { code: "zh-TW", name: "繁體中文" },
  { code: "en", name: "English" },
  { code: "fr", name: "Français" },
  { code: "de", name: "Deutsch" },
  { code: "it", name: "Italiano" },
  { code: "es", name: "Español" },
  { code: "ru", name: "Русский" },
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
