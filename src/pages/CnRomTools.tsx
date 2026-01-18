import { useState, useMemo, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { Languages, Download, Loader2, Info, ExternalLink, FolderSearch, RefreshCw, Search, Tag, FileText, AlertTriangle, FileDown, ChevronDown, X, ArrowUp, ArrowDown } from "lucide-react";
import { isTauri, api } from "@/lib/api";
import { clsx } from "clsx";

interface MatchProgress {
  current: number;
  total: number;
}

interface CheckResult {
  file: string;
  name: string;
  english_name?: string;
  extracted_cn_name?: string;
  confidence?: number; // 匹配置信度 0-100
}

// Pegasus 支持的所有系统 (基于用户实际配置)
const PEGASUS_SYSTEMS: { id: string; name: string; aliases: string[] }[] = [
  // Nintendo
  { id: "fc", name: "FC / NES", aliases: ["fc", "nes", "famicom"] },
  { id: "fc-hd", name: "FC HD", aliases: ["fc-hd", "fc hd"] },
  { id: "fc hack", name: "FC Hack", aliases: ["fc hack"] },
  { id: "sfc", name: "SFC / SNES", aliases: ["sfc", "snes", "super famicom"] },
  { id: "sfc hack", name: "SFC Hack", aliases: ["sfc hack"] },
  { id: "sfc-msu1", name: "SFC MSU-1", aliases: ["sfc-msu1"] },
  { id: "n64", name: "Nintendo 64", aliases: ["n64"] },
  { id: "ngc", name: "Nintendo GameCube", aliases: ["ngc", "gc", "gamecube"] },
  { id: "wii", name: "Nintendo Wii", aliases: ["wii"] },
  { id: "wii ware", name: "Wii Ware", aliases: ["wii ware", "wiiware"] },
  { id: "gb", name: "Game Boy", aliases: ["gb", "gameboy"] },
  { id: "gbc", name: "Game Boy Color", aliases: ["gbc"] },
  { id: "gba", name: "Game Boy Advance", aliases: ["gba"] },
  { id: "nds", name: "Nintendo DS", aliases: ["nds", "ds"] },
  { id: "3ds", name: "Nintendo 3DS", aliases: ["3ds"] },
  { id: "virtual boy", name: "Virtual Boy", aliases: ["virtual boy", "vb"] },
  { id: "game watch", name: "Game & Watch", aliases: ["game watch", "game & watch"] },
  { id: "poke mini", name: "Pokemon Mini", aliases: ["poke mini", "pokemini"] },
  // Sega
  { id: "sms", name: "Sega Master System", aliases: ["sms", "master system"] },
  { id: "md", name: "Mega Drive / Genesis", aliases: ["md", "megadrive", "genesis"] },
  { id: "md hack", name: "MD Hack", aliases: ["md hack"] },
  { id: "md-32x", name: "Sega 32X", aliases: ["md-32x", "32x", "sega 32x"] },
  { id: "gg", name: "Sega Game Gear", aliases: ["gg", "gamegear"] },
  { id: "ss", name: "Sega Saturn", aliases: ["ss", "saturn"] },
  { id: "dc", name: "Sega Dreamcast", aliases: ["dc", "dreamcast"] },
  { id: "dc hack", name: "DC Hack", aliases: ["dc hack"] },
  { id: "naomi", name: "Sega NAOMI", aliases: ["naomi"] },
  // Sony
  { id: "ps1", name: "PlayStation", aliases: ["ps1", "psx", "playstation"] },
  { id: "ps1 hack", name: "PS1 Hack", aliases: ["ps1 hack"] },
  { id: "ps2", name: "PlayStation 2", aliases: ["ps2"] },
  { id: "psp", name: "PlayStation Portable", aliases: ["psp"] },
  // NEC
  { id: "pce", name: "PC Engine / TurboGrafx-16", aliases: ["pce", "pcengine", "tg16"] },
  // SNK
  { id: "ngpc", name: "Neo Geo Pocket Color", aliases: ["ngpc", "neogeo pocket"] },
  // Bandai
  { id: "ws", name: "WonderSwan", aliases: ["ws", "wonderswan"] },
  { id: "wsc", name: "WonderSwan Color", aliases: ["wsc"] },
  // Atari
  { id: "atari2600", name: "Atari 2600", aliases: ["atari2600", "2600"] },
  { id: "atari5200", name: "Atari 5200", aliases: ["atari5200", "5200"] },
  { id: "atari7800", name: "Atari 7800", aliases: ["atari7800", "7800"] },
  { id: "lynx", name: "Atari Lynx", aliases: ["lynx"] },
  // Arcade
  { id: "fbneo act", name: "FBNeo 动作", aliases: ["fbneo act"] },
  { id: "fbneo stg", name: "FBNeo 射击", aliases: ["fbneo stg"] },
  { id: "fbneo ftg", name: "FBNeo 格斗", aliases: ["fbneo ftg"] },
  { id: "fbneo fly", name: "FBNeo 飞行", aliases: ["fbneo fly"] },
  { id: "fbneo rac", name: "FBNeo 竞速", aliases: ["fbneo rac"] },
  { id: "fbneo spo", name: "FBNeo 体育", aliases: ["fbneo spo"] },
  { id: "fbneo etc", name: "FBNeo 其他", aliases: ["fbneo etc"] },
  { id: "mame act", name: "MAME 动作", aliases: ["mame act"] },
  { id: "mame stg", name: "MAME 射击", aliases: ["mame stg"] },
  { id: "mame ftg", name: "MAME 格斗", aliases: ["mame ftg"] },
  { id: "mame fly", name: "MAME 飞行", aliases: ["mame fly"] },
  { id: "mame rac", name: "MAME 竞速", aliases: ["mame rac"] },
  { id: "mame spo", name: "MAME 体育", aliases: ["mame spo"] },
  { id: "mame etc", name: "MAME 其他", aliases: ["mame etc"] },
  { id: "light gun", name: "光枪游戏", aliases: ["light gun"] },
  // PC
  { id: "dos", name: "DOS", aliases: ["dos"] },
];

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
  
  // 系统选择相关
  const [selectedSystem, setSelectedSystem] = useState<typeof PEGASUS_SYSTEMS[0] | null>(null);
  const [showSystemPicker, setShowSystemPicker] = useState(false);
  const [systemFilter, setSystemFilter] = useState("");

  // 匹配进度
  const [matchProgress, setMatchProgress] = useState<MatchProgress | null>(null);

  // 编辑状态管理
  const [editingIndex, setEditingIndex] = useState<number | null>(null);
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

  // 过滤后的系统列表
  const filteredSystems = useMemo(() => {
    if (!systemFilter.trim()) return PEGASUS_SYSTEMS;
    const lower = systemFilter.toLowerCase();
    return PEGASUS_SYSTEMS.filter(s => 
      s.name.toLowerCase().includes(lower) || 
      s.id.toLowerCase().includes(lower) ||
      s.aliases.some(a => a.includes(lower))
    );
  }, [systemFilter]);

  // 从目录名匹配系统（优先当前目录，其次上级目录）
  const matchSystemFromPath = (path: string): typeof PEGASUS_SYSTEMS[0] | null => {
    const parts = path.split(/[/\\]/).filter(Boolean);

    // 尝试匹配的目录名列表：当前目录优先，然后是上级目录
    const dirsToTry = [
      parts[parts.length - 1]?.toLowerCase() || "",  // 当前目录
      parts[parts.length - 2]?.toLowerCase() || "",  // 上级目录
    ].filter(Boolean);

    for (const dirName of dirsToTry) {
      for (const sys of PEGASUS_SYSTEMS) {
        if (sys.id === dirName || sys.aliases.some(a => a === dirName)) {
          return sys;
        }
      }
    }
    return null;
  };

  // 当前显示的系统名
  const displaySystemName = selectedSystem?.name || "选择平台";

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
        // 尝试从目录名匹配系统
        const matched = matchSystemFromPath(selected);
        setSelectedSystem(matched);
        // 如果没匹配到，显示系统选择器
        if (!matched) {
          setShowSystemPicker(true);
        }
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

    if (!selectedSystem) {
      alert("请先选择游戏平台");
      return;
    }

    if (!confirm(`确定要匹配英文名吗？\n这将扫描该目录下的所有 ROM，并根据本地数据库自动匹配英文名。\n仅置信度极高 (>95%) 的匹配会被应用。\n结果将保存到临时目录。`)) {
      return;
    }

    setIsFixing(true);
    setMatchProgress(null);
    try {
      const result = await api.autoFixNaming(checkPath, selectedSystem.id);
      alert(`匹配完成！\n成功: ${result.success}\n失败/跳过: ${result.failed}`);
      // 重新扫描以显示最新状态
      handleCheck();
    } catch (error) {
      console.error("Failed to auto fix:", error);
      alert("匹配失败: " + String(error));
    } finally {
      setIsFixing(false);
      setMatchProgress(null);
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

  // 开始编辑英文名
  const handleStartEdit = (index: number, currentValue: string) => {
    setEditingIndex(index);
    setEditingValue(currentValue || "");
  };

  // 确认编辑
  const handleConfirmEdit = async (index: number) => {
    if (editingIndex !== index) return;

    const result = checkResults[index];
    const newValue = editingValue.trim();

    // 更新本地状态
    const updatedResults = [...checkResults];
    updatedResults[index] = { ...result, english_name: newValue || undefined };
    setCheckResults(updatedResults);

    // 调用后端API保存更改
    try {
      if (isTauri()) {
        const { invoke } = await import("@tauri-apps/api/core");
        await invoke("update_english_name", {
          directory: checkPath,
          file: result.file,
          englishName: newValue,
        });
      }
    } catch (error) {
      console.error("Failed to save english name:", error);
      alert("保存失败: " + String(error));
    }

    setEditingIndex(null);
    setEditingValue("");
  };

  // 取消编辑
  const handleCancelEdit = () => {
    setEditingIndex(null);
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
                      <th className="px-6 py-3 font-bold relative" style={{ width: `${columnWidths[0]}%` }}>
                        文件名
                        <div
                          className="absolute top-0 right-0 w-1 h-full cursor-col-resize hover:bg-accent-primary/50 transition-colors"
                          onMouseDown={(e) => handleResizeStart(0, e)}
                        />
                      </th>
                      <th className="px-6 py-3 font-bold relative" style={{ width: `${columnWidths[1]}%` }}>
                        ROM名 (Meta)
                        <div
                          className="absolute top-0 right-0 w-1 h-full cursor-col-resize hover:bg-accent-primary/50 transition-colors"
                          onMouseDown={(e) => handleResizeStart(1, e)}
                        />
                      </th>
                      <th className="px-6 py-3 font-bold relative" style={{ width: `${columnWidths[2]}%` }}>
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
                            title="点击排序"
                          >
                            <span>匹配英文名</span>
                            {sortOrder === 'desc' && <ArrowDown className="w-3 h-3" />}
                            {sortOrder === 'asc' && <ArrowUp className="w-3 h-3" />}
                          </button>
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
                    {sortedResults.map((res, idx) => {
                      const isMissing = !res.english_name;
                      const isEditing = editingIndex === idx;
                      const bgColor = getConfidenceColor(res.confidence);

                      return (
                        <tr key={idx} className={clsx("hover:bg-bg-tertiary/30 transition-colors", isMissing && "bg-red-500/5")}>
                          <td className="px-6 py-3 text-text-primary font-mono text-xs truncate" title={res.file}>{res.file}</td>

                          <td className={clsx("px-6 py-3 font-medium truncate", res.name && res.name !== res.file ? "text-text-primary" : "text-text-muted italic")}>
                            {res.name === res.file ? "未设置" : res.name}
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
                                    handleConfirmEdit(idx);
                                  } else if (e.key === "Escape") {
                                    handleCancelEdit();
                                  }
                                }}
                                onBlur={() => handleConfirmEdit(idx)}
                                onFocus={(e) => e.target.select()}
                                autoFocus
                                className="w-full bg-bg-tertiary border border-accent-primary rounded px-2 py-1 text-sm text-text-primary focus:outline-none"
                              />
                            ) : (
                              <button
                                onClick={() => handleStartEdit(idx, res.english_name || "")}
                                className={clsx(
                                  "w-full text-left truncate hover:underline cursor-pointer",
                                  res.english_name ? "text-blue-400" : "text-text-muted italic"
                                )}
                                title={res.confidence ? `置信度: ${res.confidence.toFixed(1)}% - 点击编辑` : "点击编辑"}
                              >
                                {res.english_name || "未匹配"}
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
                    className="flex items-center gap-2 px-4 py-2 bg-blue-500 text-white font-bold rounded-lg hover:bg-blue-600 transition-all shadow-lg shadow-blue-500/20 text-xs disabled:opacity-50 min-w-[120px] justify-center"
                  >
                    {isFixing ? <Loader2 className="w-3.5 h-3.5 animate-spin" /> : <Search className="w-3.5 h-3.5" />}
                    {isFixing && matchProgress
                      ? `(${matchProgress.current}/${matchProgress.total})`
                      : "匹配英文名"}
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

      {/* 系统选择弹窗 */}
      {showSystemPicker && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm">
          <div className="bg-bg-secondary border border-border-default rounded-2xl shadow-2xl w-full max-w-md max-h-[80vh] flex flex-col overflow-hidden animate-in fade-in zoom-in-95 duration-200">
            {/* 弹窗头部 */}
            <div className="shrink-0 flex items-center justify-between px-6 py-4 border-b border-border-default">
              <h3 className="text-lg font-bold text-text-primary">选择游戏平台</h3>
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
                  placeholder="搜索平台..."
                  className="w-full bg-bg-tertiary border border-border-default rounded-lg pl-10 pr-4 py-2 text-sm text-text-primary focus:outline-none focus:border-accent-primary"
                  autoFocus
                />
              </div>
            </div>
            
            {/* 系统列表 */}
            <div className="flex-1 min-h-0 overflow-auto custom-scrollbar">
              {filteredSystems.length === 0 ? (
                <div className="p-6 text-center text-text-muted text-sm">
                  没有找到匹配的平台
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
