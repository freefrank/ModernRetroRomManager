import { useEffect, useMemo } from "react";
import { useNavigate } from "react-router-dom";
import { Ghost, Plus } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useRomStore } from "@/stores/romStore";
import { Button, Card, EmptyState } from "@/components/ui";
import SystemCard from "@/components/rom/SystemCard";
import type { GameSystem } from "@/types";

const norm = (s: string | undefined) => (s ?? "").trim().toLowerCase();

/** 用 ROM 目录名匹配预置系统(get_systems),取其 logo 文件名 */
function findSystemLogo(systems: GameSystem[], folderName: string): string | undefined {
  const target = norm(folderName);
  if (!target) return undefined;
  const hit = systems.find(
    (s) => norm(s.id) === target || norm(s.shortName) === target || norm(s.name) === target
  );
  return hit?.logo;
}

/** 系统货架:每个系统一张卡片,点击进入单系统库页 */
export default function LibraryShelf() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const { availableSystems, stats, systems, fetchSystems } = useRomStore();

  // 装载预置系统列表以解析各系统 logo
  useEffect(() => {
    if (systems.length === 0) {
      fetchSystems();
    }
  }, [systems.length, fetchSystems]);

  const logoByName = useMemo(() => {
    const map = new Map<string, string | undefined>();
    for (const sys of availableSystems) {
      map.set(sys.name, findSystemLogo(systems, sys.name));
    }
    return map;
  }, [availableSystems, systems]);

  return (
    <div className="rr-page flex flex-col h-full max-w-[1600px] mx-auto w-full">
      {/* Header */}
      <header className="py-4">
        <h1 className="text-4xl font-bold tracking-tight text-text-primary mb-2">
          {t("library.title")}
        </h1>
        <p className="text-text-secondary font-medium">
          {t("library.shelf.systemCount", { count: availableSystems.length })}
          {" · "}
          {t("library.gameCount", { count: stats.totalRoms })}
        </p>
      </header>

      {availableSystems.length === 0 ? (
        /* 空态:库为空,引导去设置添加扫描目录 */
        <div className="flex-1 flex items-center justify-center">
          <EmptyState
            className="w-full max-w-md"
            icon={<Ghost />}
            title={t("library.empty.title")}
            description={t("library.empty.description")}
            action={
              <Button onClick={() => navigate("/settings?tab=general&addDirectory=1")}>
                <Plus className="w-4 h-4" />
                {t("library.empty.addDirectory")}
              </Button>
            }
          />
        </div>
      ) : (
        <div className="flex-1 min-h-0 overflow-y-auto custom-scrollbar pb-6">
          <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-4">
            {availableSystems.map((sys) => (
              <SystemCard
                key={sys.name}
                name={sys.name}
                romCount={sys.romCount}
                logoFile={logoByName.get(sys.name)}
                onClick={() => navigate(`/library/${encodeURIComponent(sys.name)}`)}
              />
            ))}

            {/* 「+ 添加目录」卡:直接打开添加目录界面 */}
            <Card
              role="button"
              tabIndex={0}
              onClick={() => navigate("/settings?tab=general&addDirectory=1")}
              onKeyDown={(e) => {
                if (e.key === "Enter" || e.key === " ") {
                  e.preventDefault();
                  navigate("/settings?tab=general&addDirectory=1");
                }
              }}
              className="cursor-pointer select-none border-dashed bg-transparent p-6 flex flex-col items-center justify-center gap-2 text-text-muted hover:text-text-primary hover:border-border-hover"
            >
              <Plus className="w-8 h-8" />
              <span className="text-sm font-medium">{t("library.shelf.addCard")}</span>
              <span className="text-xs">{t("library.shelf.addCardHint")}</span>
            </Card>
          </div>
        </div>
      )}
    </div>
  );
}
