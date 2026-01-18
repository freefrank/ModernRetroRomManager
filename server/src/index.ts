import express, { Request, Response } from "express";
import cors from "cors";
import path from "path";
import fs from "fs";
import { fileURLToPath } from "url";
import { scanRomsDirectory } from "./rom-service.js";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

// Read package.json to get version
const packageJsonPath = path.join(__dirname, "..", "package.json");
const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, "utf-8"));
const APP_VERSION = packageJson.version || "unknown";

const app = express();
const PORT = process.env.PORT || 3001;
const ROMS_DIR = process.env.ROMS_DIR || "/roms";
const PUBLIC_DIR = process.env.PUBLIC_DIR || path.join(__dirname, "..", "public");

app.use(cors());
app.use(express.json());

const API_KEY = process.env.API_KEY || "default-dev-key";

// API Key Middleware
app.use("/api/*", (req: Request, res: Response, next) => {
  const apiKey = req.headers["x-api-key"];
  if (!apiKey || apiKey !== API_KEY) {
    res.status(401).json({ error: "Unauthorized" });
    return;
  }
  next();
});

app.get("/api/health", (_req: Request, res: Response) => {
  res.json({ status: "ok", romsDir: ROMS_DIR, version: APP_VERSION });
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

  // Security: Prevent path traversal and restrict to ROMS_DIR or config dir
  const resolvedPath = path.resolve(filePath);
  const allowedDirs = [path.resolve(ROMS_DIR)];
  // Add other allowed dirs if needed, e.g. a separate media config dir

  const isAllowed = allowedDirs.some(dir => resolvedPath.startsWith(dir));
  if (!isAllowed) {
    res.status(403).json({ error: "Access denied" });
    return;
  }

  if (!fs.existsSync(resolvedPath)) {
    res.status(404).json({ error: "File not found" });
    return;
  }

  res.sendFile(resolvedPath);
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
