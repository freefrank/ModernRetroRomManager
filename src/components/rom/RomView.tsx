import { useState, useEffect, useRef, useMemo, useCallback } from "react";
import { Gamepad2, Play, Star, CheckCircle2 } from "lucide-react";
import { useVirtualizer } from "@tanstack/react-virtual";
import { useTranslation } from "react-i18next";
import { resolveMediaUrlAsync, getCachedMediaUrl } from "@/lib/api";
import type { Rom, ViewMode } from "@/types";
import { clsx } from "clsx";

interface RomViewProps {
  roms: Rom[];
  viewMode: ViewMode;
  selectedIds: Set<string>;
  onRomClick: (rom: Rom) => void;
  onToggleSelect: (id: string) => void;
}

// 获取 ROM 封面路径，优先使用 temp_data
function getRomCover(rom: Rom): string | undefined {
  return rom.temp_data?.box_front || rom.box_front || rom.gridicon;
}

// Shared hook for async media URL resolution with cache support
function useMediaUrl(path: string | undefined): string | null {
  // Check cache first for instant display
  const cachedUrl = path ? getCachedMediaUrl(path) : null;
  const [url, setUrl] = useState<string | null>(cachedUrl);

  useEffect(() => {
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
    resolveMediaUrlAsync(path).then(setUrl);
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
      const textAreaHeight = 72; // p-4 padding + title + rating
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

// Hook for responsive column count based on view mode
function useColumnCount(viewMode: ViewMode) {
  const [columns, setColumns] = useState(() => 
    VIEW_CONFIG[viewMode].getColumns(typeof window !== "undefined" ? window.innerWidth : 1280)
  );

  useEffect(() => {
    const updateColumns = () => {
      setColumns(VIEW_CONFIG[viewMode].getColumns(window.innerWidth));
    };
    updateColumns();
    window.addEventListener("resize", updateColumns);
    return () => window.removeEventListener("resize", updateColumns);
  }, [viewMode]);

  return columns;
}

// ============ Card Components ============

interface CardProps {
  rom: Rom;
  isSelected: boolean;
  onRomClick: (rom: Rom) => void;
  onToggleSelect: (id: string) => void;
}

// Cover Card - Compact, image-focused
function CoverCard({ rom, isSelected, onRomClick, onToggleSelect }: CardProps) {
  const coverUrl = useMediaUrl(getRomCover(rom));
  const [imgError, setImgError] = useState(false);

  return (
    <div
      onClick={() => onRomClick(rom)}
      className={clsx(
        "group relative aspect-[3/4] rounded-lg overflow-hidden transition-all duration-300 cursor-pointer",
        isSelected
          ? "ring-2 ring-accent-primary ring-offset-2 ring-offset-bg-primary shadow-[0_0_20px_rgba(124,58,237,0.3)]"
          : "hover:ring-1 hover:ring-accent-primary/50 hover:shadow-lg"
      )}
    >
      {coverUrl && !imgError ? (
        <img
          src={coverUrl}
          alt=""
          loading="lazy"
          className="w-full h-full object-cover transition-transform duration-500 group-hover:scale-105"
          onError={() => setImgError(true)}
        />
      ) : (
        <div className="w-full h-full bg-gradient-to-br from-bg-tertiary to-bg-secondary flex items-center justify-center">
          <Gamepad2 className="w-10 h-10 text-text-muted/20 group-hover:text-accent-primary/30 transition-colors duration-300" />
        </div>
      )}

      {/* Hover overlay */}
      <div className="absolute inset-0 bg-gradient-to-t from-black/90 via-black/40 to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-300 flex flex-col justify-end p-3">
        <div className="absolute inset-0 flex items-center justify-center">
          <button
            className="p-3 rounded-full bg-accent-primary text-text-primary transform scale-50 opacity-0 group-hover:scale-100 group-hover:opacity-100 transition-all duration-300 hover:bg-accent-primary/90 shadow-lg"
            onClick={(e) => e.stopPropagation()}
          >
            <Play className="w-5 h-5 ml-0.5" />
          </button>
        </div>
        <h3 className="text-sm font-semibold text-white truncate transform translate-y-2 group-hover:translate-y-0 transition-transform duration-300">
          {rom.name}
        </h3>
        <span className="text-[10px] text-white/60 uppercase tracking-wider">
          {rom.system}
        </span>
      </div>

      {/* Selection checkbox */}
      <div
        className={clsx(
          "absolute top-2 right-2 z-10 transition-all duration-200",
          isSelected ? "opacity-100" : "opacity-0 group-hover:opacity-100"
        )}
      >
        <button
          onClick={(e) => {
            e.stopPropagation();
            onToggleSelect(rom.file);
          }}
          className={clsx(
            "w-5 h-5 rounded-full flex items-center justify-center border-2 transition-colors shadow-lg",
            isSelected
              ? "bg-accent-primary border-accent-primary text-white"
              : "bg-black/50 border-white/50 text-transparent hover:border-white hover:bg-black/70"
          )}
        >
          <CheckCircle2 className="w-3 h-3" />
        </button>
      </div>
    </div>
  );
}

// Grid Card - Larger with metadata
function GridCard({ rom, isSelected, onRomClick, onToggleSelect }: CardProps) {
  const coverUrl = useMediaUrl(getRomCover(rom));
  const [imgError, setImgError] = useState(false);

  return (
    <div
      onClick={() => onRomClick(rom)}
      className={clsx(
        "group relative bg-bg-secondary rounded-2xl border overflow-hidden transition-all duration-300 hover:-translate-y-1 cursor-pointer",
        isSelected
          ? "border-accent-primary ring-1 ring-accent-primary shadow-[0_0_30px_rgba(124,58,237,0.2)]"
          : "border-border-default hover:border-accent-primary/50 hover:shadow-[0_0_30px_rgba(124,58,237,0.1)]"
      )}
    >
      <div className="aspect-[3/4] bg-gradient-to-br from-bg-tertiary to-bg-primary relative overflow-hidden">
        <div className="absolute inset-0 bg-accent-primary/5 group-hover:bg-accent-primary/10 transition-colors"></div>

        {coverUrl && !imgError ? (
          <img
            src={coverUrl}
            alt=""
            loading="lazy"
            className="w-full h-full object-cover transition-transform duration-500 group-hover:scale-110"
            onError={() => setImgError(true)}
          />
        ) : (
          <div className="absolute inset-0 flex items-center justify-center">
            <Gamepad2 className="w-12 h-12 text-text-muted/10 group-hover:text-accent-primary/20 transition-colors duration-500" />
          </div>
        )}

        <div className="absolute inset-0 bg-black/60 opacity-0 group-hover:opacity-100 transition-opacity duration-300 flex items-center justify-center backdrop-blur-sm">
          <button
            className="p-3 rounded-full bg-accent-primary text-text-primary transform scale-50 group-hover:scale-100 transition-all duration-300 hover:bg-accent-primary/90 shadow-lg"
            onClick={(e) => e.stopPropagation()}
          >
            <Play className="w-6 h-6 ml-1" />
          </button>
        </div>

        <div
          className={clsx(
            "absolute top-3 right-3 z-10 transition-all duration-200",
            isSelected ? "opacity-100" : "opacity-0 group-hover:opacity-100"
          )}
        >
          <button
            onClick={(e) => {
              e.stopPropagation();
              onToggleSelect(rom.file);
            }}
            className={clsx(
              "w-6 h-6 rounded-full flex items-center justify-center border transition-colors",
              isSelected
                ? "bg-accent-primary border-accent-primary text-text-primary"
                : "bg-black/50 border-white/30 text-transparent hover:border-white hover:bg-black/70"
            )}
          >
            <CheckCircle2 className="w-4 h-4" />
          </button>
        </div>

        <div className="absolute top-3 left-3">
          <span className="px-2 py-1 rounded-md bg-bg-primary/60 backdrop-blur-md text-[10px] font-bold text-text-primary border border-border-default uppercase">
            {rom.system}
          </span>
        </div>
      </div>

      <div className="p-4">
        <h3
          className="font-semibold text-text-primary truncate mb-1 group-hover:text-accent-primary transition-colors"
          title={rom.name}
        >
          {rom.name}
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
function ListRow({ rom, isSelected, onRomClick, onToggleSelect }: CardProps) {
  const { t } = useTranslation();
  const coverUrl = useMediaUrl(getRomCover(rom));
  const [imgError, setImgError] = useState(false);

  return (
    <div
      onClick={() => onRomClick(rom)}
      className={clsx(
        "group flex items-center gap-4 px-4 py-3 transition-colors cursor-pointer border-b border-border-default",
        isSelected ? "bg-accent-primary/10 hover:bg-accent-primary/20" : "hover:bg-bg-tertiary"
      )}
    >
      {/* Checkbox */}
      <button
        onClick={(e) => {
          e.stopPropagation();
          onToggleSelect(rom.file);
        }}
        className={clsx(
          "w-5 h-5 rounded flex items-center justify-center border transition-colors flex-shrink-0",
          isSelected
            ? "bg-accent-primary border-accent-primary text-text-primary"
            : "bg-transparent border-border-default text-transparent hover:border-border-hover"
        )}
      >
        <CheckCircle2 className="w-3 h-3" />
      </button>

      {/* Cover + Name */}
      <div className="flex items-center gap-3 flex-1 min-w-0">
        <div className="w-10 h-10 rounded bg-bg-primary flex items-center justify-center text-text-muted flex-shrink-0 overflow-hidden">
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
          <div className="font-medium text-text-primary group-hover:text-accent-primary transition-colors truncate">
            {rom.name}
          </div>
          <div className="text-xs text-text-muted truncate">
            {rom.directory}
          </div>
        </div>
      </div>

      {/* System */}
      <span className="px-2 py-1 rounded bg-bg-primary border border-border-default text-xs font-medium text-text-secondary uppercase flex-shrink-0">
        {rom.system}
      </span>

      {/* Size - hidden on small screens */}
      <span className="text-sm text-text-secondary w-20 text-right hidden md:block">
        {t("common.notAvailable")}
      </span>

      {/* Date - hidden on small screens */}
      <span className="text-sm text-text-muted w-24 text-right hidden lg:block">
        {t("common.notAvailable")}
      </span>
    </div>
  );
}

// ============ Main Component ============

export default function RomView({ roms, viewMode, selectedIds, onRomClick, onToggleSelect }: RomViewProps) {
  const { t } = useTranslation();
  const parentRef = useRef<HTMLDivElement>(null);
  const columns = useColumnCount(viewMode);
  const config = VIEW_CONFIG[viewMode];
  const [containerWidth, setContainerWidth] = useState(1200);

  // Track container width for dynamic row height calculation
  useEffect(() => {
    const updateWidth = () => {
      if (parentRef.current) {
        setContainerWidth(parentRef.current.clientWidth);
      }
    };
    updateWidth();
    window.addEventListener("resize", updateWidth);
    return () => window.removeEventListener("resize", updateWidth);
  }, []);

  // Calculate dynamic row height based on container width
  const rowHeight = useMemo(() => {
    return config.getRowHeight(containerWidth, columns, config.gap);
  }, [config, containerWidth, columns]);

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
      <div className="w-20">{t("common.system")}</div>
      <div className="w-20 text-right hidden md:block">{t("common.size")}</div>
      <div className="w-24 text-right hidden lg:block">{t("common.date")}</div>
    </div>
  );

  return (
    <div
      ref={parentRef}
      className={clsx(
        "h-full overflow-auto",
        viewMode === "list" && "bg-bg-secondary rounded-xl border border-border-default"
      )}
      style={{ contain: "strict" }}
    >
      {listHeader}
      <div
        className="transition-opacity duration-200 ease-out"
        style={{
          height: `${virtualizer.getTotalSize()}px`,
          width: "100%",
          position: "relative",
        }}
      >
        {virtualRows.map((virtualRow) => {
          const rowRoms = rows[virtualRow.index];
          
          return (
            <div
              key={virtualRow.key}
              className="transition-all duration-200 ease-out"
              style={{
                position: "absolute",
                top: 0,
                left: 0,
                width: "100%",
                height: `${virtualRow.size}px`,
                transform: `translateY(${virtualRow.start}px)`,
              }}
            >
              {viewMode === "list" ? (
                // List view - single item per row
                <ListRow
                  rom={rowRoms[0]}
                  isSelected={selectedIds.has(rowRoms[0].file)}
                  onRomClick={onRomClick}
                  onToggleSelect={onToggleSelect}
                />
              ) : (
                // Grid/Cover view - multiple items per row
                <div
                  className="grid transition-all duration-200 ease-out"
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

                    return viewMode === "cover" ? (
                      <CoverCard
                        key={key}
                        rom={rom}
                        isSelected={isSelected}
                        onRomClick={onRomClick}
                        onToggleSelect={onToggleSelect}
                      />
                    ) : (
                      <GridCard
                        key={key}
                        rom={rom}
                        isSelected={isSelected}
                        onRomClick={onRomClick}
                        onToggleSelect={onToggleSelect}
                      />
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
