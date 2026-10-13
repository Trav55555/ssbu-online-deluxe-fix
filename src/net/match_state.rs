use std::sync::atomic::{AtomicU8, Ordering};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchStatus {
    Inactive = 0,
    Singles = 1,
    Doubles = 2,
    Training = 3,
}

impl MatchStatus {
    pub fn from_stage(stage_id: u32, training: bool, fighter_count: i32) -> Self {
        if matches!(stage_id, 310 | 354) {
            Self::Inactive
        } else if training || stage_id == 311 {
            Self::Training
        } else if fighter_count > 2 {
            Self::Doubles
        } else {
            Self::Singles
        }
    }

    pub fn is_real_game(self) -> bool {
        matches!(self, Self::Singles | Self::Doubles)
    }
}

pub(super) struct MatchState(AtomicU8);

impl MatchState {
    pub const fn new() -> Self {
        Self(AtomicU8::new(MatchStatus::Inactive as u8))
    }

    pub fn get(&self) -> MatchStatus {
        match self.0.load(Ordering::SeqCst) {
            1 => MatchStatus::Singles,
            2 => MatchStatus::Doubles,
            3 => MatchStatus::Training,
            _ => MatchStatus::Inactive,
        }
    }

    pub fn set(&self, status: MatchStatus, force_update: bool) -> bool {
        let previous = self.0.swap(status as u8, Ordering::SeqCst);
        previous != status as u8 || force_update
    }
}

#[cfg(test)]
mod tests {
    use super::{MatchState, MatchStatus};

    #[test]
    fn result_stages_clean_up_even_when_training_is_reported() {
        for stage in [310, 354] {
            assert_eq!(
                MatchStatus::from_stage(stage, true, 4),
                MatchStatus::Inactive
            );
        }
    }

    #[test]
    fn waiting_and_training_do_not_count_as_real_online_matches() {
        let waiting = MatchStatus::from_stage(311, false, 4);
        let training = MatchStatus::from_stage(1, true, 2);
        assert_eq!(waiting, MatchStatus::Training);
        assert_eq!(training, MatchStatus::Training);
        assert!(!waiting.is_real_game());
        assert!(!training.is_real_game());
    }

    #[test]
    fn fighter_count_selects_singles_or_doubles() {
        assert_eq!(MatchStatus::from_stage(1, false, 2), MatchStatus::Singles);
        assert_eq!(MatchStatus::from_stage(1, false, 3), MatchStatus::Doubles);
        assert_eq!(MatchStatus::from_stage(1, false, 8), MatchStatus::Doubles);
    }

    #[test]
    fn repeated_stage_entry_still_initializes_a_new_match() {
        let state = MatchState::new();
        assert!(state.set(MatchStatus::Singles, true));
        assert!(state.set(MatchStatus::Singles, true));
        assert_eq!(state.get(), MatchStatus::Singles);
    }

    #[test]
    fn repeated_menu_notification_does_not_repeat_cleanup() {
        let state = MatchState::new();
        assert!(state.set(MatchStatus::Doubles, true));
        assert!(state.set(MatchStatus::Inactive, false));
        assert!(!state.set(MatchStatus::Inactive, false));
        assert_eq!(state.get(), MatchStatus::Inactive);
    }
}
