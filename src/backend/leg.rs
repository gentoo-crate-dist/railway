use std::cell::RefCell;

use gdk::glib::Object;
use gdk::subclass::prelude::ObjectSubclassIsExt;

gtk::glib::wrapper! {
    pub struct Leg(ObjectSubclass<imp::Leg>);
}

impl Leg {
    pub fn new(leg: hafas_rs::Leg) -> Self {
        let s: Self = Object::new(&[]).expect("Failed to create `Leg`.");
        s.imp().leg.swap(&RefCell::new(Some(leg)));
        s
    }

    pub fn leg(&self) -> hafas_rs::Leg {
        self.imp()
            .leg
            .borrow()
            .clone()
            .expect("Leg has not yet been set up")
    }
}

mod imp {
    use gtk::glib;
    use std::cell::RefCell;

    use gdk::{
        glib::{ParamFlags, ParamSpec, ParamSpecObject, ParamSpecString, Value},
        prelude::{ObjectExt, StaticType, ToValue},
        subclass::prelude::{ObjectImpl, ObjectSubclass},
    };
    use once_cell::sync::Lazy;

    use crate::backend::Place;

    #[derive(Default, Clone)]
    pub struct Leg {
        pub(super) leg: RefCell<Option<hafas_rs::Leg>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Leg {
        const NAME: &'static str = "DBLeg";
        type Type = super::Leg;
    }

    impl ObjectImpl for Leg {
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
                        Place::static_type(),
                        ParamFlags::READABLE,
                    ),
                    ParamSpecObject::new(
                        "destination",
                        "destination",
                        "destination",
                        Place::static_type(),
                        ParamFlags::READABLE,
                    ),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _obj: &Self::Type, _id: usize, _value: &Value, _pspec: &ParamSpec) {}

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "direction" => self
                    .leg
                    .borrow()
                    .as_ref()
                    .map(|o| o.direction.as_ref())
                    .flatten()
                    .unwrap_or(
                        &obj.property::<Place>("destination")
                            .name()
                            .unwrap_or_default(),
                    )
                    .to_value(),
                "name" => self
                    .leg
                    .borrow()
                    .as_ref()
                    .map(|o| o.line.as_ref())
                    .flatten()
                    .and_then(|o| o.name.as_ref())
                    .unwrap_or(&gettextrs::gettext("Walk"))
                    .to_value(),
                "departure" => self
                    .leg
                    .borrow()
                    .as_ref()
                    .map(|o| o.departure)
                    .flatten()
                    .map(|d| d.format("%H:%M").to_string())
                    .to_value(),
                "arrival" => self
                    .leg
                    .borrow()
                    .as_ref()
                    .map(|o| o.arrival)
                    .flatten()
                    .map(|d| d.format("%H:%M").to_string())
                    .to_value(),
                "planned-departure" => self
                    .leg
                    .borrow()
                    .as_ref()
                    .map(|o| o.planned_departure)
                    .flatten()
                    .map(|d| d.format("%H:%M").to_string())
                    .to_value(),
                "planned-arrival" => self
                    .leg
                    .borrow()
                    .as_ref()
                    .map(|o| o.planned_arrival)
                    .flatten()
                    .map(|d| d.format("%H:%M").to_string())
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
                    .map(|o| Place::new(o.origin.clone()))
                    .to_value(),
                "destination" => self
                    .leg
                    .borrow()
                    .as_ref()
                    .map(|o| Place::new(o.destination.clone()))
                    .to_value(),
                _ => unimplemented!(),
            }
        }
    }
}
