use std::cell::RefCell;

use gdk::glib::Object;
use gdk::subclass::prelude::ObjectSubclassIsExt;

gtk::glib::wrapper! {
    pub struct Leg(ObjectSubclass<imp::Leg>);
}

impl Leg {
    pub fn new(leg: rcore::Leg) -> Self {
        let s: Self = Object::builder::<Self>().build();
        s.imp().leg.swap(&RefCell::new(Some(leg)));
        s
    }

    pub fn leg(&self) -> rcore::Leg {
        self.imp()
            .leg
            .borrow()
            .clone()
            .expect("Leg has not yet been set up")
    }
}

mod imp {
    use chrono::Local;
    use gtk::glib;
    use std::cell::RefCell;

    use gdk::{
        glib::Properties,
        prelude::ObjectExt,
        subclass::prelude::{
            DerivedObjectProperties, ObjectImpl, ObjectSubclass, ObjectSubclassExt,
        },
    };

    use crate::backend::{Frequency, LateFactor, LoadFactor, Place};

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::Leg)]
    pub struct Leg {
        #[property(name = "direction", type = String, get = Self::direction)]
        #[property(name = "name", type = String, get = Self::name)]
        #[property(name = "departure", type = Option<String>, get = Self::departure)]
        #[property(name = "arrival", type = Option<String>, get = Self::arrival)]
        #[property(name = "planned-departure", type = Option<String>, get = Self::planned_departure)]
        #[property(name = "planned-arrival", type = Option<String>, get = Self::planned_arrival)]
        #[property(name = "departure-platform", type = Option<String>, get = Self::departure_platform)]
        #[property(name = "arrival-platform", type = Option<String>, get = Self::arrival_platform)]
        #[property(name = "planned-departure-platform", type = Option<String>, get = Self::planned_departure_platform)]
        #[property(name = "planned-arrival-platform", type = Option<String>, get = Self::planned_arrival_platform)]
        #[property(name = "origin", type = Option<Place>, get = Self::origin)]
        #[property(name = "destination", type = Option<Place>, get = Self::destination)]
        #[property(name = "load-factor", type = LoadFactor, get = Self::load_factor, builder(LoadFactor::default()))]
        #[property(name = "late-factor", type = LateFactor, get = Self::late_factor, builder(LateFactor::default()))]
        #[property(name = "frequency", type = Option<Frequency>, get = Self::frequency)]
        #[property(name = "change-platform", type = bool, get = Self::change_platform)]
        #[property(name = "is-unreachable", type = bool, get = Self::is_unreachable)]
        #[property(name = "is-cancelled", type = bool, get = Self::is_cancelled)]
        pub(super) leg: RefCell<Option<rcore::Leg>>,
    }

    impl Leg {
        fn direction(&self) -> String {
            self.leg
                .borrow()
                .as_ref()
                .and_then(|o| o.direction.clone())
                .unwrap_or(
                    self.obj()
                        .destination()
                        .map(|d| d.name())
                        .unwrap_or_default(),
                )
        }

        fn name(&self) -> String {
            self.leg
                .borrow()
                .as_ref()
                .and_then(|o| o.line.clone())
                .and_then(|o| o.name.clone())
                .unwrap_or(gettextrs::gettext("Walk"))
        }

        fn departure(&self) -> Option<String> {
            self.leg
                .borrow()
                .as_ref()
                .and_then(|o| o.departure)
                .map(|d| d.with_timezone(&Local).format("%H:%M").to_string())
        }

        fn arrival(&self) -> Option<String> {
            self.leg
                .borrow()
                .as_ref()
                .and_then(|o| o.arrival)
                .map(|d| d.with_timezone(&Local).format("%H:%M").to_string())
        }

        fn planned_departure(&self) -> Option<String> {
            self.leg
                .borrow()
                .as_ref()
                .and_then(|o| o.planned_departure)
                .map(|d| d.with_timezone(&Local).format("%H:%M").to_string())
        }

        fn planned_arrival(&self) -> Option<String> {
            self.leg
                .borrow()
                .as_ref()
                .and_then(|o| o.planned_arrival)
                .map(|d| d.with_timezone(&Local).format("%H:%M").to_string())
        }

        fn departure_platform(&self) -> Option<String> {
            self.leg
                .borrow()
                .as_ref()
                .and_then(|o| o.departure_platform.clone())
        }

        fn arrival_platform(&self) -> Option<String> {
            self.leg
                .borrow()
                .as_ref()
                .and_then(|o| o.arrival_platform.clone())
        }

        fn planned_departure_platform(&self) -> Option<String> {
            self.leg
                .borrow()
                .as_ref()
                .and_then(|o| o.planned_departure_platform.clone())
        }

        fn planned_arrival_platform(&self) -> Option<String> {
            self.leg
                .borrow()
                .as_ref()
                .and_then(|o| o.planned_arrival_platform.clone())
        }

        fn origin(&self) -> Option<Place> {
            self.leg
                .borrow()
                .as_ref()
                .map(|o| Place::new(o.origin.clone()))
        }

        fn destination(&self) -> Option<Place> {
            self.leg
                .borrow()
                .as_ref()
                .map(|o| Place::new(o.destination.clone()))
        }

        fn load_factor(&self) -> LoadFactor {
            self.leg
                .borrow()
                .as_ref()
                .map(|o| LoadFactor::from(o.load_factor))
                .unwrap_or_default()
        }

        fn late_factor(&self) -> LateFactor {
            self.leg
                .borrow()
                .as_ref()
                .map(|o| {
                    std::cmp::max(
                        match (o.arrival, o.planned_arrival) {
                            (Some(real), Some(planned)) => LateFactor::from(real - planned),
                            _ => LateFactor::default(),
                        },
                        match (o.departure, o.planned_departure) {
                            (Some(real), Some(planned)) => LateFactor::from(real - planned),
                            _ => LateFactor::default(),
                        },
                    )
                })
                .unwrap_or_default()
        }

        fn frequency(&self) -> Option<Frequency> {
            self.leg
                .borrow()
                .as_ref()
                .and_then(|o| o.frequency.clone())
                .map(Frequency::new)
        }

        fn change_platform(&self) -> bool {
            self.leg
                .borrow()
                .as_ref()
                .map(|o| {
                    o.departure_platform != o.planned_departure_platform
                        || o.arrival_platform != o.planned_arrival_platform
                })
                .unwrap_or_default()
        }

        fn is_unreachable(&self) -> bool {
            self.leg
                .borrow()
                .as_ref()
                .map(|o| !o.reachable)
                .unwrap_or_default()
        }

        fn is_cancelled(&self) -> bool {
            self.leg
                .borrow()
                .as_ref()
                .map(|o| o.cancelled)
                .unwrap_or_default()
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Leg {
        const NAME: &'static str = "DBLeg";
        type Type = super::Leg;
    }

    #[glib::derived_properties]
    impl ObjectImpl for Leg {}
}
