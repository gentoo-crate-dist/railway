use chrono::{Datelike, Days, Local};
use gdk::glib::{Object, Value};
use gtk::subclass::prelude::WidgetImpl;
use gtk::prelude::IsA;
use gtk::subclass::prelude::ObjectSubclassExt;
use gtk::DirectionType;
use gtk::Widget;
use gtk::prelude::WidgetExt;

pub struct Utility {}

#[derive(PartialEq)]
enum Direction {
    Forward,
    Backward,
}

#[derive(PartialEq)]
enum ChildAccess {
    Skip,
    Include,
    Only,
}

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

    pub fn format_duration_tabular(duration: chrono::Duration) -> String {
        if duration.num_hours() == 0 {
            // Translators: duration in minutes, standalone or tabular setting, {} must not be translated as it will be replaced with an actual number
            gettextrs::gettext("{} min").replace("{}", duration.num_minutes().to_string().as_str())
        } else if duration.num_days() == 0 {
            (chrono::NaiveDate::from_ymd_opt(2022, 1, 1)
                .unwrap_or_default()
                .and_hms_opt(0, 0, 0)
                .unwrap_or_default()
                + duration)
                // Translators: duration format with hours and minutes, standalone or tabular setting, see https://docs.rs/chrono/latest/chrono/format/strftime/index.html#specifiers
                .format(&gettextrs::gettext("%_H h %_M min"))
                .to_string()
        } else {
            // Start one day before the new year, otherwise %_d would skip 2.
            (chrono::NaiveDate::from_ymd_opt(2021, 12, 31)
                .unwrap_or_default()
                .and_hms_opt(0, 0, 0)
                .unwrap_or_default()
                + duration)
                // Translators: duration format with days, hours and minutes, standalone or tabular setting, see https://docs.rs/chrono/latest/chrono/format/strftime/index.html#specifiers
                .format(&gettextrs::ngettext(
                    " %_d day %_H h %_M min",
                    "%_d days %_H h %_M min",
                    duration.num_hours().try_into().unwrap_or_default(),
                ))
                .to_string()
        }
    }

    pub fn format_duration_inline(duration: chrono::Duration) -> String {
        if duration.num_hours() < 2 {
            // Translators: duration in minutes, embedded in text, {} must not be translated as it will be replaced with an actual number
            gettextrs::gettext("{} min.").replace("{}", duration.num_minutes().to_string().as_str())
        } else if duration.num_days() == 0 {
            if duration.num_minutes() == 0 {
                // Translators: duration in hours, embedded in text, {} must not be translated as it will be replaced with an actual number
                gettextrs::gettext("{} hrs.").replace("{}", duration.num_hours().to_string().as_str())
            } else {
                (chrono::NaiveDate::from_ymd_opt(2022, 1, 1)
                    .unwrap_or_default()
                    .and_hms_opt(0, 0, 0)
                    .unwrap_or_default()
                    + duration)
                    // Translators: duration format with hours and minutes, embedded in text, see https://docs.rs/chrono/latest/chrono/format/strftime/index.html#specifiers
                    .format(&gettextrs::gettext("%-H hrs. %-M min."))
                    .to_string()
            }
        } else {
            // Start one day before the new year, otherwise %_d would skip 2.
            (chrono::NaiveDate::from_ymd_opt(2021, 12, 31)
                .unwrap_or_default()
                .and_hms_opt(0, 0, 0)
                .unwrap_or_default()
                + duration)
                // Translators: duration format with days, hours and minutes, embedded in text, see https://docs.rs/chrono/latest/chrono/format/strftime/index.html#specifiers
                .format(&gettextrs::ngettext(
                    "%-d day %-H hrs. %-M min.",
                    "%-d days %-H hrs. %-M min.",
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

    pub fn move_focus_within_container<T: IsA<Widget>>(widget: &(impl WidgetImpl + ObjectSubclassExt<Type = T>), direction: DirectionType) -> bool {
        /* if has child with focus and it keeps focus within, keep within this widget as well */
        if let Some(focus_child) = widget.obj().focus_child() {
            if focus_child.child_focus(direction) {
                return true;
            }
        }

        let move_direction = match direction {
            DirectionType::TabBackward
            | DirectionType::Up
            | DirectionType::Left => Direction::Backward,
            DirectionType::TabForward
            | DirectionType::Down
            | DirectionType::Right => Direction::Forward,
            _ => {
                log::error!("Widget's focus implementation incomplete");
                Direction::Forward
            }
        };
        let child_access = match direction {
            DirectionType::TabBackward | DirectionType::TabForward => ChildAccess::Include,
            DirectionType::Up | DirectionType::Down => ChildAccess::Skip,
            DirectionType::Left | DirectionType::Right => ChildAccess::Only,
            _ => {
                log::error!("Widget's focus implementation incomplete");
                ChildAccess::Include
            }
        };

        match move_direction {
            Direction::Backward => {
                /* if this widget has focus, it is tabbing out */
                if widget.obj().has_focus() {
                    child_access == ChildAccess::Only
                } else {
                    /* when tabbing in, start it children */
                    if child_access != ChildAccess::Skip && widget.obj().last_child()
                        .map(|child| child.child_focus(direction))
                        .unwrap_or(false) {
                        return true;
                    }
                    /* when tabbing in and no child grabs focus, focus this widget */
                    widget.grab_focus()
                }
            }
            Direction::Forward => {
                /* when a child had focus and didn't keep it, it is tabbing out */
                if widget.obj().focus_child().is_some () {
                    child_access == ChildAccess::Only
                } else {
                    /* if this widget had no focus, it is tabbing in */
                    if !widget.obj().has_focus() && widget.grab_focus() {
                        true
                    } else if child_access != ChildAccess::Skip {
                        /* if this widget had focus, pass on to children, tabbing out otherwise */
                        widget.obj().first_child()
                            .map(|child| child.child_focus(direction))
                            .unwrap_or(false)
                    } else {
                        child_access == ChildAccess::Only
                    }
                }
            }
        }
    }
}
