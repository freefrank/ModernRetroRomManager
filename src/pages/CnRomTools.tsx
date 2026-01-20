import { useState, useMemo, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { Languages, Download, Loader2, Info, ExternalLink, FolderSearch, RefreshCw, Search, Tag, FileText, AlertTriangle, FileDown, ChevronDown, X, ArrowUp, ArrowDown } from "lucide-react";
import { isTauri, api } from "@/lib/api";
import { clsx } from "clsx";
import type { GameSystem } from "@/types";
import { useCnRomToolsStore, type MatchProgress, type ScanProgress } from "@/stores/cnRomToolsStore";

export default function CnName() {
  const { t } = useTranslation();
  const [isUpdating, setIsUpdating] = useState(false);
  const [isSettingName, setIsSettingName] = useState(false);
  const [isAddingTag, setIsAddingTag] = useState(false);
  const [isExporting, setIsExporting] = useState(false);
  const [showExportMenu, setShowExportMenu] = useState(false);

  // 从 store 获取状态
  const {
    checkPath,
    setCheckPath,
    checkResults,
    isScanning: isChecking,
    isFixing,
    scanProgress,
    matchProgress,
    setScanProgress,
    setMatchProgress,
    scan,
    autoFix,
    updateEnglishName,
  } = useCnRomToolsStore();

  // 系统列表（从后端获取）
  const [systems, setSystems] = useState<GameSystem[]>([]);

  // 系统选择相关
  const [selectedSystem, setSelectedSystem] = useState<GameSystem | null>(null);
  const [showSystemPicker, setShowSystemPicker] = useState(false);
  const [systemFilter, setSystemFilter] = useState("");

  // 加载系统列表
  useEffect(() => {
    const loadSystems = async () => {
      try {
        const systemList = await api.getSystems();
        setSystems(systemList);
      } catch (error) {
        console.error("Failed to load systems:", error);
      }
    };
    loadSystems();
  }, []);

  // 编辑状态管理
  const [editingFile, setEditingFile] = useState<string | null>(null);
  const [editingValue, setEditingValue] = useState("");

  // 排序状态
  const [sortOrder, setSortOrder] = useState<'asc' | 'desc' | null>(null);

  // 列宽状态（百分比）
  const [columnWidths, setColumnWidths] = useState([25, 25, 25, 25]);
  const [resizingColumn, setResizingColumn] = useState<number | null>(null);
  const [startX, setStartX] = useState(0);
  const [startWidth, setStartWidth] = useState(0);

  // 切换排序
  const toggleSort = () => {
    if (sortOrder === null) {
      setSortOrder('desc'); // 第一次点击：降序（高分在前）
    } else if (sortOrder === 'desc') {
      setSortOrder('asc'); // 第二次点击：升序（低分在前）
    } else {
      setSortOrder(null); // 第三次点击：取消排序
    }
  };

  // 开始调整列宽
  const handleResizeStart = (columnIndex: number, e: React.MouseEvent) => {
    e.preventDefault();
    setResizingColumn(columnIndex);
    setStartX(e.clientX);
    setStartWidth(columnWidths[columnIndex]);
  };

  // 调整列宽中
  useEffect(() => {
    if (resizingColumn === null) return;

    const handleMouseMove = (e: MouseEvent) => {
      const diff = e.clientX - startX;
      const tableWidth = document.querySelector('table')?.offsetWidth || 1000;
      const diffPercent = (diff / tableWidth) * 100;

      const newWidths = [...columnWidths];
      const newWidth = Math.max(10, Math.min(50, startWidth + diffPercent));
      const oldWidth = columnWidths[resizingColumn];
      const delta = newWidth - oldWidth;

      newWidths[resizingColumn] = newWidth;

      // 调整下一列的宽度以保持总宽度为100%
      if (resizingColumn < 3) {
        newWidths[resizingColumn + 1] = Math.max(10, columnWidths[resizingColumn + 1] - delta);
      }

      setColumnWidths(newWidths);
    };

    const handleMouseUp = () => {
      setResizingColumn(null);
    };

    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);

    return () => {
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
    };
  }, [resizingColumn, startX, startWidth, columnWidths]);

  // 排序后的结果
  const sortedResults = useMemo(() => {
    if (!sortOrder) return checkResults;

    return [...checkResults].sort((a, b) => {
      const confA = a.confidence ?? -1; // 没有置信度的排在最后
      const confB = b.confidence ?? -1;

      if (sortOrder === 'asc') {
        return confA - confB;
      } else {
        return confB - confA;
      }
    });
  }, [checkResults, sortOrder]);

  // 根据置信度计算背景色 (0-100 -> 红色到无色)
  const getConfidenceColor = (confidence?: number): string => {
    if (!confidence) return "transparent";
    // 置信度越低越红，越高越透明
    // 0-50: 深红到浅红
    // 50-80: 浅红到橙色
    // 80-100: 橙色到透明
    const normalized = Math.max(0, Math.min(100, confidence));
    if (normalized >= 95) return "transparent";
    if (normalized >= 80) return `rgba(251, 146, 60, ${(95 - normalized) / 15 * 0.3})`; // orange
    if (normalized >= 50) return `rgba(239, 68, 68, ${(80 - normalized) / 30 * 0.5})`; // red
    return `rgba(220, 38, 38, ${(50 - normalized) / 50 * 0.7 + 0.3})`; // dark red
  };

// 监听匹配进度事件
  useEffect(() => {
    if (!isTauri()) return;

    let unlisten: (() => void) | undefined;

    const setupListener = async () => {
      const { listen } = await import("@tauri-apps/api/event");
      unlisten = await listen<MatchProgress>("naming-match-progress", (event) => {
        setMatchProgress(event.payload);
      });
    };

    setupListener();

    return () => {
      if (unlisten) unlisten();
    };
  }, []);

  // 监听扫描进度事件
  useEffect(() => {
    if (!isTauri()) return;

    let unlisten: (() => void) | undefined;

    const setupListener = async () => {
      const { listen } = await import("@tauri-apps/api/event");
      unlisten = await listen<ScanProgress>("scan-progress", (event) => {
        setScanProgress(event.payload);
      });
    };

    setupListener();

    return () => {
      if (unlisten) unlisten();
    };
  }, []);

  // 过滤后的系统列表
  const filteredSystems = useMemo(() => {
    if (!systemFilter.trim()) return systems;
    const lower = systemFilter.toLowerCase();
    return systems.filter(s =>
      s.name.toLowerCase().includes(lower) ||
      s.id.toLowerCase().includes(lower)
    );
  }, [systemFilter, systems]);

  // 从目录名匹配系统（优先当前目录，其次上级目录）
  const matchSystemFromPath = (path: string): GameSystem | null => {
    const parts = path.split(/[/\\]/).filter(Boolean);

    // 尝试匹配的目录名列表：当前目录优先，然后是上级目录
    const dirsToTry = [
      parts[parts.length - 1]?.toLowerCase() || "",  // 当前目录
      parts[parts.length - 2]?.toLowerCase() || "",  // 上级目录
    ].filter(Boolean);

    for (const dirName of dirsToTry) {
      for (const sys of systems) {
        if (sys.id === dirName) {
          return sys;
        }
      }
    }
    return null;
  };

  // 当前显示的系统名
  const displaySystemName = selectedSystem?.name || t("cnRomTools.selectPlatform");

  const handleUpdate = async () => {
    setIsUpdating(true);
    try {
      if (isTauri()) {
        const { invoke } = await import("@tauri-apps/api/core");
        await invoke("update_cn_repo");
      }
      alert(t("cnRomTools.alerts.databaseUpdateSuccess"));
    } catch (error) {
      console.error("Failed to update CN repo:", error);
      alert(t("cnRomTools.alerts.updateFailed", { error: String(error) }));
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
        // 尝试从目录名匹配系统
        const matched = matchSystemFromPath(selected);
        setSelectedSystem(matched);
        // 如果没匹配到，显示系统选择器
        if (!matched) {
          setShowSystemPicker(true);
        }
        // 自动触发扫描
        scan(selected);
      }
    } catch (error) {
      console.error("Failed to select directory:", error);
    }
  };

const handleCheck = async () => {
    if (!checkPath) return;
    scan();
  };

  const handleAutoFix = async () => {
    if (!checkPath) return;

    if (!selectedSystem) {
      alert(t("cnRomTools.alerts.selectPlatformFirst"));
      return;
    }

    if (!confirm(t("cnRomTools.confirms.autoFixNaming"))) {
      return;
    }

    const result = await autoFix(selectedSystem.id);
    if (result) {
      alert(t("cnRomTools.alerts.matchComplete", { success: result.success, failed: result.failed }));
    }
  };

  const handleSetAsRomName = async () => {
    if (!checkPath) return;
    
    if (!confirm(t("cnRomTools.confirms.setAsRomName"))) {
      return;
    }

    setIsSettingName(true);
    try {
      if (isTauri()) {
        const { invoke } = await import("@tauri-apps/api/core");
        await invoke("set_extracted_cn_as_name", { directory: checkPath });
      }
      alert(t("cnRomTools.alerts.setComplete"));
      handleCheck();
    } catch (error) {
      console.error("Failed to set ROM name:", error);
      alert(t("cnRomTools.alerts.setFailed", { error: String(error) }));
    } finally {
      setIsSettingName(false);
    }
  };

  const handleAddAsTag = async () => {
    if (!checkPath) return;
    
    if (!confirm(t("cnRomTools.confirms.addAsTag"))) {
      return;
    }

    setIsAddingTag(true);
    try {
      if (isTauri()) {
        const { invoke } = await import("@tauri-apps/api/core");
        await invoke("add_english_as_tag", { directory: checkPath });
      }
      alert(t("cnRomTools.alerts.addComplete"));
      handleCheck();
    } catch (error) {
      console.error("Failed to add tag:", error);
      alert(t("cnRomTools.alerts.addFailed", { error: String(error) }));
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
      alert(t("cnRomTools.alerts.exportSuccess", { path: savePath }));
    } catch (error) {
      console.error("Failed to export:", error);
      alert(t("cnRomTools.alerts.exportFailed", { error: String(error) }));
    } finally {
      setIsExporting(false);
    }
  };

  // 开始编辑英文名
  const handleStartEdit = (file: string, currentValue: string) => {
    setEditingFile(file);
    setEditingValue(currentValue || "");
  };

  // 确认编辑
  const handleConfirmEdit = async (file: string) => {
    if (editingFile !== file) return;

    const newValue = editingValue.trim();

    // 在 checkResults 中查找对应的条目
    const resultIndex = checkResults.findIndex(r => r.file === file);
if (resultIndex === -1) return;

    // 更新本地状态（通过store）
    updateEnglishName(file, newValue || "");

    // 调用后端API保存更改
    try {
      if (isTauri()) {
        const { invoke } = await import("@tauri-apps/api/core");
        await invoke("update_english_name", {
          directory: checkPath,
          file: file,
          englishName: newValue,
        });
      }
    } catch (error) {
      console.error("Failed to save english name:", error);
      alert(t("cnRomTools.alerts.saveFailed", { error: String(error) }));
    }

    setEditingFile(null);
    setEditingValue("");
  };

  // 取消编辑
  const handleCancelEdit = () => {
    setEditingFile(null);
    setEditingValue("");
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
              <h2 className="text-lg font-bold text-text-primary">{t("cnRomTools.about.title")}</h2>
            </div>
            <p className="text-sm text-text-secondary leading-relaxed">
              {t("cnRomTools.about.description.part1")} <a href="https://github.com/yingw/rom-name-cn" target="_blank" rel="noopener noreferrer" className="text-accent-primary hover:underline font-bold">yingw/rom-name-cn</a> {t("cnRomTools.about.description.part2")}
            </p>
          </div>
          <div className="bg-bg-tertiary/50 p-4 flex items-center justify-between">
            <div className="text-xs text-text-muted font-medium">
              {t("cnRomTools.about.dataSource")}
            </div>
            <div className="flex gap-4">
              <a 
                href="https://github.com/yingw/rom-name-cn" 
                target="_blank" 
                rel="noopener noreferrer"
                className="flex items-center gap-1.5 text-xs font-bold text-text-primary hover:text-accent-primary transition-colors"
              >
                {t("cnRomTools.about.visitRepo")} <ExternalLink className="w-3 h-3" />
              </a>
              <button
                onClick={handleUpdate}
                disabled={isUpdating}
                className="flex items-center gap-1.5 text-xs font-bold text-accent-primary hover:text-accent-primary/80 transition-colors disabled:opacity-50"
              >
                {isUpdating ? <Loader2 className="w-3 h-3 animate-spin" /> : <Download className="w-3 h-3" />}
                {isUpdating ? t("cnRomTools.about.updating") : t("cnRomTools.about.updateDatabase")}
              </button>
            </div>
          </div>
        </section>

        {/* Directory Check - flex-1 to fill remaining space */}
        <section className="flex-1 min-h-0 flex flex-col gap-4">
          <div className="shrink-0 flex items-center gap-2">
            <FolderSearch className="w-5 h-5 text-accent-primary" />
            {/* 系统名称按钮 - 点击可选择 */}
            <button
              onClick={() => setShowSystemPicker(true)}
              className="text-lg font-bold text-text-primary flex items-center gap-1 hover:text-accent-primary transition-colors"
            >
              {displaySystemName}
              <ChevronDown className="w-4 h-4" />
            </button>
          </div>
          
          <div className="shrink-0 flex gap-3">
            <div className="flex-1 flex gap-2">
              <input 
                type="text" 
                value={checkPath}
                readOnly
                placeholder={t("cnRomTools.selectDirectoryPlaceholder")}
                className="flex-1 bg-bg-secondary border border-border-default rounded-xl px-4 py-3 text-sm text-text-primary focus:outline-none"
              />
              <button 
                onClick={handleBrowse}
                className="px-4 py-2 bg-bg-tertiary hover:bg-border-hover text-text-primary rounded-xl font-bold transition-all border border-border-default text-sm whitespace-nowrap"
              >
                {t("cnRomTools.browse")}
              </button>
            </div>
<button
              onClick={handleCheck}
              disabled={!checkPath || isChecking || isFixing}
              className="px-6 py-2 bg-accent-primary text-bg-primary font-bold rounded-xl hover:opacity-90 active:scale-95 transition-all shadow-lg shadow-accent-primary/20 disabled:opacity-50 disabled:cursor-not-allowed text-sm flex items-center gap-2 whitespace-nowrap"
            >
              {isChecking ? <Loader2 className="w-4 h-4 animate-spin" /> : <RefreshCw className="w-4 h-4" />}
              {isChecking && scanProgress 
                ? `(${scanProgress.current}/${scanProgress.total})`
                : t("cnRomTools.refresh")}
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
                      <th className="px-6 py-3 font-bold relative" style={{ width: `${columnWidths[0]}%` }}>
                        {t("cnRomTools.table.fileName")}
                        <div
                          className="absolute top-0 right-0 w-1 h-full cursor-col-resize hover:bg-accent-primary/50 transition-colors"
                          onMouseDown={(e) => handleResizeStart(0, e)}
                        />
                      </th>
                      <th className="px-6 py-3 font-bold relative" style={{ width: `${columnWidths[1]}%` }}>
                        {t("cnRomTools.table.romName")}
                        <div
                          className="absolute top-0 right-0 w-1 h-full cursor-col-resize hover:bg-accent-primary/50 transition-colors"
                          onMouseDown={(e) => handleResizeStart(1, e)}
                        />
                      </th>
                      <th className="px-6 py-3 font-bold relative" style={{ width: `${columnWidths[2]}%` }}>
                        <div className="flex items-center justify-between">
                          <span>{t("cnRomTools.table.extractedCnName")}</span>
                          <button
                            onClick={handleSetAsRomName}
                            disabled={isSettingName}
                            className="px-2 py-1 text-[10px] bg-green-500/20 text-green-400 rounded hover:bg-green-500/30 transition-colors disabled:opacity-50 normal-case font-medium"
                            title={t("cnRomTools.table.setAsRomNameTitle")}
                          >
                            {isSettingName ? <Loader2 className="w-3 h-3 animate-spin" /> : <FileText className="w-3 h-3 inline mr-1" />}
                            {t("cnRomTools.table.setAsRomName")}
                          </button>
                        </div>
                        <div
                          className="absolute top-0 right-0 w-1 h-full cursor-col-resize hover:bg-accent-primary/50 transition-colors"
                          onMouseDown={(e) => handleResizeStart(2, e)}
                        />
                      </th>
                      <th className="px-6 py-3 font-bold relative" style={{ width: `${columnWidths[3]}%` }}>
                        <div className="flex items-center justify-between">
                          <button
                            onClick={toggleSort}
                            className="flex items-center gap-1 hover:text-accent-primary transition-colors cursor-pointer"
                            title={t("cnRomTools.table.clickToSort")}
                          >
                            <span>{t("cnRomTools.table.matchedEnglishName")}</span>
                            {sortOrder === 'desc' && <ArrowDown className="w-3 h-3" />}
                            {sortOrder === 'asc' && <ArrowUp className="w-3 h-3" />}
                          </button>
                          <button
                            onClick={handleAddAsTag}
                            disabled={isAddingTag}
                            className="px-2 py-1 text-[10px] bg-blue-500/20 text-blue-400 rounded hover:bg-blue-500/30 transition-colors disabled:opacity-50 normal-case font-medium"
                            title={t("cnRomTools.table.addAsTagTitle")}
                          >
                            {isAddingTag ? <Loader2 className="w-3 h-3 animate-spin" /> : <Tag className="w-3 h-3 inline mr-1" />}
                            {t("cnRomTools.table.addAsTag")}
                          </button>
                        </div>
                      </th>
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-border-default">
                    {sortedResults.map((res, idx) => {
                      const isMissing = !res.english_name;
                      const isEditing = editingFile === res.file;
                      const bgColor = getConfidenceColor(res.confidence);

                      return (
                        <tr key={idx} className={clsx("hover:bg-bg-tertiary/30 transition-colors", isMissing && "bg-red-500/5")}>
                          <td className="px-6 py-3 text-text-primary font-mono text-xs truncate" title={res.file}>{res.file}</td>

                          <td className={clsx("px-6 py-3 font-medium truncate", res.name && res.name !== res.file ? "text-text-primary" : "text-text-muted italic")}>
                            {res.name === res.file ? t("cnRomTools.table.notSet") : res.name}
                          </td>

                          <td className={clsx("px-6 py-3 font-medium truncate", res.extracted_cn_name ? "text-green-400" : "text-text-muted italic")}>
                            {res.extracted_cn_name || "-"}
                          </td>

                          <td
                            className="px-6 py-3 font-medium truncate relative"
                            style={{ backgroundColor: bgColor }}
                          >
                            {isEditing ? (
                              <input
                                type="text"
                                value={editingValue}
                                onChange={(e) => setEditingValue(e.target.value)}
                                onKeyDown={(e) => {
                                  if (e.key === "Enter") {
                                    handleConfirmEdit(res.file);
                                  } else if (e.key === "Escape") {
                                    handleCancelEdit();
                                  }
                                }}
                                onBlur={() => handleConfirmEdit(res.file)}
                                onFocus={(e) => e.target.select()}
                                autoFocus
                                className="w-full bg-bg-tertiary border border-accent-primary rounded px-2 py-1 text-sm text-text-primary focus:outline-none"
                              />
                            ) : (
                              <button
                                onClick={() => handleStartEdit(res.file, res.english_name || "")}
                                className={clsx(
                                  "w-full text-left truncate hover:underline cursor-pointer",
                                  res.english_name ? "text-blue-400" : "text-text-muted italic"
                                )}
                                title={res.confidence ? t("cnRomTools.table.confidenceWithEdit", { confidence: res.confidence.toFixed(1) }) : t("cnRomTools.table.clickToEdit")}
                              >
                                {res.english_name || t("cnRomTools.table.notMatched")}
                              </button>
                            )}
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
                  <span>{t("cnRomTools.footer.totalFiles", { count: checkResults.length })}</span>
                  {missingCount > 0 && (
                    <span className="text-red-400 flex items-center gap-1.5">
                      <AlertTriangle className="w-3.5 h-3.5" />
                      {t("cnRomTools.footer.missingEnglishNames", { count: missingCount })}
                    </span>
                  )}
                </div>

                <div className="flex items-center gap-2">
                  {/* 匹配英文名按钮 */}
                  <button
                    onClick={handleAutoFix}
                    disabled={isFixing}
                    className="flex items-center gap-2 px-4 py-2 bg-blue-500 text-white font-bold rounded-lg hover:bg-blue-600 transition-all shadow-lg shadow-blue-500/20 text-xs disabled:opacity-50 min-w-[120px] justify-center"
                  >
                    {isFixing ? <Loader2 className="w-3.5 h-3.5 animate-spin" /> : <Search className="w-3.5 h-3.5" />}
                    {isFixing && matchProgress
                      ? `(${matchProgress.current}/${matchProgress.total})`
                      : t("cnRomTools.footer.matchEnglishName")}
                  </button>

                  {/* 导出按钮 */}
                  <div className="relative">
                    <button
                      onClick={() => setShowExportMenu(!showExportMenu)}
                      disabled={isExporting}
                      className="flex items-center gap-2 px-4 py-2 bg-accent-primary text-bg-primary font-bold rounded-lg hover:opacity-90 transition-all shadow-lg shadow-accent-primary/20 text-xs disabled:opacity-50"
                    >
                      {isExporting ? <Loader2 className="w-3.5 h-3.5 animate-spin" /> : <FileDown className="w-3.5 h-3.5" />}
                      {t("cnRomTools.footer.exportMetadata")}
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

      {/* 系统选择弹窗 */}
      {showSystemPicker && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm">
          <div className="bg-bg-secondary border border-border-default rounded-2xl shadow-2xl w-full max-w-md max-h-[80vh] flex flex-col overflow-hidden animate-in fade-in zoom-in-95 duration-200">
            {/* 弹窗头部 */}
            <div className="shrink-0 flex items-center justify-between px-6 py-4 border-b border-border-default">
              <h3 className="text-lg font-bold text-text-primary">{t("cnRomTools.systemPicker.title")}</h3>
              <button
                onClick={() => setShowSystemPicker(false)}
                className="p-1 hover:bg-bg-tertiary rounded-lg transition-colors"
              >
                <X className="w-5 h-5 text-text-muted" />
              </button>
            </div>
            
            {/* 搜索过滤栏 */}
            <div className="shrink-0 px-6 py-3 border-b border-border-default">
              <div className="relative">
                <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-text-muted" />
                <input
                  type="text"
                  value={systemFilter}
                  onChange={(e) => setSystemFilter(e.target.value)}
                  placeholder={t("cnRomTools.systemPicker.searchPlaceholder")}
                  className="w-full bg-bg-tertiary border border-border-default rounded-lg pl-10 pr-4 py-2 text-sm text-text-primary focus:outline-none focus:border-accent-primary"
                  autoFocus
                />
              </div>
            </div>
            
            {/* 系统列表 */}
            <div className="flex-1 min-h-0 overflow-auto custom-scrollbar">
              {filteredSystems.length === 0 ? (
                <div className="p-6 text-center text-text-muted text-sm">
                  {t("cnRomTools.systemPicker.noResults")}
                </div>
              ) : (
                <div className="p-2">
                  {filteredSystems.map((sys) => (
                    <button
                      key={sys.id}
                      onClick={() => {
                        setSelectedSystem(sys);
                        setShowSystemPicker(false);
                        setSystemFilter("");
                      }}
                      className={clsx(
                        "w-full px-4 py-3 text-left rounded-lg transition-colors flex items-center gap-3",
                        selectedSystem?.id === sys.id
                          ? "bg-accent-primary/20 text-accent-primary"
                          : "hover:bg-bg-tertiary text-text-primary"
                      )}
                    >
                      <span className="font-medium">{sys.name}</span>
                      <span className="text-xs text-text-muted ml-auto">{sys.id}</span>
                    </button>
                  ))}
                </div>
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
