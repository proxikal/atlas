# Performance Dashboard - Future Enhancement

**Status:** Planned (not yet implemented)
**Priority:** Nice-to-have
**Estimated Effort:** 4-8 hours

---

## Overview

A web-based dashboard to visualize Atlas runtime performance trends over time. Shows benchmark history, regression detection, and comparative analysis across versions.

---

## Why This Matters

1. **Visibility:** See performance trends at a glance, not buried in CI logs
2. **Regression Detection:** Visual anomalies are easier to spot than reading numbers
3. **Historical Context:** "Is this slower than v0.1?" answered instantly
4. **Communication:** Shareable performance story for users/contributors

---

## Implementation Options

### Option 1: GitHub Pages (Simplest, Recommended)

The `github-action-benchmark` action (already configured in bench.yml) can auto-generate a dashboard.

**What it does:**
- Stores benchmark data in `gh-pages` branch
- Auto-generates an HTML dashboard
- Accessible at `https://proxikal.github.io/atlas/dev/bench/`

**To enable:**
1. Go to Repository → Settings → Pages
2. Set Source: "Deploy from a branch"
3. Select branch: `gh-pages`, folder: `/ (root)`
4. Dashboard appears automatically after next benchmark run

**Pros:** Zero maintenance, already integrated
**Cons:** Basic visualization, limited customization

---

### Option 2: Dedicated Dashboard Service

Use a service like [Bencher.dev](https://bencher.dev/) or [Datadog](https://www.datadoghq.com/).

**Bencher.dev (open source friendly):**
```yaml
- name: Track benchmarks
  uses: bencherdev/bencher@main
  with:
    project: atlas
    token: ${{ secrets.BENCHER_API_TOKEN }}
    adapter: rust_criterion
```

**Pros:** Rich visualizations, alerts, API
**Cons:** External dependency, may have costs at scale

---

### Option 3: Custom Dashboard (Most Effort)

Build a custom Next.js/Astro site that fetches benchmark data from GitHub.

**Architecture:**
```
GitHub Actions → Store JSON artifacts
     ↓
Cloudflare Worker → Aggregate data
     ↓
Static Site → Display charts (Chart.js/D3)
```

**Pros:** Full control, custom metrics, branding
**Cons:** Maintenance burden, 8+ hours to build

---

## Recommended Approach

**Phase 1 (Now):** Enable GitHub Pages for basic dashboard (5 minutes)

**Phase 2 (When needed):** Migrate to Bencher.dev if more features required

**Phase 3 (If ever):** Custom dashboard only if Atlas becomes a major project with specific needs

---

## Metrics to Track

| Metric | Description |
|--------|-------------|
| `vm_arithmetic_*` | Core VM operation speed |
| `interp_*` | Interpreter performance |
| `typecheck_*` | Type checker throughput |
| `parse_*` | Parser speed |
| `compile_*` | Compilation time |

---

## Dashboard Mockup

```
┌─────────────────────────────────────────────────────────────┐
│  Atlas Performance Dashboard                    v0.2.3      │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  VM Arithmetic (1000 ops)                                   │
│  ┌─────────────────────────────────────────────────────┐   │
│  │     ●───●───●───●───●                               │   │
│  │                       ╲                             │   │
│  │                         ●───●───●  ← v0.2.1 +15%   │   │
│  │                                  ╲                  │   │
│  │                                    ●───●  current   │   │
│  └─────────────────────────────────────────────────────┘   │
│  300µs                                              200µs   │
│                                                             │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐          │
│  │ Parser      │ │ Typechecker │ │ Interpreter │          │
│  │ 45µs/1KB    │ │ 112µs/fn    │ │ 310µs/10K   │          │
│  │ ↓2% better  │ │ →0% same    │ │ ↑5% slower  │          │
│  └─────────────┘ └─────────────┘ └─────────────┘          │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

---

## Quick Enable (GitHub Pages Dashboard)

After the benchmark workflow runs at least once:

```bash
# The gh-pages branch is auto-created by github-action-benchmark
# Just enable GitHub Pages in settings:

# Repository → Settings → Pages
# Source: Deploy from branch
# Branch: gh-pages / (root)
# Save

# Dashboard will be at:
# https://proxikal.github.io/atlas/dev/bench/
```

---

## Related Files

- `.github/workflows/bench.yml` - Benchmark workflow (stores data)
- `crates/atlas-runtime/benches/` - Criterion benchmark definitions

---

## Notes

- Dashboard is auto-generated, no custom code needed
- Data persists in `gh-pages` branch
- Each benchmark run adds a data point
- Alerts configured at 110% threshold (10% regression)
