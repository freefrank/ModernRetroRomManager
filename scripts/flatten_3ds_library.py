#!/usr/bin/env python3
"""安全移除 3DSCH### 包装目录；默认仅生成计划，--apply 才移动。"""

import argparse
import csv
import re
from collections import Counter
from pathlib import Path


WRAPPER_RE = re.compile(r"^3DSCH\d+(?: \(\d+\))?$", re.IGNORECASE)


def build_plan(root: Path) -> list[dict[str, str]]:
    candidates = []
    reserved = {path.name.casefold() for path in root.iterdir() if not WRAPPER_RE.fullmatch(path.name)}
    for wrapper in sorted(root.iterdir(), key=lambda path: path.name.casefold()):
        if not wrapper.is_dir() or not WRAPPER_RE.fullmatch(wrapper.name):
            continue
        entries = list(wrapper.iterdir())
        directories = [entry for entry in entries if entry.is_dir()]
        files = [entry for entry in entries if entry.is_file()]
        if len(directories) != 1 or files:
            candidates.append({
                "wrapper": wrapper.name,
                "source": "",
                "target": "",
                "status": "跳过",
                "detail": f"包装层结构不是 1 个目录 + 0 个文件（目录 {len(directories)}，文件 {len(files)}）",
            })
            continue
        candidates.append({
            "wrapper": wrapper.name,
            "source": str(directories[0].relative_to(root)),
            "target": directories[0].name,
            "status": "计划",
            "detail": "",
        })

    duplicate_names = Counter(
        row["target"].casefold() for row in candidates if row["status"] == "计划"
    )
    claimed = set(reserved)
    for row in candidates:
        if row["status"] != "计划":
            continue
        original = row["target"]
        key = original.casefold()
        if duplicate_names[key] > 1 or key in claimed:
            row["target"] = f"{original} [{row['wrapper']}]"
            row["detail"] = "目标名称重复，追加原编号"
        claimed.add(row["target"].casefold())
    return candidates


def write_report(rows: list[dict[str, str]], report: Path) -> None:
    report.parent.mkdir(parents=True, exist_ok=True)
    with report.open("w", encoding="utf-8-sig", newline="") as output:
        writer = csv.DictWriter(
            output,
            fieldnames=["wrapper", "source", "target", "status", "detail"],
            lineterminator="\n",
        )
        writer.writeheader()
        writer.writerows(rows)


def apply_plan(root: Path, rows: list[dict[str, str]]) -> None:
    for row in rows:
        if row["status"] != "计划":
            continue
        source = root / row["source"]
        target = root / row["target"]
        wrapper = root / row["wrapper"]
        if not source.is_dir():
            row["status"], row["detail"] = "失败", "源目录不存在"
            continue
        if target.exists():
            row["status"], row["detail"] = "失败", "目标已存在，未覆盖"
            continue
        try:
            source.rename(target)
            wrapper.rmdir()
            row["status"] = "已完成"
        except OSError as exc:
            row["status"], row["detail"] = "失败", str(exc)


def rollback(root: Path, rows: list[dict[str, str]]) -> None:
    for row in reversed(rows):
        if row["status"] != "已完成":
            continue
        target = root / row["target"]
        wrapper = root / row["wrapper"]
        source = root / row["source"]
        if not target.is_dir() or source.exists():
            row["status"], row["detail"] = "回滚失败", "目标不存在或原路径已占用"
            continue
        try:
            wrapper.mkdir(exist_ok=True)
            target.rename(source)
            row["status"] = "已回滚"
        except OSError as exc:
            row["status"], row["detail"] = "回滚失败", str(exc)


def read_report(report: Path) -> list[dict[str, str]]:
    with report.open("r", encoding="utf-8-sig", newline="") as source:
        return list(csv.DictReader(source))


def main() -> None:
    parser = argparse.ArgumentParser(description="移除 3DSCH### 纯包装目录；不会覆盖或删除 ROM 文件")
    parser.add_argument("root", type=Path)
    parser.add_argument("--report", type=Path, default=Path("3ds-flatten-plan.csv"))
    parser.add_argument("--apply", action="store_true")
    parser.add_argument("--rollback", action="store_true")
    args = parser.parse_args()
    if args.apply and args.rollback:
        parser.error("--apply 与 --rollback 不能同时使用")

    if args.rollback:
        rows = read_report(args.report)
        rollback(args.root, rows)
    else:
        rows = build_plan(args.root)
        if args.apply:
            apply_plan(args.root, rows)
    write_report(rows, args.report)
    counts = Counter(row["status"] for row in rows)
    print("，".join(f"{status} {count}" for status, count in sorted(counts.items())))


if __name__ == "__main__":
    main()
