import fs from "fs";
import path from "path";

export interface RomInfo {
  file: string;
  name: string;
  description?: string;
  summary?: string;
  developer?: string;
  publisher?: string;
  genre?: string;
  players?: string;
  release?: string;
  rating?: string;
  directory: string;
  system: string;
  box_front?: string;
  box_back?: string;
  box_spine?: string;
  box_full?: string;
  cartridge?: string;
  logo?: string;
  marquee?: string;
  bezel?: string;
  gridicon?: string;
  flyer?: string;
  background?: string;
  music?: string;
  screenshot?: string;
  titlescreen?: string;
  video?: string;
}

export interface SystemRoms {
  system: string;
  path: string;
  roms: RomInfo[];
}

interface PegasusGame {
  name: string;
  file?: string;
  files: string[];
  developer?: string;
  publisher?: string;
  genre?: string;
  players?: string;
  summary?: string;
  description?: string;
  release?: string;
  rating?: string;
  sort_title?: string;
  box_front?: string;
  box_back?: string;
  box_spine?: string;
  box_full?: string;
  cartridge?: string;
  logo?: string;
  marquee?: string;
  bezel?: string;
  gridicon?: string;
  flyer?: string;
  background?: string;
  music?: string;
  screenshot?: string;
  titlescreen?: string;
  video?: string;
}

const ROM_EXTENSIONS = [
  "nes", "sfc", "smc", "gba", "gb", "gbc", "n64", "z64", "v64",
  "iso", "bin", "cue", "img", "zip", "7z", "rar",
  "md", "gen", "smd", "gg", "sms",
  "pce", "ngp", "ngc", "ws", "wsc",
  "a26", "a52", "a78", "lnx",
  "nds", "3ds", "cia",
  "psx", "pbp", "chd",
];

export async function scanRomsDirectory(romsDir: string): Promise<SystemRoms[]> {
  const systems: SystemRoms[] = [];

  if (!fs.existsSync(romsDir)) {
    return systems;
  }

  const entries = fs.readdirSync(romsDir, { withFileTypes: true });

  for (const entry of entries) {
    if (!entry.isDirectory()) continue;

    const systemPath = path.join(romsDir, entry.name);
    const systemName = entry.name;
    const format = detectMetadataFormat(systemPath);
    const roms = await getRomsFromDirectory(systemPath, format, systemName);

    if (roms.length > 0) {
      systems.push({
        system: systemName,
        path: systemPath,
        roms,
      });
    }
  }

  return systems;
}

function detectMetadataFormat(dirPath: string): string {
  if (
    fs.existsSync(path.join(dirPath, "metadata.pegasus.txt")) ||
    fs.existsSync(path.join(dirPath, "metadata.txt"))
  ) {
    return "pegasus";
  }
  if (fs.existsSync(path.join(dirPath, "gamelist.xml"))) {
    return "emulationstation";
  }
  return "none";
}

async function getRomsFromDirectory(
  dirPath: string,
  format: string,
  systemName: string
): Promise<RomInfo[]> {
  switch (format) {
    case "pegasus":
      return readPegasusRoms(dirPath, systemName);
    case "emulationstation":
      return readEmulationStationRoms(dirPath, systemName);
    default:
      return scanRomFiles(dirPath, systemName);
  }
}

function readPegasusRoms(dirPath: string, systemName: string): RomInfo[] {
  const possibleFiles = ["metadata.pegasus.txt", "metadata.txt"];

  for (const filename of possibleFiles) {
    const metadataPath = path.join(dirPath, filename);
    if (fs.existsSync(metadataPath)) {
      const content = fs.readFileSync(metadataPath, "utf-8");
      const games = parsePegasusContent(content);

      return games.map((game) => {
        const rom = pegasusGameToRomInfo(game, dirPath, systemName);
        scanMediaDirectory(rom, dirPath);
        return rom;
      });
    }
  }

  return scanRomFiles(dirPath, systemName);
}

function parsePegasusContent(content: string): PegasusGame[] {
  const games: PegasusGame[] = [];
  let currentGame: PegasusGame | null = null;
  let currentKey: string | null = null;
  let currentValue = "";

  const lines = content.split("\n");

  for (const line of lines) {
    if (line.startsWith("#") || line.trim() === "") continue;

    if (line.startsWith(" ") || line.startsWith("\t")) {
      const trimmed = line.trim();
      if (trimmed === ".") {
        currentValue += "\n\n";
      } else {
        if (currentValue) currentValue += " ";
        currentValue += trimmed;
      }
      continue;
    }

    if (currentKey && currentGame) {
      applyKeyValue(currentGame, currentKey, currentValue);
      currentKey = null;
      currentValue = "";
    }

    const colonPos = line.indexOf(":");
    if (colonPos === -1) continue;

    const key = line.slice(0, colonPos).trim().toLowerCase();
    const value = line.slice(colonPos + 1).trim();

    if (key === "game") {
      if (currentGame) games.push(currentGame);
      currentGame = { name: value, files: [] };
    } else if (key === "collection") {
      // Skip collection entries
    } else if (currentGame) {
      currentKey = key;
      currentValue = value;
    }
  }

  if (currentKey && currentGame) {
    applyKeyValue(currentGame, currentKey, currentValue);
  }
  if (currentGame) games.push(currentGame);

  return games;
}

function applyKeyValue(game: PegasusGame, key: string, value: string): void {
  const firstValue = () => value.split(/\s+/)[0];

  switch (key) {
    case "file":
      game.file = value;
      break;
    case "files":
      game.files = value.split(/\s+/);
      break;
    case "developer":
    case "developers":
      game.developer = value;
      break;
    case "publisher":
    case "publishers":
      game.publisher = value;
      break;
    case "genre":
    case "genres":
      game.genre = value;
      break;
    case "players":
      game.players = value;
      break;
    case "summary":
      game.summary = value;
      break;
    case "description":
      game.description = value;
      break;
    case "release":
      game.release = value;
      break;
    case "rating":
      game.rating = value;
      break;
    case "sort_title":
    case "sort_name":
    case "sort-by":
      game.sort_title = value;
      break;
    case "assets.boxfront":
    case "assets.box_front":
    case "assets.boxart2d":
    case "boxart":
    case "cover":
      game.box_front = firstValue();
      break;
    case "assets.boxback":
    case "assets.box_back":
      game.box_back = firstValue();
      break;
    case "assets.boxspine":
    case "assets.box_spine":
      game.box_spine = firstValue();
      break;
    case "assets.boxfull":
    case "assets.box_full":
      game.box_full = firstValue();
      break;
    case "assets.cartridge":
    case "assets.disc":
    case "assets.cart":
      game.cartridge = firstValue();
      break;
    case "assets.logo":
    case "assets.wheel":
      game.logo = firstValue();
      break;
    case "assets.marquee":
    case "assets.banner":
      game.marquee = firstValue();
      break;
    case "assets.bezel":
    case "assets.screenmarquee":
      game.bezel = firstValue();
      break;
    case "assets.gridicon":
    case "assets.steam":
    case "assets.poster":
      game.gridicon = firstValue();
      break;
    case "assets.flyer":
      game.flyer = firstValue();
      break;
    case "assets.background":
    case "assets.fanart":
      game.background = firstValue();
      break;
    case "assets.music":
      game.music = firstValue();
      break;
    case "assets.screenshot":
    case "assets.screenshots":
      game.screenshot = firstValue();
      break;
    case "assets.titlescreen":
    case "assets.title_screen":
      game.titlescreen = firstValue();
      break;
    case "assets.video":
    case "assets.videos":
      game.video = firstValue();
      break;
  }
}

function pegasusGameToRomInfo(
  game: PegasusGame,
  dirPath: string,
  systemName: string
): RomInfo {
  const resolveMediaPath = (value?: string) => {
    if (!value) return undefined;
    return path.isAbsolute(value) ? value : path.join(dirPath, value);
  };

  return {
    file: game.file || "",
    name: game.name,
    description: game.description,
    summary: game.summary,
    developer: game.developer,
    publisher: game.publisher,
    genre: game.genre,
    players: game.players,
    release: game.release,
    rating: game.rating,
    directory: dirPath,
    system: systemName,
    box_front: resolveMediaPath(game.box_front),
    box_back: resolveMediaPath(game.box_back),
    box_spine: resolveMediaPath(game.box_spine),
    box_full: resolveMediaPath(game.box_full),
    cartridge: resolveMediaPath(game.cartridge),
    logo: resolveMediaPath(game.logo),
    marquee: resolveMediaPath(game.marquee),
    bezel: resolveMediaPath(game.bezel),
    gridicon: resolveMediaPath(game.gridicon),
    flyer: resolveMediaPath(game.flyer),
    background: resolveMediaPath(game.background),
    music: resolveMediaPath(game.music),
    screenshot: resolveMediaPath(game.screenshot),
    titlescreen: resolveMediaPath(game.titlescreen),
    video: resolveMediaPath(game.video),
  };
}

function scanMediaDirectory(rom: RomInfo, dirPath: string): void {
  const mediaDir = path.join(dirPath, "media", rom.name);
  if (!fs.existsSync(mediaDir)) return;

  const entries = fs.readdirSync(mediaDir, { withFileTypes: true });

  for (const entry of entries) {
    if (!entry.isFile()) continue;

    const fullPath = path.join(mediaDir, entry.name);
    const stem = path.parse(entry.name).name.toLowerCase();

    const mapping: Record<string, keyof RomInfo> = {
      boxfront: "box_front",
      box_front: "box_front",
      boxart: "box_front",
      cover: "box_front",
      boxback: "box_back",
      box_back: "box_back",
      boxspine: "box_spine",
      box_spine: "box_spine",
      boxfull: "box_full",
      box_full: "box_full",
      cartridge: "cartridge",
      cart: "cartridge",
      disc: "cartridge",
      logo: "logo",
      wheel: "logo",
      marquee: "marquee",
      banner: "marquee",
      bezel: "bezel",
      screenmarquee: "bezel",
      gridicon: "gridicon",
      steam: "gridicon",
      poster: "gridicon",
      flyer: "flyer",
      background: "background",
      fanart: "background",
      music: "music",
      screenshot: "screenshot",
      screenshots: "screenshot",
      screen: "screenshot",
      titlescreen: "titlescreen",
      title_screen: "titlescreen",
      title: "titlescreen",
      video: "video",
      videos: "video",
    };

    const field = mapping[stem];
    if (field && !rom[field]) {
      (rom as unknown as Record<string, unknown>)[field] = fullPath;
    }
  }
}

function readEmulationStationRoms(dirPath: string, systemName: string): RomInfo[] {
  // TODO: Implement EmulationStation XML parsing
  return scanRomFiles(dirPath, systemName);
}

function scanRomFiles(dirPath: string, systemName: string): RomInfo[] {
  const roms: RomInfo[] = [];

  if (!fs.existsSync(dirPath)) return roms;

  const entries = fs.readdirSync(dirPath, { withFileTypes: true });

  for (const entry of entries) {
    if (!entry.isFile()) continue;

    const ext = path.extname(entry.name).slice(1).toLowerCase();
    if (!ROM_EXTENSIONS.includes(ext)) continue;

    const name = path.parse(entry.name).name;

    roms.push({
      file: entry.name,
      name,
      directory: dirPath,
      system: systemName,
    });
  }

  return roms;
}
