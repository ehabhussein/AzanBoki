<div align="center">
  <img src="assets/icon.svg" width="96" height="96" alt="AzanBoki logo">
  <h1>AzanBoki</h1>
  <p>A quiet, native prayer companion that plays the Azan at prayer time.</p>

  [![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
  ![Rust](https://img.shields.io/badge/built_with-Rust-b7410e.svg)
  ![Windows, Linux, macOS](https://img.shields.io/badge/platform-Windows%20%7C%20Linux%20%7C%20macOS-315046.svg)

  [![Made with Slint](https://raw.githubusercontent.com/slint-ui/slint/master/logo/MadeWithSlint-logo-whitebg.png)](https://slint.dev/)
</div>

AzanBoki stays quietly in the logged-in user's system tray and plays the full
Azan through the speakers—not merely a notification. Prayer times are
calculated locally, so the daily scheduler works without an internet
connection. The app is written in Rust with a Slint interface and does not
embed a browser engine or run a local web server.

> [!IMPORTANT]
> AzanBoki is an early `0.1.0` release. Calculation conventions differ between
> communities; compare its times with your mosque or local authority before
> relying on automatic playback.

## Features

- Offline astronomical Fajr, sunrise, Dhuhr, Asr, Maghrib, and Isha calculation
- 12 calculation methods, Standard/Hanafi Asr, and high-latitude rules
- Second-precision scheduling with no hidden minute adjustments; displayed
  times round upward so they never advertise a time before the calculated start
- 64 built-in cities across 32 countries with IANA time zones and DST handling
- Recalculation on startup, after settings changes, on date changes, and at
  02:00 local time
- Clear 12-hour prayer times with AM/PM indicators
- A verified CC0 built-in Azan plus custom OGG, MP3, and WAV recordings
- Optional separate recording for Fajr
- Preview, volume, stop, per-prayer switches, pause, and one-hour mute controls
- Native tray menu and configurable start-on-login behavior
- Single-instance handling: opening AzanBoki again raises the existing window
- Resume protection that avoids playing a stale Azan long after wake-up
- Persistent per-user settings and no analytics or network requirement

## Platform status

| Platform | Status | Notes |
| --- | --- | --- |
| Windows 10/11 x64 | Tested | Native GUI executable and user startup entry |
| Linux x64 | Beta | Requires an AppIndicator/StatusNotifierItem-capable desktop |
| macOS | Beta | Uses a per-user LaunchAgent for start-on-login |

The source targets all three platforms; the current release has been validated
locally on Windows. Linux and macOS should be treated as beta until their
public CI builds have been enabled. Signed installers and packaged macOS
application bundles are not available yet, so this release is intended for
people comfortable building from source.

## Build and run

Install the current stable [Rust toolchain](https://rustup.rs/), then:

```console
git clone https://github.com/ehabhussein/AzanBoki.git
cd AzanBoki
cargo run --locked
```

Build an optimized binary with:

```console
cargo build --release --locked
```

The binary is created under `target/release/` (`azanboki.exe` on Windows).

### Windows requirements

Run Cargo from a Visual Studio Developer PowerShell with the **Desktop
development with C++** workload installed.

### Debian/Ubuntu requirements

```console
sudo apt-get update
sudo apt-get install build-essential libasound2-dev libfontconfig1-dev \
  libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev
```

Linux tray icons use the StatusNotifierItem/AppIndicator protocol. GNOME may
need the “AppIndicator and KStatusNotifierItem Support” extension.

## Using AzanBoki

1. Open **Location** and select your country, city, calculation method, Asr
   method, and high-latitude rule.
2. Open **Muezzin & audio** to preview the built-in recording or select audio
   files you have permission to use.
3. Enable or disable individual prayers from **Today**.
4. Use **Settings** to control playback, start-on-login, and hidden startup.
5. Closing the window hides it to the tray. Choose **Quit** from the tray menu
   to stop the scheduler completely.

AzanBoki runs in the interactive user session rather than as an operating
system service because services cannot reliably access the logged-in user's
speakers.

```text
Slint window + native tray
          │
   persisted settings
          │
astronomical calculator ── scheduler ── Rodio/CPAL audio output
```

The scheduler wakes in short intervals while the settings window can remain
hidden. It opens the audio output only while previewing or playing an Azan.

### How prayer times are calculated

AzanBoki calculates times locally from the selected city's latitude, longitude,
IANA time zone, and civil date. The solar engine derives the Sun's declination
and equation of time, then solves the solar hour angle for sunrise/sunset and
the selected method's Fajr and Isha twilight angles. Dhuhr is solar transit;
Asr uses the selected Standard (shadow factor 1) or Hanafi (factor 2) rule.

The Egyptian method uses 19.5° for Fajr and 17.5° for Isha. No blanket safety
minutes are added or subtracted. Exact seconds are retained for playback. Since
the interface shows whole minutes, it rounds an instant upward—for example,
`5:05:20 AM` is shown as `5:06 AM`—so the shown time is never earlier than the
calculated start. The scheduler still uses `5:05:20 AM` internally.

## Privacy and local data

Prayer calculation, scheduling, and settings are local. AzanBoki contains no
telemetry, advertising, account system, or location tracking. Selecting a city
uses the coordinates bundled in the source; the app does not request precise
device location.

Custom audio paths and preferences are stored in the operating system's normal
per-user configuration directory. Uninstalling the executable does not
automatically remove that settings file or a start-on-login entry; disable
start-on-login from Settings before removing the app.

## Current limitations

- The built-in catalog contains major cities rather than every locality.
- Coordinates cannot yet be entered manually or detected automatically.
- Binaries are not code-signed, notarized, or distributed as installers.
- The app must be running in the user session for Azan playback.
- System suspend may cause a prayer to be skipped; AzanBoki intentionally does
  not play it more than 75 seconds late.

## Development

```console
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
```

Tests cover prayer ordering, Hanafi Asr, bundled time zones, settings
persistence, countdown formatting, single-instance signaling, and
late/duplicate playback protection. See [CONTRIBUTING.md](CONTRIBUTING.md) for
the contribution workflow and [SECURITY.md](SECURITY.md) for reporting security
issues.

## Audio attribution

The bundled “Beautiful adhan” recording is by Wikimedia Commons user
Adam-synagda and was dedicated to the public domain under CC0 1.0. The original
source and checksum are documented in
[assets/audio/README.md](assets/audio/README.md).

## License

AzanBoki is available under the [MIT License](LICENSE). Third-party dependencies
remain under their respective licenses. Slint is used under its royalty-free
desktop application license, whose attribution requirement is fulfilled by the
official badge above. Users are responsible for having permission to use any
custom Azan recordings they select.
