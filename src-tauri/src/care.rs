use serde::{Deserialize, Serialize};

const HOUR_MS: f64 = 3_600_000.0;
const DEFAULT_FOOD: u8 = 5;
const DEFAULT_TOYS: u8 = 3;
const MAX_FOOD: u8 = 10;
const MAX_TOYS: u8 = 5;

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CareDifficulty {
    Easy,
    #[default]
    Normal,
    Hard,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DecayRates {
    pub hunger_per_hour: f64,
    pub energy_per_hour: f64,
    pub happiness_per_hour: f64,
    pub cleanliness_per_hour: f64,
    pub sleep_energy_per_hour: f64,
}

impl CareDifficulty {
    pub const fn rates(self) -> DecayRates {
        match self {
            Self::Easy => DecayRates {
                hunger_per_hour: 2.0,
                energy_per_hour: 2.0,
                happiness_per_hour: 1.0,
                cleanliness_per_hour: 1.0,
                sleep_energy_per_hour: 20.0,
            },
            Self::Normal => DecayRates {
                hunger_per_hour: 4.0,
                energy_per_hour: 4.0,
                happiness_per_hour: 2.0,
                cleanliness_per_hour: 2.0,
                sleep_energy_per_hour: 25.0,
            },
            Self::Hard => DecayRates {
                hunger_per_hour: 7.0,
                energy_per_hour: 7.0,
                happiness_per_hour: 4.0,
                cleanliness_per_hour: 4.0,
                sleep_energy_per_hour: 30.0,
            },
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CareAction {
    Feed,
    Play,
    Wash,
    Pet,
    Sleep,
    Wake,
    Rest,
    Restock,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CareActionResult {
    Applied,
    Disabled,
    OutOfStock,
}

/// `hunger` is a need: zero means full and 100 means very hungry. The other
/// values are wellbeing scores where 100 is best.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct CareState {
    pub hunger: u8,
    pub energy: u8,
    pub happiness: u8,
    pub cleanliness: u8,
    #[serde(default = "default_food")]
    pub food: u8,
    #[serde(default = "default_toys")]
    pub toys: u8,
    pub sleeping: bool,
    pub last_updated_ms: Option<u64>,
    #[serde(default)]
    decay_carry: DecayCarry,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct DecayCarry {
    hunger: f64,
    energy: f64,
    happiness: f64,
    cleanliness: f64,
}

impl Default for CareState {
    fn default() -> Self {
        Self {
            hunger: 0,
            energy: 100,
            happiness: 100,
            cleanliness: 100,
            food: DEFAULT_FOOD,
            toys: DEFAULT_TOYS,
            sleeping: false,
            last_updated_ms: None,
            decay_carry: DecayCarry::default(),
        }
    }
}

impl CareState {
    pub fn new(now_ms: u64) -> Self {
        Self {
            last_updated_ms: Some(now_ms),
            ..Self::default()
        }
    }

    /// Advances needs to `now_ms`. Disabled care stays frozen while its clock
    /// still advances, so enabling it later never applies decay retroactively.
    pub fn advance(&mut self, now_ms: u64, difficulty: CareDifficulty, enabled: bool) {
        self.clamp();

        let Some(previous_ms) = self.last_updated_ms else {
            self.last_updated_ms = Some(now_ms);
            return;
        };
        if now_ms <= previous_ms {
            return;
        }
        self.last_updated_ms = Some(now_ms);
        if !enabled {
            return;
        }

        let hours = now_ms.saturating_sub(previous_ms) as f64 / HOUR_MS;
        let rates = difficulty.rates();
        increase(
            &mut self.hunger,
            &mut self.decay_carry.hunger,
            rates.hunger_per_hour * hours,
        );
        change(
            &mut self.energy,
            &mut self.decay_carry.energy,
            if self.sleeping {
                rates.sleep_energy_per_hour * hours
            } else {
                -rates.energy_per_hour * hours
            },
        );
        decrease(
            &mut self.happiness,
            &mut self.decay_carry.happiness,
            rates.happiness_per_hour * hours,
        );
        decrease(
            &mut self.cleanliness,
            &mut self.decay_carry.cleanliness,
            rates.cleanliness_per_hour * hours,
        );
        self.clamp();
    }

    /// Applies an action after first accounting for elapsed time.
    pub fn apply(
        &mut self,
        action: CareAction,
        now_ms: u64,
        difficulty: CareDifficulty,
        enabled: bool,
    ) -> CareActionResult {
        self.advance(now_ms, difficulty, enabled);
        if !enabled {
            return CareActionResult::Disabled;
        }

        match action {
            CareAction::Feed => {
                if self.food == 0 {
                    return CareActionResult::OutOfStock;
                }
                self.food -= 1;
                self.hunger = self.hunger.saturating_sub(35);
                self.happiness = self.happiness.saturating_add(5);
            }
            CareAction::Play => {
                if self.toys == 0 {
                    return CareActionResult::OutOfStock;
                }
                self.toys -= 1;
                self.hunger = self.hunger.saturating_add(8);
                self.energy = self.energy.saturating_sub(15);
                self.happiness = self.happiness.saturating_add(25);
                self.cleanliness = self.cleanliness.saturating_sub(5);
            }
            CareAction::Wash => {
                self.cleanliness = self.cleanliness.saturating_add(45);
                self.happiness = self.happiness.saturating_sub(5);
            }
            CareAction::Pet => {
                self.happiness = self.happiness.saturating_add(15);
            }
            CareAction::Sleep => self.sleeping = true,
            CareAction::Wake => self.sleeping = false,
            CareAction::Rest => {
                self.energy = self.energy.saturating_add(20);
                self.happiness = self.happiness.saturating_add(3);
            }
            CareAction::Restock => {
                self.food = MAX_FOOD;
                self.toys = MAX_TOYS;
            }
        }
        self.clamp();
        CareActionResult::Applied
    }

    fn clamp(&mut self) {
        self.hunger = self.hunger.min(100);
        self.energy = self.energy.min(100);
        self.happiness = self.happiness.min(100);
        self.cleanliness = self.cleanliness.min(100);
        self.food = self.food.min(MAX_FOOD);
        self.toys = self.toys.min(MAX_TOYS);
    }
}

const fn default_food() -> u8 {
    DEFAULT_FOOD
}

const fn default_toys() -> u8 {
    DEFAULT_TOYS
}

fn increase(value: &mut u8, carry: &mut f64, amount: f64) {
    change(value, carry, amount);
}

fn decrease(value: &mut u8, carry: &mut f64, amount: f64) {
    change(value, carry, -amount);
}

fn change(value: &mut u8, carry: &mut f64, amount: f64) {
    let total = *carry + amount;
    let whole = total.trunc();
    *carry = total - whole;
    *value = (*value as f64 + whole).clamp(0.0, 100.0) as u8;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn average_state() -> CareState {
        CareState {
            hunger: 50,
            energy: 50,
            happiness: 50,
            cleanliness: 50,
            food: DEFAULT_FOOD,
            toys: DEFAULT_TOYS,
            sleeping: false,
            last_updated_ms: Some(1_000),
            decay_carry: DecayCarry::default(),
        }
    }

    fn apply(state: &mut CareState, action: CareAction) -> CareActionResult {
        state.apply(action, 1_000, CareDifficulty::Normal, true)
    }

    #[test]
    fn defaults_are_healthy() {
        let state = CareState::default();
        assert_eq!((state.hunger, state.energy), (0, 100));
        assert_eq!((state.happiness, state.cleanliness), (100, 100));
        assert_eq!((state.food, state.toys), (DEFAULT_FOOD, DEFAULT_TOYS));
        assert!(!state.sleeping);
    }

    #[test]
    fn values_are_clamped_to_one_hundred() {
        let mut state = average_state();
        state.hunger = 255;
        state.energy = 255;
        state.advance(1_001, CareDifficulty::Normal, true);
        assert_eq!((state.hunger, state.energy), (100, 100));
    }

    #[test]
    fn normal_decay_applies_for_elapsed_time() {
        let mut state = CareState::new(0);
        state.advance(3_600_000, CareDifficulty::Normal, true);
        assert_eq!(state.hunger, 4);
        assert_eq!(state.energy, 96);
        assert_eq!(state.happiness, 98);
        assert_eq!(state.cleanliness, 98);
    }

    #[test]
    fn difficulty_changes_decay_rate() {
        let mut easy = CareState::new(0);
        let mut hard = CareState::new(0);
        easy.advance(3_600_000, CareDifficulty::Easy, true);
        hard.advance(3_600_000, CareDifficulty::Hard, true);
        assert_eq!((easy.hunger, easy.energy), (2, 98));
        assert_eq!((hard.hunger, hard.energy), (7, 93));
    }

    #[test]
    fn fractional_decay_survives_frequent_updates() {
        let mut state = CareState::new(0);
        for quarter in 1..=4 {
            state.advance(quarter * 900_000, CareDifficulty::Normal, true);
        }
        assert_eq!(state.hunger, 4);
        assert_eq!(state.energy, 96);
    }

    #[test]
    fn disabled_care_freezes_state_and_ignores_actions() {
        let mut state = average_state();
        state.advance(3_601_000, CareDifficulty::Hard, false);
        assert_eq!(
            state.apply(CareAction::Feed, 3_601_001, CareDifficulty::Hard, false),
            CareActionResult::Disabled
        );
        assert_eq!((state.hunger, state.energy), (50, 50));
        assert_eq!(state.food, DEFAULT_FOOD);
    }

    #[test]
    fn feed_reduces_hunger() {
        let mut state = average_state();
        assert_eq!(
            apply(&mut state, CareAction::Feed),
            CareActionResult::Applied
        );
        assert_eq!((state.hunger, state.happiness), (15, 55));
        assert_eq!(state.food, DEFAULT_FOOD - 1);
    }

    #[test]
    fn play_uses_energy_and_improves_happiness() {
        let mut state = average_state();
        assert_eq!(
            apply(&mut state, CareAction::Play),
            CareActionResult::Applied
        );
        assert_eq!(
            (
                state.hunger,
                state.energy,
                state.happiness,
                state.cleanliness
            ),
            (58, 35, 75, 45)
        );
        assert_eq!(state.toys, DEFAULT_TOYS - 1);
    }

    #[test]
    fn wash_improves_cleanliness() {
        let mut state = average_state();
        assert_eq!(
            apply(&mut state, CareAction::Wash),
            CareActionResult::Applied
        );
        assert_eq!((state.cleanliness, state.happiness), (95, 45));
    }

    #[test]
    fn petting_improves_happiness() {
        let mut state = average_state();
        assert_eq!(
            apply(&mut state, CareAction::Pet),
            CareActionResult::Applied
        );
        assert_eq!(state.happiness, 65);
    }

    #[test]
    fn sleep_enables_recovery() {
        let mut state = average_state();
        assert_eq!(
            apply(&mut state, CareAction::Sleep),
            CareActionResult::Applied
        );
        state.advance(3_601_000, CareDifficulty::Normal, true);
        assert!(state.sleeping);
        assert_eq!(state.energy, 75);
    }

    #[test]
    fn wake_stops_sleeping() {
        let mut state = average_state();
        state.sleeping = true;
        assert_eq!(
            apply(&mut state, CareAction::Wake),
            CareActionResult::Applied
        );
        assert!(!state.sleeping);
    }

    #[test]
    fn rest_restores_energy() {
        let mut state = average_state();
        assert_eq!(
            apply(&mut state, CareAction::Rest),
            CareActionResult::Applied
        );
        assert_eq!((state.energy, state.happiness), (70, 53));
    }

    #[test]
    fn actions_clamp_at_bounds() {
        let mut state = average_state();
        state.hunger = 3;
        state.happiness = 99;
        assert_eq!(
            apply(&mut state, CareAction::Feed),
            CareActionResult::Applied
        );
        assert_eq!((state.hunger, state.happiness), (0, 100));
    }

    #[test]
    fn old_saved_state_receives_inventory_defaults() {
        let state: CareState = serde_json::from_str(
            r#"{"hunger":20,"energy":80,"happiness":70,"cleanliness":60,"sleeping":false}"#,
        )
        .unwrap();
        assert_eq!((state.food, state.toys), (DEFAULT_FOOD, DEFAULT_TOYS));
    }

    #[test]
    fn inventory_survives_serialization() {
        let mut state = average_state();
        state.food = 2;
        state.toys = 1;
        let restored: CareState =
            serde_json::from_str(&serde_json::to_string(&state).unwrap()).unwrap();
        assert_eq!((restored.food, restored.toys), (2, 1));
    }

    #[test]
    fn empty_inventory_blocks_only_consuming_actions() {
        let mut state = average_state();
        state.food = 0;
        state.toys = 0;
        let before = state.clone();
        assert_eq!(
            apply(&mut state, CareAction::Feed),
            CareActionResult::OutOfStock
        );
        assert_eq!(
            apply(&mut state, CareAction::Play),
            CareActionResult::OutOfStock
        );
        assert_eq!(state, before);
        assert_eq!(
            apply(&mut state, CareAction::Pet),
            CareActionResult::Applied
        );
    }

    #[test]
    fn restock_replenishes_and_clamp_enforces_caps() {
        let mut state = average_state();
        state.food = 0;
        state.toys = 1;
        assert_eq!(
            apply(&mut state, CareAction::Restock),
            CareActionResult::Applied
        );
        assert_eq!((state.food, state.toys), (MAX_FOOD, MAX_TOYS));

        state.food = u8::MAX;
        state.toys = u8::MAX;
        state.advance(1_001, CareDifficulty::Normal, true);
        assert_eq!((state.food, state.toys), (MAX_FOOD, MAX_TOYS));
    }
}
