#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod audio;
mod cities;
mod instance_signal;
mod platform;
mod prayer;
mod scheduler;
mod settings;

use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use anyhow::{Context, Result};
use slint::{
    CloseRequestResponse, ComponentHandle, ModelRc, SharedString, Timer, TimerMode, VecModel,
};

use audio::AudioPlayer;
use scheduler::{SchedulerHandle, UiSnapshot};
use settings::{Settings, SettingsStore};

slint::include_modules!();

fn main() {
    if let Err(error) = run() {
        eprintln!("AzanBoki could not start: {error:#}");
        let _ = rfd::MessageDialog::new()
            .set_title("AzanBoki could not start")
            .set_description(format!("{error:#}"))
            .set_level(rfd::MessageLevel::Error)
            .show();
    }
}

fn run() -> Result<()> {
    let store = Arc::new(SettingsStore::new()?);
    let loaded = store.load().unwrap_or_else(|error| {
        eprintln!("Using default settings after load error: {error:#}");
        Settings::default()
    });

    let lock_path = SettingsStore::data_dir()?.join("instance.lock");
    let Some(instance_claim) = instance_signal::InstanceClaim::claim(&lock_path)? else {
        return Ok(());
    };

    if loaded.start_on_login
        && let Err(error) = platform::set_start_on_login(true)
    {
        eprintln!("Could not configure start-on-login: {error:#}");
    }

    let settings = Arc::new(Mutex::new(loaded));
    let snapshot = Arc::new(Mutex::new(UiSnapshot::default()));
    let audio = AudioPlayer::default();
    let scheduler = Arc::new(scheduler::spawn(
        settings.clone(),
        snapshot.clone(),
        audio.clone(),
        store.clone(),
    ));

    let window = MainWindow::new().context("failed to create the settings window")?;
    let tray = AppTray::new().context("failed to create the system tray icon")?;

    initialize_controls(&window, &settings.lock().unwrap());
    wire_window_callbacks(&window, &settings, &store, &scheduler, &audio);
    wire_tray_callbacks(&tray, &window, &settings, &store, &scheduler, &audio);
    let _instance_listener = instance_claim.listen(window.as_weak())?;

    let window_weak = window.as_weak();
    window.window().on_close_requested(move || {
        if let Some(window) = window_weak.upgrade() {
            let _ = window.hide();
        }
        CloseRequestResponse::HideWindow
    });

    let ui_timer = Timer::default();
    {
        let window_weak = window.as_weak();
        let tray_weak = tray.as_weak();
        let snapshot = snapshot.clone();
        let settings = settings.clone();
        ui_timer.start(TimerMode::Repeated, Duration::from_secs(5), move || {
            let current = snapshot.lock().unwrap().clone();
            let state = settings.lock().unwrap().clone();
            if let Some(window) = window_weak.upgrade() {
                apply_snapshot(&window, &current);
                window.set_master_enabled(state.master_enabled);
            }
            if let Some(tray) = tray_weak.upgrade() {
                tray.set_paused(!state.master_enabled);
                tray.set_next_label(
                    format!("Next: {} · {}", current.next_prayer, current.next_time).into(),
                );
            }
        });
    }

    tray.show()?;
    let background_launch = std::env::args().any(|arg| arg == "--background");
    if !background_launch || !settings.lock().unwrap().start_hidden {
        window.show()?;
    }
    slint::run_event_loop().context("the application event loop stopped unexpectedly")?;

    ui_timer.stop();
    scheduler.shutdown();
    audio.stop();
    Ok(())
}

fn initialize_controls(window: &MainWindow, settings: &Settings) {
    window.set_method_options(string_model(prayer::CALCULATION_METHODS.iter().copied()));
    window.set_asr_options(string_model(prayer::ASR_METHODS.iter().copied()));
    window.set_high_latitude_options(string_model(prayer::HIGH_LATITUDE_RULES.iter().copied()));
    let countries = cities::countries();
    window.set_country_options(string_model(countries.iter().copied()));
    apply_location_controls(window, settings);
    apply_setting_controls(window, settings);
}

fn apply_location_controls(window: &MainWindow, settings: &Settings) {
    let selected = cities::by_id(&settings.location_id);
    let countries = cities::countries();
    let country_index = countries
        .iter()
        .position(|country| *country == selected.country)
        .unwrap_or(0);
    let city_list = cities::for_country(selected.country);
    let city_index = city_list
        .iter()
        .position(|city| city.id == selected.id)
        .unwrap_or(0);
    window.set_country_index(country_index as i32);
    window.set_city_options(string_model(city_list.iter().map(|city| city.name)));
    window.set_city_index(city_index as i32);
    window.set_current_city(format!("{}, {}", selected.name, selected.country).into());
    window.set_method_index(index_of(
        prayer::CALCULATION_METHODS,
        &settings.calculation_method,
    ));
    window.set_asr_index(index_of(prayer::ASR_METHODS, &settings.asr_method));
    window.set_high_latitude_index(index_of(
        prayer::HIGH_LATITUDE_RULES,
        &settings.high_latitude_rule,
    ));
}

fn apply_setting_controls(window: &MainWindow, settings: &Settings) {
    window.set_volume(settings.volume);
    window.set_master_enabled(settings.master_enabled);
    window.set_start_on_login(settings.start_on_login);
    window.set_start_hidden(settings.start_hidden);
    window.set_regular_audio_name(audio::display_name(&settings.regular_audio).into());
    window.set_fajr_audio_name(audio::display_name(&settings.fajr_audio).into());
}

fn apply_snapshot(window: &MainWindow, snapshot: &UiSnapshot) {
    window.set_current_city(snapshot.city.clone().into());
    window.set_date_label(snapshot.date.clone().into());
    window.set_next_prayer(snapshot.next_prayer.clone().into());
    window.set_next_arabic(snapshot.next_arabic.clone().into());
    window.set_next_time(snapshot.next_time.clone().into());
    window.set_countdown(snapshot.countdown.clone().into());
    window.set_status_message(snapshot.status.clone().into());
    let rows = snapshot
        .prayers
        .iter()
        .map(|item| PrayerItem {
            name: item.name.clone().into(),
            arabic_name: item.arabic_name.clone().into(),
            time: item.time.clone().into(),
            enabled: item.enabled,
            can_toggle: item.can_toggle,
            is_next: item.is_next,
        })
        .collect::<Vec<_>>();
    window.set_prayers(ModelRc::new(VecModel::from(rows)));
}

fn wire_window_callbacks(
    window: &MainWindow,
    settings: &Arc<Mutex<Settings>>,
    store: &Arc<SettingsStore>,
    scheduler: &Arc<SchedulerHandle>,
    audio: &AudioPlayer,
) {
    {
        let settings = settings.clone();
        let store = store.clone();
        let scheduler = scheduler.clone();
        window.on_prayer_toggled(move |index, enabled| {
            if let Some(prayer) = prayer::PrayerKind::from_display_index(index as usize) {
                mutate_settings(&settings, &store, &scheduler, |state| {
                    state.set_prayer_enabled(prayer, enabled)
                });
            }
        });
    }

    let selected_country = Arc::new(Mutex::new(
        cities::by_id(&settings.lock().unwrap().location_id)
            .country
            .to_string(),
    ));
    {
        let settings = settings.clone();
        let store = store.clone();
        let scheduler = scheduler.clone();
        let selected_country = selected_country.clone();
        let window_weak = window.as_weak();
        window.on_country_selected(move |index| {
            let countries = cities::countries();
            let Some(country) = countries.get(index as usize).copied() else {
                return;
            };
            *selected_country.lock().unwrap() = country.to_string();
            let choices = cities::for_country(country);
            let Some(first) = choices.first().copied() else {
                return;
            };
            mutate_settings(&settings, &store, &scheduler, |state| {
                state.location_id = first.id.into();
                state.calculation_method = first.suggested_method.into();
            });
            if let Some(window) = window_weak.upgrade() {
                let state = settings.lock().unwrap();
                apply_location_controls(&window, &state);
            }
        });
    }
    {
        let settings = settings.clone();
        let store = store.clone();
        let scheduler = scheduler.clone();
        let selected_country = selected_country.clone();
        let window_weak = window.as_weak();
        window.on_city_selected(move |index| {
            let country = selected_country.lock().unwrap().clone();
            let choices = cities::for_country(&country);
            let Some(city) = choices.get(index as usize).copied() else {
                return;
            };
            mutate_settings(&settings, &store, &scheduler, |state| {
                state.location_id = city.id.into();
                state.calculation_method = city.suggested_method.into();
            });
            if let Some(window) = window_weak.upgrade() {
                let state = settings.lock().unwrap();
                apply_location_controls(&window, &state);
            }
        });
    }

    wire_choice_callback(
        window,
        settings,
        store,
        scheduler,
        ChoiceKind::CalculationMethod,
    );
    wire_choice_callback(window, settings, store, scheduler, ChoiceKind::AsrMethod);
    wire_choice_callback(window, settings, store, scheduler, ChoiceKind::HighLatitude);
    wire_audio_picker(window, settings, store, scheduler, false);
    wire_audio_picker(window, settings, store, scheduler, true);

    {
        let settings = settings.clone();
        let audio = audio.clone();
        let window_weak = window.as_weak();
        window.on_preview_regular(move || {
            let state = settings.lock().unwrap().clone();
            report_audio_result(&window_weak, audio.play(&state.regular_audio, state.volume));
        });
    }
    {
        let settings = settings.clone();
        let audio = audio.clone();
        let window_weak = window.as_weak();
        window.on_preview_fajr(move || {
            let state = settings.lock().unwrap().clone();
            report_audio_result(
                &window_weak,
                audio.play(state.audio_for(prayer::PrayerKind::Fajr), state.volume),
            );
        });
    }
    {
        let audio = audio.clone();
        window.on_stop_audio(move || audio.stop());
    }
    {
        let settings = settings.clone();
        let store = store.clone();
        let scheduler = scheduler.clone();
        window.on_volume_changed(move |volume| {
            mutate_settings(&settings, &store, &scheduler, |state| {
                state.volume = volume.clamp(0.0, 1.0)
            })
        });
    }
    {
        let settings = settings.clone();
        let store = store.clone();
        let scheduler = scheduler.clone();
        window.on_master_toggled(move |enabled| {
            mutate_settings(&settings, &store, &scheduler, |state| {
                state.master_enabled = enabled
            })
        });
    }
    {
        let settings = settings.clone();
        let store = store.clone();
        let scheduler = scheduler.clone();
        let window_weak = window.as_weak();
        window.on_start_on_login_toggled(move |enabled| {
            match platform::set_start_on_login(enabled) {
                Ok(()) => mutate_settings(&settings, &store, &scheduler, |state| {
                    state.start_on_login = enabled
                }),
                Err(error) => {
                    if let Some(window) = window_weak.upgrade() {
                        window.set_start_on_login(!enabled);
                        window
                            .set_status_message(format!("Start-on-login error: {error:#}").into());
                    }
                }
            }
        });
    }
    {
        let settings = settings.clone();
        let store = store.clone();
        let scheduler = scheduler.clone();
        window.on_start_hidden_toggled(move |enabled| {
            mutate_settings(&settings, &store, &scheduler, |state| {
                state.start_hidden = enabled
            })
        });
    }
    {
        let window_weak = window.as_weak();
        window.on_close_to_tray(move || {
            if let Some(window) = window_weak.upgrade() {
                let _ = window.hide();
            }
        });
    }
}

fn wire_tray_callbacks(
    tray: &AppTray,
    window: &MainWindow,
    settings: &Arc<Mutex<Settings>>,
    store: &Arc<SettingsStore>,
    scheduler: &Arc<SchedulerHandle>,
    audio: &AudioPlayer,
) {
    let show_window = {
        let window_weak = window.as_weak();
        move || {
            if let Some(window) = window_weak.upgrade() {
                let _ = window.show();
                window.window().request_redraw();
            }
        }
    };
    tray.on_open_requested(show_window);
    {
        let settings = settings.clone();
        let store = store.clone();
        let scheduler = scheduler.clone();
        tray.on_toggle_paused(move || {
            mutate_settings(&settings, &store, &scheduler, |state| {
                state.master_enabled = !state.master_enabled
            })
        });
    }
    {
        let settings = settings.clone();
        let store = store.clone();
        let scheduler = scheduler.clone();
        tray.on_mute_hour(move || {
            mutate_settings(&settings, &store, &scheduler, |state| {
                state.muted_until_unix = jiff::Timestamp::now().as_second() + 3600
            })
        });
    }
    {
        let audio = audio.clone();
        tray.on_stop_audio(move || audio.stop());
    }
    tray.on_quit_requested(|| {
        let _ = slint::quit_event_loop();
    });
}

#[derive(Clone, Copy)]
enum ChoiceKind {
    CalculationMethod,
    AsrMethod,
    HighLatitude,
}

fn wire_choice_callback(
    window: &MainWindow,
    settings: &Arc<Mutex<Settings>>,
    store: &Arc<SettingsStore>,
    scheduler: &Arc<SchedulerHandle>,
    kind: ChoiceKind,
) {
    let settings = settings.clone();
    let store = store.clone();
    let scheduler = scheduler.clone();
    let callback = move |index: i32| {
        mutate_settings(&settings, &store, &scheduler, |state| match kind {
            ChoiceKind::CalculationMethod => {
                if let Some(value) = prayer::CALCULATION_METHODS.get(index as usize) {
                    state.calculation_method = (*value).into();
                }
            }
            ChoiceKind::AsrMethod => {
                if let Some(value) = prayer::ASR_METHODS.get(index as usize) {
                    state.asr_method = (*value).into();
                }
            }
            ChoiceKind::HighLatitude => {
                if let Some(value) = prayer::HIGH_LATITUDE_RULES.get(index as usize) {
                    state.high_latitude_rule = (*value).into();
                }
            }
        })
    };
    match kind {
        ChoiceKind::CalculationMethod => window.on_method_selected(callback),
        ChoiceKind::AsrMethod => window.on_asr_selected(callback),
        ChoiceKind::HighLatitude => window.on_high_latitude_selected(callback),
    }
}

fn wire_audio_picker(
    window: &MainWindow,
    settings: &Arc<Mutex<Settings>>,
    store: &Arc<SettingsStore>,
    scheduler: &Arc<SchedulerHandle>,
    fajr: bool,
) {
    let settings = settings.clone();
    let store = store.clone();
    let scheduler = scheduler.clone();
    let window_weak = window.as_weak();
    let callback = move || {
        let chosen = rfd::FileDialog::new()
            .set_title(if fajr {
                "Choose a Fajr Azan recording"
            } else {
                "Choose an Azan recording"
            })
            .add_filter("Audio", &["ogg", "mp3", "wav"])
            .pick_file();
        let Some(path) = chosen else { return };
        let value = path.to_string_lossy().to_string();
        mutate_settings(&settings, &store, &scheduler, |state| {
            if fajr {
                state.fajr_audio = value.clone();
            } else {
                state.regular_audio = value.clone();
            }
        });
        if let Some(window) = window_weak.upgrade() {
            apply_setting_controls(&window, &settings.lock().unwrap());
        }
    };
    if fajr {
        window.on_choose_fajr_audio(callback);
    } else {
        window.on_choose_regular_audio(callback);
    }
}

fn report_audio_result(window: &slint::Weak<MainWindow>, result: Result<()>) {
    if let Some(window) = window.upgrade() {
        match result {
            Ok(()) => window.set_status_message("Previewing Azan · press Stop at any time".into()),
            Err(error) => window.set_status_message(format!("Audio error: {error:#}").into()),
        }
    }
}

fn mutate_settings(
    settings: &Arc<Mutex<Settings>>,
    store: &Arc<SettingsStore>,
    scheduler: &Arc<SchedulerHandle>,
    mutation: impl FnOnce(&mut Settings),
) {
    let saved = {
        let mut state = settings.lock().unwrap();
        mutation(&mut state);
        state.clone()
    };
    if let Err(error) = store.save(&saved) {
        eprintln!("Could not save settings: {error:#}");
    }
    scheduler.refresh();
}

fn string_model<'a>(values: impl IntoIterator<Item = &'a str>) -> ModelRc<SharedString> {
    ModelRc::new(VecModel::from(
        values
            .into_iter()
            .map(SharedString::from)
            .collect::<Vec<_>>(),
    ))
}

fn index_of(values: &[&str], selected: &str) -> i32 {
    values
        .iter()
        .position(|value| *value == selected)
        .unwrap_or(0) as i32
}
