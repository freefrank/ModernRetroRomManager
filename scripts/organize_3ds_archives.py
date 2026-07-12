#!/usr/bin/env python3
"""盘点 RAR/7Z/ZIP 内的 3DS 内容，并可非破坏性解包到统一目录。"""

import argparse
import csv
import re
import subprocess
from pathlib import Path


ROM_SUFFIXES = {".cia", ".3ds", ".cci", ".cxi", ".app", ".ncch", ".3dsx"}
INVALID_CHARS = re.compile(r'[<>:"/\\|?*\x00-\x1f]')


def tar_args(password: str | None) -> list[str]:
    return ["tar", "--passphrase", password] if password else ["tar"]


def list_members(archive: Path, password: str | None) -> tuple[list[str], str]:
    result = subprocess.run([*tar_args(password), "-tf", str(archive)], capture_output=True)
    stdout = result.stdout.decode("gb18030", errors="replace")
    stderr = result.stderr.decode("gb18030", errors="replace")
    members = [line.strip() for line in stdout.splitlines() if Path(line.strip()).suffix.lower() in ROM_SUFFIXES]
    return members, stderr.strip() if result.returncode else ""


def display_name(member: str) -> str:
    path = Path(member)
    candidate = path.parent.name if path.parent.name not in {"", "."} else path.stem
    candidate = INVALID_CHARS.sub("_", candidate).strip(" ._")
    return candidate or path.stem


def extract_archive(archive: Path, output_root: Path, name: str, password: str | None) -> Path:
    target_dir = output_root / f"{archive.stem} - {name}"
    if target_dir.exists():
        raise FileExistsError(f"目标目录已存在: {target_dir}")
    target_dir.mkdir(parents=True)
    result = subprocess.run([*tar_args(password), "-xf", str(archive), "-C", str(target_dir)], capture_output=True)
    if result.returncode and not any(path.is_file() for path in target_dir.rglob("*")):
        raise RuntimeError(result.stderr.decode("gb18030", errors="replace").strip())
    return target_dir


def compatibility_note(path: Path) -> str:
    suffix = path.suffix.lower()
    if suffix == ".cia":
        return "Azahar：通过安装 CIA 使用；Panda3DS：不能直接加载"
    if suffix in {".3ds", ".cci", ".cxi", ".app", ".ncch", ".3dsx"}:
        return "Azahar/Panda3DS 可直接加载；加密内容需本人主机导出的密钥"
    return "非直接启动内容；按原目录保留并人工确认补丁类型"


def inventory_directory(source: Path, max_depth: int) -> list[list[object]]:
    """只枚举有限层级的目录项，不读取 ROM 内容。"""
    rows: list[list[object]] = []
    pending = [(source, 0)]
    while pending:
        directory, depth = pending.pop()
        try:
            entries = list(directory.iterdir())
        except OSError as exc:
            rows.append([str(directory), "", "", "", "读取失败", str(exc)])
            continue
        for entry in entries:
            relative = entry.relative_to(source)
            if entry.is_dir():
                if depth < max_depth:
                    pending.append((entry, depth + 1))
                continue
            suffix = entry.suffix.lower()
            if suffix in ROM_SUFFIXES:
                rows.append([relative.parts[0], str(relative), entry.stem, "", "只读盘点", compatibility_note(entry)])
    return rows


def main() -> None:
    parser = argparse.ArgumentParser(description="生成 3DS 压缩包清单；加 --apply 后复制解包到新目录")
    parser.add_argument("source", type=Path)
    parser.add_argument("--report", type=Path, default=Path("3ds-archive-report.csv"))
    parser.add_argument("--output-dir", type=Path)
    parser.add_argument("--apply", action="store_true")
    parser.add_argument("--limit", type=int, help="仅处理前 N 个压缩包（用于测试）")
    parser.add_argument("--password", help="RAR/7Z 解压密码")
    parser.add_argument("--inventory-dir", action="store_true", help="将 source 作为已解压目录，只做有限层级只读盘点")
    parser.add_argument("--max-depth", type=int, default=3, help="目录盘点最大深度（默认 3）")
    args = parser.parse_args()
    if args.apply and not args.output_dir:
        parser.error("--apply 必须同时指定 --output-dir")

    if args.inventory_dir:
        rows = inventory_directory(args.source, args.max_depth)
        args.report.parent.mkdir(parents=True, exist_ok=True)
        with args.report.open("w", encoding="utf-8-sig", newline="") as output:
            writer = csv.writer(output, lineterminator="\n")
            writer.writerow(["顶层目录", "相对路径", "整理名称", "Title ID", "状态", "兼容性建议"])
            writer.writerows(rows)
        print(f"已生成 {len(rows)} 条只读目录记录: {args.report}")
        return

    rows = []
    archives = sorted(path for path in args.source.iterdir() if path.suffix.lower() in {".rar", ".7z", ".zip"})
    if args.limit is not None:
        archives = archives[: args.limit]
    for archive in archives:
        members, error = list_members(archive, args.password)
        if not members:
            rows.append([archive.name, "", "", "", "无法读取", error])
            continue
        extracted_to = None
        extract_error = None
        if args.apply:
            try:
                extracted_to = extract_archive(archive, args.output_dir, display_name(members[0]), args.password)
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
