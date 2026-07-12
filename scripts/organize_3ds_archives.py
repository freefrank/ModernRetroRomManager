#!/usr/bin/env python3
"""盘点 RAR/7Z/ZIP 内的 3DS 内容，并可非破坏性解包到统一目录。"""

import argparse
import csv
import re
import subprocess
from pathlib import Path


ROM_SUFFIXES = {".cia", ".3ds", ".cci"}
INVALID_CHARS = re.compile(r'[<>:"/\\|?*\x00-\x1f]')


def list_members(archive: Path) -> tuple[list[str], str]:
    result = subprocess.run(["tar", "-tf", str(archive)], capture_output=True)
    stdout = result.stdout.decode("gb18030", errors="replace")
    stderr = result.stderr.decode("gb18030", errors="replace")
    members = [line.strip() for line in stdout.splitlines() if Path(line.strip()).suffix.lower() in ROM_SUFFIXES]
    return members, stderr.strip() if result.returncode else ""


def display_name(member: str) -> str:
    path = Path(member)
    candidate = path.parent.name if path.parent.name not in {"", "."} else path.stem
    candidate = INVALID_CHARS.sub("_", candidate).strip(" ._")
    return candidate or path.stem


def extract_archive(archive: Path, output_root: Path, name: str) -> Path:
    target_dir = output_root / f"{archive.stem} - {name}"
    if target_dir.exists():
        raise FileExistsError(f"目标目录已存在: {target_dir}")
    target_dir.mkdir(parents=True)
    result = subprocess.run(["tar", "-xf", str(archive), "-C", str(target_dir)], capture_output=True)
    if result.returncode and not any(path.is_file() for path in target_dir.rglob("*")):
        raise RuntimeError(result.stderr.decode("gb18030", errors="replace").strip())
    return target_dir


def main() -> None:
    parser = argparse.ArgumentParser(description="生成 3DS 压缩包清单；加 --apply 后复制解包到新目录")
    parser.add_argument("source", type=Path)
    parser.add_argument("--report", type=Path, default=Path("3ds-archive-report.csv"))
    parser.add_argument("--output-dir", type=Path)
    parser.add_argument("--apply", action="store_true")
    parser.add_argument("--limit", type=int, help="仅处理前 N 个压缩包（用于测试）")
    args = parser.parse_args()
    if args.apply and not args.output_dir:
        parser.error("--apply 必须同时指定 --output-dir")

    rows = []
    archives = sorted(path for path in args.source.iterdir() if path.suffix.lower() in {".rar", ".7z", ".zip"})
    if args.limit is not None:
        archives = archives[: args.limit]
    for archive in archives:
        members, error = list_members(archive)
        if not members:
            rows.append([archive.name, "", "", "", "无法读取", error])
            continue
        extracted_to = None
        extract_error = None
        if args.apply:
            try:
                extracted_to = extract_archive(archive, args.output_dir, display_name(members[0]))
            except Exception as exc:
                extract_error = str(exc)
        for member in members:
            name = display_name(member)
            title_id = ""
            status = "仅清单"
            detail = "CIA 仍需使用本人主机密钥验证/解密" if Path(member).suffix.lower() == ".cia" else "需验证 NCCH 加密状态"
            if args.apply:
                status = "已复制解包" if extracted_to else "解包失败"
                detail = str(extracted_to or extract_error)
            rows.append([archive.name, member, name, title_id, status, detail])

    args.report.parent.mkdir(parents=True, exist_ok=True)
    with args.report.open("w", encoding="utf-8-sig", newline="") as output:
        writer = csv.writer(output, lineterminator="\n")
        writer.writerow(["压缩包", "内部文件", "整理名称", "Title ID", "状态", "说明"])
        writer.writerows(rows)
    print(f"已生成 {len(rows)} 条记录: {args.report}")


if __name__ == "__main__":
    main()
