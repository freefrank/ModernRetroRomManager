#!/usr/bin/env python3
"""normalize_jy6d_dz_exports.py

Normalize CSV exports under scripts/data/jy6d-dz into a consistent mapping format.

Input: CSV files produced by scripts/scrape_jy6d_dz.py
Output:
  scripts/data/jy6d-dz/normalized/{system}.csv
  scripts/data/jy6d-dz/normalized/all.csv

Goals:
- Drop redundant `system` column (system is inferred from filename)
- Standardize to (english_name, chinese_name, source_id, extra_json)
- Keep extra columns without forcing a schema (stored as JSON)
- Repair common mojibake cases for Chinese strings from the source site

Usage:
  python scripts/normalize_jy6d_dz_exports.py
  python scripts/normalize_jy6d_dz_exports.py --in "scripts/data/jy6d-dz" --out "scripts/data/jy6d-dz/normalized"
"""

from __future__ import annotations

import argparse
import csv
import json
import sys
from pathlib import Path
from typing import Dict, Iterable, List, Optional, Tuple


def _read_csv(path: Path) -> Tuple[List[str], List[List[str]]]:
    with path.open("r", encoding="utf-8-sig", newline="") as f:
        r = csv.reader(f)
        try:
            headers = next(r)
        except StopIteration:
            return ([], [])
        rows = [row for row in r]
    return (headers, rows)


def _write_csv(path: Path, headers: List[str], rows: Iterable[List[str]]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8", newline="") as f:
        w = csv.writer(f)
        w.writerow(headers)
        for row in rows:
            w.writerow(row)


def _attempt_mojibake_fix(s: str) -> str:
    """Try to repair common encoding-mismatch mojibake.

    The site sometimes stores Chinese strings in a wrong decoded form.
    A common reversible pattern is: UTF-8 bytes decoded as GBK -> weird Unicode.
    Reverse by encoding as GBK then decoding as UTF-8.
    """

    if not s:
        return s

    ss = s.strip()
    # Quick exit: contains real Han characters already.
    if any("\u4e00" <= ch <= "\u9fff" for ch in ss):
        return s

    # Try GBK->UTF8 reversal
    for enc in ("gb18030", "gbk", "cp936"):
        try:
            b = ss.encode(enc)
            fixed = b.decode("utf-8")
            # Must introduce some CJK to be considered a fix
            if any("\u4e00" <= ch <= "\u9fff" for ch in fixed):
                return fixed
        except Exception:
            pass

    return s


def _col_index(headers: List[str], candidates: List[str]) -> Optional[int]:
    lower = [h.strip().lower() for h in headers]
    for c in candidates:
        cc = c.strip().lower()
        if cc in lower:
            return lower.index(cc)
    return None


def _pick_mapping_columns(headers: List[str]) -> Tuple[Optional[int], Optional[int], Optional[int]]:
    # english, chinese, id
    en_idx = _col_index(headers, [
        "english_name",
        "game_name",
        "英文名",
        "英文名称",
    ])
    cn_idx = _col_index(headers, [
        "chinese_name",
        "ch_name",
        "中文名",
        "中文名称",
    ])
    id_idx = _col_index(headers, ["id", "game_id", "UMD_ID", "umd_id"])

    # Special case: MD mapping-only output already has english_name/chinese_name
    return (en_idx, cn_idx, id_idx)


def normalize_file(csv_path: Path, out_dir: Path) -> Tuple[str, int]:
    system = csv_path.stem.lower()
    headers, rows = _read_csv(csv_path)
    if not headers:
        return (system, 0)

    # Drop redundant system col if present as first header.
    if headers and headers[0].strip().lower() == "system":
        headers = headers[1:]
        rows = [r[1:] if len(r) > 0 else [] for r in rows]

    en_idx, cn_idx, id_idx = _pick_mapping_columns(headers)

    out_headers = ["english_name", "chinese_name", "source_id", "extra_json"]
    out_rows: List[List[str]] = []

    for r in rows:
        rr = r + [""] * (len(headers) - len(r))
        en = rr[en_idx].strip() if en_idx is not None and en_idx < len(rr) else ""
        cn = rr[cn_idx].strip() if cn_idx is not None and cn_idx < len(rr) else ""
        sid = rr[id_idx].strip() if id_idx is not None and id_idx < len(rr) else ""

        cn = _attempt_mojibake_fix(cn)

        extra: Dict[str, str] = {}
        for i, h in enumerate(headers):
            if i in (en_idx, cn_idx, id_idx):
                continue
            v = rr[i].strip() if i < len(rr) else ""
            if v:
                extra[h] = v

        out_rows.append([
            en,
            cn,
            sid,
            json.dumps(extra, ensure_ascii=False, separators=(",", ":")) if extra else "",
        ])

    out_path = out_dir / f"{system}.csv"
    _write_csv(out_path, out_headers, out_rows)
    return (system, len(out_rows))


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--in", dest="in_dir", default="scripts/data/jy6d-dz", help="input dir")
    ap.add_argument("--out", dest="out_dir", default="scripts/data/jy6d-dz/normalized", help="output dir")
    args = ap.parse_args()

    in_dir = Path(args.in_dir)
    out_dir = Path(args.out_dir)
    out_dir.mkdir(parents=True, exist_ok=True)

    csv_files = sorted([p for p in in_dir.glob("*.csv") if p.is_file()])
    if not csv_files:
        print(f"No CSV files found in {in_dir}", file=sys.stderr)
        return 2

    combined_headers = ["system", "english_name", "chinese_name", "source_id", "extra_json"]
    combined_rows: List[List[str]] = []

    for p in csv_files:
        system, count = normalize_file(p, out_dir)
        # Append to combined
        norm_path = out_dir / f"{system}.csv"
        with norm_path.open("r", encoding="utf-8", newline="") as f:
            r = csv.reader(f)
            next(r, None)
            for row in r:
                combined_rows.append([system, *row])
        print(f"OK  {p.name} -> {system}.csv ({count} rows)")

    _write_csv(out_dir / "all.csv", combined_headers, combined_rows)
    print(f"DONE all.csv ({len(combined_rows)} rows)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
