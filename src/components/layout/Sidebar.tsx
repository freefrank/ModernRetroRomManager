import { NavLink, useLocation } from "react-router-dom";
import {
  Library,
  Database,
  ArrowRightLeft,
  Settings,
  Gamepad2,
  PenTool,
  PanelLeftClose,
  PanelLeftOpen,
  type LucideIcon,
} from "lucide-react";
import { clsx } from "clsx";
import { useTranslation } from "react-i18next";
import { useAppStore } from "@/stores/appStore";
import { Tooltip } from "@/components/ui";
import SidebarPanel, {
  TOOL_ITEMS,
  sidebarFocusRing,
  type SidebarSection,
} from "./SidebarPanel";

/**
 * rail 导航项:44×44 居中命中区 + 右侧 Tooltip。
 * active = 10% accent 圆角填充块 + 左侧 2px accent 条(定位到 rail 左缘)。
 * `active` 为可选覆盖(工具箱 section 需匹配任意子项路由);
 * 库项不传覆盖,NavLink 默认前缀匹配即含 /library/:systemId 子路由。
 */
function RailItem({
  to,
  icon: Icon,
  label,
  active,
}: {
  to: string;
  icon: LucideIcon;
  label: string;
  active?: boolean;
}) {
  return (
    <Tooltip content={label} side="right" className="w-full justify-center">
      <NavLink
        to={to}
        className={({ isActive }) =>
          clsx(
            "relative flex items-center justify-center size-11 rounded-[var(--radius-md)]",
            "transition-colors duration-[var(--motion-fast)] ease-[var(--motion-easing)]",
            sidebarFocusRing,
            (active ?? isActive)
              ? "bg-accent-primary/10 text-accent-secondary"
              : "text-text-secondary hover:text-text-primary hover:bg-bg-tertiary",
          )
        }
      >
        {({ isActive }) => (
          <>
            {(active ?? isActive) && (
              <span className="pointer-events-none absolute -left-0.5 top-1/2 -translate-y-1/2 h-5 w-0.5 bg-accent-primary" />
            )}
            <Icon className="w-5 h-5" />
          </>
        )}
      </NavLink>
    </Tooltip>
  );
}

/** 双栏侧边栏:48px 常驻 icon rail + 200px 上下文面板(仅库/工具箱 section 出现) */
export default function Sidebar() {
  const { t } = useTranslation();
  const location = useLocation();
  const { sidebarPanelCollapsed, setSidebarPanelCollapsed } = useAppStore();

  const isToolsSection = TOOL_ITEMS.some((item) => location.pathname.startsWith(item.to));
  const section: SidebarSection = location.pathname.startsWith("/library")
    ? "library"
    : isToolsSection
      ? "tools"
      : null;

  const panelOpen = section !== null && !sidebarPanelCollapsed;
  const toggleLabel = sidebarPanelCollapsed ? t("nav.expandPanel") : t("nav.collapsePanel");

  return (
    <aside className="relative z-20 flex h-full">
      {/* 48px icon rail */}
      <div className="rr-sidebar w-12 shrink-0 h-full flex flex-col items-center bg-bg-primary/95 backdrop-blur-xl">
        {/* Logo 小图标 */}
        <div className="h-12 shrink-0 flex items-center justify-center">
          <div className="size-8 bg-gradient-to-br from-accent-primary to-accent-secondary rounded-[var(--radius-md)] flex items-center justify-center">
            <Gamepad2 className="w-5 h-5 text-text-primary" />
          </div>
        </div>

        <nav className="flex-1 w-full flex flex-col items-center gap-1 py-2">
          <RailItem to="/library" icon={Library} label={t("nav.library")} />
          <RailItem to="/scraper" icon={Database} label={t("nav.scraper")} />
          <RailItem to="/import" icon={ArrowRightLeft} label={t("nav.import")} />
          <RailItem
            to={TOOL_ITEMS[0].to}
            icon={PenTool}
            label={t("nav.tools")}
            active={isToolsSection}
          />
          <RailItem to="/settings" icon={Settings} label={t("nav.settings")} />
        </nav>

        {/* 收起/展开面板(仅当前 section 有面板时出现) */}
        {section !== null && (
          <div className="shrink-0 pb-2 w-full flex justify-center">
            <Tooltip content={toggleLabel} side="right">
              <button
                type="button"
                onClick={() => setSidebarPanelCollapsed(!sidebarPanelCollapsed)}
                aria-expanded={panelOpen}
                aria-label={toggleLabel}
                className={clsx(
                  "flex items-center justify-center size-11 rounded-[var(--radius-md)]",
                  "text-text-muted hover:text-text-primary hover:bg-bg-tertiary",
                  "transition-colors duration-[var(--motion-fast)] ease-[var(--motion-easing)]",
                  sidebarFocusRing,
                )}
              >
                {sidebarPanelCollapsed ? (
                  <PanelLeftOpen className="w-5 h-5" />
                ) : (
                  <PanelLeftClose className="w-5 h-5" />
                )}
              </button>
            </Tooltip>
          </div>
        )}
      </div>

      {/* 200px 上下文面板 */}
      <SidebarPanel section={section} open={!sidebarPanelCollapsed} />
    </aside>
  );
}
