//! Start-of-game effects of the R44 `--metro=` flags: the safe house claim, the rooftop shop screen
//! and the hour. The poses are in `metro_flags_r44`; the state changes use the R44-05 play code.

use crate::app::metro_flags::MetroSpot;
use crate::app::state::{PlaySide, SessionState, UiState};

/// Hour the R44 test spots start at (the owner's screenshot hour). // new: DECISIONS 2026-11-28
const R44_TEST_HOUR: f32 = 10.0;

/// Applies the R44 flag state after the pose is set: claims the safe house for
/// `safehouse_claimed` and `rooftop_trader`, and opens the rooftop Shop for `rooftop_trader`.
pub(super) fn apply_safehouse_flags(play: &mut PlaySide, ui: &mut UiState, session: &SessionState) {
    let Some(spot) = session.test_flags.metro.as_deref().and_then(MetroSpot::parse) else {
        return;
    };
    let is_r44 = matches!(
        spot,
        MetroSpot::SafeHouse
            | MetroSpot::SafeHouseClaimed
            | MetroSpot::SafeHouseBed
            | MetroSpot::RooftopCamp
            | MetroSpot::RooftopTrader
            | MetroSpot::RescueFloor
    );
    if !is_r44 {
        return;
    }
    if !play.clock_frozen {
        play.hour = R44_TEST_HOUR;
        play.clock.set_time(R44_TEST_HOUR);
        play.clock_frozen = true;
    }
    if matches!(spot, MetroSpot::SafeHouseClaimed | MetroSpot::SafeHouseBed | MetroSpot::RooftopTrader) {
        play.safe_house.state.claim();
    }
    if spot == MetroSpot::RooftopTrader {
        play.safe_house.shop_on_roof = true;
        ui.camp_ui = Some(rotland_ui::camp_ui::CampUiState::new(rotland_ui::camp_ui::CampScreen::Shop));
        ui.cursor_captured = false;
    }
}
