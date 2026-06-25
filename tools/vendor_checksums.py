#!/usr/bin/env python3
"""Check or rebuild Cargo vendor checksums for the filtered contest clone."""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
import sys
from collections import defaultdict
from pathlib import Path, PurePosixPath


REPO_ROOT = Path(__file__).resolve().parents[1]
CHECKSUM_NAME = "cargo-checksum.json"
GENERATED_CHECKSUM_NAME = ".cargo-checksum.json"


def is_eligible(relative: PurePosixPath) -> bool:
    if relative.name in {CHECKSUM_NAME, GENERATED_CHECKSUM_NAME}:
        return False
    return not any(part.startswith(".") for part in relative.parts)


def tracked_vendor_files(
    vendor_rel: PurePosixPath,
) -> dict[str, set[PurePosixPath]] | None:
    try:
        result = subprocess.run(
            ["git", "ls-files", "-z", "--", vendor_rel.as_posix()],
            cwd=REPO_ROOT,
            check=True,
            capture_output=True,
        )
    except (FileNotFoundError, subprocess.CalledProcessError):
        return None

    files: dict[str, set[PurePosixPath]] = defaultdict(set)
    prefix_parts = len(vendor_rel.parts)
    for raw_path in result.stdout.split(b"\0"):
        if not raw_path:
            continue
        path = PurePosixPath(raw_path.decode("utf-8"))
        parts = path.parts[prefix_parts:]
        if len(parts) < 2:
            continue
        crate, relative = parts[0], PurePosixPath(*parts[1:])
        if is_eligible(relative):
            files[crate].add(relative)
    return files


def filesystem_vendor_files(
    vendor_root: Path,
) -> dict[str, set[PurePosixPath]]:
    files: dict[str, set[PurePosixPath]] = defaultdict(set)
    for crate_dir in sorted(path for path in vendor_root.iterdir() if path.is_dir()):
        for path in crate_dir.rglob("*"):
            if not path.is_file():
                continue
            relative = PurePosixPath(path.relative_to(crate_dir).as_posix())
            if is_eligible(relative):
                files[crate_dir.name].add(relative)
    return files


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as file:
        for chunk in iter(lambda: file.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def tracked_blob_hashes(
    vendor_rel: PurePosixPath,
    files_by_crate: dict[str, set[PurePosixPath]],
) -> dict[tuple[str, PurePosixPath], str]:
    entries = [
        (crate, relative)
        for crate, relative_paths in sorted(files_by_crate.items())
        for relative in sorted(relative_paths, key=lambda item: item.as_posix())
    ]
    process = subprocess.Popen(
        ["git", "cat-file", "--batch"],
        cwd=REPO_ROOT,
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
    )
    assert process.stdin is not None
    assert process.stdout is not None

    hashes: dict[tuple[str, PurePosixPath], str] = {}
    try:
        for crate, relative in entries:
            path = vendor_rel / crate / relative
            process.stdin.write(f":{path.as_posix()}\n".encode())
            process.stdin.flush()

            header = process.stdout.readline().decode().strip().split()
            if len(header) != 3 or header[1] != "blob":
                raise ValueError(f"{path.as_posix()}: unavailable Git blob")
            size = int(header[2])
            content = process.stdout.read(size)
            if len(content) != size or process.stdout.read(1) != b"\n":
                raise ValueError(f"{path.as_posix()}: truncated Git blob")
            hashes[(crate, relative)] = hashlib.sha256(content).hexdigest()
    finally:
        process.stdin.close()
        if process.wait() != 0:
            raise ValueError("git cat-file --batch failed")

    return hashes


def source_hash(
    crate_dir: Path,
    crate: str,
    relative: PurePosixPath,
    blob_hashes: dict[tuple[str, PurePosixPath], str] | None,
) -> str:
    if blob_hashes is not None:
        return blob_hashes[(crate, relative)]
    return sha256(crate_dir / Path(relative.as_posix()))


def load_manifest(crate_dir: Path) -> dict:
    manifest_path = crate_dir / CHECKSUM_NAME
    try:
        return json.loads(manifest_path.read_text(encoding="utf-8"))
    except FileNotFoundError as error:
        raise ValueError(f"{crate_dir.name}: missing {CHECKSUM_NAME}") from error
    except json.JSONDecodeError as error:
        raise ValueError(f"{crate_dir.name}: invalid {CHECKSUM_NAME}: {error}") from error


def rebuild(
    vendor_root: Path,
    vendor_rel: PurePosixPath,
    files_by_crate: dict[str, set[PurePosixPath]],
    blob_hashes: dict[tuple[str, PurePosixPath], str] | None,
) -> int:
    if tracked_vendor_files(vendor_rel) is None:
        print("error: --fix must run inside the Git working tree", file=sys.stderr)
        return 1

    changed = 0
    for crate, relative_paths in sorted(files_by_crate.items()):
        crate_dir = vendor_root / crate
        manifest = load_manifest(crate_dir)
        rebuilt = {
            "files": {
                path.as_posix(): source_hash(
                    crate_dir, crate, path, blob_hashes
                )
                for path in sorted(relative_paths, key=lambda item: item.as_posix())
            },
            "package": manifest.get("package"),
        }
        manifest_path = crate_dir / CHECKSUM_NAME
        content = json.dumps(rebuilt, separators=(",", ":"), ensure_ascii=True) + "\n"
        if manifest_path.read_text(encoding="utf-8") != content:
            manifest_path.write_text(content, encoding="utf-8", newline="\n")
            changed += 1

    print(f"rebuilt {len(files_by_crate)} vendor manifests; changed {changed}")
    return 0


def check(
    vendor_root: Path,
    files_by_crate: dict[str, set[PurePosixPath]],
    blob_hashes: dict[tuple[str, PurePosixPath], str] | None,
) -> int:
    issues: list[str] = []
    crate_dirs = sorted(path for path in vendor_root.iterdir() if path.is_dir())
    expected_crates = {path.name for path in crate_dirs}

    for missing_crate in sorted(expected_crates - files_by_crate.keys()):
        issues.append(f"{missing_crate}: no eligible source files")

    for crate_dir in crate_dirs:
        try:
            manifest = load_manifest(crate_dir)
        except ValueError as error:
            issues.append(str(error))
            continue

        listed = {
            PurePosixPath(path)
            for path in manifest.get("files", {})
            if is_eligible(PurePosixPath(path))
        }
        expected = files_by_crate.get(crate_dir.name, set())

        for path in sorted(expected - listed, key=lambda item: item.as_posix()):
            issues.append(f"{crate_dir.name}: unlisted file {path.as_posix()}")
        for path in sorted(listed - expected, key=lambda item: item.as_posix()):
            issues.append(f"{crate_dir.name}: missing file {path.as_posix()}")
        for path in sorted(expected & listed, key=lambda item: item.as_posix()):
            actual = source_hash(
                crate_dir, crate_dir.name, path, blob_hashes
            )
            wanted = manifest["files"][path.as_posix()]
            if actual != wanted:
                issues.append(f"{crate_dir.name}: checksum mismatch {path.as_posix()}")

    for issue in issues:
        print(issue, file=sys.stderr)
    print(f"checked {len(crate_dirs)} vendor manifests; issues {len(issues)}")
    return 1 if issues else 0


def main() -> int:
    parser = argparse.ArgumentParser()
    action = parser.add_mutually_exclusive_group(required=True)
    action.add_argument("--check", action="store_true", help="validate vendor manifests")
    action.add_argument("--fix", action="store_true", help="rebuild vendor manifests")
    parser.add_argument(
        "--vendor",
        default="SWTC/vendor",
        help="vendor directory relative to the repository root",
    )
    parser.add_argument(
        "--source",
        choices=("filesystem", "index"),
        default="filesystem",
        help="hash working-tree files or normalized blobs from the Git index",
    )
    args = parser.parse_args()

    vendor_rel = PurePosixPath(args.vendor)
    vendor_root = REPO_ROOT / Path(vendor_rel.as_posix())
    if not vendor_root.is_dir():
        print(f"error: vendor directory not found: {vendor_root}", file=sys.stderr)
        return 1

    tracked = tracked_vendor_files(vendor_rel)
    files_by_crate = (
        tracked if tracked is not None else filesystem_vendor_files(vendor_root)
    )
    if args.source == "index" and tracked is None:
        print("error: --source index requires a Git working tree", file=sys.stderr)
        return 1
    blob_hashes = (
        tracked_blob_hashes(vendor_rel, files_by_crate)
        if args.source == "index"
        else None
    )
    return (
        rebuild(vendor_root, vendor_rel, files_by_crate, blob_hashes)
        if args.fix
        else check(vendor_root, files_by_crate, blob_hashes)
    )


if __name__ == "__main__":
    raise SystemExit(main())
