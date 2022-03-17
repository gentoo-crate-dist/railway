use std::cell::RefCell;

use gdk::glib::Object;
use gdk::subclass::prelude::ObjectSubclassIsExt;
use hafas_rest::Station;

gtk::glib::wrapper! {
    pub struct StationObject(ObjectSubclass<imp::StationObject>);
}

impl StationObject {
    pub fn new(station: Station) -> Self {
        let s: Self = Object::new(&[]).expect("Failed to create `StationObject`.");
        s.imp().station.swap(&RefCell::new(Some(station)));
        s
    }

    pub fn station(&self) -> Station {
        self.imp()
            .station
            .borrow()
            .clone()
            .expect("Station not yet set up")
    }
}

mod imp {
    use gtk::glib;
    use hafas_rest::Station;
    use std::cell::RefCell;

    use gdk::{
        glib::{ParamFlags, ParamSpec, ParamSpecString, Value},
        prelude::ToValue,
        subclass::prelude::{ObjectImpl, ObjectSubclass},
    };
    use once_cell::sync::Lazy;

    #[derive(Default, Clone)]
    pub struct StationObject {
        pub(super) station: RefCell<Option<Station>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for StationObject {
        const NAME: &'static str = "DBStationObject";
        type Type = super::StationObject;
    }

    impl ObjectImpl for StationObject {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![ParamSpecString::new(
                    "name",
                    "name",
                    "name",
                    None,
                    ParamFlags::READABLE,
                )]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _obj: &Self::Type, _id: usize, _value: &Value, _pspec: &ParamSpec) {}

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "name" => self
                    .station
                    .borrow()
                    .as_ref()
                    .map(|o| o.name.clone())
                    .to_value(),
                _ => unimplemented!(),
            }
        }
    }
}
