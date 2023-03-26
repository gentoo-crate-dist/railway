use std::cell::RefCell;

use gdk::glib::Object;
use gdk::subclass::prelude::ObjectSubclassIsExt;

gtk::glib::wrapper! {
    pub struct Stop(ObjectSubclass<imp::Stop>);
}

impl Stop {
    pub fn new(stop: hafas_rs::Stop) -> Self {
        let s: Self = Object::builder().build();
        s.imp().stop.swap(&RefCell::new(Some(stop)));
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
        pub(super) stop: RefCell<Option<hafas_rs::Stop>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Stop {
        const NAME: &'static str = "DBStop";
        type Type = super::Stop;
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
                    .stop
                    .borrow()
                    .as_ref()
                    .and_then(|o| o.name.clone())
                    .to_value(),
                _ => unimplemented!(),
            }
        }
    }
}
