use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    fn distance(self, other: Self) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Rect {
    pub const fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct TickInput {
    pub now_ms: u64,
    pub cursor: Point,
    pub work_area: Rect,
    pub window_size: Point,
    pub window_position: Point,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PetKind {
    Cat,
    Puppy,
}

impl PetKind {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "cat" => Some(Self::Cat),
            "puppy" => Some(Self::Puppy),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Auto,
    Follow,
    Play,
    Bark,
    Sleep,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Behavior {
    Idle,
    Walk,
    Run,
    Bark,
    Play,
    Attention,
    Sniff,
    Groom,
    Sleep,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Facing {
    Left,
    Right,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PetSettings {
    pub name: String,
    pub kind: PetKind,
}

impl Default for PetSettings {
    fn default() -> Self {
        Self {
            name: "Mochi".to_string(),
            kind: PetKind::Puppy,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PetSnapshot {
    pub name: String,
    pub kind: PetKind,
    pub mode: Mode,
    pub behavior: Behavior,
    pub facing: Facing,
    pub frame: u8,
    pub position: Point,
    pub next_tick_ms: u64,
}

pub struct PetEngine {
    settings: PetSettings,
    mode: Mode,
    behavior: Behavior,
    facing: Facing,
    last_tick_ms: Option<u64>,
    last_cursor: Option<Point>,
    last_input_ms: u64,
    behavior_started_ms: u64,
    velocity: Point,
}

impl PetEngine {
    pub fn new(settings: PetSettings) -> Self {
        Self {
            settings,
            mode: Mode::Auto,
            behavior: Behavior::Idle,
            facing: Facing::Right,
            last_tick_ms: None,
            last_cursor: None,
            last_input_ms: 0,
            behavior_started_ms: 0,
            velocity: Point::new(0.0, 0.0),
        }
    }

    pub fn settings(&self) -> PetSettings {
        self.settings.clone()
    }

    pub fn set_settings(&mut self, settings: PetSettings) {
        self.settings = settings;
    }

    pub fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
        if mode != Mode::Sleep {
            self.last_input_ms = self.last_tick_ms.unwrap_or(0);
        }
    }

    pub fn tick(&mut self, input: TickInput) -> PetSnapshot {
        let dt = self
            .last_tick_ms
            .map(|previous| input.now_ms.saturating_sub(previous) as f64 / 1000.0)
            .unwrap_or(0.05)
            .clamp(0.01, 1.0);
        self.last_tick_ms = Some(input.now_ms);

        if self.mode == Mode::Bark
            && self.behavior == Behavior::Bark
            && input.now_ms.saturating_sub(self.behavior_started_ms) >= 900
        {
            self.mode = Mode::Auto;
            self.last_input_ms = input.now_ms;
        }

        let cursor_moved = self
            .last_cursor
            .is_none_or(|previous| previous.distance(input.cursor) > 1.0);
        if cursor_moved {
            self.last_input_ms = input.now_ms;
        }
        self.last_cursor = Some(input.cursor);

        let idle_ms = input.now_ms.saturating_sub(self.last_input_ms);
        let effective_mode = match self.mode {
            Mode::Auto if idle_ms >= 45_000 => Mode::Sleep,
            Mode::Auto if idle_ms <= 2_500 => Mode::Follow,
            mode => mode,
        };

        let window_center = Point::new(
            input.window_position.x + input.window_size.x / 2.0,
            input.window_position.y + input.window_size.y / 2.0,
        );

        let (target, requested_behavior, speed): (Point, Behavior, f64) = match effective_mode {
            Mode::Sleep => {
                let home = sleep_position(input.work_area, input.window_size);
                let close = input.window_position.distance(home) < 4.0;
                (
                    home,
                    if close {
                        Behavior::Sleep
                    } else {
                        Behavior::Walk
                    },
                    260.0,
                )
            }
            Mode::Play => {
                let phase = input.now_ms as f64 / 420.0;
                let target_center = Point::new(
                    input.cursor.x + phase.sin() * 105.0,
                    input.cursor.y + 55.0 + phase.cos().abs() * 45.0,
                );
                (
                    top_left(target_center, input.window_size),
                    Behavior::Play,
                    match self.settings.kind {
                        PetKind::Cat => 430.0,
                        PetKind::Puppy => 620.0,
                    },
                )
            }
            Mode::Bark => (input.window_position, Behavior::Bark, 0.0),
            Mode::Follow => {
                let stop_distance = match self.settings.kind {
                    PetKind::Cat => 78.0,
                    PetKind::Puppy => 24.0,
                };
                let target_center = follow_target(window_center, input.cursor, stop_distance);
                let distance = window_center.distance(input.cursor);
                let behavior = match self.settings.kind {
                    PetKind::Cat if distance > 280.0 => Behavior::Run,
                    PetKind::Cat => Behavior::Walk,
                    PetKind::Puppy if distance > 80.0 => Behavior::Run,
                    PetKind::Puppy => Behavior::Walk,
                };
                let speed = match behavior {
                    Behavior::Run if self.settings.kind == PetKind::Puppy => 640.0,
                    Behavior::Run => 460.0,
                    Behavior::Walk if self.settings.kind == PetKind::Puppy => 390.0,
                    Behavior::Walk => 270.0,
                    _ => 0.0,
                };
                (top_left(target_center, input.window_size), behavior, speed)
            }
            Mode::Auto => {
                if idle_ms >= 8_000 {
                    let waypoint =
                        wander_position(input.now_ms, input.work_area, input.window_size);
                    (waypoint, Behavior::Walk, 180.0)
                } else {
                    let behavior = match self.settings.kind {
                        PetKind::Cat if idle_ms >= 5_000 => Behavior::Groom,
                        PetKind::Cat => Behavior::Idle,
                        PetKind::Puppy => match (idle_ms / 1_400) % 3 {
                            1 => Behavior::Attention,
                            2 => Behavior::Sniff,
                            _ => Behavior::Idle,
                        },
                    };
                    (input.window_position, behavior, 0.0)
                }
            }
        };

        let target = clamp_position(target, input.work_area, input.window_size);
        let delta = Point::new(
            target.x - input.window_position.x,
            target.y - input.window_position.y,
        );
        let distance = Point::new(0.0, 0.0).distance(delta);
        let desired_speed = speed.min(distance * 5.0);
        let desired_velocity = if distance > 1.0 {
            Point::new(
                delta.x / distance * desired_speed,
                delta.y / distance * desired_speed,
            )
        } else {
            Point::new(0.0, 0.0)
        };
        let acceleration = match requested_behavior {
            Behavior::Run | Behavior::Play => 2_200.0,
            Behavior::Walk => 1_200.0,
            _ => 1_800.0,
        };
        let velocity_step = acceleration * dt;
        self.velocity = Point::new(
            approach(self.velocity.x, desired_velocity.x, velocity_step),
            approach(self.velocity.y, desired_velocity.y, velocity_step),
        );

        let movement = Point::new(self.velocity.x * dt, self.velocity.y * dt);
        let movement_distance = Point::new(0.0, 0.0).distance(movement);
        let position = if distance > 1.0 && movement_distance >= distance {
            self.velocity = Point::new(0.0, 0.0);
            target
        } else {
            constrain_transition(
                input.window_position,
                Point::new(
                    input.window_position.x + movement.x,
                    input.window_position.y + movement.y,
                ),
                input.work_area,
                input.window_size,
            )
        };

        if self.velocity.x.abs() > 1.0 {
            self.facing = if self.velocity.x < 0.0 {
                Facing::Left
            } else {
                Facing::Right
            };
        }

        let next_behavior =
            if distance < 5.0 && matches!(requested_behavior, Behavior::Walk | Behavior::Run) {
                if effective_mode == Mode::Sleep {
                    Behavior::Sleep
                } else if self.settings.kind == PetKind::Puppy {
                    Behavior::Attention
                } else {
                    Behavior::Idle
                }
            } else {
                requested_behavior
            };
        if next_behavior != self.behavior {
            self.behavior = next_behavior;
            self.behavior_started_ms = input.now_ms;
        }

        let next_tick_ms = match self.behavior {
            Behavior::Sleep => 500,
            Behavior::Idle | Behavior::Groom | Behavior::Attention | Behavior::Sniff => 120,
            Behavior::Bark | Behavior::Play => 80,
            Behavior::Walk | Behavior::Run => 16,
        };
        let elapsed = input.now_ms.saturating_sub(self.behavior_started_ms);

        PetSnapshot {
            name: self.settings.name.clone(),
            kind: self.settings.kind,
            mode: self.mode,
            behavior: self.behavior,
            facing: self.facing,
            frame: animation_frame(self.behavior, elapsed),
            position,
            next_tick_ms,
        }
    }
}

fn approach(current: f64, target: f64, max_delta: f64) -> f64 {
    if (target - current).abs() <= max_delta {
        target
    } else if target > current {
        current + max_delta
    } else {
        current - max_delta
    }
}

fn animation_frame(behavior: Behavior, elapsed_ms: u64) -> u8 {
    match behavior {
        Behavior::Run => ((elapsed_ms / 90) % 4) as u8,
        Behavior::Walk => ((elapsed_ms / 140) % 4) as u8,
        Behavior::Bark | Behavior::Play => ((elapsed_ms / 130) % 4) as u8,
        Behavior::Sniff => ((elapsed_ms / 170) % 4) as u8,
        Behavior::Attention | Behavior::Groom => ((elapsed_ms / 220) % 4) as u8,
        Behavior::Idle => ((elapsed_ms / 350) % 4) as u8,
        Behavior::Sleep if elapsed_ms < 880 => (elapsed_ms / 220).min(3) as u8,
        Behavior::Sleep => 2 + (((elapsed_ms - 880) / 650) % 2) as u8,
    }
}

fn top_left(center: Point, size: Point) -> Point {
    Point::new(center.x - size.x / 2.0, center.y - size.y / 2.0)
}

fn follow_target(current: Point, cursor: Point, stop_distance: f64) -> Point {
    let distance = current.distance(cursor);
    if distance <= stop_distance || distance == 0.0 {
        return current;
    }
    let ratio = stop_distance / distance;
    Point::new(
        cursor.x + (current.x - cursor.x) * ratio,
        cursor.y + (current.y - cursor.y) * ratio + 28.0,
    )
}

fn sleep_position(work: Rect, size: Point) -> Point {
    Point::new(
        work.x + work.width - size.x - 18.0,
        work.y + work.height - size.y - 10.0,
    )
}

fn wander_position(now_ms: u64, work: Rect, size: Point) -> Point {
    let inset = 36.0;
    let left = work.x + inset;
    let right = work.x + work.width - size.x - inset;
    let top = work.y + work.height * 0.55;
    let bottom = work.y + work.height - size.y - inset;
    match (now_ms / 4_000) % 4 {
        0 => Point::new(left, bottom),
        1 => Point::new(right, bottom),
        2 => Point::new(right, top),
        _ => Point::new(left, top),
    }
}

fn clamp_position(point: Point, work: Rect, size: Point) -> Point {
    Point::new(
        point.x.clamp(work.x, work.x + work.width - size.x),
        point.y.clamp(work.y, work.y + work.height - size.y),
    )
}

fn constrain_transition(current: Point, next: Point, work: Rect, size: Point) -> Point {
    let current_is_on_destination = current.x >= work.x
        && current.x <= work.x + work.width - size.x
        && current.y >= work.y
        && current.y <= work.y + work.height - size.y;

    if current_is_on_destination {
        clamp_position(next, work, size)
    } else {
        next
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cat_stops_farther_from_cursor_than_puppy() {
        let current = Point::new(0.0, 0.0);
        let cursor = Point::new(300.0, 0.0);
        let cat = follow_target(current, cursor, 78.0);
        let puppy = follow_target(current, cursor, 24.0);

        assert!(cat.distance(cursor) > puppy.distance(cursor));
    }

    #[test]
    fn position_stays_inside_negative_origin_monitor() {
        let work = Rect::new(-1920.0, 24.0, 1920.0, 1056.0);
        let size = Point::new(200.0, 200.0);

        assert_eq!(
            clamp_position(Point::new(-3000.0, 2000.0), work, size),
            Point::new(-1920.0, 880.0)
        );
    }

    #[test]
    fn sleeping_home_is_bottom_right_with_margin() {
        let home = sleep_position(
            Rect::new(0.0, 24.0, 1440.0, 876.0),
            Point::new(200.0, 200.0),
        );

        assert_eq!(home, Point::new(1222.0, 690.0));
    }

    #[test]
    fn walk_animation_wraps_after_four_frames() {
        assert_eq!(animation_frame(Behavior::Walk, 3 * 140), 3);
        assert_eq!(animation_frame(Behavior::Walk, 4 * 140), 0);
    }

    #[test]
    fn run_animation_wraps_after_four_frames() {
        assert_eq!(animation_frame(Behavior::Run, 3 * 90), 3);
        assert_eq!(animation_frame(Behavior::Run, 4 * 90), 0);
    }

    #[test]
    fn cross_monitor_step_is_not_clamped_to_destination_edge() {
        let destination = Rect::new(1920.0, 0.0, 1920.0, 1080.0);
        let step = constrain_transition(
            Point::new(1600.0, 500.0),
            Point::new(1610.0, 500.0),
            destination,
            Point::new(128.0, 128.0),
        );

        assert_eq!(step, Point::new(1610.0, 500.0));
    }

    #[test]
    fn bark_is_one_shot_then_returns_to_auto() {
        let mut engine = PetEngine::new(PetSettings::default());
        engine.set_mode(Mode::Bark);
        let input = |now_ms| TickInput {
            now_ms,
            cursor: Point::new(700.0, 400.0),
            work_area: Rect::new(0.0, 0.0, 1000.0, 800.0),
            window_size: Point::new(128.0, 128.0),
            window_position: Point::new(100.0, 100.0),
        };

        assert_eq!(engine.tick(input(1_000)).behavior, Behavior::Bark);
        let resumed = engine.tick(input(1_901));
        assert_eq!(resumed.mode, Mode::Auto);
        assert_ne!(resumed.behavior, Behavior::Bark);
    }
}
