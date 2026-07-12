#!/usr/bin/env python3
"""从 Libretro/No-Intro DAT 生成应用内置的精简 GBA Serial 数据。"""

import argparse
import csv
import re
from pathlib import Path


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("dat_file", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()

    content = args.dat_file.read_text(encoding="utf-8", errors="replace")
    rows: list[tuple[str, str, str]] = []
    for block in re.findall(r"game \(\n(.*?)\n\)", content, re.DOTALL):
        name = re.search(r'^\s*name "(.*)"$', block, re.MULTILINE)
        serial = re.search(r'^\s*serial "([^"]+)"$', block, re.MULTILINE)
        region = re.search(r'^\s*region "([^"]+)"$', block, re.MULTILINE)
        if name and serial and len(serial.group(1)) == 4:
            rows.append((serial.group(1), name.group(1), region.group(1) if region else ""))

    args.output.parent.mkdir(parents=True, exist_ok=True)
    with args.output.open("w", encoding="utf-8", newline="") as output:
        writer = csv.writer(output)
        writer.writerow(("serial", "name", "region"))
        writer.writerows(rows)
    print(f"已生成 {len(rows)} 条 GBA Serial 数据: {args.output}")


if __name__ == "__main__":
    main()
