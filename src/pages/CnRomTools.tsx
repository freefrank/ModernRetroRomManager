import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Languages, Download, Loader2, Info, ExternalLink, FolderSearch, RefreshCw, Wrench, AlertTriangle } from "lucide-react";
import { isTauri, api } from "@/lib/api";
import { clsx } from "clsx";

interface CheckResult {
  file: string;
  name: string;
  english_name?: string;
  extracted_cn_name?: string;
}

export default function CnName() {
  const { t } = useTranslation();
  const [isUpdating, setIsUpdating] = useState(false);
  const [checkPath, setCheckPath] = useState("");
  const [checkResults, setCheckResults] = useState<CheckResult[]>([]);
  const [isChecking, setIsChecking] = useState(false);
  const [isFixing, setIsFixing] = useState(false);

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
        // 清空之前的结果并自动扫描
        setCheckResults([]);
        // 自动触发扫描
        setIsChecking(true);
        try {
          const results = await api.scanDirectoryForNamingCheck(selected);
          setCheckResults(results);
        } catch (error) {
          console.error("Failed to check directory:", error);
        } finally {
          setIsChecking(false);
        }
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

  const handleAutoFix = async () => {
    if (!checkPath) return;
    
    // 简单的确认对话框 (生产环境建议用 Modal)
    if (!confirm(`确定要尝试自动修复吗？\n这将扫描该目录下的所有 ROM，并根据本地数据库自动匹配和写入中文名。\n仅置信度极高 (>95%) 的匹配会被应用。`)) {
      return;
    }

    setIsFixing(true);
    try {
      const result = await api.autoFixNaming(checkPath);
      alert(`修复完成！\n成功: ${result.success}\n失败/跳过: ${result.failed}`);
      // 重新扫描以显示最新状态
      handleCheck();
    } catch (error) {
      console.error("Failed to auto fix:", error);
      alert("自动修复失败: " + String(error));
    } finally {
      setIsFixing(false);
    }
  };

  const missingCount = checkResults.filter(r => !r.english_name).length;

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
                disabled={!checkPath || isChecking || isFixing}
                className="px-6 py-2 bg-accent-primary text-bg-primary font-bold rounded-xl hover:opacity-90 active:scale-95 transition-all shadow-lg shadow-accent-primary/20 disabled:opacity-50 disabled:cursor-not-allowed text-sm flex items-center gap-2 whitespace-nowrap"
              >
                {isChecking ? <Loader2 className="w-4 h-4 animate-spin" /> : <RefreshCw className="w-4 h-4" />}
                刷新
              </button>
            </div>

            {/* Results Table */}
            {checkResults.length > 0 && (
              <div className="bg-bg-secondary rounded-2xl border border-border-default overflow-hidden animate-in fade-in slide-in-from-bottom-4 duration-500">
                <div className="overflow-x-auto max-h-[600px] custom-scrollbar">
                  <table className="w-full text-left text-sm table-fixed">
                    <thead className="sticky top-0 z-10">
                      <tr className="bg-bg-tertiary border-b border-border-default text-xs uppercase tracking-wider text-text-muted">
                        <th className="w-1/4 px-6 py-3 font-bold">文件名</th>
                        <th className="w-1/4 px-6 py-3 font-bold">ROM名 (Meta)</th>
                        <th className="w-1/4 px-6 py-3 font-bold">提取中文名</th>
                        <th className="w-1/4 px-6 py-3 font-bold">匹配英文名</th>
                      </tr>
                    </thead>
                    <tbody className="divide-y divide-border-default">
                      {checkResults.map((res, idx) => {
                        // 逻辑调整:
                        // 1. ROM名(Meta) = res.name (从 metadata 读取)
                        // 2. 提取中文名 = res.extracted_cn_name (从文件名解析)
                        // 3. 英文名 = res.english_name (从 metadata 读取，或即将写入的)
                        
                        // 警告条件：没有英文名，或者 ROM 名和文件名完全一样(说明没匹配过)
                        const isMissing = !res.english_name;
                        
                        return (
                          <tr key={idx} className={clsx("hover:bg-bg-tertiary/30 transition-colors", isMissing && "bg-red-500/5")}>
                            <td className="px-6 py-3 text-text-primary font-mono text-xs max-w-xs truncate" title={res.file}>{res.file}</td>
                            
                            {/* ROM名 (Meta) */}
                            <td className={clsx("px-6 py-3 font-medium max-w-xs truncate", res.name && res.name !== res.file ? "text-text-primary" : "text-text-muted italic")}>
                              {res.name === res.file ? "未设置" : res.name}
                            </td>

                            {/* 提取中文名 */}
                            <td className={clsx("px-6 py-3 font-medium max-w-xs truncate", res.extracted_cn_name ? "text-green-400" : "text-text-muted italic")}>
                              {res.extracted_cn_name || "-"}
                            </td>

                            {/* 英文名 */}
                            <td className={clsx("px-6 py-3 font-medium max-w-xs truncate", res.english_name ? "text-blue-400" : "text-text-muted italic")}>
                              {res.english_name || "未匹配"}
                            </td>
                          </tr>
                        );
                      })}
                    </tbody>
                  </table>
                </div>
                
                <div className="px-6 py-4 bg-bg-tertiary/30 border-t border-border-default flex items-center justify-between">
                  <div className="flex items-center gap-4 text-xs text-text-muted font-medium">
                    <span>共 {checkResults.length} 个文件</span>
                    {missingCount > 0 && (
                      <span className="text-red-400 flex items-center gap-1.5">
                        <AlertTriangle className="w-3.5 h-3.5" />
                        {missingCount} 个可能缺失中文名
                      </span>
                    )}
                  </div>

                  {missingCount > 0 && (
                    <button
                      onClick={handleAutoFix}
                      disabled={isFixing}
                      className="flex items-center gap-2 px-4 py-2 bg-blue-500 text-white font-bold rounded-lg hover:bg-blue-600 transition-all shadow-lg shadow-blue-500/20 text-xs disabled:opacity-50"
                    >
                      {isFixing ? <Loader2 className="w-3.5 h-3.5 animate-spin" /> : <Wrench className="w-3.5 h-3.5" />}
                      一键自动修复
                    </button>
                  )}
                </div>
              </div>
            )}
          </section>

        </div>
      </div>
    </div>
  );
}
