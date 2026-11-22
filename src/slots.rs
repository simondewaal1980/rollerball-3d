//! The three-reel slot machine on the second screen (bell, cherries and
//! aubergine), including its spin animation and pay table.

use bevy::prelude::*;

use crate::art::Art;
use crate::audio::{SoundEvent, SoundQueue};
use crate::game::Game;

pub const SLOT_SYMBOL_BELL: usize = 0;
pub const SLOT_SYMBOL_CHERRY: usize = 1;
pub const SLOT_SYMBOL_EGGPLANT: usize = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SlotOutcome {
    Jackpot,
    Pair,
    Nothing,
}

#[derive(Clone, Debug)]
pub struct SlotMachine {
    pub symbols: [usize; 3],
    pub spinning: [bool; 3],
    pub spin_time: f32,
    pub stop_at: [f32; 3],
    pub cycle_timer: f32,
    pub active: bool,
    pub cooldown: f32,
    pub rng: u32,
}

impl Default for SlotMachine {
    fn default() -> Self {
        Self {
            symbols: [SLOT_SYMBOL_BELL, SLOT_SYMBOL_CHERRY, SLOT_SYMBOL_EGGPLANT],
            spinning: [false; 3],
            spin_time: 0.0,
            stop_at: [0.85, 1.20, 1.55],
            cycle_timer: 0.0,
            active: false,
            cooldown: 0.0,
            rng: 0x51ee_7a11,
        }
    }
}

impl SlotMachine {
    pub fn reset_for_ball(&mut self) {
        self.spinning = [false; 3];
        self.active = false;
        self.spin_time = 0.0;
        self.cooldown = 0.8;
    }

    fn random(&mut self) -> usize {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 17;
        self.rng ^= self.rng << 5;
        (self.rng % 3) as usize
    }

    pub fn can_start(&self) -> bool {
        !self.active && self.cooldown <= 0.0
    }

    pub fn begin(&mut self) -> bool {
        if !self.can_start() {
            return false;
        }
        self.active = true;
        self.spinning = [true; 3];
        self.spin_time = 0.0;
        self.cycle_timer = 0.0;
        true
    }

    /// Advance the reels. Returns the result on the frame they all stop.
    pub fn tick(&mut self, dt: f32) -> Option<SlotOutcome> {
        self.cooldown = (self.cooldown - dt).max(0.0);
        if !self.active {
            return None;
        }

        self.spin_time += dt;
        self.cycle_timer += dt;
        if self.cycle_timer >= 0.075 {
            self.cycle_timer = 0.0;
            for i in 0..3 {
                if self.spinning[i] {
                    self.symbols[i] = self.random();
                }
            }
        }

        for i in 0..3 {
            if self.spinning[i] && self.spin_time >= self.stop_at[i] {
                self.spinning[i] = false;
            }
        }

        if self.spinning.iter().all(|v| !*v) {
            self.active = false;
            self.cooldown = 2.5;
            let [a, b, c] = self.symbols;
            if a == b && b == c {
                Some(SlotOutcome::Jackpot)
            } else if a == b || b == c || a == c {
                Some(SlotOutcome::Pair)
            } else {
                Some(SlotOutcome::Nothing)
            }
        } else {
            None
        }
    }
}

impl Game {
    pub fn try_start_slot(&mut self, queue: &mut SoundQueue) {
        if self.slot.begin() {
            self.add_score(crate::game::SLOT_TRIGGER_SCORE, queue);
            self.message("SLOT SPIN!", 1.4);
            queue.push(SoundEvent::SlotSpin);
        }
    }

    pub fn tick_slot(&mut self, dt: f32, queue: &mut SoundQueue) {
        if let Some(outcome) = self.slot.tick(dt) {
            self.resolve_slot(outcome, queue);
        }
    }

    fn resolve_slot(&mut self, outcome: SlotOutcome, queue: &mut SoundQueue) {
        match outcome {
            SlotOutcome::Jackpot => {
                let symbol = self.slot.symbols[0];
                match symbol {
                    SLOT_SYMBOL_BELL => {
                        self.add_score(3_000, queue);
                        let player = self.current_player_mut();
                        player.balls_left = player.balls_left.saturating_add(1);
                        self.message("BELL! EXTRA BALL", 2.2);
                        queue.push(SoundEvent::ExtraBall);
                    }
                    SLOT_SYMBOL_CHERRY => {
                        self.add_score(2_000, queue);
                        self.multiplier = self.multiplier.saturating_add(1);
                        self.bonus = self.bonus.saturating_add(1_000);
                        self.message(format!("CHERRY! BONUS X{}", self.multiplier), 2.2);
                    }
                    _ => {
                        self.add_score(5_000, queue);
                        self.message("AUBERGINE! 5000", 2.2);
                    }
                }
                queue.push(SoundEvent::SlotWin);
            }
            SlotOutcome::Pair => {
                self.add_score(500, queue);
                self.message("PAIR +500", 1.4);
                queue.push(SoundEvent::RolloverLane);
            }
            SlotOutcome::Nothing => {
                self.add_score(100, queue);
                self.message("NO MATCH +100", 1.2);
                queue.push(SoundEvent::RolloverLane);
            }
        }
    }
}

#[derive(Component)]
pub struct SlotReel {
    pub index: usize,
}

/// Keep the three visible reel sprites in sync with the slot state.
pub fn sync_reel_visuals(
    game: Res<Game>,
    art: Res<Art>,
    mut reels: Query<(&SlotReel, &mut Sprite)>,
) {
    let symbols = art.symbols();
    for (reel, mut sprite) in &mut reels {
        let index = reel.index.min(2);
        let symbol = game.slot.symbols[index];
        if sprite.image != symbols[symbol] {
            sprite.image = symbols[symbol].clone();
        }
        sprite.color = if game.slot.spinning[index] {
            Color::srgb(0.75, 0.9, 1.0)
        } else {
            Color::WHITE
        };
    }
}
