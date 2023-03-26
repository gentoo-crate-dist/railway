use std::cell::RefCell;

use gdk::glib::Object;
use gdk::subclass::prelude::ObjectSubclassIsExt;

gtk::glib::wrapper! {
    pub struct Leg(ObjectSubclass<imp::Leg>);
}

impl Leg {
    pub fn new(leg: hafas_rs::Leg) -> Self {
        let s: Self = Object::builder::<Self>().build();
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
        glib::{ParamSpec, ParamSpecObject, ParamSpecString, Value},
        prelude::{ObjectExt, ParamSpecBuilderExt, ToValue},
        subclass::prelude::{ObjectImpl, ObjectSubclass, ObjectSubclassExt},
    };
    use once_cell::sync::Lazy;

    use crate::backend::Place;

    #[derive(Default)]
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
                    ParamSpecString::builder("direction").read_only().build(),
                    ParamSpecString::builder("name").read_only().build(),
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
                    ParamSpecObject::builder::<Place>("origin")
                        .read_only()
                        .build(),
                    ParamSpecObject::builder::<Place>("destination")
                        .read_only()
                        .build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, _value: &Value, _pspec: &ParamSpec) {}

        fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
            let obj = self.obj();
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
