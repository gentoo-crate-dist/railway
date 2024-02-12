use gdk::{prelude::ObjectExt, subclass::prelude::ObjectSubclassIsExt};

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
}

pub mod imp {
    use std::cell::Cell;
    use std::cell::RefCell;

    use chrono::Local;
    use gdk::glib::clone;
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
    use hafas_rs::api::refresh_journey::RefreshJourneyOptions;
    use once_cell::sync::Lazy;

    use chrono::Duration;

    use crate::backend::HafasClient;
    use crate::backend::Journey;
    use crate::backend::Leg;
    use crate::backend::Place;
    use crate::gui::leg_item::LegItem;
    use crate::gui::transition::Transition;
    use crate::gui::utility::Utility;
    use crate::gui::window::Window;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/ui/journey_detail_page.ui")]
    pub struct JourneyDetailPage {
        #[template_child]
        box_legs: TemplateChild<gtk::Box>,
        #[template_child]
        label_last_refreshed: TemplateChild<gtk::Label>,

        refresh_in_progress: Cell<bool>,

        journey: RefCell<Option<Journey>>,

        client: RefCell<Option<HafasClient>>,
    }

    impl JourneyDetailPage {
        pub(super) fn reload(&self, obj: &super::JourneyDetailPage) {
            let main_context = MainContext::default();
            let window = self.obj().root().and_downcast::<Window>()
                .expect("search page must be mapped and realised when a template callback is called");
            main_context.spawn_local(clone!(
                       @strong obj,
                       @strong window,
                       @strong self.journey as journey => async move {
                let token = journey.borrow().as_ref().and_then(|j| j.journey().refresh_token);

                if let Some(token) = token {
                    obj.set_refresh_in_progress(true);
                    let result_journey = obj.property::<HafasClient>("client")
                        .refresh_journey(token, RefreshJourneyOptions {
                            stopovers: Some(true),
                            language: Some(gettextrs::gettext("language")),
                            ..Default::default()
                        }).await;
                    if let Ok(result_journey) = result_journey {
                        obj.set_property("journey", result_journey);
                        obj.imp().update_last_refreshed();
                    } else {
                        window.display_error_toast(result_journey.expect_err("A error"));
                    }
                    obj.set_refresh_in_progress(false);
                }
            }));
        }

        fn update_last_refreshed(&self) {
            self.label_last_refreshed.set_label(&gettextrs::gettext("Last refreshed {}")
                .replace("{}", &Utility::format_time_human(&Local::now().time())
            ))
        }
    }

    #[template_callbacks]
    impl JourneyDetailPage {
        #[template_callback(function)]
        fn format_source_destination(source: &str, destination: &str) -> String {
            format!("{source} → {destination}")
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
                    ParamSpecObject::builder::<HafasClient>("client").build(),
                    ParamSpecBoolean::builder("refresh-in-progress").build(),
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

                    // Clear box_legs if journey has completely changed (i.e. completely new journey with different legs).
                    // Different journeys can be identified by different refresh tokens.
                    if obj.as_ref().and_then(|j| j.refresh_token())
                        != self
                            .journey
                            .borrow()
                            .as_ref()
                            .and_then(|j| j.refresh_token())
                    {
                        while let Some(child) = self.box_legs.first_child() {
                            self.box_legs.remove(&child);
                        }
                    }

                    let mut current_child = self.box_legs.first_child();

                    // Fill box_legs
                    let legs = obj.as_ref().map(|j| j.journey().legs).unwrap_or_default();
                    let mut i = 0;
                    while i < legs.len() {
                        let mut walking_time: Option<Duration> = None;
                        let is_start = i == 0;
                        let i_start = i;
                        let mut to = &legs[i];

                        while to.walking.unwrap_or(false) {
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
                            let has_walk = walking_time.is_some()
                                || (!is_end && to.origin != legs[i_start].origin);

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
                                        self.box_legs.remove(&c);
                                    }

                                    self.box_legs.append(&Transition::new(
                                        &walking_time,
                                        &waiting_time,
                                        has_walk,
                                        is_start || is_end,
                                        &final_station,
                                    ));
                                }
                            } else {
                                // There is no child left, append a new transition.
                                self.box_legs.append(&Transition::new(
                                    &walking_time,
                                    &waiting_time,
                                    has_walk,
                                    is_start || is_end,
                                    &final_station,
                                ));
                            }

                            current_child = current_child.and_then(|c| c.next_sibling());
                        }

                        if !is_end {
                            if let Some(child) = &current_child {
                                if let Some(leg_item) = child.dynamic_cast_ref::<LegItem>() {
                                    // There is already a leg item in the correct place.
                                    leg_item.set_leg(&Leg::new(legs[i].clone()));
                                } else {
                                    // There is something there, but it is no leg item. Clear the box from here to the end and insert a new transition.
                                    while let Some(c) = current_child {
                                        current_child = c.next_sibling();
                                        self.box_legs.remove(&c);
                                    }

                                    self.box_legs
                                        .append(&LegItem::new(&Leg::new(legs[i].clone())));
                                }
                            } else {
                                // There is no child left, append a new leg item.
                                self.box_legs
                                    .append(&LegItem::new(&Leg::new(legs[i].clone())));
                            }

                            current_child = current_child.and_then(|c| c.next_sibling());
                        }

                        i += 1;
                    }

                    // Remove the remaining children.
                    while let Some(c) = current_child {
                        current_child = c.next_sibling();
                        self.box_legs.remove(&c);
                    }

                    self.journey.replace(obj);
                }
                "refresh-in-progress" => {
                    let obj = value.get::<bool>().expect(
                        "Property `refresh-in-progress` of `JourneyDetailPage` has to be of type `bool`",
                    );

                    self.refresh_in_progress.replace(obj);
                }
                "client" => {
                    let obj = value.get::<Option<HafasClient>>().expect(
                        "Property `client` of `JourneyDetailPage` has to be of type `HafasClient`",
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
                "client" => self.client.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for JourneyDetailPage {}
    impl BoxImpl for JourneyDetailPage {}
}
