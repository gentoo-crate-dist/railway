use std::cell::RefCell;

use gdk::glib::Object;
use gdk::subclass::prelude::ObjectSubclassIsExt;
use hafas_rest::Leg;

gtk::glib::wrapper! {
    pub struct LegObject(ObjectSubclass<imp::LegObject>);
}

impl LegObject {
    pub fn new(leg: Leg) -> Self {
        let s: Self = Object::new(&[]).expect("Failed to create `LegObject`.");
        s.imp().leg.swap(&RefCell::new(Some(leg)));
        s
    }

    pub fn leg(&self) -> Leg {
        self.imp()
            .leg
            .borrow()
            .clone()
            .expect("LegObject has not yet been set up")
    }
}

mod imp {
    use gtk::glib;
    use hafas_rest::Leg;
    use std::cell::RefCell;

    use gdk::{
        glib::{ParamFlags, ParamSpec, ParamSpecObject, ParamSpecString, Value},
        prelude::{StaticType, ToValue},
        subclass::prelude::{ObjectImpl, ObjectSubclass},
    };
    use once_cell::sync::Lazy;

    use crate::gui::objects::StopObject;

    #[derive(Default, Clone)]
    pub struct LegObject {
        pub(super) leg: RefCell<Option<Leg>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for LegObject {
        const NAME: &'static str = "DBLegObject";
        type Type = super::LegObject;
    }

    impl ObjectImpl for LegObject {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecString::new(
                        "direction",
                        "direction",
                        "direction",
                        None,
                        ParamFlags::READABLE,
                    ),
                    ParamSpecString::new("name", "name", "name", None, ParamFlags::READABLE),
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
                    ParamSpecObject::new(
                        "origin",
                        "origin",
                        "origin",
                        StopObject::static_type(),
                        ParamFlags::READABLE,
                    ),
                    ParamSpecObject::new(
                        "destination",
                        "destination",
                        "destination",
                        StopObject::static_type(),
                        ParamFlags::READABLE,
                    ),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _obj: &Self::Type, _id: usize, _value: &Value, _pspec: &ParamSpec) {}

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "direction" => self
                    .leg
                    .borrow()
                    .as_ref()
                    .map(|o| o.direction.as_ref())
                    .flatten()
                    .to_value(),
                "name" => self
                    .leg
                    .borrow()
                    .as_ref()
                    .map(|o| o.line.as_ref())
                    .flatten()
                    .map(|o| &o.name)
                    .to_value(),
                "departure" => self
                    .leg
                    .borrow()
                    .as_ref()
                    .map(|o| o.departure.format("%H:%M").to_string())
                    .to_value(),
                "arrival" => self
                    .leg
                    .borrow()
                    .as_ref()
                    .map(|o| o.arrival.format("%H:%M").to_string())
                    .to_value(),
                "planned-departure" => self
                    .leg
                    .borrow()
                    .as_ref()
                    .map(|o| o.planned_departure.format("%H:%M").to_string())
                    .to_value(),
                "planned-arrival" => self
                    .leg
                    .borrow()
                    .as_ref()
                    .map(|o| o.planned_arrival.format("%H:%M").to_string())
                    .to_value(),
                "departure-platform" => self
                    .leg
                    .borrow()
                    .as_ref()
                    .map(|o| o.departure_platform.clone())
                    .flatten()
                    .to_value(),
                "arrival-platform" => self
                    .leg
                    .borrow()
                    .as_ref()
                    .map(|o| o.arrival_platform.clone())
                    .flatten()
                    .to_value(),
                "planned-departure-platform" => self
                    .leg
                    .borrow()
                    .as_ref()
                    .map(|o| o.planned_departure_platform.clone())
                    .flatten()
                    .to_value(),
                "planned-arrival-platform" => self
                    .leg
                    .borrow()
                    .as_ref()
                    .map(|o| o.planned_arrival_platform.clone())
                    .flatten()
                    .to_value(),
                "origin" => self
                    .leg
                    .borrow()
                    .as_ref()
                    .map(|o| StopObject::new(o.origin.clone()))
                    .to_value(),
                "destination" => self
                    .leg
                    .borrow()
                    .as_ref()
                    .map(|o| StopObject::new(o.destination.clone()))
                    .to_value(),
                _ => unimplemented!(),
            }
        }
    }
}
