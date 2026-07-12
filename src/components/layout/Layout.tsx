import { useEffect, useState } from "react";
import { Outlet } from "react-router-dom";
import { listen } from "@tauri-apps/api/event";
import { FolderPlus } from "lucide-react";
import { clsx } from "clsx";
import { useTranslation } from "react-i18next";
import { useRomStore } from "@/stores/romStore";
import Backdrop from "@/theme/Backdrop";
import { Toaster } from "@/components/ui";
import Sidebar from "./Sidebar";
import StatusBar from "./StatusBar";

export default function Layout() {
  const { t } = useTranslation();
  const { addScanDirectory } = useRomStore();
  const [isDragging, setIsDragging] = useState(false);

  useEffect(() => {
    // 监听拖拽事件
    const unlistenDrop = listen<string[]>("tauri://file-drop", async (event) => {
      setIsDragging(false);
      const paths = event.payload;
      if (paths && paths.length > 0) {
        // 目前只处理第一个路径
        // TODO: 检查是否是文件夹，或循环处理
        // 由于前端无法直接判断是否为文件夹（除非调用 Rust），我们假设用户拖拽的是文件夹
        // 或者让后端 addScanDirectory 处理（如果不是文件夹则报错或忽略）
        for (const path of paths) {
          try {
            await addScanDirectory(path);
            // Optional: Trigger scan immediately? addScanDirectory currently just adds config.
            // Consider auto-scanning.
          } catch (e) {
            console.error("Failed to add drop path:", path, e);
          }
        }
      }
    });

    const unlistenHover = listen("tauri://file-drop-hover", () => {
      setIsDragging(true);
    });

    const unlistenCancel = listen("tauri://file-drop-cancelled", () => {
      setIsDragging(false);
    });

    return () => {
      unlistenDrop.then((f) => f());
      unlistenHover.then((f) => f());
      unlistenCancel.then((f) => f());
    };
  }, [addScanDirectory]);

  return (
    <div className="h-screen flex flex-col bg-bg-primary text-text-primary relative overflow-hidden">
      {/* 主题背景叠加层 */}
      <Backdrop />

      {/* Drag Overlay(常驻挂载,淡入淡出走 motion 令牌的 CSS 过渡) */}
      <div
        aria-hidden={!isDragging}
        className={clsx(
          "absolute inset-0 z-50 bg-accent-primary/80 backdrop-blur-md flex flex-col items-center justify-center text-text-primary border-4 border-text-primary/20 border-dashed m-4 rounded-[var(--radius-lg)]",
          "transition-opacity duration-[var(--motion-normal)] ease-[var(--motion-easing)]",
          isDragging ? "opacity-100" : "opacity-0 pointer-events-none",
        )}
      >
        <FolderPlus className="w-24 h-24 mb-6 animate-bounce" />
        <h2 className="text-3xl font-bold mb-2">{t("dropOverlay.title")}</h2>
        <p className="text-text-primary/80">{t("dropOverlay.subtitle")}</p>
      </div>

      {/* Background Ambience */}
      <div className="absolute top-0 left-0 w-full h-full overflow-hidden pointer-events-none z-0">
        <div className="absolute top-[-10%] left-[-10%] w-[40%] h-[40%] bg-accent-primary/20 rounded-full blur-[120px] mix-blend-screen animate-pulse"></div>
        <div className="absolute bottom-[-10%] right-[-10%] w-[40%] h-[40%] bg-accent-secondary/10 rounded-full blur-[120px] mix-blend-screen"></div>
      </div>

      <div className="flex-1 flex overflow-hidden relative z-10">
        <Sidebar />
        <main className="flex-1 overflow-hidden relative flex flex-col">
          {/* Main Content Area */}
          <div className="flex-1 overflow-y-auto custom-scrollbar p-6">
            <Outlet />
          </div>
        </main>
      </div>
      
      <div className="relative z-20">
        <StatusBar />
      </div>

      {/* 全局 Toast 通知(右下角堆叠) */}
      <Toaster />
    </div>
  );
}
