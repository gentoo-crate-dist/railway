use chrono::DateTime;
use chrono::Local;
use gdk::subclass::prelude::ObjectSubclassIsExt;

gtk::glib::wrapper! {
    pub struct DateTimePicker(ObjectSubclass<imp::DateTimePicker>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget;
}

impl DateTimePicker {
    pub fn get(&self) -> DateTime<Local> {
        self.imp().get()
    }
}

pub mod imp {
    use std::cell::Cell;

    use chrono::DateTime;
    use chrono::Datelike;
    use chrono::Local;
    use chrono::NaiveDate;
    use chrono::TimeZone;
    use chrono::Timelike;
    use gdk::glib::clone;
    use gdk::glib::subclass::InitializingObject;
    use gdk::glib::Properties;
    use gdk::prelude::*;
    use gtk::glib;
    use gtk::prelude::PopoverExt;
    use gtk::subclass::prelude::*;
    use gtk::template_callbacks;
    use gtk::CompositeTemplate;
    use libadwaita::prelude::EditableExt;
    use libadwaita::prelude::ToggleButtonExt;
    use libadwaita::prelude::ActionRowExt;

    #[derive(CompositeTemplate, Default, Properties)]
    #[template(resource = "/ui/date_time_picker.ui")]
    #[properties(wrapper_type = super::DateTimePicker)]
    pub struct DateTimePicker {
        #[template_child]
        pick_cal: TemplateChild<gtk::Calendar>,
        #[template_child]
        pick_hour: TemplateChild<gtk::SpinButton>,
        #[template_child]
        pick_minute: TemplateChild<gtk::SpinButton>,

        #[template_child]
        btn_input_time: TemplateChild<libadwaita::ActionRow>,
        #[template_child]
        btn_input_date: TemplateChild<libadwaita::ActionRow>,

        #[template_child]
        popover_time: TemplateChild<gtk::Popover>,
        #[template_child]
        popover_date: TemplateChild<gtk::Popover>,

        #[property(get, set)]
        now: Cell<bool>,
    }

    #[template_callbacks]
    impl DateTimePicker {
        pub(super) fn get(&self) -> DateTime<Local> {
            let obj = self.obj();
            if obj.now() {
                Local::now()
            } else {
                let cal_date = self.pick_cal.date();
                let year = cal_date.year();
                let month = cal_date.month() as u32;
                let day = cal_date.day_of_month() as u32;

                let hour = self.pick_hour.value().floor() as u32;
                let minute = self.pick_minute.value().floor() as u32;

                let naive = NaiveDate::from_ymd_opt(year, month, day)
                    .unwrap_or_default()
                    .and_hms_opt(hour, minute, 0)
                    .unwrap_or_default();
                Local.from_local_datetime(&naive).unwrap()
            }
        }

        fn update_date_label(&self) {
            let date = self.get().date_naive();
            if date == Local::now().date_naive() {
                self.btn_input_date.set_subtitle(&gettextrs::gettext("Today"));
            } else {
                // How to format a date with time. Should probably be similar to %a %b %d (meaning print day of the week (short), month of the year (short), day from 01-31).
                // For a full list of supported identifiers, see <https://docs.rs/chrono/latest/chrono/format/strftime/index.html#specifiers>
                let format = gettextrs::gettext("%a %b %d");
                self.btn_input_date.set_subtitle(&date.format(&format).to_string())
            }
        }

        fn update_time_label(&self) {
            if self.obj().now() {
                self.btn_input_time.set_subtitle(&gettextrs::gettext("Now"));
            } else {
                let hour = self.pick_hour.value().floor() as u32;
                let minute = self.pick_minute.value().floor() as u32;

                // TODO: Internationalization
                self.btn_input_time
                    .set_subtitle(&format!("{:02}:{:02}", hour, minute))
            }
        }

        fn connect_btn_input_time_label(&self) {
            let obj = self.obj();
            self.pick_hour.connect_value_changed(clone!(
                @weak obj
                => move |_| {
                    obj.set_now(false);
                    obj.imp().update_time_label();
            }));

            self.pick_minute.connect_value_changed(clone!(
                @weak obj
                => move |_| {
                    obj.set_now(false);
                    obj.imp().update_time_label();
            }));
        }

        fn connect_expander_date_subtitle(&self) {
            let obj = self.obj();
            self.pick_cal.connect_day_selected(clone!(
                @weak obj
                => move |_| {
                    obj.set_now(false);
                    obj.imp().update_date_label();
            }));
        }

        #[template_callback]
        fn handle_spinner_output_two_digit(&self, s: gtk::SpinButton) -> bool {
            s.set_text(&format!("{:02}", s.value()));
            true
        }

        #[template_callback]
        fn handle_time_popover_open(&self) {
            let obj = self.obj();
            let was_now = obj.now();

            let time = self.get();

            self.pick_hour.set_value(time.hour() as f64);

            self.pick_minute.set_value((time.minute() / 5 * 5) as f64);

            // Reset the old `now` as it will be changed by `set_value`.
            obj.set_now(was_now);
        }

        #[template_callback]
        fn handle_date_popover_open(&self) {
            let obj = self.obj();
            let was_now = obj.now();

            let time = self.get();
            self.pick_cal.set_day(time.day().try_into().unwrap_or(1));
            // chrono months are 01 - 12, while glib months are 00 - 11.
            self.pick_cal
                .set_month(time.month().try_into().unwrap_or(1) - 1);
            self.pick_cal.set_year(time.year());

            // Reset the old `now` as it will be changed by `set_value`.
            obj.set_now(was_now);
        }

        #[template_callback]
        fn handle_toggle_now(&self, button: gtk::ToggleButton) {
            self.obj().set_now(button.is_active());
            self.handle_date_popover_open();
            self.handle_time_popover_open();
            self.update_date_label();
            self.update_time_label();
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for DateTimePicker {
        const NAME: &'static str = "DBDateTimePicker";
        type Type = super::DateTimePicker;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            Self::bind_template_callbacks(klass);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for DateTimePicker {
        fn constructed(&self) {
            let obj = self.obj();

            self.parent_constructed();
            self.connect_expander_date_subtitle();
            self.connect_btn_input_time_label();

            obj.connect_now_notify(clone!(@weak self as s => move |_| {
                s.update_date_label();
                s.update_time_label();
            }));

            obj.set_now(true);
            self.handle_date_popover_open();
            self.handle_time_popover_open();
            self.update_date_label();
            self.update_time_label();
            // Now will be unset has `handle_*` will be set to new values.
            obj.set_now(true);

            let popover_date = self.popover_date.get();
            self.btn_input_date.connect_activated(clone!(@weak popover_date => move |_| {
                popover_date.popup();
            }));

            let popover_time = self.popover_time.get();
            self.btn_input_time.connect_activated(clone!(@weak popover_time => move |_| {
                popover_time.popup();
            }));
        }
    }

    impl WidgetImpl for DateTimePicker {}
    impl BoxImpl for DateTimePicker {}
}
