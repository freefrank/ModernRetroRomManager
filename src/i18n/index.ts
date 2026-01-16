import i18n from "i18next";
import { initReactI18next } from "react-i18next";

import zhCN from "./locales/zh-CN.json";
import en from "./locales/en.json";

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
