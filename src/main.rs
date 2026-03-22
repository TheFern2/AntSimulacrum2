use macroquad::prelude::*;

mod ant;
mod brood;
mod camera;
mod colony;
mod ecology;
mod input;
mod pheromone;
mod predator;
mod rendering;
mod ui;
mod weather;
mod world;

use ant::{Ant, Caste, SOLDIER_SCAN_RANGE};
use brood::{advance_brood, BroodMember};
use camera::Camera;
use colony::{Colony, GameMode};
use ecology::Ecology;
use input::{InputState, ToolResult};
use pheromone::{PheromoneGrid, DEFAULT_DECAY_RATE};
use predator::{
    PredatorSystem, SpiderState,
    SPIDER_KILL_RANGE, COMBAT_RANGE, SOLDIER_DAMAGE_PER_SEC, SPIDER_DAMAGE_PER_SEC,
};
use rendering::{draw_debug_overlay, draw_scene};
use ui::{UiAction, UiState, draw_top_bar, draw_bottom_bar, draw_tool_cursor,
         draw_settings_panel, draw_stats_panel, draw_toasts,
         is_ui_hovered};
use weather::Weather;
use world::World;

const GRID_W: usize = 200;
const GRID_H: usize = 150;
const CELL_SIZE: f32 = 10.0;
const INITIAL_ANT_COUNT: usize = 20;
const INITIAL_BATCH_SIZE: usize = 5;
const INITIAL_BATCH_INTERVAL: f32 = 30.0;

/// Damage dealt per second by a soldier to an enemy ant of any caste.
const INTER_COLONY_SOLDIER_DMG: f32 = 15.0;

// Colony nest grid positions and colors
const NEST_GRIDS: [(usize, usize); 3] = [(30, 130), (100, 20), (170, 130)];
const COLONY_COLORS: [Color; 3] = [
    Color { r: 1.0, g: 0.7, b: 0.1, a: 1.0 }, // amber
    Color { r: 0.0, g: 0.9, b: 0.9, a: 1.0 }, // cyan
    Color { r: 0.9, g: 0.2, b: 0.9, a: 1.0 }, // magenta
];

fn window_conf() -> Conf {
    Conf {
        window_title: "AntSimulacrum".to_owned(),
        window_width: 1280,
        window_height: 720,
        ..Default::default()
    }
}

fn nest_world_pos(grid: (usize, usize)) -> Vec2 {
    Vec2::new(
        (grid.0 as f32 + 0.5) * CELL_SIZE,
        (grid.1 as f32 + 0.5) * CELL_SIZE,
    )
}

fn make_ant_batch(nest: Vec2, count: usize, colony_id: usize) -> Vec<Ant> {
    use ::rand::Rng;
    let mut rng = ::rand::thread_rng();
    (0..count)
        .map(|_| {
            let mut ant = Ant::new_with_caste(nest, Caste::Worker, colony_id);
            ant.max_age = rng.gen_range(ant.max_age * 0.67..ant.max_age * 1.67);
            ant
        })
        .collect()
}

// ── Milestone tracker ────────────────────────────────────────────────────────

struct Milestones {
    first_food:    bool,
    pop_50:        bool,
    pop_100:       bool,
    pop_500:       bool,
    pop_1000:      bool,
    collapse_shown: bool,
}

impl Milestones {
    fn new() -> Self {
        Self { first_food: false, pop_50: false, pop_100: false,
               pop_500: false, pop_1000: false, collapse_shown: false }
    }

    fn reset(&mut self) { *self = Self::new(); }

    fn check_and_toast(
        &mut self,
        colony:    &Colony,
        ants:      &[Ant],
        brood:     &[BroodMember],
        ui:        &mut UiState,
        colony_id: usize,
    ) {
        let pop = ants.len() + brood.len();
        let label = |id: usize| format!("C{}", id + 1);
        if !self.first_food && colony.total_food_delivered > 0 {
            self.first_food = true;
            ui.push_toast(&format!("{}: First food delivered!", label(colony_id)));
        }
        if !self.pop_50   && pop >= 50   { self.pop_50   = true; ui.push_toast(&format!("{}: 50 ants!",   label(colony_id))); }
        if !self.pop_100  && pop >= 100  { self.pop_100  = true; ui.push_toast(&format!("{}: 100 ants!",  label(colony_id))); }
        if !self.pop_500  && pop >= 500  { self.pop_500  = true; ui.push_toast(&format!("{}: 500 ants!",  label(colony_id))); }
        if !self.pop_1000 && pop >= 1000 { self.pop_1000 = true; ui.push_toast(&format!("{}: 1000 ants!", label(colony_id))); }
        if !self.collapse_shown && colony.collapsed {
            self.collapse_shown = true;
            ui.push_toast(&format!("{} collapsed!", label(colony_id)));
        }
    }
}

// ── Colony instance ────────────────────────────────────────────────────────

struct ColonyInstance {
    colony:    Colony,
    ants:      Vec<Ant>,
    brood:     Vec<BroodMember>,
    pheromones: PheromoneGrid,
    initial_spawned: usize,
    batch_timer: f32,
    milestones: Milestones,
}

impl ColonyInstance {
    fn new(id: usize, nest_grid: (usize, usize), color: Color, mode: GameMode) -> Self {
        let nest_pos = nest_world_pos(nest_grid);
        let colony   = Colony::new(id, nest_pos, color, mode);
        let ants     = make_ant_batch(nest_pos, INITIAL_BATCH_SIZE, id);
        Self {
            colony,
            ants,
            brood: Vec::new(),
            pheromones: PheromoneGrid::new(GRID_W, GRID_H),
            initial_spawned: INITIAL_BATCH_SIZE,
            batch_timer: INITIAL_BATCH_INTERVAL,
            milestones: Milestones::new(),
        }
    }

    fn reset(&mut self, mode: GameMode) {
        let id       = self.colony.id;
        let nest_pos = self.colony.nest_pos;
        let color    = self.colony.color;
        self.colony    = Colony::new(id, nest_pos, color, mode);
        self.ants      = make_ant_batch(nest_pos, INITIAL_BATCH_SIZE, id);
        self.brood.clear();
        self.pheromones     = PheromoneGrid::new(GRID_W, GRID_H);
        self.initial_spawned = INITIAL_BATCH_SIZE;
        self.batch_timer     = INITIAL_BATCH_INTERVAL;
        self.milestones.reset();
    }
}

// ── Reset helpers ─────────────────────────────────────────────────────────────

fn make_colonies(mode: GameMode) -> Vec<ColonyInstance> {
    NEST_GRIDS.iter().enumerate()
        .map(|(i, &grid)| ColonyInstance::new(i, grid, COLONY_COLORS[i], mode))
        .collect()
}

fn full_reset(
    world: &mut World,
    ecology: &mut Ecology,
    colonies: &mut Vec<ColonyInstance>,
    camera: &mut Camera,
    predators: &mut PredatorSystem,
    weather: &mut Weather,
    mode: GameMode,
) {
    *world    = make_world();
    *ecology  = Ecology::new(world);
    *colonies = make_colonies(mode);
    *camera   = Camera::new(world.world_center());
    predators.reset();
    weather.reset();
}

fn colony_reset(colonies: &mut Vec<ColonyInstance>, predators: &mut PredatorSystem) {
    let mode = colonies[0].colony.mode;
    for ci in colonies.iter_mut() {
        ci.reset(mode);
    }
    predators.reset();
}

fn make_world() -> World {
    World::new(GRID_W, GRID_H, CELL_SIZE)
}

// ── Main loop ────────────────────────────────────────────────────────────────

#[macroquad::main(window_conf)]
async fn main() {
    let mut world    = make_world();
    let mut ecology  = Ecology::new(&mut world);
    let mut camera   = Camera::new(world.world_center());
    let mut colonies = make_colonies(GameMode::Zen);
    let mut decay_rate = DEFAULT_DECAY_RATE;

    let mut predators = PredatorSystem::new();
    let mut weather   = Weather::new();

    let mut input     = InputState::new();
    let mut ui_state  = UiState::new();

    loop {
        let dt  = get_frame_time().min(0.05);
        let shift = is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift);

        // ── Input hotkeys ────────────────────────────────────────────────────
        input.handle_hotkeys();

        // ── Debug controls (Shift+key) ───────────────────────────────────────
        if shift && is_key_pressed(KeyCode::R) {
            let mode = colonies[0].colony.mode;
            full_reset(&mut world, &mut ecology, &mut colonies, &mut camera, &mut predators, &mut weather, mode);
        }
        if shift && is_key_pressed(KeyCode::M) {
            let new_mode = match colonies[0].colony.mode {
                GameMode::Zen    => GameMode::Normal,
                GameMode::Normal => GameMode::Zen,
            };
            for ci in colonies.iter_mut() { ci.colony.mode = new_mode; }
        }
        if shift && is_key_pressed(KeyCode::Up)   { decay_rate = (decay_rate * 1.5).min(5.0);  }
        if shift && is_key_pressed(KeyCode::Down)  { decay_rate = (decay_rate / 1.5).max(0.01); }

        // ── Camera input ─────────────────────────────────────────────────────
        let ui_hover = is_ui_hovered(&input);
        camera.handle_input(world.world_center(), ui_hover, dt);

        // ── Tool mouse input ──────────────────────────────────────────────────
        let nest_positions: Vec<Vec2> = colonies.iter().map(|ci| ci.colony.nest_pos).collect();
        if !ui_hover {
            let mp = Vec2::from(mouse_position());
            if is_mouse_button_pressed(MouseButton::Left) {
                let result = input.apply_tool_click(mp, &camera, &mut world, &nest_positions, &mut ecology);
                if let ToolResult::DropAnts(world_pos) = result {
                    // Drop ants into the closest colony
                    let closest = colonies.iter()
                        .enumerate()
                        .min_by(|(_, a), (_, b)| {
                            a.colony.nest_pos.distance(world_pos)
                                .partial_cmp(&b.colony.nest_pos.distance(world_pos))
                                .unwrap_or(std::cmp::Ordering::Equal)
                        })
                        .map(|(i, _)| i)
                        .unwrap_or(0);
                    let id = colonies[closest].colony.id;
                    for pos in InputState::drop_ant_positions(world_pos) {
                        colonies[closest].ants.push(Ant::new_with_caste(pos, Caste::Worker, id));
                    }
                }
            }
            if is_mouse_button_down(MouseButton::Left) {
                input.apply_tool_hold(mp, &camera, &mut world);
            }
        }

        // ── Scaled simulation time ────────────────────────────────────────────
        let dt_sim = dt * input.speed_multiplier();

        // ── Staggered initial spawning (per colony) ───────────────────────────
        if dt_sim > 0.0 {
            for ci in colonies.iter_mut() {
                if ci.initial_spawned < INITIAL_ANT_COUNT {
                    ci.batch_timer -= dt_sim;
                    if ci.batch_timer <= 0.0 {
                        ci.batch_timer = INITIAL_BATCH_INTERVAL;
                        let remaining = INITIAL_ANT_COUNT - ci.initial_spawned;
                        let batch = remaining.min(INITIAL_BATCH_SIZE);
                        let id = ci.colony.id;
                        let nest_pos = ci.colony.nest_pos;
                        ci.ants.extend(make_ant_batch(nest_pos, batch, id));
                        ci.initial_spawned += batch;
                    }
                }
            }
        }

        // ── Simulate ──────────────────────────────────────────────────────────

        // Weather
        let wx = weather.update(dt_sim);
        if wx.warn_triggered  { ui_state.push_toast("Rain approaching..."); }
        if wx.storm_ended     { ui_state.push_toast("Rain cleared");        }

        ecology.update(dt_sim, &mut world, weather.food_multiplier());

        // Per-colony: queen/food/brood
        for ci in colonies.iter_mut() {
            let new_eggs = ci.colony.update(dt_sim, ci.ants.len(), ci.brood.len());
            for _ in 0..new_eggs { ci.brood.push(BroodMember::new_egg()); }

            let id = ci.colony.id;
            let nest_pos = ci.colony.nest_pos;
            let hatched = advance_brood(&mut ci.brood, dt_sim, nest_pos, id);
            ci.ants.extend(hatched);

            ci.pheromones.decay(dt_sim, decay_rate * weather.decay_multiplier());
        }

        // Predators — combine all ant positions for spider hunting
        let all_ant_positions: Vec<Vec2> = colonies.iter()
            .flat_map(|ci| ci.ants.iter().map(|a| a.position))
            .collect();
        let total_soldiers: usize = colonies.iter()
            .map(|ci| ci.ants.iter().filter(|a| a.caste == Caste::Soldier).count())
            .sum();
        let total_pop: usize = colonies.iter().map(|ci| ci.ants.len()).sum();
        let mode = colonies[0].colony.mode;
        let spider_spawned = predators.update(dt_sim, &world, &all_ant_positions, mode, total_pop, total_soldiers);
        if spider_spawned { ui_state.push_toast("Spider spotted!"); }

        // Set attack targets: spider priority, then enemy ant
        // Pre-compute enemy snapshots (positions only, no borrow conflicts)
        let snapshots: Vec<Vec<Vec2>> = colonies.iter()
            .map(|ci| ci.ants.iter().map(|a| a.position).collect())
            .collect();

        for i in 0..colonies.len() {
            let enemy_positions: Vec<Vec2> = snapshots.iter().enumerate()
                .filter(|(j, _)| *j != i)
                .flat_map(|(_, positions)| positions.iter().copied())
                .collect();

            for ant in colonies[i].ants.iter_mut() {
                if ant.caste == Caste::Soldier {
                    let pos = ant.position;
                    let spider_target = predators.spiders.iter()
                        .filter(|s| !matches!(s.state, SpiderState::Feeding))
                        .filter(|s| s.position.distance(pos) < SOLDIER_SCAN_RANGE)
                        .min_by(|a, b| {
                            a.position.distance(pos)
                                .partial_cmp(&b.position.distance(pos))
                                .unwrap_or(std::cmp::Ordering::Equal)
                        })
                        .map(|s| s.position);

                    let enemy_target = enemy_positions.iter()
                        .filter(|&&ep| ep.distance(pos) < SOLDIER_SCAN_RANGE)
                        .min_by(|&&a, &&b| {
                            a.distance(pos)
                                .partial_cmp(&b.distance(pos))
                                .unwrap_or(std::cmp::Ordering::Equal)
                        })
                        .copied();

                    ant.attack_target = spider_target.or(enemy_target);
                }
            }
        }

        // Update ants (mem::take to satisfy borrow checker)
        let speed_mult = weather.speed_multiplier();
        for ci in colonies.iter_mut() {
            let nest_pos = ci.colony.nest_pos;
            let old_ants = std::mem::take(&mut ci.ants);
            for mut ant in old_ants {
                if ant.update(dt_sim, &mut world, &mut ci.pheromones, &mut ci.colony, nest_pos, speed_mult) {
                    ci.ants.push(ant);
                }
            }
        }

        // Spider contact kills any ant it touches while Hunting
        if mode == GameMode::Normal {
            for spider in predators.spiders.iter_mut() {
                if spider.state != SpiderState::Hunting { continue; }
                'hunt: for ci in colonies.iter_mut() {
                    for idx in 0..ci.ants.len() {
                        if ci.ants[idx].position.distance(spider.position) < SPIDER_KILL_RANGE {
                            let was_soldier = ci.ants[idx].caste == Caste::Soldier;
                            ci.ants.remove(idx);
                            spider.feed();
                            if was_soldier { ui_state.push_toast("Soldier fell in battle!"); }
                            break 'hunt;
                        }
                    }
                }
            }
        }

        // Soldier–spider mutual damage
        for spider in predators.spiders.iter_mut() {
            if matches!(spider.state, SpiderState::Feeding) { continue; }
            for ci in colonies.iter_mut() {
                for ant in ci.ants.iter_mut() {
                    if ant.caste == Caste::Soldier
                        && ant.attack_target.is_some()
                        && ant.position.distance(spider.position) < COMBAT_RANGE
                    {
                        spider.health -= SOLDIER_DAMAGE_PER_SEC * dt_sim;
                        ant.health    -= SPIDER_DAMAGE_PER_SEC  * dt_sim;
                    }
                }
            }
        }

        // Inter-colony combat: soldiers attack enemy ants
        // Collect damage into a buffer to avoid simultaneous borrows
        {
            let mut dmg_buf: Vec<(usize, usize, f32)> = Vec::new(); // (colony_idx, ant_idx, damage)
            for i in 0..colonies.len() {
                for (si, soldier) in colonies[i].ants.iter().enumerate() {
                    if soldier.caste != Caste::Soldier { continue; }
                    if soldier.attack_target.is_none() { continue; }
                    for j in 0..colonies.len() {
                        if j == i { continue; }
                        for (ej, enemy) in colonies[j].ants.iter().enumerate() {
                            if soldier.position.distance(enemy.position) < COMBAT_RANGE {
                                // Soldier damages enemy
                                dmg_buf.push((j, ej, INTER_COLONY_SOLDIER_DMG * dt_sim));
                                // Enemy soldier damages this soldier back
                                if enemy.caste == Caste::Soldier {
                                    dmg_buf.push((i, si, INTER_COLONY_SOLDIER_DMG * dt_sim));
                                }
                            }
                        }
                    }
                }
            }
            for (ci_idx, ant_idx, dmg) in dmg_buf {
                if ant_idx < colonies[ci_idx].ants.len() {
                    colonies[ci_idx].ants[ant_idx].health -= dmg;
                }
            }
        }

        // Remove soldiers killed in combat (health ≤ 0 caught in next update's health check,
        // but do an eager pass here so soldier_count stays accurate this frame)
        let mut soldier_combat_death = false;
        for ci in colonies.iter_mut() {
            ci.ants.retain(|a| {
                if a.health <= 0.0 {
                    if a.caste == Caste::Soldier { soldier_combat_death = true; }
                    false
                } else { true }
            });
        }
        if soldier_combat_death { ui_state.push_toast("Soldier fell in battle!"); }

        // Remove dead spiders
        let killed_spiders = predators.remove_dead();
        if killed_spiders > 0 { ui_state.push_toast("Spider defeated!"); }

        // Zen mode: enforce minimum worker floor per colony
        for ci in colonies.iter_mut() {
            if ci.colony.mode == GameMode::Zen {
                let workers = ci.ants.iter().filter(|a| a.caste == Caste::Worker).count();
                let floor   = Colony::zen_min_workers();
                let id      = ci.colony.id;
                let nest_pos = ci.colony.nest_pos;
                for _ in workers..floor {
                    ci.ants.push(Ant::new_with_caste(nest_pos, Caste::Worker, id));
                }
            }
        }

        // Colony collapse check (Normal mode only)
        for ci in colonies.iter_mut() {
            if !ci.colony.collapsed && ci.colony.check_collapse(ci.ants.len(), ci.brood.len()) {
                ci.colony.collapsed = true;
            }
        }

        // ── Milestone toasts ───────────────────────────────────────────────────
        for ci in colonies.iter_mut() {
            let id = ci.colony.id;
            ci.milestones.check_and_toast(&ci.colony, &ci.ants, &ci.brood, &mut ui_state, id);
        }
        ui_state.update(dt);

        // ── Render world ───────────────────────────────────────────────────────
        // Build colony_data and nest_data slices for the renderer
        let colony_data_owned: Vec<(Color, &PheromoneGrid, &[Ant])> = colonies.iter()
            .map(|ci| (ci.colony.color, &ci.pheromones, ci.ants.as_slice()))
            .collect();
        let nest_data_owned: Vec<(Vec2, Color)> = colonies.iter()
            .map(|ci| (ci.colony.nest_pos, ci.colony.color))
            .collect();

        draw_scene(
            &world, &camera,
            &colony_data_owned,
            &nest_data_owned,
            &ecology,
            &predators.spiders,
            weather.is_raining(),
            input.phero_vis,
            input.show_ant_labels,
        );

        // ── HUD ────────────────────────────────────────────────────────────────
        let colony_bar_data: Vec<(&Colony, &[Ant])> = colonies.iter()
            .map(|ci| (&ci.colony, ci.ants.as_slice()))
            .collect();
        let action = draw_top_bar(&colony_bar_data, &mut input);
        apply_ui_action(action, &mut world, &mut ecology, &mut colonies, &mut camera, &mut predators, &mut weather);

        draw_bottom_bar(&mut input, &ecology, weather.is_raining());
        draw_tool_cursor(&input, &camera, &world);

        if input.settings_open {
            let action = draw_settings_panel(&mut input);
            apply_ui_action(action, &mut world, &mut ecology, &mut colonies, &mut camera, &mut predators, &mut weather);
        }

        if input.stats_open {
            let sel = input.selected_colony.min(colonies.len() - 1);
            let ci  = &colonies[sel];
            draw_stats_panel(&ci.colony, &ci.ants, &ci.brood, &ecology, &camera, ci.colony.nest_pos);
        }

        draw_toasts(&ui_state);

        if input.show_debug {
            let debug_data: Vec<(&Colony, &[Ant], &[BroodMember])> = colonies.iter()
                .map(|ci| (&ci.colony, ci.ants.as_slice(), ci.brood.as_slice()))
                .collect();
            draw_debug_overlay(get_fps() as f32, &debug_data, decay_rate, &ecology);
        }

        next_frame().await;
    }
}

// ── UiAction handler ─────────────────────────────────────────────────────────

fn apply_ui_action(
    action: UiAction,
    world:     &mut World,
    ecology:   &mut Ecology,
    colonies:  &mut Vec<ColonyInstance>,
    camera:    &mut Camera,
    predators: &mut PredatorSystem,
    weather:   &mut Weather,
) {
    match action {
        UiAction::None => {}
        UiAction::ResetColony => {
            colony_reset(colonies, predators);
        }
        UiAction::NewWorld => {
            let mode = colonies[0].colony.mode;
            full_reset(world, ecology, colonies, camera, predators, weather, mode);
        }
        UiAction::SwitchMode(mode) => {
            for ci in colonies.iter_mut() { ci.colony.mode = mode; }
        }
    }
}
