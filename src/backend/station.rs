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
        glib::{ParamSpec, ParamSpecString, Value},
        prelude::{ParamSpecBuilderExt, ToValue},
        subclass::prelude::{ObjectImpl, ObjectSubclass},
    };
    use once_cell::sync::Lazy;

    #[derive(Default)]
    pub struct Stop {
        pub(super) station: RefCell<Option<rcore::Station>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Stop {
        const NAME: &'static str = "DBStation";
        type Type = super::Station;
    }

    impl ObjectImpl for Stop {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> =
                Lazy::new(|| vec![ParamSpecString::builder("name").read_only().build()]);
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, _value: &Value, _pspec: &ParamSpec) {}

        fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "name" => self
                    .station
                    .borrow()
                    .as_ref()
                    .and_then(|o| o.name.clone())
                    .to_value(),
                _ => unimplemented!(),
            }
        }
    }
}
