import { beforeEach, describe, expect, it, vi } from "vitest";

const invoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({ invoke }));

import { scraperApi } from "./api";

describe("export option wiring", () => {
  beforeEach(() => {
    invoke.mockReset();
    invoke.mockResolvedValue(undefined);
    Object.defineProperty(window, "__TAURI_INTERNALS__", {
      configurable: true,
      value: {},
    });
  });

  it("passes the selected mode to platform and library exports", async () => {
    await scraperApi.exportScrapedData("gba", "Y:\\gba", "pegasus", "D:\\out", "chinese");
    await scraperApi.exportLibraryScrapedData(
      "library-gba",
      "emulationstation",
      "D:\\out",
      "original",
      true,
      ["Y:\\gba", "Y:\\psp"],
    );

    expect(invoke).toHaveBeenNthCalledWith(1, "export_scraped_data", {
      system: "gba",
      directory: "Y:\\gba",
      format: "pegasus",
      targetDirectory: "D:\\out",
      nameMode: "chinese",
      romAssetsOnly: false,
    });
    expect(invoke).toHaveBeenNthCalledWith(2, "export_library_scraped_data", {
      libraryId: "library-gba",
      format: "emulationstation",
      targetDirectory: "D:\\out",
      nameMode: "original",
      romAssetsOnly: true,
      systemPaths: ["Y:\\gba", "Y:\\psp"],
      syncDelete: false,
    });
  });
});
