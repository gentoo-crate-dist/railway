use std::cmp::Ordering;

use chrono::Duration;
use gdk::glib;

#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy, glib::Enum, Default)]
#[repr(u32)]
#[enum_type(name = "DBLateFactor")]
pub enum LateFactor {
    #[default]
    OnTime, // 0 minutes
    LittleLate,    // 1 - 4 minutes
    Late,          // 5 - 14 minutes
    VeryLate,      // 15 - 59 minutes
    ExtremelyLate, // 1+ hours
}

impl From<Duration> for LateFactor {
    fn from(value: Duration) -> Self {
        let minutes = value.num_minutes();

        if minutes <= 0 {
            Self::OnTime
        } else if minutes < 5 {
            Self::LittleLate
        } else if minutes < 15 {
            Self::Late
        } else if minutes < 60 {
            Self::VeryLate
        } else {
            Self::ExtremelyLate
        }
    }
}

impl PartialOrd for LateFactor {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(match (self, other) {
            (v1, v2) if v1 == v2 => Ordering::Equal,
            (Self::OnTime, _) => Ordering::Less,
            (_, Self::OnTime) => Ordering::Greater,
            (Self::LittleLate, _) => Ordering::Less,
            (_, Self::LittleLate) => Ordering::Greater,
            (Self::Late, _) => Ordering::Less,
            (_, Self::Late) => Ordering::Greater,
            (Self::VeryLate, _) => Ordering::Less,
            (_, Self::VeryLate) => Ordering::Greater,
            // Not sure why this is required. Probably the compiler cannot figure out the first case.
            (_, _) => Ordering::Equal,
        })
    }
}

impl Ord for LateFactor {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}
