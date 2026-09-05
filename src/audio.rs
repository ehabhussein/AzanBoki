use std::{
    fs::File,
    io::Cursor,
    path::Path,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
    thread,
};

use anyhow::{Context, Result, bail};
use rodio::{DeviceSinkBuilder, Player};

use crate::settings::BUILTIN_AUDIO_ID;

static BUILTIN_ADHAN: &[u8] = include_bytes!("../assets/audio/beautiful-adhan.ogg");

#[derive(Clone, Default)]
pub struct AudioPlayer {
    current: Arc<Mutex<Option<Arc<Player>>>>,
    generation: Arc<AtomicU64>,
}

impl AudioPlayer {
    pub fn play(&self, selection: &str, volume: f32) -> Result<()> {
        self.stop();
        if selection.is_empty() {
            bail!("no Azan recording has been selected");
        }

        let output = DeviceSinkBuilder::open_default_sink()
            .context("no working speaker or audio output was found")?;
        let player = if selection == BUILTIN_AUDIO_ID {
            rodio::play(output.mixer(), Cursor::new(BUILTIN_ADHAN))
                .context("the built-in Azan recording could not be decoded")?
        } else {
            let file = File::open(selection)
                .with_context(|| format!("could not open Azan recording at {selection}"))?;
            rodio::play(output.mixer(), file)
                .context("the selected Azan recording could not be decoded")?
        };
        player.set_volume(volume.clamp(0.0, 1.0));

        let player = Arc::new(player);
        let generation = self.generation.fetch_add(1, Ordering::SeqCst) + 1;
        *self.current.lock().unwrap() = Some(player.clone());

        let current = self.current.clone();
        let current_generation = self.generation.clone();
        thread::Builder::new()
            .name("azan-audio".into())
            .spawn(move || {
                player.sleep_until_end();
                drop(output);
                if current_generation.load(Ordering::SeqCst) == generation {
                    current.lock().unwrap().take();
                }
            })
            .context("failed to start the audio playback thread")?;
        Ok(())
    }

    pub fn stop(&self) {
        self.generation.fetch_add(1, Ordering::SeqCst);
        if let Some(player) = self.current.lock().unwrap().take() {
            player.stop();
        }
    }

    pub fn is_playing(&self) -> bool {
        self.current
            .lock()
            .unwrap()
            .as_ref()
            .is_some_and(|player| !player.empty())
    }
}

pub fn display_name(selection: &str) -> String {
    if selection == BUILTIN_AUDIO_ID {
        return "Beautiful Adhan · Adam-synagda".into();
    }
    if selection.is_empty() {
        return "Uses the regular Azan".into();
    }
    Path::new(selection)
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("Custom recording")
        .replace(['_', '-'], " ")
}
