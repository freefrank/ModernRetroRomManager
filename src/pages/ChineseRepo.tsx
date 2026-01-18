import { useState } from "react";
import { Languages, Download, Loader2, Info, ExternalLink } from "lucide-react";
import { isTauri } from "@/lib/api";

export default function ChineseRepo() {
  const [isUpdating, setIsUpdating] = useState(false);

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

  return (
    <div className="flex flex-col h-full bg-bg-primary">
      {/* Header */}
      <div className="flex items-center justify-between px-6 py-4 border-b border-border-default bg-bg-primary/50 backdrop-blur-md sticky top-0 z-10">
        <div className="flex items-center gap-3">
          <Languages className="w-6 h-6 text-accent-primary" />
          <h1 className="text-xl font-bold text-text-primary tracking-tight">中文 ROM 数据库</h1>
        </div>
      </div>

      <div className="flex-1 p-8 overflow-auto">
        <div className="max-w-3xl mx-auto space-y-8">
          
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
              <a 
                href="https://github.com/yingw/rom-name-cn" 
                target="_blank" 
                rel="noopener noreferrer"
                className="flex items-center gap-1.5 text-xs font-bold text-text-primary hover:text-accent-primary transition-colors"
              >
                访问仓库 <ExternalLink className="w-3 h-3" />
              </a>
            </div>
          </section>

          {/* Update Card */}
          <section className="bg-bg-secondary rounded-2xl border border-border-default p-6 flex items-center justify-between">
            <div>
              <h3 className="font-bold text-text-primary text-lg mb-1">数据库更新</h3>
              <p className="text-sm text-text-muted">
                从 GitHub 拉取最新的对照表数据。建议定期更新以获取新游戏的中文译名。
              </p>
            </div>
            
            <button
              onClick={handleUpdate}
              disabled={isUpdating}
              className="flex items-center gap-2 px-6 py-3 bg-accent-primary text-bg-primary font-bold rounded-xl hover:opacity-90 active:scale-95 transition-all shadow-lg shadow-accent-primary/20 disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {isUpdating ? (
                <>
                  <Loader2 className="w-5 h-5 animate-spin" />
                  更新中...
                </>
              ) : (
                <>
                  <Download className="w-5 h-5" />
                  立即更新
                </>
              )}
            </button>
          </section>

          {/* Stats Placeholder */}
          <div className="grid grid-cols-3 gap-4">
            <div className="p-4 bg-bg-secondary rounded-xl border border-border-default text-center">
              <div className="text-2xl font-black text-text-primary mb-1">10k+</div>
              <div className="text-[10px] uppercase tracking-widest text-text-muted font-bold">收录游戏</div>
            </div>
            <div className="p-4 bg-bg-secondary rounded-xl border border-border-default text-center">
              <div className="text-2xl font-black text-text-primary mb-1">20+</div>
              <div className="text-[10px] uppercase tracking-widest text-text-muted font-bold">支持平台</div>
            </div>
            <div className="p-4 bg-bg-secondary rounded-xl border border-border-default text-center">
              <div className="text-2xl font-black text-green-400 mb-1">本地</div>
              <div className="text-[10px] uppercase tracking-widest text-text-muted font-bold">数据状态</div>
            </div>
          </div>

        </div>
      </div>
    </div>
  );
}
