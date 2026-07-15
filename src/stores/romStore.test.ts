import { beforeEach, describe, expect, it, vi } from "vitest";
import { api } from "@/lib/api";
import { useRomStore } from "@/stores/romStore";
import type { ScanDirectory } from "@/types";

const firstLibrary: ScanDirectory = {
  id: "library-one",
  name: "主游戏库",
  path: "G:/ROMS",
  isRootDirectory: true,
  metadataFormat: "auto",
  isActive: true,
};

const secondLibrary: ScanDirectory = {
  id: "library-two",
  name: "掌机收藏",
  path: "Y:/Handhelds",
  isRootDirectory: true,
  metadataFormat: "auto",
  isActive: true,
};

describe("multi-library store", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
    useRomStore.setState({
      roms: [{
        file: "old.gba",
        name: "Old Game",
        directory: "G:/ROMS/gba",
        system: "gba",
        has_temp_metadata: false,
      }],
      systemRoms: [{
        system: "gba",
        path: "G:/ROMS/gba",
        roms: [],
      }],
      availableSystems: [{
        name: "gba",
        path: "G:/ROMS/gba",
        romCount: 1,
        scrapedCount: 0,
        totalSize: 1,
      }],
      selectedSystem: "gba",
      selectedRomIds: new Set(["old.gba"]),
      scanDirectories: [
        firstLibrary,
        { ...secondLibrary, isActive: false },
      ],
      stats: { totalRoms: 1, scrapedRoms: 0, totalSize: 1 },
      isScanning: false,
      isLoadingRoms: false,
    });
  });

  it("clears the previous library before loading the selected library", async () => {
    vi.spyOn(api, "setActiveLibrary").mockResolvedValue(secondLibrary);
    vi.spyOn(api, "getDirectories").mockResolvedValue([
      { ...firstLibrary, isActive: false },
      secondLibrary,
    ]);
    vi.spyOn(api, "getLibraryRomSummary").mockResolvedValue([{
      system: "nds",
      path: "Y:/Handhelds/nds",
      romCount: 2,
      scrapedCount: 1,
      totalSize: 200,
    }]);
    vi.spyOn(api, "scanRomLibrary").mockResolvedValue([{
      system: "nds",
      path: "Y:/Handhelds/nds",
      romCount: 3,
      scrapedCount: 1,
      totalSize: 300,
    }]);

    await useRomStore.getState().activateLibrary("library-two");
    await vi.waitFor(() => expect(useRomStore.getState().stats.totalRoms).toBe(3));

    expect(api.setActiveLibrary).toHaveBeenCalledWith("library-two");
    expect(api.scanRomLibrary).toHaveBeenCalledWith(false, "library-two");
    expect(useRomStore.getState()).toMatchObject({
      roms: [],
      systemRoms: [],
      selectedSystem: null,
      availableSystems: [{ name: "nds", romCount: 3 }],
      stats: { totalRoms: 3, scrapedRoms: 1, totalSize: 300 },
    });
    expect(useRomStore.getState().selectedRomIds.size).toBe(0);
    expect(useRomStore.getState().scanDirectories[1].isActive).toBe(true);
  });
});
