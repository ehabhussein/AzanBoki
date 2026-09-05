# Local `adhaan` fork

This directory contains the MIT-licensed `adhaan` 0.3.0 calculation engine by
Muhammad Ragib Hasin, itself based on the Batoul Apps Adhan library.

AzanBoki keeps this small source fork so it can use the engine's astronomical
instants before its public API rounds them to whole minutes. The only local API
addition is `PrayerTimes::unrounded_time_of`; the solar calculation itself is
unchanged. See `LICENSE` for the original copyright and license terms.
