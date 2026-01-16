import { Routes, Route } from "react-router-dom";
import { Layout } from "./components/layout";
import Library from "./pages/Library";
import Scraper from "./pages/Scraper";
import Import from "./pages/Import";
import Settings from "./pages/Settings";

export default function App() {
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
