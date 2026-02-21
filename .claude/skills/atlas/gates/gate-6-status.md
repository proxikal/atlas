# GATE 6: Update STATUS.md (Structured Development Only)

**Condition:** Structured development workflow, all gates passed

---

## Action (MANDATORY — do not skip)

Update STATUS.md with EXACTLY these two fields:

```
**Last Completed:** phases/v0.3/block-0X-{block-name}/phase-NN-{phase-name}.md
**Next Phase:** phases/v0.3/block-0X-{block-name}/phase-NN+1-{next-phase-name}.md
```

### Rules

1. **Last Completed** → the phase file you just finished
2. **Next Phase** → the next sequential phase file in the same block
3. **If this was the last phase in a block:**
   - Set `**Last Completed:**` to the final phase of this block
   - Set `**Next Phase:**` to `BLOCK COMPLETE — run acceptance criteria in V03_PLAN.md`
   - Update the block's status in the Block Progress table: `⬜ Not started` → `✅ Complete`
4. **Always update "Last Updated" date** at the top of STATUS.md
5. **Update Block Progress table** if block status changed

### Example (mid-block)

```markdown
**Last Completed:** phases/v0.3/block-01-memory-model/phase-03-migrate-collection-variants.md
**Next Phase:** phases/v0.3/block-01-memory-model/phase-04-implement-shared-type.md
```

### Example (block complete)

```markdown
**Last Completed:** phases/v0.3/block-01-memory-model/phase-25-commit-and-handoff.md
**Next Phase:** BLOCK COMPLETE — verify acceptance criteria in V03_PLAN.md before scaffolding Block 2
```

---

**BLOCKING:** Required for structured development. An agent starting a new session reads
STATUS.md first — if Last Completed and Next Phase are stale, they will re-execute completed
work or skip work entirely. This field is the handoff contract between sessions.

**Next:** GATE 7
