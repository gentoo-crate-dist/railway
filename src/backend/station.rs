use std::cell::RefCell;

use gdk::glib::Object;
use gdk::subclass::prelude::ObjectSubclassIsExt;

gtk::glib::wrapper! {
    pub struct Station(ObjectSubclass<imp::Stop>);
}

impl Station {
    pub fn new(stop: rcore::Station) -> Self {
        let s: Self = Object::builder().build();
        s.imp().station.swap(&RefCell::new(Some(stop)));
        s
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
    #[properties(wrapper_type = super::Station)]
    pub struct Stop {
        #[property(name = "name", type = Option<String>, get = |s: &Self| s.station.borrow().as_ref().and_then(|o| o.name.clone()))]
        pub(super) station: RefCell<Option<rcore::Station>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Stop {
        const NAME: &'static str = "DBStation";
        type Type = super::Station;
    }

    #[glib::derived_properties]
    impl ObjectImpl for Stop {}
}
