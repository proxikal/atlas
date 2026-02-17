# Phase-11c Handoff: Async Primitives - Stdlib Registration Required

**Status:** 🟡 IN PROGRESS
**Blocker:** Functions implemented but NOT registered in stdlib
**Next Agent Task:** Register async primitive functions + complete tests

---

## What Was Discovered

The async primitive **implementations exist** but aren't callable from Atlas code:
- ✅ `task.rs` (387 lines) - spawn_task, TaskHandle, etc.
- ✅ `channel.rs` (258 lines) - channels implemented
- ✅ `primitives.rs` (296 lines) - sleep, timeout, mutex
- ❌ **NOT registered in stdlib/mod.rs**

Only `await` and future functions are registered. Need to add ~20-25 more.

---

## What Needs To Be Done

### 1. Create stdlib wrapper module (~600 lines)
**File:** `crates/atlas-runtime/src/stdlib/async_primitives.rs` (NEW)

Implement wrappers for:
- spawn, taskJoin, taskStatus, taskName, taskId, taskCancel, joinAll
- channelBounded, channelUnbounded, channelSend, channelReceive, channelSelect
- sleep, timer, interval, cancelTimer
- timeout, retryWithTimeout
- asyncMutex, asyncMutexLock, asyncMutexGet, asyncMutexSet, asyncMutexUnlock

**Pattern:** Follow `stdlib/future.rs` - extract args, validate, call runtime function, return Value

### 2. Update Value enum
**File:** `crates/atlas-runtime/src/value.rs`

Add variants:
```rust
TaskHandle(Arc<Mutex<TaskHandle>>),
ChannelSender(Arc<Mutex<ChannelSender>>),
ChannelReceiver(Arc<Mutex<ChannelReceiver>>),
AsyncMutex(Arc<tokio::sync::Mutex<Value>>),
```

Update: type_name(), Display, Clone

### 3. Register in stdlib
**File:** `crates/atlas-runtime/src/stdlib/mod.rs`

- Add `pub mod async_primitives;`
- Add function names to `is_builtin()`
- Add match arms in `call_builtin()`

### 4. Fix tests
**File:** `crates/atlas-runtime/tests/async_primitives_tests.rs` (EXISTS - 47 tests)

**Critical:** Change `await expr` to `await(expr)` (it's a function, not syntax!)

Add 16 more tests to reach 63+

### 5. Run & verify
```bash
cargo test -p atlas-runtime --test async_primitives_tests
cargo clippy -p atlas-runtime -- -D warnings
cargo fmt -p atlas-runtime
```

---

## Key Files

**Study these patterns:**
- `stdlib/future.rs` - How to wrap async functions
- `stdlib/async_io.rs` - Async wrapper examples
- `async_runtime/task.rs` - What you're wrapping

**Test file ready:** `tests/async_primitives_tests.rs` (needs function registrations)

---

## Acceptance

- ✅ 23+ functions registered
- ✅ 63+ tests passing
- ✅ Clippy clean
- ✅ Update STATUS.md: phase-11c complete

---

**Estimate:** 4-6 hours
**Phase File:** `phases/stdlib/phase-11c-async-primitives.md`
