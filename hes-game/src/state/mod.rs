mod game;
mod settings;
mod ui;

pub use game::StateExt;
pub use settings::Settings;
use ui::Points;
pub use ui::{
    format_year_log,
    Phase,
    PlanChange,
    Tutorial,
    UIState,
};

use std::sync::{LazyLock, RwLock};

use enum_map::EnumMap;
use hes_engine::{Output, OutputMap, State, World};
use leptos::*;

use crate::{
    debug::get_debug_opts,
    views::{rank_factors, Factor},
    Var,
};

const SAVE_KEY: &str = "hes.save";
pub static BASE_OUTPUT_DEMAND: LazyLock<
    RwLock<[OutputMap; 4]>,
> = LazyLock::new(|| RwLock::new([OutputMap::default(); 4]));

pub static FACTORS: LazyLock<
    RwLock<EnumMap<Var, Vec<Factor>>>,
> = LazyLock::new(|| RwLock::new(EnumMap::default()));

pub fn update_factors(state: &State) {
    if let Ok(mut factors) = FACTORS.write() {
        *factors = rank_factors(state);
    }
}

pub fn base_demand_by_income_levels(
    output: Output,
) -> [f32; 4] {
    BASE_OUTPUT_DEMAND
        .read()
        .expect("Can read base output demand")
        .iter()
        .map(|demand| demand[output])
        .collect::<Vec<_>>()
        .try_into()
        .expect("Mapping from same size arrays")
}

fn read_save() -> Result<Option<(State, UIState)>, anyhow::Error>
{
    if let Some(storage) = window().local_storage().unwrap() {
        storage
            .get_item(SAVE_KEY)
            .unwrap()
            .map(|ser| {
                Ok(serde_json::from_str::<(State, UIState)>(
                    &ser,
                )?)
            })
            .transpose()
    } else {
        Ok(None)
    }
}
fn write_save(
    game: &State,
    ui: &UIState,
) -> Result<(), anyhow::Error> {
    if let Some(storage) = window().local_storage().unwrap() {
        let ser = serde_json::to_string(&(game, ui))?;
        storage.set_item(SAVE_KEY, &ser).unwrap();
    }
    Ok(())
}
pub fn clear_save() {
    if let Some(storage) = window().local_storage().unwrap() {
        storage.remove_item(SAVE_KEY).unwrap();
    }
}

pub fn new_game(world: World) -> (State, UIState) {
    let mut game = State::new(world);
    let mut ui_state = UIState::default();
    let (settings, _) = Settings::rw();

    let runs = settings.with_untracked(|s| s.runs_played);
    ui_state.start_year = game.world.year;
    ui_state.tutorial = settings.with_untracked(|s| s.tutorial);
    game.runs = runs;

    // Set all starting projects/processes as "viewed"
    ui_state.viewed = game
        .world
        .projects
        .unlocked()
        .map(|p| p.id)
        .chain(game.world.processes.unlocked().map(|p| p.id))
        .collect();

    if get_debug_opts().very_popular {
        game.political_capital = 1000;
    }

    if get_debug_opts().skip_tutorial {
        ui_state.tutorial = Tutorial::Ready;
    }

    if get_debug_opts().skip_to_planning {
        ui_state.phase = Phase::Planning;
    }

    init_vars(&game);

    (game, ui_state)
}

fn init_vars(game: &State) {
    *BASE_OUTPUT_DEMAND
        .write()
        .expect("Can write to shared value") =
        game.world.per_capita_demand.clone().map(|d| d.base);
}

pub fn load() -> (State, UIState) {
    tracing::debug!("Loading saved game...");
    let save = read_save().unwrap();
    if let Some((game, mut ui)) = save {
        init_vars(&game);
        ui.phase = Phase::Planning;
        (game, ui)
    } else {
        new_game(World::default())
    }
}

pub fn save(game: &State, ui: &UIState) {
    tracing::debug!("Saving game...");
    write_save(game, ui).unwrap();
}

pub fn has_save() -> bool {
    match read_save() {
        Ok(Some(_)) => true,
        Ok(None) => false,
        Err(_) => {
            // May mean something about the serialization
            // structure changed, so clear to avoid a crash.
            tracing::debug!(
                "Failed to deserialize save, clearing."
            );
            clear_save();
            false
        }
    }
}

pub fn start_new_run() {
    clear_save();
    let _ = window().location().reload();
}
