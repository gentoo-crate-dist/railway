use gdk::glib::Object;
use gtk::{gdk, glib, prelude::*, CompositeTemplate};
use libadwaita::subclass::prelude::*;

glib::wrapper! {
    pub struct TimeDivider(ObjectSubclass<imp::TimeDivider>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for TimeDivider {
    fn default() -> Self {
        Object::builder::<Self>().build()
    }
}

mod imp {
    use chrono::Local;

    use gdk::glib::Properties;
    use glib::subclass::InitializingObject;
    use gtk::subclass::box_::BoxImpl;
    use std::cell::Cell;
    use std::cell::RefCell;
    use std::marker::PhantomData;

    use crate::backend::Journey;
    use crate::backend::JourneysResult;
    use crate::gui::utility::Utility;

    use super::*;

    #[derive(Debug, Default, CompositeTemplate, Properties)]
    #[properties(wrapper_type = super::TimeDivider)]
    #[template(resource = "/ui/time_divider.ui")]
    pub struct TimeDivider {
        #[template_child]
        label_date: TemplateChild<gtk::Label>,

        #[property(name = "start", type = u32, set = Self::set_start)]
        is_start: Cell<bool>,
        #[property(name = "initial", set = Self::set_initial)]
        is_initial: Cell<bool>,
        #[property(name = "journeys-result", set = Self::set_journeys_result)]
        journeys_result: RefCell<Option<JourneysResult>>,
        #[property(name = "item", set = Self::set_item)]
        _item: PhantomData<Option<Journey>>,
    }

    impl TimeDivider {
        fn set_start(&self, v: u32) {
            self.is_start.replace(v == 0);
            self.update_visibility();
        }

        fn set_initial(&self, v: bool) {
            self.is_initial.replace(v);
            self.update_visibility();
        }

        fn set_journeys_result(&self, v: Option<JourneysResult>) {
            self.journeys_result.replace(v);
            self.update_visibility();
        }

        fn set_item(&self, v: Option<Journey>) {
            let formatted = v.map(|v| v.departure_day());
            self.label_date
                .set_text(&formatted.clone().unwrap_or_default());
            self.update_visibility();
        }

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

    #[glib::derived_properties]
    impl ObjectImpl for TimeDivider {}

    impl WidgetImpl for TimeDivider {}
    impl BoxImpl for TimeDivider {}
}
