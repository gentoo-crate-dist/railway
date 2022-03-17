use std::cell::RefCell;

use gdk::glib::Object;
use gdk::subclass::prelude::ObjectSubclassIsExt;
use hafas_rest::Journey;

gtk::glib::wrapper! {
    pub struct JourneyObject(ObjectSubclass<imp::JourneyObject>);
}

impl JourneyObject {
    pub fn new(journey: Journey) -> Self {
        let s: Self = Object::new(&[]).expect("Failed to create `JourneyObject`.");
        s.imp().journey.swap(&RefCell::new(Some(journey)));
        s
    }

    pub fn journey(&self) -> Journey {
        self.imp()
            .journey
            .borrow()
            .as_ref()
            .expect("JourneyObject has not yet been set up")
            .clone()
    }
}

mod imp {
    use gtk::glib;
    use hafas_rest::Journey;
    use once_cell::sync::Lazy;
    use std::cell::RefCell;

    use chrono::NaiveDate;

    use gdk::{
        glib::{ParamFlags, ParamSpec, ParamSpecObject, ParamSpecString, Value},
        prelude::{StaticType, ToValue},
        subclass::prelude::{ObjectImpl, ObjectSubclass},
    };

    use crate::gui::objects::LegObject;

    #[derive(Default, Clone)]
    pub struct JourneyObject {
        pub(super) journey: RefCell<Option<Journey>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for JourneyObject {
        const NAME: &'static str = "DBJourneyObject";
        type Type = super::JourneyObject;
    }

    impl ObjectImpl for JourneyObject {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecObject::new(
                        "first-leg",
                        "first-leg",
                        "first-leg",
                        LegObject::static_type(),
                        ParamFlags::READABLE,
                    ),
                    ParamSpecObject::new(
                        "last-leg",
                        "last-leg",
                        "last-leg",
                        LegObject::static_type(),
                        ParamFlags::READABLE,
                    ),
                    ParamSpecString::new(
                        "total-time",
                        "total-time",
                        "total-time",
                        None,
                        ParamFlags::READABLE,
                    ),
                    ParamSpecString::new(
                        "transitions",
                        "transitions",
                        "transitions",
                        None,
                        ParamFlags::READABLE,
                    ),
                    ParamSpecString::new("types", "types", "types", None, ParamFlags::READABLE),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _obj: &Self::Type, _id: usize, _value: &Value, _pspec: &ParamSpec) {}

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "first-leg" => self
                    .journey
                    .borrow()
                    .as_ref()
                    .map(|o| o.legs.get(0).clone())
                    .flatten()
                    .map(|o| LegObject::new(o.clone()))
                    .to_value(),
                "last-leg" => self
                    .journey
                    .borrow()
                    .as_ref()
                    .map(|o| o.legs.last().clone())
                    .flatten()
                    .map(|o| LegObject::new(o.clone()))
                    .to_value(),
                "total-time" => {
                    let journey_borrow = self.journey.borrow();
                    let journey = journey_borrow.as_ref();
                    let leg_first = journey.map(|o| o.legs.first().clone()).flatten();
                    let leg_last = journey.map(|o| o.legs.last().clone()).flatten();

                    let departure = leg_first.map(|o| o.departure);
                    let arrival = leg_last.map(|o| o.arrival);

                    if departure.is_none() || arrival.is_none() {
                        "".to_string().to_value()
                    } else {
                        let needed_time = arrival.unwrap() - departure.unwrap();

                        (NaiveDate::from_ymd(2022, 1, 1).and_hms(0, 0, 0) + needed_time)
                            .format("%H:%M")
                            .to_string()
                            .to_value()
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
                                    .map(|l| l.product_name.clone())
                                    .unwrap_or_default()
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
