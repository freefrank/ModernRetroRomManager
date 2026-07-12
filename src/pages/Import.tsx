import { useTranslation } from "react-i18next";
import { useNavigate } from "react-router-dom";
import { Construction } from "lucide-react";
import { Button, EmptyState } from "@/components/ui";

/**
 * 导入/导出页
 *
 * 原导入(EmulationStation 等)与导出(export_to_emulationstation)入口
 * 依赖的后端命令均未实现(仅占位符),已下线;
 * 当前请使用中文 ROM 工具箱的导出能力。
 */
export default function Import() {
  const { t } = useTranslation();
  const navigate = useNavigate();

  return (
    <div className="rr-page flex flex-col h-full">
      {/* 工具栏 */}
      <div className="flex items-center justify-between px-6 py-4 border-b border-border-default bg-bg-primary/50 backdrop-blur-md sticky top-0 z-10">
        <h1 className="text-xl font-bold text-text-primary">{t("import.title")}</h1>
      </div>

      {/* 内容区 */}
      <div className="flex-1 p-6 overflow-auto">
        <div className="max-w-3xl">
          <EmptyState
            icon={<Construction />}
            title={t("import.underDevelopment.title")}
            description={t("import.underDevelopment.description")}
            action={
              <Button onClick={() => navigate("/cn-tools")}>
                {t("import.underDevelopment.goToCnTools")}
              </Button>
            }
          />
        </div>
      </div>
    </div>
  );
}
