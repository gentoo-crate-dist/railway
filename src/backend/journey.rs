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
        glib::{ParamSpec, ParamSpecObject, ParamSpecString, Value},
        prelude::{ParamSpecBuilderExt, ToValue},
        subclass::prelude::{ObjectImpl, ObjectSubclass},
    };

    use crate::backend::Leg;

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
                    .map(|o| o.price.as_ref())
                    .flatten()
                    .map(|p| format!("{:.2} {}", p.amount, p.currency))
                    .to_value(),
                "first-leg" => self
                    .journey
                    .borrow()
                    .as_ref()
                    .map(|o| o.legs.get(0))
                    .flatten()
                    .map(|o| Leg::new(o.clone()))
                    .to_value(),
                "last-leg" => self
                    .journey
                    .borrow()
                    .as_ref()
                    .map(|o| o.legs.last())
                    .flatten()
                    .map(|o| Leg::new(o.clone()))
                    .to_value(),
                "total-time" => {
                    let journey_borrow = self.journey.borrow();
                    let journey = journey_borrow.as_ref();
                    let leg_first = journey.map(|o| o.legs.first()).flatten();
                    let leg_last = journey.map(|o| o.legs.last()).flatten();

                    let departure = leg_first.map(|o| o.departure).flatten();
                    let arrival = leg_last.map(|o| o.arrival).flatten();

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
                            .map(|l| {
                                l.line
                                    .as_ref()
                                    .map(|l| {
                                        l.product_name
                                            .clone()
                                            .unwrap_or_else(|| l.product.name.to_string())
                                    })
                                    .unwrap_or(gettextrs::gettext("Walk"))
                            })
                            .collect::<Vec<String>>()
                            .join(" - ")
                    })
                    .unwrap_or_default()
                    .to_value(),
                _ => unimplemented!(),
            }
        }
    }
}
