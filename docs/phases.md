# AntSimulacrum — Development Phases

> Rust + macroquad. Desktop (Windows/macOS/Linux) + Web (WASM).
> Full design: [`gdd.md`](gdd.md) · UI reference: [`gui-design.md`](gui-design.md)

---

## Overview

```
Phase 1  Scaffold & Rendering     ██████████  [x] complete
Phase 2  Ants & Pheromones        ██████████  [x] complete
Phase 3  Colony & Lifecycle       ██████████  [x] complete
Phase 4  Living Ecology           ░░░░░░░░░░  [ ] not started
Phase 5  Player Interaction & UI  ░░░░░░░░░░  [ ] not started
Phase 6  Persistence & Web        ░░░░░░░░░░  [ ] not started
```

**Gate:** Phase 2 is the critical go/no-go. If pheromone trails don't look satisfying → revisit simulation parameters before continuing.

---

## Phase 1 — Scaffold & Basic Rendering

**Goal:** App launches, shows a world with placeholder shapes, camera works.

### Tasks
- [x] `cargo init`, add `macroquad` to `Cargo.toml`
- [x] Main game loop with delta time (`get_frame_time()`)
- [x] `World` struct: fixed-size grid of cells (`Empty | Wall | Food`)
- [x] `Camera` struct: offset + zoom, right-click drag to pan, scroll to zoom, Home key centers nest
- [x] Render layers (draw order):
  - [x] Background fill (`#1a1208` dark soil)
  - [x] Walls (filled grey rects)
  - [x] Food sources (green circles, radius = quantity)
  - [x] Nest (concentric rings, warm gold)
  - [x] Placeholder ants (amber dots, static)
- [x] FPS counter (top-left debug overlay)

### Files
```
Cargo.toml
src/main.rs       game loop, window init
src/world.rs      grid, cell types
src/camera.rs     pan/zoom transform
src/rendering.rs  all draw_* calls
```

### Done when
App window opens, dark world with gold nest ring, green food circles, grey walls, static amber dots. Camera pans and zooms smoothly without jitter.

---

## Phase 2 — Ant Movement & Pheromones

**Goal:** Ants wander, discover food, return to nest, form visible pheromone highways.
This is the **core visual gate** — if this doesn't look cool, nothing else matters.

### Tasks
- [x] `Ant` struct: `position`, `direction`, `speed`, `state: AntState`, `carrying_food: bool`
- [x] `AntState` enum: `Foraging | Returning`
- [x] Movement: advance along direction each tick, apply forward-cone noise (`±30°` random jitter)
- [x] `PheromoneGrid`: two `f32` grids same size as world — `to_home[x][y]` and `to_food[x][y]`
- [x] Pheromone deposit: ants deposit on current cell each tick with cooldown
  - Foraging ants → deposit `to_home`
  - Returning ants → deposit `to_food`
- [x] Pheromone decay: `intensity -= decay_rate * dt` each tick, clamp to 0
- [x] Pheromone follow: sample 32 directions in 90° forward cone, pick cell with highest matching channel
- [x] Trail degradation: `intensity *= 0.999` on chosen best cell only (prevents oversaturation)
- [x] Liberty coefficient: per-ant `f32` chance to skip pheromone sampling and wander freely
- [x] Wall collision: check target cell before moving, bounce on wall hit (reflect direction)
- [x] Food interaction: when Foraging ant reaches food cell → pick up (`carrying_food = true`, switch to `Returning`)
- [x] Nest interaction: when Returning ant reaches nest → deposit food, switch to `Foraging`
- [x] Pheromone render: draw colored quad per grid cell, `alpha = intensity / max_intensity`
  - `to_food`: amber `#ff9900` → transparent
  - `to_home`: blue `#0099ff` → transparent

### Implementation notes
- `to_food` and `to_home` decay at the same rate (unified for testing; reverted from 2× split)
- Returning ants use a direct nest-homing pull (15–60% weight by distance) alongside pheromone steering to prevent orbital loops around the nest
- Nest interaction radius = 44px (matches visual outer ring)
- `gen` is a reserved keyword in Rust 2024 edition — use `r#gen` when calling `rand::Rng::gen`
- Debug controls added: Shift+R reset, Shift+↑/↓ adjust decay rate live

### Files
```
src/ant.rs          Ant struct, update(), state machine
src/pheromone.rs    PheromoneGrid, deposit(), decay(), sample()
src/world.rs        extend: food quantity on cells, nest position
src/rendering.rs    extend: draw ants (circle + direction tick), pheromone overlay
```

### Done when
50+ ants launched from nest. Within 30 seconds, visible pheromone highways form between nest and food. Trails fade naturally when food depletes. Ants reroute when a food source empties.

---

## Phase 3 — Colony & Lifecycle

**Goal:** Colony grows organically over time. Queen lays eggs, brood matures, ants age and die.

### Tasks
- [x] `Colony` struct: `queen`, `food_stored: f32`, `population: u32`, `stats`
- [x] `Queen` struct: `health`, `egg_rate`, `last_egg_time` — lays eggs proportional to `food_stored`
- [x] `Brood` stages with timers:
  - `Egg` (60s) → `Larva` (120s) → `Adult` (assigned caste)
- [x] Caste assignment on maturity — weighted distribution:
  - Worker 70%, Scout 10%, Soldier 10%, Nurse 10%
- [x] Caste-specific behavior:
  - `Worker`: standard forage loop (Phase 2 behavior)
  - `Scout`: 180° detection cone, higher liberty coefficient, +30% speed
  - `Soldier`: patrol ring around nest (SOLDIER_PATROL_RADIUS = 85px, band ±20px)
  - `Nurse`: stay within 70px of nest, move at 70% speed
- [x] Food consumption: each ant costs `food_per_tick` from `colony.food_stored`
- [x] Ant lifespan: each ant has `age` counter, dies at max age
- [x] **Normal mode**: starvation (`food_stored == 0`) accelerates aging; queen stops laying
- [x] **Zen mode**: minimum population floor (never below 10 workers); queen immortal
- [x] Colony collapse condition (Normal): queen dead + no brood + 0 workers → trigger end screen

### Files
```
src/colony.rs    Colony, Queen structs, food economy
src/brood.rs     Egg/Larva lifecycle, caste assignment
src/ant.rs       extend: caste enum, lifespan, caste behaviors
```

### Implementation notes
- `food_stored` moved entirely to `Colony`; removed from `World`
- Ant lifespan: Worker 360s, Scout 240s, Soldier/Nurse 480s
- Starvation aging multiplier: 2.5× (Normal mode only)
- Egg rate scales with food: 0.5× scarce → 2.0× abundant (base interval 15s)
- Visual caste identification: Worker=amber, Scout=pale white-blue, Soldier=red, Nurse=lavender
- Initial colony starts at 20 workers (INITIAL_ANT_COUNT) to allow growth to be visible
- Zen minimum floor (10 workers) enforced by spawning workers directly at nest

### Done when
Colony starts at 20 ants, grows to 100+ over ~10 minutes. Population fluctuates with food supply. In Normal mode, starving a colony kills it. In Zen mode, colony never fully dies.

---

## Phase 4 — Living Ecology

**Goal:** The world feels alive with no player input. Food grows, cycles, and shifts.

### Tasks
- [ ] `FoodSource` struct: `position`, `quantity: f32`, `max_quantity: f32`, `regrowth_rate: f32`
- [ ] Regrowth tick: `quantity = (quantity + regrowth_rate * dt).min(max_quantity)`
- [ ] Natural food spawning: new sources appear at random intervals, clustered near existing ones (Poisson-disk-ish distribution)
- [ ] Food decay: sources sitting at max for too long slowly reduce `max_quantity` (simulates seasonal depletion)
- [ ] Day/night cycle:
  - Cycle length: ~5 min real-time per sim-day (configurable)
  - Visual: background color oscillates from `#1a1208` (day) to `#0a0a0a` (night)
  - Mechanical: `regrowth_rate *= day_modifier` (1.5x day, 0.5x night)
  - HUD: day counter + ☀/☾ icon in bottom bar

### Files
```
src/ecology.rs    FoodSource, spawning, regrowth, decay
src/world.rs      extend: day/night state, time accumulator
src/rendering.rs  extend: background color lerp for day/night
```

### Done when
Open app, watch for 5 minutes: new food sources appear naturally, existing ones deplete and regrow. Background visibly dims and brightens on cycle. Ants dynamically shift foraging routes as food landscape changes.

---

## Phase 5 — Player Interaction & UI

**Goal:** Player has all tools from the GDD. HUD shows live colony state. GUI matches `gui-design.md`.

### Tasks

**Tool system**
- [ ] `Tool` enum: `Observe | PlaceFood | DrawWall | DropAnts | Eraser`
- [ ] Active tool cursor: ghost overlay at mouse position
- [ ] Left-click: apply active tool at world coords (map screen pos through camera)
- [ ] Hotkeys: `1`–`5` to select tool, `Space` pause, `Home` center camera

**Speed controls**
- [ ] Speed multiplier: `Paused | 1x | 2x | Max`
- [ ] `dt_scaled = dt * speed_multiplier` fed to all simulation systems
- [ ] Keyboard: `Space` pause, number row or UI buttons for speed

**HUD — top bar** (matches `gui-design.md §2`)
- [ ] Ant count
- [ ] Food stored
- [ ] Queen status: `OK | LOW | CRITICAL | DEAD`
- [ ] Mode badge: `[ZEN]` / `[NORMAL]`
- [ ] Speed buttons
- [ ] Settings gear icon

**HUD — bottom bar** (matches `gui-design.md §3`)
- [ ] Tool selector buttons with active highlight
- [ ] Day counter + day/night icon + time-in-day

**Colony stats panel** (matches `gui-design.md §6`)
- [ ] Click nest → show panel anchored near nest
- [ ] Caste breakdown, brood counts, food economy, peak population, colony age

**Settings panel** (matches `gui-design.md §5`)
- [ ] Mode toggle
- [ ] Sim speed slider
- [ ] Pheromone layer toggle (ToFood / ToHome / Both / Off)
- [ ] Ant labels toggle
- [ ] Reset colony / New world buttons (with confirmation prompt)

**Milestone toasts** (matches `gui-design.md §7`)
- [ ] Toast queue: bottom-right, stacks up, auto-dismiss after 3s
- [ ] Events: first food delivered, population 50/100/500/1000, colony survived 1h/1d/1w, colony collapsed

### Files
```
src/ui.rs       HUD draw calls, panels, toasts
src/input.rs    mouse/keyboard dispatch, tool application
src/main.rs     extend: wire input → simulation → render
```

### Done when
All 5 tools work and react visibly. HUD updates live. Colony stats panel opens on nest click. Speed controls change simulation pace. Toasts appear for milestones.

---

## Phase 6 — Persistence & Web

**Goal:** Colony survives app restarts. Runs minimized to tray. Builds for web.

### Tasks

**Save / Load**
- [ ] Add `#[derive(Serialize, Deserialize)]` to: `World`, `PheromoneGrid`, `Colony`, `Vec<Ant>`, `Vec<FoodSource>`
- [ ] `save_game(path)`: serialize all state to `bincode` file
- [ ] `load_game(path)`: deserialize and restore — world, ants, pheromones, colony, ecology
- [ ] Auto-save on window close event
- [ ] Load on startup if save file exists; otherwise start fresh
- [ ] Save location:
  - Desktop: OS data dir (`dirs` crate) — e.g. `~/.local/share/antsimulacrum/save.bin`
  - Web: `localStorage` / IndexedDB (platform-abstracted)

**System tray (desktop only)**
- [ ] Conditional compilation: `#[cfg(not(target_arch = "wasm32"))]`
- [ ] `tray-item` crate: add tray icon on minimize
- [ ] Tray menu: "Show", "Pause sim", "Quit"
- [ ] Minimized state: sim continues running, rendering paused (no window update needed)
- [ ] Configurable: full speed vs. throttled (50% tick rate) when in tray

**WASM / Web**
- [ ] Add `wasm32-unknown-unknown` target
- [ ] Platform abstraction for save: `#[cfg(target_arch = "wasm32")]` → `localStorage`
- [ ] `index.html` wrapper with canvas element
- [ ] Test in Firefox + Chrome
- [ ] `cargo build --target wasm32-unknown-unknown --release`

### Files
```
src/persistence.rs    save/load, platform abstraction
src/main.rs           extend: close event, tray, startup load
Cargo.toml            add: serde, bincode, dirs, tray-item (desktop feature)
index.html            web wrapper
```

### Done when
- Close desktop app → reopen → colony exactly where it left off
- Minimize to tray → sim keeps running in background
- `cargo build --target wasm32-unknown-unknown` succeeds and runs in browser with save/load via localStorage

---

## Source File Map

```
AntSimulacrum2/
├── Cargo.toml
├── index.html                  (Phase 6 — web)
├── docs/
│   ├── gdd.md
│   ├── gui-design.md
│   └── phases.md               (this file)
└── src/
    ├── main.rs                 game loop, window, event wiring
    ├── world.rs                grid, cell types, spatial helpers
    ├── camera.rs               pan/zoom transform            (Phase 1)
    ├── rendering.rs            all draw_* calls              (Phase 1+)
    ├── ant.rs                  Ant struct, state machine     (Phase 2+)
    ├── pheromone.rs            PheromoneGrid                 (Phase 2)
    ├── colony.rs               Colony, Queen, food economy   (Phase 3)
    ├── brood.rs                Egg/Larva lifecycle           (Phase 3)
    ├── ecology.rs              FoodSource, day/night         (Phase 4)
    ├── ui.rs                   HUD, panels, toasts           (Phase 5)
    ├── input.rs                mouse/keyboard, tools         (Phase 5)
    └── persistence.rs          save/load, serde              (Phase 6)
```

---

## Key Dependencies

```toml
[dependencies]
macroquad = "0.4"
serde = { version = "1", features = ["derive"] }
bincode = "1"
rand = "0.8"

[target.'cfg(not(target_arch = "wasm32"))'.dependencies]
tray-item = "0.7"
dirs = "5"
```
