# Deep Code Audit & Fix Plan

## 1. 核心性能审计 (Performance Audit)

### 🔴 Critical Issue: O(N*M) Complexity in Naming Check
**Location**: `src-tauri/src/commands/naming_check.rs` (`match_naming`, `auto_fix_naming`)
**Issue**:
The current implementation performs a "Fast Match" which is actually an O(N*M) operation.
- **N** (ROMs) can be ~1,000s.
- **M** (CSV Database) is ~10,000s.
- Inside the loop, `smart_cn_similarity` converts strings to Pinyin and calculates Jaccard/Jaro-Winkler similarity, which is computationally expensive.
- Total operations: 1,000 * 10,000 * Cost(Similarity) ≈ 10M+ heavy ops. This blocks the thread (even if `spawn_blocking` is used, it's slow).

**Fix Plan**:
1. **Indexing**: Pre-process the `cn_repo` and `jy6d` data into HashMaps.
   - `lookup_map_cn`: `HashMap<String, Vec<Entry>>` (Key: Chinese Name)
   - `lookup_map_en`: `HashMap<String, Vec<Entry>>` (Key: English Name)
2. **Optimized Lookup Flow**:
   - Check Exact Match (O(1)) -> Check Cleaned English Match (O(1)) -> Only then fallback to iterative fuzzy match.
3. **Fuzzy Scope Reduction**:
   - Filter fuzzy candidates by length difference (e.g., within +/- 3 chars) or pinyin initial.

### 🟡 Efficiency: Recursive Directory Scanning
**Location**: `src-tauri/src/rom_service.rs`
**Issue**:
Recursive scanning using `fs::read_dir` in a loop can be slow for large libraries, especially on network drives.
**Fix**:
- Current implementation uses `rayon`, which helps.
- Ensure `is_root_directory` logic doesn't re-scan deep trees unnecessarily if metadata exists. `try_load_from_temp_metadata` is a good existing safeguard.

---

## 2. 代码逻辑审计 (Logic Audit)

### 🟠 Logic Gap: Incomplete `get_metadata` in LocalCnProvider
**Location**: `src-tauri/src/scraper/local_cn.rs`
**Issue**:
`get_metadata` currently returns a mocked or minimal struct, sometimes hardcoding "Metadata not found" if it doesn't hit the cache exactly right. It relies on `source_id` being the English name, but doesn't cross-reference robustly.
**Fix**:
- Implement a proper lookup in `get_metadata` that can recover the full `CnRomEntry` from the `source_id`.

### 🟡 Error Handling
**Location**: Global (Backend)
**Issue**:
Frequent use of `.map_err(|e| e.to_string())` swallows original error types, making debugging harder if specific IO errors need handling (e.g., Permission Denied vs Not Found).
**Recommendation**:
- Use `anyhow` or specific `thiserror` enums for better error propagation in the future. (Low priority for now).

---

## 3. 前端优化建议 (Frontend Optimization)

### 🟢 React Performance
**Location**: `src/pages/CnRomTools.tsx`, `src/pages/Library.tsx`
**Issue**:
- `Library.tsx`: Filters ROMs in a `useEffect` based on search query. This causes a double-render (state update -> effect -> state update).
- **Fix**: Move filtering logic to the render phase (derive state) or use a selector in `zustand`.

### 🟢 UI/UX
- **Virtuoso/Virtualization**: The list implementations seem to use standard mapping. If lists grow > 1000 items, verify `react-window` or `virtuoso` is used (Plan mentions it, need to ensure `RomView` uses it).

---

## 4. Pending Action Items

1. **Refactor `naming_check.rs`**: Implement the `PreComputedIndex` struct and replace the linear scan in `auto_fix_naming`.
2. **Enhance `local_cn.rs`**: Fix `get_metadata` implementation.
3. **Frontend Tweak**: Optimize `Library.tsx` search filtering.
