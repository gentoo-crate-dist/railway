use gdk::glib::Object;

gtk::glib::wrapper! {
    pub struct Provider(ObjectSubclass<imp::Provider>);
}

impl Provider {
    pub fn new(
        id: &'static str,
        short_name: &str,
        name: Option<&str>,
        regional_group: &str,
        has_icon: bool,
    ) -> Provider {
        let icon_name = if has_icon {
            id.to_lowercase()
        } else {
            crate::config::APP_ID.to_string()
        };
        Object::builder::<Self>()
            .property("id", id)
            .property("short-name", short_name)
            .property("name", name)
            .property("icon-name", &icon_name)
            .property("regional-group", regional_group)
            .build()
    }
}

mod imp {
    use gtk::glib;
    use std::cell::RefCell;

    use gdk::{
        glib::Properties,
        prelude::ObjectExt,
        subclass::prelude::{DerivedObjectProperties, ObjectImpl, ObjectSubclass},
    };

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::Provider)]
    pub struct Provider {
        #[property(get, set, construct_only)]
        id: RefCell<String>,
        #[property(get, set, construct_only)]
        short_name: RefCell<String>,
        #[property(get, set, construct_only)]
        name: RefCell<Option<String>>,
        #[property(get, set, construct_only)]
        icon_name: RefCell<String>,
        #[property(get, set, construct_only)]
        regional_group: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Provider {
        const NAME: &'static str = "DBProvider";
        type Type = super::Provider;
    }

    #[glib::derived_properties]
    impl ObjectImpl for Provider {}
}
