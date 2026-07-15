import { useMemo, useState } from "react";
import { NavLink, useNavigate } from "react-router-dom";
import { clsx } from "clsx";
import { useTranslation } from "react-i18next";
import { useRomStore } from "@/stores/romStore";
import { Input, Select, toast } from "@/components/ui";

/** 工具箱子项(rail 的 section 判定与面板列表共用) */
export const TOOL_ITEMS: { to: string; labelKey: string }[] = [
  { to: "/cn-tools", labelKey: "nav.cnTools" },
];

/** 当前路由所属的侧边栏 section;null = 无上下文面板 */
export type SidebarSection = "library" | "tools" | null;

/** 键盘可达:focus-visible 走 border-highlight 令牌的内嵌 outline */
export const sidebarFocusRing =
  "focus-visible:outline focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-border-highlight";

/** 面板导航行(36px、名称左对齐;active = 2px 左 accent 条 + accent 8% 底色) */
const panelRowClass = (active: boolean) =>
  clsx(
    "group flex items-center gap-2 h-9 px-2.5 text-sm rounded-[var(--radius-sm)] border-l-2",
    "transition-colors duration-[var(--motion-fast)] ease-[var(--motion-easing)]",
    sidebarFocusRing,
    active
      ? "border-accent-primary bg-accent-primary/8 text-text-primary"
      : "border-transparent text-text-secondary hover:text-text-primary hover:bg-bg-tertiary",
  );

/* 面板小标题:12px 下像素字与中文混排基线不协调,强制走正文字体(令牌) */
const panelTitleClass =
  "text-xs font-semibold text-text-muted tracking-wide font-[family-name:var(--font-body)]";

/** 库 section 面板:系统树(标题 + 实时搜索过滤 + 系统行) */
function LibraryPanel() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const {
    availableSystems,
    scanDirectories,
    activateLibrary,
    isBatchScraping,
  } = useRomStore();
  const [query, setQuery] = useState("");
  const activeLibrary = scanDirectories.find(item => item.isActive);

  const handleLibraryChange = async (libraryId: string) => {
    if (!libraryId || libraryId === activeLibrary?.id) return;
    navigate("/library");
    setQuery("");
    try {
      await activateLibrary(libraryId);
    } catch (error) {
      toast.error(t("nav.librarySwitchFailed", { error: String(error) }));
    }
  };

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!q) return availableSystems;
    return availableSystems.filter((sys) => sys.name.toLowerCase().includes(q));
  }, [availableSystems, query]);

  return (
    <div className="w-50 h-full flex flex-col">
      <div className="px-3 pt-4 pb-2 space-y-2 shrink-0">
        <Select
          value={activeLibrary?.id ?? ""}
          onChange={(event) => void handleLibraryChange(event.target.value)}
          disabled={scanDirectories.length === 0 || isBatchScraping}
          aria-label={t("nav.selectLibrary")}
          title={activeLibrary?.path}
          className="h-8 px-2 font-semibold"
        >
          {scanDirectories.length === 0 && <option value="">{t("nav.library")}</option>}
          {scanDirectories.map((library) => (
            <option key={library.id} value={library.id}>{library.name}</option>
          ))}
        </Select>
        <Input
          type="search"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder={t("nav.searchSystems")}
          aria-label={t("nav.searchSystems")}
        />
      </div>
      <nav className="flex-1 overflow-y-auto custom-scrollbar px-2 pb-3 space-y-0.5">
        {filtered.map((sys) => (
          <NavLink
            key={sys.name}
            to={`/library/${encodeURIComponent(sys.name)}`}
            className={({ isActive }) => panelRowClass(isActive)}
          >
            {({ isActive }) => (
              <>
                <span className="flex-1 truncate text-left">{sys.name}</span>
                <span
                  className={clsx(
                    "text-xs tabular-nums transition-colors duration-[var(--motion-fast)] ease-[var(--motion-easing)]",
                    isActive
                      ? "text-accent-secondary"
                      : "text-text-muted group-hover:text-text-secondary",
                  )}
                >
                  {sys.romCount}
                </span>
              </>
            )}
          </NavLink>
        ))}
        {query.trim() !== "" && filtered.length === 0 && (
          <p className="px-2.5 py-2 text-xs text-text-muted">{t("nav.noMatchingSystems")}</p>
        )}
      </nav>
    </div>
  );
}

/** 工具箱 section 面板:工具子项列表(与系统行同一套 36px 行样式) */
function ToolsPanel() {
  const { t } = useTranslation();
  return (
    <div className="w-50 h-full flex flex-col">
      <div className="px-3 pt-4 pb-2 shrink-0">
        <h2 className={panelTitleClass}>{t("nav.tools")}</h2>
      </div>
      <nav className="flex-1 overflow-y-auto custom-scrollbar px-2 pb-3 space-y-0.5">
        {TOOL_ITEMS.map((item) => (
          <NavLink
            key={item.to}
            to={item.to}
            className={({ isActive }) => panelRowClass(isActive)}
          >
            <span className="flex-1 truncate text-left">{t(item.labelKey)}</span>
          </NavLink>
        ))}
      </nav>
    </div>
  );
}

/**
 * 200px 上下文面板:仅当前 section 需要时展开,收起态宽度 0。
 * 展开/收起动画只动宽度(motion 令牌);收起时 inert 移出焦点链与无障碍树。
 */
export default function SidebarPanel({
  section,
  open,
}: {
  section: SidebarSection;
  open: boolean;
}) {
  const visible = open && section !== null;
  return (
    <div
      className={clsx(
        "rr-sidebar-panel h-full overflow-hidden bg-bg-secondary/40 backdrop-blur-xl",
        "border-r border-border-default",
        "transition-[width] duration-[var(--motion-normal)] ease-[var(--motion-easing)]",
        visible ? "w-50" : "w-0",
      )}
      inert={!visible}
      aria-hidden={!visible}
    >
      {section === "library" && <LibraryPanel />}
      {section === "tools" && <ToolsPanel />}
    </div>
  );
}
