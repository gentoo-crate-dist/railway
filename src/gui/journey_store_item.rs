use gdk::glib::Object;

use crate::backend::Journey;

gtk::glib::wrapper! {
    pub struct JourneyStoreItem(ObjectSubclass<imp::JourneyStoreItem>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget;
}

impl JourneyStoreItem {
    pub fn new(journey: Journey) -> Self {
        Object::builder().property("journey", &journey).build()
    }
}

pub mod imp {
    use std::cell::RefCell;

    use gdk::glib::Properties;
    use glib::subclass::InitializingObject;
    use gtk::CompositeTemplate;
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;

    use crate::backend::Journey;

    #[derive(CompositeTemplate, Default, Properties)]
    #[properties(wrapper_type = super::JourneyStoreItem)]
    #[template(resource = "/ui/journey_store_item.ui")]
    pub struct JourneyStoreItem {
        #[property(get, set)]
        journey: RefCell<Option<Journey>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for JourneyStoreItem {
        const NAME: &'static str = "DBJourneyStoreItem";
        type Type = super::JourneyStoreItem;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for JourneyStoreItem {}

    impl WidgetImpl for JourneyStoreItem {}
    impl BoxImpl for JourneyStoreItem {}
}
