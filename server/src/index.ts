import express, { Request, Response } from "express";
import cors from "cors";
import path from "path";
import fs from "fs";
import { fileURLToPath } from "url";
import { scanRomsDirectory } from "./rom-service.js";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

const app = express();
const PORT = process.env.PORT || 3001;
const ROMS_DIR = process.env.ROMS_DIR || "/roms";
const PUBLIC_DIR = process.env.PUBLIC_DIR || path.join(__dirname, "..", "public");

app.use(cors());
app.use(express.json());

app.get("/api/health", (_req: Request, res: Response) => {
  res.json({ status: "ok", romsDir: ROMS_DIR });
});

app.get("/api/roms", async (_req: Request, res: Response) => {
  try {
    const systems = await scanRomsDirectory(ROMS_DIR);
    res.json(systems);
  } catch (error) {
    res.status(500).json({ error: String(error) });
  }
});

app.get("/api/media", (req: Request, res: Response) => {
  const filePath = req.query.path as string;
  if (!filePath) {
    res.status(400).json({ error: "Missing path parameter" });
    return;
  }

  if (!fs.existsSync(filePath)) {
    res.status(404).json({ error: "File not found" });
    return;
  }

  res.sendFile(filePath);
});

if (fs.existsSync(PUBLIC_DIR)) {
  app.use(express.static(PUBLIC_DIR));
  app.get("*", (_req: Request, res: Response) => {
    res.sendFile(path.join(PUBLIC_DIR, "index.html"));
  });
}

app.listen(PORT, () => {
  console.log(`Server running on http://localhost:${PORT}`);
  console.log(`ROMs directory: ${ROMS_DIR}`);
});
