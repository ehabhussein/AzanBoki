use anyhow::{Context, Result};
use jiff::{Zoned, civil::Date, tz::TimeZone};

use adhaan::{Coordinates, HighLatitudeRule, Parameters, Prayer, PrayerTimes, prominent_methods};

use crate::{cities::City, settings::Settings};

pub const CALCULATION_METHODS: &[&str] = &[
    "Egyptian",
    "Muslim World League",
    "Umm al-Qura",
    "Karachi",
    "North America (ISNA)",
    "Dubai",
    "Kuwait",
    "Qatar",
    "Singapore",
    "Turkey (Diyanet)",
    "Tehran",
    "Moonsighting Committee",
];

pub const ASR_METHODS: &[&str] = &["Standard", "Hanafi"];
pub const HIGH_LATITUDE_RULES: &[&str] = &[
    "Middle of the night",
    "Seventh of the night",
    "Twilight angle",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PrayerKind {
    Fajr,
    Sunrise,
    Dhuhr,
    Asr,
    Maghrib,
    Isha,
}

impl PrayerKind {
    pub const ALL: [Self; 6] = [
        Self::Fajr,
        Self::Sunrise,
        Self::Dhuhr,
        Self::Asr,
        Self::Maghrib,
        Self::Isha,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Fajr => "Fajr",
            Self::Sunrise => "Sunrise",
            Self::Dhuhr => "Dhuhr",
            Self::Asr => "Asr",
            Self::Maghrib => "Maghrib",
            Self::Isha => "Isha",
        }
    }

    pub fn arabic_name(self) -> &'static str {
        match self {
            Self::Fajr => "الفجر",
            Self::Sunrise => "الشروق",
            Self::Dhuhr => "الظهر",
            Self::Asr => "العصر",
            Self::Maghrib => "المغرب",
            Self::Isha => "العشاء",
        }
    }

    pub fn from_display_index(index: usize) -> Option<Self> {
        Self::ALL.get(index).copied()
    }

    pub fn can_play(self) -> bool {
        self != Self::Sunrise
    }
}

#[derive(Clone, Debug)]
pub struct PrayerEvent {
    pub kind: PrayerKind,
    pub time: Zoned,
}

#[derive(Clone, Debug)]
pub struct DailySchedule {
    pub date: Date,
    pub events: Vec<PrayerEvent>,
}

pub fn local_now(city: &City) -> Result<Zoned> {
    let timezone = TimeZone::get(city.timezone)
        .with_context(|| format!("unknown time zone {}", city.timezone))?;
    Ok(Zoned::now().with_time_zone(timezone))
}

pub fn calculate(date: Date, city: &City, settings: &Settings) -> Result<DailySchedule> {
    let timezone = TimeZone::get(city.timezone)
        .with_context(|| format!("unknown time zone {}", city.timezone))?;
    let method: &'static dyn adhaan::Method = match settings.calculation_method.as_str() {
        "Egyptian" => &prominent_methods::Egyptian,
        "Umm al-Qura" => &prominent_methods::UmmAlQura,
        "Karachi" => &prominent_methods::Karachi,
        "North America (ISNA)" => &prominent_methods::NorthAmerica,
        "Dubai" => &prominent_methods::Dubai,
        "Kuwait" => &prominent_methods::Kuwait,
        "Qatar" => &prominent_methods::Qatar,
        "Singapore" => &prominent_methods::Singapore,
        "Turkey (Diyanet)" => &prominent_methods::Turkey,
        "Tehran" => &prominent_methods::Tehran,
        "Moonsighting Committee" => &prominent_methods::MOONSIGHTING_COMMITTEE,
        _ => &prominent_methods::MuslimWorldLeague,
    };
    let high_latitude_rule = match settings.high_latitude_rule.as_str() {
        "Seventh of the night" => HighLatitudeRule::SeventhOfTheNight,
        "Twilight angle" => HighLatitudeRule::TwilightAngle,
        _ => HighLatitudeRule::MiddleOfTheNight,
    };
    let parameters = Parameters::new(method).with_high_latitude_rule(high_latitude_rule);
    let coordinates = Coordinates {
        latitude: city.latitude,
        longitude: city.longitude,
    };
    let calculated = PrayerTimes::calculate(date, coordinates, parameters)
        .map_err(|_| anyhow::anyhow!("prayer calculation failed at this latitude"))?;
    let asr = if settings.asr_method == "Hanafi" {
        Prayer::AsrThaani
    } else {
        Prayer::AsrAwwal
    };

    let pairs = [
        (PrayerKind::Fajr, Prayer::Fajr),
        (PrayerKind::Sunrise, Prayer::Sunrise),
        (PrayerKind::Dhuhr, Prayer::Dhuhr),
        (PrayerKind::Asr, asr),
        (PrayerKind::Maghrib, Prayer::Maghrib),
        (PrayerKind::Isha, Prayer::Isha),
    ];
    let events = pairs
        .into_iter()
        .map(|(kind, prayer)| PrayerEvent {
            kind,
            time: calculated.time_of(prayer).with_time_zone(timezone.clone()),
        })
        .collect();

    Ok(DailySchedule { date, events })
}

pub fn format_clock(time: &Zoned) -> String {
    format!("{:02}:{:02}", time.hour(), time.minute())
}

pub fn format_countdown(seconds: i64) -> String {
    let seconds = seconds.max(0);
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    if hours > 0 {
        format!("{hours}h {minutes:02}m")
    } else if minutes > 0 {
        format!("{minutes}m")
    } else {
        "less than a minute".into()
    }
}

pub fn next_event<'a>(
    now: &Zoned,
    today: &'a DailySchedule,
    tomorrow: &'a DailySchedule,
) -> Option<&'a PrayerEvent> {
    let now_second = now.timestamp().as_second();
    today
        .events
        .iter()
        .chain(tomorrow.events.iter())
        .find(|event| event.time.timestamp().as_second() > now_second)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{cities, settings::Settings};
    use jiff::civil::date;

    #[test]
    fn cairo_schedule_is_ordered_and_local() {
        let city = cities::by_id("eg-cairo");
        let schedule = calculate(date(2026, 9, 5), city, &Settings::default()).unwrap();
        assert_eq!(schedule.events.len(), 6);
        assert!(
            schedule
                .events
                .windows(2)
                .all(|pair| pair[0].time < pair[1].time)
        );
        assert_eq!(
            schedule.events[0].time.time_zone().iana_name(),
            Some("Africa/Cairo")
        );
    }

    #[test]
    fn hanafi_asr_is_later_than_standard() {
        let city = cities::by_id("eg-cairo");
        let standard = calculate(date(2026, 9, 5), city, &Settings::default()).unwrap();
        let settings = Settings {
            asr_method: "Hanafi".into(),
            ..Settings::default()
        };
        let hanafi = calculate(date(2026, 9, 5), city, &settings).unwrap();
        assert!(hanafi.events[3].time > standard.events[3].time);
    }

    #[test]
    fn countdown_is_human_readable() {
        assert_eq!(format_countdown(3_900), "1h 05m");
        assert_eq!(format_countdown(125), "2m");
        assert_eq!(format_countdown(20), "less than a minute");
    }
}
