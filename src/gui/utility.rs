use chrono::{Datelike, Days, Local};
use gdk::glib::{Object, Value};

pub struct Utility {}

#[gtk::template_callbacks(functions)]
impl Utility {
    #[template_callback]
    fn and(#[rest] values: &[Value]) -> bool {
        values
            .iter()
            .map(|v| v.get::<bool>().expect("Bool for an argument"))
            .all(|b| b)
    }

    #[template_callback]
    fn is_some(#[rest] values: &[Value]) -> bool {
        values
            .iter()
            .next()
            .expect("At least one argument has to exist")
            .get::<Option<Object>>()
            .expect("Expected Option for arguments")
            .is_some()
    }

    #[template_callback]
    fn is_none(#[rest] values: &[Value]) -> bool {
        !Utility::is_some(values)
    }

    #[template_callback]
    fn not(value: bool) -> bool {
        !value
    }

    pub fn format_duration(duration: chrono::Duration) -> String {
        if duration.num_hours() == 0 {
            // Translators: duration in minutes, {} must not be translated as it will be replaced with an actual number
            gettextrs::gettext("{} min").replace("{}", duration.num_minutes().to_string().as_str())
        } else if duration.num_days() == 0 {
            (chrono::NaiveDate::from_ymd_opt(2022, 1, 1)
                .unwrap_or_default()
                .and_hms_opt(0, 0, 0)
                .unwrap_or_default()
                + duration)
                // Translators: duration format with hours and minutes, see https://docs.rs/chrono/latest/chrono/format/strftime/index.html#specifiers
                .format(&gettextrs::gettext("%_H hrs %_M min"))
                .to_string()
        } else {
            // Start one day before the new year, otherwise %_d would skip 2.
            (chrono::NaiveDate::from_ymd_opt(2021, 12, 31)
                .unwrap_or_default()
                .and_hms_opt(0, 0, 0)
                .unwrap_or_default()
                + duration)
                // Translators: duration format with days, hours and minutes, see https://docs.rs/chrono/latest/chrono/format/strftime/index.html#specifiers
                .format(&gettextrs::ngettext(
                    "%_d day %_H hrs %_M min",
                    "%_d days %_H hrs %_M min",
                    duration.num_hours().try_into().unwrap_or_default(),
                ))
                .to_string()
        }
    }

    pub fn format_time_human(time: &chrono::NaiveTime) -> String {
        // Translators: formatting of time in a human-readable fashion, see https://docs.rs/chrono/latest/chrono/format/strftime/index.html#specifiers
        time.format(&gettextrs::gettext("%H:%M")).to_string()
    }

    pub fn format_date_human(date: chrono::NaiveDate) -> String {
        let today = Local::now().date_naive();
        if today == date {
            gettextrs::gettext("Today")
        } else if today + Days::new(1) == date {
            gettextrs::gettext("Tomorrow")
        } else if today - Days::new(1) == date {
            gettextrs::gettext("Yesterday")
        } else if today.year() == date.year() {
            // Translators: formatting of dates without year in a human-readable fashion, see https://docs.rs/chrono/latest/chrono/format/strftime/index.html#specifiers
            date.format(&gettextrs::gettext("%a, %d. %B")).to_string()
        } else {
            // Translators: formatting of dates with year in a human-readable fashion, see https://docs.rs/chrono/latest/chrono/format/strftime/index.html#specifiers
            date.format(&gettextrs::gettext("%Y-%m-%d")).to_string()
        }
    }
}
