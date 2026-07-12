import { NavLink, useLocation, useNavigate } from "react-router-dom";
import {
  Library,
  Database,
  ArrowRightLeft,
  Settings,
  Gamepad2,
  ChevronDown,
  Languages,
  PenTool,
  type LucideIcon,
} from "lucide-react";
import { clsx } from "clsx";
import { useTranslation } from "react-i18next";
import { useState, type ReactNode } from "react";
import { useRomStore } from "@/stores/romStore";
import { Badge } from "@/components/ui";

const TOOL_ITEMS = [{ to: "/cn-tools", icon: Languages, labelKey: "nav.cnTools" }];

const navRowClass = (active: boolean) =>
  clsx(
    "relative flex items-center gap-3 px-4 py-3 rounded-[var(--radius-md)] group overflow-hidden",
    "transition-all duration-[var(--motion-normal)] ease-[var(--motion-easing)]",
    active
      ? "text-text-primary"
      : "text-text-secondary hover:text-text-primary hover:bg-bg-tertiary",
  );

const navIconClass = (active: boolean) =>
  clsx(
    "w-5 h-5 z-10 transition-transform duration-[var(--motion-normal)] ease-[var(--motion-easing)]",
    active ? "text-accent-secondary scale-110" : "group-hover:scale-110",
  );

/** 激活态高亮条 */
function ActiveIndicator() {
  return (
    <span className="absolute inset-0 bg-gradient-to-r from-accent-primary/20 to-transparent border-l-4 border-accent-primary rounded-[var(--radius-md)]" />
  );
}

/** 展开/收起 chevron,旋转过渡走 motion 令牌 */
function Chevron({ expanded }: { expanded: boolean }) {
  return (
    <ChevronDown
      className={clsx(
        "w-4 h-4 transition-transform duration-[var(--motion-normal)] ease-[var(--motion-easing)]",
        !expanded && "-rotate-90",
      )}
    />
  );
}

/** 可折叠区块容器(grid-rows 过渡,受 data-motion 门控) */
function Collapsible({ expanded, children }: { expanded: boolean; children: ReactNode }) {
  return (
    <div
      className={clsx(
        "grid transition-[grid-template-rows] duration-[var(--motion-normal)] ease-[var(--motion-easing)]",
        expanded ? "grid-rows-[1fr]" : "grid-rows-[0fr]",
      )}
    >
      <div className="min-h-0 overflow-hidden">
        <div className="ml-4 pl-4 border-l border-border-default space-y-1 py-1">{children}</div>
      </div>
    </div>
  );
}

/** 普通导航项(Scraper / 导入 / 设置) */
function NavItem({ to, icon: Icon, label }: { to: string; icon: LucideIcon; label: string }) {
  return (
    <NavLink to={to} className={({ isActive }) => navRowClass(isActive)}>
      {({ isActive }) => (
        <>
          {isActive && <ActiveIndicator />}
          <Icon className={navIconClass(isActive)} />
          <span className="font-medium z-10 text-sm tracking-wide">{label}</span>
        </>
      )}
    </NavLink>
  );
}

export default function Sidebar() {
  const { t } = useTranslation();
  const location = useLocation();
  const navigate = useNavigate();
  const { availableSystems } = useRomStore();

  const [isLibraryExpanded, setIsLibraryExpanded] = useState(true);
  const [isToolsExpanded, setIsToolsExpanded] = useState(false);

  const isShelfActive = location.pathname === "/library";
  const isToolsActive = TOOL_ITEMS.some((item) => location.pathname === item.to);

  return (
    <aside className="rr-sidebar w-64 flex flex-col bg-bg-primary/95 backdrop-blur-xl border-r border-border-default relative z-20 h-full">
      {/* Logo */}
      <div className="h-20 flex items-center px-6 flex-shrink-0">
        <div className="flex items-center gap-3">
          <div className="relative">
            <div className="absolute inset-0 bg-accent-primary blur-md opacity-50 rounded-[var(--radius-md)]"></div>
            <div className="w-10 h-10 bg-gradient-to-br from-accent-primary to-accent-secondary rounded-[var(--radius-md)] flex items-center justify-center relative z-10">
              <Gamepad2 className="text-text-primary w-6 h-6" />
            </div>
          </div>
          <div className="flex flex-col">
            <span className="font-bold text-lg tracking-tight leading-none text-text-primary">
              RetroRom
            </span>
            <span className="text-[10px] font-medium text-accent-secondary uppercase tracking-widest">
              Manager
            </span>
          </div>
        </div>
      </div>

      {/* Navigation */}
      <nav className="flex-1 px-4 py-6 space-y-2 overflow-y-auto custom-scrollbar">
        {/* ROM 库(可展开系统树) */}
        <div className="space-y-1">
          <div
            className={clsx(navRowClass(isShelfActive), "cursor-pointer select-none")}
            onClick={() => navigate("/library")}
          >
            {isShelfActive && <ActiveIndicator />}
            <Library className={navIconClass(isShelfActive)} />
            <span className="font-medium z-10 text-sm tracking-wide flex-1">
              {t("nav.library")}
            </span>
            <button
              onClick={(e) => {
                e.stopPropagation();
                setIsLibraryExpanded(!isLibraryExpanded);
              }}
              aria-label={isLibraryExpanded ? t("nav.collapse") : t("nav.expand")}
              className="z-10 p-1 rounded-[var(--radius-sm)] hover:bg-bg-primary/50 text-text-muted hover:text-text-primary transition-colors duration-[var(--motion-fast)] ease-[var(--motion-easing)]"
            >
              <Chevron expanded={isLibraryExpanded} />
            </button>
          </div>

          <Collapsible expanded={isLibraryExpanded}>
            {availableSystems.map((sys) => (
              <NavLink
                key={sys.name}
                to={`/library/${encodeURIComponent(sys.name)}`}
                className={({ isActive }) =>
                  clsx(
                    "w-full flex items-center gap-3 px-4 py-2 rounded-[var(--radius-sm)] text-sm group",
                    "transition-all duration-[var(--motion-fast)] ease-[var(--motion-easing)]",
                    isActive
                      ? "text-text-primary bg-accent-primary/10"
                      : "text-text-muted hover:text-text-primary hover:bg-bg-tertiary",
                  )
                }
              >
                {({ isActive }) => (
                  <>
                    <span
                      className={clsx(
                        "w-1.5 h-1.5 rounded-full transition-colors duration-[var(--motion-fast)] ease-[var(--motion-easing)]",
                        isActive
                          ? "bg-accent-secondary"
                          : "bg-border-default group-hover:bg-text-secondary",
                      )}
                    ></span>
                    <span className="truncate flex-1 text-left">{sys.name}</span>
                    <Badge>{sys.romCount}</Badge>
                  </>
                )}
              </NavLink>
            ))}
          </Collapsible>
        </div>

        <NavItem to="/scraper" icon={Database} label={t("nav.scraper")} />
        <NavItem to="/import" icon={ArrowRightLeft} label={t("nav.import")} />

        {/* 工具箱(可展开) */}
        <div className="space-y-1">
          <div
            className={clsx(navRowClass(isToolsActive), "cursor-pointer select-none")}
            onClick={() => setIsToolsExpanded(!isToolsExpanded)}
          >
            <PenTool className={navIconClass(isToolsActive)} />
            <span className="font-medium z-10 text-sm tracking-wide flex-1">{t("nav.tools")}</span>
            <span className="z-10 p-1 text-text-muted">
              <Chevron expanded={isToolsExpanded} />
            </span>
          </div>

          <Collapsible expanded={isToolsExpanded}>
            {TOOL_ITEMS.map((item) => (
              <NavLink
                key={item.to}
                to={item.to}
                className={({ isActive }) =>
                  clsx(
                    "relative flex items-center gap-3 px-4 py-2 rounded-[var(--radius-sm)] text-sm group overflow-hidden",
                    "transition-all duration-[var(--motion-fast)] ease-[var(--motion-easing)]",
                    isActive
                      ? "text-text-primary bg-accent-primary/10"
                      : "text-text-muted hover:text-text-primary hover:bg-bg-tertiary",
                  )
                }
              >
                {({ isActive }) => (
                  <>
                    <item.icon
                      className={clsx(
                        "w-4 h-4 z-10 transition-transform duration-[var(--motion-normal)] ease-[var(--motion-easing)]",
                        isActive ? "text-accent-secondary scale-110" : "group-hover:scale-110",
                      )}
                    />
                    <span className="font-medium z-10 tracking-wide">{t(item.labelKey)}</span>
                  </>
                )}
              </NavLink>
            ))}
          </Collapsible>
        </div>

        <NavItem to="/settings" icon={Settings} label={t("nav.settings")} />
      </nav>

      {/* Footer Info */}
      <div className="p-6 border-t border-border-default flex-shrink-0">
        <div className="flex items-center justify-between">
          <p className="text-xs text-text-muted font-medium">v{APP_VERSION}</p>
          <div className="w-2 h-2 rounded-full bg-accent-success [box-shadow:0_0_8px_var(--accent-success)]"></div>
        </div>
      </div>
    </aside>
  );
}
