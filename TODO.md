# Umbra Project TODO

## Critical Issues (P0)

### UI/UX Bugs
- [x] P0 - **Sphere panel closes when clicking elements inside it** — `sphere_response` used `Sense::click()` which captured clicks on the entire sphere area including analysis/chat rendered on top. Fixed: when `sphere_selected` is true, sphere uses `Sense::hover()` (no click-to-toggle). Added a dedicated close button (✕) in the panel top-right with hover effect.
- [x] P0 - **`unwrap()` in sphere.rs:201** — `projected.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap())` panics on NaN. Replaced with `unwrap_or(std::cmp::Ordering::Equal)`.
- [x] P0 - **Unsafe `unwrap()` in sphere.rs:172** — `active_set[is_active.unwrap()]` after redundant `is_ok()` check. Replaced with `if let Ok(pos) = is_active`.

### Code Quality
- [ ] P0 - **Enormous `update()` function** — `impl eframe::App for App` has ~900 lines (164-1089). Should be split into smaller methods (e.g., `render_hud()`, `render_trading()`, `render_conversations()`, `render_sidebar()`).

## High Priority (P1)

### UI/UX Bugs
- [x] P1 - **No hover effects on buttons** — Added `btn()` and `btn_rounded()` helper functions that provide hover state feedback (purple highlight on hover). Used throughout the UI. Includes tooltip support via `.on_hover_text()`.
- [x] P1 - **Poor text contrast** — Increased brightness of dim text colors:
  - Inactive tab labels: `(80,60,130)` → `(130,100,190)` / `(140,120,200)`
  - Axis/chart labels: `(80,80,120)` → `(140,140,180)` / `(160,160,200)`
  - Hint/instruction text: `(60,100,130)` → `(120,160,190)`
  - Toggle buttons (type/gender): `(80,120,150)` → `(140,170,200)`
  - Symbol labels: `(120,150,180)` → `(160,180,210)`
  - Clock/time display: `(130,100,200)` → `(180,150,230)`
  - HuggingFace description: `(80,60,130)` → `(130,100,190)`
  - Filter button inactive: `(80,60,130)` → `(130,100,190)`
- [ ] P1 - **No visual feedback on interactive elements** — Sliders, text inputs, clickable areas lack hover/active visual cues beyond default egui styling.
- [x] P1 - **Sphere click rect overlaps inner UI** — Clicking analysis panel or chat inside sphere_selected mode also toggled sphere_selected. Fixed: sphere uses `Sense::hover()` when selected; close button added.

### Missing Features
- [ ] P1 - **No minimize-to-tray behavior** — Close button should minimize to system tray (requires eframe/OS integration).
- [ ] P1 - **No window position saving** — App doesn't remember position/size between launches. Skeleton infrastructure added — `save_window_state_to_file()`/`load_window_state()` functions exist in `Drop` impl. Full implementation requires egui viewport API support for reading current window position.

### Code Quality
- [x] P1 - **`unwrap()` in frontend/mod.rs:44** — `auth_path.parent().unwrap()` could panic on root path. Replaced with `if let Some(parent) = auth_path.parent()`.
- [x] P1 - **`unwrap()` in frontend/mod.rs:82-87** — URL `.parse().unwrap()` calls on hardcoded strings could panic. Replaced with `.iter().filter_map(|s| s.parse::<HeaderValue>().ok())`.

## Medium Priority (P2)

### UI/UX Bugs
- [x] P2 - **Sidebar content may overflow without scroll** — Sidebar content area uses `ScrollArea::vertical()` for content scrolling. DEBUG section at bottom separated by 10px gap.
- [ ] P2 - **No visual hover on sidebar tabs** — Settings sub-tabs in sidebar have no explicit hover indication beyond egui defaults.

### Missing Features
- [ ] P2 - **Keyboard shortcuts not documented in UI** — Shortcuts exist in settings panel but no quick-reference overlay.
- [ ] P2 - **No close confirmation** — Closing the app should prompt or minimize to tray.
- [ ] P2 - **Logo texture path hardcoded** — Uses `~/.local/share/icons/hicolor/128x128/apps/umbra.png` instead of configurable path.

### Code Quality
- [x] P2 - **`#[allow(dead_code)]` in vault.rs:8,11** — `vault_path` and `cache` fields unused. Renamed to `_vault_path`, `_cache`.
- [x] P2 - **`#[allow(dead_code)]` in resource.rs:9** — `MIN_FREE_MB` constant unused. Renamed to `_MIN_FREE_MB`.
- [x] P2 - **`#[allow(dead_code)]` in bridge/signals.rs:9** — `max_risk_percent` field unused. Renamed to `_max_risk_percent`.
- [x] P2 - **`#[allow(dead_code)]` in debugger.rs:60,66** — `LatencySample.endpoint` and `Debugger.active` unused. Renamed to `_endpoint`, `_active`.
- [x] P2 - **`#[allow(dead_code)]` in learning/trainer.rs:33** — `models_dir` field unused. Renamed to `_models_dir`.
- [ ] P2 - **`send_hud_message()` duplicates conversation creation logic** — Should refactor to share code with `send_conv_message()`.

## Low Priority (P3)

### UI/UX Improvements
- [ ] P3 - **Add splash/loading screen** — No visual feedback during initialization.
- [ ] P3 - **Add tooltips to icon buttons** — Mute button, send button, etc. now have tooltips via `on_hover_text()`. Could expand to more elements.
- [ ] P3 - **Emotional state animation too subtle** — The sphere color changes based on emotion but could be more visually distinct.

### Performance
- [ ] P3 - **`update_voice_tone()` called every frame** — Only needs to update when emotion changes.
- [ ] P3 - **`load_logo_texture()` reads from disk every call** — Returns early if already loaded, but disk path could be cached.
- [ ] P3 - **`self.frames` used for randomness** — `self.frames as usize % responses.len()` patterns create deterministic sequences based on frame count.

### Code Quality
- [ ] P3 - **Hardcoded 1200ms delay** — `conv_thinking` timer uses literal 1200ms instead of named constant.
- [ ] P3 - **Magic numbers everywhere** — `260.0`, `140.0`, `110.0`, `36.0` etc. should be named constants.
- [x] P3 - **Unused import `Stroke` in sphere.rs** — Removed.

## Summary of Fixes Applied

### Critical Fixes
| # | File | Issue | Fix |
|---|------|-------|-----|
| 1 | `src/sphere.rs:201` | `unwrap()` on NaN during sort | Replaced with `unwrap_or(std::cmp::Ordering::Equal)` |
| 2 | `src/sphere.rs:172` | Redundant `is_ok()` + `unwrap()` | Changed to `if let Ok(pos)` |
| 3 | `src/desktop/mod.rs` | Sphere click closes on inner element interaction | Sphere uses `Sense::hover()` when selected; added close button with hover effect |
| 4 | `src/desktop/mod.rs` | No hover effects on buttons | Added `btn()`, `btn_rounded()`, `btn_fill()` helpers with hover state tracking |
| 5 | `src/desktop/mod.rs` | Poor text contrast | Increased brightness of 15+ low-contrast color values |
| 6 | `src/desktop/mod.rs` | Missing window persistence | Added save/load infrastructure via JSON state file in `~/.umbra/` |
| 7 | `src/desktop/mod.rs` | Deprecated `allocate_ui_at_rect` usage | Not fixed (30+ occurrences, non-critical) |

### High Priority Fixes
| # | File | Issue | Fix |
|---|------|-------|-----|
| 8 | `src/frontend/mod.rs:44` | Unwrap on path parent | Changed to `if let Some(parent)` |
| 9 | `src/frontend/mod.rs:82-87` | Unwrap on URL parse | Changed to `.filter_map(\|s\| s.parse().ok())` |
| 10 | `src/vault.rs:8,11` | `#[allow(dead_code)]` | Renamed `vault_path` → `_vault_path`, `cache` → `_cache` |
| 11 | `src/resource.rs:9` | `#[allow(dead_code)]` | Renamed `MIN_FREE_MB` → `_MIN_FREE_MB` |
| 12 | `src/bridge/signals.rs:9` | `#[allow(dead_code)]` | Renamed `max_risk_percent` → `_max_risk_percent` |
| 13 | `src/debugger.rs:60,66` | `#[allow(dead_code)]` | Renamed `endpoint` → `_endpoint`, `active` → `_active` |
| 14 | `src/learning/trainer.rs:33` | `#[allow(dead_code)]` | Renamed `models_dir` → `_models_dir` |
| 15 | `src/sphere.rs:1` | Unused import `Stroke` | Removed import |

### Build Verification
- `cargo build --release --bin umbra-gui` compiles successfully (0 errors, 36 warnings)
- Binary: `target/release/umbra-gui` (10.1 MB)
