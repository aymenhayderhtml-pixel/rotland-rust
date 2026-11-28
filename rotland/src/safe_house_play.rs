//! Play side of the rooftop safe house (R44-05): claim, bed, chest, rope ladder, travel signs,
//! the rooftop trader's stock and the save keys. Numbers and prompts come from `data/safe_house.json`.

use game::economy::{RooftopTraderConfig, TraderStock, Wallet};
use game::safe_house::{SafeHouseConfig, SafeHouseState};
use game::{Inventory, ItemRegistry, LadderRail};
use glam::{IVec2, Vec3};
use world::WorldGenerator;
use world::metro::safehouse::{SafeHouseSpots, safe_house_layout, safe_house_spots};

use crate::safe_house_travel::{TravelDecision, travel_decision};
use crate::streaming::Streamer;

/// Save key for the safe house state (`systems.extra`).
pub const SAVE_KEY: &str = "metro_safehouse";
/// Save key for the rooftop trader's stock (`systems.extra`).
pub const TRADER_SAVE_KEY: &str = "rooftop_trader";

/// What the frame must do besides the state change (the state is already updated).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SafeHouseEffect {
    /// Shows the claim toast.
    Claimed,
    /// Sets the clock to `hour` (a rest). The game is saved at the end of the frame.
    Rested { hour: f32 },
    /// Opens the shared bank screen.
    OpenBank,
    /// The roof rope was pulled up or lowered.
    ToggleLadder { down: bool },
    /// Fast travel: the clock moves `hours`; the player is placed at `pending_travel`.
    Travel { to_roof: bool, hours: f32 },
    /// Zombies are chasing: no travel, no time passes.
    TravelBlocked,
}

/// Prompt and key result for one frame.
pub struct SafeHouseInteract {
    pub prompt: Option<String>,
    pub distance: f32,
    pub consumed_press: bool,
    pub effect: Option<SafeHouseEffect>,
}

#[derive(Clone, Copy, PartialEq)]
enum Spot {
    Claim,
    Bed,
    Chest,
    Ladder,
    SignCamp,
    HomeSign,
}

/// The safe house runtime: config, spots on the tower, state, and the rooftop stock.
#[derive(Default)]
pub struct SafeHousePlay {
    pub state: SafeHouseState,
    config: Option<SafeHouseConfig>,
    spots: Option<SafeHouseSpots>,
    pub rooftop_stock: Option<TraderStock>,
    /// The camp screen open now is the rooftop shop (its stock, not the camp's).
    pub shop_on_roof: bool,
    /// Where the player is placed next frame after a fast travel.
    pub pending_travel: Option<Vec3>,
    /// The frame saves the game at its end (set by the bed).
    pub pending_save: bool,
}

impl SafeHousePlay {
    /// Loads config, the tower spots and the rooftop stock; restores the saved state.
    /// Any missing piece turns the safe house off, with a warning.
    pub fn load(world_gen: &WorldGenerator, save: Option<&save::SaveFile>) -> Self {
        let mut me = Self::default();
        let Some(layout) = safe_house_layout() else {
            log::warn!("data/safe_house.json layout not loaded, safe house off");
            return me;
        };
        match SafeHouseConfig::from_repo() {
            Ok(c) => me.config = Some(c),
            Err(e) => log::warn!("data/safe_house.json not loaded, safe house off: {e}"),
        }
        match RooftopTraderConfig::from_repo() {
            Ok(c) => me.rooftop_stock = Some(c.rooftop_stock()),
            Err(e) => log::warn!("data/rooftop_trader.json not loaded: {e}"),
        }
        if let Some(metro) = world_gen.metro_world() {
            me.spots = safe_house_spots(metro, layout, &|x, z| world_gen.height(x, z));
        }
        match me.spots.as_ref() {
            Some(sp) => log::info!("safe house on tower {} at {:?}, roof deck y {:.2}", sp.tower, sp.shack, sp.deck_y),
            None => log::warn!("this city has no safe house tower"),
        }
        if let Some(file) = save {
            if let Some(v) = file.systems.extra.get(SAVE_KEY) {
                me.state = SafeHouseState::from_json(v);
            }
            if let (Some(stock), Some(v)) = (me.rooftop_stock.as_mut(), file.systems.extra.get(TRADER_SAVE_KEY)) {
                stock.from_save(v);
            }
        }
        me
    }

    /// The radius (m) that counts chasing zombies for the travel block; 0 when not loaded.
    pub fn chase_radius_m(&self) -> f32 {
        self.config.as_ref().map_or(0.0, |c| c.chase_radius_m)
    }

    /// The tower spots, when the city has a safe house.
    pub fn spots(&self) -> Option<&SafeHouseSpots> {
        self.spots.as_ref()
    }

    /// Writes the safe house state and the rooftop stock into `systems.extra`.
    pub fn save_into(&self, extra: &mut std::collections::HashMap<String, serde_json::Value>) {
        extra.insert(SAVE_KEY.into(), self.state.to_json());
        if let Some(stock) = &self.rooftop_stock {
            extra.insert(TRADER_SAVE_KEY.into(), stock.to_save());
        }
    }

    /// Toast text for an effect, from the prompts in the config.
    pub fn toast(&self, effect: SafeHouseEffect) -> Option<String> {
        let p = &self.config.as_ref()?.prompts;
        Some(match effect {
            SafeHouseEffect::Claimed => p.claimed_toast.clone(),
            SafeHouseEffect::Rested { .. } => p.rested.clone(),
            SafeHouseEffect::TravelBlocked => p.travel_blocked.clone(),
            _ => return None,
        })
    }

    /// Shows the prompt for the nearest safe house spot in reach and handles [E].
    /// `home_sign` is the home camp's travel sign. `chasing` counts zombies chasing the player.
    pub fn interact_tick(
        &mut self,
        player: Vec3,
        pressed: bool,
        home_sign: Option<Vec3>,
        chasing: &dyn Fn() -> usize,
    ) -> Option<SafeHouseInteract> {
        let cfg = self.config.as_ref()?;
        let spots = self.spots.as_ref()?;
        let pr = &cfg.prompts;
        let reach = cfg.use_radius_m;
        let rest_hour = cfg.rest_hour;
        let travel_hours = cfg.travel_hours;
        let claimed = self.state.claimed;
        let mut options: Vec<(Vec3, Spot, String)> = Vec::new();
        if claimed {
            options.push((spots.bed, Spot::Bed, pr.bed.clone()));
            options.push((spots.chest, Spot::Chest, pr.chest.clone()));
            let ladder = if self.state.ladder_down { pr.ladder_up.clone() } else { pr.ladder_down.clone() };
            options.push((spots.ladder_deck, Spot::Ladder, ladder));
            options.push((spots.sign, Spot::SignCamp, pr.travel_to_camp.clone()));
        } else {
            options.push((spots.shack, Spot::Claim, pr.claim.clone()));
        }
        if let Some(home) = home_sign {
            let text = if claimed { pr.travel_to_safe_house.clone() } else { pr.not_claimed.clone() };
            options.push((home, Spot::HomeSign, text));
        }
        let (dist, spot, text) = options
            .into_iter()
            .map(|(p, s, t)| (p.distance(player), s, t))
            .filter(|(d, _, _)| *d <= reach)
            .min_by(|a, b| a.0.total_cmp(&b.0))?;
        let prompt_only = SafeHouseInteract { prompt: Some(text.clone()), distance: dist, consumed_press: false, effect: None };
        if !pressed || (spot == Spot::HomeSign && !claimed) {
            return Some(prompt_only);
        }
        let effect = match spot {
            Spot::Claim => {
                self.state.claim();
                SafeHouseEffect::Claimed
            }
            Spot::Bed => {
                if let Some(stock) = self.rooftop_stock.as_mut() {
                    stock.fill_all();
                }
                self.pending_save = true;
                SafeHouseEffect::Rested { hour: rest_hour }
            }
            Spot::Chest => SafeHouseEffect::OpenBank,
            Spot::Ladder => SafeHouseEffect::ToggleLadder { down: self.state.toggle_ladder() },
            Spot::SignCamp | Spot::HomeSign => {
                let to_roof = spot == Spot::HomeSign;
                match travel_decision(claimed, chasing()) {
                    TravelDecision::Go => {
                        self.pending_travel = if to_roof { Some(spots.travel_point) } else { home_sign };
                        SafeHouseEffect::Travel { to_roof, hours: travel_hours }
                    }
                    TravelDecision::Blocked => SafeHouseEffect::TravelBlocked,
                }
            }
        };
        Some(SafeHouseInteract { prompt: None, distance: dist, consumed_press: true, effect: Some(effect) })
    }

    /// Buys one item from the rooftop stock (the rooftop shop's Buy button).
    pub fn buy_on_roof(
        &mut self,
        wallet: &mut Wallet,
        bag: &mut Inventory,
        item: &str,
        count: u32,
        buy_rate: f32,
        items: &ItemRegistry,
    ) -> Option<String> {
        let stock = self.rooftop_stock.as_mut()?;
        let got = game::buy(wallet, bag, stock, item, count, buy_rate, items);
        Some(if got > 0 { format!("Bought {got} {item}") } else { "Can't buy that".into() })
    }

    /// Keeps the rope rail and the chunks around the roof in step with `state.ladder_down` (R44).
    /// Called once per frame before streaming. Does nothing while they already match.
    pub fn sync_rope(&self, streamer: &mut Streamer) {
        let down = self.state.ladder_down;
        WorldGenerator::set_safehouse_ladder_down(down);
        let Some(spots) = self.spots.as_ref() else {
            return;
        };
        let has_rope = !streamer.extra_ladders.is_empty();
        if has_rope == down {
            return;
        }
        streamer.extra_ladders = if down {
            vec![LadderRail::new(spots.ladder_foot, spots.ladder_deck, "metro_safehouse")]
        } else {
            Vec::new()
        };
        let size = WorldGenerator::CHUNK_SIZE;
        let cx = (spots.shack.x / size).floor() as i32;
        let cz = (spots.shack.z / size).floor() as i32;
        let coords: Vec<IVec2> = (-1..=1)
            .flat_map(|dz| (-1..=1).map(move |dx| IVec2::new(cx + dx, cz + dz)))
            .collect();
        streamer.reload_chunks(&coords);
    }
}
