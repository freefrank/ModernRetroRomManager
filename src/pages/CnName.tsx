import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Languages, Download, Loader2, Info, ExternalLink, FolderSearch, CheckCircle2 } from "lucide-react";
import { isTauri, api } from "@/lib/api";
import { clsx } from "clsx";

interface CheckResult {
  file: string;
  name: string;
  english_name?: string;
}

export default function CnName() {
  const { t } = useTranslation();
  const [isUpdating, setIsUpdating] = useState(false);
  const [checkPath, setCheckPath] = useState("");
  const [checkResults, setCheckResults] = useState<CheckResult[]>([]);
  const [isChecking, setIsChecking] = useState(false);

  const handleUpdate = async () => {
    setIsUpdating(true);
    try {
      if (isTauri()) {
        const { invoke } = await import("@tauri-apps/api/core");
        await invoke("update_cn_repo");
      }
      alert("数据库更新成功");
    } catch (error) {
      console.error("Failed to update CN repo:", error);
      alert("更新失败: " + String(error));
    } finally {
      setIsUpdating(false);
    }
  };

  const handleBrowse = async () => {
    if (!isTauri()) return;
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const selected = await open({
        directory: true,
        multiple: false,
      });
      if (selected && typeof selected === "string") {
        setCheckPath(selected);
      }
    } catch (error) {
      console.error("Failed to select directory:", error);
    }
  };

  const handleCheck = async () => {
    if (!checkPath) return;
    setIsChecking(true);
    try {
      const results = await api.scanDirectoryForNamingCheck(checkPath);
      setCheckResults(results);
    } catch (error) {
      console.error("Failed to check directory:", error);
    } finally {
      setIsChecking(false);
    }
  };

  return (
    <div className="flex flex-col h-full bg-bg-primary">
      {/* Header */}
      <div className="flex items-center justify-between px-6 py-4 border-b border-border-default bg-bg-primary/50 backdrop-blur-md sticky top-0 z-10">
        <div className="flex items-center gap-3">
          <Languages className="w-6 h-6 text-accent-primary" />
          <h1 className="text-xl font-bold text-text-primary tracking-tight">{t("nav.cnName", { defaultValue: "中文命名检查" })}</h1>
        </div>
      </div>

      <div className="flex-1 p-8 overflow-auto">
        <div className="max-w-4xl mx-auto space-y-8">
          
          {/* Info Card */}
          <section className="bg-bg-secondary rounded-2xl border border-border-default overflow-hidden">
            <div className="p-6 border-b border-border-default">
              <div className="flex items-center gap-3 mb-2">
                <Info className="w-5 h-5 text-accent-primary" />
                <h2 className="text-lg font-bold text-text-primary">关于本项目</h2>
              </div>
              <p className="text-sm text-text-secondary leading-relaxed">
                本软件集成了 <a href="https://github.com/yingw/rom-name-cn" target="_blank" rel="noopener noreferrer" className="text-accent-primary hover:underline font-bold">yingw/rom-name-cn</a> 项目的数据。
                该项目提供了极其详尽的 ROM 中英文名称对照表，涵盖了绝大多数主流复古游戏平台。通过模糊匹配算法，我们能够在抓取元数据时自动识别并匹配中文名称。
              </p>
            </div>
            <div className="bg-bg-tertiary/50 p-4 flex items-center justify-between">
              <div className="text-xs text-text-muted font-medium">
                数据来源: GitHub (yingw/rom-name-cn)
              </div>
              <div className="flex gap-4">
                <a 
                  href="https://github.com/yingw/rom-name-cn" 
                  target="_blank" 
                  rel="noopener noreferrer"
                  className="flex items-center gap-1.5 text-xs font-bold text-text-primary hover:text-accent-primary transition-colors"
                >
                  访问仓库 <ExternalLink className="w-3 h-3" />
                </a>
                <button
                  onClick={handleUpdate}
                  disabled={isUpdating}
                  className="flex items-center gap-1.5 text-xs font-bold text-accent-primary hover:text-accent-primary/80 transition-colors disabled:opacity-50"
                >
                  {isUpdating ? <Loader2 className="w-3 h-3 animate-spin" /> : <Download className="w-3 h-3" />}
                  {isUpdating ? "更新中..." : "更新数据库"}
                </button>
              </div>
            </div>
          </section>

          {/* Directory Check */}
          <section className="space-y-4">
            <h2 className="text-lg font-bold text-text-primary flex items-center gap-2">
              <FolderSearch className="w-5 h-5 text-accent-primary" />
              目录命名检查
            </h2>
            
            <div className="flex gap-3">
              <div className="flex-1 flex gap-2">
                <input 
                  type="text" 
                  value={checkPath}
                  readOnly
                  placeholder="选择要检查的 ROM 目录..."
                  className="flex-1 bg-bg-secondary border border-border-default rounded-xl px-4 py-3 text-sm text-text-primary focus:outline-none"
                />
                <button 
                  onClick={handleBrowse}
                  className="px-4 py-2 bg-bg-tertiary hover:bg-border-hover text-text-primary rounded-xl font-bold transition-all border border-border-default text-sm whitespace-nowrap"
                >
                  浏览...
                </button>
              </div>
              <button
                onClick={handleCheck}
                disabled={!checkPath || isChecking}
                className="px-6 py-2 bg-accent-primary text-bg-primary font-bold rounded-xl hover:opacity-90 active:scale-95 transition-all shadow-lg shadow-accent-primary/20 disabled:opacity-50 disabled:cursor-not-allowed text-sm flex items-center gap-2 whitespace-nowrap"
              >
                {isChecking ? <Loader2 className="w-4 h-4 animate-spin" /> : <CheckCircle2 className="w-4 h-4" />}
                开始检查
              </button>
            </div>

            {/* Results Table */}
            {checkResults.length > 0 && (
              <div className="bg-bg-secondary rounded-2xl border border-border-default overflow-hidden">
                <div className="overflow-x-auto">
                  <table className="w-full text-left text-sm">
                    <thead>
                      <tr className="bg-bg-tertiary/50 border-b border-border-default text-xs uppercase tracking-wider text-text-muted">
                        <th className="px-6 py-3 font-bold">文件名</th>
                        <th className="px-6 py-3 font-bold">中文名 (Metadata)</th>
                        <th className="px-6 py-3 font-bold">英文名 (Metadata)</th>
                      </tr>
                    </thead>
                    <tbody className="divide-y divide-border-default">
                      {checkResults.map((res, idx) => (
                        <tr key={idx} className="hover:bg-bg-tertiary/30 transition-colors">
                          <td className="px-6 py-3 text-text-primary font-mono text-xs">{res.file}</td>
                          <td className={clsx("px-6 py-3 font-medium", res.name ? "text-green-400" : "text-text-muted italic")}>
                            {res.name || "未设置"}
                          </td>
                          <td className={clsx("px-6 py-3 font-medium", res.english_name ? "text-blue-400" : "text-text-muted italic")}>
                            {res.english_name || "未设置"}
                          </td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
                <div className="px-6 py-3 bg-bg-tertiary/30 border-t border-border-default text-xs text-text-muted font-medium text-right">
                  共检查 {checkResults.length} 个文件
                </div>
              </div>
            )}
          </section>

        </div>
      </div>
    </div>
  );
}
