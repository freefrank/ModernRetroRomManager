import { useState, useEffect } from "react";
import { Search, LayoutGrid, List, Filter, Plus, Ghost } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useRomStore } from "@/stores/romStore";
import { useAppStore } from "@/stores/appStore";
import { open } from "@tauri-apps/plugin-dialog";
import { clsx } from "clsx";
import { useDebounce } from "@/hooks/useDebounce";
import type { Rom } from "@/types";

import RomGrid from "@/components/rom/RomGrid";
import RomList from "@/components/rom/RomList";
import RomDetail from "@/components/rom/RomDetail";

export default function Library() {
  const { t } = useTranslation();
  const { roms, fetchRoms, addScanDirectory, stats } = useRomStore();
  const { viewMode, setViewMode, searchQuery, setSearchQuery } = useAppStore();
  const debouncedSearch = useDebounce(searchQuery, 300);
  const [activeRom, setActiveRom] = useState<Rom | null>(null);

  useEffect(() => {
    fetchRoms({ searchQuery: debouncedSearch });
  }, [fetchRoms, debouncedSearch]);

  const handleAddDirectory = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
      });

      if (selected && typeof selected === "string") {
        await addScanDirectory(selected);
      }
    } catch (error) {
      console.error("Error adding directory:", error);
    }
  };

  return (
    <div className="flex flex-col h-full space-y-8 max-w-[1600px] mx-auto w-full pb-8">
      {/* Header Section */}
      <div className="flex flex-col gap-6 md:flex-row md:items-center md:justify-between sticky top-0 z-10 bg-bg-primary/50 backdrop-blur-md py-4 -mt-4 pt-8">
        <div>
          <h1 className="text-4xl font-bold tracking-tight text-white mb-2">
            {t("library.title")}
          </h1>
          <p className="text-text-secondary font-medium">
            {t("library.gameCount", { count: stats.totalRoms })}
          </p>
        </div>

        <div className="flex items-center gap-4">
          {/* Spotlight Search */}
          <div className="relative group">
            <div className="absolute -inset-0.5 bg-gradient-to-r from-accent-primary to-accent-secondary rounded-xl blur opacity-20 group-hover:opacity-40 transition duration-200"></div>
            <div className="relative flex items-center bg-[#151621] rounded-xl border border-white/10 w-full md:w-80 transition-colors focus-within:border-accent-primary/50 focus-within:bg-[#1E1F2E]">
              <Search className="w-5 h-5 text-text-muted ml-4" />
              <input
                type="text"
                placeholder={t("library.search")}
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                className="w-full bg-transparent border-none focus:ring-0 text-sm px-3 py-3 text-white placeholder:text-text-muted focus:outline-none"
              />
              <div className="hidden md:flex items-center gap-1 pr-3">
                <kbd className="hidden sm:inline-block px-2 py-0.5 rounded text-[10px] font-mono font-medium bg-white/5 text-text-muted border border-white/10">⌘K</kbd>
              </div>
            </div>
          </div>

          {/* View Toggle & Filters */}
          <div className="flex items-center gap-2 p-1 bg-[#151621] rounded-xl border border-white/10">
            <button 
              onClick={() => setViewMode("grid")}
              className={clsx(
                "p-2 rounded-lg transition-all",
                viewMode === "grid" 
                  ? "bg-accent-primary text-white shadow-lg shadow-accent-primary/20" 
                  : "text-text-muted hover:text-white hover:bg-white/5"
              )}
              title={t("library.viewMode.grid")}
            >
              <LayoutGrid className="w-5 h-5" />
            </button>
            <button 
              onClick={() => setViewMode("list")}
              className={clsx(
                "p-2 rounded-lg transition-all",
                viewMode === "list" 
                  ? "bg-accent-primary text-white shadow-lg shadow-accent-primary/20" 
                  : "text-text-muted hover:text-white hover:bg-white/5"
              )}
              title={t("library.viewMode.list")}
            >
              <List className="w-5 h-5" />
            </button>
          </div>
          
          <button className="p-3 rounded-xl bg-[#151621] border border-white/10 text-text-secondary hover:text-white hover:border-white/20 hover:bg-white/5 transition-all">
            <Filter className="w-5 h-5" />
          </button>
        </div>
      </div>

      {/* Content Area */}
      <div className="flex-1">
        {roms.length === 0 ? (
          <div className="flex flex-col items-center justify-center min-h-[400px]">
            <div className="text-center max-w-md mx-auto relative group cursor-default mb-16">
              {/* Glowing Effect Background */}
              <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-64 h-64 bg-accent-primary/10 rounded-full blur-[80px] group-hover:bg-accent-primary/20 transition-all duration-700"></div>
              
              <div className="relative">
                <div className="w-32 h-32 mx-auto mb-8 rounded-3xl bg-gradient-to-br from-[#1E1F2E] to-[#151621] border border-white/5 flex items-center justify-center shadow-2xl group-hover:scale-105 transition-transform duration-500 ring-1 ring-white/10">
                  <Ghost className="w-16 h-16 text-text-muted group-hover:text-accent-primary transition-colors duration-300" />
                </div>
                
                <h2 className="text-3xl font-bold text-white mb-3 tracking-tight">
                  {t("library.empty.title")}
                </h2>
                <p className="text-text-secondary mb-8 text-lg leading-relaxed">
                  {t("library.empty.description")}
                </p>
                
                <button 
                  onClick={handleAddDirectory}
                  className="relative inline-flex group/btn"
                >
                  <div className="absolute transition-all duration-300 opacity-70 -inset-px bg-gradient-to-r from-accent-primary to-accent-secondary rounded-xl blur-lg group-hover/btn:opacity-100 group-hover/btn:-inset-1 group-hover/btn:duration-200 animate-tilt"></div>
                  <div className="relative inline-flex items-center gap-2 px-8 py-4 bg-[#0B0C15] rounded-xl leading-none text-white transition duration-200 border border-white/10 hover:bg-[#151621]">
                    <Plus className="w-5 h-5 text-accent-secondary" />
                    <span className="font-semibold tracking-wide">{t("library.empty.addDirectory")}</span>
                  </div>
                </button>
              </div>
            </div>
          </div>
        ) : (
          viewMode === "grid" ? (
            <RomGrid roms={roms} onRomClick={setActiveRom} />
          ) : (
            <RomList roms={roms} onRomClick={setActiveRom} />
          )
        )}
      </div>

      {/* Detail Panel */}
      <RomDetail rom={activeRom} onClose={() => setActiveRom(null)} />
    </div>
  );
}
