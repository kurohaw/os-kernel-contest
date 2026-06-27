#!/usr/bin/env python3
"""Summarize per-case LTP results from an oscomp serial log."""

from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import asdict, dataclass
from pathlib import Path


RUN_RE = re.compile(r"^RUN LTP CASE (?P<name>\S+)\s*$")
STATUS_RE = re.compile(r"^FAIL LTP CASE (?P<name>\S+)\s*:\s*(?P<status>-?\d+)\s*$")
ANSI_RE = re.compile(r"\x1b\[[0-?]*[ -/]*[@-~]")
RESULT_KINDS = ("TPASS", "TFAIL", "TBROK", "TCONF", "TWARN")


@dataclass
class CaseResult:
    name: str
    tpass: int = 0
    tfail: int = 0
    tbrok: int = 0
    tconf: int = 0
    twarn: int = 0
    status: int | None = None
    timed_out: bool = False

    @property
    def safe(self) -> bool:
        return (
            self.tpass > 0
            and self.tfail == 0
            and self.tbrok == 0
            and not self.timed_out
            and self.status == 0
        )


def parse_log(text: str) -> tuple[list[CaseResult], bool]:
    results: list[CaseResult] = []
    current: CaseResult | None = None
    panicked = False

    for raw_line in text.splitlines():
        line = ANSI_RE.sub("", raw_line.strip().lstrip("\ufeff"))
        if "Panicked" in line or "panicked at" in line:
            panicked = True

        run_match = RUN_RE.match(line)
        if run_match:
            if current is not None:
                results.append(current)
            current = CaseResult(run_match.group("name"))
            continue

        if current is None:
            continue

        for kind in RESULT_KINDS:
            if re.search(rf"\b{kind}\b", line):
                field = kind.lower()
                setattr(current, field, getattr(current, field) + 1)

        if "timeout after" in line or " : 124" in line:
            current.timed_out = True

        status_match = STATUS_RE.match(line)
        if status_match and status_match.group("name") == current.name:
            current.status = int(status_match.group("status"))

    if current is not None:
        results.append(current)
    return results, panicked


def print_table(results: list[CaseResult]) -> None:
    print("case\tTPASS\tTFAIL\tTBROK\tTCONF\tstatus\ttimeout\tsafe")
    for result in results:
        status = "missing" if result.status is None else str(result.status)
        print(
            f"{result.name}\t{result.tpass}\t{result.tfail}\t{result.tbrok}\t"
            f"{result.tconf}\t{status}\t{int(result.timed_out)}\t{int(result.safe)}"
        )


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("log", type=Path, help="QEMU serial log to analyze")
    parser.add_argument(
        "--format",
        choices=("table", "json", "safe"),
        default="table",
        help="output format (default: table)",
    )
    args = parser.parse_args()

    try:
        text = args.log.read_text(encoding="utf-8", errors="replace")
    except OSError as error:
        parser.error(str(error))

    results, panicked = parse_log(text)
    if args.format == "json":
        print(
            json.dumps(
                {
                    "panicked": panicked,
                    "cases": [asdict(result) | {"safe": result.safe} for result in results],
                },
                indent=2,
            )
        )
    elif args.format == "safe":
        for result in results:
            if result.safe:
                print(result.name)
    else:
        print_table(results)
        print(f"summary\tcases={len(results)}\tsafe={sum(r.safe for r in results)}\tpanicked={int(panicked)}")

    if not results:
        print("error: no LTP cases found", file=sys.stderr)
        return 2
    return 1 if panicked else 0


if __name__ == "__main__":
    raise SystemExit(main())
