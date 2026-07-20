import { useEffect, useMemo, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { ChevronDown, ChevronUp, CircleAlert, Terminal, Trash2 } from "lucide-react";
import { clsx } from "clsx";
import { useTranslation } from "react-i18next";
import { useConsoleStore, type ConsoleLevel } from "@/stores/consoleStore";
import { formatDuration, formatFileSize, formatSpeed } from "@/lib/format";

interface ProgressPayload {
  current: number;
  total: number;
  message: string;
  finished: boolean;
  cancelled?: boolean;
}

interface FileProgressPayload {
  file: string;
  file_done: number;
  file_total: number;
  written: number;
  pending: number;
  speed_bytes: number;
  eta_secs: number;
}

interface LogPayload {
  level?: ConsoleLevel;
  source?: string;
  message: string;
}

interface RomScanPayload extends ProgressPayload {
  system?: string;
  mode: "incremental" | "full";
  changed: boolean;
}

const levelClass: Record<ConsoleLevel, string> = {
  debug: "text-text-muted",
  info: "text-text-secondary",
  warn: "text-accent-warning",
  error: "text-accent-error",
};

const levels: ConsoleLevel[] = ["debug", "info", "warn", "error"];

export default function ConsolePanel() {
  const { t } = useTranslation();
  const { entries, expanded, addEntry, upsertProgress, clear, toggle } = useConsoleStore();
  const [visibleLevels, setVisibleLevels] = useState<Set<ConsoleLevel>>(
    () => new Set(levels),
  );
  const [fileProgress, setFileProgress] = useState<FileProgressPayload | null>(null);
  const scrollRef = useRef<HTMLDivElement>(null);
  const visibleEntries = useMemo(
    () => entries.filter((entry) => visibleLevels.has(entry.level)),
    [entries, visibleLevels],
  );
  const latest = visibleEntries[visibleEntries.length - 1];
  const errorCount = entries.filter((entry) => entry.level === "error").length;

  useEffect(() => {
    const cleanups = [
      listen<ProgressPayload>("batch-scrape-progress", ({ payload }) => {
        const level = payload.finished
          ? payload.cancelled ? "warn" : "info"
          : "info";
        const message = `${payload.message} (${payload.current}/${payload.total})`;
        // 进行中就地更新,完成时留存一条独立记录
        if (payload.finished) addEntry(level, message, "scraper");
        else upsertProgress(level, message, "scraper");
      }),
      listen<ProgressPayload>("export-progress", ({ payload }) => {
        if (payload.finished) {
          addEntry("info", payload.message, "export");
          setFileProgress(null);
        } else {
          upsertProgress("info", payload.message, "export");
        }
      }),
      listen<FileProgressPayload>("export-file-progress", ({ payload }) => {
        setFileProgress(payload);
      }),
      listen<LogPayload>("app-log", ({ payload }) => {
        addEntry(payload.level || "info", payload.message, payload.source || "backend");
      }),
      listen<RomScanPayload>("rom-scan-progress", ({ payload }) => {
        const level: ConsoleLevel = payload.finished ? "info" : payload.changed ? "info" : "debug";
        const message = `${payload.message}${payload.total > 0 ? ` (${payload.current}/${payload.total})` : ""}`;
        if (payload.finished) addEntry(level, message, `rom-${payload.mode}`);
        else upsertProgress(level, message, `rom-${payload.mode}`);
      }),
    ];
    return () => {
      for (const cleanup of cleanups) cleanup.then((unlisten) => unlisten());
    };
  }, [addEntry, upsertProgress]);

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
        <span className="shrink-0 border-l border-border-default pl-2 text-text-muted">
          v{APP_VERSION}
        </span>
      </div>

      {expanded && (
        <div className="border-t border-border-default bg-black/25 font-mono text-[11px]">
          {fileProgress && (
            <div className="rr-console-file-progress border-b border-border-default px-3 py-1.5">
              <div className="mb-1 flex items-center justify-between gap-2">
                <span className="min-w-0 flex-1 truncate text-text-secondary">
                  {t("console.fileProgress.copying")} {fileProgress.file}
                </span>
                <span className="shrink-0 tabular-nums text-text-muted">
                  {formatFileSize(fileProgress.written)} / {formatFileSize(fileProgress.pending)}
                </span>
              </div>
              <div className="h-1.5 overflow-hidden rounded-[var(--radius-sm)] bg-bg-tertiary">
                <div
                  className="h-full bg-accent-primary transition-[width] duration-[var(--motion-fast)] ease-[var(--motion-easing)]"
                  style={{
                    width: `${
                      fileProgress.file_total > 0
                        ? Math.min(100, (fileProgress.file_done / fileProgress.file_total) * 100)
                        : 0
                    }%`,
                  }}
                />
              </div>
              <div className="mt-1 flex items-center justify-between gap-2 tabular-nums text-text-muted">
                <span className="truncate">{formatSpeed(fileProgress.speed_bytes)}</span>
                <span className="shrink-0">
                  {t("console.fileProgress.remaining")}{" "}
                  {fileProgress.eta_secs > 0
                    ? formatDuration(fileProgress.eta_secs, t)
                    : t("console.fileProgress.calculating")}
                </span>
              </div>
            </div>
          )}
          <div className="flex items-center gap-1 border-b border-border-default px-3 py-1">
            {levels.map((level) => {
              const active = visibleLevels.has(level);
              const count = entries.filter((entry) => entry.level === level).length;
              return (
                <button
                  key={level}
                  type="button"
                  aria-pressed={active}
                  onClick={() => setVisibleLevels((current) => {
                    const next = new Set(current);
                    if (next.has(level)) next.delete(level);
                    else next.add(level);
                    return next;
                  })}
                  className={clsx(
                    "rounded border px-1.5 py-0.5 uppercase transition-opacity",
                    levelClass[level],
                    active ? "border-current opacity-100" : "border-border-default opacity-40",
                  )}
                >
                  {level} {count}
                </button>
              );
            })}
          </div>
          <div ref={scrollRef} className="select-text h-[calc(24vh-2rem)] overflow-y-auto px-3 py-2 custom-scrollbar">
          {visibleEntries.length === 0 ? (
            <p className="text-text-muted">{t("console.empty")}</p>
          ) : visibleEntries.map((entry) => (
            <div key={entry.id} className="grid grid-cols-[5rem_3.5rem_6rem_1fr] gap-2 py-0.5 leading-5">
              <time className="text-text-muted">
                {new Date(entry.timestamp).toLocaleTimeString([], { hour12: false })}
              </time>
              <span className={clsx("font-bold uppercase", levelClass[entry.level])}>{entry.level}</span>
              <span className={clsx("truncate", levelClass[entry.level])}>[{entry.source}]</span>
              <span className={clsx("whitespace-pre-wrap break-words", levelClass[entry.level])}>{entry.message}</span>
            </div>
          ))}
          </div>
        </div>
      )}
    </section>
  );
}
