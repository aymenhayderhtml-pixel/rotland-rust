//! Application startup, engine argument parsing, and initial state configuration.

use engine::render::LitRenderer;
use engine::{EngineArgs, Gpu};

use super::frame_input::apply_hand;
use super::start_world::start_world;
use super::state::{ActorsState, AudioState, PlaySide, RenderState, SessionState, UiState, WorldState};
use crate::{hub_ui, menus, play, settings};

/// Initializes engine and CLI arguments, sets up save folders, and starts the world.
pub fn init(
    gpu: &Gpu,
    args: &EngineArgs,
    render: &mut RenderState,
    world: &mut WorldState,
    actors: &mut ActorsState,
    play: &mut PlaySide,
    ui: &mut UiState,
    session: &mut SessionState,
    audio: &mut AudioState,
) {
    session.seed = args
        .value("--seed")
        .and_then(|s| s.parse().ok())
        .unwrap_or(42);
    session.scene = args.value("--scene").unwrap_or("play").to_string();

    let bench_arg = args.bench.clone().or_else(|| args.value("--bench").map(std::path::PathBuf::from));
    if let Some(path) = bench_arg {
        session.scene = "play".to_string();
        session.seed = 42;
        session.forced_zombies = Some(0);
        session.bench = Some(super::bench::BenchState::new(path));
    }

    // --hour=H fixes the time (screenshots, benchmarks); play mode otherwise starts at 08:00 and runs
    let forced_hour: Option<f32> = args.value("--hour").and_then(|s| s.parse().ok());
    let start = forced_hour.unwrap_or(if session.scene == "play" {
        world::time::Clock::new().hour
    } else {
        12.0
    });
    play.clock = world::time::Clock::new();
    play.clock.set_time(start);
    play.hour = start;
    play.clock_frozen = session.scene != "play" || forced_hour.is_some();
    render.flashlight_on = args.flag("--flashlight");
    session.free_cam = args.flag("--free");
    session.autowalk = args.flag("--autowalk");
    session.autoattack = args.flag("--autoattack");
    session.die_at = args.value("--die-at").and_then(|v| v.parse().ok());
    session.save_at = args.value("--save-at").and_then(|v| v.parse().ok());
    // Scripted runs (screenshots, timed runs, stats) are silent, so tests never make noise
    let muted = args.flag("--mute")
        || ["--shot", "--seconds", "--frames", "--stats"]
            .iter()
            .any(|f| args.value(f).is_some())
        || args.bench.is_some()
        || args.value("--bench").is_some();
    if session.scene == "play" {
        audio.audio = Some(audio::Audio::new(muted));
    }
    // Test option: open a camp screen at the start (shop, bank, crafting, bounties)
    ui.hub = args.value("--hub").and_then(|h| {
        use hub_ui::HubTab as T;
        let tab = match h {
            "bag" => T::Bag,
            "skills" => T::Skills,
            "bounties" => T::Bounties,
            "profile" => T::Profile,
            "map" => T::Map,
            _ => return None,
        };
        Some(hub_ui::HubUiState::new(hub_ui::HubScreen::Tab(tab)))
    });
    ui.camp_ui = match args.value("--camp-screen") {
        Some("shop") => Some(crate::camp_ui::CampUiState::new(crate::camp_ui::CampScreen::Shop)),
        Some("bank") => Some(crate::camp_ui::CampUiState::new(crate::camp_ui::CampScreen::Bank)),
        Some("crafting") => Some(crate::camp_ui::CampUiState::new(crate::camp_ui::CampScreen::Crafting)),
        Some("bounties") => Some(crate::camp_ui::CampUiState::new(crate::camp_ui::CampScreen::Bounties)),
        Some("upgrades") | Some("board") => Some(crate::camp_ui::CampUiState::new(crate::camp_ui::CampScreen::Upgrades)),
        Some("guild") => Some(crate::camp_ui::CampUiState::new(crate::camp_ui::CampScreen::Guild)),
        _ => None,
    };
    session.walk = session.scene == "forest" && !session.free_cam;
    ui.cursor_captured = session.scene == "play" && !session.free_cam;
    render.renderer = Some(LitRenderer::new(gpu));

    // Saves: --save-dir keeps tests away from the real folder; --import-godot=N copies a Godot slot
    // (read only) into Rust slot N; --load=N continues slot N; --menu starts on the main menu
    let slots = save::Slots::new(
        args.value("--save-dir")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(save::default_base_dir),
    );
    let settings_file = match args.value("--save-dir") {
        Some(dir) => std::path::PathBuf::from(dir).join("settings.json"),
        None => settings::settings_path(),
    };
    let mut save_file = None;
    if let Some(n) = args
        .value("--import-godot")
        .and_then(|v| v.parse::<usize>().ok())
    {
        match slots.import_godot(n) {
            Ok(f) => {
                log::info!(
                    "imported Godot slot {n} into {}",
                    slots.path_for(n).map(|p| p.display().to_string()).unwrap_or_default()
                );
                session.active_slot = n as u8;
                save_file = Some(f);
            }
            Err(e) => log::warn!("import of Godot slot {n} failed: {e:?}"),
        }
    }
    if let Some(n) = args.value("--load").and_then(|v| v.parse::<usize>().ok()) {
        match slots.read(n) {
            Ok(f) => {
                session.active_slot = n as u8;
                save_file = Some(f);
            }
            Err(e) => log::warn!("slot {n} not loaded: {e:?}"),
        }
    }
    if session.active_slot == 0 {
        session.active_slot = 1;
    }
    if let Some(seed) = save_file.as_ref().and_then(crate::session::seed_of) {
        session.seed = seed;
    }
    if session.scene == "play" && !session.free_cam {
        let mut m = if args.flag("--menu") {
            menus::Menus::new(settings::Settings::load(&settings_file))
        } else {
            menus::Menus::in_game(settings::Settings::load(&settings_file))
        };
        m.set_save_path(settings_file);
        ui.menus = Some(m);
    }
    session.slot_cards = crate::session::summaries(&slots);
    session.slots = Some(slots);
    ui.dev_panel.open = args.flag("--dev-panel");
    session.start_at = args.value("--at").and_then(game::dev_panel::parse_at_flag);
    if session.bench.is_some() {
        session.forced_zombies = Some(0);
    } else {
        session.forced_zombies = args.value("--zombies").and_then(|s| s.parse::<usize>().ok());
    }
    session.test_climb = args.flag("--climb");
    session.test_chop = args.flag("--chop");
    session.test_first_person = args.flag("--first-person");
    session.test_poi = args.flag("--poi");
    session.near_animals = args
        .value("--animals")
        .map(|s| {
            s.split(',')
                .filter(|e| !e.is_empty())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default();
    session.start_yaw = args
        .value("--yaw")
        .and_then(|s| s.parse::<f32>().ok())
        .map(f32::to_radians);
    session.give = args
        .value("--give")
        .map(|s| {
            s.split(',')
                .filter(|e| !e.is_empty())
                .map(|e| match e.split_once(':') {
                    Some((id, n)) => (id.to_string(), n.parse().unwrap_or(1)),
                    None => (e.to_string(), 1),
                })
                .collect()
        })
        .unwrap_or_default();

    match pack::ModelPack::load(paths::repo_path("assets/models.pack")) {
        Ok(pack) => render.pack = Some(pack),
        Err(e) => log::warn!("no model pack ({e}); run `cargo run --release -p baker`"),
    }
    start_world(gpu, save_file, render, world, actors, play, ui, session, audio);

    if session.bench.is_some() {
        if let Some(z) = actors.zombies.as_mut() {
            z.list.clear();
        }
        world.poi_zombies = None;
    }

    // Test option: take out hotbar slot N at the start (e.g. --slot=2 for the pistol)
    // Test option --chop: the axe in hand
    if session.test_chop
        && let Some(axe) = play.play.as_ref().and_then(|p| p.weapons.get("axe").cloned())
    {
        apply_hand(play::HandChange::Weapon(Some(axe)), render, play, actors, world);
    }
    if let Some(n) = args.value("--slot").and_then(|v| v.parse::<usize>().ok())
        && let Some(change) = play.play.as_mut().and_then(|p| p.press(n))
    {
        apply_hand(change, render, play, actors, world);
    }
}
