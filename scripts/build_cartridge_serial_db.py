#!/usr/bin/env python3
"""从 Libretro serial DAT 生成应用内置的精简卡带产品号数据。"""

import argparse
import csv
import re
from pathlib import Path


GAME_RE = re.compile(r"game\s*\((.*?)(?=\n\))", re.S)
NAME_RE = re.compile(r'^\s*(?:name|comment)\s+"([^"]+)"', re.M)
SERIAL_RE = re.compile(r'^\s*serial\s+"([^"]+)"', re.M)
REGION_RE = re.compile(r'^\s*region\s+"([^"]+)"', re.M)


def region_from_name(name: str) -> str:
    regions = {"USA", "Europe", "Japan", "World", "Australia", "Asia", "Brazil", "Canada", "China", "Korea"}
    for value in re.findall(r"\(([^()]*)\)", name):
        region = value.split(",", 1)[0]
        if region in regions:
            return region
    return ""


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("dat_file", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()

    text = args.dat_file.read_text(encoding="utf-8-sig")
    rows = []
    for block in GAME_RE.findall(text):
        name_match = NAME_RE.search(block)
        serial_match = SERIAL_RE.search(block)
        if not name_match or not serial_match:
            continue
        name = name_match.group(1)
        region_match = REGION_RE.search(block)
        region = region_match.group(1) if region_match else region_from_name(name)
        rows.append((serial_match.group(1), name, region))
    args.output.parent.mkdir(parents=True, exist_ok=True)
    with args.output.open("w", encoding="utf-8", newline="") as file:
        writer = csv.writer(file, lineterminator="\n")
        writer.writerow(["serial", "name", "region"])
        writer.writerows(rows)
    print(f"已生成 {len(rows)} 条产品号数据: {args.output}")


if __name__ == "__main__":
    main()
