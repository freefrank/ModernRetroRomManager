import { Clock, Gamepad2, Play, Star, CheckCircle2 } from "lucide-react";
import type { Rom } from "@/types";
import { getMediaUrl } from "@/utils/media";
import { clsx } from "clsx";

interface RomGridProps {
  roms: Rom[];
  selectedIds: Set<string>;
  onRomClick: (rom: Rom) => void;
  onToggleSelect: (id: string) => void;
}

export default function RomGrid({ roms, selectedIds, onRomClick, onToggleSelect }: RomGridProps) {
  return (
    <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 2xl:grid-cols-6 gap-6">
      {roms.map((rom) => {
        const boxfront = rom.media?.find((m) => m.assetType === "boxfront");
        const coverUrl = getMediaUrl(boxfront?.path);
        const isSelected = selectedIds.has(rom.id);

        return (
          <div
            key={rom.id}
            onClick={() => onRomClick(rom)}
            className={clsx(
              "group relative bg-bg-secondary rounded-2xl border overflow-hidden transition-all duration-300 hover:-translate-y-1 cursor-pointer",
              isSelected
                ? "border-accent-primary ring-1 ring-accent-primary shadow-[0_0_30px_rgba(124,58,237,0.2)]"
                : "border-border-default hover:border-accent-primary/50 hover:shadow-[0_0_30px_rgba(124,58,237,0.1)]"
            )}
          >
            {/* Image Section */}
            <div className="aspect-[3/4] bg-gradient-to-br from-bg-tertiary to-bg-primary relative overflow-hidden">
              <div className="absolute inset-0 bg-accent-primary/5 group-hover:bg-accent-primary/10 transition-colors"></div>

              {coverUrl ? (
                <img
                  src={coverUrl}
                  alt=""
                  className="w-full h-full object-cover transition-transform duration-500 group-hover:scale-110"
                />
              ) : (
                <div className="absolute inset-0 flex items-center justify-center">
                  <Gamepad2 className="w-12 h-12 text-text-muted/10 group-hover:text-accent-primary/20 transition-colors duration-500" />
                </div>
              )}

              {/* Hover Overlay */}
              <div className="absolute inset-0 bg-black/60 opacity-0 group-hover:opacity-100 transition-opacity duration-300 flex items-center justify-center backdrop-blur-sm">
                <button
                  className="p-3 rounded-full bg-accent-primary text-text-primary transform scale-50 group-hover:scale-100 transition-all duration-300 hover:bg-accent-primary/90 shadow-lg"
                  onClick={(e) => {
                    e.stopPropagation();
                    // Play logic
                  }}
                >
                  <Play className="w-6 h-6 ml-1" />
                </button>
              </div>

              {/* Selection Checkbox (Visible on hover or selected) */}
              <div
                className={clsx(
                  "absolute top-3 right-3 z-10 transition-all duration-200",
                  isSelected ? "opacity-100" : "opacity-0 group-hover:opacity-100"
                )}
              >
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    onToggleSelect(rom.id);
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
                  {rom.systemId}
                </span>
              </div>
            </div>

            {/* Content */}
            <div className="p-4">
              <h3
                className="font-semibold text-text-primary truncate mb-1 group-hover:text-accent-primary transition-colors"
                title={rom.metadata?.name || rom.filename}
              >
                {rom.metadata?.name || rom.filename}
              </h3>
              <div className="flex items-center justify-between text-xs text-text-secondary">
                <div className="flex items-center gap-1">
                  <Clock className="w-3 h-3" />
                  <span>{Math.round((rom.size / 1024 / 1024) * 100) / 100} MB</span>
                </div>
                {rom.metadata?.rating && (
                  <div className="flex items-center gap-1 text-accent-warning">
                    <Star className="w-3 h-3 fill-current" />
                    <span>{rom.metadata.rating.toFixed(1)}</span>
                  </div>
                )}
              </div>
            </div>
          </div>
        );
      })}
    </div>
  );
}
