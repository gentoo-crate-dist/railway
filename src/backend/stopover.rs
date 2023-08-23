use std::cell::RefCell;

use gdk::glib::Object;
use gdk::subclass::prelude::ObjectSubclassIsExt;

gtk::glib::wrapper! {
    pub struct Stopover(ObjectSubclass<imp::Stopover>);
}

impl Stopover {
    pub fn new(stopover: hafas_rs::Stopover) -> Self {
        let s: Self = Object::builder().build();
        s.imp().stopover.swap(&RefCell::new(Some(stopover)));
        s
    }
}

mod imp {
    use gtk::glib;
    use std::cell::RefCell;

    use gdk::{
        glib::{ParamSpec, ParamSpecObject, ParamSpecString, Value},
        prelude::{ParamSpecBuilderExt, ToValue},
        subclass::prelude::{ObjectImpl, ObjectSubclass},
    };
    use once_cell::sync::Lazy;

    use crate::backend::Place;

    #[derive(Default)]
    pub struct Stopover {
        pub(super) stopover: RefCell<Option<hafas_rs::Stopover>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Stopover {
        const NAME: &'static str = "DBStopover";
        type Type = super::Stopover;
    }

    impl ObjectImpl for Stopover {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecObject::builder::<Place>("stop")
                        .read_only()
                        .build(),
                    ParamSpecString::builder("departure").read_only().build(),
                    ParamSpecString::builder("arrival").read_only().build(),
                    ParamSpecString::builder("planned-departure")
                        .read_only()
                        .build(),
                    ParamSpecString::builder("planned-arrival")
                        .read_only()
                        .build(),
                    ParamSpecString::builder("departure-platform")
                        .read_only()
                        .build(),
                    ParamSpecString::builder("arrival-platform")
                        .read_only()
                        .build(),
                    ParamSpecString::builder("planned-departure-platform")
                        .read_only()
                        .build(),
                    ParamSpecString::builder("planned-arrival-platform")
                        .read_only()
                        .build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, _value: &Value, _pspec: &ParamSpec) {}

        fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "stop" => self
                    .stopover
                    .borrow()
                    .as_ref()
                    .map(|o| Place::new(o.stop.clone()))
                    .to_value(),
                "departure" => self
                    .stopover
                    .borrow()
                    .as_ref()
                    .and_then(|o| o.departure.map(|o| o.format("%H:%M").to_string()))
                    .to_value(),
                "arrival" => self
                    .stopover
                    .borrow()
                    .as_ref()
                    .and_then(|o| o.arrival.map(|o| o.format("%H:%M").to_string()))
                    .to_value(),
                "planned-departure" => self
                    .stopover
                    .borrow()
                    .as_ref()
                    .and_then(|o| o.planned_departure.map(|o| o.format("%H:%M").to_string()))
                    .to_value(),
                "planned-arrival" => self
                    .stopover
                    .borrow()
                    .as_ref()
                    .and_then(|o| o.planned_arrival.map(|o| o.format("%H:%M").to_string()))
                    .to_value(),
                "departure-platform" => self
                    .stopover
                    .borrow()
                    .as_ref()
                    .and_then(|o| o.departure_platform.clone())
                    .to_value(),
                "arrival-platform" => self
                    .stopover
                    .borrow()
                    .as_ref()
                    .and_then(|o| o.arrival_platform.clone())
                    .to_value(),
                "planned-departure-platform" => self
                    .stopover
                    .borrow()
                    .as_ref()
                    .and_then(|o| o.planned_departure_platform.clone())
                    .to_value(),
                "planned-arrival-platform" => self
                    .stopover
                    .borrow()
                    .as_ref()
                    .and_then(|o| o.planned_arrival_platform.clone())
                    .to_value(),
                _ => unimplemented!(),
            }
        }
    }
}
