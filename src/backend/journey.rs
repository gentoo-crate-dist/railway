use std::cell::RefCell;

use gdk::glib::Object;
use gdk::subclass::prelude::ObjectSubclassIsExt;

gtk::glib::wrapper! {
    pub struct Journey(ObjectSubclass<imp::Journey>);
}

impl Journey {
    pub fn new(journey: hafas_rs::Journey) -> Self {
        let s: Self = Object::builder().build();
        s.imp().journey.swap(&RefCell::new(Some(journey)));
        s
    }

    pub fn journey(&self) -> hafas_rs::Journey {
        self.imp()
            .journey
            .borrow()
            .as_ref()
            .expect("Journey has not yet been set up")
            .clone()
    }
}

mod imp {
    use gtk::glib;
    use once_cell::sync::Lazy;
    use std::cell::RefCell;

    use chrono::NaiveDate;

    use gdk::{
        glib::{
            ParamSpec, ParamSpecBoolean, ParamSpecEnum, ParamSpecObject, ParamSpecString, Value,
        },
        prelude::{ParamSpecBuilderExt, ToValue},
        subclass::prelude::{ObjectImpl, ObjectSubclass},
    };

    use crate::backend::{LateFactor, Leg, LoadFactor};

    pub struct Journey {
        pub(super) journey: RefCell<Option<hafas_rs::Journey>>,
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
                    ParamSpecString::builder("price").read_only().build(),
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
                    .map(|p| format!("{:.2} {}", p.amount, p.currency))
                    .to_value(),
                "first-leg" => self
                    .journey
                    .borrow()
                    .as_ref()
                    .and_then(|o| o.legs.get(0))
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
                        let needed_time = arrival - departure;

                        (NaiveDate::from_ymd_opt(2022, 1, 1)
                            .unwrap_or_default()
                            .and_hms_opt(0, 0, 0)
                            .unwrap_or_default()
                            + needed_time)
                            .format("%H:%M")
                            .to_string()
                            .to_value()
                    } else {
                        "".to_string().to_value()
                    }
                }
                "transitions" => self
                    .journey
                    .borrow()
                    .as_ref()
                    .map(|o| (o.legs.len() - 1) as u32)
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
                    .map(|o| o.legs.iter().flat_map(|l| l.reachable).any(|b| !b))
                    .unwrap_or_default()
                    .to_value(),
                "is-cancelled" => self
                    .journey
                    .borrow()
                    .as_ref()
                    .map(|o| o.legs.iter().flat_map(|l| l.cancelled).any(|b| b))
                    .unwrap_or_default()
                    .to_value(),
                _ => unimplemented!(),
            }
        }
    }
}
