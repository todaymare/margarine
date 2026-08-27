#!/usr/bin/env python3
"""Turn llvm-cov export JSON and LCOV branch edges into margarine reports."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any


SourceKey = tuple[str, str]
BranchKey = tuple[str, str, tuple[int, int, int, int]]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("coverage_json", type=Path)
    parser.add_argument("--source-root", type=Path, required=True)
    parser.add_argument("--lcov", type=Path)
    parser.add_argument("--output-json", type=Path, required=True)
    parser.add_argument("--output-text", type=Path, required=True)
    return parser.parse_args()


def external_filename(path: Path, source_root: Path) -> str:
    """Return a stable, non-machine-specific display path for external Rust."""
    try:
        relative = path.relative_to(source_root / "margarine" / "vendor")
    except ValueError:
        relative = None
    if relative is not None:
        return (Path("third_party") / "vendor" / relative).as_posix()

    parts = path.parts
    if "checkouts" in parts:
        checkout_index = parts.index("checkouts")
        package = parts[checkout_index + 1].rsplit("-", 1)[0]
        try:
            source_index = parts.index("src", checkout_index + 1)
            suffix = Path(*parts[source_index:])
            return (Path("third_party") / package / suffix).as_posix()
        except (ValueError, IndexError):
            return (Path("third_party") / package / path.name).as_posix()

    if "registry" in parts:
        try:
            source_index = parts.index("src", parts.index("registry") + 1)
            package = parts[source_index - 1]
            return (Path("third_party") / package / Path(*parts[source_index:])).as_posix()
        except (ValueError, IndexError):
            pass

    return (Path("third_party") / path.name).as_posix()


def source_info(filename: str, source_root: Path) -> tuple[str, str, Path] | None:
    path = Path(filename)
    if not path.is_absolute():
        path = source_root / path
    path = path.resolve()
    if path.suffix != ".rs":
        return None

    root = source_root.resolve()
    try:
        relative = path.relative_to(root)
    except ValueError:
        return "third_party", external_filename(path, root), path
    if relative.parts[:2] == ("margarine", "vendor"):
        return "third_party", external_filename(path, root), path
    return "first_party", relative.as_posix(), path


def source_context(path: Path, line: int) -> list[str]:
    try:
        lines = path.read_text(encoding="utf-8").splitlines()
    except (OSError, UnicodeDecodeError):
        return []
    start = max(1, line - 2)
    end = min(len(lines), line + 2)
    return [f"{number}: {lines[number - 1]}" for number in range(start, end + 1)]


def branch_key(branch: dict[str, Any]) -> tuple[int, int, int, int]:
    return (
        int(branch.get("line", 0)),
        int(branch.get("start_col", 0)),
        int(branch.get("end_line", branch.get("line", 0))),
        int(branch.get("end_col", branch.get("start_col", 0))),
    )


def branch_count(branch: dict[str, Any]) -> int:
    value = branch.get("count", 0)
    try:
        return int(value)
    except (TypeError, ValueError):
        return 0


def add_branch(
    groups: dict[BranchKey, list[dict[str, Any]]],
    source_paths: dict[SourceKey, Path],
    scope: str,
    filename: str,
    path: Path,
    key: tuple[int, int, int, int],
    branch: dict[str, Any],
) -> None:
    groups.setdefault((scope, filename, key), []).append(branch)
    source_paths[(scope, filename)] = path


def read_lcov(
    lcov_path: Path,
    source_root: Path,
) -> tuple[dict[BranchKey, list[dict[str, Any]]], dict[SourceKey, Path]]:
    groups: dict[BranchKey, list[dict[str, Any]]] = {}
    source_paths: dict[SourceKey, Path] = {}
    current: tuple[str, str, Path] | None = None
    for line in lcov_path.read_text(encoding="utf-8").splitlines():
        if line.startswith("SF:"):
            current = source_info(line[3:], source_root)
        elif line.startswith("BRDA:") and current is not None:
            fields = line[5:].split(",", 3)
            if len(fields) != 4:
                continue
            source_line, block, branch_id, taken = fields
            try:
                source_line_number = int(source_line)
            except ValueError:
                continue
            try:
                block_number = int(block)
            except ValueError:
                block_number = 0
            try:
                count = int(taken)
            except ValueError:
                count = 0
            scope, filename, path = current
            add_branch(
                groups,
                source_paths,
                scope,
                filename,
                path,
                (source_line_number, 0, source_line_number, block_number),
                {"count": count, "branch_id": branch_id},
            )
    return groups, source_paths


def read_json(
    payload: dict[str, Any],
    source_root: Path,
) -> tuple[dict[BranchKey, list[dict[str, Any]]], dict[SourceKey, Path]]:
    groups: dict[BranchKey, list[dict[str, Any]]] = {}
    source_paths: dict[SourceKey, Path] = {}
    for datum in payload.get("data", []):
        for file_data in datum.get("files", []):
            current = source_info(str(file_data.get("filename", "")), source_root)
            if current is None:
                continue
            scope, filename, path = current
            for branch in file_data.get("branches", []) or []:
                if not isinstance(branch, dict):
                    continue
                add_branch(groups, source_paths, scope, filename, path, branch_key(branch), branch)
    return groups, source_paths


def summarize(entries: list[dict[str, Any]]) -> dict[str, int]:
    total_edges = sum(len(entry["edges"]) for entry in entries)
    covered_edges = sum(
        edge["count"] > 0
        for entry in entries
        for edge in entry["edges"]
    )
    total_groups = len(entries)
    fully_hit = sum(entry["status"] == "FULLY HIT" for entry in entries)
    partial = sum(entry["status"] == "PARTIALLY HIT" for entry in entries)
    not_hit = sum(entry["status"] == "NOT HIT" for entry in entries)
    return {
        "covered_edges": covered_edges,
        "total_edges": total_edges,
        "fully_hit": fully_hit,
        "partial": partial,
        "not_hit": not_hit,
        "total_groups": total_groups,
        "actionable": total_groups - fully_hit,
    }


def analyze(
    payload: dict[str, Any],
    source_root: Path,
    lcov_path: Path | None = None,
) -> dict[str, Any]:
    # LCOV BRDA records are the portable edge-level representation. LLVM's
    # JSON export uses positional branch arrays on current toolchains, while
    # older versions emitted objects, so use JSON only when LCOV is absent.
    if lcov_path is not None:
        groups, source_paths = read_lcov(lcov_path, source_root)
    else:
        groups, source_paths = read_json(payload, source_root)

    entries: list[dict[str, Any]] = []
    for (scope, filename, key), branches in sorted(groups.items()):
        counts = [branch_count(branch) for branch in branches]
        if not any(counts):
            status = "NOT HIT"
        elif all(count > 0 for count in counts):
            status = "FULLY HIT"
        else:
            status = "PARTIALLY HIT"
        line, start_col, end_line, end_col = key
        entries.append(
            {
                "scope": scope,
                "file": filename,
                "line": line,
                "start_column": start_col,
                "end_line": end_line,
                "end_column": end_col,
                "status": status,
                "edges": [
                    {
                        "index": index,
                        "branch_id": branch.get("branch_id"),
                        "count": count,
                    }
                    for index, (branch, count) in enumerate(zip(branches, counts))
                ],
                "context": source_context(source_paths[(scope, filename)], line),
            }
        )

    scope_entries: dict[str, list[dict[str, Any]]] = {"first_party": [], "third_party": []}
    for entry in entries:
        scope_entries.setdefault(entry["scope"], []).append(entry)
    scopes = {scope: summarize(scope_entries[scope]) for scope in sorted(scope_entries)}
    summary = summarize(entries)
    return {
        "schema_version": 3,
        "source": "llvm-cov LCOV export" if lcov_path is not None else "llvm-cov export",
        "summary": {
            "branches": {"covered": summary["covered_edges"], "total": summary["total_edges"]},
            "branch_groups": {
                "fully_hit": summary["fully_hit"],
                "partial": summary["partial"],
                "not_hit": summary["not_hit"],
                "total": summary["total_groups"],
            },
            "actionable": summary["actionable"],
            "scopes": scopes,
        },
        "branches": entries,
    }


def render_text(report: dict[str, Any]) -> str:
    summary = report["summary"]
    branches = report["branches"]
    total_edges = summary["branches"]["total"]
    covered_edges = summary["branches"]["covered"]
    edge_percent = covered_edges * 100 / total_edges if total_edges else 100.0
    groups = summary["branch_groups"]
    lines = [
        "margarine compiler coverage",
        "",
        f"Branch edges: {covered_edges}/{total_edges} ({edge_percent:.2f}%)",
        f"Fully covered branch groups: {groups['fully_hit']}/{groups['total']}",
        f"Partially covered branch groups: {groups['partial']}",
        f"Uncovered branch groups: {groups['not_hit']}",
        "",
        "Coverage by source scope:",
    ]
    for scope, values in summary["scopes"].items():
        percent = values["covered_edges"] * 100 / values["total_edges"] if values["total_edges"] else 100.0
        lines.append(
            f" {scope}: {values['covered_edges']}/{values['total_edges']} edges ({percent:.2f}%); "
            f"{values['fully_hit']}/{values['total_groups']} groups fully hit"
        )
    lines.extend(["", "Actionable branch targets:"])
    actionable = [entry for entry in branches if entry["status"] != "FULLY HIT"]
    if not actionable:
        lines.append("  none")
    else:
        for entry in actionable:
            lines.extend(
                [
                    "",
                    f"[{entry['scope']}] {entry['file']}:{entry['line']}:{entry['start_column']}",
                    f"  {entry['status']} ({len(entry['edges'])} edges; counts: "
                    + ", ".join(str(edge["count"]) for edge in entry["edges"]),
                ]
            )
            lines.extend(f"    {line}" for line in entry["context"])
    return "\n".join(lines) + "\n"


def main() -> None:
    args = parse_args()
    payload = json.loads(args.coverage_json.read_text(encoding="utf-8"))
    report = analyze(payload, args.source_root, args.lcov)
    args.output_json.parent.mkdir(parents=True, exist_ok=True)
    args.output_json.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    args.output_text.write_text(render_text(report), encoding="utf-8")


if __name__ == "__main__":
    main()
