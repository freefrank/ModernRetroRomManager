import { create } from "zustand";

export type ConsoleLevel = "info" | "success" | "warning" | "error";

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

  const methods: Array<["log" | "info" | "warn" | "error", ConsoleLevel]> = [
    ["log", "info"],
    ["info", "info"],
    ["warn", "warning"],
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
