import { Clock, Gamepad2, Play, Star } from "lucide-react";
import type { Rom } from "@/types";

interface RomGridProps {
  roms: Rom[];
  onRomClick: (rom: Rom) => void;
}

export default function RomGrid({ roms, onRomClick }: RomGridProps) {
  return (
    <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 2xl:grid-cols-6 gap-6">
      {roms.map((rom) => (
        <div
          key={rom.id}
          onClick={() => onRomClick(rom)}
          className="group relative bg-[#151621] rounded-2xl border border-white/5 overflow-hidden hover:border-accent-primary/50 transition-all duration-300 hover:shadow-[0_0_30px_rgba(124,58,237,0.1)] hover:-translate-y-1 cursor-pointer"
        >
          {/* Image Placeholder */}
          <div className="aspect-[3/4] bg-gradient-to-br from-[#1E1F2E] to-[#0B0C15] relative overflow-hidden">
            <div className="absolute inset-0 bg-accent-primary/5 group-hover:bg-accent-primary/10 transition-colors"></div>

            {rom.media?.length ? (
              // TODO: 显示真实图片
              <div className="absolute inset-0 flex items-center justify-center">
                <Gamepad2 className="w-12 h-12 text-white/5" />
              </div>
            ) : (
              <div className="absolute inset-0 flex items-center justify-center">
                <Gamepad2 className="w-12 h-12 text-white/5 group-hover:text-accent-primary/20 transition-colors duration-500" />
              </div>
            )}

            {/* Hover Overlay */}
            <div className="absolute inset-0 bg-black/60 opacity-0 group-hover:opacity-100 transition-opacity duration-300 flex items-center justify-center backdrop-blur-sm">
              <button className="p-3 rounded-full bg-accent-primary text-white transform scale-50 group-hover:scale-100 transition-all duration-300 hover:bg-accent-primary/90 shadow-lg">
                <Play className="w-6 h-6 ml-1" />
              </button>
            </div>

            <div className="absolute top-3 left-3">
              <span className="px-2 py-1 rounded-md bg-black/60 backdrop-blur-md text-[10px] font-bold text-white border border-white/10 uppercase">
                {rom.systemId}
              </span>
            </div>
          </div>

          {/* Content */}
          <div className="p-4">
            <h3
              className="font-semibold text-white truncate mb-1 group-hover:text-accent-primary transition-colors"
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
      ))}
    </div>
  );
}
