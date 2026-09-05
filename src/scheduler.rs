use std::{
    sync::{Arc, Mutex, mpsc},
    thread,
    time::Duration,
};

use jiff::civil::Date;

use crate::{
    audio::AudioPlayer,
    cities,
    prayer::{self, DailySchedule, PrayerKind},
    settings::{Settings, SettingsStore},
};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PrayerView {
    pub kind: Option<PrayerKind>,
    pub name: String,
    pub arabic_name: String,
    pub time: String,
    pub enabled: bool,
    pub can_toggle: bool,
    pub is_next: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct UiSnapshot {
    pub city: String,
    pub date: String,
    pub next_prayer: String,
    pub next_arabic: String,
    pub next_time: String,
    pub countdown: String,
    pub status: String,
    pub prayers: Vec<PrayerView>,
}

#[derive(Debug)]
enum Command {
    Refresh,
    Shutdown,
}

pub struct SchedulerHandle {
    sender: mpsc::Sender<Command>,
    worker: Mutex<Option<thread::JoinHandle<()>>>,
}

impl SchedulerHandle {
    pub fn refresh(&self) {
        let _ = self.sender.send(Command::Refresh);
    }

    pub fn shutdown(&self) {
        let _ = self.sender.send(Command::Shutdown);
        if let Some(worker) = self.worker.lock().unwrap().take() {
            let _ = worker.join();
        }
    }
}

impl Drop for SchedulerHandle {
    fn drop(&mut self) {
        let _ = self.sender.send(Command::Shutdown);
    }
}

pub fn spawn(
    settings: Arc<Mutex<Settings>>,
    snapshot: Arc<Mutex<UiSnapshot>>,
    audio: AudioPlayer,
    store: Arc<SettingsStore>,
) -> SchedulerHandle {
    let (sender, receiver) = mpsc::channel();
    let worker = thread::Builder::new()
        .name("prayer-scheduler".into())
        .spawn(move || run(receiver, settings, snapshot, audio, store))
        .expect("failed to start prayer scheduler");
    SchedulerHandle {
        sender,
        worker: Mutex::new(Some(worker)),
    }
}

fn run(
    receiver: mpsc::Receiver<Command>,
    settings: Arc<Mutex<Settings>>,
    snapshot: Arc<Mutex<UiSnapshot>>,
    audio: AudioPlayer,
    store: Arc<SettingsStore>,
) {
    let mut cached: Option<(Settings, Date, DailySchedule, DailySchedule)> = None;
    let mut last_played = settings.lock().unwrap().last_played_prayer.clone();
    let mut last_two_am_refresh: Option<Date> = None;
    let mut force_refresh = true;

    loop {
        let current_settings = settings.lock().unwrap().clone();
        let city = cities::by_id(&current_settings.location_id);
        match prayer::local_now(city) {
            Ok(now) => {
                let date = now.date();
                let needs_two_am_refresh = now.hour() >= 2 && last_two_am_refresh != Some(date);
                let settings_changed = cached.as_ref().is_none_or(|(old, old_date, _, _)| {
                    old != &current_settings || *old_date != date
                });
                if force_refresh || settings_changed || needs_two_am_refresh {
                    match build_schedules(date, city, &current_settings) {
                        Ok((today, tomorrow)) => {
                            cached = Some((current_settings.clone(), date, today, tomorrow));
                            if needs_two_am_refresh {
                                last_two_am_refresh = Some(date);
                            }
                        }
                        Err(error) => {
                            snapshot.lock().unwrap().status = format!("Schedule error: {error:#}");
                        }
                    }
                    force_refresh = false;
                }

                if let Some((_, _, today, tomorrow)) = &cached {
                    *snapshot.lock().unwrap() = make_snapshot(
                        &now,
                        city,
                        today,
                        tomorrow,
                        &current_settings,
                        audio.is_playing(),
                    );
                    maybe_play(
                        &now,
                        today,
                        PlaybackContext {
                            settings: &current_settings,
                            audio: &audio,
                            last_played: &mut last_played,
                            snapshot: &snapshot,
                            settings_state: &settings,
                            store: &store,
                        },
                    );
                }
            }
            Err(error) => snapshot.lock().unwrap().status = format!("Clock error: {error:#}"),
        }

        match receiver.recv_timeout(Duration::from_secs(15)) {
            Ok(Command::Refresh) => force_refresh = true,
            Ok(Command::Shutdown) | Err(mpsc::RecvTimeoutError::Disconnected) => break,
            Err(mpsc::RecvTimeoutError::Timeout) => {}
        }
    }
    audio.stop();
}

fn build_schedules(
    date: Date,
    city: &cities::City,
    settings: &Settings,
) -> anyhow::Result<(DailySchedule, DailySchedule)> {
    let today = prayer::calculate(date, city, settings)?;
    let tomorrow_date = date
        .tomorrow()
        .expect("civil dates near the present have a tomorrow");
    let tomorrow = prayer::calculate(tomorrow_date, city, settings)?;
    Ok((today, tomorrow))
}

struct PlaybackContext<'a> {
    settings: &'a Settings,
    audio: &'a AudioPlayer,
    last_played: &'a mut String,
    snapshot: &'a Arc<Mutex<UiSnapshot>>,
    settings_state: &'a Arc<Mutex<Settings>>,
    store: &'a SettingsStore,
}

fn maybe_play(now: &jiff::Zoned, today: &DailySchedule, context: PlaybackContext<'_>) {
    let PlaybackContext {
        settings,
        audio,
        last_played,
        snapshot,
        settings_state,
        store,
    } = context;
    let now_second = now.timestamp().as_second();
    let now_nanosecond = now.timestamp().as_nanosecond();
    if settings.muted_until_unix > now_second || !settings.master_enabled {
        return;
    }
    for event in &today.events {
        if !event.kind.can_play() || !settings.prayer_enabled(event.kind) {
            continue;
        }
        let delta_nanoseconds = now_nanosecond - event.time.timestamp().as_nanosecond();
        let key = format!("{}:{}", today.date, event.kind.name());
        if should_fire(delta_nanoseconds, &key, last_played) {
            match audio.play(settings.audio_for(event.kind), settings.volume) {
                Ok(()) => {
                    *last_played = key.clone();
                    let saved = {
                        let mut state = settings_state.lock().unwrap();
                        state.last_played_prayer = key;
                        state.clone()
                    };
                    if let Err(error) = store.save(&saved) {
                        eprintln!("Could not persist playback state: {error:#}");
                    }
                    snapshot.lock().unwrap().status = format!("Playing {} Azan", event.kind.name());
                }
                Err(error) => {
                    snapshot.lock().unwrap().status = format!("Audio error: {error:#}");
                }
            }
        }
    }
}

fn should_fire(delta_nanoseconds: i128, key: &str, last_played: &str) -> bool {
    const PLAYBACK_GRACE_NANOSECONDS: i128 = 75_000_000_000;
    (0..=PLAYBACK_GRACE_NANOSECONDS).contains(&delta_nanoseconds) && key != last_played
}

fn seconds_until(now: &jiff::Zoned, event: &jiff::Zoned) -> i64 {
    const NANOSECONDS_PER_SECOND: i128 = 1_000_000_000;
    let remaining = event.timestamp().as_nanosecond() - now.timestamp().as_nanosecond();
    let rounded_up = (remaining.max(0) + NANOSECONDS_PER_SECOND - 1) / NANOSECONDS_PER_SECOND;
    i64::try_from(rounded_up).unwrap_or(i64::MAX)
}

fn make_snapshot(
    now: &jiff::Zoned,
    city: &cities::City,
    today: &DailySchedule,
    tomorrow: &DailySchedule,
    settings: &Settings,
    is_playing: bool,
) -> UiSnapshot {
    let next = prayer::next_event(now, today, tomorrow);
    let next_kind = next.map(|event| event.kind);
    let now_second = now.timestamp().as_second();
    let status = if is_playing {
        "Azan is playing · use Stop from the app or tray".into()
    } else if settings.muted_until_unix > now_second {
        let minutes = (settings.muted_until_unix - now_second + 59) / 60;
        format!("Azan paused for {minutes} more minutes")
    } else if !settings.master_enabled {
        "Azan playback is paused".into()
    } else {
        "Running quietly · schedule refreshes at 02:00 and on startup".into()
    };
    let prayers = today
        .events
        .iter()
        .map(|event| PrayerView {
            kind: Some(event.kind),
            name: event.kind.name().into(),
            arabic_name: event.kind.arabic_name().into(),
            time: prayer::format_clock(&event.time),
            enabled: settings.prayer_enabled(event.kind),
            can_toggle: event.kind.can_play(),
            is_next: next_kind == Some(event.kind) && event.time.date() == today.date,
        })
        .collect();

    UiSnapshot {
        city: format!("{}, {}", city.name, city.country),
        date: today.date.to_string(),
        next_prayer: next.map_or_else(|| "—".into(), |event| event.kind.name().into()),
        next_arabic: next.map_or_else(String::new, |event| event.kind.arabic_name().into()),
        next_time: next.map_or_else(|| "—".into(), |event| prayer::format_clock(&event.time)),
        countdown: next.map_or_else(String::new, |event| {
            prayer::format_countdown(seconds_until(now, &event.time))
        }),
        status,
        prayers,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn playback_has_a_short_resume_grace_and_deduplicates() {
        assert!(should_fire(0, "2026-09-05:Fajr", ""));
        assert!(should_fire(75_000_000_000, "2026-09-05:Fajr", ""));
        assert!(!should_fire(75_000_000_001, "2026-09-05:Fajr", ""));
        assert!(!should_fire(-1, "2026-09-05:Fajr", ""));
        assert!(!should_fire(
            10_000_000_000,
            "2026-09-05:Fajr",
            "2026-09-05:Fajr"
        ));
    }
}
