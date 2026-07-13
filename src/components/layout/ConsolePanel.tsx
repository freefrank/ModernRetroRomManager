import { useEffect, useRef } from "react";
import { listen } from "@tauri-apps/api/event";
import { ChevronDown, ChevronUp, CircleAlert, Terminal, Trash2 } from "lucide-react";
import { clsx } from "clsx";
import { useTranslation } from "react-i18next";
import { useConsoleStore, type ConsoleLevel } from "@/stores/consoleStore";

interface ProgressPayload {
  current: number;
  total: number;
  message: string;
  finished: boolean;
  cancelled?: boolean;
}

interface LogPayload {
  level?: ConsoleLevel;
  source?: string;
  message: string;
}

const levelClass: Record<ConsoleLevel, string> = {
  info: "text-text-secondary",
  success: "text-accent-success",
  warning: "text-accent-warning",
  error: "text-accent-error",
};

export default function ConsolePanel() {
  const { t } = useTranslation();
  const { entries, expanded, addEntry, clear, toggle } = useConsoleStore();
  const scrollRef = useRef<HTMLDivElement>(null);
  const latest = entries[entries.length - 1];
  const errorCount = entries.filter((entry) => entry.level === "error").length;

  useEffect(() => {
    const cleanups = [
      listen<ProgressPayload>("batch-scrape-progress", ({ payload }) => {
        const level = payload.finished
          ? payload.cancelled ? "warning" : "success"
          : "info";
        addEntry(level, `${payload.message} (${payload.current}/${payload.total})`, "scraper");
      }),
      listen<ProgressPayload>("export-progress", ({ payload }) => {
        addEntry(payload.finished ? "success" : "info", payload.message, "export");
      }),
      listen<LogPayload>("app-log", ({ payload }) => {
        addEntry(payload.level || "info", payload.message, payload.source || "backend");
      }),
    ];
    return () => {
      for (const cleanup of cleanups) cleanup.then((unlisten) => unlisten());
    };
  }, [addEntry]);

  useEffect(() => {
    if (expanded) scrollRef.current?.scrollTo({ top: scrollRef.current.scrollHeight });
  }, [entries, expanded]);

  return (
    <section
      aria-label={t("console.title")}
      className="relative z-[1000] shrink-0 border-t-[length:var(--border-width)] border-border-highlight bg-bg-primary [box-shadow:0_-4px_20px_rgba(0,0,0,0.35)]"
    >
      <div className="h-8 flex items-center gap-2 px-3 font-mono text-[11px]">
        <button
          type="button"
          onClick={toggle}
          className="flex min-w-0 flex-1 items-center gap-2 text-left hover:text-accent-primary"
          aria-expanded={expanded}
        >
          <Terminal className="h-3.5 w-3.5 shrink-0 text-accent-primary" />
          <span className="shrink-0 font-bold uppercase tracking-wider">{t("console.title")}</span>
          {errorCount > 0 && (
            <span className="flex shrink-0 items-center gap-1 text-accent-error">
              <CircleAlert className="h-3 w-3" />{errorCount}
            </span>
          )}
          {!expanded && latest && (
            <span className={clsx("min-w-0 truncate", levelClass[latest.level])}>
              [{latest.source}] {latest.message}
            </span>
          )}
          <span className="ml-auto shrink-0 text-text-muted">
            {expanded ? <ChevronDown className="h-3.5 w-3.5" /> : <ChevronUp className="h-3.5 w-3.5" />}
          </span>
        </button>
        <button
          type="button"
          onClick={clear}
          className="text-text-muted hover:text-accent-error"
          title={t("console.clear")}
          aria-label={t("console.clear")}
        >
          <Trash2 className="h-3.5 w-3.5" />
        </button>
      </div>

      {expanded && (
        <div ref={scrollRef} className="h-[32vh] overflow-y-auto border-t border-border-default bg-black/25 px-3 py-2 font-mono text-[11px] custom-scrollbar">
          {entries.length === 0 ? (
            <p className="text-text-muted">{t("console.empty")}</p>
          ) : entries.map((entry) => (
            <div key={entry.id} className="grid grid-cols-[5rem_6rem_1fr] gap-2 py-0.5 leading-5">
              <time className="text-text-muted">
                {new Date(entry.timestamp).toLocaleTimeString([], { hour12: false })}
              </time>
              <span className={clsx("truncate", levelClass[entry.level])}>[{entry.source}]</span>
              <span className={clsx("whitespace-pre-wrap break-words", levelClass[entry.level])}>{entry.message}</span>
            </div>
          ))}
        </div>
      )}
    </section>
  );
}
