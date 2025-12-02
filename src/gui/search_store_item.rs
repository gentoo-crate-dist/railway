use gdk::glib::Object;

gtk::glib::wrapper! {
    pub struct SearchStoreItem(ObjectSubclass<imp::SearchStoreItem>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget;
}

impl SearchStoreItem {
    pub fn new(origin: String, destination: String) -> Self {
        Object::builder::<Self>()
            .property("origin", &origin)
            .property("destination", &destination)
            .build()
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

    #[derive(CompositeTemplate, Default, Properties)]
    #[properties(wrapper_type = super::SearchStoreItem)]
    #[template(resource = "/ui/search_store_item.ui")]
    pub struct SearchStoreItem {
        #[property(get, set)]
        origin: RefCell<Option<String>>,
        #[property(get, set)]
        destination: RefCell<Option<String>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SearchStoreItem {
        const NAME: &'static str = "DBSearchStoreItem";
        type Type = super::SearchStoreItem;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for SearchStoreItem {}

    impl WidgetImpl for SearchStoreItem {}
    impl BoxImpl for SearchStoreItem {}
}
