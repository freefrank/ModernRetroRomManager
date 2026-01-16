import { useEffect } from "react";
import { Routes, Route } from "react-router-dom";
import { useAppStore } from "@/stores/appStore";
import { Layout } from "./components/layout";
import Library from "./pages/Library";
import Scraper from "./pages/Scraper";
import Import from "./pages/Import";
import Settings from "./pages/Settings";

export default function App() {
  const { theme } = useAppStore();

  useEffect(() => {
    if (theme === "dark") {
      document.documentElement.classList.add("dark");
    } else {
      document.documentElement.classList.remove("dark");
    }
  }, [theme]);

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
