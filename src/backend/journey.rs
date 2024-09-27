use std::cell::{Ref, RefCell};
use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Datelike, Duration, Local};
use chrono_tz::Tz;
use gdk::gio::{Application, Notification};
use gdk::glib::{clone, BoxedAnyObject, Object};
use gdk::prelude::{ApplicationExt, ObjectExt};
use gdk::subclass::prelude::{ObjectImpl, ObjectSubclassIsExt};
use rcore::RefreshJourneyOptions;
use serde::{Deserialize, Serialize};

use crate::gui::utility::Utility;
use crate::Error;

use super::{Client, Leg, Place};

gtk::glib::wrapper! {
    pub struct Journey(ObjectSubclass<imp::Journey>);
}

impl Journey {
    pub fn new(journey: rcore::Journey, client: &Client) -> Self {
        let s: Self = Object::builder().property("client", client).build();
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

    pub async fn refresh(&self) -> Result<(), Error> {
        log::debug!("Refreshing journey {}", self.id());
        self.set_refresh_in_progress(true);
        let result = self
            .property::<Client>("client")
            .refresh_journey(
                self,
                RefreshJourneyOptions {
                    stopovers: true,
                    language: Some(Utility::language_code()),
                    ..Default::default()
                },
            )
            .await;
        self.set_refresh_in_progress(false);

        if result.is_ok() {
            self.update_last_refreshed();
        }

        result.map(|_| ())
    }

    fn set_refresh_in_progress(&self, b: bool) {
        self.set_property("refresh-in-progress", b)
    }

    fn update_last_refreshed(&self) {
        *self.imp().last_refreshed.borrow_mut() = Some(Local::now());
        self.notify("last-refreshed");
    }

    pub fn last_refreshed(&self) -> Option<DateTime<Local>> {
        *self.imp().last_refreshed.borrow()
    }

    fn should_refresh(&self) -> bool {
        let current_event = self.property::<BoxedAnyObject>("current-event");
        let current_event: Ref<Event> = current_event.borrow();

        let Some(duration_next_event) = current_event
            .time_of_next_action()
            .map(|t| t.with_timezone(&Local) - Local::now())
        else {
            return false;
        };

        let Some(time_since_last_refreshed) = self
            .last_refreshed()
            .map(|r| Local::now().signed_duration_since(r))
        else {
            return true;
        };

        (duration_next_event < Duration::days(1) && time_since_last_refreshed > Duration::hours(1))
            || (duration_next_event < Duration::hours(1)
                && time_since_last_refreshed > Duration::minutes(15))
            || (duration_next_event < Duration::minutes(15)
                && time_since_last_refreshed > Duration::minutes(5))
            || (duration_next_event < Duration::minutes(5)
                && time_since_last_refreshed > Duration::minutes(1))
    }

    pub fn next_background_tasks_in(&self) -> Duration {
        let current_event = self.property::<BoxedAnyObject>("current-event");
        let current_event: Ref<Event> = current_event.borrow();

        let Some(duration_next_event) = current_event
            .time_of_next_action()
            .map(|t| t.with_timezone(&Local) - Local::now())
        else {
            return Duration::zero();
        };

        if duration_next_event < Duration::minutes(5) {
            Duration::minutes(1)
        } else if duration_next_event < Duration::minutes(15) {
            Duration::minutes(5)
        } else if duration_next_event < Duration::hours(1) {
            Duration::minutes(15)
        } else if duration_next_event < Duration::days(1) {
            Duration::hours(1)
        } else {
            Duration::days(1)
        }
    }

    pub fn background_tasks(&self) {
        log::debug!("Running background task on journey {}", self.id());
        gspawn!(clone!(
            #[strong(rename_to = s)]
            self,
            async move {
                if s.should_refresh() {
                    // Ignore errors refreshing.
                    // XXX: Show them maybe?
                    let _ = s.refresh().await;
                }
                let now = Local::now();
                *s.imp().current_event.borrow_mut() = Some(s.event_at_time(&now));
                *s.imp().alerts.borrow_mut() = s.alerts(&now);
                s.notify("current-event");
                s.notify("alerts");
                s.potentially_notify();
            }
        ));
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
                } else if leg.arrival.is_some_and(|a| time < &a) && !leg.walking {
                    return Event::InLeg(
                        Leg::new(leg.clone()),
                        legs.iter()
                            .skip(i + 1)
                            .find(|l| !l.walking)
                            .map(|l| Leg::new(l.clone())),
                    );
                }
            }

            Event::AfterJourney
        }
    }

    pub fn alerts(&self, time: &DateTime<Local>) -> Vec<Alert> {
        let mut alerts = vec![];

        let journey = self.imp().journey.borrow();
        let legs = &journey.as_ref().expect("Journey to be set").legs;
        for leg in legs {
            // Only add an alert for future legs
            if leg.departure.is_some_and(|d| time < &d) {
                if leg.departure != leg.planned_departure {
                    alerts.push(Alert::DepartureDelayed(Leg::new(leg.clone())))
                }
                if leg.departure_platform != leg.planned_departure_platform {
                    alerts.push(Alert::DeparturePlatformChange(Leg::new(leg.clone())))
                }
                if leg.arrival != leg.planned_arrival {
                    alerts.push(Alert::ArrivalDelayed(Leg::new(leg.clone())))
                }
            }
            if leg.arrival.is_some_and(|a| time < &a) && leg.arrival != leg.planned_arrival {
                alerts.push(Alert::ArrivalDelayed(Leg::new(leg.clone())))
            }
        }

        alerts
    }

    pub fn potentially_notify(&self) {
        let mut notify_status = self.imp().notify_status.borrow_mut();
        let app = Application::default().expect("Application to be active");

        let current_event = self.property::<BoxedAnyObject>("current-event");
        let current_event: Ref<Event> = current_event.borrow();

        let duration_next_event = current_event
            .time_of_next_action()
            .map(|t| t.with_timezone(&Local) - Local::now());

        let notification = match &*current_event {
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
                        .contains(&l_current.leg().id()) =>
            {
                notify_status
                    .in_leg_soon_transition
                    .insert(l_current.leg().id());
                if let Some(l_next) = l_next {
                    let station = l_current
                        .property::<Place>("destination")
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
            app.send_notification(
                Some(&format!("railway-journey-current-event-{}", self.id())),
                &notification,
            );
        }

        let alerts = self.property::<BoxedAnyObject>("alerts");
        let alerts: Ref<Vec<Alert>> = alerts.borrow();

        for alert in &*alerts {
            match alert {
                Alert::DepartureDelayed(leg_obj) => {
                    let leg = leg_obj.leg();
                    let Some(delay) = leg
                        .departure
                        .and_then(|d| leg.planned_departure.map(|p| d - p))
                    else {
                        log::warn!("Departure delayed, but cannot compute the delay. Ignoring");
                        continue;
                    };
                    let notified_delay = notify_status
                        .departure_delays
                        .get(&leg.id())
                        .copied()
                        .unwrap_or_default();

                    if delay - Duration::minutes(notified_delay) >= Duration::minutes(5) {
                        notify_status
                            .departure_delays
                            .insert(leg.id(), delay.num_minutes());
                        app.send_notification(
                            Some(&format!("railway-leg-departure-delayed-{}", leg.id())),
                            &Notification::new(
                                &gettextrs::gettext("{train} departure is delayed by {time}")
                                    .replace("{train}", &leg_obj.property::<String>("name"))
                                    .replace("{time}", &Utility::format_duration_inline(delay)),
                            ),
                        );
                    }
                }
                Alert::ArrivalDelayed(leg_obj) => {
                    let leg = leg_obj.leg();
                    let Some(delay) = leg.arrival.and_then(|d| leg.planned_arrival.map(|p| d - p))
                    else {
                        log::warn!("Arrival delayed, but cannot compute the delay. Ignoring");
                        continue;
                    };
                    let notified_delay = notify_status
                        .arrival_delays
                        .get(&leg.id())
                        .copied()
                        .unwrap_or_default();

                    if delay - Duration::minutes(notified_delay) >= Duration::minutes(5) {
                        notify_status
                            .arrival_delays
                            .insert(leg.id(), delay.num_minutes());
                        app.send_notification(
                            Some(&format!("railway-leg-arrival-delayed-{}", leg.id())),
                            &Notification::new(
                                &gettextrs::gettext("{train} arrival is delayed by {time}")
                                    .replace("{train}", &leg_obj.property::<String>("name"))
                                    .replace("{time}", &Utility::format_duration_inline(delay)),
                            ),
                        );
                    }
                }
                Alert::DeparturePlatformChange(leg_obj) => {
                    let leg = leg_obj.leg();
                    let notified_platform = notify_status.departure_platform_changes.get(&leg.id());
                    let platform: String = leg_obj.property("departure-platform");

                    if !notified_platform.is_some_and(|p| *p == platform) {
                        notify_status
                            .departure_platform_changes
                            .insert(leg.id(), platform.clone());
                        app.send_notification(
                            Some(&format!(
                                "railway-leg-departure-platform-changed-{}",
                                leg.id()
                            )),
                            &Notification::new(
                                &gettextrs::gettext(
                                    "{train} arrival is departuring on platform {platform} today",
                                )
                                .replace("{train}", &leg_obj.property::<String>("name"))
                                .replace("{platform}", &platform),
                            ),
                        );
                    }
                }
            }
        }
    }

    pub fn set_notify_status(&self, status: NotifyStatus) {
        *self.imp().notify_status.borrow_mut() = status;
    }

    pub fn notify_status(&self) -> NotifyStatus {
        (*self.imp().notify_status.borrow()).clone()
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

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Alert {
    DepartureDelayed(Leg),
    ArrivalDelayed(Leg),
    DeparturePlatformChange(Leg),
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

    pub fn format_at_time(&self, time: DateTime<Local>) -> String {
        let current_leg = self.current_leg();
        let next_leg = self.next_leg();
        let time_of_next_action = self.time_of_next_action();
        let duration = time_of_next_action
            .map(|t| t.with_timezone(&Local) - time)
            .map(Utility::format_duration_tabular);

        let next_leg_name = next_leg.map(|l| l.property::<String>("name"));
        let next_platform =
            next_leg.and_then(|l| l.property::<Option<String>>("departure-platform"));
        let next_origin =
            next_leg.map(|l| l.property::<Place>("origin").property::<String>("name"));
        let current_destination = current_leg.map(|l| {
            l.property::<Place>("destination")
                .property::<String>("name")
        });

        match (
            self,
            &duration,
            &next_leg_name,
            &next_platform,
            &next_origin,
            &current_destination,
        ) {
            (Event::BeforeJourney(_), Some(_), _, Some(_), Some(_), _) => gettextrs::gettext(
                "Start at {next-origin} in {duration} on platform {next-platform}",
            ),
            (Event::BeforeJourney(_), Some(_), _, None, Some(_), _) => {
                gettextrs::gettext("Start at {next-origin} in {duration}")
            }
            (Event::InLeg(_, _), Some(_), _, _, _, Some(_)) => {
                gettextrs::gettext("Arriving at {current-destination} in {duration}")
            }
            (Event::TransitionTo(_), Some(_), Some(_), Some(_), _, _) => gettextrs::gettext(
                "Transition to {next-leg-name} on platform {next-platform} in {duration}",
            ),
            (Event::TransitionTo(_), Some(_), Some(_), None, _, _) => {
                gettextrs::gettext("Transition to {next-leg-name} in {duration}")
            }
            (Event::AfterJourney, _, _, _, _, _) => {
                gettextrs::gettext("The journey arrived at its destination")
            }
            (Event::Cancelled, _, _, _, _, _) => gettextrs::gettext("The journey was cancelled"),
            (Event::Unreachable, _, _, _, _, _) => {
                gettextrs::gettext("A connection is unreachable")
            }
            // The remaining cases should never happen
            _ => "".to_string(),
        }
        .replace("{duration}", &duration.unwrap_or_default())
        .replace("{next-leg-name}", &next_leg_name.unwrap_or_default())
        .replace("{next-platform}", &next_platform.unwrap_or_default())
        .replace("{next-origin}", &next_origin.unwrap_or_default())
        .replace(
            "{current-destination}",
            &current_destination.unwrap_or_default(),
        )
    }
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct NotifyStatus {
    beginning_of_journey: bool,
    in_leg_soon_transition: HashSet<String>,
    unreachable: bool,
    cancelled: bool,
    departure_delays: HashMap<String, i64>,
    arrival_delays: HashMap<String, i64>,
    departure_platform_changes: HashMap<String, String>,
}

mod imp {
    use chrono::{DateTime, Local};
    use gtk::glib;
    use once_cell::sync::Lazy;
    use std::cell::{Cell, RefCell};

    use crate::{backend::Client, gui::utility::Utility};

    use gdk::{
        glib::{
            BoxedAnyObject, ParamSpec, ParamSpecBoolean, ParamSpecEnum, ParamSpecObject,
            ParamSpecString, Value,
        },
        prelude::{ParamSpecBuilderExt, ToValue},
        subclass::prelude::{ObjectImpl, ObjectSubclass, ObjectSubclassExt},
    };

    use crate::backend::{LateFactor, Leg, LoadFactor, Price};

    use super::{Alert, Event, NotifyStatus};

    pub struct Journey {
        pub(super) journey: RefCell<Option<rcore::Journey>>,

        pub(super) current_event: RefCell<Option<Event>>,
        pub(super) alerts: RefCell<Vec<Alert>>,
        pub(super) notify_status: RefCell<NotifyStatus>,

        pub(super) last_refreshed: RefCell<Option<DateTime<Local>>>,
        refresh_in_progress: Cell<bool>,

        pub(super) client: RefCell<Option<Client>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Journey {
        const NAME: &'static str = "DBJourney";
        type Type = super::Journey;

        fn new() -> Self {
            Self {
                journey: RefCell::default(),
                current_event: Default::default(),
                alerts: Default::default(),
                notify_status: Default::default(),
                refresh_in_progress: Default::default(),
                last_refreshed: Default::default(),
                client: Default::default(),
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
                    ParamSpecObject::builder::<BoxedAnyObject>("alerts")
                        .read_only()
                        .build(),
                    ParamSpecString::builder("last-refreshed")
                        .read_only()
                        .build(),
                    ParamSpecBoolean::builder("refresh-in-progress").build(),
                    ParamSpecObject::builder::<Client>("client").build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "refresh-in-progress" => {
                    let obj = value.get::<bool>().expect(
                        "Property `refresh-in-progress` of `JourneyDetailPage` has to be of type `bool`",
                    );

                    self.refresh_in_progress.replace(obj);
                }
                "client" => {
                    let obj = value
                        .get::<Option<Client>>()
                        .expect("Property `client` of `JourneysPage` has to be of type `Client`");

                    self.client.replace(obj);
                }
                _ => unimplemented!(),
            }
        }

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
                "alerts" => {
                    let alerts = self.alerts.borrow();
                    BoxedAnyObject::new(alerts.clone()).into()
                }
                "refresh-in-progress" => self.refresh_in_progress.get().to_value(),
                "last-refreshed" => self
                    .last_refreshed
                    .borrow()
                    .as_ref()
                    .map(|t| Utility::format_time_human(&t.time()))
                    .to_value(),
                "client" => self.client.borrow().as_ref().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}
