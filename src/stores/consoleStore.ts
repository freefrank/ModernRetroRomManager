import { create } from "zustand";

export type ConsoleLevel = "debug" | "info" | "warn" | "error";

export interface ConsoleEntry {
  id: number;
  timestamp: number;
  level: ConsoleLevel;
  source: string;
  message: string;
}

interface ConsoleState {
  entries: ConsoleEntry[];
  expanded: boolean;
  addEntry: (level: ConsoleLevel, message: string, source?: string) => void;
  /** 高频进度事件:若末尾已是同源进度条目则就地更新,否则新增一条,避免刷屏。 */
  upsertProgress: (level: ConsoleLevel, message: string, source: string) => void;
  clear: () => void;
  toggle: () => void;
}

const MAX_ENTRIES = 500;
let nextId = 1;

export const useConsoleStore = create<ConsoleState>((set) => ({
  entries: [],
  expanded: false,
  addEntry: (level, message, source = "app") =>
    set((state) => ({
      entries: [
        ...state.entries,
        { id: nextId++, timestamp: Date.now(), level, source, message },
      ].slice(-MAX_ENTRIES),
    })),
  upsertProgress: (level, message, source) =>
    set((state) => {
      const last = state.entries[state.entries.length - 1];
      if (last && last.source === source) {
        // 同源进度就地覆盖最后一条(行内动态更新)
        const entries = state.entries.slice();
        entries[entries.length - 1] = { ...last, level, message, timestamp: Date.now() };
        return { entries };
      }
      return {
        entries: [
          ...state.entries,
          { id: nextId++, timestamp: Date.now(), level, source, message },
        ].slice(-MAX_ENTRIES),
      };
    }),
  clear: () => set({ entries: [] }),
  toggle: () => set((state) => ({ expanded: !state.expanded })),
}));

function formatConsoleValue(value: unknown): string {
  if (value instanceof Error) return value.stack || value.message;
  if (typeof value === "string") return value;
  try {
    return JSON.stringify(value);
  } catch {
    return String(value);
  }
}

declare global {
  interface Window {
    __mrrmConsoleCaptureInstalled?: boolean;
  }
}

/** 捕获应用已有 console 输出，同时保留 WebView 开发者工具中的原始输出。 */
export function installConsoleCapture(): void {
  if (typeof window === "undefined" || window.__mrrmConsoleCaptureInstalled) return;
  window.__mrrmConsoleCaptureInstalled = true;

  const methods: Array<["debug" | "log" | "info" | "warn" | "error", ConsoleLevel]> = [
    ["debug", "debug"],
    ["log", "info"],
    ["info", "info"],
    ["warn", "warn"],
    ["error", "error"],
  ];
  for (const [method, level] of methods) {
    const original = console[method].bind(console);
    console[method] = (...values: unknown[]) => {
      original(...values);
      useConsoleStore.getState().addEntry(level, values.map(formatConsoleValue).join(" "), "frontend");
    };
  }
}

export function appendConsoleEntry(
  level: ConsoleLevel,
  message: string,
  source = "app",
): void {
  useConsoleStore.getState().addEntry(level, message, source);
}
