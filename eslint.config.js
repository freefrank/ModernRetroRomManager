import js from "@eslint/js";
import tseslint from "typescript-eslint";
import reactHooks from "eslint-plugin-react-hooks";
import reactRefresh from "eslint-plugin-react-refresh";
import globals from "globals";

export default tseslint.config(
  { ignores: ["dist", "src-tauri", "server", "node_modules", "scripts"] },
  {
    files: ["src/**/*.{ts,tsx}"],
    extends: [js.configs.recommended, ...tseslint.configs.recommended],
    languageOptions: { globals: globals.browser },
    plugins: { "react-hooks": reactHooks, "react-refresh": reactRefresh },
    rules: {
      ...reactHooks.configs.recommended.rules,
      "react-refresh/only-export-components": "off",
      "@typescript-eslint/no-unused-vars": ["error", { argsIgnorePattern: "^_" }],
      // eslint-plugin-react-hooks v7 引入了面向 React Compiler 的实验性规则，
      // 在存量代码里对多处“依赖 prop/异步结果重置或初始化状态”的 effect 写法报错，
      // 逐个改写涉及重构 effect 时序，存在行为回归风险，故在 Wave 0 阶段关闭。
      "react-hooks/set-state-in-effect": "off",
    },
  }
);
