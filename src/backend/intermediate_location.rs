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
        glib::Properties,
        prelude::ObjectExt,
        subclass::prelude::{DerivedObjectProperties, ObjectImpl, ObjectSubclass},
    };

    use crate::backend::Place;

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::IntermediateLocation)]
    pub struct IntermediateLocation {
        #[property(name = "place", type = Place, get = Self::place)]
        #[property(name = "departure", type = Option<String>, get = Self::departure)]
        #[property(name = "arrival", type = Option<String>, get = Self::arrival)]
        #[property(name = "planned-departure", type = Option<String>, get = Self::planned_departure)]
        #[property(name = "planned-arrival", type = Option<String>, get = Self::planned_arrival)]
        #[property(name = "departure-platform", type = Option<String>, get = Self::departure_platform)]
        #[property(name = "arrival-platform", type = Option<String>, get = Self::arrival_platform)]
        #[property(name = "planned-departure-platform", type = Option<String>, get = Self::planned_departure_platform)]
        #[property(name = "planned-arrival-platform", type = Option<String>, get = Self::planned_arrival_platform)]
        #[property(name = "is-stop", type = bool, get = Self::is_stop)]
        pub(super) intermediate_location: RefCell<Option<rcore::IntermediateLocation>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for IntermediateLocation {
        const NAME: &'static str = "DBIntermediateLocation";
        type Type = super::IntermediateLocation;
    }

    impl IntermediateLocation {
        fn place(&self) -> Place {
            self.intermediate_location
                .borrow()
                .as_ref()
                .map(|o| {
                    Place::new(match o {
                        rcore::IntermediateLocation::Stop(s) => s.place.clone(),
                        rcore::IntermediateLocation::Railway(r) => r.clone(),
                    })
                })
                .expect("IntermediateLocation to be set to query the place")
        }

        fn departure(&self) -> Option<String> {
            self.intermediate_location
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
        }

        fn arrival(&self) -> Option<String> {
            self.intermediate_location
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
        }

        fn planned_departure(&self) -> Option<String> {
            self.intermediate_location
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
        }

        fn planned_arrival(&self) -> Option<String> {
            self.intermediate_location
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
        }

        fn departure_platform(&self) -> Option<String> {
            self.intermediate_location
                .borrow()
                .as_ref()
                .and_then(|o| match o {
                    rcore::IntermediateLocation::Stop(s) => Some(s),
                    rcore::IntermediateLocation::Railway(_) => None,
                })
                .and_then(|o| o.departure_platform.clone())
        }

        fn arrival_platform(&self) -> Option<String> {
            self.intermediate_location
                .borrow()
                .as_ref()
                .and_then(|o| match o {
                    rcore::IntermediateLocation::Stop(s) => Some(s),
                    rcore::IntermediateLocation::Railway(_) => None,
                })
                .and_then(|o| o.arrival_platform.clone())
        }

        fn planned_departure_platform(&self) -> Option<String> {
            self.intermediate_location
                .borrow()
                .as_ref()
                .and_then(|o| match o {
                    rcore::IntermediateLocation::Stop(s) => Some(s),
                    rcore::IntermediateLocation::Railway(_) => None,
                })
                .and_then(|o| o.planned_departure_platform.clone())
        }

        fn planned_arrival_platform(&self) -> Option<String> {
            self.intermediate_location
                .borrow()
                .as_ref()
                .and_then(|o| match o {
                    rcore::IntermediateLocation::Stop(s) => Some(s),
                    rcore::IntermediateLocation::Railway(_) => None,
                })
                .and_then(|o| o.planned_arrival_platform.clone())
        }

        fn is_stop(&self) -> bool {
            self.intermediate_location
                .borrow()
                .as_ref()
                .map(|o| match o {
                    rcore::IntermediateLocation::Stop(_) => true,
                    rcore::IntermediateLocation::Railway(_) => false,
                })
                .unwrap_or_default()
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for IntermediateLocation {}
}
