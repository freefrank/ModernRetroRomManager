#!/usr/bin/env python3
"""从 3DSDB XML 生成应用内置的精简 Title ID 数据。"""

import argparse
import csv
import xml.etree.ElementTree as ET
from pathlib import Path


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("xml_file", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()

    root = ET.parse(args.xml_file).getroot()
    rows = []
    for release in root.findall("release"):
        title_id = (release.findtext("titleid") or "").strip().upper()
        name = (release.findtext("name") or "").strip()
        serial = (release.findtext("serial") or "").strip().upper()
        region = (release.findtext("region") or "").strip().upper()
        if len(title_id) == 16 and name:
            rows.append((title_id, serial, name, region))

    args.output.parent.mkdir(parents=True, exist_ok=True)
    with args.output.open("w", encoding="utf-8", newline="") as output:
        writer = csv.writer(output, lineterminator="\n")
        writer.writerow(["title_id", "serial", "name", "region"])
        writer.writerows(rows)
    print(f"已生成 {len(rows)} 条 3DS Title ID 数据: {args.output}")


if __name__ == "__main__":
    main()
