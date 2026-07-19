import { useState, useEffect, useRef, useMemo, useCallback } from "react";
import { Gamepad2, Star, CheckCircle2 } from "lucide-react";
import { useVirtualizer } from "@tanstack/react-virtual";
import { useTranslation } from "react-i18next";
import { resolveMediaUrlAsync, getCachedMediaUrl } from "@/lib/api";
import type { Rom, ViewMode } from "@/types";
import { clsx } from "clsx";
import { getRomDisplayName } from "@/lib/romName";

interface RomViewProps {
  roms: Rom[];
  viewMode: ViewMode;
  /** 卡片尺寸缩放系数(>1 卡片更大、列数更少),仅 cover/grid 生效 */
  cardScale?: number;
  /** true 时展示原始文件名,否则展示游戏名(中文名/标题) */
  showFileName?: boolean;
  selectedIds: Set<string>;
  onRomClick: (rom: Rom) => void;
  onToggleSelect: (id: string) => void;
  /** 判断某 ROM 是否被隐藏(隐藏项灰显) */
  isRomHidden?: (rom: Rom) => boolean;
  /** 右键卡片时打开自定义菜单 */
  onRomContextMenu?: (rom: Rom, event: React.MouseEvent) => void;
}

// 获取 ROM 封面路径，优先使用 temp_data
function getRomCover(rom: Rom): string | undefined {
  return rom.temp_data?.box_front || rom.box_front || rom.gridicon;
}

// 横向的 ES <image> 往往是 mix/screenshot，而不是竖版 boxart。
// 这类回退图片应完整显示，避免被 3:4 卡片从中间严重裁切。
function shouldContainCover(image: HTMLImageElement): boolean {
  return image.naturalWidth > image.naturalHeight * 1.05;
}

// 文件大小自适应格式化(KB/MB/GB)
function formatFileSize(bytes: number | undefined): string {
  if (bytes === undefined || bytes < 0) return "—";
  const KB = 1024;
  const MB = KB * 1024;
  const GB = MB * 1024;
  if (bytes >= GB) return `${(bytes / GB).toFixed(2)} GB`;
  if (bytes >= MB) return `${(bytes / MB).toFixed(1)} MB`;
  if (bytes >= KB) return `${Math.round(bytes / KB)} KB`;
  return `${bytes} B`;
}

// unix 秒 -> YYYY-MM-DD
function formatModifiedDate(unixSeconds: number | undefined): string {
  if (unixSeconds === undefined || unixSeconds <= 0) return "—";
  const d = new Date(unixSeconds * 1000);
  const year = d.getFullYear();
  const month = String(d.getMonth() + 1).padStart(2, "0");
  const day = String(d.getDate()).padStart(2, "0");
  return `${year}-${month}-${day}`;
}

// Shared hook for async media URL resolution with cache support
function useMediaUrl(path: string | undefined): string | null {
  // Check cache first for instant display
  const cachedUrl = path ? getCachedMediaUrl(path) : null;
  const [url, setUrl] = useState<string | null>(cachedUrl);

  useEffect(() => {
    let isMounted = true;
    if (!path) {
      setUrl(null);
      return;
    }
    // If already cached, use it
    const cached = getCachedMediaUrl(path);
    if (cached) {
      setUrl(cached);
      return;
    }
    // Otherwise fetch async
    resolveMediaUrlAsync(path).then((resolvedUrl) => {
      if (isMounted) {
        setUrl(resolvedUrl);
      }
    });

    return () => {
      isMounted = false;
    };
  }, [path]);

  return url;
}

// View mode configuration
const VIEW_CONFIG = {
  cover: {
    getColumns: (width: number) => {
      if (width < 640) return 3;
      if (width < 768) return 4;
      if (width < 1024) return 5;
      if (width < 1280) return 6;
      if (width < 1536) return 8;
      return 10;
    },
    // Dynamic row height based on card width (aspect 3:4)
    getRowHeight: (containerWidth: number, columns: number, gap: number) => {
      const totalGap = gap * (columns - 1);
      const cardWidth = (containerWidth - totalGap) / columns;
      return Math.ceil(cardWidth * (4 / 3));
    },
    gap: 12,
    overscan: 3,
  },
  grid: {
    getColumns: (width: number) => {
      if (width < 768) return 2;
      if (width < 1024) return 3;
      if (width < 1280) return 4;
      if (width < 1536) return 5;
      return 6;
    },
    // Dynamic row height: aspect 3:4 image + text area
    getRowHeight: (containerWidth: number, columns: number, gap: number) => {
      const totalGap = gap * (columns - 1);
      const cardWidth = (containerWidth - totalGap) / columns;
      const imageHeight = cardWidth * (4 / 3);
      const textAreaHeight = 80; // padding + title + optional rating + border safety
      return Math.ceil(imageHeight + textAreaHeight);
    },
    gap: 24,
    overscan: 2,
  },
  list: {
    getColumns: () => 1,
    getRowHeight: () => 72,
    gap: 0,
    overscan: 5,
  },
} as const;

// 按缩放系数调整列数:系数越大卡片越大、列数越少(列表模式不缩放)
function applyCardScale(base: number, viewMode: ViewMode, cardScale: number) {
  if (viewMode === "list") return base;
  return Math.max(1, Math.round(base / cardScale));
}

// ============ Card Components ============

interface CardProps {
  rom: Rom;
  isSelected: boolean;
  onRomClick: (rom: Rom) => void;
  onToggleSelect: (id: string) => void;
  language?: string;
  showFileName?: boolean;
  isHidden?: boolean;
  onContextMenu?: (rom: Rom, event: React.MouseEvent) => void;
}

// 根据显示模式解析卡片标题:文件名模式直接用原始文件名,否则用游戏名
function resolveCardName(rom: Rom, language: string | undefined, showFileName: boolean): string {
  return showFileName ? rom.file : getRomDisplayName(rom, language);
}

// Cover Card - Compact, image-focused
function CoverCard({ rom, isSelected, onRomClick, onToggleSelect, language, showFileName = false, isHidden = false, onContextMenu }: CardProps) {
  const coverUrl = useMediaUrl(getRomCover(rom));
  const displayName = resolveCardName(rom, language, showFileName);
  const [imgError, setImgError] = useState(false);
  const [containCover, setContainCover] = useState(false);

  return (
    <div
      onClick={() => onRomClick(rom)}
      onContextMenu={onContextMenu ? (event) => { event.preventDefault(); onContextMenu(rom, event); } : undefined}
      className={clsx(
        "rr-card group relative aspect-[3/4] rounded-[var(--radius-md)] overflow-hidden cursor-pointer",
        "border-[length:var(--border-width)]",
        "transition-all duration-[var(--motion-normal)] ease-[var(--motion-easing)]",
        isHidden && "opacity-40",
        isSelected
          ? "border-accent-primary ring-2 ring-accent-primary ring-offset-2 ring-offset-bg-primary"
          : "border-transparent"
      )}
    >
      {coverUrl && !imgError ? (
        <img
          src={coverUrl}
          alt=""
          loading="lazy"
          className={clsx(
            "w-full h-full transition-transform duration-[var(--motion-normal)] ease-[var(--motion-easing)] group-hover:scale-105",
            containCover ? "object-contain bg-bg-primary" : "object-cover"
          )}
          onLoad={(event) => setContainCover(shouldContainCover(event.currentTarget))}
          onError={() => setImgError(true)}
        />
      ) : (
        <div className="w-full h-full bg-gradient-to-br from-bg-tertiary to-bg-secondary flex flex-col items-center justify-center gap-2 px-2 py-3">
          <Gamepad2 className="w-10 h-10 shrink-0 text-text-muted/20 group-hover:text-accent-primary/30 transition-colors duration-[var(--motion-normal)] ease-[var(--motion-easing)]" />
          <span className="select-text w-full text-center text-xs font-medium text-text-secondary line-clamp-3 break-all">
            {displayName}
          </span>
        </div>
      )}

      {/* Hover overlay */}
      <div className="absolute inset-0 bg-gradient-to-t from-bg-primary/90 via-bg-primary/40 to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-[var(--motion-normal)] ease-[var(--motion-easing)] flex flex-col justify-end p-3">
        <h3 className="select-text text-sm font-semibold text-text-primary truncate transform translate-y-2 group-hover:translate-y-0 transition-transform duration-[var(--motion-normal)] ease-[var(--motion-easing)]">
          {displayName}
        </h3>
        <span className="text-[10px] text-text-secondary uppercase tracking-wider">
          {rom.system}
        </span>
      </div>

      {/* Selection checkbox */}
      <div
        className={clsx(
          "absolute top-2 right-2 z-10 transition-all duration-[var(--motion-fast)] ease-[var(--motion-easing)]",
          isSelected ? "opacity-100" : "opacity-0 group-hover:opacity-100"
        )}
      >
        <button
          onClick={(e) => {
            e.stopPropagation();
            onToggleSelect(rom.file);
          }}
          className={clsx(
            "p-2 -m-2 rounded-full flex items-center justify-center transition-colors duration-[var(--motion-fast)] ease-[var(--motion-easing)]"
          )}
        >
          <span
            className={clsx(
              "w-5 h-5 rounded-full flex items-center justify-center border-2 transition-colors duration-[var(--motion-fast)] ease-[var(--motion-easing)]",
              isSelected
                ? "bg-accent-primary border-accent-primary text-text-primary"
                : "bg-bg-primary/60 border-text-primary/50 text-transparent hover:border-text-primary hover:bg-bg-primary/80"
            )}
          >
            <CheckCircle2 className="w-3 h-3" />
          </span>
        </button>
      </div>
    </div>
  );
}

// Grid Card - Larger with metadata
function GridCard({ rom, isSelected, onRomClick, onToggleSelect, language, showFileName = false, isHidden = false, onContextMenu }: CardProps) {
  const coverUrl = useMediaUrl(getRomCover(rom));
  const displayName = resolveCardName(rom, language, showFileName);
  const [imgError, setImgError] = useState(false);
  const [containCover, setContainCover] = useState(false);

  return (
    <div
      onClick={() => onRomClick(rom)}
      onContextMenu={onContextMenu ? (event) => { event.preventDefault(); onContextMenu(rom, event); } : undefined}
      className={clsx(
        "rr-card group relative bg-bg-secondary rounded-[var(--radius-lg)] border-[length:var(--border-width)] overflow-hidden cursor-pointer",
        "transition-all duration-[var(--motion-normal)] ease-[var(--motion-easing)]",
        isHidden && "opacity-40",
        isSelected
          ? "border-accent-primary ring-1 ring-accent-primary"
          : "border-border-default"
      )}
    >
      <div className="aspect-[3/4] bg-gradient-to-br from-bg-tertiary to-bg-primary relative overflow-hidden">
        <div className="absolute inset-0 bg-accent-primary/5 group-hover:bg-accent-primary/10 transition-colors duration-[var(--motion-fast)] ease-[var(--motion-easing)]"></div>

        {coverUrl && !imgError ? (
          <img
            src={coverUrl}
            alt=""
            loading="lazy"
            className={clsx(
              "w-full h-full transition-transform duration-[var(--motion-normal)] ease-[var(--motion-easing)] group-hover:scale-110",
              containCover ? "object-contain bg-bg-primary" : "object-cover"
            )}
            onLoad={(event) => setContainCover(shouldContainCover(event.currentTarget))}
            onError={() => setImgError(true)}
          />
        ) : (
          <div className="absolute inset-0 flex items-center justify-center">
            <Gamepad2 className="w-12 h-12 text-text-muted/10 group-hover:text-accent-primary/20 transition-colors duration-[var(--motion-normal)] ease-[var(--motion-easing)]" />
          </div>
        )}

        <div
          className={clsx(
            "absolute top-3 right-3 z-10 transition-all duration-[var(--motion-fast)] ease-[var(--motion-easing)]",
            isSelected ? "opacity-100" : "opacity-0 group-hover:opacity-100"
          )}
        >
          <button
            onClick={(e) => {
              e.stopPropagation();
              onToggleSelect(rom.file);
            }}
            className="p-2 -m-2 rounded-full flex items-center justify-center"
          >
            <span
              className={clsx(
                "w-6 h-6 rounded-full flex items-center justify-center border transition-colors duration-[var(--motion-fast)] ease-[var(--motion-easing)]",
                isSelected
                  ? "bg-accent-primary border-accent-primary text-text-primary"
                  : "bg-bg-primary/60 border-text-primary/40 text-transparent hover:border-text-primary hover:bg-bg-primary/80"
              )}
            >
              <CheckCircle2 className="w-4 h-4" />
            </span>
          </button>
        </div>

        <div className="absolute top-3 left-3">
          <span className="px-2 py-1 rounded-[var(--radius-sm)] bg-bg-primary/60 backdrop-blur-md text-[10px] font-bold text-text-primary border border-border-default uppercase">
            {rom.system}
          </span>
        </div>
      </div>

      <div className="p-4">
        <h3
          className="select-text font-semibold text-text-primary truncate mb-1 group-hover:text-accent-primary transition-colors duration-[var(--motion-fast)] ease-[var(--motion-easing)]"
          title={displayName}
        >
          {displayName}
        </h3>
        <div className="flex items-center justify-between text-xs text-text-secondary">
          <div className="flex items-center gap-1"></div>
          {rom.rating && (
            <div className="flex items-center gap-1 text-accent-warning">
              <Star className="w-3 h-3 fill-current" />
              <span>{rom.rating}</span>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

// List Row - Compact table-like row
function ListRow({ rom, isSelected, onRomClick, onToggleSelect, language, showFileName = false, isHidden = false, onContextMenu }: CardProps) {
  const coverUrl = useMediaUrl(getRomCover(rom));
  const displayName = resolveCardName(rom, language, showFileName);
  const [imgError, setImgError] = useState(false);

  return (
    <div
      onClick={() => onRomClick(rom)}
      onContextMenu={onContextMenu ? (event) => { event.preventDefault(); onContextMenu(rom, event); } : undefined}
      className={clsx(
        "group flex items-center gap-4 px-4 py-3 transition-colors duration-[var(--motion-fast)] ease-[var(--motion-easing)] cursor-pointer border-b border-border-default",
        isHidden && "opacity-40",
        isSelected ? "bg-accent-primary/10 hover:bg-accent-primary/20" : "hover:bg-bg-tertiary"
      )}
    >
      {/* Checkbox */}
      <button
        onClick={(e) => {
          e.stopPropagation();
          onToggleSelect(rom.file);
        }}
        className="p-2 -m-2 flex items-center justify-center flex-shrink-0"
      >
        <span
          className={clsx(
            "w-5 h-5 rounded-[var(--radius-sm)] flex items-center justify-center border transition-colors duration-[var(--motion-fast)] ease-[var(--motion-easing)]",
            isSelected
              ? "bg-accent-primary border-accent-primary text-text-primary"
              : "bg-transparent border-border-default text-transparent hover:border-border-hover"
          )}
        >
          <CheckCircle2 className="w-3 h-3" />
        </span>
      </button>

      {/* Cover + Name */}
      <div className="flex items-center gap-3 flex-1 min-w-0">
        <div className="w-10 h-10 rounded-[var(--radius-sm)] bg-bg-primary flex items-center justify-center text-text-muted flex-shrink-0 overflow-hidden">
          {coverUrl && !imgError ? (
            <img
              src={coverUrl}
              alt=""
              loading="lazy"
              className="w-full h-full object-cover"
              onError={() => setImgError(true)}
            />
          ) : (
            <Gamepad2 className="w-4 h-4" />
          )}
        </div>
        <div className="min-w-0 flex-1">
          <div className="select-text font-medium text-text-primary group-hover:text-accent-primary transition-colors duration-[var(--motion-fast)] ease-[var(--motion-easing)] truncate">
            {displayName}
          </div>
          <div className="select-text text-xs text-text-muted truncate">
            {rom.directory}
          </div>
        </div>
      </div>

      {/* Size */}
      <span className="w-20 text-right text-xs text-text-secondary flex-shrink-0 tabular-nums">
        {formatFileSize(rom.file_size)}
      </span>

      {/* Date */}
      <span className="w-24 text-xs text-text-secondary flex-shrink-0 tabular-nums">
        {formatModifiedDate(rom.modified_at)}
      </span>

      {/* System */}
      <span className="w-20 flex-shrink-0">
        <span className="px-2 py-1 rounded-[var(--radius-sm)] bg-bg-primary border border-border-default text-xs font-medium text-text-secondary uppercase inline-block max-w-full truncate">
          {rom.system}
        </span>
      </span>
    </div>
  );
}

// ============ Main Component ============

export default function RomView({ roms, viewMode, cardScale = 1, showFileName = false, selectedIds, onRomClick, onToggleSelect, isRomHidden, onRomContextMenu }: RomViewProps) {
  const { t, i18n } = useTranslation();
  const parentRef = useRef<HTMLDivElement>(null);
  const config = VIEW_CONFIG[viewMode];
  const [containerWidth, setContainerWidth] = useState(1200);
  const contentPadding = viewMode === "list" ? 0 : 16;
  const contentWidth = Math.max(1, containerWidth - contentPadding * 2);
  const columns = useMemo(
    () => applyCardScale(config.getColumns(contentWidth), viewMode, cardScale),
    [cardScale, config, contentWidth, viewMode],
  );

  // Track container width for dynamic row height calculation
  useEffect(() => {
    const element = parentRef.current;
    if (!element) return;
    const updateWidth = () => setContainerWidth(element.clientWidth);
    updateWidth();
    const observer = new ResizeObserver(updateWidth);
    observer.observe(element);
    return () => observer.disconnect();
  }, []);

  // Calculate dynamic row height based on container width
  const rowHeight = useMemo(() => {
    return config.getRowHeight(contentWidth, columns, config.gap);
  }, [config, contentWidth, columns]);

  // Group roms into rows (for grid/cover modes)
  const rows = useMemo(() => {
    if (viewMode === "list") {
      // For list view, each rom is its own row
      return roms.map((rom) => [rom]);
    }
    const result: Rom[][] = [];
    for (let i = 0; i < roms.length; i += columns) {
      result.push(roms.slice(i, i + columns));
    }
    return result;
  }, [roms, columns, viewMode]);

  // Memoize estimateSize to prevent virtualizer recreation
  const estimateSize = useCallback(() => rowHeight, [rowHeight]);

  // @tanstack/react-virtual 返回的函数无法被 React Compiler 安全记忆化，属于三方库限制，非本项目代码问题。
  // eslint-disable-next-line react-hooks/incompatible-library
  const virtualizer = useVirtualizer({
    count: rows.length,
    getScrollElement: () => parentRef.current,
    estimateSize,
    overscan: config.overscan,
    gap: config.gap, // Add gap between rows
  });

  const virtualRows = virtualizer.getVirtualItems();

  // List view header
  const listHeader = viewMode === "list" && (
    <div className="flex items-center gap-4 px-4 py-3 bg-bg-tertiary text-text-secondary text-xs uppercase font-medium border-b border-border-default sticky top-0 z-10">
      <div className="w-5"></div>
      <div className="flex-1">{t("common.name")}</div>
      <div className="w-20 text-right">{t("common.size")}</div>
      <div className="w-24">{t("common.date")}</div>
      <div className="w-20">{t("common.system")}</div>
    </div>
  );

  return (
    <div
      ref={parentRef}
      className={clsx(
        "h-full overflow-auto",
        viewMode === "list" && "bg-bg-secondary rounded-[var(--radius-lg)] border-[length:var(--border-width)] border-border-default"
      )}
      style={{ contain: "strict" }}
    >
      {listHeader}
      <div
        className="transition-opacity duration-[var(--motion-fast)] ease-[var(--motion-easing)]"
        style={{
          height: `${virtualizer.getTotalSize() + contentPadding * 2}px`,
          width: "100%",
          position: "relative",
        }}
      >
        {virtualRows.map((virtualRow) => {
          const rowRoms = rows[virtualRow.index];

          return (
            <div
              key={virtualRow.key}
              className="transition-all duration-[var(--motion-fast)] ease-[var(--motion-easing)]"
              style={{
                position: "absolute",
                top: 0,
                left: `${contentPadding}px`,
                width: `calc(100% - ${contentPadding * 2}px)`,
                height: `${virtualRow.size}px`,
                transform: `translateY(${virtualRow.start + contentPadding}px)`,
              }}
            >
              {viewMode === "list" ? (
                // List view - single item per row
                <ListRow
                  rom={rowRoms[0]}
                  isSelected={selectedIds.has(rowRoms[0].file)}
                  onRomClick={onRomClick}
                  onToggleSelect={onToggleSelect}
                  language={i18n.resolvedLanguage}
                  showFileName={showFileName}
                  isHidden={isRomHidden?.(rowRoms[0])}
                  onContextMenu={onRomContextMenu}
                />
              ) : (
                // Grid/Cover view - multiple items per row
                <div
                  className="grid min-w-0 transition-all duration-[var(--motion-fast)] ease-[var(--motion-easing)]"
                  style={{
                    gridTemplateColumns: `repeat(${columns}, 1fr)`,
                    gap: `${config.gap}px`,
                  }}
                >
                  {rowRoms.map((rom, index) => {
                    const key = rom.file
                      ? `${rom.directory}/${rom.file}`
                      : `${rom.directory}/${rom.name}-${index}`;
                    const isSelected = selectedIds.has(rom.file);

                    return (
                      <div key={key} className="min-w-0" style={{ padding: "8px" }}>
                        {viewMode === "cover" ? (
                          <CoverCard
                            rom={rom}
                            isSelected={isSelected}
                            onRomClick={onRomClick}
                            onToggleSelect={onToggleSelect}
                            language={i18n.resolvedLanguage}
                            showFileName={showFileName}
                            isHidden={isRomHidden?.(rom)}
                            onContextMenu={onRomContextMenu}
                          />
                        ) : (
                          <GridCard
                            rom={rom}
                            isSelected={isSelected}
                            onRomClick={onRomClick}
                            onToggleSelect={onToggleSelect}
                            language={i18n.resolvedLanguage}
                            showFileName={showFileName}
                            isHidden={isRomHidden?.(rom)}
                            onContextMenu={onRomContextMenu}
                          />
                        )}
                      </div>
                    );
                  })}
                </div>
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}
