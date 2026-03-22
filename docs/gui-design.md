# AntSimulacrum — GUI Design
*Visual reference for all screens and UI states*

---

## 1. Main Simulation View (Default)

The primary view. HUD is minimal and unobtrusive — the simulation is the focus.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ Ants: 247  Food: 83   Queen: ♛ OK        [ZEN]    ░░ ▶ ▶▶ ▶▶▶  [⚙]        │  <- top bar
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│        ·  ·   · ·                    ·  ·                                  │
│    ·  · ·  ·   ·  · ·  ·   ·   ·  ·  ·   ·   ·  ·  ·   ·                 │
│   · ·   ·  · ·  · ·    ·    · ·     ·  ·   · ·  · ·  ·                   │
│  ·  ·  ·  ·   · · ·    ·   · ·  ·   ·    ·   · ·   ·  ·   ·              │
│      ·   ·  ·    ·  · ·  · ·   ·     ·  ·  ·    ·    ·  ·                │
│    ·  · ·   ·   ·    ·   ·   ·  ·  ·   ·   ·  ·  ·  ·    ·               │
│  ·  ·     ·  ·    ·    ·   ·  ·    ·  ·  ·     ·   ·   ·   ·  ·          │
│       ·  ·   · ·   ·  ·  ·  ·   ·    ·   ·  ·   ·  ·  ·   ·              │  <- pheromone
│  ·  ·  · ·  ·   ·    ·   (@@@@)  ·  ·   ·   ·  ·   ·    ·                │     trails +
│    ·   ·  ·  ·   ·  ·  ( @ ♛ @ )  ·  ·  ·     ·  ·   ·   ·              │     ants (·)
│     ·  ·   ·  ·   ·  ·   (@@@)  ·  ·   ·    ·    ·  ·   ·  ·             │
│   ·  ·   ·    ·  ·   ·    ·  ·   ·  ·   ·  ·   ·    ·    ·   ·           │
│  ·    ·  ·  ·  ·  ·  ·  ·   ·    ·   ·  ·    ·  ·  ·  ·     ·  ·        │
│    ·  ·   ·  ·    ·   ·  ·    ·  ·  ·   · ·  ·   ·  ·    ·               │
│       ·  ·    ·  ·   ·   ·  ·  ·    ·  ·   ·   ·  ·   ·  ·               │
│  ·  ·    ·  ·   ·   ·    ·  ·   ·  (●)   ·  ·    ·  ·    ·   ·           │  <- food (●)
│    ·   ·  ·  ·     ·  ·    ·  ·  ·  ·  ·   ·  ·    ·  ·   ·              │
│       ·  ·   ·  ·   ·  ·     ·   ·    ·  ·   ·   ·    ·  ·   ·           │
│   ·  ·    ·   ·  ·   ·  ·  ·  ·  ·     ·   ·  ·   ·  ·    ·              │
│  ·   ·  ·  ·    ·  ·   ·   ·    ·  (●●)  ·   ·  ·   ·   ·  ·  ·         │
│    ·  ·   ·  ·     ·  ·  ·   ·  ·  ·   ·    ·   ·  ·  ·   ·              │
│       ·   ·   ·  ·   ·   ·  ·    ·  ·   ·  ·   ·  ·    ·   ·             │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│  [1:Observe] [2:Food] [3:Wall] [4:Ants] [5:Erase]    Day 3  ☀  02:14      │  <- bottom bar
└─────────────────────────────────────────────────────────────────────────────┘

  Legend:
  ·       = ant (small, moves)
  (@@@)   = nest (concentric rings, pulses)
  ♛       = queen (visible at zoom-in)
  (●)     = food source (size = quantity)
  ░░░     = pheromone heat overlay (ToHome = cool blue, ToFood = warm amber)
  ████    = wall (player-placed)
```

---

## 2. Top Bar — Detail

```
┌─────────────────────────────────────────────────────────────────────────────┐
│  Ants: 247  Food: 83  Queen: ♛ OK        [ZEN]    ░░ ▶ ▶▶ ▶▶▶   [⚙]      │
│  └──────────┘ └──────┘ └──────────┘      └───┘    └────────────┘  └──┘    │
│   colony info  stores   queen health     mode     speed controls  settings │
└─────────────────────────────────────────────────────────────────────────────┘

Queen status variants:
  ♛ OK        = healthy, laying eggs
  ♛ LOW       = food reserves critical, egg rate reduced
  ♛ CRITICAL  = starvation imminent (Normal mode, text turns red)
  ♛ DEAD      = no more eggs (Normal mode only)

Mode toggle:
  [ZEN]     = zen mode active (green tint)
  [NORMAL]  = normal mode active (amber tint)

Speed controls:
  ░░  = paused (Space)
  ▶   = 1x  (key: 1)
  ▶▶  = 2x  (key: 2)
  ▶▶▶ = max (key: 3)
```

---

## 3. Bottom Bar — Tool Selector

```
┌─────────────────────────────────────────────────────────────────────────────┐
│  [1:Observe*] [2:Food] [3:Wall] [4:Ants] [5:Erase]    Day 3  ☀  02:14     │
└─────────────────────────────────────────────────────────────────────────────┘

Active tool shown with * and highlight box.

Tool cursor previews (shown at mouse position on map):
  Observe  →  default arrow cursor, no overlay
  Food     →  green circle ghost at cursor  ○
  Wall     →  filled grey square at cursor  ■
  Ants     →  small ant-dot cluster at cursor  ···
  Erase    →  dashed circle eraser  ⊘

Day/night indicator:
  ☀  02:14  = daytime, 2min 14sec into current sim-day
  ☾  07:41  = nighttime (darker bg, slower food regrowth)
```

---

## 4. Zoom Levels

### 4a. Zoomed Out (overview, default start)
```
┌────────────────────────────────────────┐
│  · ·  ·   · ·  ·   · · ·  ·  · ·  ·  │
│ ·  · ·  ·   · ·  ·   (@@)  ·  ·  ·   │
│  · ·   ·  ·   · · ·  (@♛@)  · ·  ·   │
│   · ·  · ·  ·   · · · (@@)  · ·  · · │
│  ·  · ·   ·   · ·  ·   ·  ·   (●)  · │
│ ·  ·   · ·  ·   ·   · ·  ·  ·   ·  · │
│  · ·  ·   · ·  ·  (●●)  ·  ·   · ·  │
└────────────────────────────────────────┘
  Ants: small dots (·)
  Nest: tight ring (@@)
  Food: circles scaled to quantity
  Pheromones: faint color wash across cells
```

### 4b. Zoomed In (individual ant detail)
```
┌────────────────────────────────────────┐
│  ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░  │
│  ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░  │  <- pheromone
│  ░░▓▓▓▓▓▓▓▓▓▓▓▓░░░░░░░░░░░░░░░░░░░░  │     overlay
│  ░░▓▓▓▓▓▓▓▓▓▓▓▓░░░░░░░░░░░░░░░░░░░░  │     (strong
│  ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░  │      trail)
│                                        │
│    O>      O>  O>                      │  <- ants (O>)
│       <O       O>   <O                 │     with direction
│    O>    O>       <O                   │
│                                        │
│         ( ●●● )                        │  <- food source
│        (  ●●●  )                       │     (larger, visible
│         ( ●●● )                        │      quantity)
│                                        │
└────────────────────────────────────────┘
  Ants shown as circle + direction tick: O>  <O  Ov  O^
  Pheromone trails: ░ light  ▒ medium  ▓ heavy
  ToHome trail = cool (░▒▓ in blue chars, warm amber in actual game)
  ToFood trail = warm
```

---

## 5. Settings Panel (overlay, toggled with ⚙)

```
┌──────────────────────────────────────┐
│  Settings                        [X] │
├──────────────────────────────────────┤
│                                      │
│  Mode         ( ZEN )  ( NORMAL )    │
│                                      │
│  Sim Speed    [──●────────]  2x      │
│                                      │
│  Pheromones   [ON ]  [OFF]           │
│                                      │
│  Ant Labels   [ON ]  [OFF]           │
│                                      │
│  Pheromone    [ToFood] [ToHome] [Both│
│  Layer        ●                      │
│                                      │
│  ─────────────────────────────────── │
│  [  Reset Colony  ]  [  New World  ] │
│                                      │
└──────────────────────────────────────┘
```

---

## 6. Colony Stats Panel (hover over nest, or click nest)

```
┌──────────────────────────────────────┐
│  Colony — Day 3, 02:14               │
├──────────────────────────────────────┤
│                                      │
│  Population      247  (+12 today)    │
│  ├─ Workers      173  (70%)          │
│  ├─ Scouts        25  (10%)          │
│  ├─ Soldiers      25  (10%)          │
│  ├─ Nurses        23  (10%)          │
│  └─ Queen          1  ♛ OK           │
│                                      │
│  Brood                               │
│  ├─ Eggs          14                 │
│  └─ Larvae         8                 │
│                                      │
│  Food                                │
│  ├─ Stored        83                 │
│  ├─ Gathered     412  (all time)     │
│  └─ Consumption  ~6 / min            │
│                                      │
│  ────────────────────────────────    │
│  Peak population   312  (Day 2)      │
│  Colony age        3d 02h 14m        │
│                                      │
└──────────────────────────────────────┘
```

---

## 7. Milestone Toast Notifications (bottom-right, non-blocking)

```
                                         ┌──────────────────────────┐
                                         │ ✦ First food delivered!  │  <- fades in, 3s
                                         └──────────────────────────┘

                                         ┌──────────────────────────┐
                                         │ ✦ Colony reached 100!    │  <- stacks upward
                                         └──────────────────────────┘
```

Toast stack (multiple simultaneous):
```
                                         ┌──────────────────────────┐
                                         │ ✦ Colony reached 500!    │
                                         └──────────────────────────┘
                                         ┌──────────────────────────┐
                                         │ ✦ Pheromone highway!     │
                                         └──────────────────────────┘
```

---

## 8. Colony Collapsed Screen (Normal mode only)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│                                                                             │
│                          Colony Collapsed                                   │
│                        ─────────────────────                               │
│                                                                             │
│                    The colony survived  3d 07h 42m                         │
│                                                                             │
│                    Peak population         312                              │
│                    Total food gathered    4,821                             │
│                    Cause of collapse      Starvation                        │
│                                                                             │
│                                                                             │
│                   [ Start New Colony ]   [ Switch to Zen ]                 │
│                                                                             │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 9. Pheromone Overlay — Color Scheme Reference

```
  ToFood trail (warm amber → orange)
  ┌──────────────────────────────────────────────┐
  │  none   faint      medium      strong  peak  │
  │  ···    ░░░░░░░    ▒▒▒▒▒▒▒    ▓▓▓▓▓▓▓  ███   │
  │  bg     #2a1a00    #6b3a00    #c47000  #ff9900│
  └──────────────────────────────────────────────┘

  ToHome trail (cool teal → blue)
  ┌──────────────────────────────────────────────┐
  │  none   faint      medium      strong  peak  │
  │  ···    ░░░░░░░    ▒▒▒▒▒▒▒    ▓▓▓▓▓▓▓  ███   │
  │  bg     #001a2a    #003a6b    #0070c4  #0099ff│
  └──────────────────────────────────────────────┘

  Overlap (both trails present = blended):
  ┌──────────────────────────────────────────────┐
  │  ToFood ████  +  ToHome ████  =  ████ green  │
  └──────────────────────────────────────────────┘

  World elements:
  Background   #1a1208   dark soil brown
  Nest         #8B6914   warm gold ring, pulsing glow
  Food (full)  #22c55e   bright green
  Food (low)   #166534   dark green
  Wall         #4a4a4a   grey
  Ant worker   #d97706   amber
  Ant scout    #ffffff   white (faster, visible)
  Ant soldier  #ef4444   red
  Ant nurse    #a78bfa   purple
  Queen        #fbbf24   bright gold
```

---

## 10. World Rendering Layers (draw order, bottom to top)

```
  Layer 6 (top)   UI / HUD overlay
  Layer 5         Milestone toasts
  Layer 4         Ant bodies (circles + direction tick)
  Layer 3         Food sources (glowing circles)
  Layer 2         Walls (filled cells)
  Layer 1         Pheromone heat overlay (colored quads, alpha-blended)
  Layer 0 (bottom) Background (solid dark soil color)
```

---

## 11. Window & Layout Constraints

```
  Minimum window size:   800 × 600
  Default window size:  1280 × 800

  Top bar height:         32px
  Bottom bar height:      32px
  Simulation viewport:   full window minus top/bottom bars

  Panel widths (overlays, not persistent):
    Settings panel:      320px wide, centered
    Colony stats panel:  280px wide, anchored to nest position
    Toast width:         240px, anchored bottom-right
    Toast height:         36px each, 8px gap between
```
