#!/usr/bin/env python3
"""Build a self-contained interactive dashboard from Margarine coverage reports."""

from __future__ import annotations

import argparse
import html
import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--coverage-dir", type=Path, required=True)
    parser.add_argument("--output-json", type=Path, required=True)
    parser.add_argument("--output-html", type=Path, required=True)
    return parser.parse_args()


def load_json(path: Path) -> dict[str, Any] | None:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return None


def history_entries(history_dir: Path) -> list[dict[str, Any]]:
    entries = []
    for path in sorted(history_dir.glob("*.json")):
        report = load_json(path)
        if report is None or "summary" not in report:
            continue
        entries.append(
            {
                "timestamp": path.stem,
                "summary": report["summary"],
            }
        )
    return entries


def file_summary(report: dict[str, Any]) -> list[dict[str, Any]]:
    files: dict[tuple[str, str], dict[str, int | str]] = {}
    for entry in report.get("branches", []):
        key = (entry.get("scope", "first_party"), entry["file"])
        stats = files.setdefault(
            key,
            {
                "scope": key[0],
                "file": key[1],
                "covered_edges": 0,
                "total_edges": 0,
                "fully_hit": 0,
                "partial": 0,
                "not_hit": 0,
            },
        )
        counts = [edge.get("count", 0) for edge in entry.get("edges", [])]
        stats["covered_edges"] += sum(count > 0 for count in counts)
        stats["total_edges"] += len(counts)
        status_key = {
            "FULLY HIT": "fully_hit",
            "PARTIALLY HIT": "partial",
            "NOT HIT": "not_hit",
        }[entry["status"]]
        stats[status_key] += 1
    return sorted(files.values(), key=lambda item: (str(item["scope"]), str(item["file"])))


def dashboard_data(coverage_dir: Path) -> dict[str, Any]:
    current = load_json(coverage_dir / "uncovered.json")
    if current is None:
        raise SystemExit(f"coverage report not found: {coverage_dir / 'uncovered.json'}")
    return {
        "schema_version": 1,
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "current": current,
        "files": file_summary(current),
        "history": history_entries(coverage_dir / "history"),
    }


HTML_TEMPLATE = r"""<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Margarine coverage dashboard</title>
<style>
:root { color-scheme: dark; --bg:#111827; --panel:#1f2937; --muted:#9ca3af; --text:#f3f4f6; --accent:#60a5fa; --good:#34d399; --warn:#fbbf24; --bad:#f87171; }
* { box-sizing:border-box; }
body { margin:0; padding:2rem; background:var(--bg); color:var(--text); font:14px/1.45 ui-monospace,SFMono-Regular,Menlo,monospace; }
main { max-width:1500px; margin:auto; }
h1 { margin:0 0 .3rem; font-size:1.6rem; } h2 { font-size:1rem; margin:0 0 1rem; }
.subtitle,.muted { color:var(--muted); }
.grid { display:grid; grid-template-columns:repeat(auto-fit,minmax(170px,1fr)); gap:.8rem; margin:1.2rem 0; }
.card,section { background:var(--panel); border:1px solid #374151; border-radius:8px; padding:1rem; }
.card strong { display:block; font-size:1.4rem; margin-top:.25rem; }
.controls { display:flex; flex-wrap:wrap; gap:.6rem; align-items:center; margin-bottom:1rem; }
select,input,button { background:#111827; color:var(--text); border:1px solid #4b5563; border-radius:5px; padding:.45rem .6rem; font:inherit; }
button { cursor:pointer; } button:hover { border-color:var(--accent); }
.layout { display:grid; grid-template-columns:minmax(0,2fr) minmax(280px,1fr); gap:1rem; }
@media(max-width:900px) { body { padding:1rem; } .layout { grid-template-columns:1fr; } }
table { width:100%; border-collapse:collapse; } th,td { text-align:left; vertical-align:top; border-bottom:1px solid #374151; padding:.55rem .4rem; } th { color:var(--muted); position:sticky; top:0; background:var(--panel); }
.status { font-weight:bold; } .FULLY-HIT { color:var(--good); } .PARTIALLY-HIT { color:var(--warn); } .NOT-HIT { color:var(--bad); }
code { color:#bfdbfe; } pre { white-space:pre-wrap; color:#d1d5db; margin:.5rem 0 0; }
.scroll { max-height:650px; overflow:auto; } .small { font-size:.82rem; }
svg { width:100%; height:180px; overflow:visible; } .chart-line { fill:none; stroke:var(--accent); stroke-width:2; } .chart-dot { fill:var(--accent); }
.empty { color:var(--muted); padding:1rem 0; }
</style>
</head>
<body>
<main>
<h1>Margarine coverage dashboard</h1>
<div class="subtitle">Generated <span id="generated"></span>. Branch edges are the coverage denominator; groups identify source-level conditionals.</div>
<div class="grid" id="cards"></div>
<section>
<h2>Coverage trend</h2>
<div id="trend"></div>
</section>
<div class="controls">
<label>Scope <select id="scope"></select></label>
<label>Status <select id="status"><option value="actionable">Actionable</option><option value="NOT HIT">Not hit</option><option value="PARTIALLY HIT">Partially hit</option><option value="FULLY HIT">Fully hit</option><option value="all">All</option></select></label>
<label>Search <input id="query" type="search" placeholder="file or source text"></label>
<button id="copy-all">Copy dashboard continuation prompt</button>
</div>
<div class="layout">
<section><h2>Branch targets</h2><div class="scroll"><table><thead><tr><th>Scope / location</th><th>Status</th><th>Edges</th><th>Source context</th><th></th></tr></thead><tbody id="targets"></tbody></table></div></section>
<section><h2>Files</h2><div class="scroll"><table><thead><tr><th>Scope / file</th><th>Edges</th><th>Groups</th></tr></thead><tbody id="files"></tbody></table></div></section>
</div>
</main>
<script>
const DATA = __DATA__;
const state = { scope: 'all', status: 'actionable', query: '' };
const esc = value => String(value ?? '').replace(/[&<>"']/g, c => ({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
const entries = () => DATA.current.branches || [];
const visible = () => entries().filter(entry => {
  if (state.scope !== 'all' && entry.scope !== state.scope) return false;
  if (state.status === 'actionable' && entry.status === 'FULLY HIT') return false;
  if (state.status !== 'all' && state.status !== 'actionable' && entry.status !== state.status) return false;
  if (state.query) {
    const haystack = `${entry.scope} ${entry.file} ${entry.context.join(' ')}`.toLowerCase();
    if (!haystack.includes(state.query.toLowerCase())) return false;
  }
  return true;
});
const scopeSummary = scope => {
  const selected = scope === 'all' ? entries() : entries().filter(entry => entry.scope === scope);
  let total=0, covered=0, full=0, partial=0, none=0;
  selected.forEach(entry => { const counts=entry.edges.map(edge=>edge.count); total+=counts.length; covered+=counts.filter(count=>count>0).length; if(entry.status==='FULLY HIT')full++; else if(entry.status==='PARTIALLY HIT')partial++; else none++; });
  return { total, covered, full, partial, none, groups:selected.length, percent: total ? covered*100/total : 100 };
};
function renderCards() {
  const s = scopeSummary(state.scope);
  document.getElementById('cards').innerHTML = [
    ['Edge coverage', `${s.covered}/${s.total}`, `${s.percent.toFixed(2)}%`],
    ['Fully covered groups', `${s.full}/${s.groups}`, `${s.groups ? (s.full*100/s.groups).toFixed(1) : '100.0'}%`],
    ['Partially covered', s.partial, 'groups'],
    ['Not hit', s.none, 'groups'],
    ['Visible targets', visible().length, 'after filters'],
  ].map(card => `<div class="card"><span class="muted">${card[0]}</span><strong>${card[1]}</strong><span class="muted">${card[2]}</span></div>`).join('');
}
function renderTrend() {
  const history = DATA.history || [];
  if (!history.length) { document.getElementById('trend').innerHTML='<div class="empty">No previous runs yet.</div>'; return; }
  const width=900, height=150, pad=24, max=Math.max(100, ...history.map(item => { const b=item.summary.branches; return b.total ? b.covered*100/b.total : 100; }));
  const points=history.map((item,index) => { const b=item.summary.branches; const value=b.total ? b.covered*100/b.total : 100; const x=pad+(width-2*pad)*(history.length===1?.5:index/(history.length-1)); const y=height-pad-(height-2*pad)*value/max; return {x,y,value,item}; });
  document.getElementById('trend').innerHTML=`<svg viewBox="0 0 ${width} ${height}" role="img" aria-label="Coverage trend"><polyline class="chart-line" points="${points.map(point=>`${point.x},${point.y}`).join(' ')}"></polyline>${points.map(point=>`<circle class="chart-dot" cx="${point.x}" cy="${point.y}" r="3"><title>${esc(point.item.timestamp)}: ${point.value.toFixed(2)}%</title></circle>`).join('')}</svg><div class="muted small">${esc(history[0].timestamp)} → ${esc(history[history.length-1].timestamp)}; scope filter affects cards and targets, not this aggregate trend.</div>`;
}
function renderTargets() {
  const list=visible();
  document.getElementById('targets').innerHTML=list.length ? list.map((entry,index) => `<tr><td><code>[${esc(entry.scope)}] ${esc(entry.file)}:${entry.line}:${entry.start_column}</code></td><td class="status ${entry.status.replaceAll(' ','-')}">${entry.status}</td><td>${entry.edges.map(edge=>esc(edge.count)).join(', ')}</td><td><details><summary>${entry.context.length ? esc(entry.context[0]) : 'source unavailable'}</summary><pre>${esc(entry.context.join('\n'))}</pre></details></td><td><button onclick="copyTarget(${entries().indexOf(entry)})">Copy</button></td></tr>`).join('') : '<tr><td colspan="5" class="empty">No matching targets.</td></tr>';
}
function renderFiles() {
  const list=DATA.files.filter(item => state.scope==='all' || item.scope===state.scope).filter(item => !state.query || `${item.scope} ${item.file}`.toLowerCase().includes(state.query.toLowerCase()));
  document.getElementById('files').innerHTML=list.length ? list.map(item => `<tr><td><code>[${esc(item.scope)}] ${esc(item.file)}</code></td><td>${item.covered_edges}/${item.total_edges}</td><td>${item.fully_hit} full, ${item.partial} partial, ${item.not_hit} none</td></tr>`).join('') : '<tr><td colspan="3" class="empty">No matching files.</td></tr>';
}
function render() { renderCards(); renderTargets(); renderFiles(); }
function copyText(text) { navigator.clipboard?.writeText(text).catch(() => window.prompt('Copy this text:', text)); }
window.copyTarget = index => { const entry=entries()[index]; copyText(`Continue Margarine compiler coverage work. Investigate [${entry.scope}] ${entry.file}:${entry.line}. Status: ${entry.status}. Edge counts: ${entry.edges.map(edge=>edge.count).join(', ')}. Source context:\n${entry.context.join('\n')}\nAdd only a meaningful Margarine regression test; do not modify compiler behavior for coverage.`); };
document.getElementById('copy-all').onclick = () => copyText(`Continue Margarine compiler coverage work. Read artifacts/coverage/dashboard.json and target the highest-value actionable branch. Add a meaningful Margarine regression test, run cargo run -p margarine -- test tests/core.mar, rerun ./scripts/coverage.sh, and compare the dashboard. Do not modify compiler/runtime implementation solely for coverage.`);
const scopes=['all', ...new Set(entries().map(entry=>entry.scope))]; document.getElementById('scope').innerHTML=scopes.map(scope=>`<option value="${esc(scope)}">${esc(scope)}</option>`).join('');
document.getElementById('scope').onchange=e=>{state.scope=e.target.value;render();}; document.getElementById('status').onchange=e=>{state.status=e.target.value;render();}; document.getElementById('query').oninput=e=>{state.query=e.target.value;render();};
document.getElementById('generated').textContent=DATA.generated_at; renderTrend(); render();
</script>
</body>
</html>
"""


def render_html(data: dict[str, Any]) -> str:
    serialized = json.dumps(data, separators=(",", ":")).replace("<", "\\u003c")
    return HTML_TEMPLATE.replace("__DATA__", serialized)


def main() -> None:
    args = parse_args()
    data = dashboard_data(args.coverage_dir)
    args.output_json.parent.mkdir(parents=True, exist_ok=True)
    args.output_json.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")
    args.output_html.parent.mkdir(parents=True, exist_ok=True)
    args.output_html.write_text(render_html(data), encoding="utf-8")


if __name__ == "__main__":
    main()
