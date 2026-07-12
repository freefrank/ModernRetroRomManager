import { NavLink, useLocation } from "react-router-dom";
import {
  Library,
  Database,
  ArrowRightLeft,
  Settings,
  Gamepad2,
  ChevronDown,
  Languages,
  PenTool,
  PanelLeftClose,
  PanelLeftOpen,
  type LucideIcon,
} from "lucide-react";
import { clsx } from "clsx";
import { useTranslation } from "react-i18next";
import { useState, type ReactNode } from "react";
import { useRomStore } from "@/stores/romStore";
import { useAppStore } from "@/stores/appStore";
import { Badge, IconButton, Tooltip } from "@/components/ui";

const TOOL_ITEMS = [{ to: "/cn-tools", icon: Languages, labelKey: "nav.cnTools" }];

/** 键盘可达:focus-visible 走 border-highlight 令牌的内嵌 outline */
const focusRingClass =
  "focus-visible:outline focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-border-highlight";

/** 一级导航行(≥40px、全行可点、hover 与选中反馈) */
const navRowClass = (active: boolean) =>
  clsx(
    "relative flex items-center gap-3 w-full min-h-10 px-3 rounded-[var(--radius-md)] group overflow-hidden",
    "transition-all duration-[var(--motion-normal)] ease-[var(--motion-easing)]",
    focusRingClass,
    active
      ? "text-text-primary"
      : "text-text-secondary hover:text-text-primary hover:bg-bg-tertiary",
  );

const navIconClass = (active: boolean) =>
  clsx(
    "w-5 h-5 shrink-0 z-10 transition-transform duration-[var(--motion-normal)] ease-[var(--motion-easing)]",
    active ? "text-accent-secondary scale-110" : "group-hover:scale-110",
  );

/** 激活态高亮条(border-l + 渐变 bg;不拦截点击) */
function ActiveIndicator() {
  return (
    <span className="pointer-events-none absolute inset-0 bg-gradient-to-r from-accent-primary/20 to-transparent border-l-4 border-accent-primary rounded-[var(--radius-md)]" />
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

/** chevron 独立命中区(32×32,点击只展开/收起、不导航) */
function TreeToggle({
  expanded,
  onToggle,
  label,
}: {
  expanded: boolean;
  onToggle: () => void;
  label: string;
}) {
  return (
    <button
      type="button"
      onClick={(e) => {
        e.stopPropagation();
        onToggle();
      }}
      aria-expanded={expanded}
      aria-label={label}
      className={clsx(
        "z-10 w-8 h-8 shrink-0 flex items-center justify-center rounded-[var(--radius-sm)]",
        "text-text-muted hover:text-text-primary hover:bg-bg-primary/50",
        "transition-colors duration-[var(--motion-fast)] ease-[var(--motion-easing)]",
        focusRingClass,
      )}
    >
      <Chevron expanded={expanded} />
    </button>
  );
}

/** 可折叠区块容器(grid-rows 过渡,受 data-motion 门控;子项缩进引导线) */
function Collapsible({ expanded, children }: { expanded: boolean; children: ReactNode }) {
  return (
    <div
      className={clsx(
        "grid transition-[grid-template-rows] duration-[var(--motion-normal)] ease-[var(--motion-easing)]",
        expanded ? "grid-rows-[1fr]" : "grid-rows-[0fr]",
      )}
    >
      <div className="min-h-0 overflow-hidden">
        <div className="ml-5 pl-3 border-l border-border-default space-y-1 py-1">{children}</div>
      </div>
    </div>
  );
}

/** 折叠窄栏时用 Tooltip 包裹图标项;展开时原样透传 */
function CollapsedTooltip({
  collapsed,
  label,
  children,
}: {
  collapsed: boolean;
  label: string;
  children: ReactNode;
}) {
  if (!collapsed) return <>{children}</>;
  return (
    <Tooltip content={label} side="right" className="w-full">
      {children}
    </Tooltip>
  );
}

/** 普通导航项(折叠时仅图标 + Tooltip) */
function NavItem({
  to,
  icon: Icon,
  label,
  collapsed,
}: {
  to: string;
  icon: LucideIcon;
  label: string;
  collapsed: boolean;
}) {
  return (
    <CollapsedTooltip collapsed={collapsed} label={label}>
      <NavLink
        to={to}
        className={({ isActive }) => clsx(navRowClass(isActive), collapsed && "justify-center px-0")}
      >
        {({ isActive }) => (
          <>
            {isActive && <ActiveIndicator />}
            <Icon className={navIconClass(isActive)} />
            {!collapsed && (
              <span className="font-medium z-10 text-sm tracking-wide truncate">{label}</span>
            )}
          </>
        )}
      </NavLink>
    </CollapsedTooltip>
  );
}

export default function Sidebar() {
  const { t } = useTranslation();
  const location = useLocation();
  const { availableSystems } = useRomStore();
  const { sidebarCollapsed, setSidebarCollapsed } = useAppStore();

  const [isLibraryExpanded, setIsLibraryExpanded] = useState(true);
  const [isToolsExpanded, setIsToolsExpanded] = useState(false);

  const isShelfActive = location.pathname === "/library";
  const isToolsActive = TOOL_ITEMS.some((item) => location.pathname === item.to);

  const toggleLabel = sidebarCollapsed ? t("nav.expandSidebar") : t("nav.collapseSidebar");

  return (
    <aside
      className={clsx(
        "rr-sidebar flex flex-col bg-bg-primary/95 backdrop-blur-xl border-r border-border-default relative z-20 h-full",
        "transition-[width] duration-[var(--motion-normal)] ease-[var(--motion-easing)]",
        sidebarCollapsed ? "w-14" : "w-60",
      )}
    >
      {/* Logo */}
      <div
        className={clsx(
          "h-20 flex items-center flex-shrink-0",
          sidebarCollapsed ? "justify-center px-0" : "px-5",
        )}
      >
        <div className="flex items-center gap-3">
          <div className="relative">
            <div className="absolute inset-0 bg-accent-primary blur-md opacity-50 rounded-[var(--radius-md)]"></div>
            <div className="w-10 h-10 bg-gradient-to-br from-accent-primary to-accent-secondary rounded-[var(--radius-md)] flex items-center justify-center relative z-10">
              <Gamepad2 className="text-text-primary w-6 h-6" />
            </div>
          </div>
          {!sidebarCollapsed && (
            <div className="flex flex-col">
              <span className="font-bold text-lg tracking-tight leading-none text-text-primary">
                RetroRom
              </span>
              <span className="text-[10px] font-medium text-accent-secondary uppercase tracking-widest">
                Manager
              </span>
            </div>
          )}
        </div>
      </div>

      {/* Navigation(折叠窄栏时不滚动,让 Tooltip 溢出可见) */}
      <nav
        className={clsx(
          "flex-1 py-6 space-y-2",
          sidebarCollapsed ? "px-2 overflow-visible" : "px-3 overflow-y-auto custom-scrollbar",
        )}
      >
        {/* ROM 库(可展开系统树;折叠窄栏时点图标直接导航 /library) */}
        <div className="space-y-1">
          <CollapsedTooltip collapsed={sidebarCollapsed} label={t("nav.library")}>
            <div
              className={clsx(
                "relative flex items-center min-h-10 rounded-[var(--radius-md)] group overflow-hidden",
                "transition-all duration-[var(--motion-normal)] ease-[var(--motion-easing)]",
                isShelfActive
                  ? "text-text-primary"
                  : "text-text-secondary hover:text-text-primary hover:bg-bg-tertiary",
              )}
            >
              {isShelfActive && <ActiveIndicator />}
              <NavLink
                to="/library"
                end
                className={clsx(
                  "flex-1 flex items-center gap-3 min-h-10 rounded-[var(--radius-md)]",
                  sidebarCollapsed ? "justify-center px-0" : "px-3",
                  focusRingClass,
                )}
              >
                <Library className={navIconClass(isShelfActive)} />
                {!sidebarCollapsed && (
                  <span className="font-medium z-10 text-sm tracking-wide truncate">
                    {t("nav.library")}
                  </span>
                )}
              </NavLink>
              {!sidebarCollapsed && (
                <TreeToggle
                  expanded={isLibraryExpanded}
                  onToggle={() => setIsLibraryExpanded(!isLibraryExpanded)}
                  label={isLibraryExpanded ? t("nav.collapse") : t("nav.expand")}
                />
              )}
            </div>
          </CollapsedTooltip>

          {!sidebarCollapsed && (
            <Collapsible expanded={isLibraryExpanded}>
              {availableSystems.map((sys) => (
                <NavLink
                  key={sys.name}
                  to={`/library/${encodeURIComponent(sys.name)}`}
                  className={({ isActive }) =>
                    clsx(
                      "relative w-full flex items-center gap-2.5 min-h-10 px-3 rounded-[var(--radius-sm)] text-sm group overflow-hidden",
                      "transition-all duration-[var(--motion-fast)] ease-[var(--motion-easing)]",
                      focusRingClass,
                      isActive
                        ? "text-text-primary bg-accent-primary/10 border-l-2 border-accent-primary"
                        : "border-l-2 border-transparent text-text-muted hover:text-text-primary hover:bg-bg-tertiary",
                    )
                  }
                >
                  {({ isActive }) => (
                    <>
                      <span
                        className={clsx(
                          "w-1.5 h-1.5 shrink-0 rounded-full transition-colors duration-[var(--motion-fast)] ease-[var(--motion-easing)]",
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
          )}
        </div>

        <NavItem
          to="/scraper"
          icon={Database}
          label={t("nav.scraper")}
          collapsed={sidebarCollapsed}
        />
        <NavItem
          to="/import"
          icon={ArrowRightLeft}
          label={t("nav.import")}
          collapsed={sidebarCollapsed}
        />

        {/* 工具箱:展开态为可折叠分组;折叠窄栏时子项平铺为图标 */}
        {sidebarCollapsed ? (
          TOOL_ITEMS.map((item) => (
            <NavItem
              key={item.to}
              to={item.to}
              icon={item.icon}
              label={t(item.labelKey)}
              collapsed
            />
          ))
        ) : (
          <div className="space-y-1">
            <button
              type="button"
              onClick={() => setIsToolsExpanded(!isToolsExpanded)}
              aria-expanded={isToolsExpanded}
              className={navRowClass(isToolsActive)}
            >
              <PenTool className={navIconClass(isToolsActive)} />
              <span className="font-medium z-10 text-sm tracking-wide flex-1 text-left truncate">
                {t("nav.tools")}
              </span>
              <span className="z-10 w-8 h-8 shrink-0 flex items-center justify-center text-text-muted">
                <Chevron expanded={isToolsExpanded} />
              </span>
            </button>

            <Collapsible expanded={isToolsExpanded}>
              {TOOL_ITEMS.map((item) => (
                <NavLink
                  key={item.to}
                  to={item.to}
                  className={({ isActive }) =>
                    clsx(
                      "relative flex items-center gap-2.5 min-h-10 px-3 rounded-[var(--radius-sm)] text-sm group overflow-hidden",
                      "transition-all duration-[var(--motion-fast)] ease-[var(--motion-easing)]",
                      focusRingClass,
                      isActive
                        ? "text-text-primary bg-accent-primary/10 border-l-2 border-accent-primary"
                        : "border-l-2 border-transparent text-text-muted hover:text-text-primary hover:bg-bg-tertiary",
                    )
                  }
                >
                  {({ isActive }) => (
                    <>
                      <item.icon
                        className={clsx(
                          "w-4 h-4 shrink-0 z-10 transition-transform duration-[var(--motion-normal)] ease-[var(--motion-easing)]",
                          isActive ? "text-accent-secondary scale-110" : "group-hover:scale-110",
                        )}
                      />
                      <span className="font-medium z-10 tracking-wide truncate">
                        {t(item.labelKey)}
                      </span>
                    </>
                  )}
                </NavLink>
              ))}
            </Collapsible>
          </div>
        )}

        <NavItem
          to="/settings"
          icon={Settings}
          label={t("nav.settings")}
          collapsed={sidebarCollapsed}
        />
      </nav>

      {/* Footer:折叠切换按钮 + 版本信息 */}
      <div
        className={clsx(
          "border-t border-border-default flex-shrink-0",
          sidebarCollapsed ? "p-2 flex justify-center" : "p-4",
        )}
      >
        {sidebarCollapsed ? (
          <Tooltip content={toggleLabel} side="right">
            <IconButton
              size="sm"
              aria-label={toggleLabel}
              aria-expanded={false}
              onClick={() => setSidebarCollapsed(false)}
            >
              <PanelLeftOpen className="w-4 h-4" />
            </IconButton>
          </Tooltip>
        ) : (
          <div className="flex items-center justify-between gap-2">
            <div className="flex items-center gap-2 min-w-0">
              <span className="w-2 h-2 shrink-0 rounded-full bg-accent-success [box-shadow:0_0_8px_var(--accent-success)]"></span>
              <p className="text-xs text-text-muted font-medium truncate">v{APP_VERSION}</p>
            </div>
            <IconButton
              size="sm"
              aria-label={toggleLabel}
              aria-expanded={true}
              onClick={() => setSidebarCollapsed(true)}
            >
              <PanelLeftClose className="w-4 h-4" />
            </IconButton>
          </div>
        )}
      </div>
    </aside>
  );
}
