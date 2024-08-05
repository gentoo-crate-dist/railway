# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

- Don't panic when a journey is walk-only.

## [2.7.0] - 2024-07-24

### Added

- Ability to use arrival time instead of departure time.
- Region divider in provider popover.
- Highlight currently selected region in the provider popover.

### Fixed

- Some providers displaying "0 minute walks" in transitions (upstream fix).

## [2.6.0] - 2024-06-12

### Added

- Screen reader support for the station entry.
- Notify users about automatic bookmark deletion with a toast.
- Transitous provider support.
- SBB provider support.
- IVB provider support.
- KVB provider support.
- BART provider support.

### Removed

- TPG provider support.

### Changed

- Rename stopover to stop.

### Fixed

- Display first day header when initial day deviates from request 

### Packaging

Railway does now not depend upon `hafas-rs` any more.
Instead, there are a set of crates `railway-core`, `railway-provider-*`, `railway-api-derive` and `railway-api` replacing it.
Those are packaged on <crates.io>.

## [2.5.0] - 2024-04-30

### Added

- PKP profile support.

### Removed

- INSA profile support.

### Changed

- Port to GNOME 46 widgets.

### Fixed

- Remarks possibly being queried in the incorrect language.
- Crash on Swedish translation.

## [2.4.0] - 2024-03-03

### Added

- Highlight the selected connection in the connections list.
- Made the trip details page keyboard-navigable.
- Improved accessibility

### Changed

- Improved positioning for arrival and departure labels in the connections list.
- Minor UI update regarding journey remarks.

### Fixed

- Some strings not being translatable or using incorrect forms of a plural.
- Also allow `Ctrl+Q` to close the window.
- Performance issues with showing the details of large journeys.
- Bug where an incorrect timezone was used.

## [2.3.0] - 2024-02-11

### Added

- Keyboard navigation to station search.
- Label for frequency of trains.

### Changed

- Strings used in the UI.
- Improved remarks for journey legs.
- Many minor UI improvements.

### Fixed

- For the station search, clear the place when the entry is cleared.

## [2.2.0] - 2023-12-11

### Added

- Translation support using Weblate.
- Filter for providers.
- Indicate when journey was last refreshed.
- Save window height between startups.

### Changed

- Improved UI for date-dividers.
- Disable search button while search is in progress.
- Disable refresh button while refreshing journey is in progress.
- Added timeout to all requests.

### Fixed

- Don't collapse journey legs stopovers and remarks after refreshing.
- Build on i386.

## [2.1.0] - 2023-10-07

### Added

- Keyboard shortcuts
- Date dividers between journeys.

### Changed

- UI improvements on the preferences.
- Don't display walk destination, unless it is the final walk.

### Fixed

- Spinners in the time dropdown not being two-digit sometimes.
- Duration of over one day being displayed incorrectly.
- The date and time cards having a different color than the rest when in dark mode.

### Packaging

- `hafas-rs` is now taken from `crates.io` instead of git.

## [2.0.0] - 2023-09-21

### Changed

- Complete UI overhaul

[Unreleased]: https://gitlab.com/schmiddi-on-mobile/railway/-/compare/2.7.0...master
[2.7.0]: https://gitlab.com/schmiddi-on-mobile/railway/-/compare/2.6.0...2.7.0
[2.6.0]: https://gitlab.com/schmiddi-on-mobile/railway/-/compare/2.5.0...2.6.0
[2.5.0]: https://gitlab.com/schmiddi-on-mobile/railway/-/compare/2.4.0...2.5.0
[2.4.0]: https://gitlab.com/schmiddi-on-mobile/railway/-/compare/2.3.0...2.4.0
[2.3.0]: https://gitlab.com/schmiddi-on-mobile/railway/-/compare/2.2.0...2.3.0
[2.2.0]: https://gitlab.com/schmiddi-on-mobile/railway/-/compare/2.1.0...2.2.0
[2.1.0]: https://gitlab.com/schmiddi-on-mobile/railway/-/compare/2.0.0...2.1.0
[2.0.0]: https://gitlab.com/schmiddi-on-mobile/railway/-/compare/1.5.0...2.0.0
