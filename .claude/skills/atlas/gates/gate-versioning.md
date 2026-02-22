# GATE V: Versioning

**Purpose:** Decide autonomously whether to advance patch or minor version.
No user input required. The rules are the intelligence.

---

## When This Gate Runs

| Event | Action |
|-------|--------|
| Any `fix/` PR merged to main | Patch check → tag if warranted |
| Block AC check phase committed | Minor check → advance if all exit criteria verified |

---

## Patch Version (`vX.Y.Z → vX.Y.Z+1`)

**Trigger:** A `fix/` PR merged to main.

**Rule:** Automatic. No checklist. Tag immediately.

```bash
# 1. Read current version from STATUS.md
# 2. Increment patch: v0.3.1 → v0.3.2
git tag vX.Y.Z+1
git push origin vX.Y.Z+1
# 3. Update STATUS.md version field
```

**Only for `fix/` PRs.** `ci/`, `docs/`, `chore/` PRs do not get a patch tag.

**If unsure whether a fix warrants a tag:** It does. A bug existed in a tagged version —
the fix deserves a tag. Err on the side of tagging.

---

## Minor Version (`vX.Y.0 → vX.Y+1.0`)

**Trigger:** The final AC check phase of a block is committed — but ONLY run this check
if the current version plan's exit criteria could plausibly all be met. Do not run
speculatively on every block.

### Step 1: Find the current version plan

```bash
ls docs/internal/V*_PLAN.md | sort | tail -1
```

### Step 2: Read the Exit Criteria section

Every version plan has an `## Exit Criteria` or `## v0.X Exit Criteria` section.
Read it. Every item is a gate.

### Step 3: Verify each criterion

For each criterion — **verify, never assume:**

| Criterion type | How to verify |
|----------------|---------------|
| Block N complete | STATUS.md shows ✅ AND final AC phase committed |
| Test count ≥ X | `cargo nextest run --workspace 2>&1 \| grep "tests run"` |
| No Arc<Mutex<...>> | `grep -r "Arc<Mutex<" crates/ --include="*.rs" \| grep -v test \| grep -v "//"` |
| Spec updated | Grep for expected sections in `docs/specification/` |
| Example program exists | `ls examples/` — real Atlas program demonstrating the version's capabilities |
| Clippy/fmt clean | CI green on latest main commit |
| Feature works end-to-end | Example program compiles and runs correctly |

### Step 4: Decision

- **ALL criteria ✅** → Advance version, tag, update STATUS.md and ROADMAP.md
- **ANY criteria ❌** → Do NOT advance. Log what's missing in STATUS.md under "Version Gate Blockers". Continue building.

### On version advance

```bash
git tag vX.Y+1.0
git push origin vX.Y+1.0
# Update STATUS.md: version field
# Update ROADMAP.md: mark vX.Y complete, note vX.Y+1 target
# Commit these updates on main (via PR)
```

---

## Rules

- **Never ask the user.** The checklist is the decision.
- **Never advance on block count alone.** Exit criteria must ALL be verified against
  actual codebase state — not inferred from STATUS.md alone.
- **Patch ≠ minor.** If a `fix/` PR corrects something that was architecturally wrong
  (not just a bug), flag it in STATUS.md. The user decides if it warrants rethinking scope.
- **Tags are permanent.** Never re-tag. Verify before tagging.
- **No version exists without a tag.** STATUS.md saying "v0.3" means nothing without
  `git tag v0.3.0` existing. Check: `git tag | grep "^v"`.
