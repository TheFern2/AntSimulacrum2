// Brood lifecycle: Egg → Larva → Adult (new Ant).

use macroquad::prelude::Vec2;

use crate::ant::{Ant, Caste};

pub const EGG_DURATION: f32 = 60.0;   // seconds as an egg
pub const LARVA_DURATION: f32 = 120.0; // seconds as a larva

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum BroodStage {
    Egg,
    Larva,
}

pub struct BroodMember {
    pub stage: BroodStage,
    pub timer: f32, // seconds remaining in this stage
}

impl BroodMember {
    pub fn new_egg() -> Self {
        Self { stage: BroodStage::Egg, timer: EGG_DURATION }
    }
}

/// Advance all brood timers by `dt`. Returns newly hatched ants.
pub fn advance_brood(brood: &mut Vec<BroodMember>, dt: f32, nest_pos: Vec2) -> Vec<Ant> {
    let mut new_ants = Vec::new();
    let mut i = 0;

    while i < brood.len() {
        brood[i].timer -= dt;
        if brood[i].timer <= 0.0 {
            match brood[i].stage {
                BroodStage::Egg => {
                    // Egg hatches into larva
                    brood[i].stage = BroodStage::Larva;
                    brood[i].timer = LARVA_DURATION;
                    i += 1;
                }
                BroodStage::Larva => {
                    // Larva matures into an adult ant
                    let caste = assign_caste();
                    new_ants.push(Ant::new_with_caste(nest_pos, caste));
                    // swap_remove is O(1) and order doesn't matter
                    brood.swap_remove(i);
                    // don't increment i — the swapped element needs to be checked
                }
            }
        } else {
            i += 1;
        }
    }

    new_ants
}

/// Weighted random caste assignment: Worker 70%, Scout 10%, Soldier 10%, Nurse 10%.
fn assign_caste() -> Caste {
    use ::rand::Rng;
    let roll: f32 = ::rand::thread_rng().r#gen();
    if roll < 0.70      { Caste::Worker  }
    else if roll < 0.80 { Caste::Scout   }
    else if roll < 0.90 { Caste::Soldier }
    else                { Caste::Nurse   }
}
