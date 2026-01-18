import { NavLink, useLocation, useNavigate } from "react-router-dom";
import { Library, Database, ArrowRightLeft, Settings, Gamepad2, ChevronDown, ChevronRight } from "lucide-react";
import { motion, AnimatePresence } from "framer-motion";
import { clsx } from "clsx";
import { useTranslation } from "react-i18next";
import { useState } from "react";
import { useRomStore } from "@/stores/romStore";

const NAV_ITEMS = [
  { to: "/scraper", icon: Database, labelKey: "nav.scraper" },
  { to: "/import", icon: ArrowRightLeft, labelKey: "nav.import" },
  { to: "/settings", icon: Settings, labelKey: "nav.settings" },
];

export default function Sidebar() {
  const { t } = useTranslation();
  const location = useLocation();
  const navigate = useNavigate();
  const { availableSystems, selectedSystem, setSelectedSystem } = useRomStore();
  
  const [isLibraryExpanded, setIsLibraryExpanded] = useState(true);

  const isLibraryActive = location.pathname === "/";

  const handleLibraryClick = () => {
    navigate("/");
    setSelectedSystem(null);
  };

  const handleSystemClick = (systemName: string) => {
    navigate("/");
    setSelectedSystem(systemName);
  };

  return (
    <aside className="w-64 flex flex-col bg-bg-primary/95 backdrop-blur-xl border-r border-border-default relative z-20 h-full">
      {/* Logo */}
      <div className="h-20 flex items-center px-6 flex-shrink-0">
        <div className="flex items-center gap-3">
          <div className="relative">
            <div className="absolute inset-0 bg-accent-primary blur-md opacity-50 rounded-lg"></div>
            <div className="w-10 h-10 bg-gradient-to-br from-accent-primary to-accent-secondary rounded-lg flex items-center justify-center relative z-10 shadow-lg">
              <Gamepad2 className="text-text-primary w-6 h-6" />
            </div>
          </div>
          <div className="flex flex-col">
            <span className="font-bold text-lg tracking-tight leading-none text-text-primary">RetroRom</span>
            <span className="text-[10px] font-medium text-accent-secondary uppercase tracking-widest">Manager</span>
          </div>
        </div>
      </div>

      {/* Navigation */}
      <nav className="flex-1 px-4 py-6 space-y-2 overflow-y-auto custom-scrollbar">
        {/* Library Item with Expansion */}
        <div className="space-y-1">
            <div
                className={clsx(
                "relative flex items-center gap-3 px-4 py-3 rounded-xl transition-all duration-300 group cursor-pointer select-none",
                isLibraryActive && !selectedSystem
                    ? "text-text-primary"
                    : "text-text-secondary hover:text-text-primary hover:bg-bg-tertiary"
                )}
                onClick={handleLibraryClick}
            >
                {isLibraryActive && !selectedSystem && (
                    <motion.div
                    layoutId="activeNav"
                    className="absolute inset-0 bg-gradient-to-r from-accent-primary/20 to-transparent border-l-4 border-accent-primary rounded-xl"
                    initial={{ opacity: 0 }}
                    animate={{ opacity: 1 }}
                    exit={{ opacity: 0 }}
                    />
                )}
                
                <Library
                    className={clsx(
                    "w-5 h-5 z-10 transition-transform duration-300",
                    isLibraryActive && !selectedSystem ? "text-accent-secondary scale-110" : "group-hover:scale-110"
                    )}
                />
                <span className="font-medium z-10 text-sm tracking-wide flex-1">
                    {t("nav.library", { defaultValue: "ROM 库" })}
                </span>
                
                <button
                    onClick={(e) => {
                        e.stopPropagation();
                        setIsLibraryExpanded(!isLibraryExpanded);
                    }}
                    className="z-10 p-1 rounded-md hover:bg-bg-primary/50 text-text-muted hover:text-text-primary transition-colors"
                >
                    {isLibraryExpanded ? (
                        <ChevronDown className="w-4 h-4" />
                    ) : (
                        <ChevronRight className="w-4 h-4" />
                    )}
                </button>
            </div>

            {/* Sub-items (Systems) */}
            <AnimatePresence initial={false}>
                {isLibraryExpanded && (
                    <motion.div
                        initial={{ height: 0, opacity: 0 }}
                        animate={{ height: "auto", opacity: 1 }}
                        exit={{ height: 0, opacity: 0 }}
                        transition={{ duration: 0.2 }}
                        className="overflow-hidden ml-4 pl-4 border-l border-border-default space-y-1"
                    >
                        {availableSystems.map((sys) => (
                            <button
                                key={sys.name}
                                onClick={() => handleSystemClick(sys.name)}
                                className={clsx(
                                    "w-full flex items-center gap-3 px-4 py-2 rounded-lg text-sm transition-all duration-200 group relative",
                                    selectedSystem === sys.name
                                        ? "text-text-primary bg-accent-primary/10"
                                        : "text-text-muted hover:text-text-primary hover:bg-bg-tertiary"
                                )}
                            >
                                <span className={clsx(
                                    "w-1.5 h-1.5 rounded-full transition-colors",
                                    selectedSystem === sys.name ? "bg-accent-secondary" : "bg-border-default group-hover:bg-text-secondary"
                                )}></span>
                                <span className="truncate flex-1 text-left">{sys.name}</span>
                                <span className="text-[10px] text-text-muted bg-bg-tertiary px-1.5 py-0.5 rounded-md">
                                    {sys.romCount}
                                </span>
                            </button>
                        ))}
                    </motion.div>
                )}
            </AnimatePresence>
        </div>

        {/* Other Nav Items */}
        {NAV_ITEMS.map((item) => (
          <NavLink
            key={item.to}
            to={item.to}
            className={({ isActive }) =>
              clsx(
                "relative flex items-center gap-3 px-4 py-3 rounded-xl transition-all duration-300 group overflow-hidden",
                isActive
                  ? "text-text-primary"
                  : "text-text-secondary hover:text-text-primary hover:bg-bg-tertiary"
              )
            }
          >
            {({ isActive }) => (
              <>
                {isActive && (
                  <motion.div
                    layoutId="activeNav"
                    className="absolute inset-0 bg-gradient-to-r from-accent-primary/20 to-transparent border-l-4 border-accent-primary rounded-xl"
                    initial={{ opacity: 0 }}
                    animate={{ opacity: 1 }}
                    exit={{ opacity: 0 }}
                  />
                )}

                <item.icon
                  className={clsx(
                    "w-5 h-5 z-10 transition-transform duration-300",
                    isActive ? "text-accent-secondary scale-110" : "group-hover:scale-110"
                  )}
                />
                <span className={clsx("font-medium z-10 text-sm tracking-wide")}>
                  {t(item.labelKey)}
                </span>
              </>
            )}
          </NavLink>
        ))}
      </nav>

      {/* Footer Info */}
      <div className="p-6 border-t border-border-default flex-shrink-0">
        <div className="flex items-center justify-between">
          <p className="text-xs text-text-muted font-medium">v{APP_VERSION}</p>
          <div className="w-2 h-2 rounded-full bg-accent-success shadow-[0_0_8px_rgba(34,197,94,0.5)]"></div>
        </div>
      </div>
    </aside>
  );
}
