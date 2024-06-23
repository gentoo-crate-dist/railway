use std::cell::{Ref, RefCell};

use chrono::{DateTime, Datelike, Duration, Local};
use chrono_tz::Tz;
use gdk::gio::{Application, Notification};
use gdk::glib::{BoxedAnyObject, Object};
use gdk::prelude::{ApplicationExt, ObjectExt};
use gdk::subclass::prelude::{ObjectImpl, ObjectSubclassIsExt};

use crate::gui::utility::Utility;

use super::{Leg, Place};

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

    pub fn background_tasts(&self) {
        // TODO: Potentially reload
        *self.imp().current_event.borrow_mut() = Some(self.event_at_time(&Local::now()));
        self.notify("current-event");
        self.potentially_notify();
    }

    pub fn event_at_time(&self, time: &DateTime<Local>) -> Event {
        if self.property("is-unreachable") {
            Event::Unreachable
        } else if self.property("is-cancelled") {
            Event::Cancelled
        } else {
            let journey = self.imp().journey.borrow();
            let legs = &journey.as_ref().expect("Journey to be set").legs;

            for (i, leg) in legs.iter().enumerate() {
                if leg.departure.is_some_and(|d| time < &d) {
                    if i == 0 {
                        return Event::BeforeJourney(Leg::new(leg.clone()));
                    } else {
                        return Event::TransitionTo(Leg::new(leg.clone()));
                    }
                } else if leg.arrival.is_some_and(|a| time < &a) {
                    return Event::InLeg(
                        Leg::new(leg.clone()),
                        legs.get(i + 1).map(|l| Leg::new(l.clone())),
                    );
                }
            }

            Event::AfterJourney
        }
    }

    pub fn potentially_notify(&self) {
        let mut notify_status = self.imp().notify_status.borrow_mut();

        let current_event = self.property::<BoxedAnyObject>("current-event");
        let current_event: Ref<Event> = current_event.borrow();

        let duration_next_event = current_event
            .time_of_next_action()
            .map(|t| t.with_timezone(&Local) - Local::now());

        let notification = match &*current_event {
            // TODO: Notify on significant delay.
            // TODO: Notify on platform change.
            // TODO: Update notification if already exists, e.g. if delayed or platform changed.
            // TODO: Settings for durations
            Event::BeforeJourney(l)
                if duration_next_event.is_some_and(|d| d < Duration::hours(1))
                    && !notify_status.beginning_of_journey =>
            {
                notify_status.beginning_of_journey = true;

                let origin = l.property::<Place>("origin").property::<String>("name");
                let destination = self
                    .property::<Leg>("last-leg")
                    .property::<Place>("destination")
                    .property::<String>("name");
                let platform = l.property::<Option<String>>("departure-platform");
                let time = l.property::<String>("departure");

                let body = if let Some(platform) = platform {
                    gettextrs::gettext("Start in {station} on platform {platform} at {time}")
                        .replace("{platform}", &platform)
                } else {
                    gettextrs::gettext("Start in {station} at {time}")
                }
                .replace("{station}", &origin)
                .replace("{time}", &time);

                let notification = Notification::new(
                    &gettextrs::gettext("Your trip to {destination} starts soon")
                        .replace("{destination}", &destination),
                );
                notification.set_body(Some(&body));
                Some(notification)
            }
            Event::InLeg(l_current, l_next)
                if duration_next_event.is_some_and(|d| d < Duration::minutes(5))
                    && !notify_status
                        .in_leg_soon_transition
                        .as_ref()
                        .is_some_and(|l| l.leg() == l_current.leg()) =>
            {
                notify_status.in_leg_soon_transition = Some(l_current.clone());
                if let Some(l_next) = l_next {
                    let station = l_current
                        .property::<Place>("origin")
                        .property::<String>("name");
                    let platform = l_next.property::<Option<String>>("departure-platform");
                    let duration = l_next
                        .leg()
                        .departure
                        .and_then(|d| l_current.leg().arrival.map(|a| d - a))
                        .map(Utility::format_duration_inline);

                    let body = match (platform, duration) {
                        (Some(platform), Some(duration)) => Some(
                            gettextrs::gettext("You have {time} to get to platform {platform}")
                                .replace("{time}", &duration)
                                .replace("{platform}", &platform),
                        ),
                        (None, Some(duration)) => Some(
                            gettextrs::gettext("You have {time} for the transition")
                                .replace("{time}", &duration),
                        ),
                        (Some(platform), _) => Some(
                            gettextrs::gettext("Transition to platform {platform}")
                                .replace("{platform}", &platform),
                        ),
                        _ => None,
                    };

                    let notification = Notification::new(
                        &gettextrs::gettext("You need to transition in {station} soon")
                            .replace("{station}", &station),
                    );
                    notification.set_body(body.as_deref());
                    Some(notification)
                } else {
                    let destination = l_current
                        .property::<Place>("destination")
                        .property::<String>("name");
                    Some(Notification::new(
                        &gettextrs::gettext("You will arrive in {destination} soon")
                            .replace("{destination}", &destination),
                    ))
                }
            }
            Event::Unreachable if !notify_status.unreachable => {
                notify_status.unreachable = true;
                Some(Notification::new(&gettextrs::gettext(
                    "Connection Unreachable",
                )))
            }
            Event::Cancelled if !notify_status.cancelled => {
                notify_status.cancelled = true;
                Some(Notification::new(&gettextrs::gettext(
                    "The journey was cancelled",
                )))
            }
            _ => None,
        };

        if let Some(notification) = notification {
            let app = Application::default().expect("Application to be active");
            app.send_notification(
                Some(&format!("railway-journey-{}", self.id())),
                &notification,
            );
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Event {
    BeforeJourney(Leg),
    InLeg(Leg, Option<Leg>),
    TransitionTo(Leg),
    AfterJourney,
    Unreachable,
    Cancelled,
}

impl Event {
    pub fn current_leg(&self) -> Option<&Leg> {
        match self {
            Event::InLeg(l, _) => Some(l),
            _ => None,
        }
    }

    pub fn next_leg(&self) -> Option<&Leg> {
        match self {
            Event::BeforeJourney(l) => Some(l),
            Event::InLeg(_, l) => l.as_ref(),
            Event::TransitionTo(l) => Some(l),
            _ => None,
        }
    }

    pub fn time_of_next_action(&self) -> Option<DateTime<Tz>> {
        match self {
            Event::BeforeJourney(l) => l.leg().departure,
            Event::InLeg(l, _) => l.leg().arrival,
            Event::TransitionTo(l) => l.leg().departure,
            _ => None,
        }
    }
}

#[derive(Default, Debug)]
struct NotifyStatus {
    beginning_of_journey: bool,
    in_leg_soon_transition: Option<Leg>,
    unreachable: bool,
    cancelled: bool,
}

mod imp {
    use chrono::Local;
    use gtk::glib;
    use once_cell::sync::Lazy;
    use std::cell::RefCell;

    use crate::gui::utility::Utility;

    use gdk::{
        glib::{
            BoxedAnyObject, ParamSpec, ParamSpecBoolean, ParamSpecEnum, ParamSpecObject,
            ParamSpecString, Value,
        },
        prelude::{ParamSpecBuilderExt, ToValue},
        subclass::prelude::{ObjectImpl, ObjectSubclass, ObjectSubclassExt},
    };

    use crate::backend::{LateFactor, Leg, LoadFactor, Price};

    use super::{Event, NotifyStatus};

    pub struct Journey {
        pub(super) journey: RefCell<Option<rcore::Journey>>,

        pub(super) current_event: RefCell<Option<Event>>,
        pub(super) notify_status: RefCell<NotifyStatus>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Journey {
        const NAME: &'static str = "DBJourney";
        type Type = super::Journey;

        fn new() -> Self {
            Self {
                journey: RefCell::default(),
                current_event: Default::default(),
                notify_status: Default::default(),
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
                    ParamSpecObject::builder::<BoxedAnyObject>("current-event")
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
                "current-event" => {
                    let mut event = self.current_event.borrow_mut();
                    if event.is_none() {
                        *event = Some(self.obj().event_at_time(&Local::now()));
                    }
                    let event = event.clone().unwrap();

                    BoxedAnyObject::new(event).into()
                }
                _ => unimplemented!(),
            }
        }
    }
}
