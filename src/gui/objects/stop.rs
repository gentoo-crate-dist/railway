use std::cell::RefCell;

use gdk::glib::Object;
use gdk::subclass::prelude::ObjectSubclassIsExt;
use hafas_rest::Stop;

gtk::glib::wrapper! {
    pub struct StopObject(ObjectSubclass<imp::StopObject>);
}

impl StopObject {
    pub fn new(stop: Stop) -> Self {
        let s: Self = Object::new(&[]).expect("Failed to create `StopObject`.");
        s.imp().stop.swap(&RefCell::new(Some(stop)));
        s
    }
}

mod imp {
    use gtk::glib;
    use hafas_rest::Stop;
    use std::cell::RefCell;

    use gdk::{
        glib::{ParamFlags, ParamSpec, ParamSpecString, Value},
        prelude::ToValue,
        subclass::prelude::{ObjectImpl, ObjectSubclass},
    };
    use once_cell::sync::Lazy;

    #[derive(Default, Clone)]
    pub struct StopObject {
        pub(super) stop: RefCell<Option<Stop>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for StopObject {
        const NAME: &'static str = "DBStopObject";
        type Type = super::StopObject;
    }

    impl ObjectImpl for StopObject {
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
                    .stop
                    .borrow()
                    .as_ref()
                    .map(|o| o.name.clone())
                    .to_value(),
                _ => unimplemented!(),
            }
        }
    }
}
