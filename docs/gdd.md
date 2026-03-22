# AntSimulacrum — Game Design Document
*Version 1.0 — Draft — 2026-03-22*

## 1. Overview
- **Concept:** A 2D ant colony simulation where realistic emergent behavior unfolds in a living, self-sustaining world. The player observes a colony grow through pheromone-based foraging, caste systems, and brood cycles — occasionally intervening by placing food, walls, or ants. Two modes offer either a zen screensaver experience or a survival challenge with real stakes.
- **Genre:** Simulation / Sandbox (with emergent systems)
- **Platform:** Desktop (Windows, macOS, Linux) + Web (WASM). Rust + macroquad.
- **Scope:** Solo dev. ~3–4 months to first playable, ~6 months to v1. Phased delivery.
- **Target audience:** Fans of simulations, cellular automata, and digital ant farms. People who enjoyed SimAnt, AntSimulator, or leave Conway's Game of Life running. Players who find beauty in emergent complexity from simple rules.

## 2. Core Design Pillars

1. **Emergent Order** — All colony behavior arises from simple per-ant rules (pheromone deposit/follow, state machines). No central AI directs traffic. This means every feature must be tested bottom-up: "does this rule produce interesting emergent patterns?" trumps "does this look correct top-down?"

2. **Living World** — The environment is not a blank canvas waiting for the player. Food grows, decays, and cycles. The world has its own rhythm. Design decisions should favor ecology simulation over static sandbox — the world should be interesting even with zero player input.

3. **Gentle Agency** — The player is an observer with god-hand powers. They place, nudge, and experiment — but never select an ant and tell it where to go. Every interaction should feel like dropping a pebble in a pond: indirect, with emergent ripple effects.

## 3. Core Gameplay Loop

**Micro loop (10–30s):** Watch ants stream along pheromone highways between nest and food. Notice a trail forming, fading, or shifting. Optionally drop food in a new spot and watch the colony discover and adapt to it.

**Macro loop (session, 10–60 min):** Open the app → colony resumes → observe the current state of foraging, population, and food supply → experiment (place walls to create mazes, drop food clusters, add ants) → watch the colony adapt over minutes → check colony stats (population trend, food reserves) → leave it running or close.

**Meta loop (days/weeks):** Return to see how the colony evolved. In Normal mode: did it survive? Did the queen produce enough workers? Has the ecology shifted? Over days, the colony grows from a handful of ants to a thriving civilization — or collapses if neglected. The long-term draw is the "digital pet" attachment and curiosity about what happened while you were away.

## 4. Player Experience Goals

| Timeframe | Feeling |
|---|---|
| **Minute 1** | "Oh cool, little dots moving around." Immediate visual activity. Ants already wandering from the nest. Low friction — no tutorial needed, just watch. |
| **Minute 10** | "Wait, they're forming trails!" The first pheromone highways become visible. The player drops food somewhere new and watches ants discover it. First spark of curiosity about the rules. |
| **Hour 1** | "This is my colony." The player has experimented with walls and food placement. They understand the pheromone system intuitively. They check population stats. They feel ownership. |
| **Hour 10+** | "I wonder what happened overnight." The colony has grown significantly. The ecology has shifted. New foraging routes exist. In Normal mode, maybe a food crisis caused a population dip. The player feels the colony is *alive*. |

## 5. Mechanics

### 5.1 Core Mechanics

**Pheromone System**
- The world is a grid of cells. Each cell stores marker intensity for two channels: **ToHome** and **ToFood**.
- Ants deposit markers as they walk: returning ants leave ToFood markers, outgoing ants leave ToHome markers.
- Marker intensity decays each tick (`intensity -= decay_rate * dt`), so trails naturally fade.
- Ants follow trails by sampling ~32 random directions in a 90° forward cone, reading marker intensity at each sample point, and choosing the strongest signal.
- Ants slightly degrade trails they follow (`intensity *= 0.99`), preventing oversaturation and naturally favoring shorter paths.
- **Liberty coefficient:** Each ant has a random chance to ignore pheromones and explore freely, preventing the colony from locking into suboptimal routes.

**Ant State Machine**
Each ant operates as a simple finite state machine:
- **Foraging:** Leave nest → follow ToFood markers or wander → find food → pick up food → switch to Returning.
- **Returning:** Follow ToHome markers → reach nest → deposit food → switch to Foraging.
- **Collision:** Ray-cast against walls. On hit, bounce/redirect. Ants near walls deposit weaker markers.

**Colony Lifecycle**
- **Queen** lays eggs at a rate proportional to food reserves. No food = no eggs.
- **Eggs** hatch into **larvae** after ~60 sim-seconds.
- **Larvae** mature into **adults** after ~120 sim-seconds. Larvae consume food (nurses bring it).
- **Adults** are assigned a caste based on colony needs (configurable ratios).
- **Death:** Ants have a lifespan. In Normal mode, starvation accelerates death. If the queen dies, no new ants are born — colony slowly dies out.

### 5.2 Secondary Mechanics

**Ant Castes**

| Caste | Behavior | Ratio (default) |
|---|---|---|
| Worker | Standard foraging loop: find food, bring it home | 70% |
| Scout | Wider exploration cone (180°), higher liberty coefficient, faster movement. Finds new food sources. | 10% |
| Soldier | Patrols colony perimeter. In v1: visual only (combat comes with competing colonies in v2). | 10% |
| Nurse | Stays near brood. Carries food from storage to larvae. | 10% |
| Queen | Stationary at nest center. Lays eggs. One per colony. | 1 |

**Living Ecology**
- Food sources are objects with: position, current quantity, max quantity, regrowth rate.
- Food regrows over time up to its max (like a fruit tree regrowing fruit).
- New food sources spawn naturally at random intervals, with clustering bias (food appears near existing food).
- Food decays if quantity stays at max too long (prevents infinite stockpiling in the world).
- A simple **day/night cycle** (visual dimming + food regrowth happens faster during "day").

**Zen vs. Normal Mode**

| Aspect | Zen Mode | Normal Mode |
|---|---|---|
| Colony death | Disabled — colony always has minimum ants | Enabled — starvation kills, queen can die |
| Food scarcity | Food spawns abundantly, regrows fast | Food is scarce, regrows slowly |
| Ant lifespan | Very long | Realistic (shorter) |
| Purpose | Screensaver, experimentation, relaxation | Challenge, stakes, realistic biology |

### 5.3 Progression Systems

There is no traditional unlock-based progression. Instead, progression is **organic and emergent:**

- **Colony size** grows over time (starting ~20 ants, potentially reaching thousands).
- **Trail network complexity** increases as the colony discovers more food sources.
- **Player knowledge** deepens — understanding which wall configurations create efficient highways, how food placement affects colony behavior.
- **Colony milestones** (displayed but not gating): "First 100 ants," "First pheromone highway," "Colony survived 24 hours" (Normal mode).

## 6. Game World & Setting

- **Setting:** Top-down view of a patch of ground. Abstract — no specific real-world location. The "world" is the ant's scale: food sources are large relative to ants, walls are impassable terrain.
- **Aesthetic:** Dark background (soil/earth tone). Primitive shapes only — no pixel art, no textures. Ants are small colored circles with a direction tick mark. Pheromone trails render as a colored heat-map overlay (warm tones for ToFood, cool tones for ToHome). Food glows green. The nest pulses subtly. The overall feel is **clean, minimal, data-visualization-meets-nature-documentary**.
- **Audio direction (future):** Ambient nature sounds. Subtle audio feedback for colony events (new ant born: soft chime, food discovered: gentle ping). Not in v1 scope.

## 7. Win / Fail Conditions

**Zen Mode:** No win or fail. The simulation runs indefinitely. The colony is immortal (minimum population enforced). Pure sandbox.

**Normal Mode:**
- **Fail:** Colony dies — all ants dead, or queen dead with no remaining brood. The player sees a "Colony Collapsed" summary (peak population, time survived, food gathered). Option to start a new colony or switch to Zen mode.
- **"Win":** There is no explicit win state. Success is measured by colony longevity and peak size. Milestones mark achievement but don't end the game.
- **Reset:** Player can manually reset the world at any time (with confirmation).

## 8. UI / UX Principles

**HUD (always visible, minimal):**
- Top-left: Ant count, food stored, queen status
- Top-right: Speed controls (pause / 1x / 2x / max), Zen/Normal toggle
- Bottom: Active tool indicator

**Tool bar (toggle or hotkeys):**
- `1` — Observe (default, no placement)
- `2` — Place food
- `3` — Draw wall
- `4` — Drop ants
- `5` — Eraser (remove walls/food)

**Camera:**
- Scroll wheel: zoom in/out
- Middle-click drag or right-click drag: pan
- Home key: center on nest

**Accessibility:**
- Pheromone channels use hue AND brightness (not just color) for colorblind accessibility
- All controls have keyboard equivalents
- Pause is always one keypress away (Space)

**Settings (minimal for v1):**
- Simulation speed
- Zen/Normal toggle
- Pheromone visibility toggle
- Ant count display toggle

## 9. Content Outline

This is a systems-driven game, not a content-driven one. "Content" is emergent from the simulation rules. However, the world has structure:

**World elements:**
- **Nest** — colony home base, center of the map. Permanent ToHome pheromone beacon.
- **Food sources** — 5–10 initial sources scattered around the nest. More spawn naturally over time.
- **Walls** — none initially (player-placed only). Could offer preset "map templates" later.
- **Ants** — starting population of ~20 workers + 1 queen.

**Milestone notifications (non-blocking, corner toasts):**
- "Colony founded" (start)
- "First food delivered"
- "Population: 50 / 100 / 500 / 1000"
- "First pheromone highway formed"
- "Colony survived 1 hour / 1 day / 1 week" (Normal mode)
- "Colony collapsed" (Normal mode fail)

## 10. Out of Scope (v1)

- Multiple competing colonies (wars, territory)
- Environmental hazards (rain, flooding, predators, temperature)
- Underground / tunnel digging (z-layers)
- Multiplayer / shared colonies
- Audio / sound effects
- Map editor / custom starting layouts
- Ant genetics / evolution
- Achievements / unlockables
- Mobile platform support

## 11. Open Design Questions

1. **Simulation fidelity at scale:** Can macroquad handle 5,000+ ants at 60fps with pheromone grid rendering? Need to prototype early. Batched circle rendering + grid-based pheromone overlay (not per-pixel) should work, but must verify.

2. **Pheromone grid resolution:** What cell size balances visual quality vs. performance? AntSimulator used a fixed compile-time grid. We should test 2x2, 4x4, and 8x8 pixel cells.

3. **Caste assignment algorithm:** How does the colony "decide" what caste a new adult becomes? Options: fixed ratio, need-based (fewer scouts → more scouts born), random weighted. Need-based is more realistic but harder to tune.

4. **Day/night cycle pacing:** How long is a sim-day? Too fast feels arcade-y, too slow is invisible. Likely 5–10 real minutes per cycle — needs playtesting.

5. **Save file format:** Bincode (fast, compact, not human-readable) vs. JSON (slower, larger, debuggable). Likely bincode for production, JSON for dev/debug.

6. **WASM persistence:** Web version can't use filesystem. Need IndexedDB or localStorage for save/load on web. This may require a platform abstraction layer.

7. **System tray UX:** When minimized to tray, should sim run at full speed or throttled? Full speed uses CPU. Throttled saves power but colony grows slower. Configurable?

## 12. Prototype Goals

**First playable prototype (Phase 1+2) should prove:**

1. **Emergent trail formation works and is visually satisfying.** 50+ ants should form visible pheromone highways to food within 30 seconds. If this doesn't look cool, the entire game doesn't work.

2. **Performance is viable.** 500+ ants + pheromone grid rendering at 60fps on desktop. This validates macroquad as the framework choice.

3. **The "ant farm" feeling is there.** Even without castes, lifecycle, or ecology — does watching ants forage with pheromones feel zen and curiosity-inducing? This is the subjective gate.

**Prototype deliverable:** A window showing a nest, scattered food, and ants that wander, discover food, form pheromone trails, and bring food home. Player can place food and walls. No UI, no persistence, no castes — just the core simulation loop.
