# Pending Documentation Updates

**Purpose:** Queue non-critical doc updates for batched processing

**When to process:** Every 10-20 phases OR when category completes

**Last batch:** Never (initial file)
**Next batch:** Phase 10 or first category completion

---

## docs/api/stdlib.md
**Trigger:** All stdlib category phases complete

**Phase 04 - JSON & Type Utilities (2026-02-14):**
- Add 5 JSON functions: parseJSON, toJSON, isValidJSON, prettifyJSON, minifyJSON
- Add 7 type guard functions: typeof, isString, isNumber, isBool, isNull, isArray, isFunction
- Add 5 type conversion functions: toString, toNumber, toBool, parseInt, parseFloat
- Total: 17 new stdlib functions

---

## docs/specification/language-semantics.md
**Trigger:** Every 20 phases or semantics category complete

(No updates queued yet)

---

## docs/api/runtime-api.md
**Trigger:** All runtime category phases complete

(No updates queued yet)

---

## Other docs
**Trigger:** As needed

(No updates queued yet)

---

## Batch Processing Instructions

**When triggered (category complete or phase 10/20/30/etc):**

1. Read this file
2. Group updates by document
3. Update each document ONCE with all queued changes
4. Clear processed updates from this file
5. Update "Last batch" timestamp

**Cost savings:** Batching 10 updates = $0.40 vs individual updates = $1.40 (save $1.00)
