#!/usr/bin/env python3
"""scrape_jy6d_dz.py

Scrape ROM CN/EN mapping tables from:
  https://emu.jy6d.com/dz/

The site has per-platform pages (e.g. /dz/md/, /dz/psp/ ...) containing a
"数据列表" table. This script downloads those tables and exports them as CSV.

Design goals:
- No third-party deps (stdlib only)
- Resilient to different column sets (some pages have "编号/英文名/中文名",
  some have "中英对照/ROM名称", etc.)
- Save UTF-8 CSV with headers

Examples:
  python scripts/scrape_jy6d_dz.py --list
  python scripts/scrape_jy6d_dz.py --system psp
  python scripts/scrape_jy6d_dz.py --all
  python scripts/scrape_jy6d_dz.py --all --outdir "data/jy6d-dz"
"""

from __future__ import annotations

import argparse
import csv
import json
import re
import sys
import time
import urllib.error
import urllib.parse
import urllib.request
from dataclasses import dataclass
from html.parser import HTMLParser
from pathlib import Path
from typing import List, Optional, Tuple


# Note: the site is often reachable via HTTP but times out on HTTPS.
BASE_URL = "http://emu.jy6d.com/dz/"


def _log(msg: str, *, verbose: bool = True) -> None:
    if not verbose:
        return
    print(msg, flush=True)


def _http_get(
    url: str,
    *,
    timeout: int = 30,
    retries: int = 2,
    verbose: bool = True,
) -> str:
    last_err: Optional[Exception] = None
    for attempt in range(1, retries + 2):
        try:
            _log(f"[HTTP] GET {url} (attempt {attempt}/{retries + 1})", verbose=verbose)
            req = urllib.request.Request(
                url,
                headers={
                    "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120 Safari/537.36",
                    "Accept": "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
                },
                method="GET",
            )
            with urllib.request.urlopen(req, timeout=timeout) as resp:
                # Try a couple common encodings; default to utf-8.
                raw = resp.read()
                _log(f"[HTTP] {len(raw)} bytes", verbose=verbose)
                for enc in ("utf-8", "gb18030"):
                    try:
                        return raw.decode(enc)
                    except UnicodeDecodeError:
                        continue
                return raw.decode("utf-8", errors="replace")
        except Exception as e:
            last_err = e
            _log(f"[HTTP] error: {e}", verbose=verbose)
            if attempt < retries + 1:
                time.sleep(0.8)
            continue

    assert last_err is not None
    raise last_err


def _http_get_bytes(
    url: str,
    *,
    timeout: int = 30,
    retries: int = 2,
    verbose: bool = True,
) -> bytes:
    last_err: Optional[Exception] = None
    for attempt in range(1, retries + 2):
        try:
            _log(f"[HTTP] GET {url} (attempt {attempt}/{retries + 1})", verbose=verbose)
            req = urllib.request.Request(
                url,
                headers={
                    "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120 Safari/537.36",
                    "Accept": "*/*",
                },
                method="GET",
            )
            with urllib.request.urlopen(req, timeout=timeout) as resp:
                raw = resp.read()
                _log(f"[HTTP] {len(raw)} bytes", verbose=verbose)
                return raw
        except Exception as e:
            last_err = e
            _log(f"[HTTP] error: {e}", verbose=verbose)
            if attempt < retries + 1:
                time.sleep(0.8)
            continue

    assert last_err is not None
    raise last_err


def _decode_text(raw: bytes) -> str:
    # Handle UTF-8 BOM and common Chinese encodings.
    for enc in ("utf-8-sig", "utf-8", "gb18030"):
        try:
            return raw.decode(enc)
        except UnicodeDecodeError:
            continue
    return raw.decode("utf-8", errors="replace")


def _try_fetch_system_json(
    system: SystemLink,
    *,
    timeout: int,
    retries: int,
    verbose: bool,
) -> Optional[Tuple[List[str], List[List[str]]]]:
    # Observed JSON endpoints are not fully consistent.
    # Most: {system}/{system}.json
    # Known exceptions:
    # - psvall -> psv_all.json
    # - psvdlc -> psv_dlc.json
    overrides = {
        "psvall": "psv_all.json",
        "psvdlc": "psv_dlc.json",
    }

    base = system.url if system.url.endswith("/") else system.url + "/"
    candidates = []
    if system.key in overrides:
        candidates.append(overrides[system.key])
    candidates.append(f"{system.key}.json")

    raw: Optional[bytes] = None
    json_url: Optional[str] = None
    for fname in candidates:
        u = urllib.parse.urljoin(base, fname)
        try:
            raw = _http_get_bytes(u, timeout=timeout, retries=retries, verbose=verbose)
            json_url = u
            break
        except urllib.error.HTTPError as e:
            if e.code == 404:
                _log(f"[JSON] 404 {u}", verbose=verbose)
                continue
            _log(f"[JSON] error {u}: {e}", verbose=verbose)
            return None
        except Exception as e:
            _log(f"[JSON] not available: {e}", verbose=verbose)
            return None

    if raw is None or json_url is None:
        return None

    text = _decode_text(raw)
    try:
        data = json.loads(text)
    except Exception as e:
        _log(f"[JSON] parse failed: {e}", verbose=verbose)
        return None

    if not isinstance(data, list) or not data:
        _log("[JSON] unexpected format (not non-empty list)", verbose=verbose)
        return None

    if not isinstance(data[0], dict):
        _log("[JSON] unexpected row type (not dict)", verbose=verbose)
        return None

    # Build stable header order: game_id, game_name, ch_name first; then remaining keys sorted.
    keys = set()
    for row in data:
        if isinstance(row, dict):
            keys.update(row.keys())
    preferred = ["game_id", "game_name", "ch_name", "UMD_ID", "date"]
    rest = sorted(k for k in keys if k not in preferred)
    headers = [k for k in preferred if k in keys] + rest

    rows: List[List[str]] = []
    for row in data:
        if not isinstance(row, dict):
            continue
        out_row: List[str] = []
        for k in headers:
            v = row.get(k, "")
            if isinstance(v, list):
                out_row.append("|".join(str(x) for x in v))
            else:
                out_row.append(str(v) if v is not None else "")
        rows.append(out_row)

    _log(f"[JSON] {json_url} rows={len(rows)} cols={len(headers)}", verbose=verbose)
    return (headers, rows)


class _TableExtractor(HTMLParser):
    def __init__(self) -> None:
        super().__init__()
        self._in_table = 0
        self._in_tr = False
        self._in_cell = False
        self._cell_parts: List[str] = []
        self._current_row: List[str] = []
        self.tables: List[List[List[str]]] = []
        self._current_table: List[List[str]] = []

    def handle_starttag(self, tag: str, attrs):
        if tag == "table":
            self._in_table += 1
            if self._in_table == 1:
                self._current_table = []

        if self._in_table > 0 and tag == "tr":
            self._in_tr = True
            self._current_row = []

        if self._in_table > 0 and self._in_tr and tag in ("td", "th"):
            self._in_cell = True
            self._cell_parts = []

        # Preserve line breaks in cells for pages that pack many entries into one <td>
        if self._in_cell and tag == "br":
            self._cell_parts.append("\n")

        # Many pages use <p> blocks for each entry; treat them as line breaks.
        if self._in_cell and tag == "p":
            # Avoid leading blank line
            if self._cell_parts and not self._cell_parts[-1].endswith("\n"):
                self._cell_parts.append("\n")

    def handle_endtag(self, tag: str):
        if self._in_cell and tag == "p":
            if self._cell_parts and not self._cell_parts[-1].endswith("\n"):
                self._cell_parts.append("\n")

        if self._in_table > 0 and self._in_tr and tag in ("td", "th"):
            self._in_cell = False
            cell = "".join(self._cell_parts)
            cell = cell.replace("\r\n", "\n").replace("\r", "\n")
            # Collapse spaces/tabs but keep newlines
            cell = re.sub(r"[ \t\f\v]+", " ", cell)
            cell = re.sub(r"\n\s+", "\n", cell)
            cell = cell.strip()
            self._current_row.append(cell)

        if self._in_table > 0 and tag == "tr":
            self._in_tr = False
            if any(c.strip() for c in self._current_row):
                self._current_table.append(self._current_row)
            self._current_row = []

        if tag == "table" and self._in_table > 0:
            self._in_table -= 1
            if self._in_table == 0:
                if self._current_table:
                    self.tables.append(self._current_table)
                self._current_table = []

    def handle_data(self, data: str):
        if self._in_cell:
            self._cell_parts.append(data)


def _expand_single_cell_rows(headers: List[str], rows: List[List[str]]) -> List[List[str]]:
    # Some pages (e.g. MD) put all entries inside one <td> with many lines.
    if len(rows) != 1 or not headers:
        return rows
    if not rows[0] or len(rows[0]) < 1:
        return rows

    cell0 = rows[0][0]
    if "\n" not in cell0:
        return rows

    lines = [ln.strip() for ln in cell0.split("\n") if ln.strip()]
    # Heuristic: must look like a long list, otherwise keep original.
    if len(lines) < 10:
        return rows

    expanded: List[List[str]] = []
    for ln in lines:
        expanded.append([ln] + [""] * (len(headers) - 1))
    return expanded


def _expand_two_column_paired_rows(headers: List[str], rows: List[List[str]]) -> List[List[str]]:
    # Some pages (notably MD) pack all entries for two columns into a single <tr>,
    # using <br> inside each <td>.
    if len(headers) < 2 or not rows:
        return rows

    def split_lines(s: str) -> List[str]:
        return [ln.strip() for ln in s.replace("\r\n", "\n").split("\n") if ln.strip()]

    expanded: List[List[str]] = []
    changed = False

    for r in rows:
        rr = r + [""] * (len(headers) - len(r))
        left = split_lines(rr[0])
        right = split_lines(rr[1])

        # Only expand when at least one column contains multiple lines.
        if len(left) <= 1 and len(right) <= 1:
            expanded.append(rr)
            continue

        changed = True
        n = max(len(left), len(right))
        for i in range(n):
            row_out = rr.copy()
            row_out[0] = left[i] if i < len(left) else ""
            row_out[1] = right[i] if i < len(right) else ""
            expanded.append(row_out)

    return expanded if changed else rows


_RE_CJK = re.compile(r"[\u4e00-\u9fff]")


def _split_cn_en_pair(s: str) -> Tuple[str, str]:
    """Split a line like:
    'Light Crusader (JE) 光之十字军战士(日欧)'
    into (english, chinese).

    Heuristic: the Chinese part starts at the first CJK character.
    """

    ss = re.sub(r"\s+", " ", (s or "").strip())
    if not ss:
        return ("", "")

    m = _RE_CJK.search(ss)
    if not m:
        return (ss, "")

    i = m.start()

    # If an ASCII token is directly attached to the first CJK char (e.g. "EA..."),
    # treat that token as part of the Chinese name.
    # Example: "EA Hockey League (U) EA..." -> english="EA Hockey League (U)", chinese="EA..."
    j = i
    while j > 0 and ss[j - 1].isascii() and re.match(r"[A-Za-z0-9&._+\-]", ss[j - 1]):
        j -= 1
    token = ss[j:i]
    if j < i and token and token.isupper() and len(token) <= 4 and (j == 0 or ss[j - 1] == " "):
        i = j

    en = ss[:i].rstrip()
    cn = ss[i:].lstrip()
    return (en, cn)


def _extract_cn_en_mapping(headers: List[str], rows: List[List[str]]) -> Tuple[List[str], List[List[str]]]:
    """If a table contains a '中英对照' column, extract it as (english_name, chinese_name).

    This is primarily for pages like MD where the main value is a packed '中英对照' text.
    """

    if "中英对照" not in headers:
        return (headers, rows)

    idx = headers.index("中英对照")
    out_headers = ["english_name", "chinese_name"]
    out_rows: List[List[str]] = []

    for r in rows:
        if idx >= len(r):
            continue
        cell = (r[idx] or "").strip()
        if not cell:
            continue
        en, cn = _split_cn_en_pair(cell)
        if not en and not cn:
            continue
        out_rows.append([en, cn])

    return (out_headers, out_rows)


@dataclass
class SystemLink:
    key: str
    url: str
    title: str


def _extract_system_links(index_html: str, base_url: str) -> List[SystemLink]:
    # The index page contains links to per-platform pages.
    # Observed formats:
    # - href="/dz/psp/"
    # - href="psp" / href="psp/" (relative to /dz/)
    links: List[SystemLink] = []

    # Absolute-ish paths under /dz/
    for m in re.finditer(
        r"href=\"(/dz/([a-z0-9]+?)/?)\"[^>]*>(.*?)</a>",
        index_html,
        re.IGNORECASE | re.DOTALL,
    ):
        href = m.group(1)
        key = m.group(2).lower()
        title = re.sub(r"<.*?>", "", m.group(3))
        title = re.sub(r"\s+", " ", title).strip()
        # Normalize to /dz/{key}/
        url = urllib.parse.urljoin(base_url, f"{key}/")
        if key and url and all(l.key != key for l in links):
            links.append(SystemLink(key=key, url=url, title=title))

    # Relative links like href="psp" (common on this site)
    for m in re.finditer(r"href=\"([a-z0-9]+)(/?)\"[^>]*>(.*?)</a>", index_html, re.IGNORECASE | re.DOTALL):
        key = m.group(1).lower()
        if key in ("javascript", "#"):
            continue
        # Avoid non-system nav links
        if key in ("all", "jd", "quanji", "class", "dz", "article", "list"):
            continue
        title = re.sub(r"<.*?>", "", m.group(3))
        title = re.sub(r"\s+", " ", title).strip()
        url = urllib.parse.urljoin(base_url, f"{key}/")
        if key and url and all(l.key != key for l in links):
            links.append(SystemLink(key=key, url=url, title=title))

    # Fallback: sometimes links are not wrapped as above; also accept plain /dz/{key}/ occurrences.
    if not links:
        for m in re.finditer(r"/dz/([a-z0-9]+?)/", index_html, re.IGNORECASE):
            key = m.group(1).lower()
            url = urllib.parse.urljoin(base_url, f"{key}/")
            if key and all(l.key != key for l in links):
                links.append(SystemLink(key=key, url=url, title=key))

    # Filter obvious non-system pages
    links = [l for l in links if l.key not in ("dz", "all", "index")]
    return links


def _pick_main_table(tables: List[List[List[str]]]) -> Optional[List[List[str]]]:
    if not tables:
        return None

    # Prefer a table whose header row contains known columns.
    def score(tbl: List[List[str]]) -> Tuple[int, int]:
        # (signal_score, rows)
        header = " ".join(tbl[0]).lower() if tbl else ""
        signal = 0
        for kw in ("英文", "中文", "rom", "对照", "编号"):
            if kw in header:
                signal += 10
        # A lot of pages have a big table; rows is secondary.
        return (signal, len(tbl))

    return max(tables, key=score)


def _normalize_table(table: List[List[str]]) -> Tuple[List[str], List[List[str]]]:
    # Ensure we have headers.
    if not table:
        return ([], [])

    headers = table[0]
    rows = table[1:]

    # Some pages have merged text in one cell; keep as-is.
    max_cols = max((len(headers),) + tuple(len(r) for r in rows))
    headers = headers + [f"col_{i+1}" for i in range(len(headers), max_cols)]
    norm_rows: List[List[str]] = []
    for r in rows:
        rr = r + [""] * (max_cols - len(r))
        norm_rows.append(rr)

    # Trim empty tail rows
    norm_rows = [r for r in norm_rows if any(c.strip() for c in r)]
    return (headers, norm_rows)


def scrape_system(
    system: SystemLink,
    outdir: Path,
    *,
    delay: float = 0.2,
    timeout: int = 30,
    retries: int = 2,
    verbose: bool = True,
) -> Path:
    _log(f"[SCRAPE] system={system.key} title={system.title}", verbose=verbose)

    # Prefer JSON endpoint (fast + complete). Fallback to HTML table.
    json_result = _try_fetch_system_json(system, timeout=timeout, retries=retries, verbose=verbose)
    if json_result is not None:
        headers, rows = json_result
    else:
        html = _http_get(system.url, timeout=timeout, retries=retries, verbose=verbose)
        parser = _TableExtractor()
        parser.feed(html)
        _log(f"[PARSE] found {len(parser.tables)} <table> elements", verbose=verbose)

        table = _pick_main_table(parser.tables)
        if table is None:
            raise RuntimeError(f"No table found for {system.key} ({system.url})")

        headers, rows = _normalize_table(table)
        rows = _expand_two_column_paired_rows(headers, rows)
        rows = _expand_single_cell_rows(headers, rows)
        _log(f"[TABLE] columns={len(headers)} rows={len(rows)}", verbose=verbose)

    # For mapping-style pages, only keep the CN/EN mapping column.
    headers, rows = _extract_cn_en_mapping(headers, rows)

    outdir.mkdir(parents=True, exist_ok=True)
    out_path = outdir / f"{system.key}.csv"

    with out_path.open("w", encoding="utf-8", newline="") as f:
        w = csv.writer(f)
        w.writerow(["system", *headers])
        for idx, r in enumerate(rows, 1):
            w.writerow([system.key, *r])
            if verbose and idx % 5000 == 0:
                _log(f"[WRITE] {system.key}.csv wrote {idx}/{len(rows)} rows...", verbose=True)

    # Also emit a lightweight JSON alongside for programmatic use.
    json_path = outdir / f"{system.key}.json"
    with json_path.open("w", encoding="utf-8") as f:
        json.dump(
            {
                "system": system.key,
                "title": system.title,
                "source": system.url,
                "headers": headers,
                "rows": rows,
            },
            f,
            ensure_ascii=False,
        )

    _log(f"[OUT] {out_path}", verbose=verbose)
    _log(f"[OUT] {json_path}", verbose=verbose)

    if delay:
        _log(f"[SLEEP] {delay:.2f}s", verbose=verbose)
        time.sleep(delay)
    return out_path


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--base-url", default=BASE_URL, help="base url for dz index (default: http://emu.jy6d.com/dz/)")
    ap.add_argument("--outdir", default="data/jy6d-dz", help="output directory")
    ap.add_argument("--list", action="store_true", help="list available systems")
    ap.add_argument("--system", action="append", help="system key to scrape (repeatable)")
    ap.add_argument("--all", action="store_true", help="scrape all systems")
    ap.add_argument("--delay", type=float, default=0.2, help="delay between requests")
    ap.add_argument("--timeout", type=int, default=30, help="HTTP timeout seconds")
    ap.add_argument("--retries", type=int, default=2, help="HTTP retries")
    ap.add_argument("--quiet", action="store_true", help="less output")
    args = ap.parse_args()

    verbose = not args.quiet

    outdir = Path(args.outdir)

    base_url = args.base_url.strip()
    if not base_url.endswith("/"):
        base_url += "/"

    try:
        index_html = _http_get(
            base_url,
            timeout=args.timeout,
            retries=args.retries,
            verbose=verbose,
        )
    except urllib.error.URLError as e:
        print(f"Failed to fetch index page: {e}", file=sys.stderr)
        return 2

    systems = _extract_system_links(index_html, base_url)
    if not systems:
        print("Failed to find any system links on index page", file=sys.stderr)
        return 2

    systems_by_key = {s.key: s for s in systems}

    if args.list:
        _log(f"[INFO] found {len(systems)} systems", verbose=verbose)
        for s in sorted(systems, key=lambda x: x.key):
            print(f"{s.key}\t{s.url}\t{s.title}")
        return 0

    wanted: List[SystemLink] = []
    if args.all:
        wanted = sorted(systems, key=lambda x: x.key)
    elif args.system:
        for k in args.system:
            kk = k.strip().lower()
            if kk in systems_by_key:
                wanted.append(systems_by_key[kk])
            else:
                print(f"Unknown system: {kk}", file=sys.stderr)
                print("Use --list to see available systems", file=sys.stderr)
                return 2
    else:
        print("No target specified. Use --system <key> or --all (or --list)", file=sys.stderr)
        return 2

    ok = 0
    _log(f"[INFO] scraping {len(wanted)} system(s) -> {outdir}", verbose=verbose)

    for idx, s in enumerate(wanted, 1):
        try:
            _log(f"\n[{idx}/{len(wanted)}] start {s.key}", verbose=verbose)
            out = scrape_system(
                s,
                outdir,
                delay=args.delay,
                timeout=args.timeout,
                retries=args.retries,
                verbose=verbose,
            )
            print(f"OK  {s.key} -> {out}")
            ok += 1
        except Exception as e:
            print(f"ERR {s.key}: {e}", file=sys.stderr)

    print(f"Done. {ok}/{len(wanted)} succeeded.")
    return 0 if ok == len(wanted) else 1


if __name__ == "__main__":
    raise SystemExit(main())
