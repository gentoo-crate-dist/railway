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
    use gdk::glib::subclass::InitializingObject;
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::CompositeTemplate;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/ui/date_time_picker.ui")]
    pub struct DateTimePicker {
        #[template_child]
        pick_cal: TemplateChild<gtk::Calendar>,
        #[template_child]
        pick_hour: TemplateChild<gtk::SpinButton>,
        #[template_child]
        pick_minute: TemplateChild<gtk::SpinButton>,
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
        }
    }

    impl WidgetImpl for DateTimePicker {}
    impl BoxImpl for DateTimePicker {}
}
