use std::cell::RefCell;

use chrono::{Datelike, Local};
use gdk::glib::Object;
use gdk::prelude::ObjectExt;
use gdk::subclass::prelude::{ObjectImpl, ObjectSubclassIsExt};

use crate::gui::utility::Utility;

gtk::glib::wrapper! {
    pub struct Journey(ObjectSubclass<imp::Journey>);
}

impl Journey {
    pub fn new(journey: rcore::Journey) -> Self {
        let s: Self = Object::builder().build();
        s.imp().journey.swap(&RefCell::new(Some(journey)));
        s
    }

    pub fn journey(&self) -> rcore::Journey {
        self.imp()
            .journey
            .borrow()
            .as_ref()
            .expect("Journey has not yet been set up")
            .clone()
    }

    pub fn update(&self, journey: rcore::Journey) {
        *self.imp().journey.borrow_mut() = Some(journey);

        for prop in imp::Journey::properties() {
            self.notify_by_pspec(prop);
        }
    }

    pub fn is_unreachable(&self) -> bool {
        self.property("is-unreachable")
    }

    pub fn is_cancelled(&self) -> bool {
        self.property("is-cancelled")
    }

    pub fn day_timestamp(&self) -> u32 {
        self.property::<super::Leg>("first-leg")
            .leg()
            .departure
            .map(|d| d.ordinal())
            .unwrap_or_default()
    }

    pub fn departure_day(&self) -> String {
        self.property::<super::Leg>("first-leg")
            .leg()
            .departure
            .map(|d| Utility::format_date_human(d.with_timezone(&Local).date_naive()))
            .unwrap_or_default()
    }

    pub fn id(&self) -> String {
        self.journey().id.clone()
    }
}

mod imp {
    use gtk::glib;
    use once_cell::sync::Lazy;
    use std::cell::RefCell;

    use crate::gui::utility::Utility;

    use gdk::{
        glib::{
            ParamSpec, ParamSpecBoolean, ParamSpecEnum, ParamSpecObject, ParamSpecString, Value,
        },
        prelude::{ParamSpecBuilderExt, ToValue},
        subclass::prelude::{ObjectImpl, ObjectSubclass},
    };

    use crate::backend::{LateFactor, Leg, LoadFactor, Price};

    pub struct Journey {
        pub(super) journey: RefCell<Option<rcore::Journey>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Journey {
        const NAME: &'static str = "DBJourney";
        type Type = super::Journey;

        fn new() -> Self {
            Self {
                journey: RefCell::default(),
            }
        }
    }

    impl ObjectImpl for Journey {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecObject::builder::<Price>("price")
                        .read_only()
                        .build(),
                    ParamSpecObject::builder::<Leg>("first-leg")
                        .read_only()
                        .build(),
                    ParamSpecObject::builder::<Leg>("last-leg")
                        .read_only()
                        .build(),
                    ParamSpecString::builder("total-time").read_only().build(),
                    ParamSpecString::builder("transitions").read_only().build(),
                    ParamSpecString::builder("types").read_only().build(),
                    ParamSpecEnum::builder::<LoadFactor>("load-factor")
                        .read_only()
                        .build(),
                    ParamSpecEnum::builder::<LateFactor>("late-factor")
                        .read_only()
                        .build(),
                    ParamSpecBoolean::builder("change-platform")
                        .read_only()
                        .build(),
                    ParamSpecBoolean::builder("is-unreachable")
                        .read_only()
                        .build(),
                    ParamSpecBoolean::builder("is-cancelled")
                        .read_only()
                        .build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, _value: &Value, _pspec: &ParamSpec) {}

        fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "price" => self
                    .journey
                    .borrow()
                    .as_ref()
                    .and_then(|o| o.price.as_ref())
                    .map(|p| Price::new(p.clone()))
                    .to_value(),
                "first-leg" => self
                    .journey
                    .borrow()
                    .as_ref()
                    .and_then(|o| o.legs.first())
                    .map(|o| Leg::new(o.clone()))
                    .to_value(),
                "last-leg" => self
                    .journey
                    .borrow()
                    .as_ref()
                    .and_then(|o| o.legs.last())
                    .map(|o| Leg::new(o.clone()))
                    .to_value(),
                "total-time" => {
                    let journey_borrow = self.journey.borrow();
                    let journey = journey_borrow.as_ref();
                    let leg_first = journey.and_then(|o| o.legs.first());
                    let leg_last = journey.and_then(|o| o.legs.last());

                    let departure = leg_first.and_then(|o| o.departure);
                    let arrival = leg_last.and_then(|o| o.arrival);

                    if let (Some(arrival), Some(departure)) = (arrival, departure) {
                        Utility::format_duration_tabular(arrival - departure).to_value()
                    } else {
                        "".to_string().to_value()
                    }
                }
                "transitions" => self
                    .journey
                    .borrow()
                    .as_ref()
                    .map(|o| {
                        o.legs
                            .iter()
                            .filter(|leg| !leg.walking)
                            .collect::<Vec<_>>()
                            .len()
                            .saturating_sub(1) as u32
                    })
                    .unwrap_or_default()
                    .to_value(),
                "types" => self
                    .journey
                    .borrow()
                    .as_ref()
                    .map(|o| {
                        o.legs
                            .iter()
                            .filter_map(|l| {
                                l.line.as_ref().map(|l| {
                                    l.product_name
                                        .clone()
                                        .unwrap_or_else(|| l.product.name.to_string())
                                })
                            })
                            .collect::<Vec<String>>()
                            .join(" • ")
                    })
                    .unwrap_or_default()
                    .to_value(),
                "load-factor" => self
                    .journey
                    .borrow()
                    .as_ref()
                    .and_then(|o| o.legs.iter().map(|l| LoadFactor::from(l.load_factor)).max())
                    .unwrap_or_default()
                    .to_value(),
                "late-factor" => self
                    .journey
                    .borrow()
                    .as_ref()
                    .and_then(|o| {
                        o.legs
                            .iter()
                            .map(|l| {
                                std::cmp::max(
                                    match (l.arrival, l.planned_arrival) {
                                        (Some(real), Some(planned)) => {
                                            LateFactor::from(real - planned)
                                        }
                                        _ => LateFactor::default(),
                                    },
                                    match (l.departure, l.planned_departure) {
                                        (Some(real), Some(planned)) => {
                                            LateFactor::from(real - planned)
                                        }
                                        _ => LateFactor::default(),
                                    },
                                )
                            })
                            .max()
                    })
                    .unwrap_or_default()
                    .to_value(),
                "change-platform" => self
                    .journey
                    .borrow()
                    .as_ref()
                    .map(|o| {
                        o.legs
                            .iter()
                            .map(|l| {
                                l.departure_platform != l.planned_departure_platform
                                    || l.arrival_platform != l.planned_arrival_platform
                            })
                            .any(|b| b)
                    })
                    .unwrap_or_default()
                    .to_value(),
                "is-unreachable" => self
                    .journey
                    .borrow()
                    .as_ref()
                    .map(|o| o.legs.iter().map(|l| l.reachable).any(|b| !b))
                    .unwrap_or_default()
                    .to_value(),
                "is-cancelled" => self
                    .journey
                    .borrow()
                    .as_ref()
                    .map(|o| o.legs.iter().map(|l| l.cancelled).any(|b| b))
                    .unwrap_or_default()
                    .to_value(),
                _ => unimplemented!(),
            }
        }
    }
}
