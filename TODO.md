# Improvement TODO List

> Generated from code review. Items are grouped by priority.

---

## P0 — Critical

### [ ] Replace pervasive `.unwrap()` with proper error handling
- **Files:** `src/base_lib.rs`, `src/handles.rs`, `src/business_handles.rs`, `src/stream_handlers.rs`
- **Details:** Nearly every fallible operation uses `.unwrap()`, causing the server to panic on any error (DB unavailable, file missing, bad input, etc.). Handlers should return `Result<Json<T>, StatusCode>` and propagate errors with `?`.
- **Key spots:**
  - `base_lib.rs:44` — `Connection::open(db_path_env).unwrap()`
  - `stream_handlers.rs:613` — `File::open(file_path).unwrap()`
  - `handles.rs:400` — `designation.char_final.unwrap()`
  - `business_handles.rs:241` — `result.unwrap().clone()`

### [ ] Fix SQL injection risk from string concatenation
- **File:** `src/business_handles.rs:485-494`
- **Details:** `statistic_handler` builds SQL via string concatenation:
  ```rust
  let sql1 = "... where base_index = ".to_string() + &id.to_string();
  ```
  Always use parameterized queries (`named_params! {":id": id}`).

### [ ] Add null checks for FFI raw pointers
- **Files:** `src/handles.rs`, `src/video_name_util.rs`, `src/stream_handlers.rs`
- **Details:** C library return pointers are dereferenced without null checks.
  - `video_name_util.rs:19-28` — `p_meta_info` could be null
  - `handles.rs:650-656` — `snapshot_st.buff` raw pointer used in `from_raw_parts`
  - `stream_handlers.rs:222-230` — `avif_to_png` return pointer

---

## P1 — High

### [ ] Introduce a SQLite connection pool
- **Details:** Every request creates a new `Connection`. Use `r2d2` + `rusqlite` (or `deadpool-sqlite`) to reuse connections.

### [ ] Guard test modules with `#[cfg(test)]`
- **File:** `src/main.rs:38-41`
- **Details:** `test_aes`, `test_designation`, `test_main`, `test_video_name_util` are compiled into release binaries. Wrap them in `#[cfg(test)]` or move to `tests/` directory.

### [ ] Replace `thread::spawn` with `tokio::task::spawn_blocking`
- **Files:** `src/handles.rs:543`, `src/handles.rs:869`, `src/handles.rs:990`
- **Details:** Blocking IO and CPU-bound work inside async handlers should run on Tokio's blocking thread pool, not the async worker threads.

### [ ] Simplify `query_tags_handler` custom Future
- **File:** `src/business_handles.rs:538-596`
- **Details:** Hand-written `Future` with `Arc<Mutex<St>>` and manual waker management is unnecessary. Rewrite as a plain `async fn`.

### [ ] Cache mount config queries
- **Details:** `query_mount_configs()` is called repeatedly per request. Cache results with `std::sync::OnceLock` or similar, refreshing on demand.

---

## P2 — Medium

### [ ] Extract common response building logic
- **Files:** All handler modules
- **Details:** The following header setup is duplicated 20+ times:
  ```rust
  let mut header = HeaderMap::new();
  header.insert(ACCESS_CONTROL_ALLOW_ORIGIN, "*".parse().unwrap());
  header.insert(CONTENT_TYPE, "application/json; charset=utf-8".parse().unwrap());
  ```
  Create a helper like `json_response<T>(body: T) -> (StatusCode, HeaderMap, Json<T>)`.

### [ ] Centralize OS-specific path field selection
- **Files:** `src/base_lib.rs`, `src/handles.rs`, `src/business_handles.rs`, `src/stream_handlers.rs`
- **Details:** The logic choosing `dir_path` / `mac_dir_path` / `win_dir_path` is duplicated in four files. Extract to a single utility function.

### [ ] Remove duplicated functions
- **Details:**
  - `get_sqlite_connection` exists in both `base_lib.rs` and `business_handles.rs`
  - `parse_dir_path` exists in both `base_lib.rs` and `business_handles.rs`

### [ ] Fix typos
- **File:** `src/entity.rs:143`
  - `duratoin` -> `duration`
- **File:** `src/stream_handlers.rs:572`
  - `readed_lenght` -> `readed_length`
- **File:** `src/designation.rs`
  - `tranc_code` -> `trans_code`

### [ ] Clean up dead / debug code
- **File:** `src/stream_handlers.rs:545`
  - `FileStream::new()` hardcodes a Java source file path (`/home/knightingal/source/jflow1000-server/...`). Remove or replace with a meaningful default.
- **File:** `src/main.rs:253-304`
  - `root()` contains commented-out route definitions and GitHub host IP lists. Remove or move to documentation.
- **File:** `src/stream_handlers.rs`
  - Remove large blocks of commented-out code.

### [ ] Eliminate hardcoded paths
- **Details:** Multiple absolute paths are hardcoded to developer's home directory:
  - `base_lib.rs:43` — `/home/knightingal/source/keys/mp41000.db`
  - `stream_handlers.rs:205` — `/home/knightingal/linux1000/`
  - Use environment variables with sensible defaults, not personal directories.

### [ ] Fix `find_cover_by_id` return type
- **File:** `src/base_lib.rs:287`
- **Details:** Returns `Vec<(u32, String, String, u64, u64)>` but only the first element is ever used. Return a single `Option<T>` or a dedicated struct instead of a 4-tuple.

### [ ] Improve `TagEntity` derive
- **File:** `src/entity.rs:207-214`
- **Details:** Manually implements `Clone` instead of using `#[derive(Clone, Serialize)]`.

### [ ] Consider Builder pattern for `VideoEntity`
- **File:** `src/entity.rs`
- **Details:** 5 constructors with many repeated default values. A builder or `Default` impl would reduce duplication.

### [ ] Fix crate self-reference
- **File:** `src/stream_handlers.rs:10`
- **Details:** `use rs_flow1000_parse::base_lib::IS_MACOS;` should be `use crate::base_lib::IS_MACOS;`.

---

## P3 — Low / Nice to have

### [ ] Strengthen Range request parsing
- **File:** `src/stream_handlers.rs:146-155`
- **Details:** Current implementation only handles `bytes=start-end`. Support:
  - `bytes=-500` (last 500 bytes)
  - `bytes=0-` (from start to end)
  - Reject malformed ranges gracefully instead of panicking.

### [ ] Make file deletion atomic with DB update
- **File:** `src/business_handles.rs:249-294`
- **Details:** `delete_video_handler` deletes video file, then cover file, then updates DB. If step 2 fails, the DB and filesystem are inconsistent. Consider wrapping in a transaction-like pattern or better error recovery.

### [ ] Replace `println!` with structured logging
- **Details:** Many handlers use `println!` for debug output. Use `tracing::debug!` / `tracing::info!` consistently.

### [ ] Optimize `sub_string_matched`
- **File:** `src/video_name_util.rs:95-121`
- **Details:** Current implementation is O(n*m) character-by-character comparison. Consider using standard string search methods.

---

## Reviewed

- [x] 2026-05-30 — Initial review completed.
