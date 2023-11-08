# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Translation support using Weblate.
- Filter for providers.
- Indicate when journey was last refreshed.

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

[Unreleased]: https://gitlab.com/schmiddi-on-mobile/railway/-/compare/v2.1.0...master
[2.1.0]: https://gitlab.com/schmiddi-on-mobile/railway/-/compare/v2.0.0...v2.1.0
[2.0.0]: https://gitlab.com/schmiddi-on-mobile/railway/-/compare/v1.5.0...v2.0.0
