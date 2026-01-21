import { create } from "zustand";
import { api } from "@/lib/api";

export interface CheckResult {
  file: string;
  name: string;
  english_name?: string;
  extracted_cn_name?: string;
  confidence?: number;
}

export interface ScanProgress {
  current: number;
  total: number;
  current_folder: string;
}

export interface MatchProgress {
  current: number;
  total: number;
}

interface CnRomToolsState {
  checkPath: string;
  checkResults: CheckResult[];
  isScanning: boolean;
  isFixing: boolean;
  scanProgress: ScanProgress | null;
  matchProgress: MatchProgress | null;
  
  setCheckPath: (path: string) => void;
  setScanProgress: (progress: ScanProgress | null) => void;
  setMatchProgress: (progress: MatchProgress | null) => void;
  
  scan: (path?: string) => Promise<void>;
  autoFix: (system: string) => Promise<{ success: number; failed: number } | null>;
  updateEnglishName: (file: string, englishName: string) => void;
}

export const useCnRomToolsStore = create<CnRomToolsState>((set, get) => ({
  checkPath: "",
  checkResults: [],
  isScanning: false,
  isFixing: false,
  scanProgress: null,
  matchProgress: null,
  
  setCheckPath: (path) => set({ checkPath: path }),
  setScanProgress: (progress) => set({ scanProgress: progress }),
  setMatchProgress: (progress) => set({ matchProgress: progress }),
  
  scan: async (path?: string) => {
    const scanPath = path || get().checkPath;
    if (!scanPath || get().isScanning) return;
    
    if (path) set({ checkPath: path });
    set({ isScanning: true, scanProgress: null, checkResults: [] });
    
    try {
      const results = await api.scanDirectoryForNamingCheck(scanPath);
      set({ checkResults: results as CheckResult[] });
    } catch (error) {
      console.error("Failed to scan directory:", error);
    } finally {
      set({ isScanning: false, scanProgress: null });
    }
  },
  
  autoFix: async (system: string) => {
    const { checkPath } = get();
    if (!checkPath || get().isFixing) return null;
    
    set({ isFixing: true, matchProgress: null });
    
    try {
      const result = await api.autoFixNaming(checkPath, system);
      // 使用快速API读取临时元数据，不重新扫描文件系统
      const results = await api.getNamingCheckResults(checkPath);
      
      // 默认按置信度升序排序（未匹配/低置信度在前），方便用户处理
      const sorted = (results as CheckResult[]).sort((a, b) => {
        const confA = a.confidence ?? -1;
        const confB = b.confidence ?? -1;
        if (confA !== confB) return confA - confB;
        return a.file.localeCompare(b.file);
      });
      
      set({ checkResults: sorted });
      return result;
    } catch (error) {
      console.error("Failed to auto fix:", error);
      return null;
    } finally {
      set({ isFixing: false, matchProgress: null });
    }
  },
  
  updateEnglishName: (file: string, englishName: string) => {
    const { checkResults } = get();
    const updated = checkResults.map((r) =>
      r.file === file ? { ...r, english_name: englishName, confidence: 100 } : r
    );
    set({ checkResults: updated });
  },
}));
