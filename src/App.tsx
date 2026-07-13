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
import { preloadMediaUrls } from "./lib/api";

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
  const { fetchScanDirectories, fetchRoms } = useRomStore();
  const initStarted = useRef(false);

  useEffect(() => {
    const init = async () => {
      // Prevent double init in StrictMode
      if (initStarted.current) return;
      initStarted.current = true;

      // 1. Load settings (theme, language, etc.)
      updateSplashText("splash.loadingSettings");
      await initFromBackend();

      // 2. Load ROM data and directories in parallel
      updateSplashText("splash.loadingRoms");
      await Promise.all([
        fetchRoms(),
        fetchScanDirectories(),
      ]);

      // 3. fetchRoms 已统一更新列表、系统与统计，避免启动流程绕过 Store 逻辑。
      const { systemRoms } = useRomStore.getState();
      const roms = systemRoms.flatMap(s => s.roms);

      // 4. Preload first 50 ROM covers BEFORE showing UI
      if (roms.length > 0) {
        updateSplashText("splash.loadingCovers");
        await preloadMediaUrls(roms, 50);
      }

      // 5. Hide splash after everything is ready
      updateSplashText("splash.ready");
      hideSplash();
    };

    init();
  }, [initFromBackend, fetchRoms, fetchScanDirectories]);

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
