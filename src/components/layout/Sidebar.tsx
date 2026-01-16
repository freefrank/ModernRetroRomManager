import { NavLink } from "react-router-dom";
import { Library, Database, ArrowRightLeft, Settings, Gamepad2 } from "lucide-react";
import { motion } from "framer-motion";
import { clsx } from "clsx";

const navItems = [
  {
    to: "/",
    label: "ROM 库",
    icon: Library,
  },
  {
    to: "/scraper",
    label: "Scraper",
    icon: Database,
  },
  {
    to: "/import",
    label: "导入/导出",
    icon: ArrowRightLeft,
  },
  {
    to: "/settings",
    label: "设置",
    icon: Settings,
  },
];

export default function Sidebar() {
  return (
    <aside className="w-64 flex flex-col bg-[#0B0C15]/95 backdrop-blur-xl border-r border-white/5 relative z-20 h-full">
      {/* Logo */}
      <div className="h-20 flex items-center px-6">
        <div className="flex items-center gap-3">
          <div className="relative">
            <div className="absolute inset-0 bg-accent-primary blur-md opacity-50 rounded-lg"></div>
            <div className="w-10 h-10 bg-gradient-to-br from-accent-primary to-accent-secondary rounded-lg flex items-center justify-center relative z-10 shadow-lg">
              <Gamepad2 className="text-white w-6 h-6" />
            </div>
          </div>
          <div className="flex flex-col">
            <span className="font-bold text-lg tracking-tight leading-none text-white">RetroRom</span>
            <span className="text-[10px] font-medium text-accent-secondary uppercase tracking-widest">Manager</span>
          </div>
        </div>
      </div>

      {/* Navigation */}
      <nav className="flex-1 px-4 py-6 space-y-2">
        {navItems.map((item) => (
          <NavLink
            key={item.to}
            to={item.to}
            className={({ isActive }) =>
              clsx(
                "relative flex items-center gap-3 px-4 py-3 rounded-xl transition-all duration-300 group overflow-hidden",
                isActive
                  ? "text-white"
                  : "text-text-secondary hover:text-white hover:bg-white/5"
              )
            }
          >
            {({ isActive }) => (
              <>
                {isActive && (
                  <motion.div
                    layoutId="activeNav"
                    className="absolute inset-0 bg-gradient-to-r from-accent-primary/20 to-transparent border-l-4 border-accent-primary"
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
                  {item.label}
                </span>
              </>
            )}
          </NavLink>
        ))}
      </nav>

      {/* Footer Info */}
      <div className="p-6 border-t border-white/5">
        <div className="flex items-center justify-between">
          <p className="text-xs text-text-muted font-medium">v0.1.0</p>
          <div className="w-2 h-2 rounded-full bg-accent-success shadow-[0_0_8px_rgba(34,197,94,0.5)]"></div>
        </div>
      </div>
    </aside>
  );
}
