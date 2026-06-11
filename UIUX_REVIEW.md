# Umbra UI/UX Review

## Current Issues

### Critical
1. **1000+ line `update()` function** (`src/desktop/mod.rs:181-1192`) — Single monolithic method handles ALL rendering, event processing, data generation, state mutation. Impossible to test, debug, or maintain.
2. **Broker password in plaintext** (line 72, 1110) — `broker_password: String` stored in-memory and displayed in a password field but never zeroed. Any memory dump leaks the password.
3. **Mock responses disguised as real AI** (lines 1327-1347, 1353-1363) — Trading AI and conversation responses are hardcoded strings with no actual AI inference. Users will believe trading is functional.

### High
4. **Fake price data** (lines 452-458, 552-567) — Hardcoded prices never update. Chart shows synthetic sine-wave data, not real market data.
5. **SYMBOL_SETS declared but unused** (lines 80-86) — 5 symbol sets defined but never referenced. Symbols are hardcoded again on line 452.
6. **Duplicate hsv_to_rgb** — Identical HSV→RGB conversion in `sphere.rs` and `agent_memory.rs`. Violates DRY.
7. **Missing loading states** — No skeleton screens, spinners, or progress indicators for any async operation (model download, provider connection, TTS).
8. **Clock allocates every frame** (line 330) — `format!()` called every 16ms for time display. Impacts battery on laptops.

### Medium
9. **Inconsistent font sizes** — Code uses 8px, 9px, 10px, 11px, 12px, 13px, 14px, 18px mixed arbitrarily. No typographic scale.
10. **Magic numbers everywhere** — Widths (440.0, 360.0, 180.0, 110.0), heights (36.0, 28.0, 30.0), positions (260.0, 140.0, 200.0) scattered with zero comments.
11. **String allocation per frame in tab arrays** (lines 858, 874) — `["SETTINGS", "SKILLS", ...]` and `["VAULT", "PROVIDERS", ...]` re-created on every render. Should be `const`.
12. **Gender vectors re-allocated per frame** (line 967) — `genders` vec created in each render pass instead of being a `const`.
13. **`update_voice_tone()` called every frame** (line 192) — Only needs to fire when emotion changes.
14. **Dead variable `_dim_bg`** (line 206) — Computed every frame but never used.
15. **Emotion alpha calculation every frame** (lines 373-375) — Fine for now, but could be optimized.

### Low
16. **Emoji in code** (lines 333, 389, 716, 736, 764, 811, 867, 888, 901, 976, 988, 1169) — Hardcoded emoji as UI icons. Cross-platform rendering varies. Prefer SVG icons or font-based icons.
17. **Hardcoded logo path** (line 1198) — `~/.local/share/icons/hicolor/128x128/apps/umbra.png`. Should be configurable.

## Design System

### Colors
Currently hardcoded RGB values everywhere. Define a color palette:

```
// Design Tokens
// Backgrounds
BG_PRIMARY:       Color32::from_rgb(0, 4, 12)     // Near-black (lines 205, 706)
BG_PANEL:         Color32::from_rgb(0, 6, 16)     // Panel background (lines 408, 437, 848)
BG_PANEL_LIGHT:   Color32::from_rgb(0, 8, 20)     // Lighter panel (lines 386, 668, 711, 808)
BG_OVERLAY:       Color32::from_rgba(0, 2, 8, 50) // Sidebar overlay (line 841)

// Text
TEXT_PRIMARY:     Color32::from_rgb(0, 220, 100)  // Green accent (used everywhere as `primary`)
TEXT_DIM:         Color32::from_rgb(130, 100, 190) // Dim labels (lines 419, 450, 606, 879)
TEXT_MUTED:       Color32::from_rgb(80, 80, 120)  // Muted hints (lines 639, 674)
TEXT_BRIGHT:      Color32::from_rgb(180, 200, 220) // Bright body (lines 391, 813)

// Semantic
GREEN_BUY:        Color32::from_rgb(0, 220, 100)  // Buy/P&L positive (lines 415, 486, 587)
RED_SELL:         Color32::from_rgb(220, 50, 50)  // Sell/P&L negative (lines 486, 587, 634)
PURPLE_ACCENT:    Color32::from_rgb(124, 58, 237) // User messages (lines 414, 680, 765)
HOVER_PURPLE:     Color32::from_rgba(167, 139, 250, 40) // Hover state (line 8)
BORDER_DIM:       Color32::from_rgba(40, 30, 80, 80)    // Panel borders (lines 409, 438, 503)
```

**Issues:**
- Primary color varies by theme theme (blue/purple/green/red/white) but all use the `primary` / `primary_color()` method — good pattern, partially implemented.
- `HOVER_PURPLE` is the only design token extracted (line 8). All other colors are inline.
- Color contrast on `TEXT_DIM` (130, 100, 190) against `BG_PANEL` (0, 6, 16) is ~6.5:1 — acceptable. Muted text (80, 80, 120) on same background is ~4.2:1 — marginal.
- Red/green for buy/sell is fine for trading but red-green color blindness affects 8% of males; consider adding shapes or patterns.

### Typography
```
// Font scale (monospace for terminal aesthetic)
const FONT_XXS: f32 = 8.0;   // Legal/disclaimer text (line 1114)
const FONT_XS: f32 = 9.0;    // Labels, metadata, chart values (lines 394, 412, 450)
const FONT_SM: f32 = 10.0;   // Body text, input fields (lines 377, 391, 813)
const FONT_MD: f32 = 11.0;   // Tab labels, panel headers (lines 321, 508, 714)
const FONT_LG: f32 = 12.0;   // Section titles, conversation titles (lines 631, 634, 757)
const FONT_XL: f32 = 13.0;   // Settings headers (lines 389, 811, 886, 897)
const FONT_XXL: f32 = 14.0;  // Primary headers, About title (lines 389, 1170)
const FONT_HUGE: f32 = 18.0; // Menu icons, hamburger (lines 307, 1080)
```

**Issues:**
- 8px is below minimum readable size (especially for UI labels on line 1114, 1116). Move to 9px minimum, 10px preferred for body.
- Font sizes jump erratically within 50-line spans (e.g., lines 389-396: 14, 10, 9, 9). Standardize.
- `RichText::new(...).size(N).monospace()` is the universal pattern — good, but inconsistent N values.

### Spacing
```
const SPACING_XXS: f32 = 2.0;
const SPACING_XS: f32 = 4.0;   // (lines 311, 332, 625)
const SPACING_SM: f32 = 6.0;   // (lines 1106, 1111, 1132)
const SPACING_MD: f32 = 8.0;   // (lines 605, 633, 754)
const SPACING_LG: f32 = 10.0;  // (lines 407, 436, 1161)
const SPACING_XL: f32 = 12.0;  // (lines 389, 808, 850)
const SPACING_XXL: f32 = 20.0; // (lines 385)
```

**Issues:**
- `ui.add_space(4.0)`, `ui.add_space(6.0)` scattered. Use named constants.
- Panel padding varies: analysis panel uses 12px (line 388), sidebar uses 12px (line 858), trading panels use 8px (line 605), chat uses 6px (line 670). Inconsistent.

## Accessibility

### Font Size Compliance
- **Minimum readable: 11px** for body text (WCAG AA for UI text). Current code uses 8px and 9px extensively.
- 8px is used for: broker list (line 1116), provider status (line 912), chart timeframe labels (line 518), hint text (line 620).
- **Recommendation:** Floor at 10px for metadata, 11px for interactive elements, 12px for body.

### Color Contrast
| Pair | Foreground | Background | Ratio | WCAG AA |
|---|---|---|---|---|
| Muted labels | (80,80,120) | (0,6,16) | 4.2:1 | ❌ (needs 4.5:1 for text) |
| Dim tabs | (130,100,190) | (0,6,16) | 6.5:1 | ✅ |
| Primary text | (0,220,100) | (0,6,16) | 8.1:1 | ✅ |
| Hover button | (167,139,250,40%) | (0,4,12) | ~1.5:1 | ❌ (non-text contrast needs 3:1) |
| Sell button text | (220,50,50) | (60,0,0,100) | ~2.1:1 | ❌ |

**Fixes:**
- Hover background: increase opacity from 40 to 60, or use a lighter purple.
- Sell button: increase text to (255,80,80) and background to (80,0,0,120).
- Muted labels: brighten to at least (100,100,150).

### Button Hover States
- `HOVER_PURPLE` is applied in `btn()`, `btn_rounded()`, `btn_fill()` — good pattern.
- **Inconsistency:** Some buttons use `btn()` helpers (lines 307, 334, 419), others use raw `Button::new()` with manual hover tracking (lines 467, 518, 733). The manual hover tracking duplicates the logic already in `btn_fill()`.
- **Recommendation:** Replace all `Button::new()` with `btn_fill()` to ensure consistent hover behavior.

### Keyboard Navigation
- Tab order is default egui behavior — acceptable.
- Shortcut recording works but no visual indicator of which element is focused.
- No `:focus` visible style — keyboard users cannot track focus.

### Screen Reader
- egui has no native accessibility tree support in v0.29. No fix possible at this layer.
- Angular frontend has better a11y (ARIA labels) — recommend investing there for accessibility.

## Professional Trading UI

### TradingView/cTrader Pattern Analysis

**What TradingView does well:**
- **Dark theme with proper contrast** — Pure black background (#131722), bright cyan/white for data, green/red for directionality. Umbra's (0,4,12) near-black is a good start.
- **Price scale on right** — Contextual, always visible. Umbra has it on the right (line 579) — correct.
- **Timeframe strip** — Horizontally scrollable. Umbra has fixed timeframes (line 510) — acceptable for MVP.
- **Order panel** — Floats or docks. Umbra uses a bottom panel (line 602) — fine.
- **Instrument search** — Type to find. Umbra has hardcoded symbols — needs search.

**What cTrader does well:**
- **Clean, uncluttered layout** — Minimalist, information-dense without feeling crowded. Umbra is on the right track but panels could be tighter.
- **One-click trading** — Buy/sell buttons with volume preset always visible. Umbra has buy/sell in order panel (line 627) — good placement.
- **Position watchlist** — Table view with P&L, swap, exposure. Umbra has a simple list (line 480) — needs columns.

### Missing Trading UI Features
1. **Order book / depth of market** — Essential for serious trading.
2. **Multiple chart types** — Only line + candle stubs. No Heikin Ashi, Renko, Point & Figure.
3. **Indicators** — No overlay (MA, EMA, Bollinger, RSI, MACD). Trading floor requirement.
4. **Multiple timeframes visible simultaneously** — Pros want 1m/5m/1H/4H side-by-side.
5. **Alerts** — Price level alerts (e.g., "alert when BTCUSD crosses 65000").
6. **Trade history** — Closed positions, P&L over time.
7. **Grid trading** — Popular automated strategy, not present.

### Dark Theme Recommendations
```
// Professional Trading Dark Theme (inspired by TradingView)

// Backgrounds
BG_MAIN:        Color32::from_rgb(0, 4, 12)       // Main canvas (current)
BG_PANEL:       Color32::from_rgb(7, 11, 20)      // Slightly lighter panel
BG_INPUT:       Color32::from_rgb(14, 19, 28)     // Input fields
BG_HOVER:       Color32::from_rgb(21, 28, 40)     // Hover state

// Chart
CHART_GRID:     Color32::from_rgba(100, 100, 140, 30)  // Grid lines
CHART_CURSOR:   Color32::from_rgba(255, 255, 255, 100) // Crosshair

// Semantic
BUY_GREEN:      Color32::from_rgb(22, 199, 132)   // cTrader green
SELL_RED:       Color32::from_rgb(234, 73, 73)    // cTrader red
NEUTRAL:        Color32::from_rgb(120, 140, 180)   // Neutral text

// Accents
ACCENT_CYAN:    Color32::from_rgb(0, 200, 255)    // PANEL_BORDER color
ACCENT_PURPLE:  Color32::from_rgb(124, 58, 237)   // User messages
```

## Recommendations

### Immediate Fixes (Before Next Build)
1. **[P0] Split `update()`** into `render_hud()`, `render_trading()`, `render_conversations()`, `render_sidebar()`, `handle_shortcuts()`, `handle_emotion()`. Each <100 lines.
2. **[P0] Use Vault for `broker_password`** — Zeroize on drop, do not derive Clone/Debug, store in vault abstraction.
3. **[P1] Extract all colors into const design tokens** — Single palette file (`src/ui/palette.rs`). No more inline RGB.
4. **[P1] Extract all font sizes into const scale** — `FONT_XS` through `FONT_XXL`. Enforce floor of 10px.
5. **[P1] Extract all spacing into const tokens** — `SPACING_XS` through `SPACING_XXL`.
6. **[P1] Replace all `Button::new()` with `btn_fill()`** — Ensure consistent hover states everywhere.

### Short-Term (Next 2 Sprints)
7. **[P1] Real-time clock only updates on second change** — Cache formatted string, update only when `s` changes.
8. **[P1] Remove dead `SYMBOL_SETS`** — Use it or delete it. Currently dead code.
9. **[P1] Make `detect_local_models()` async** — Blocking HTTP calls on startup delay UI. Use `tokio::spawn_blocking`.
10. **[P2] Replace emoji icons with vector icons** — Use a custom font (Material Symbols or Font Awesome) for consistent cross-platform rendering.
11. **[P2] Add search/filter to symbol list** — Hardcoded 5 symbols is not usable for real trading.
12. **[P2] Add keyboard shortcuts overlay** — `?` to show quick reference.

### Medium-Term (Sprint C onwards)
13. **[P2] Implement proper chart rendering** — egui has `egui_plot` crate. Use it instead of manual line drawing.
14. **[P2] Add loading indicators** — Spinners or skeleton screens for provider detection, model download, TTS connection.
15. **[P3] Theme persistence** — Save selected theme/primary color to config. Currently resets on restart.
16. **[P3] Window state persistence** — Save position and size. Infrastructure exists (Drop impl), but viewport API integration is incomplete.
17. **[P3] Make logo path configurable** — Move from hardcoded `~/.local/share/icons/hicolor/...` to config setting.
18. **[P3] Add "simulated" badge** — Clearly label trading panel as SIMULATED to avoid user confusion about fake P&L data.

### Trading Panel Specific
19. **[P1] Replace hardcoded prices with real data feed** — Fake prices undermine trust. Label clearly as "simulated" until real feed is connected.
20. **[P1] Provide real mock generation** — If keeping simulated mode, generate prices with better stochastic model (GARCH/Ornstein-Uhlenbeck) not just `sin()`.
21. **[P2] Add columns view for positions** — Current horizontal layout wastes space. Use table with columns: Symbol, Direction, Entry, Current, P&L, Swap.
22. **[P2] Add order confirmation dialog** — "Are you sure you want to BUY 0.1 BTCUSD?" prevents fat-finger errors.
