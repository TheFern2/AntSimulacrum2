# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

AntSimulacrum2 is a 2D ant colony simulation game built with **Rust** and **macroquad**. It targets Desktop (Windows, macOS, Linux) and Web (WASM). The project is currently in the design/planning phase — no Rust source code exists yet.

## Commands

Once the Rust project is initialized:

```sh
cargo run                                                          # run debug build
cargo build --release                                             # optimized desktop build
cargo build --target wasm32-unknown-unknown --release             # web/WASM build
cargo clippy                                                      # lint
cargo fmt                                                         # format
cargo test                                                        # run tests
```

## Architecture

The simulation is organized around **6 development phases** (see `docs/phases.md`). Phase 2 (pheromone trails) is the critical go/no-go gate — the core emergent behavior must look satisfying before proceeding.

### Planned module layout (`src/`)

| Module | Responsibility |
|--------|---------------|
| `main.rs` | Game loop, window init, event wiring |
| `world.rs` | Grid of cells (`Empty`, `Wall`, `Food`), spatial helpers |
| `camera.rs` | Pan/zoom transform |
| `rendering.rs` | All draw calls, pheromone overlay, HUD |
| `ant.rs` | `Ant` struct, state machine (`Foraging` / `Returning`) |
| `pheromone.rs` | `PheromoneGrid` (two f32 grids: `ToHome`, `ToFood`), deposit, decay, directional sampling |
| `colony.rs` | `Colony`, `Queen`, food economy, caste counts |
| `brood.rs` | Egg→Larva→Adult lifecycle, caste assignment |
| `ecology.rs` | `FoodSource`, spawning, regrowth, day/night cycle |
| `ui.rs` | HUD, panels, milestone toasts |
| `input.rs` | Mouse/keyboard handling, tool state |
| `persistence.rs` | Save/load via `bincode` (desktop) or `localStorage` (web) |

### Core design pillars

1. **Emergent Order** — All colony behavior arises from simple per-ant rules; no central AI.
2. **Living World** — Food grows, decays, and cycles; the world is interesting without player input.
3. **Gentle Agency** — Player is observer with god-hand powers (place food, draw walls, drop ants).

### Key mechanics

- **Pheromones:** Each ant deposits `ToHome` or `ToFood` markers as it walks. Markers decay over time. Ants steer by sampling 32 directions in a 90° forward cone, biasing toward the stronger trail. A *liberty coefficient* gives ants a random chance to ignore trails and explore, preventing over-saturation.
- **Castes:** Worker (70%), Scout (10%), Soldier (10%), Nurse (10%), Queen (1). Assigned at larva→adult transition.
- **Game modes:** *Zen* (immortal colony, abundant food) and *Normal* (realistic starvation, colony collapse possible).
- **Platform split:** Desktop uses `tray-item` (system tray) and `dirs` (save paths); Web uses `localStorage` / IndexedDB.

## Key documents

- `docs/gdd.md` — Full game design document
- `docs/phases.md` — Phased development roadmap with implementation details
- `docs/gui-design.md` — UI/HUD layout with ASCII mockups
