use std::cell::RefCell;

use gdk::glib::Object;
use gdk::subclass::prelude::ObjectSubclassIsExt;
use hafas_rest::Stopover;

gtk::glib::wrapper! {
    pub struct StopoverObject(ObjectSubclass<imp::StopoverObject>);
}

impl StopoverObject {
    pub fn new(stopover: Stopover) -> Self {
        let s: Self = Object::new(&[]).expect("Failed to create `StopoverObject`.");
        s.imp().stopover.swap(&RefCell::new(Some(stopover)));
        s
    }
}

mod imp {
    use gtk::glib;
    use hafas_rest::Stopover;
    use std::cell::RefCell;

    use gdk::{
        glib::{ParamFlags, ParamSpec, ParamSpecObject, ParamSpecString, Value},
        prelude::{StaticType, ToValue},
        subclass::prelude::{ObjectImpl, ObjectSubclass},
    };
    use once_cell::sync::Lazy;

    use crate::gui::objects::StopObject;

    #[derive(Default, Clone)]
    pub struct StopoverObject {
        pub(super) stopover: RefCell<Option<Stopover>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for StopoverObject {
        const NAME: &'static str = "DBStopoverObject";
        type Type = super::StopoverObject;
    }

    impl ObjectImpl for StopoverObject {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecObject::new(
                        "stop",
                        "stop",
                        "stop",
                        StopObject::static_type(),
                        ParamFlags::READABLE,
                    ),
                    ParamSpecString::new(
                        "departure",
                        "departure",
                        "departure",
                        None,
                        ParamFlags::READABLE,
                    ),
                    ParamSpecString::new(
                        "arrival",
                        "arrival",
                        "arrival",
                        None,
                        ParamFlags::READABLE,
                    ),
                    ParamSpecString::new(
                        "planned-departure",
                        "planned-departure",
                        "planned-departure",
                        None,
                        ParamFlags::READABLE,
                    ),
                    ParamSpecString::new(
                        "planned-arrival",
                        "planned-arrival",
                        "planned-arrival",
                        None,
                        ParamFlags::READABLE,
                    ),
                    ParamSpecString::new(
                        "departure-platform",
                        "departure-platform",
                        "departure-platform",
                        None,
                        ParamFlags::READABLE,
                    ),
                    ParamSpecString::new(
                        "arrival-platform",
                        "arrival-platform",
                        "arrival-platform",
                        None,
                        ParamFlags::READABLE,
                    ),
                    ParamSpecString::new(
                        "planned-departure-platform",
                        "planned-departure-platform",
                        "planned-departure-platform",
                        None,
                        ParamFlags::READABLE,
                    ),
                    ParamSpecString::new(
                        "planned-arrival-platform",
                        "planned-arrival-platform",
                        "planned-arrival-platform",
                        None,
                        ParamFlags::READABLE,
                    ),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _obj: &Self::Type, _id: usize, _value: &Value, _pspec: &ParamSpec) {}

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "stop" => self
                    .stopover
                    .borrow()
                    .as_ref()
                    .map(|o| StopObject::new(o.stop.clone()))
                    .to_value(),
                "departure" => self
                    .stopover
                    .borrow()
                    .as_ref()
                    .map(|o| o.departure.map(|o| o.format("%H:%M").to_string()))
                    .flatten()
                    .to_value(),
                "arrival" => self
                    .stopover
                    .borrow()
                    .as_ref()
                    .map(|o| o.arrival.map(|o| o.format("%H:%M").to_string()))
                    .flatten()
                    .to_value(),
                "planned-departure" => self
                    .stopover
                    .borrow()
                    .as_ref()
                    .map(|o| o.planned_departure.map(|o| o.format("%H:%M").to_string()))
                    .flatten()
                    .to_value(),
                "planned-arrival" => self
                    .stopover
                    .borrow()
                    .as_ref()
                    .map(|o| o.planned_arrival.map(|o| o.format("%H:%M").to_string()))
                    .flatten()
                    .to_value(),
                "departure-platform" => self
                    .stopover
                    .borrow()
                    .as_ref()
                    .map(|o| o.departure_platform.clone())
                    .flatten()
                    .to_value(),
                "arrival-platform" => self
                    .stopover
                    .borrow()
                    .as_ref()
                    .map(|o| o.arrival_platform.clone())
                    .flatten()
                    .to_value(),
                "planned-departure-platform" => self
                    .stopover
                    .borrow()
                    .as_ref()
                    .map(|o| o.planned_departure_platform.clone())
                    .flatten()
                    .to_value(),
                "planned-arrival-platform" => self
                    .stopover
                    .borrow()
                    .as_ref()
                    .map(|o| o.planned_arrival_platform.clone())
                    .flatten()
                    .to_value(),
                _ => unimplemented!(),
            }
        }
    }
}
