use std::cmp::Ordering;

use gdk::glib;

#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy, glib::Enum, Default)]
#[repr(u32)]
#[enum_type(name = "DBLoadFactor")]
pub enum LoadFactor {
    #[default]
    Unknown,
    LowToMedium,
    High,
    VeryHigh,
    ExceptionallyHigh,
}

impl From<Option<hafas_rs::LoadFactor>> for LoadFactor {
    fn from(value: Option<hafas_rs::LoadFactor>) -> Self {
        match value {
            Some(hafas_rs::LoadFactor::LowToMedium) => Self::LowToMedium,
            Some(hafas_rs::LoadFactor::High) => Self::High,
            Some(hafas_rs::LoadFactor::VeryHigh) => Self::VeryHigh,
            Some(hafas_rs::LoadFactor::ExceptionallyHigh) => Self::ExceptionallyHigh,
            None => Self::Unknown,
        }
    }
}

impl Ord for LoadFactor {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (v1, v2) if v1 == v2 => Ordering::Equal,
            (Self::Unknown, _) => Ordering::Less,
            (_, Self::Unknown) => Ordering::Greater,
            (Self::LowToMedium, _) => Ordering::Less,
            (_, Self::LowToMedium) => Ordering::Greater,
            (Self::High, _) => Ordering::Less,
            (_, Self::High) => Ordering::Greater,
            (Self::VeryHigh, _) => Ordering::Less,
            (_, Self::VeryHigh) => Ordering::Greater,
            // Not sure why this is required. Probably the compiler cannot figure out the first case.
            (_, _) => Ordering::Equal,
        }
    }
}

impl PartialOrd for LoadFactor {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
