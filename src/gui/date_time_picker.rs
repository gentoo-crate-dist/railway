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

    pub fn reset(&self) {
        self.imp().reset();
    }
}

pub mod imp {
    use chrono::DateTime;
    use chrono::Local;
    use chrono::NaiveDate;
    use chrono::TimeZone;
    use gdk::glib::clone;
    use gdk::glib::subclass::InitializingObject;
    use gtk::glib;
    use gtk::subclass::prelude::*;
    use gtk::traits::WidgetExt;
    use gtk::Align;
    use gtk::CompositeTemplate;
    use libadwaita::prelude::EditableExt;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/ui/date_time_picker.ui")]
    pub struct DateTimePicker {
        #[template_child]
        pick_cal: TemplateChild<gtk::Calendar>,
        #[template_child]
        pick_hour: TemplateChild<gtk::SpinButton>,
        #[template_child]
        pick_minute: TemplateChild<gtk::SpinButton>,

        #[template_child]
        btn_input_time: TemplateChild<gtk::MenuButton>,
        #[template_child]
        btn_input_date: TemplateChild<gtk::MenuButton>,

        #[template_child]
        label_time: TemplateChild<gtk::Label>,
        #[template_child]
        label_date: TemplateChild<gtk::Label>,
    }

    impl DateTimePicker {
        pub(super) fn get(&self) -> DateTime<Local> {
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

        pub(super) fn reset(&self) {
            if let Ok(now) = gdk::glib::DateTime::now_local() {
                self.pick_cal.select_day(&now);
                self.pick_hour.set_value(now.hour() as f64);
                self.pick_minute.set_value(now.minute() as f64);
            }
            self.update_date_label();
            self.update_time_label();
        }

        fn update_date_label(&self) {
            let cal_date = self.pick_cal.date();

            // How to format a date with time. Should probably be similar to %a %b %d (meaning print day of the week (short), month of the year (short), day from 01-31).
            // For a full list of supported identifiers, see <https://docs.gtk.org/glib/method.DateTime.format.html>
            let format = gettextrs::gettext("%a %b %d");
            self.label_date.set_label(
                &cal_date
                    .format(&format)
                    .map(|s| s.to_string())
                    .unwrap_or_default(),
            )
        }

        fn update_time_label(&self) {
            let hour = self.pick_hour.value().floor() as u32;
            let minute = self.pick_minute.value().floor() as u32;

            self.pick_hour.set_text(&format!("{:02}", hour));
            self.pick_minute.set_text(&format!("{:02}", minute));

            // TODO: Internationalization
            self.label_time
                .set_label(&format!("{:02}:{:02}", hour, minute))
        }

        fn connect_expander_date_subtitle(&self) {
            let obj = self.obj();
            self.pick_cal.connect_day_selected(clone!(
                @weak obj
                => move |_| {
                    obj.imp().update_date_label();
            }));
        }

        fn connect_btn_input_time_label(&self) {
            let obj = self.obj();
            self.pick_hour.connect_value_changed(clone!(
                @weak obj
                => move |_| {
                    obj.imp().update_time_label();
            }));

            self.pick_minute.connect_value_changed(clone!(
                @weak obj
                => move |_| {
                    obj.imp().update_time_label();
            }));
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for DateTimePicker {
        const NAME: &'static str = "DBDateTimePicker";
        type Type = super::DateTimePicker;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for DateTimePicker {
        fn constructed(&self) {
            self.parent_constructed();
            self.reset();
            self.connect_expander_date_subtitle();
            self.connect_btn_input_time_label();

            // Ever menu button looks like:
            // MenuButton -> ToggleButton -> Box
            // For some reason, the Box is set to `halign=center`, which breaks the halign of the child widget.
            if let Some(b) = self
                .btn_input_time
                .first_child()
                .and_then(|w| w.first_child())
            {
                b.set_halign(Align::Fill)
            }
            if let Some(b) = self
                .btn_input_date
                .first_child()
                .and_then(|w| w.first_child())
            {
                b.set_halign(Align::Fill)
            }
        }
    }

    impl WidgetImpl for DateTimePicker {}
    impl BoxImpl for DateTimePicker {}
}
