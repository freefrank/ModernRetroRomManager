import { useEffect, useRef } from "react";
import { Routes, Route } from "react-router-dom";
import i18next from "i18next";
import { Layout } from "./components/layout";
import Library from "./pages/Library";
import Scraper from "./pages/Scraper";
import Import from "./pages/Import";
import Settings from "./pages/Settings";
import CnRomTools from "./pages/CnRomTools";
import { useAppStore } from "./stores/appStore";
import { useRomStore } from "./stores/romStore";
import { api, preloadMediaUrls } from "./lib/api";

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
  const { fetchScanDirectories } = useRomStore();
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
      const [systemRoms] = await Promise.all([
        api.getRoms(),
        fetchScanDirectories(),
      ]);

      // 3. 展平 ROM 列表并更新 store（包含统计信息）
      const roms = systemRoms.flatMap(s => s.roms);
      const totalRoms = roms.length;
      
      useRomStore.setState({
        systemRoms,
        availableSystems: systemRoms.map(s => ({ name: s.system, romCount: s.roms.length })),
        roms,
        isLoadingRoms: false,
        stats: {
          totalRoms,
          scrapedRoms: 0,
          totalSize: 0,
        },
      });

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
  }, [initFromBackend, fetchScanDirectories]);

  // Show nothing while initializing (splash is visible)
  if (!initialized) {
    return null;
  }

  return (
    <Routes>
      <Route path="/" element={<Layout />}>
        <Route index element={<Library />} />
        <Route path="scraper" element={<Scraper />} />
        <Route path="cn-tools" element={<CnRomTools />} />
        <Route path="import" element={<Import />} />
        <Route path="settings" element={<Settings />} />
      </Route>
    </Routes>
  );
}
