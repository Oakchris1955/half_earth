use leptos::*;

use crate::{
    eval::{eval_badges, summarize, Ending},
    i18n, icons,
    state::{format_year_log, Settings, StateExt, UIState},
    t,
    util::load_images,
    views::{tip, Events, HasTip},
};
use hes_engine::{EventPhase, State};
// we also need this macro or else load_images won't compile
use include_dir::include_dir;

#[component]
pub fn End(lose: bool) -> impl IntoView {
    let WIN_IMGS: Box<[Box<str>]> = load_images!(
        "$CARGO_MANIFEST_DIR/public/assets/sharing/win/",
        "jpg"
    );
    let COUP_IMGS: Box<[Box<str>]> = load_images!(
        "$CARGO_MANIFEST_DIR/public/assets/sharing/lose/coup/",
        "jpg"
    );
    let DEATH_IMGS: Box<[Box<str>]> = load_images!(
        "$CARGO_MANIFEST_DIR/public/assets/sharing/lose/death/",
        "jpg"
    );
    let LOSE_IMGS: Box<[Box<str>]> = load_images!(
        "$CARGO_MANIFEST_DIR/public/assets/sharing/lose/generic/", "jpg"
    );

    let events = create_rw_signal(vec![]);
    let game = expect_context::<RwSignal<State>>();
    let ui = expect_context::<RwSignal<UIState>>();
    game.update_untracked(|game: &mut State| {
        let evs = if lose {
            StateExt::roll_events(game, EventPhase::BreakStart)
        } else {
            StateExt::roll_events(game, EventPhase::EndStart)
        };
        events.set(evs);
    });

    let (_, set_settings) = Settings::rw();
    set_settings.update_untracked(|settings| {
        settings.runs_played += 1;
    });

    let message = if lose {
        t!("This is not the end...")
    } else {
        t!("Well Played!")
    };

    let summary =
        game.with_untracked(|game| summarize(&game, !lose));

    // A little hacky (ideally we do this at compile time)
    // but turn the file paths into the proper urls.
    let root = format!("{}/public", env!("CARGO_MANIFEST_DIR"));
    let image_opts = match summary.ending {
        Ending::Win => WIN_IMGS,
        Ending::Died => DEATH_IMGS,
        Ending::Coup => COUP_IMGS,
        Ending::LostOther => LOSE_IMGS,
    };
    let image_opts: Vec<_> = image_opts
        .iter()
        .filter(|url| url.contains(&summary.faction))
        .map(|path| path.replace(&root, "").to_string())
        .collect();
    let idx = (js_sys::Math::random() * image_opts.len() as f64)
        .floor() as usize;
    let share_image = store_value(image_opts[idx].clone());

    let (show_start, set_show_start) = create_signal(false);

    let badges_view = move || {
        let badges =
            game.with_untracked(|game| eval_badges(&game));
        badges
            .into_iter()
            .map(|badge| {
                let text = i18n::t(&badge.to_string());
                let tip = tip(icons::HELP, text);
                view! {
                    <HasTip tip>
                        <img src=badge.image_url()/>
                        </HasTip>
                }
            })
            .collect::<Vec<_>>()
    };

    let start_new_run = move |_| {
        crate::state::start_new_run();
    };

    let log = ui.with_untracked(|ui| {
        ui.change_history
            .iter()
            .zip(
                ui.process_mix_history
                    .iter()
                    .map(|(_, mixes)| mixes),
            )
            .map(|((year, changes), mixes)| {
                format_year_log(*year, changes, mixes)
            })
            .collect::<Vec<_>>()
            .join("\n")
    });

    view! {
        <div class="break">
            <Events
                events
                on_done=move |_| set_show_start.set(true)
            />
            <Show when=move || {
                show_start.get()
            }>
                <div class="badges-section">
                    <div class="badges">
                        {badges_view}
                    </div>
                </div>
            </Show>
            <Show when=move || show_start.get()>
                <div class="break--actions">
                    <h2>{&message}</h2>
                    <button class="try-again-button" on:click=start_new_run>
                        {t!("Try Again?")}
                    </button>
                </div>
                <div>
                    <img class="share-image" src={share_image.get_value()} />
                </div>
            </Show>
            <pre class="game-history">"Your History\n------------\n"{log}</pre>
        </div>
    }
}
