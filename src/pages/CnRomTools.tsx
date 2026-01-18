import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Languages, Download, Loader2, Info, ExternalLink, FolderSearch, RefreshCw, Search, Tag, FileText, AlertTriangle, FileDown } from "lucide-react";
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
  const [isSettingName, setIsSettingName] = useState(false);
  const [isAddingTag, setIsAddingTag] = useState(false);
  const [isExporting, setIsExporting] = useState(false);
  const [showExportMenu, setShowExportMenu] = useState(false);

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
    
    if (!confirm(`确定要匹配英文名吗？\n这将扫描该目录下的所有 ROM，并根据本地数据库自动匹配英文名。\n仅置信度极高 (>95%) 的匹配会被应用。\n结果将保存到临时目录。`)) {
      return;
    }

    setIsFixing(true);
    try {
      const result = await api.autoFixNaming(checkPath);
      alert(`匹配完成！\n成功: ${result.success}\n失败/跳过: ${result.failed}`);
      // 重新扫描以显示最新状态
      handleCheck();
    } catch (error) {
      console.error("Failed to auto fix:", error);
      alert("匹配失败: " + String(error));
    } finally {
      setIsFixing(false);
    }
  };

  const handleSetAsRomName = async () => {
    if (!checkPath) return;
    
    if (!confirm(`确定要将提取的中文名设置为 ROM 名吗？\n这将把从文件名提取的中文名写入到临时 metadata 中。`)) {
      return;
    }

    setIsSettingName(true);
    try {
      if (isTauri()) {
        const { invoke } = await import("@tauri-apps/api/core");
        await invoke("set_extracted_cn_as_name", { directory: checkPath });
      }
      alert("设置完成！");
      handleCheck();
    } catch (error) {
      console.error("Failed to set ROM name:", error);
      alert("设置失败: " + String(error));
    } finally {
      setIsSettingName(false);
    }
  };

  const handleAddAsTag = async () => {
    if (!checkPath) return;
    
    if (!confirm(`确定要将英文名添加为额外 tag 吗？\n这将把匹配到的英文名作为 x-mrrm-eng tag 写入临时 metadata。`)) {
      return;
    }

    setIsAddingTag(true);
    try {
      if (isTauri()) {
        const { invoke } = await import("@tauri-apps/api/core");
        await invoke("add_english_as_tag", { directory: checkPath });
      }
      alert("添加完成！");
      handleCheck();
    } catch (error) {
      console.error("Failed to add tag:", error);
      alert("添加失败: " + String(error));
    } finally {
      setIsAddingTag(false);
    }
  };

  const handleExport = async (format: "pegasus" | "gamelist") => {
    if (!checkPath) return;
    setShowExportMenu(false);
    
    try {
      if (!isTauri()) return;
      
      const { save } = await import("@tauri-apps/plugin-dialog");
      const defaultName = format === "pegasus" ? "metadata.pegasus.txt" : "gamelist.xml";
      
      const savePath = await save({
        defaultPath: checkPath + "/" + defaultName,
        filters: format === "pegasus" 
          ? [{ name: "Pegasus Metadata", extensions: ["txt"] }]
          : [{ name: "EmulationStation Gamelist", extensions: ["xml"] }],
      });
      
      if (!savePath) return;
      
      setIsExporting(true);
      const { invoke } = await import("@tauri-apps/api/core");
      await invoke("export_cn_metadata", { 
        directory: checkPath,
        targetPath: savePath,
        format 
      });
      alert(`导出成功！\n文件已保存到: ${savePath}`);
    } catch (error) {
      console.error("Failed to export:", error);
      alert("导出失败: " + String(error));
    } finally {
      setIsExporting(false);
    }
  };

  const missingCount = checkResults.filter(r => !r.english_name).length;

  return (
    <div className="flex flex-col h-full bg-bg-primary">
      {/* Header */}
      <div className="shrink-0 flex items-center justify-between px-6 py-4 border-b border-border-default bg-bg-primary/50 backdrop-blur-md">
        <div className="flex items-center gap-3">
          <Languages className="w-6 h-6 text-accent-primary" />
          <h1 className="text-xl font-bold text-text-primary tracking-tight">{t("nav.cnName", { defaultValue: "中文命名检查" })}</h1>
        </div>
      </div>

      {/* Content - flex-1 with min-h-0 to allow shrinking */}
      <div className="flex-1 min-h-0 flex flex-col p-6 gap-6 overflow-hidden">
        
        {/* Info Card - shrink-0 to maintain size */}
        <section className="shrink-0 bg-bg-secondary rounded-2xl border border-border-default overflow-hidden">
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

        {/* Directory Check - flex-1 to fill remaining space */}
        <section className="flex-1 min-h-0 flex flex-col gap-4">
          <h2 className="shrink-0 text-lg font-bold text-text-primary flex items-center gap-2">
            <FolderSearch className="w-5 h-5 text-accent-primary" />
            {checkPath ? (
              // 从路径提取目录名作为系统名
              checkPath.split(/[/\\]/).filter(Boolean).pop() || "选择平台"
            ) : (
              "选择平台"
            )}
          </h2>
          
          <div className="shrink-0 flex gap-3">
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

          {/* Results Table - flex-1 to fill remaining space */}
          {checkResults.length > 0 && (
            <div className="flex-1 min-h-0 flex flex-col bg-bg-secondary rounded-2xl border border-border-default overflow-hidden animate-in fade-in slide-in-from-bottom-4 duration-500">
              {/* Table container - flex-1 with overflow */}
              <div className="flex-1 min-h-0 overflow-auto custom-scrollbar">
                <table className="w-full text-left text-sm table-fixed">
                  <thead className="sticky top-0 z-10">
                    <tr className="bg-bg-tertiary border-b border-border-default text-xs uppercase tracking-wider text-text-muted">
                      <th className="w-1/4 px-6 py-3 font-bold">文件名</th>
                      <th className="w-1/4 px-6 py-3 font-bold">ROM名 (Meta)</th>
                      <th className="w-1/4 px-6 py-3 font-bold">
                        <div className="flex items-center justify-between">
                          <span>提取中文名</span>
                          <button
                            onClick={handleSetAsRomName}
                            disabled={isSettingName}
                            className="px-2 py-1 text-[10px] bg-green-500/20 text-green-400 rounded hover:bg-green-500/30 transition-colors disabled:opacity-50 normal-case font-medium"
                            title="将提取的中文名设置为 ROM 名"
                          >
                            {isSettingName ? <Loader2 className="w-3 h-3 animate-spin" /> : <FileText className="w-3 h-3 inline mr-1" />}
                            设置为ROM名
                          </button>
                        </div>
                      </th>
                      <th className="w-1/4 px-6 py-3 font-bold">
                        <div className="flex items-center justify-between">
                          <span>匹配英文名</span>
                          <button
                            onClick={handleAddAsTag}
                            disabled={isAddingTag}
                            className="px-2 py-1 text-[10px] bg-blue-500/20 text-blue-400 rounded hover:bg-blue-500/30 transition-colors disabled:opacity-50 normal-case font-medium"
                            title="将英文名添加为额外 tag"
                          >
                            {isAddingTag ? <Loader2 className="w-3 h-3 animate-spin" /> : <Tag className="w-3 h-3 inline mr-1" />}
                            添加为额外tag
                          </button>
                        </div>
                      </th>
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-border-default">
                    {checkResults.map((res, idx) => {
                      const isMissing = !res.english_name;
                      
                      return (
                        <tr key={idx} className={clsx("hover:bg-bg-tertiary/30 transition-colors", isMissing && "bg-red-500/5")}>
                          <td className="px-6 py-3 text-text-primary font-mono text-xs truncate" title={res.file}>{res.file}</td>
                          
                          <td className={clsx("px-6 py-3 font-medium truncate", res.name && res.name !== res.file ? "text-text-primary" : "text-text-muted italic")}>
                            {res.name === res.file ? "未设置" : res.name}
                          </td>

                          <td className={clsx("px-6 py-3 font-medium truncate", res.extracted_cn_name ? "text-green-400" : "text-text-muted italic")}>
                            {res.extracted_cn_name || "-"}
                          </td>

                          <td className={clsx("px-6 py-3 font-medium truncate", res.english_name ? "text-blue-400" : "text-text-muted italic")}>
                            {res.english_name || "未匹配"}
                          </td>
                        </tr>
                      );
                    })}
                  </tbody>
                </table>
              </div>
              
              {/* Footer bar - shrink-0 to stay at bottom */}
              <div className="shrink-0 px-6 py-4 bg-bg-tertiary/30 border-t border-border-default flex items-center justify-between">
                <div className="flex items-center gap-4 text-xs text-text-muted font-medium">
                  <span>共 {checkResults.length} 个文件</span>
                  {missingCount > 0 && (
                    <span className="text-red-400 flex items-center gap-1.5">
                      <AlertTriangle className="w-3.5 h-3.5" />
                      {missingCount} 个未匹配英文名
                    </span>
                  )}
                </div>

                <div className="flex items-center gap-2">
                  {/* 匹配英文名按钮 */}
                  <button
                    onClick={handleAutoFix}
                    disabled={isFixing}
                    className="flex items-center gap-2 px-4 py-2 bg-blue-500 text-white font-bold rounded-lg hover:bg-blue-600 transition-all shadow-lg shadow-blue-500/20 text-xs disabled:opacity-50"
                  >
                    {isFixing ? <Loader2 className="w-3.5 h-3.5 animate-spin" /> : <Search className="w-3.5 h-3.5" />}
                    匹配英文名
                  </button>

                  {/* 导出按钮 */}
                  <div className="relative">
                    <button
                      onClick={() => setShowExportMenu(!showExportMenu)}
                      disabled={isExporting}
                      className="flex items-center gap-2 px-4 py-2 bg-accent-primary text-bg-primary font-bold rounded-lg hover:opacity-90 transition-all shadow-lg shadow-accent-primary/20 text-xs disabled:opacity-50"
                    >
                      {isExporting ? <Loader2 className="w-3.5 h-3.5 animate-spin" /> : <FileDown className="w-3.5 h-3.5" />}
                      导出 Metadata
                    </button>
                    
                    {/* 导出格式选择菜单 */}
                    {showExportMenu && (
                      <div className="absolute bottom-full right-0 mb-2 bg-bg-secondary border border-border-default rounded-lg shadow-xl overflow-hidden z-20 min-w-[180px]">
                        <button
                          onClick={() => handleExport("pegasus")}
                          className="w-full px-4 py-3 text-left text-sm text-text-primary hover:bg-bg-tertiary transition-colors flex items-center gap-2"
                        >
                          <FileText className="w-4 h-4 text-accent-primary" />
                          Pegasus (.txt)
                        </button>
                        <button
                          onClick={() => handleExport("gamelist")}
                          className="w-full px-4 py-3 text-left text-sm text-text-primary hover:bg-bg-tertiary transition-colors flex items-center gap-2 border-t border-border-default"
                        >
                          <FileText className="w-4 h-4 text-green-400" />
                          EmulationStation (.xml)
                        </button>
                      </div>
                    )}
                  </div>
                </div>
              </div>
            </div>
          )}
        </section>

      </div>
    </div>
  );
}
