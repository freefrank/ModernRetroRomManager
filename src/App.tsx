import { useEffect } from "react";
import { Routes, Route } from "react-router-dom";
import { Layout } from "./components/layout";
import Library from "./pages/Library";
import Scraper from "./pages/Scraper";
import Import from "./pages/Import";
import Settings from "./pages/Settings";
import { useAppStore } from "./stores/appStore";

export default function App() {
  const { initialized, initFromBackend } = useAppStore();

  useEffect(() => {
    if (!initialized) {
      initFromBackend();
    }
  }, [initialized, initFromBackend]);

  return (
    <Routes>
      <Route path="/" element={<Layout />}>
        <Route index element={<Library />} />
        <Route path="scraper" element={<Scraper />} />
        <Route path="import" element={<Import />} />
        <Route path="settings" element={<Settings />} />
      </Route>
    </Routes>
  );
}

