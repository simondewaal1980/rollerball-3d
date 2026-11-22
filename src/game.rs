//! Game rules: players, scoring, bonus countdown, tilt and the menu state.
//! Keeping this separate from physics and rendering makes the pinball rules
//! easy to read and tune.

use bevy::prelude::*;

use crate::audio::SoundQueue;

use crate::slots::SlotMachine;

// ---------------------------------------------------------------------------
// Scoring sheet (all values also live in MECHANICS.md / mechanics.json).
// ---------------------------------------------------------------------------
pub const BUMPER_SCORE: u64 = 100;
pub const SLINGSHOT_SCORE: u64 = 50;
pub const LANE_SCORE: u64 = 250;
pub const LANE_BONUS: u64 = 100;
pub const LANE_COMPLETE_SCORE: u64 = 2_500;
pub const LANE_COMPLETE_BONUS: u64 = 1_000;
pub const DROP_SCORE: u64 = 500;
pub const DROP_COMPLETE_SCORE: u64 = 5_000;
pub const DROP_COMPLETE_BONUS: u64 = 1_000;
pub const SPINNER_SCORE: u64 = 100;
pub const KICKBACK_SCORE: u64 = 1_000;
pub const SLOT_TRIGGER_SCORE: u64 = 200;
pub const BONUS_START: u64 = 1_000;
pub const BALLS_PER_GAME: u32 = 3;
pub const MATCH_START_SCORE: u64 = 5_000;

pub const BONUS_TICKS: u32 = 20;
pub const BONUS_TICK_SECS: f32 = 0.075;
pub const START_MULTIPLIER: u32 = 1;

pub const NUDGE_LIMIT: u8 = 3;
pub const NUDGE_WINDOW: f32 = 3.0;
pub const TILT_SECS: f32 = 3.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Menu,
    Playing,
    Paused,
    GameOver,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    /// Ball sitting in the plunger lane.
    Ready,
    /// Ball rolling on the table.
    Play,
    /// End-of-ball bonus countdown.
    Bonus,
    /// No ball in play.
    Over,
}

#[derive(Clone, Debug)]
pub struct Player {
    pub score: u64,
    pub balls_left: u32,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            score: 0,
            balls_left: BALLS_PER_GAME,
        }
    }
}

#[derive(Clone, Debug)]
pub struct BonusCountdown {
    pub remaining: u64,
    pub ticks_left: u32,
    pub timer: f32,
    pub per_tick: u64,
}

#[derive(Resource)]
pub struct Game {
    pub mode: Mode,
    pub phase: Phase,
    pub players: [Player; 2],
    pub num_players: usize,
    pub match_play: bool,
    pub current: usize,
    pub high_score: u64,
    pub bonus: u64,
    pub multiplier: u32,
    pub letters: [[bool; 3]; 4],
    /// 0 = bottom-left kickback, 1 = bottom-right, 2 = top-right.
    pub kickback: [bool; 3],
    pub targets_down: [bool; 3],
    pub target_reset: f32,
    pub nudge_stock: u8,
    pub nudge_window: f32,
    pub tilt: f32,
    pub message: String,
    pub message_timer: f32,
    pub menu_index: usize,
    pub ball_active: bool,
    pub request_ball: bool,
    pub bonus_countdown: Option<BonusCountdown>,
    pub slot: SlotMachine,
    pub spinner_speed: f32,
    pub active_screen: usize,
    pub game_over_winner: Option<usize>,
    pub balls_played: u32,
    pub total_scored: u64,
    pub started: bool,
    /// Used by the scripted screenshot/demo driver so it can plunge as a player.
    pub announced_game_over: bool,
}

impl Default for Game {
    fn default() -> Self {
        Self {
            mode: Mode::Menu,
            phase: Phase::Over,
            players: [Player::default(), Player::default()],
            num_players: 1,
            match_play: false,
            current: 0,
            high_score: load_high_score(),
            bonus: BONUS_START,
            multiplier: START_MULTIPLIER,
            letters: [[false; 3]; 4],
            kickback: [true, true, false],
            targets_down: [false; 3],
            target_reset: 0.0,
            nudge_stock: NUDGE_LIMIT,
            nudge_window: NUDGE_WINDOW,
            tilt: 0.0,
            message: String::new(),
            message_timer: 0.0,
            menu_index: 0,
            ball_active: false,
            request_ball: false,
            bonus_countdown: None,
            slot: SlotMachine::default(),
            spinner_speed: 0.0,
            active_screen: 0,
            game_over_winner: None,
            balls_played: 1,
            total_scored: 0,
            started: false,
            announced_game_over: false,
        }
    }
}

impl Game {
    pub fn start_game(&mut self, players: usize, match_play: bool, queue: &mut SoundQueue) {
        queue.push(crate::audio::SoundEvent::Start);
        self.mode = Mode::Playing;
        self.phase = Phase::Ready;
        self.num_players = players.clamp(1, 2);
        self.match_play = match_play && players == 2;
        self.current = 0;
        self.players = [Player::default(), Player::default()];
        if self.match_play {
            self.players[0].score = MATCH_START_SCORE;
            self.players[1].score = MATCH_START_SCORE;
        } else {
            self.players[0].balls_left = BALLS_PER_GAME;
            self.players[1].balls_left = BALLS_PER_GAME;
        }
        self.bonus = BONUS_START;
        self.multiplier = START_MULTIPLIER;
        self.letters = [[false; 3]; 4];
        self.kickback = [true, true, false];
        self.targets_down = [false; 3];
        self.target_reset = 0.0;
        self.nudge_stock = NUDGE_LIMIT;
        self.nudge_window = NUDGE_WINDOW;
        self.tilt = 0.0;
        self.slot = SlotMachine::default();
        self.spinner_speed = 0.0;
        self.active_screen = 0;
        self.game_over_winner = None;
        self.balls_played = 1;
        self.total_scored = 0;
        self.started = true;
        self.announced_game_over = false;
        self.message = "BALL 1".into();
        self.message_timer = 1.6;
        self.bonus_countdown = None;
        self.begin_ball();
    }

    pub fn begin_ball(&mut self) {
        self.phase = Phase::Ready;
        self.ball_active = false;
        self.request_ball = true;
        self.letters = [[false; 3]; 4];
        self.targets_down = [false; 3];
        self.target_reset = 0.0;
        self.kickback = [true, true, self.kickback[2]];
        self.nudge_stock = NUDGE_LIMIT;
        self.nudge_window = NUDGE_WINDOW;
        self.slot.reset_for_ball();
        self.message = format!("BALL {}", self.balls_played.min(99));
        self.message_timer = 1.4;
    }

    pub fn current_player_mut(&mut self) -> &mut Player {
        &mut self.players[self.current]
    }

    pub fn in_play(&self) -> bool {
        self.mode == Mode::Playing && self.phase == Phase::Play
    }

    pub fn tilted(&self) -> bool {
        self.tilt > 0.0
    }

    pub fn message(&mut self, text: impl Into<String>, secs: f32) {
        self.message = text.into();
        self.message_timer = secs;
    }

    /// Add points. In Match Play the points are removed from the opponent.
    pub fn add_score(&mut self, amount: u64, queue: &mut SoundQueue) {
        if amount == 0 {
            return;
        }
        if self.match_play {
            let other = 1 - self.current;
            let before = self.players[other].score;
            self.players[other].score = before.saturating_sub(amount);
            if self.players[other].score == 0 && before > 0 {
                self.end_game(Some(self.current));
            }
        } else {
            let player = &mut self.players[self.current];
            player.score += amount;
            self.total_scored += amount;
            if player.score > self.high_score {
                self.high_score = player.score;
            }
        }
        if amount >= 1_000 {
            self.message(format!("+{amount}"), 1.1);
        }
        let _ = queue;
    }

    pub fn award_letter(&mut self, screen: usize, letter: usize, queue: &mut SoundQueue) {
        if screen >= 4 || letter >= 3 {
            return;
        }
        if self.letters[screen][letter] {
            return;
        }
        self.letters[screen][letter] = true;
        self.add_score(LANE_SCORE, queue);
        self.bonus = self.bonus.saturating_add(LANE_BONUS);
        queue.push(crate::audio::SoundEvent::RolloverLane);
        if self.letters[screen].iter().all(|v| *v) {
            self.add_score(LANE_COMPLETE_SCORE, queue);
            self.bonus = self.bonus.saturating_add(LANE_COMPLETE_BONUS);
            self.multiplier = self.multiplier.saturating_add(1);
            match screen {
                0 => {
                    self.kickback[0] = true;
                    self.kickback[1] = true;
                }
                3 => self.kickback[2] = true,
                _ => {}
            }
            self.message(
                format!("ABC COMPLETE!  X{}", self.multiplier),
                2.0,
            );
            queue.push(crate::audio::SoundEvent::SlotWin);
        }
    }

    pub fn drop_target_hit(&mut self, index: usize, queue: &mut SoundQueue) {
        if index >= 3 || self.targets_down[index] {
            return;
        }
        self.targets_down[index] = true;
        self.add_score(DROP_SCORE, queue);
        self.bonus = self.bonus.saturating_add(100);
        queue.push(crate::audio::SoundEvent::BumperHit);
        if self.targets_down.iter().all(|v| *v) {
            self.add_score(DROP_COMPLETE_SCORE, queue);
            self.bonus = self.bonus.saturating_add(DROP_COMPLETE_BONUS);
            self.multiplier = self.multiplier.saturating_add(1);
            self.target_reset = 1.6;
            self.message(format!("A B C DROPPED!  X{}", self.multiplier), 2.0);
            queue.push(crate::audio::SoundEvent::SlotWin);
        }
    }

    pub fn reset_drop_targets(&mut self) {
        self.targets_down = [false; 3];
    }

    /// Called by the drain sensor at the bottom of the table.
    pub fn drain(&mut self, queue: &mut SoundQueue) {
        if !self.in_play() {
            return;
        }
        self.phase = Phase::Bonus;
        self.ball_active = false;
        if self.players[self.current].balls_left > 0 {
            self.players[self.current].balls_left -= 1;
        }
        let total = self.bonus.saturating_mul(self.multiplier as u64);
        let per_tick = total / BONUS_TICKS as u64;
        self.bonus_countdown = Some(BonusCountdown {
            remaining: total,
            ticks_left: BONUS_TICKS,
            timer: 0.35,
            per_tick,
        });
        self.message = "BONUS".into();
        self.message_timer = 3.5;
        queue.push(crate::audio::SoundEvent::BallDrain);
    }

    pub fn tick_bonus(&mut self, dt: f32, queue: &mut SoundQueue) {
        let Some(mut countdown) = self.bonus_countdown.take() else {
            return;
        };
        countdown.timer -= dt;
        if countdown.timer <= 0.0 && countdown.ticks_left > 0 {
            let last = countdown.ticks_left == 1;
            let amount = if last {
                countdown.remaining
            } else {
                countdown.per_tick
            };
            countdown.remaining = countdown.remaining.saturating_sub(amount);
            countdown.ticks_left -= 1;
            countdown.timer = BONUS_TICK_SECS;
            self.add_score(amount, queue);
            queue.push(crate::audio::SoundEvent::BonusCountdownTick);
        }
        if countdown.ticks_left == 0 && countdown.remaining > 0 {
            self.add_score(countdown.remaining, queue);
            countdown.remaining = 0;
        }
        if countdown.ticks_left == 0 && countdown.timer <= 0.0 {
            self.after_bonus();
        } else {
            self.bonus_countdown = Some(countdown);
        }
    }

    pub fn after_bonus(&mut self) {
        self.bonus_countdown = None;
        self.bonus = BONUS_START;
        self.phase = Phase::Over;
        if self.mode != Mode::Playing {
            return;
        }
        self.next_turn();
    }

    fn next_turn(&mut self) {
        if self.match_play {
            // A match ends only when a player's score reaches zero.
            if self.players[self.current].balls_left > 0 {
                self.balls_played += 1;
                self.begin_ball();
                return;
            }
            // Both players just keep playing until the score objective is met;
            // give the turn to the other player.
            self.current = 1 - self.current;
            self.balls_played += 1;
            self.begin_ball();
            return;
        }

        if self.players[self.current].balls_left > 0 {
            self.balls_played += 1;
            self.begin_ball();
            return;
        }

        // Move to the other player when they still have balls.
        for _ in 0..self.num_players {
            self.current = (self.current + 1) % self.num_players;
            if self.players[self.current].balls_left > 0 {
                self.multiplier = START_MULTIPLIER;
                self.balls_played = 1;
                self.begin_ball();
                return;
            }
        }
        self.end_game(None);
    }

    pub fn end_game(&mut self, winner: Option<usize>) {
        self.mode = Mode::GameOver;
        self.phase = Phase::Over;
        self.ball_active = false;
        self.bonus_countdown = None;
        self.game_over_winner = winner;
        if self.match_play {
            self.high_score = self.high_score.max(self.players[0].score.max(self.players[1].score));
        } else {
            self.high_score = self.high_score.max(self.players[0].score).max(self.players[1].score);
        }
        save_high_score(self.high_score);
    }

    pub fn to_menu(&mut self) {
        self.mode = Mode::Menu;
        self.phase = Phase::Over;
        self.ball_active = false;
        self.request_ball = false;
        self.bonus_countdown = None;
        self.tilt = 0.0;
        self.message.clear();
        self.message_timer = 0.0;
        self.spinner_speed = 0.0;
    }
}

fn high_score_path() -> std::path::PathBuf {
    std::path::PathBuf::from("highscore.txt")
}

fn load_high_score() -> u64 {
    std::fs::read_to_string(high_score_path())
        .ok()
        .and_then(|text| text.trim().parse::<u64>().ok())
        .unwrap_or(12_000)
}

pub fn save_high_score(score: u64) {
    let _ = std::fs::write(high_score_path(), format!("{score}\n"));
}


/// One system for the non-physics game clock: messages, tilt, slot payouts,
/// drop-target reset, spinner spin-down and the bonus countdown.
pub fn tick_game(
    time: Res<Time>,
    mut game: ResMut<Game>,
    mut queue: ResMut<SoundQueue>,
) {
    if game.mode == Mode::Paused || game.mode == Mode::Menu {
        return;
    }
    let dt = time.delta_secs();
    if game.mode == Mode::GameOver && !game.announced_game_over {
        game.announced_game_over = true;
        queue.push(crate::audio::SoundEvent::GameOver);
    }

    game.message_timer = (game.message_timer - dt).max(0.0);
    game.nudge_window = (game.nudge_window - dt).max(0.0);
    if game.nudge_window <= 0.0 {
        game.nudge_stock = NUDGE_LIMIT;
    }
    if game.tilt > 0.0 {
        game.tilt = (game.tilt - dt).max(0.0);
        if game.tilt == 0.0 && game.mode == Mode::Playing {
            game.message("TILT OVER", 1.2);
        }
    }
    game.spinner_speed = (game.spinner_speed - dt * 18.0).max(0.0);

    if game.target_reset > 0.0 {
        game.target_reset -= dt;
        if game.target_reset <= 0.0 {
            game.target_reset = 0.0;
            game.reset_drop_targets();
        }
    }

    if game.mode == Mode::Playing {
        game.tick_slot(dt, &mut queue);
        if game.phase == Phase::Bonus {
            game.tick_bonus(dt, &mut queue);
        }
    }

    if game.phase == Phase::Bonus {
        // Keep the ball out of play while the bonus is counted.
        game.ball_active = false;
    }
}
