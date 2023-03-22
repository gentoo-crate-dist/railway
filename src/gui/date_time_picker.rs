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
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::CompositeTemplate;
    use libadwaita::traits::ExpanderRowExt;

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
        expander_date: TemplateChild<libadwaita::ExpanderRow>,
        #[template_child]
        btn_input_time: TemplateChild<gtk::MenuButton>,
    }

    impl DateTimePicker {
        pub(super) fn get(&self) -> DateTime<Local> {
            let cal_date = self.pick_cal.date();
            let year = cal_date.year();
            let month = cal_date.month() as u32;
            let day = cal_date.day_of_month() as u32;

            let hour = self.pick_hour.value().floor() as u32;
            let minute = self.pick_minute.value().floor() as u32;

            let naive = NaiveDate::from_ymd(year, month, day).and_hms(hour, minute, 0);
            Local.from_local_datetime(&naive).unwrap()
        }

        pub(super) fn reset(&self) {
            if let Ok(now) = gdk::glib::DateTime::now_local() {
                self.pick_cal.select_day(&now);
                self.pick_hour.set_value(now.hour() as f64);
                self.pick_minute.set_value(now.minute() as f64);
            }
            self.update_expander_date_subtitle();
            self.update_btn_input_time_label();
        }

        fn update_expander_date_subtitle(&self) {
            let cal_date = self.pick_cal.date();

            // How to format a date with time. Should probably be similar to %Y-%m-%d (meaning print year, month from 01-12, day from 01-31 (each separated by -)).
            // For a full list of supported identifiers, see <https://docs.gtk.org/glib/method.DateTime.format.html>
            let format = gettextrs::gettext("%Y-%m-%d");
            // TODO: Internationalization
            self.expander_date.set_subtitle(
                &cal_date
                    .format(&format)
                    .map(|s| s.to_string())
                    .unwrap_or_default(),
            )
        }

        fn update_btn_input_time_label(&self) {
            let hour = self.pick_hour.value().floor() as u32;
            let minute = self.pick_minute.value().floor() as u32;

            // TODO: Internationalization
            self.btn_input_time
                .set_label(&format!("{:02}:{:02}", hour, minute))
        }

        fn connect_expander_date_subtitle(&self) {
            let obj = self.instance();
            self.pick_cal.connect_day_selected(clone!(
                @weak obj
                => move |_| {
                    obj.imp().update_expander_date_subtitle();
            }));
        }

        fn connect_btn_input_time_label(&self) {
            let obj = self.instance();
            self.pick_hour.connect_value_changed(clone!(
                @weak obj
                => move |_| {
                    obj.imp().update_btn_input_time_label();
            }));

            self.pick_minute.connect_value_changed(clone!(
                @weak obj
                => move |_| {
                    obj.imp().update_btn_input_time_label();
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
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            self.reset();
            self.connect_expander_date_subtitle();
            self.connect_btn_input_time_label();
        }
    }

    impl WidgetImpl for DateTimePicker {}
    impl BoxImpl for DateTimePicker {}
}
