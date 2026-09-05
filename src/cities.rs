use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct City {
    pub id: &'static str,
    pub country: &'static str,
    pub name: &'static str,
    pub latitude: f64,
    pub longitude: f64,
    pub timezone: &'static str,
    pub suggested_method: &'static str,
}

macro_rules! city {
    ($id:literal, $country:literal, $name:literal, $lat:literal, $lon:literal, $tz:literal, $method:literal) => {
        City {
            id: $id,
            country: $country,
            name: $name,
            latitude: $lat,
            longitude: $lon,
            timezone: $tz,
            suggested_method: $method,
        }
    };
}

// A deliberately compact built-in catalog keeps the resident footprint tiny.
// The location layer is isolated so a full on-disk GeoNames index can be added
// without touching prayer calculation or UI code.
pub static CITIES: &[City] = &[
    city!(
        "au-melbourne",
        "Australia",
        "Melbourne",
        -37.8136,
        144.9631,
        "Australia/Melbourne",
        "Muslim World League"
    ),
    city!(
        "au-sydney",
        "Australia",
        "Sydney",
        -33.8688,
        151.2093,
        "Australia/Sydney",
        "Muslim World League"
    ),
    city!(
        "bd-dhaka",
        "Bangladesh",
        "Dhaka",
        23.8103,
        90.4125,
        "Asia/Dhaka",
        "Karachi"
    ),
    city!(
        "ca-montreal",
        "Canada",
        "Montreal",
        45.5017,
        -73.5673,
        "America/Toronto",
        "North America (ISNA)"
    ),
    city!(
        "ca-toronto",
        "Canada",
        "Toronto",
        43.6532,
        -79.3832,
        "America/Toronto",
        "North America (ISNA)"
    ),
    city!(
        "ca-vancouver",
        "Canada",
        "Vancouver",
        49.2827,
        -123.1207,
        "America/Vancouver",
        "North America (ISNA)"
    ),
    city!(
        "cn-beijing",
        "China",
        "Beijing",
        39.9042,
        116.4074,
        "Asia/Shanghai",
        "Muslim World League"
    ),
    city!(
        "eg-alexandria",
        "Egypt",
        "Alexandria",
        31.2001,
        29.9187,
        "Africa/Cairo",
        "Egyptian"
    ),
    city!(
        "eg-aswan",
        "Egypt",
        "Aswan",
        24.0889,
        32.8998,
        "Africa/Cairo",
        "Egyptian"
    ),
    city!(
        "eg-cairo",
        "Egypt",
        "Cairo",
        30.0444,
        31.2357,
        "Africa/Cairo",
        "Egyptian"
    ),
    city!(
        "eg-giza",
        "Egypt",
        "Giza",
        30.0131,
        31.2089,
        "Africa/Cairo",
        "Egyptian"
    ),
    city!(
        "eg-luxor",
        "Egypt",
        "Luxor",
        25.6872,
        32.6396,
        "Africa/Cairo",
        "Egyptian"
    ),
    city!(
        "fr-lyon",
        "France",
        "Lyon",
        45.7640,
        4.8357,
        "Europe/Paris",
        "Muslim World League"
    ),
    city!(
        "fr-paris",
        "France",
        "Paris",
        48.8566,
        2.3522,
        "Europe/Paris",
        "Muslim World League"
    ),
    city!(
        "de-berlin",
        "Germany",
        "Berlin",
        52.5200,
        13.4050,
        "Europe/Berlin",
        "Muslim World League"
    ),
    city!(
        "de-frankfurt",
        "Germany",
        "Frankfurt",
        50.1109,
        8.6821,
        "Europe/Berlin",
        "Muslim World League"
    ),
    city!(
        "de-munich",
        "Germany",
        "Munich",
        48.1351,
        11.5820,
        "Europe/Berlin",
        "Muslim World League"
    ),
    city!(
        "in-delhi",
        "India",
        "Delhi",
        28.6139,
        77.2090,
        "Asia/Kolkata",
        "Karachi"
    ),
    city!(
        "in-hyderabad",
        "India",
        "Hyderabad",
        17.3850,
        78.4867,
        "Asia/Kolkata",
        "Karachi"
    ),
    city!(
        "in-mumbai",
        "India",
        "Mumbai",
        19.0760,
        72.8777,
        "Asia/Kolkata",
        "Karachi"
    ),
    city!(
        "id-jakarta",
        "Indonesia",
        "Jakarta",
        -6.2088,
        106.8456,
        "Asia/Jakarta",
        "Singapore"
    ),
    city!(
        "id-surabaya",
        "Indonesia",
        "Surabaya",
        -7.2575,
        112.7521,
        "Asia/Jakarta",
        "Singapore"
    ),
    city!(
        "ir-mashhad",
        "Iran",
        "Mashhad",
        36.2605,
        59.6168,
        "Asia/Tehran",
        "Tehran"
    ),
    city!(
        "ir-tehran",
        "Iran",
        "Tehran",
        35.6892,
        51.3890,
        "Asia/Tehran",
        "Tehran"
    ),
    city!(
        "iq-baghdad",
        "Iraq",
        "Baghdad",
        33.3152,
        44.3661,
        "Asia/Baghdad",
        "Muslim World League"
    ),
    city!(
        "jo-amman",
        "Jordan",
        "Amman",
        31.9539,
        35.9106,
        "Asia/Amman",
        "Muslim World League"
    ),
    city!(
        "ke-nairobi",
        "Kenya",
        "Nairobi",
        -1.2921,
        36.8219,
        "Africa/Nairobi",
        "Muslim World League"
    ),
    city!(
        "kw-kuwait",
        "Kuwait",
        "Kuwait City",
        29.3759,
        47.9774,
        "Asia/Kuwait",
        "Kuwait"
    ),
    city!(
        "lb-beirut",
        "Lebanon",
        "Beirut",
        33.8938,
        35.5018,
        "Asia/Beirut",
        "Muslim World League"
    ),
    city!(
        "my-kuala-lumpur",
        "Malaysia",
        "Kuala Lumpur",
        3.1390,
        101.6869,
        "Asia/Kuala_Lumpur",
        "Singapore"
    ),
    city!(
        "ma-casablanca",
        "Morocco",
        "Casablanca",
        33.5731,
        -7.5898,
        "Africa/Casablanca",
        "Muslim World League"
    ),
    city!(
        "ma-rabat",
        "Morocco",
        "Rabat",
        34.0209,
        -6.8416,
        "Africa/Casablanca",
        "Muslim World League"
    ),
    city!(
        "nl-amsterdam",
        "Netherlands",
        "Amsterdam",
        52.3676,
        4.9041,
        "Europe/Amsterdam",
        "Muslim World League"
    ),
    city!(
        "ng-abuja",
        "Nigeria",
        "Abuja",
        9.0765,
        7.3986,
        "Africa/Lagos",
        "Muslim World League"
    ),
    city!(
        "ng-lagos",
        "Nigeria",
        "Lagos",
        6.5244,
        3.3792,
        "Africa/Lagos",
        "Muslim World League"
    ),
    city!(
        "no-oslo",
        "Norway",
        "Oslo",
        59.9139,
        10.7522,
        "Europe/Oslo",
        "Moonsighting Committee"
    ),
    city!(
        "pk-islamabad",
        "Pakistan",
        "Islamabad",
        33.6844,
        73.0479,
        "Asia/Karachi",
        "Karachi"
    ),
    city!(
        "pk-karachi",
        "Pakistan",
        "Karachi",
        24.8607,
        67.0011,
        "Asia/Karachi",
        "Karachi"
    ),
    city!(
        "pk-lahore",
        "Pakistan",
        "Lahore",
        31.5204,
        74.3587,
        "Asia/Karachi",
        "Karachi"
    ),
    city!(
        "ps-gaza",
        "Palestine",
        "Gaza",
        31.5017,
        34.4668,
        "Asia/Gaza",
        "Muslim World League"
    ),
    city!(
        "ps-jerusalem",
        "Palestine",
        "Jerusalem",
        31.7683,
        35.2137,
        "Asia/Jerusalem",
        "Muslim World League"
    ),
    city!(
        "qa-doha",
        "Qatar",
        "Doha",
        25.2854,
        51.5310,
        "Asia/Qatar",
        "Qatar"
    ),
    city!(
        "sa-jeddah",
        "Saudi Arabia",
        "Jeddah",
        21.4858,
        39.1925,
        "Asia/Riyadh",
        "Umm al-Qura"
    ),
    city!(
        "sa-madinah",
        "Saudi Arabia",
        "Madinah",
        24.4672,
        39.6024,
        "Asia/Riyadh",
        "Umm al-Qura"
    ),
    city!(
        "sa-makkah",
        "Saudi Arabia",
        "Makkah",
        21.3891,
        39.8579,
        "Asia/Riyadh",
        "Umm al-Qura"
    ),
    city!(
        "sa-riyadh",
        "Saudi Arabia",
        "Riyadh",
        24.7136,
        46.6753,
        "Asia/Riyadh",
        "Umm al-Qura"
    ),
    city!(
        "sg-singapore",
        "Singapore",
        "Singapore",
        1.3521,
        103.8198,
        "Asia/Singapore",
        "Singapore"
    ),
    city!(
        "za-cape-town",
        "South Africa",
        "Cape Town",
        -33.9249,
        18.4241,
        "Africa/Johannesburg",
        "Muslim World League"
    ),
    city!(
        "za-johannesburg",
        "South Africa",
        "Johannesburg",
        -26.2041,
        28.0473,
        "Africa/Johannesburg",
        "Muslim World League"
    ),
    city!(
        "es-barcelona",
        "Spain",
        "Barcelona",
        41.3874,
        2.1686,
        "Europe/Madrid",
        "Muslim World League"
    ),
    city!(
        "es-madrid",
        "Spain",
        "Madrid",
        40.4168,
        -3.7038,
        "Europe/Madrid",
        "Muslim World League"
    ),
    city!(
        "se-stockholm",
        "Sweden",
        "Stockholm",
        59.3293,
        18.0686,
        "Europe/Stockholm",
        "Moonsighting Committee"
    ),
    city!(
        "tr-ankara",
        "Türkiye",
        "Ankara",
        39.9334,
        32.8597,
        "Europe/Istanbul",
        "Turkey (Diyanet)"
    ),
    city!(
        "tr-istanbul",
        "Türkiye",
        "Istanbul",
        41.0082,
        28.9784,
        "Europe/Istanbul",
        "Turkey (Diyanet)"
    ),
    city!(
        "ae-abu-dhabi",
        "United Arab Emirates",
        "Abu Dhabi",
        24.4539,
        54.3773,
        "Asia/Dubai",
        "Dubai"
    ),
    city!(
        "ae-dubai",
        "United Arab Emirates",
        "Dubai",
        25.2048,
        55.2708,
        "Asia/Dubai",
        "Dubai"
    ),
    city!(
        "gb-birmingham",
        "United Kingdom",
        "Birmingham",
        52.4862,
        -1.8904,
        "Europe/London",
        "Muslim World League"
    ),
    city!(
        "gb-london",
        "United Kingdom",
        "London",
        51.5072,
        -0.1276,
        "Europe/London",
        "Muslim World League"
    ),
    city!(
        "gb-manchester",
        "United Kingdom",
        "Manchester",
        53.4808,
        -2.2426,
        "Europe/London",
        "Muslim World League"
    ),
    city!(
        "us-chicago",
        "United States",
        "Chicago",
        41.8781,
        -87.6298,
        "America/Chicago",
        "North America (ISNA)"
    ),
    city!(
        "us-houston",
        "United States",
        "Houston",
        29.7604,
        -95.3698,
        "America/Chicago",
        "North America (ISNA)"
    ),
    city!(
        "us-los-angeles",
        "United States",
        "Los Angeles",
        34.0522,
        -118.2437,
        "America/Los_Angeles",
        "North America (ISNA)"
    ),
    city!(
        "us-new-york",
        "United States",
        "New York",
        40.7128,
        -74.0060,
        "America/New_York",
        "North America (ISNA)"
    ),
    city!(
        "us-washington",
        "United States",
        "Washington, D.C.",
        38.9072,
        -77.0369,
        "America/New_York",
        "North America (ISNA)"
    ),
];

pub fn by_id(id: &str) -> &'static City {
    CITIES
        .iter()
        .find(|city| city.id == id)
        .unwrap_or_else(|| CITIES.iter().find(|city| city.id == "eg-cairo").unwrap())
}

pub fn countries() -> Vec<&'static str> {
    CITIES
        .iter()
        .map(|city| city.country)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

pub fn for_country(country: &str) -> Vec<&'static City> {
    CITIES
        .iter()
        .filter(|city| city.country == country)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_city_has_a_valid_timezone() {
        for city in CITIES {
            assert!(
                jiff::tz::TimeZone::get(city.timezone).is_ok(),
                "{}",
                city.id
            );
        }
    }

    #[test]
    fn country_lists_are_sorted_and_non_empty() {
        let countries = countries();
        assert!(!countries.is_empty());
        assert!(countries.windows(2).all(|pair| pair[0] < pair[1]));
    }
}
