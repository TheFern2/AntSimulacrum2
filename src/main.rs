use macroquad::prelude::*;

mod ant;
mod brood;
mod camera;
mod colony;
mod ecology;
mod input;
mod pheromone;
mod rendering;
mod ui;
mod world;

use ant::{Ant, Caste};
use brood::{advance_brood, BroodMember};
use camera::Camera;
use colony::{Colony, GameMode};
use ecology::Ecology;
use input::InputState;
use pheromone::{PheromoneGrid, DEFAULT_DECAY_RATE};
use rendering::{draw_debug_overlay, draw_scene};
use ui::{UiAction, UiState, draw_top_bar, draw_bottom_bar, draw_tool_cursor,
         draw_settings_panel, draw_stats_panel, draw_toasts, draw_collapse_screen,
         is_ui_hovered};
use world::World;

const GRID_W: usize = 80;
const GRID_H: usize = 60;
const CELL_SIZE: f32 = 12.0;
const INITIAL_ANT_COUNT: usize = 50;
const INITIAL_BATCH_SIZE: usize = 10;
const INITIAL_BATCH_INTERVAL: f32 = 30.0;

fn window_conf() -> Conf {
    Conf {
        window_title: "AntSimulacrum".to_owned(),
        window_width: 1280,
        window_height: 720,
        ..Default::default()
    }
}

fn make_world() -> World {
    World::new(GRID_W, GRID_H, CELL_SIZE)
}

fn make_ant_batch(nest: Vec2, count: usize) -> Vec<Ant> {
    use ::rand::Rng;
    let mut rng = ::rand::thread_rng();
    (0..count)
        .map(|_| {
            let mut ant = Ant::new_with_caste(nest, Caste::Worker);
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
        colony: &Colony,
        ants:   &[Ant],
        brood:  &[BroodMember],
        ui:     &mut UiState,
    ) {
        let pop = ants.len() + brood.len();
        if !self.first_food && colony.total_food_delivered > 0 {
            self.first_food = true; ui.push_toast("First food delivered!");
        }
        if !self.pop_50   && pop >= 50   { self.pop_50   = true; ui.push_toast("Colony reached 50!"); }
        if !self.pop_100  && pop >= 100  { self.pop_100  = true; ui.push_toast("Colony reached 100!"); }
        if !self.pop_500  && pop >= 500  { self.pop_500  = true; ui.push_toast("Colony reached 500!"); }
        if !self.pop_1000 && pop >= 1000 { self.pop_1000 = true; ui.push_toast("Colony reached 1000!"); }
        if !self.collapse_shown && colony.collapsed {
            self.collapse_shown = true; ui.push_toast("Colony collapsed!");
        }
    }
}

// ── Reset helpers ─────────────────────────────────────────────────────────────

fn full_reset(
    world: &mut World,
    ecology: &mut Ecology,
    pheromones: &mut PheromoneGrid,
    ants: &mut Vec<Ant>,
    brood: &mut Vec<BroodMember>,
    colony: &mut Colony,
    camera: &mut Camera,
    initial_spawned: &mut usize,
    batch_timer: &mut f32,
    milestones: &mut Milestones,
    mode: GameMode,
) {
    *world = make_world();
    *ecology = Ecology::new(world);
    *pheromones = PheromoneGrid::new(GRID_W, GRID_H);
    *ants = make_ant_batch(world.nest_pos, INITIAL_BATCH_SIZE);
    brood.clear();
    *colony = Colony::new(mode);
    *camera = Camera::new(world.nest_pos);
    *initial_spawned = INITIAL_BATCH_SIZE;
    *batch_timer = INITIAL_BATCH_INTERVAL;
    milestones.reset();
}

fn colony_reset(
    pheromones: &mut PheromoneGrid,
    ants: &mut Vec<Ant>,
    brood: &mut Vec<BroodMember>,
    colony: &mut Colony,
    nest_pos: Vec2,
    initial_spawned: &mut usize,
    batch_timer: &mut f32,
    milestones: &mut Milestones,
) {
    *pheromones = PheromoneGrid::new(GRID_W, GRID_H);
    *ants = make_ant_batch(nest_pos, INITIAL_BATCH_SIZE);
    brood.clear();
    *colony = Colony::new(colony.mode);
    *initial_spawned = INITIAL_BATCH_SIZE;
    *batch_timer = INITIAL_BATCH_INTERVAL;
    milestones.reset();
}

// ── Main loop ────────────────────────────────────────────────────────────────

#[macroquad::main(window_conf)]
async fn main() {
    let mut world      = make_world();
    let mut ecology    = Ecology::new(&mut world);
    let mut camera     = Camera::new(world.nest_pos);
    let mut pheromones = PheromoneGrid::new(GRID_W, GRID_H);
    let mut ants       = make_ant_batch(world.nest_pos, INITIAL_BATCH_SIZE);
    let mut brood: Vec<BroodMember> = Vec::new();
    let mut colony     = Colony::new(GameMode::Zen);
    let mut decay_rate = DEFAULT_DECAY_RATE;
    let mut initial_spawned = INITIAL_BATCH_SIZE;
    let mut batch_timer     = INITIAL_BATCH_INTERVAL;

    let mut input      = InputState::new();
    let mut ui_state   = UiState::new();
    let mut milestones = Milestones::new();

    loop {
        let dt  = get_frame_time().min(0.05);
        let shift = is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift);

        // ── Input hotkeys ────────────────────────────────────────────────────
        input.handle_hotkeys();

        // ── Debug controls (Shift+key) ───────────────────────────────────────
        if shift && is_key_pressed(KeyCode::R) {
            let mode = colony.mode;
            full_reset(&mut world, &mut ecology, &mut pheromones, &mut ants,
                       &mut brood, &mut colony, &mut camera,
                       &mut initial_spawned, &mut batch_timer, &mut milestones, mode);
        }
        if shift && is_key_pressed(KeyCode::M) {
            colony.mode = match colony.mode {
                GameMode::Zen    => GameMode::Normal,
                GameMode::Normal => GameMode::Zen,
            };
        }
        if shift && is_key_pressed(KeyCode::Up)   { decay_rate = (decay_rate * 1.5).min(5.0);  }
        if shift && is_key_pressed(KeyCode::Down)  { decay_rate = (decay_rate / 1.5).max(0.01); }

        // ── Camera input (skip when UI is hovered or colony collapsed) ───────
        let ui_hover = is_ui_hovered(&input);
        camera.handle_input(world.nest_pos, ui_hover || colony.collapsed);

        // ── Tool mouse input ──────────────────────────────────────────────────
        if !colony.collapsed {
            let mp = Vec2::from(mouse_position());
            if !ui_hover {
                if is_mouse_button_pressed(MouseButton::Left) {
                    input.apply_tool_click(mp, &camera, &mut world, &mut ants, &mut ecology);
                }
                if is_mouse_button_down(MouseButton::Left) {
                    input.apply_tool_hold(mp, &camera, &mut world);
                }
            }
        }

        // ── Scaled simulation time ────────────────────────────────────────────
        let dt_sim = if colony.collapsed { 0.0 } else { dt * input.speed_multiplier() };

        // ── Staggered initial spawning ────────────────────────────────────────
        if initial_spawned < INITIAL_ANT_COUNT && dt_sim > 0.0 {
            batch_timer -= dt_sim;
            if batch_timer <= 0.0 {
                batch_timer = INITIAL_BATCH_INTERVAL;
                let remaining = INITIAL_ANT_COUNT - initial_spawned;
                let batch = remaining.min(INITIAL_BATCH_SIZE);
                ants.extend(make_ant_batch(world.nest_pos, batch));
                initial_spawned += batch;
            }
        }

        // ── Simulate ──────────────────────────────────────────────────────────
        ecology.update(dt_sim, &mut world);

        let new_eggs = colony.update(dt_sim, ants.len(), brood.len());
        for _ in 0..new_eggs { brood.push(BroodMember::new_egg()); }

        let hatched = advance_brood(&mut brood, dt_sim, world.nest_pos);
        ants.extend(hatched);

        pheromones.decay(dt_sim, decay_rate);

        let old_ants = std::mem::take(&mut ants);
        for mut ant in old_ants {
            if ant.update(dt_sim, &mut world, &mut pheromones, &mut colony) {
                ants.push(ant);
            }
        }

        // Zen mode: enforce minimum worker floor
        if colony.mode == GameMode::Zen {
            let workers = ants.iter().filter(|a| a.caste == Caste::Worker).count();
            let floor   = Colony::zen_min_workers();
            for _ in workers..floor {
                ants.push(Ant::new_with_caste(world.nest_pos, Caste::Worker));
            }
        }

        // Colony collapse check (Normal mode only)
        if !colony.collapsed && colony.check_collapse(ants.len(), brood.len()) {
            colony.collapsed = true;
        }

        // ── Milestone toasts ───────────────────────────────────────────────────
        milestones.check_and_toast(&colony, &ants, &brood, &mut ui_state);
        ui_state.update(dt);

        // ── Render world ───────────────────────────────────────────────────────
        draw_scene(&world, &camera, &pheromones, &ants, &ecology,
                   input.phero_vis, input.show_ant_labels);

        // ── HUD ────────────────────────────────────────────────────────────────
        // Top bar (also handles speed button clicks)
        let action = draw_top_bar(&colony, &ants, &mut input);
        apply_ui_action(action, &mut world, &mut ecology, &mut pheromones,
                        &mut ants, &mut brood, &mut colony, &mut camera,
                        &mut initial_spawned, &mut batch_timer, &mut milestones);

        // Bottom bar (tool selector + day info)
        draw_bottom_bar(&mut input, &ecology);

        // Tool cursor
        draw_tool_cursor(&input, &camera, &world);

        // Settings panel
        if input.settings_open {
            let action = draw_settings_panel(&mut input);
            apply_ui_action(action, &mut world, &mut ecology, &mut pheromones,
                            &mut ants, &mut brood, &mut colony, &mut camera,
                            &mut initial_spawned, &mut batch_timer, &mut milestones);
        }

        // Colony stats panel
        if input.stats_open {
            draw_stats_panel(&colony, &ants, &brood, &ecology, &camera, &world);
        }

        // Toasts
        draw_toasts(&ui_state);

        // Collapse overlay
        if colony.collapsed {
            let action = draw_collapse_screen(&colony);
            apply_ui_action(action, &mut world, &mut ecology, &mut pheromones,
                            &mut ants, &mut brood, &mut colony, &mut camera,
                            &mut initial_spawned, &mut batch_timer, &mut milestones);
        }

        // Debug overlay (F1)
        if input.show_debug {
            draw_debug_overlay(get_fps() as f32, &colony, &ants, &brood, decay_rate, &ecology);
        }

        next_frame().await;
    }
}

// ── UiAction handler ─────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
fn apply_ui_action(
    action: UiAction,
    world:   &mut World,
    ecology: &mut Ecology,
    pheromones: &mut PheromoneGrid,
    ants:    &mut Vec<Ant>,
    brood:   &mut Vec<BroodMember>,
    colony:  &mut Colony,
    camera:  &mut Camera,
    initial_spawned: &mut usize,
    batch_timer: &mut f32,
    milestones: &mut Milestones,
) {
    match action {
        UiAction::None => {}
        UiAction::ResetColony => {
            colony_reset(pheromones, ants, brood, colony, world.nest_pos,
                         initial_spawned, batch_timer, milestones);
        }
        UiAction::NewWorld => {
            let mode = colony.mode;
            full_reset(world, ecology, pheromones, ants, brood, colony, camera,
                       initial_spawned, batch_timer, milestones, mode);
        }
        UiAction::SwitchMode(mode) => {
            colony.mode = mode;
        }
    }
}
