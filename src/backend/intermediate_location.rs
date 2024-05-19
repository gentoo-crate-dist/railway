use std::cell::RefCell;

use gdk::glib::Object;
use gdk::subclass::prelude::ObjectSubclassIsExt;

gtk::glib::wrapper! {
    pub struct IntermediateLocation(ObjectSubclass<imp::IntermediateLocation>);
}

impl IntermediateLocation {
    pub fn new(intermediate_location: rcore::IntermediateLocation) -> Self {
        let s: Self = Object::builder().build();
        s.imp()
            .intermediate_location
            .swap(&RefCell::new(Some(intermediate_location)));
        s
    }
}

mod imp {
    use chrono::Local;
    use gtk::glib;
    use std::cell::RefCell;

    use gdk::{
        glib::{ParamSpec, ParamSpecBoolean, ParamSpecObject, ParamSpecString, Value},
        prelude::{ParamSpecBuilderExt, ToValue},
        subclass::prelude::{ObjectImpl, ObjectSubclass},
    };
    use once_cell::sync::Lazy;

    use crate::backend::Place;

    #[derive(Default)]
    pub struct IntermediateLocation {
        pub(super) intermediate_location: RefCell<Option<rcore::IntermediateLocation>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for IntermediateLocation {
        const NAME: &'static str = "DBIntermediateLocation";
        type Type = super::IntermediateLocation;
    }

    impl ObjectImpl for IntermediateLocation {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecObject::builder::<Place>("place")
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
                    ParamSpecBoolean::builder("is-stop").read_only().build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, _value: &Value, _pspec: &ParamSpec) {}

        fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "place" => self
                    .intermediate_location
                    .borrow()
                    .as_ref()
                    .map(|o| {
                        Place::new(match o {
                            rcore::IntermediateLocation::Stop(s) => s.place.clone(),
                            rcore::IntermediateLocation::Railway(r) => r.clone(),
                        })
                    })
                    .to_value(),
                "departure" => self
                    .intermediate_location
                    .borrow()
                    .as_ref()
                    .and_then(|o| match o {
                        rcore::IntermediateLocation::Stop(s) => Some(s),
                        rcore::IntermediateLocation::Railway(_) => None,
                    })
                    .and_then(|o| {
                        o.departure
                            .map(|o| o.with_timezone(&Local).format("%H:%M").to_string())
                    })
                    .to_value(),
                "arrival" => self
                    .intermediate_location
                    .borrow()
                    .as_ref()
                    .and_then(|o| match o {
                        rcore::IntermediateLocation::Stop(s) => Some(s),
                        rcore::IntermediateLocation::Railway(_) => None,
                    })
                    .and_then(|o| {
                        o.arrival
                            .map(|o| o.with_timezone(&Local).format("%H:%M").to_string())
                    })
                    .to_value(),
                "planned-departure" => self
                    .intermediate_location
                    .borrow()
                    .as_ref()
                    .and_then(|o| match o {
                        rcore::IntermediateLocation::Stop(s) => Some(s),
                        rcore::IntermediateLocation::Railway(_) => None,
                    })
                    .and_then(|o| {
                        o.planned_departure
                            .map(|o| o.with_timezone(&Local).format("%H:%M").to_string())
                    })
                    .to_value(),
                "planned-arrival" => self
                    .intermediate_location
                    .borrow()
                    .as_ref()
                    .and_then(|o| match o {
                        rcore::IntermediateLocation::Stop(s) => Some(s),
                        rcore::IntermediateLocation::Railway(_) => None,
                    })
                    .and_then(|o| {
                        o.planned_arrival
                            .map(|o| o.with_timezone(&Local).format("%H:%M").to_string())
                    })
                    .to_value(),
                "departure-platform" => self
                    .intermediate_location
                    .borrow()
                    .as_ref()
                    .and_then(|o| match o {
                        rcore::IntermediateLocation::Stop(s) => Some(s),
                        rcore::IntermediateLocation::Railway(_) => None,
                    })
                    .and_then(|o| o.departure_platform.clone())
                    .to_value(),
                "arrival-platform" => self
                    .intermediate_location
                    .borrow()
                    .as_ref()
                    .and_then(|o| match o {
                        rcore::IntermediateLocation::Stop(s) => Some(s),
                        rcore::IntermediateLocation::Railway(_) => None,
                    })
                    .and_then(|o| o.arrival_platform.clone())
                    .to_value(),
                "planned-departure-platform" => self
                    .intermediate_location
                    .borrow()
                    .as_ref()
                    .and_then(|o| match o {
                        rcore::IntermediateLocation::Stop(s) => Some(s),
                        rcore::IntermediateLocation::Railway(_) => None,
                    })
                    .and_then(|o| o.planned_departure_platform.clone())
                    .to_value(),
                "planned-arrival-platform" => self
                    .intermediate_location
                    .borrow()
                    .as_ref()
                    .and_then(|o| match o {
                        rcore::IntermediateLocation::Stop(s) => Some(s),
                        rcore::IntermediateLocation::Railway(_) => None,
                    })
                    .and_then(|o| o.planned_arrival_platform.clone())
                    .to_value(),
                "is-stop" => self
                    .intermediate_location
                    .borrow()
                    .as_ref()
                    .map(|o| match o {
                        rcore::IntermediateLocation::Stop(_) => true,
                        rcore::IntermediateLocation::Railway(_) => false,
                    })
                    .unwrap_or_default()
                    .to_value(),
                _ => unimplemented!(),
            }
        }
    }
}
