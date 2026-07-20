/** 字节大小格式化:B / KB / MB / GB。与 RomView 中的实现保持一致。 */
export function formatFileSize(bytes: number | undefined): string {
  if (bytes === undefined || bytes < 0) return "—";
  const KB = 1024;
  const MB = KB * 1024;
  const GB = MB * 1024;
  if (bytes >= GB) return `${(bytes / GB).toFixed(2)} GB`;
  if (bytes >= MB) return `${(bytes / MB).toFixed(1)} MB`;
  if (bytes >= KB) return `${Math.round(bytes / KB)} KB`;
  return `${bytes} B`;
}

/** 写入速度格式化:xx/s。 */
export function formatSpeed(bytesPerSecond: number | undefined): string {
  if (bytesPerSecond === undefined || bytesPerSecond < 0) return "—";
  return `${formatFileSize(bytesPerSecond)}/s`;
}

/** i18n 翻译函数的最小签名,避免依赖 react-i18next 的复杂泛型。 */
type Translate = (key: string, options?: Record<string, unknown>) => string;

/**
 * 剩余时长格式化:如「1小时2分」「5分30秒」「45秒」。
 * 时间单位文案走 i18n(common.json 的 duration 命名空间)。
 */
export function formatDuration(secs: number, t: Translate): string {
  const total = Math.max(0, Math.round(secs));
  const hours = Math.floor(total / 3600);
  const minutes = Math.floor((total % 3600) / 60);
  const seconds = total % 60;

  let text: string;
  if (hours > 0) {
    text = t("duration.hour", { count: hours }) + t("duration.minute", { count: minutes });
  } else if (minutes > 0) {
    text = t("duration.minute", { count: minutes }) + t("duration.second", { count: seconds });
  } else {
    text = t("duration.second", { count: seconds });
  }
  return text.trim();
}
