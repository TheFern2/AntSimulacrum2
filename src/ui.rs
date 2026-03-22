// Phase 5 — HUD, panels, toasts.

use macroquad::prelude::*;

use crate::ant::{Ant, Caste};
use crate::brood::{BroodMember, BroodStage};
use crate::camera::Camera;
use crate::colony::{Colony, GameMode};
use crate::ecology::Ecology;
use crate::input::{InputState, SimSpeed, Tool, TOP_BAR_H, BOTTOM_BAR_H};
use crate::pheromone::PheromoneVis;
use crate::world::World;

// ── Colors ──────────────────────────────────────────────────────────────────

const BAR_BG:     Color = Color { r: 0.07, g: 0.05, b: 0.03, a: 0.95 };
const PANEL_BG:   Color = Color { r: 0.09, g: 0.07, b: 0.04, a: 0.97 };
const PANEL_BORDER: Color = Color { r: 0.50, g: 0.40, b: 0.20, a: 0.70 };
const BTN_IDLE:   Color = Color { r: 0.18, g: 0.14, b: 0.08, a: 1.00 };
const BTN_HOVER:  Color = Color { r: 0.28, g: 0.22, b: 0.12, a: 1.00 };
const BTN_ACTIVE: Color = Color { r: 0.55, g: 0.38, b: 0.08, a: 1.00 };
const BTN_BORDER: Color = Color { r: 0.60, g: 0.50, b: 0.30, a: 0.45 };
const TOAST_BG:   Color = Color { r: 0.12, g: 0.10, b: 0.06, a: 0.93 };
const DIM:        Color = Color { r: 0.70, g: 0.65, b: 0.55, a: 1.00 };
const GOLD:       Color = Color { r: 0.90, g: 0.75, b: 0.40, a: 1.00 };
const COLOR_OK:       Color = Color { r: 0.40, g: 0.90, b: 0.40, a: 1.00 };
const COLOR_LOW:      Color = Color { r: 1.00, g: 0.85, b: 0.20, a: 1.00 };
const COLOR_CRITICAL: Color = Color { r: 1.00, g: 0.30, b: 0.20, a: 1.00 };
const COLOR_DEAD:     Color = Color { r: 0.50, g: 0.50, b: 0.50, a: 1.00 };

// ── Toasts ───────────────────────────────────────────────────────────────────

const TOAST_DURATION: f32  = 3.5;
const MAX_TOASTS: usize    = 4;
const TOAST_W: f32         = 240.0;
const TOAST_H: f32         = 40.0;
const TOAST_GAP: f32       = 6.0;

pub struct Toast {
    pub message: String,
    pub timer:   f32,
}

pub struct UiState {
    pub toasts: Vec<Toast>,
}

impl UiState {
    pub fn new() -> Self {
        Self { toasts: Vec::new() }
    }

    pub fn push_toast(&mut self, message: &str) {
        if self.toasts.iter().any(|t| t.message == message) { return; }
        if self.toasts.len() >= MAX_TOASTS { self.toasts.remove(0); }
        self.toasts.push(Toast { message: message.to_string(), timer: TOAST_DURATION });
    }

    pub fn update(&mut self, dt: f32) {
        self.toasts.retain_mut(|t| { t.timer -= dt; t.timer > 0.0 });
    }
}

// ── UiAction (events that require main-loop state changes) ──────────────────

pub enum UiAction {
    None,
    ResetColony,
    NewWorld,
    SwitchMode(GameMode),
}

// ── Button helper ───────────────────────────────────────────────────────────

/// Immediate-mode button. Returns true on left-click this frame.
fn button(x: f32, y: f32, w: f32, h: f32, label: &str, active: bool) -> bool {
    let mp = Vec2::from(mouse_position());
    let hovered = mp.x >= x && mp.x <= x + w && mp.y >= y && mp.y <= y + h;
    let clicked  = hovered && is_mouse_button_pressed(MouseButton::Left);

    let bg = if active { BTN_ACTIVE } else if hovered { BTN_HOVER } else { BTN_IDLE };
    draw_rectangle(x, y, w, h, bg);
    draw_rectangle_lines(x, y, w, h, 1.0, BTN_BORDER);

    let fs: u16 = 17;
    let td = measure_text(label, None, fs, 1.0);
    draw_text(
        label,
        x + (w - td.width) / 2.0,
        y + (h + td.height) / 2.0 - 2.0,
        fs as f32,
        WHITE,
    );

    clicked
}

// ── Top bar ─────────────────────────────────────────────────────────────────

/// `colony_data` — slice of (colony, ants) for each colony in order.
pub fn draw_top_bar(
    colony_data: &[(&Colony, &[Ant])],
    input: &mut InputState,
) -> UiAction {
    let sw = screen_width();
    draw_rectangle(0.0, 0.0, sw, TOP_BAR_H, BAR_BG);
    draw_line(0.0, TOP_BAR_H, sw, TOP_BAR_H, 1.0, Color { r: 0.4, g: 0.3, b: 0.1, a: 0.5 });

    let ty  = TOP_BAR_H * 0.68;
    let fs  = 16.0;
    let mut x = 8.0;

    // Per-colony compact stats
    for (colony, ants) in colony_data {
        // Colored colony dot
        draw_circle(x + 5.0, ty - 5.0, 5.0, colony.color);
        x += 14.0;

        let (q_label, q_color) = match colony.queen.status_label() {
            "OK"       => ("Q:OK",   COLOR_OK),
            "LOW"      => ("Q:LOW",  COLOR_LOW),
            "CRITICAL" => ("Q:CRIT", COLOR_CRITICAL),
            _          => ("Q:DEAD", COLOR_DEAD),
        };

        let line = format!("{} ants  F:{:.0}  {}  |  ",
            ants.len(), colony.food_stored, q_label);
        let line_no_status = format!("{} ants  F:{:.0}  ", ants.len(), colony.food_stored);

        draw_text(&line_no_status, x, ty, fs, WHITE);
        let status_x = x + measure_text(&line_no_status, None, fs as u16, 1.0).width;
        draw_text(q_label, status_x, ty, fs, q_color);
        let sep_x = status_x + measure_text(q_label, None, fs as u16, 1.0).width;
        draw_text("  |  ", sep_x, ty, fs, DIM);

        x += measure_text(&line, None, fs as u16, 1.0).width;
    }

    // Speed buttons (right side)
    let bw = 44.0; let bh = 26.0;
    let by = (TOP_BAR_H - bh) / 2.0;
    let mut bx = sw - 260.0;

    if button(bx, by, bw, bh, "||",  input.sim_speed == SimSpeed::Paused) { input.sim_speed = SimSpeed::Paused; } bx += bw + 4.0;
    if button(bx, by, bw, bh, "1x",  input.sim_speed == SimSpeed::Normal) { input.sim_speed = SimSpeed::Normal; } bx += bw + 4.0;
    if button(bx, by, bw, bh, "2x",  input.sim_speed == SimSpeed::Fast)   { input.sim_speed = SimSpeed::Fast;   } bx += bw + 4.0;
    if button(bx, by, bw, bh, "Max", input.sim_speed == SimSpeed::Max)    { input.sim_speed = SimSpeed::Max;    } bx += bw + 8.0;

    // Settings button
    if button(bx, by, bw, bh, "[=]", input.settings_open) {
        input.settings_open = !input.settings_open;
        if input.settings_open { input.stats_open = false; }
    }

    UiAction::None
}

// ── Bottom bar ──────────────────────────────────────────────────────────────

pub fn draw_bottom_bar(input: &mut InputState, ecology: &Ecology, is_raining: bool) {
    let sw = screen_width();
    let sh = screen_height();
    let by = sh - BOTTOM_BAR_H;

    draw_rectangle(0.0, by, sw, BOTTOM_BAR_H, BAR_BG);
    draw_line(0.0, by, sw, by, 1.0, Color { r: 0.4, g: 0.3, b: 0.1, a: 0.5 });

    let bw = 90.0; let bh = 26.0;
    let btn_y = by + (BOTTOM_BAR_H - bh) / 2.0;
    let mut x = 8.0;

    let tools: &[(Tool, &str)] = &[
        (Tool::Observe,   "1:Observe"),
        (Tool::PlaceFood, "2:Food"),
        (Tool::DrawWall,  "3:Wall"),
        (Tool::DropAnts,  "4:Ants"),
        (Tool::Eraser,    "5:Erase"),
    ];

    for (tool, label) in tools {
        if button(x, btn_y, bw, bh, label, input.active_tool == *tool) {
            input.active_tool = *tool;
        }
        x += bw + 6.0;
    }

    // Rain indicator
    if is_raining {
        let rain_color = Color { r: 0.4, g: 0.7, b: 1.0, a: 1.0 };
        draw_text("~RAIN~", x + 8.0, btn_y + bh * 0.72, 17.0, rain_color);
    }

    // Day/night info — right side
    let day_icon = if ecology.is_day() { "day" } else { "night" };
    let mins = (ecology.day_time / 60.0) as u32;
    let secs = (ecology.day_time as u32) % 60;
    let info = format!("Day {}  [{}]  {:02}:{:02}", ecology.day_count + 1, day_icon, mins, secs);
    let td = measure_text(&info, None, 15, 1.0);
    draw_text(&info, sw - td.width - 20.0, by + BOTTOM_BAR_H * 0.68, 17.0, GOLD);
}

// ── Tool cursor ─────────────────────────────────────────────────────────────

pub fn draw_tool_cursor(input: &InputState, camera: &Camera, world: &World) {
    let mp = Vec2::from(mouse_position());
    if InputState::is_over_ui(mp.y) { return; }

    let world_pos = camera.screen_to_world(mp);
    let gx = (world_pos.x / world.cell_size) as i32;
    let gy = (world_pos.y / world.cell_size) as i32;
    let cell_screen = camera.world_to_screen(Vec2::new(
        gx as f32 * world.cell_size,
        gy as f32 * world.cell_size,
    ));
    let cs = world.cell_size * camera.zoom();

    match input.active_tool {
        Tool::Observe => {}

        Tool::PlaceFood => {
            draw_circle_lines(mp.x, mp.y, cs * 1.5, 1.5,
                Color { r: 0.2, g: 0.9, b: 0.2, a: 0.6 });
        }

        Tool::DrawWall => {
            draw_rectangle(cell_screen.x, cell_screen.y, cs, cs,
                Color { r: 0.45, g: 0.45, b: 0.45, a: 0.55 });
            draw_rectangle_lines(cell_screen.x, cell_screen.y, cs, cs, 1.2,
                Color { r: 0.8, g: 0.8, b: 0.8, a: 0.7 });
        }

        Tool::DropAnts => {
            for (ox, oy) in [(-5.0f32, -4.0), (1.0, 0.0), (5.0, -4.0), (-2.0, 5.0), (4.0, 5.0)] {
                draw_circle(mp.x + ox, mp.y + oy, 2.0,
                    Color { r: 1.0, g: 0.7, b: 0.1, a: 0.75 });
            }
        }

        Tool::Eraser => {
            draw_circle_lines(mp.x, mp.y, cs, 1.5,
                Color { r: 1.0, g: 0.3, b: 0.3, a: 0.65 });
            let d = cs * 0.65;
            draw_line(mp.x - d, mp.y - d, mp.x + d, mp.y + d, 1.5,
                Color { r: 1.0, g: 0.3, b: 0.3, a: 0.65 });
        }
    }
}

// ── Settings panel ──────────────────────────────────────────────────────────

pub fn draw_settings_panel(input: &mut InputState) -> UiAction {
    if !input.settings_open { return UiAction::None; }

    let sw = screen_width();
    let pw = 340.0; let ph = 330.0;
    let px = (sw - pw) / 2.0;
    let py = TOP_BAR_H + 10.0;

    draw_rectangle(px, py, pw, ph, PANEL_BG);
    draw_rectangle_lines(px, py, pw, ph, 1.5, PANEL_BORDER);

    // Header
    draw_text("Settings", px + 12.0, py + 22.0, 18.0, GOLD);
    if button(px + pw - 30.0, py + 6.0, 22.0, 20.0, "X", false) {
        input.settings_open = false;
        return UiAction::None;
    }
    draw_line(px, py + 30.0, px + pw, py + 30.0, 1.0,
        Color { r: 0.4, g: 0.3, b: 0.1, a: 0.4 });

    let mut ry   = py + 48.0;
    let row_gap  = 38.0;
    let lx       = px + 12.0;
    let btn_x    = px + 140.0;
    let bw       = 78.0; let bh = 24.0;
    let mut action = UiAction::None;

    // Mode
    draw_text("Mode", lx, ry + 16.0, 17.0, DIM);
    if button(btn_x,          ry, bw, bh, "ZEN",    false) { action = UiAction::SwitchMode(GameMode::Zen);    }
    if button(btn_x + bw + 6.0, ry, bw, bh, "NORMAL", false) { action = UiAction::SwitchMode(GameMode::Normal); }
    ry += row_gap;

    // Pheromone layer
    draw_text("Pheromones", lx, ry + 16.0, 17.0, DIM);
    let phero_opts: &[(PheromoneVis, &str)] = &[
        (PheromoneVis::Both,   "Both"),
        (PheromoneVis::ToFood, "Food"),
        (PheromoneVis::ToHome, "Home"),
        (PheromoneVis::Off,    "Off"),
    ];
    let pbw = 42.0;
    for (i, (vis, label)) in phero_opts.iter().enumerate() {
        let bx = btn_x + i as f32 * (pbw + 3.0);
        if button(bx, ry, pbw, bh, label, input.phero_vis == *vis) {
            input.phero_vis = *vis;
        }
    }
    ry += row_gap;

    // Ant labels
    draw_text("Ant Labels", lx, ry + 16.0, 17.0, DIM);
    if button(btn_x,          ry, bw, bh, "ON",  input.show_ant_labels)  { input.show_ant_labels = true;  }
    if button(btn_x + bw + 6.0, ry, bw, bh, "OFF", !input.show_ant_labels) { input.show_ant_labels = false; }
    ry += row_gap;

    // Speed (label + current)
    draw_text("Sim Speed", lx, ry + 16.0, 17.0, DIM);
    let spd_label = match input.sim_speed {
        SimSpeed::Paused => "Paused", SimSpeed::Normal => "1x",
        SimSpeed::Fast   => "2x",     SimSpeed::Max    => "Max",
    };
    draw_text(spd_label, btn_x, ry + 16.0, 17.0, WHITE);
    ry += row_gap;

    // Divider + reset buttons
    draw_line(px + 10.0, ry, px + pw - 10.0, ry, 1.0,
        Color { r: 0.4, g: 0.3, b: 0.1, a: 0.4 });
    ry += 12.0;
    let big_bw = (pw - 36.0) / 2.0;
    if button(px + 12.0,                  ry, big_bw, bh + 4.0, "Reset Colonies", false) {
        input.settings_open = false;
        action = UiAction::ResetColony;
    }
    if button(px + 12.0 + big_bw + 12.0, ry, big_bw, bh + 4.0, "New World", false) {
        input.settings_open = false;
        action = UiAction::NewWorld;
    }

    action
}

// ── Colony stats panel ──────────────────────────────────────────────────────

pub fn draw_stats_panel(
    colony: &Colony,
    ants:   &[Ant],
    brood:  &[BroodMember],
    ecology: &Ecology,
    camera: &Camera,
    nest_pos: Vec2,
) {
    // Anchor panel to the right of the nest
    let nest_screen = camera.world_to_screen(nest_pos);
    let pw = 290.0; let ph = 390.0;
    let sw = screen_width(); let sh = screen_height();

    let mut px = nest_screen.x + 60.0;
    if px + pw > sw - 8.0 { px = nest_screen.x - pw - 60.0; }
    px = px.clamp(5.0, sw - pw - 5.0);

    let mut py = nest_screen.y - ph / 2.0;
    py = py.clamp(TOP_BAR_H + 5.0, sh - BOTTOM_BAR_H - ph - 5.0);

    draw_rectangle(px, py, pw, ph, PANEL_BG);
    draw_rectangle_lines(px, py, pw, ph, 1.5, PANEL_BORDER);

    // Colony color swatch in header
    draw_circle(px + 18.0, py + 16.0, 7.0, colony.color);

    // Header
    let mins = (ecology.day_time / 60.0) as u32;
    let secs = (ecology.day_time as u32) % 60;
    draw_text(
        &format!("Colony {} — Day {}, {:02}:{:02}", colony.id + 1, ecology.day_count + 1, mins, secs),
        px + 30.0, py + 22.0, 18.0, GOLD,
    );
    draw_line(px, py + 28.0, px + pw, py + 28.0, 1.0,
        Color { r: 0.4, g: 0.3, b: 0.1, a: 0.4 });

    let mut ry = py + 48.0;
    let lh  = 22.0;
    let c2  = px + 180.0;
    let fs  = 16.0;

    // Population
    let workers  = ants.iter().filter(|a| a.caste == Caste::Worker).count();
    let scouts   = ants.iter().filter(|a| a.caste == Caste::Scout).count();
    let soldiers = ants.iter().filter(|a| a.caste == Caste::Soldier).count();
    let nurses   = ants.iter().filter(|a| a.caste == Caste::Nurse).count();
    let total    = ants.len();
    let pct = |n: usize| if total > 0 { n * 100 / total } else { 0 };

    let rows: &[(&str, String)] = &[
        ("Population",  total.to_string()),
        ("  Workers",   format!("{} ({}%)", workers,  pct(workers))),
        ("  Scouts",    format!("{} ({}%)", scouts,   pct(scouts))),
        ("  Soldiers",  format!("{} ({}%)", soldiers, pct(soldiers))),
        ("  Nurses",    format!("{} ({}%)", nurses,   pct(nurses))),
        ("  Queen",     format!("1  {}", colony.queen.status_label())),
    ];
    for (label, val) in rows {
        draw_text(label, px + 10.0, ry, fs, DIM);
        draw_text(val,   c2,        ry, fs, WHITE);
        ry += lh;
    }
    ry += 4.0;

    // Brood
    let eggs   = brood.iter().filter(|b| b.stage == BroodStage::Egg).count();
    let larvae = brood.iter().filter(|b| b.stage == BroodStage::Larva).count();
    draw_text("Brood",    px + 10.0, ry, fs, DIM); ry += lh;
    draw_text("  Eggs",   px + 10.0, ry, fs, DIM); draw_text(&eggs.to_string(),   c2, ry, fs, WHITE); ry += lh;
    draw_text("  Larvae", px + 10.0, ry, fs, DIM); draw_text(&larvae.to_string(), c2, ry, fs, WHITE); ry += lh + 4.0;

    // Food
    draw_text("Food",      px + 10.0, ry, fs, DIM); ry += lh;
    draw_text("  Stored",  px + 10.0, ry, fs, DIM); draw_text(&format!("{:.0}", colony.food_stored),         c2, ry, fs, WHITE); ry += lh;
    draw_text("  Gathered",px + 10.0, ry, fs, DIM); draw_text(&colony.total_food_delivered.to_string(),       c2, ry, fs, WHITE); ry += lh + 6.0;

    // Summary
    draw_line(px + 8.0, ry, px + pw - 8.0, ry, 1.0, Color { r: 0.4, g: 0.3, b: 0.1, a: 0.3 }); ry += 10.0;
    draw_text("Peak pop",   px + 10.0, ry, fs, DIM); draw_text(&colony.peak_population.to_string(), c2, ry, fs, WHITE); ry += lh;
    let age = colony.colony_age as u32;
    let age_str = format!("{}d {}h {}m", age / 86400, (age / 3600) % 24, (age / 60) % 60);
    draw_text("Colony age", px + 10.0, ry, fs, DIM); draw_text(&age_str, c2, ry, fs, WHITE);
}

// ── Toasts ──────────────────────────────────────────────────────────────────

pub fn draw_toasts(ui_state: &UiState) {
    let sw = screen_width(); let sh = screen_height();
    for (i, toast) in ui_state.toasts.iter().enumerate().rev() {
        let alpha = (toast.timer * 3.0).min(1.0).min(toast.timer / TOAST_DURATION * 3.0);
        let ty = sh - BOTTOM_BAR_H - 10.0 - (i as f32 + 1.0) * (TOAST_H + TOAST_GAP);
        let tx = sw - TOAST_W - 12.0;

        draw_rectangle(tx, ty, TOAST_W, TOAST_H,
            Color { a: TOAST_BG.a * alpha, ..TOAST_BG });
        draw_rectangle_lines(tx, ty, TOAST_W, TOAST_H, 1.0,
            Color { r: 0.6, g: 0.5, b: 0.3, a: 0.5 * alpha });

        let label = format!("* {}", toast.message);
        let td = measure_text(&label, None, 17, 1.0);
        draw_text(
            &label,
            tx + (TOAST_W - td.width) / 2.0,
            ty + TOAST_H * 0.66,
            17.0,
            Color { r: 1.0, g: 0.9, b: 0.7, a: alpha },
        );
    }
}

// ── is_ui_hovered ───────────────────────────────────────────────────────────

/// Returns true if the mouse is over a HUD bar or an open panel.
pub fn is_ui_hovered(input: &InputState) -> bool {
    let mp = Vec2::from(mouse_position());

    if mp.y < TOP_BAR_H || mp.y > screen_height() - BOTTOM_BAR_H {
        return true;
    }

    if input.settings_open {
        let sw = screen_width();
        let pw = 340.0; let ph = 330.0;
        let px = (sw - pw) / 2.0; let py = TOP_BAR_H + 10.0;
        if mp.x >= px && mp.x <= px + pw && mp.y >= py && mp.y <= py + ph {
            return true;
        }
    }

    false
}
