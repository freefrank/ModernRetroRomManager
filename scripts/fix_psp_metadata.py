#!/usr/bin/env python3
"""
Fix PSP metadata.pegasus.txt file paths.

Problem: file: fields only contain filename, missing subfolder path.
Solution: Scan subfolders, build filename->path mapping, update metadata.

Usage:
    python fix_psp_metadata.py D:\samba\psp
    python fix_psp_metadata.py /path/to/psp/folder
"""

import os
import sys
from pathlib import Path
from typing import Dict, Optional


def scan_subfolders(base_dir: Path) -> Dict[str, str]:
    """
    Scan immediate subfolders for ROM files.
    Returns: {filename: subfolder/filename}
    """
    mapping: Dict[str, str] = {}
    rom_extensions = {".iso", ".cso", ".pbp", ".chd"}

    print(f"Scanning subfolders in: {base_dir}")

    subfolders = [d for d in base_dir.iterdir() if d.is_dir()]
    print(f"Found {len(subfolders)} subfolders")

    for i, subfolder in enumerate(subfolders, 1):
        subfolder_name = subfolder.name
        try:
            for file in subfolder.iterdir():
                if file.is_file() and file.suffix.lower() in rom_extensions:
                    filename = file.name
                    relative_path = f"{subfolder_name}/{filename}"
                    mapping[filename] = relative_path

            if i % 50 == 0:
                print(
                    f"  Processed {i}/{len(subfolders)} subfolders ({len(mapping)} ROMs found)"
                )
        except PermissionError as e:
            print(f"  Warning: Cannot access {subfolder_name}: {e}")
        except Exception as e:
            print(f"  Error processing {subfolder_name}: {e}")

    print(f"Total ROMs indexed: {len(mapping)}")
    return mapping


def fix_metadata(metadata_path: Path, mapping: Dict[str, str]) -> tuple[str, int, int]:
    """
    Fix file: lines in metadata using the mapping.
    Returns: (fixed_content, total_file_lines, fixed_count)
    """
    with open(metadata_path, "r", encoding="utf-8") as f:
        lines = f.readlines()

    fixed_lines = []
    total_file_lines = 0
    fixed_count = 0
    not_found = []

    for line in lines:
        if line.startswith("file:"):
            total_file_lines += 1
            # Extract current filename
            current_value = line[5:].strip()

            # Check if already has a path (contains /)
            if "/" in current_value or "\\" in current_value:
                fixed_lines.append(line)
                continue

            # Look up in mapping
            if current_value in mapping:
                new_path = mapping[current_value]
                fixed_lines.append(f"file: {new_path}\n")
                fixed_count += 1
            else:
                # Not found, keep original
                fixed_lines.append(line)
                not_found.append(current_value)
        else:
            fixed_lines.append(line)

    if not_found:
        print(f"\nWarning: {len(not_found)} files not found in subfolders:")
        for f in not_found[:10]:
            print(f"  - {f}")
        if len(not_found) > 10:
            print(f"  ... and {len(not_found) - 10} more")

    return "".join(fixed_lines), total_file_lines, fixed_count


def main():
    if len(sys.argv) < 2:
        print("Usage: python fix_psp_metadata.py <directory>")
        print("Example: python fix_psp_metadata.py D:\\samba\\psp")
        sys.exit(1)

    base_dir = Path(sys.argv[1])

    if not base_dir.exists():
        print(f"Error: Directory not found: {base_dir}")
        sys.exit(1)

    metadata_path = base_dir / "metadata.pegasus.txt"
    if not metadata_path.exists():
        print(f"Error: metadata.pegasus.txt not found in {base_dir}")
        sys.exit(1)

    print("=" * 60)
    print("PSP Metadata Path Fixer")
    print("=" * 60)

    # Step 1: Scan subfolders
    print("\n[Step 1] Scanning subfolders for ROM files...")
    mapping = scan_subfolders(base_dir)

    if not mapping:
        print("Error: No ROM files found in subfolders!")
        sys.exit(1)

    # Step 2: Fix metadata
    print("\n[Step 2] Fixing metadata file paths...")
    fixed_content, total, fixed = fix_metadata(metadata_path, mapping)

    # Step 3: Write output
    output_path = base_dir / "metadata.pegasus.fixed.txt"
    with open(output_path, "w", encoding="utf-8") as f:
        f.write(fixed_content)

    print("\n" + "=" * 60)
    print("DONE!")
    print("=" * 60)
    print(f"Total 'file:' lines: {total}")
    print(f"Fixed: {fixed}")
    print(f"Already correct or not found: {total - fixed}")
    print(f"\nOutput saved to: {output_path}")
    print("\nReview the output file, then rename it to metadata.pegasus.txt")


if __name__ == "__main__":
    main()
