use anyhow::{Context, Result};
use jiff::{RoundMode, Unit, Zoned, ZonedRound, civil::Date, tz::TimeZone};

use adhaan::{
    Coordinates, HighLatitudeRule, Method, Parameters, Prayer, PrayerTimes, TimeAdjustment,
    prominent_methods,
};

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
    let method: &'static dyn Method = match settings.calculation_method.as_str() {
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
    let parameters = astronomical_parameters(method, high_latitude_rule);
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
            time: calculated
                .unrounded_time_of(prayer)
                .with_time_zone(timezone.clone()),
        })
        .collect();

    Ok(DailySchedule { date, events })
}

pub fn format_clock(time: &Zoned) -> String {
    let display_time = time
        .round(
            ZonedRound::new()
                .smallest(Unit::Minute)
                .mode(RoundMode::Ceil),
        )
        .expect("rounding a prayer time to minutes must succeed");
    format_clock_parts(
        i64::from(display_time.hour()),
        i64::from(display_time.minute()),
    )
}

fn astronomical_parameters(
    method: &'static dyn Method,
    high_latitude_rule: HighLatitudeRule,
) -> Parameters {
    let adjustment = method.adjustments();
    let neutralizer = TimeAdjustment {
        fajr: -adjustment.fajr,
        sunrise: -adjustment.sunrise,
        dhuhr: -adjustment.dhuhr,
        asr: -adjustment.asr,
        maghrib: -adjustment.maghrib,
        isha: -adjustment.isha,
    };
    Parameters::new(method)
        .with_high_latitude_rule(high_latitude_rule)
        .with_adjustments(neutralizer)
}

fn format_clock_parts(hour: i64, minute: i64) -> String {
    let period = if hour < 12 { "AM" } else { "PM" };
    let hour = match hour % 12 {
        0 => 12,
        hour => hour,
    };
    format!("{hour}:{minute:02} {period}")
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
    today
        .events
        .iter()
        .chain(tomorrow.events.iter())
        .find(|event| event.time.timestamp() > now.timestamp())
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
    fn cairo_schedule_preserves_astronomical_seconds() {
        let city = cities::by_id("eg-cairo");
        let schedule = calculate(date(2026, 9, 5), city, &Settings::default()).unwrap();
        assert!(schedule.events.iter().any(|event| event.time.second() != 0));
    }

    #[test]
    fn astronomical_mode_cancels_intrinsic_minute_adjustments() {
        let parameters = astronomical_parameters(
            &prominent_methods::Egyptian,
            HighLatitudeRule::MiddleOfTheNight,
        );
        for prayer in [
            Prayer::Fajr,
            Prayer::Sunrise,
            Prayer::Dhuhr,
            Prayer::AsrAwwal,
            Prayer::Maghrib,
            Prayer::Isha,
        ] {
            assert_eq!(parameters.time_adjustments(prayer), 0);
        }
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
    fn changing_city_recalculates_from_the_new_coordinates() {
        let settings = Settings::default();
        let cairo = calculate(date(2026, 9, 5), cities::by_id("eg-cairo"), &settings).unwrap();
        let giza = calculate(date(2026, 9, 5), cities::by_id("eg-giza"), &settings).unwrap();

        assert_ne!(
            cairo.events[0].time.timestamp(),
            giza.events[0].time.timestamp()
        );
        assert_ne!(
            format_clock(&cairo.events[0].time),
            format_clock(&giza.events[0].time)
        );
    }

    #[test]
    fn countdown_is_human_readable() {
        assert_eq!(format_countdown(3_900), "1h 05m");
        assert_eq!(format_countdown(125), "2m");
        assert_eq!(format_countdown(20), "less than a minute");
    }

    #[test]
    fn clock_uses_twelve_hour_am_pm_format() {
        assert_eq!(format_clock_parts(0, 5), "12:05 AM");
        assert_eq!(format_clock_parts(7, 9), "7:09 AM");
        assert_eq!(format_clock_parts(12, 0), "12:00 PM");
        assert_eq!(format_clock_parts(19, 30), "7:30 PM");
    }

    #[test]
    fn displayed_minute_never_precedes_the_calculated_instant() {
        let exact = date(2026, 9, 5)
            .at(5, 5, 1, 0)
            .in_tz("Africa/Cairo")
            .unwrap();
        assert_eq!(format_clock(&exact), "5:06 AM");

        let on_minute = date(2026, 9, 5)
            .at(5, 5, 0, 0)
            .in_tz("Africa/Cairo")
            .unwrap();
        assert_eq!(format_clock(&on_minute), "5:05 AM");
    }

    #[test]
    fn giza_week_matches_the_astronomical_golden_schedule() {
        let city = cities::by_id("eg-giza");
        let expected = [
            [
                "5:06 AM", "6:35 AM", "12:54 PM", "4:27 PM", "7:14 PM", "8:33 PM",
            ],
            [
                "5:06 AM", "6:35 AM", "12:54 PM", "4:26 PM", "7:12 PM", "8:31 PM",
            ],
            [
                "5:07 AM", "6:36 AM", "12:54 PM", "4:26 PM", "7:11 PM", "8:30 PM",
            ],
            [
                "5:08 AM", "6:36 AM", "12:53 PM", "4:25 PM", "7:10 PM", "8:29 PM",
            ],
            [
                "5:08 AM", "6:37 AM", "12:53 PM", "4:24 PM", "7:09 PM", "8:27 PM",
            ],
            [
                "5:09 AM", "6:37 AM", "12:53 PM", "4:24 PM", "7:07 PM", "8:26 PM",
            ],
            [
                "5:10 AM", "6:38 AM", "12:52 PM", "4:23 PM", "7:06 PM", "8:25 PM",
            ],
        ];
        for (day, expected_times) in (5..=11).zip(expected) {
            let schedule = calculate(date(2026, 9, day), city, &Settings::default()).unwrap();
            let actual = schedule
                .events
                .iter()
                .map(|event| format_clock(&event.time))
                .collect::<Vec<_>>();
            assert_eq!(actual, expected_times, "unexpected Giza times on day {day}");
        }
    }
}
