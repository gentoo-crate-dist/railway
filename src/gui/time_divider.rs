use gdk::glib::Object;
use gtk::{gdk, glib, prelude::*, CompositeTemplate};
use libadwaita::subclass::prelude::*;

glib::wrapper! {
    pub struct TimeDivider(ObjectSubclass<imp::TimeDivider>)
        @extends gtk::Box, gtk::Widget, @implements gtk::Accessible;
}

impl Default for TimeDivider {
    fn default() -> Self {
        Object::builder::<Self>().build()
    }
}

mod imp {
    use chrono::Local;

    use glib::subclass::InitializingObject;
    use gtk::subclass::box_::BoxImpl;
    use once_cell::sync::Lazy;
    use std::cell::Cell;
    use std::cell::RefCell;

    use crate::backend::Journey;
    use crate::backend::JourneysResult;
    use crate::gui::utility::Utility;

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/ui/time_divider.ui")]
    pub struct TimeDivider {
        #[template_child]
        label_date: TemplateChild<gtk::Label>,

        is_start: Cell<bool>,
        is_initial: Cell<bool>,
        journeys_result: RefCell<Option<JourneysResult>>,
    }

    impl TimeDivider {
        fn update_visibility(&self) {
            let is_requested_day = self.label_date.text()
                == self
                    .journeys_result
                    .borrow()
                    .as_ref()
                    .and_then(|r| r.requested_time())
                    .map(|d| Utility::format_date_human(d.with_timezone(&Local).date_naive()))
                    .unwrap_or_default();

            let hide = self.is_start.get() && (!self.is_initial.get() || is_requested_day);
            self.obj().set_visible(!hide);
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TimeDivider {
        const NAME: &'static str = "DBTimeDivider";
        type Type = super::TimeDivider;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for TimeDivider {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecObject::builder::<JourneysResult>("journeys-result")
                        .write_only()
                        .build(),
                    glib::ParamSpecObject::builder::<Journey>("item")
                        .write_only()
                        .build(),
                    glib::ParamSpecUInt::builder("start").write_only().build(),
                    glib::ParamSpecBoolean::builder("initial")
                        .write_only()
                        .build(),
                ]
            });

            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "journeys-result" => {
                    let v = value
                        .get::<Option<JourneysResult>>()
                        .expect("TimeDivider to only get a DateTime with timezone");
                    self.journeys_result.replace(v);
                    self.update_visibility();
                }
                "item" => {
                    let v = value
                        .get::<Option<Journey>>()
                        .expect("TimeDivider to only get Journey");

                    let formatted = v.map(|v| v.departure_day());
                    self.label_date
                        .set_text(&formatted.clone().unwrap_or_default());
                    self.update_visibility();
                }
                "start" => {
                    let v = value
                        .get::<u32>()
                        .expect("TimeDivider to only get an unsigned integer");
                    self.is_start.replace(v == 0);
                    self.update_visibility();
                }
                "initial" => {
                    let v = value
                        .get::<bool>()
                        .expect("TimeDivider to only get an unsigned integer");
                    self.is_initial.replace(v);
                    self.update_visibility();
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, _pspec: &glib::ParamSpec) -> glib::Value {
            unimplemented!();
        }
    }

    impl WidgetImpl for TimeDivider {}
    impl BoxImpl for TimeDivider {}
}
