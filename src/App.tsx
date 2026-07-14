import { useEffect, useRef } from "react";
import { Routes, Route, Navigate } from "react-router-dom";
import i18next from "i18next";
import { Layout } from "./components/layout";
import Library from "./pages/Library";
import LibraryShelf from "./pages/LibraryShelf";
import Scraper from "./pages/Scraper";
import Import from "./pages/Import";
import Settings from "./pages/Settings";
import CnRomTools from "./pages/CnRomTools";
import { useAppStore } from "./stores/appStore";
import { useRomStore } from "./stores/romStore";

// Update splash screen status text
const updateSplashText = (key: string) => {
  const el = document.getElementById("splash-text");
  if (el) {
    el.textContent = i18next.t(key);
  }
};

// Hide the HTML splash screen
const hideSplash = () => {
  const splash = document.getElementById("splash");
  if (splash) {
    splash.classList.add("fade-out");
    setTimeout(() => splash.remove(), 300);
  }
};

export default function App() {
  const { initialized, initFromBackend } = useAppStore();
  const { fetchScanDirectories, fetchLibrarySummary, scanLibrary, initializeScanProgress } = useRomStore();
  const initStarted = useRef(false);

  useEffect(() => {
    const init = async () => {
      // Prevent double init in StrictMode
      if (initStarted.current) return;
      initStarted.current = true;

      // 1. Load settings (theme, language, etc.)
      updateSplashText("splash.loadingSettings");
      await initFromBackend();

      // 设置和主题就绪后立即显示应用外壳；ROM 索引在后台装载/校验。
      updateSplashText("splash.ready");
      hideSplash();
      await initializeScanProgress();

      // 2. Load ROM data and directories in parallel
      await Promise.all([
        fetchLibrarySummary(),
        fetchScanDirectories(),
      ]);

      // 3. 已有库启动时按系统做后台增量校验。
      await scanLibrary(false);
    };

    init();
  }, [initFromBackend, fetchLibrarySummary, fetchScanDirectories, initializeScanProgress, scanLibrary]);

  useEffect(() => {
    // WebView 的原生右键菜单会暴露浏览器式操作，与桌面应用交互不一致。
    // 在捕获阶段统一接管，后续需要业务右键菜单时可由组件自行渲染。
    const preventWebViewMenu = (event: MouseEvent) => event.preventDefault();
    document.addEventListener("contextmenu", preventWebViewMenu, { capture: true });
    return () => document.removeEventListener("contextmenu", preventWebViewMenu, { capture: true });
  }, []);

  // Show nothing while initializing (splash is visible)
  if (!initialized) {
    return null;
  }

  return (
    <Routes>
      <Route path="/" element={<Layout />}>
        <Route index element={<Navigate to="/library" replace />} />
        <Route path="library" element={<LibraryShelf />} />
        <Route path="library/:systemId" element={<Library />} />
        <Route path="scraper" element={<Scraper />} />
        <Route path="cn-tools" element={<CnRomTools />} />
        <Route path="import" element={<Import />} />
        <Route path="settings" element={<Settings />} />
      </Route>
    </Routes>
  );
}
