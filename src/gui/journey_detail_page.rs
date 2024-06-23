use gdk::{prelude::ObjectExt, subclass::prelude::ObjectSubclassIsExt};

use crate::backend::Journey;

gtk::glib::wrapper! {
    pub struct JourneyDetailPage(ObjectSubclass<imp::JourneyDetailPage>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget;
}

impl JourneyDetailPage {
    pub fn reload(&self) {
        self.imp().reload(self);
    }

    fn set_refresh_in_progress(&self, b: bool) {
        self.set_property("refresh-in-progress", b)
    }

    fn journey(&self) -> Option<Journey> {
        self.property("journey")
    }
}

pub mod imp {
    use std::cell::Cell;
    use std::cell::RefCell;

    use chrono::Local;
    use gdk::glib::clone;
    use gdk::glib::JoinHandle;
    use gdk::glib::MainContext;
    use gdk::glib::ParamSpec;
    use gdk::glib::ParamSpecBoolean;
    use gdk::glib::ParamSpecObject;
    use gdk::glib::Value;
    use glib::subclass::InitializingObject;
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::template_callbacks;
    use gtk::CompositeTemplate;
    use once_cell::sync::Lazy;
    use rcore::RefreshJourneyOptions;

    use chrono::Duration;

    use crate::backend::Client;
    use crate::backend::Journey;
    use crate::backend::Leg;
    use crate::backend::Place;
    use crate::gui::leg_item::LegItem;
    use crate::gui::live_update_box::LiveUpdateBox;
    use crate::gui::transition::Transition;
    use crate::gui::utility::Utility;
    use crate::gui::window::Window;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/ui/journey_detail_page.ui")]
    pub struct JourneyDetailPage {
        #[template_child]
        box_legs: TemplateChild<gtk::Box>,
        #[template_child]
        pub(super) update_box: TemplateChild<LiveUpdateBox>,
        #[template_child]
        label_last_refreshed: TemplateChild<gtk::Label>,

        refresh_in_progress: Cell<bool>,
        show_live_box: Cell<bool>,

        journey: RefCell<Option<Journey>>,

        load_handle: RefCell<Option<JoinHandle<()>>>,

        client: RefCell<Option<Client>>,
    }

    impl JourneyDetailPage {
        pub(super) fn reload(&self, obj: &super::JourneyDetailPage) {
            let main_context = MainContext::default();
            let window = self.obj().root().and_downcast::<Window>().expect(
                "search page must be mapped and realised when a template callback is called",
            );
            main_context.spawn_local(clone!(
                #[strong]
                obj,
                #[strong]
                window,
                async move {
                    let journey = obj.journey();

                    if let Some(journey) = journey {
                        obj.set_refresh_in_progress(true);
                        let result_journey = obj
                            .property::<Client>("client")
                            .refresh_journey(
                                &journey,
                                RefreshJourneyOptions {
                                    stopovers: true,
                                    language: Some(Utility::language_code()),
                                    ..Default::default()
                                },
                            )
                            .await;
                        if let Ok(result_journey) = result_journey {
                            obj.set_property("journey", result_journey);
                            obj.imp().update_last_refreshed();
                        } else {
                            window.display_error_toast(result_journey.expect_err("A error"));
                        }
                        obj.set_refresh_in_progress(false);
                    }
                }
            ));
        }

        fn update_last_refreshed(&self) {
            self.label_last_refreshed.set_label(
                &gettextrs::gettext("Last refreshed {}")
                    .replace("{}", &Utility::format_time_human(&Local::now().time())),
            )
        }
    }

    #[template_callbacks]
    impl JourneyDetailPage {
        #[template_callback(function)]
        fn format_source_destination(source: &str, destination: &str) -> String {
            format!("{source} → {destination}")
        }

        async fn setup(&self, redo: bool) {
            enum SetupStep {
                Add(gtk::Widget),
                Remove(gtk::Widget),
            }

            // Plan setup steps.

            let mut steps = Vec::new();

            let mut current_child = self.box_legs.first_child();

            // Clear box_legs if journey has completely changed (i.e. completely new journey with different legs).
            if redo {
                while let Some(child) = current_child {
                    current_child = child.next_sibling();
                    steps.push(SetupStep::Remove(child));
                }
            }

            let legs = self
                .journey
                .borrow()
                .as_ref()
                .map(|j| j.journey().legs)
                .unwrap_or_default();

            // Fill box_legs
            let mut i = 0;
            while i < legs.len() {
                let mut walking_time: Option<Duration> = None;
                let is_start = i == 0;
                let i_start = i;
                let mut to = &legs[i];

                while to.walking {
                    walking_time = Some(
                        walking_time.unwrap_or(Duration::zero())
                            + to.arrival.map_or(Duration::zero(), |arrival| {
                                arrival - to.departure.unwrap_or(arrival)
                            }),
                    );
                    i += 1;
                    if i < legs.len() {
                        to = &legs[i];
                    } else {
                        break;
                    }
                }

                let is_end = i == legs.len();

                let waiting_time: Option<Duration> = if !is_start && !is_end {
                    let from = &legs[i - 1];
                    if to.departure.is_some() && from.arrival.is_some() {
                        Some(to.departure.unwrap() - from.arrival.unwrap())
                    } else {
                        None
                    }
                } else {
                    None
                };

                if walking_time.is_some() || !is_start {
                    let final_station = if is_end {
                        Some(Place::new(to.destination.clone()))
                    } else {
                        None
                    };
                    let has_walk =
                        walking_time.is_some() || (!is_end && to.origin != legs[i_start].origin);

                    if let Some(child) = &current_child {
                        if let Some(transition) = child.dynamic_cast_ref::<Transition>() {
                            // There is already a transition in the correct place.
                            transition.setup(
                                &walking_time,
                                &waiting_time,
                                has_walk,
                                is_start || is_end,
                                &final_station,
                            )
                        } else {
                            // There is something there, but it is no transition. Clear the box from here to the end and insert a new transition.
                            while let Some(c) = current_child {
                                current_child = c.next_sibling();
                                steps.push(SetupStep::Remove(c))
                            }

                            steps.push(SetupStep::Add(
                                Transition::new(
                                    &walking_time,
                                    &waiting_time,
                                    has_walk,
                                    is_start || is_end,
                                    &final_station,
                                )
                                .upcast(),
                            ));
                        }
                    } else {
                        // There is no child left, append a new transition.
                        steps.push(SetupStep::Add(
                            Transition::new(
                                &walking_time,
                                &waiting_time,
                                has_walk,
                                is_start || is_end,
                                &final_station,
                            )
                            .upcast(),
                        ));
                    }

                    current_child = current_child.and_then(|c| c.next_sibling());
                }

                // Even though we are in glib runtime now, `yield_now` is runtime-agnostic and also seems to work with glib.
                tokio::task::yield_now().await;

                if !is_end {
                    if let Some(child) = &current_child {
                        if let Some(leg_item) = child.dynamic_cast_ref::<LegItem>() {
                            // There is already a leg item in the correct place.
                            leg_item.set_leg(&Leg::new(legs[i].clone()));
                        } else {
                            // There is something there, but it is no leg item. Clear the box from here to the end and insert a new transition.
                            while let Some(c) = current_child {
                                current_child = c.next_sibling();
                                // self.box_legs.remove(&c);
                                steps.push(SetupStep::Remove(c));
                            }

                            steps.push(SetupStep::Add(
                                LegItem::new(&Leg::new(legs[i].clone())).upcast(),
                            ));
                        }
                    } else {
                        // There is no child left, append a new leg item.
                        steps.push(SetupStep::Add(
                            LegItem::new(&Leg::new(legs[i].clone())).upcast(),
                        ));
                    }

                    current_child = current_child.and_then(|c| c.next_sibling());
                }

                i += 1;
            }

            // Remove the remaining children.
            while let Some(c) = current_child {
                current_child = c.next_sibling();
                steps.push(SetupStep::Remove(c));
            }

            // Execute setup steps

            for step in steps {
                match step {
                    SetupStep::Add(w) => self.box_legs.append(&w),
                    SetupStep::Remove(w) => self.box_legs.remove(&w),
                }
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for JourneyDetailPage {
        const NAME: &'static str = "DBJourneyDetailPage";
        type Type = super::JourneyDetailPage;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            Self::bind_template_callbacks(klass);
            Utility::bind_template_callbacks(klass);
            crate::backend::Leg::ensure_type();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for JourneyDetailPage {
        fn constructed(&self) {
            self.parent_constructed();
        }

        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecObject::builder::<Journey>("journey").build(),
                    ParamSpecObject::builder::<Client>("client").build(),
                    ParamSpecBoolean::builder("refresh-in-progress").build(),
                    ParamSpecBoolean::builder("show-live-box").build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "journey" => {
                    let obj = value.get::<Option<Journey>>().expect(
                        "Property `journey` of `JourneyDetailPage` has to be of type `Journey`",
                    );

                    // Different journeys can be identified by different refresh tokens.
                    let redo = obj.as_ref().map(|j| j.id())
                        != self.journey.borrow().as_ref().map(|j| j.id());

                    self.journey.replace(obj);

                    // Ensure the load is not called twice at the same time by aborting the old one if needed.
                    if let Some(handle) = self.load_handle.replace(None) {
                        handle.abort();
                    }

                    let o = self.obj().clone();
                    let handle = gspawn!(
                        async move { o.imp().setup(redo).await },
                        glib::Priority::LOW
                    );

                    self.load_handle.replace(Some(handle));
                }
                "refresh-in-progress" => {
                    let obj = value.get::<bool>().expect(
                        "Property `refresh-in-progress` of `JourneyDetailPage` has to be of type `bool`",
                    );

                    self.refresh_in_progress.replace(obj);
                }
                "show-live-box" => {
                    let obj = value.get::<bool>().expect(
                        "Property `show-live-box` of `JourneyDetailPage` has to be of type `bool`",
                    );

                    self.show_live_box.replace(obj);
                }
                "client" => {
                    let obj = value.get::<Option<Client>>().expect(
                        "Property `client` of `JourneyDetailPage` has to be of type `Client`",
                    );

                    self.client.replace(obj);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "journey" => self.journey.borrow().to_value(),
                "refresh-in-progress" => self.refresh_in_progress.get().to_value(),
                "show-live-box" => self.show_live_box.get().to_value(),
                "client" => self.client.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for JourneyDetailPage {}
    impl BoxImpl for JourneyDetailPage {}
}
