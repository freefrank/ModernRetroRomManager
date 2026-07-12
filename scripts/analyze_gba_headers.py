#!/usr/bin/env python3
"""分析 ZIP 中的 GBA Header，并用 Libretro/No-Intro DAT 生成名称候选。"""

from __future__ import annotations

import argparse
import csv
import re
from collections import defaultdict
from dataclasses import dataclass
from pathlib import Path
from zipfile import BadZipFile, ZipFile


@dataclass(frozen=True)
class DatEntry:
    serial: str
    name: str
    region: str


def load_dat(path: Path) -> tuple[dict[str, list[DatEntry]], dict[str, list[DatEntry]]]:
    content = path.read_text(encoding="utf-8", errors="replace")
    by_serial: dict[str, list[DatEntry]] = defaultdict(list)
    by_prefix: dict[str, list[DatEntry]] = defaultdict(list)

    for block in re.findall(r"game \(\n(.*?)\n\)", content, re.DOTALL):
        name = re.search(r'^\s*name "(.*)"$', block, re.MULTILINE)
        serial = re.search(r'^\s*serial "([^"]+)"$', block, re.MULTILINE)
        region = re.search(r'^\s*region "([^"]+)"$', block, re.MULTILINE)
        if not name or not serial:
            continue
        entry = DatEntry(serial.group(1), name.group(1), region.group(1) if region else "")
        by_serial[entry.serial].append(entry)
        if len(entry.serial) == 4:
            by_prefix[entry.serial[:3]].append(entry)

    return dict(by_serial), dict(by_prefix)


def clean_release_name(name: str) -> str:
    return re.sub(r"\s+\(.*$", "", name).strip()


def ascii_header_text(value: bytes) -> str:
    return value.rstrip(b"\0 ").decode("ascii", errors="replace")


def analyze_zip(
    zip_path: Path,
    by_serial: dict[str, list[DatEntry]],
    by_prefix: dict[str, list[DatEntry]],
) -> dict[str, object]:
    result: dict[str, object] = {
        "zip_name": zip_path.name,
        "rom_name": "",
        "internal_title": "",
        "game_code": "",
        "software_version": "",
        "header_valid": False,
        "dat_name": "",
        "dat_conflicts": 0,
        "english_candidates": "",
        "status": "",
    }

    try:
        with ZipFile(zip_path) as archive:
            members = [
                item
                for item in archive.infolist()
                if not item.is_dir() and item.filename.lower().endswith((".gba", ".agb"))
            ]
            if not members:
                result["status"] = "ZIP 内没有 GBA ROM"
                return result

            member = max(members, key=lambda item: item.file_size)
            with archive.open(member) as rom:
                header = rom.read(0xC0)
    except (BadZipFile, OSError, RuntimeError) as error:
        result["status"] = f"读取失败: {error}"
        return result

    result["rom_name"] = member.filename
    if len(header) < 0xC0:
        result["status"] = "ROM Header 长度不足"
        return result

    title = ascii_header_text(header[0xA0:0xAC])
    game_code = ascii_header_text(header[0xAC:0xB0])
    checksum_valid = ((-sum(header[0xA0:0xBD]) - 0x19) & 0xFF) == header[0xBD]
    header_valid = header[0xB2] == 0x96 and checksum_valid
    result.update(
        internal_title=title,
        game_code=game_code,
        software_version=header[0xBC],
        header_valid=header_valid,
    )

    direct = by_serial.get(game_code, [])
    direct_names = list(dict.fromkeys(clean_release_name(entry.name) for entry in direct))
    result["dat_conflicts"] = len(direct_names)
    if len(direct_names) == 1:
        result["dat_name"] = direct_names[0]
    elif direct_names:
        # 保留所有真正不同的标题，禁止把 DAT 中第一条误当成准确结果。
        result["dat_name"] = " | ".join(direct_names)

    english = []
    if len(game_code) == 4:
        for entry in by_prefix.get(game_code[:3], []):
            if entry.serial != game_code and entry.serial[-1:] in {"E", "P"}:
                candidate = clean_release_name(entry.name)
                if candidate not in english:
                    english.append(candidate)
    result["english_candidates"] = " | ".join(english)

    if not header_valid:
        result["status"] = "Header 校验失败"
    elif not direct:
        result["status"] = "DAT 未命中"
    elif len(direct_names) > 1:
        result["status"] = "Serial 冲突，需结合内部标题"
    elif english:
        result["status"] = "已找到英文区域候选"
    else:
        result["status"] = "仅找到当前区域名称"
    return result


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("rom_directory", type=Path, help="包含 GBA ZIP 的目录")
    parser.add_argument("dat_file", type=Path, help="Libretro/No-Intro GBA DAT")
    parser.add_argument("-o", "--output", type=Path, default=Path("gba-header-report.csv"))
    args = parser.parse_args()

    by_serial, by_prefix = load_dat(args.dat_file)
    rows = [analyze_zip(path, by_serial, by_prefix) for path in sorted(args.rom_directory.glob("*.zip"))]
    fields = list(rows[0]) if rows else []
    with args.output.open("w", encoding="utf-8-sig", newline="") as output:
        writer = csv.DictWriter(output, fieldnames=fields)
        writer.writeheader()
        writer.writerows(rows)

    print(f"已分析 {len(rows)} 个 ZIP，报告: {args.output}")


if __name__ == "__main__":
    main()
