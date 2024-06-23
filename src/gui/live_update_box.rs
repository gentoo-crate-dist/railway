use chrono::Local;
use gdk::{glib::subclass::types::ObjectSubclassIsExt, prelude::ObjectExt};

use crate::{backend::Event, gui::utility::Utility};

gtk::glib::wrapper! {
    pub struct LiveUpdateBox(ObjectSubclass<imp::LiveUpdateBox>)
        @extends libadwaita::Bin, gtk::Widget,
        @implements gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget;
}

impl LiveUpdateBox {
    fn set_event_internal(&self, event: Option<&Event>) {
        let now = Local::now();

        let page = match event {
            Some(Event::BeforeJourney(_)) => "before-journey",
            Some(Event::InLeg(_, _)) => "in-leg",
            Some(Event::TransitionTo(_)) => "transition-to",
            Some(Event::AfterJourney) => "after-journey",
            Some(Event::Unreachable) => "unreachable",
            Some(Event::Cancelled) => "cancelled",
            None => "before-journey", // Page does not matter; invisible anyways.
        };

        let current_leg = event.and_then(Event::current_leg);
        let next_leg = event.and_then(Event::next_leg);
        let time_of_next_action = event.and_then(Event::time_of_next_action);
        let duration_next = time_of_next_action.map(|t| t.with_timezone(&Local) - now);

        self.set_leg_current(current_leg);
        self.set_leg_next(next_leg);
        self.set_duration(duration_next.map(Utility::format_duration_tabular));

        self.imp().stack.set_visible_child_name(page);

        self.notify("before-journey-label");
        self.notify("in-leg-label");
        self.notify("transition-to-label");
    }
}

pub mod imp {
    use std::cell::RefCell;
    use std::marker::PhantomData;

    use gdk::glib::BoxedAnyObject;
    use gdk::glib::Properties;
    use glib::subclass::InitializingObject;
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::CompositeTemplate;
    use libadwaita::subclass::bin::BinImpl;

    use crate::backend::Leg;
    use crate::backend::Place;
    use crate::gui::utility::Utility;

    #[derive(CompositeTemplate, Default, Properties)]
    #[properties(wrapper_type = super::LiveUpdateBox)]
    #[template(resource = "/ui/live_update_box.ui")]
    pub struct LiveUpdateBox {
        #[template_child]
        pub(super) stack: TemplateChild<gtk::Stack>,

        #[property(get, set, nullable)]
        leg_current: RefCell<Option<Leg>>,
        #[property(get, set, nullable)]
        leg_next: RefCell<Option<Leg>>,
        #[property(get, set, nullable)]
        duration: RefCell<Option<String>>,

        #[property(set = Self::set_event, nullable)]
        event: PhantomData<Option<BoxedAnyObject>>,

        #[property(get = Self::before_journey_label, nullable)]
        before_journey_label: PhantomData<Option<String>>,
        #[property(get = Self::in_leg_label, nullable)]
        in_leg_label: PhantomData<Option<String>>,
        #[property(get = Self::transition_to_label, nullable)]
        transition_to_label: PhantomData<Option<String>>,
    }

    impl LiveUpdateBox {
        fn set_event(&self, event: Option<BoxedAnyObject>) {
            self.obj()
                .set_event_internal(event.as_ref().map(|o| o.borrow()).as_deref());
        }

        fn before_journey_label(&self) -> Option<String> {
            let obj = self.obj();

            let leg = obj.leg_next()?;
            let duration = obj.duration()?;

            let platform = leg.property::<Option<String>>("departure-platform");
            let station = leg.property::<Place>("origin").property::<String>("name");

            Some(
                match platform {
                    Some(platform) => gettextrs::gettext(
                        "Start at {station} in {duration} on platform {platform}",
                    )
                    .replace("{platform}", &platform),
                    _ => gettextrs::gettext("Start at {station} in {duration}"),
                }
                .replace("{station}", &station)
                .replace("{duration}", &duration),
            )
        }

        fn in_leg_label(&self) -> Option<String> {
            let obj = self.obj();

            let leg = obj.leg_current()?;
            let duration = obj.duration()?;

            let station = leg
                .property::<Place>("destination")
                .property::<String>("name");

            Some(
                gettextrs::gettext("Arriving at {station} in {duration}")
                    .replace("{station}", &station)
                    .replace("{duration}", &duration),
            )
        }

        fn transition_to_label(&self) -> Option<String> {
            let obj = self.obj();

            let leg = obj.leg_next()?;
            let duration = obj.duration()?;

            let leg_name = leg.property::<String>("name");
            let platform = leg.property::<Option<String>>("departure-platform");

            Some(
                match platform {
                    Some(platform) => gettextrs::gettext(
                        "Transition to {leg} on platform {platform} in {duration}",
                    )
                    .replace("{platform}", &platform),
                    _ => gettextrs::gettext("Transition to {leg} in {duration}"),
                }
                .replace("{leg}", &leg_name)
                .replace("{duration}", &duration),
            )
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for LiveUpdateBox {
        const NAME: &'static str = "DBLiveUpdateBox";
        type Type = super::LiveUpdateBox;
        type ParentType = libadwaita::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            Utility::bind_template_callbacks(klass);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for LiveUpdateBox {}

    impl WidgetImpl for LiveUpdateBox {}
    impl BinImpl for LiveUpdateBox {}
}
