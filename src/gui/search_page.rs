use gdk::subclass::prelude::ObjectSubclassIsExt;

use crate::backend::Journey;

gtk::glib::wrapper! {
    pub struct SearchPage(ObjectSubclass<imp::SearchPage>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget;
}

impl SearchPage {
    pub fn add_journey_store(&self, journey: Journey) {
        self.imp().add_journey_store(journey);
    }

    pub fn remove_journey_store(&self, journey: Journey) {
        self.imp().remove_journey_store(journey);
    }

    pub fn add_search_store(&self, origin: String, destination: String) {
        self.imp().add_search_store(origin, destination);
    }

    pub fn remove_search_store(&self, origin: String, destination: String) {
        self.imp().remove_search_store(origin, destination);
    }
}

pub mod imp {
    use chrono::Duration;
    use gdk::gio::Settings;
    use gdk::glib::clone;
    use gdk::glib::subclass::Signal;
    use gdk::glib::MainContext;
    use gdk::glib::Properties;
    use glib::subclass::InitializingObject;
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::CompositeTemplate;
    use gtk::ListBoxRow;
    use once_cell::sync::Lazy;
    use rcore::JourneysOptions;
    use rcore::LoyaltyCard;
    use rcore::TariffClass;
    use rcore::TransferOptions;

    use std::cell::Cell;
    use std::cell::RefCell;

    use crate::backend::Client;
    use crate::backend::Journey;
    use crate::backend::JourneysResult;
    use crate::backend::Place;
    use crate::backend::TimeType;
    use crate::config;
    use crate::gui::date_time_picker::DateTimePicker;
    use crate::gui::journey_store_item::JourneyStoreItem;
    use crate::gui::search_store_item::SearchStoreItem;
    use crate::gui::station_entry::StationEntry;
    use crate::gui::utility::Utility;
    use crate::gui::window::Window;

    #[derive(CompositeTemplate, Properties)]
    #[properties(wrapper_type = super::SearchPage)]
    #[template(resource = "/ui/search_page.ui")]
    pub struct SearchPage {
        #[template_child]
        in_from: TemplateChild<StationEntry>,
        #[template_child]
        in_to: TemplateChild<StationEntry>,

        #[template_child]
        pick_date_time: TemplateChild<DateTimePicker>,

        #[template_child]
        btn_search: TemplateChild<gtk::Button>,
        #[template_child]
        stack_search_btn: TemplateChild<gtk::Stack>,

        #[template_child]
        box_journeys: TemplateChild<gtk::ListBox>,
        #[template_child]
        box_searches: TemplateChild<gtk::ListBox>,

        settings: Settings,
        #[property(get, set)]
        client: RefCell<Option<Client>>,
        #[property(get, set)]
        search_when_ready: Cell<bool>,
        #[property(get, set)]
        searching: Cell<bool>,
    }

    #[gtk::template_callbacks]
    impl SearchPage {
        #[template_callback]
        fn handle_journeys_row_activated(&self, row: ListBoxRow) {
            self.obj().emit_by_name::<()>(
                "details",
                &[&row
                    .first_child()
                    .expect("Activated row to have a child")
                    .property::<Journey>("journey")],
            );
        }

        #[template_callback]
        fn handle_searches_row_activated(&self, row: ListBoxRow) {
            let search = &row.first_child().expect("Activated row to have a child");
            self.in_from.set_input(search.property::<String>("origin"));
            self.in_to
                .set_input(search.property::<String>("destination"));
            self.obj().set_search_when_ready(true);
        }

        fn update_search_button(&self) {
            let from_set = self.in_from.property::<bool>("set");
            let to_set = self.in_to.property::<bool>("set");
            let searching = self.searching.get();

            let tooltip = match (from_set, to_set, searching) {
                (false, false, _) => Some(gettextrs::gettext("Start and destination are missing")),
                (false, true, _) => Some(gettextrs::gettext("Start is missing")),
                (true, false, _) => Some(gettextrs::gettext("Destination is missing")),
                (true, true, true) => Some(gettextrs::gettext("Search ongoing")),
                (_, _, _) => None,
            };

            self.btn_search.set_tooltip_text(tooltip.as_deref());

            if self.obj().searching() {
                self.stack_search_btn.set_visible_child_name("spinner");
            } else {
                self.stack_search_btn.set_visible_child_name("label");
            }
        }

        fn setup_search_when_ready(&self) {
            let obj = self.obj();
            self.btn_search.connect_sensitive_notify(clone!(
                #[weak]
                obj,
                move |_| {
                    let s = obj.imp();
                    if s.btn_search.is_sensitive() && obj.search_when_ready() {
                        s.handle_search();
                    }
                    // XXX: Not the best place to unset search when ready. It would be better to do that when the station entry changed manually, but that will not work as it also changes when directly set.
                    obj.set_search_when_ready(false);
                }
            ));
        }

        pub(super) fn add_journey_store(&self, journey: Journey) {
            let item = JourneyStoreItem::new(journey);
            self.box_journeys.append(&item);
            self.box_journeys.set_visible(true);
        }

        pub(super) fn remove_journey_store(&self, journey: Journey) {
            let mut child = self.box_journeys.first_child();

            while let Some(c) = child {
                if c.first_child()
                    .expect("ListBoxRow to have a JourneyStoreItem child")
                    .property::<Journey>("journey")
                    .journey()
                    == journey.journey()
                {
                    self.box_journeys.remove(&c);
                }

                child = c.next_sibling();
            }

            if self.box_journeys.first_child().is_none() {
                self.box_journeys.set_visible(false);
            }
        }

        pub(super) fn add_search_store(&self, origin: String, destination: String) {
            let item = SearchStoreItem::new(origin, destination);
            self.box_searches.append(&item);
            self.box_searches.set_visible(true);
        }

        pub(super) fn remove_search_store(&self, origin: String, destination: String) {
            let mut child = self.box_searches.first_child();

            while let Some(c) = child {
                let s = c.first_child().expect("ListBoxRow to have a child");
                if s.property::<Option<String>>("origin") == Some(origin.clone())
                    && s.property::<Option<String>>("destination") == Some(destination.clone())
                {
                    self.box_searches.remove(&c);
                }

                child = c.next_sibling();
            }
            if self.box_searches.first_child().is_none() {
                self.box_searches.set_visible(false);
            }
        }

        #[template_callback]
        fn handle_swap(&self) {
            let from = self.in_from.input();
            let to = self.in_to.input();
            self.in_from.set_input(to);
            self.in_to.set_input(from);
        }

        #[template_callback]
        fn handle_search(&self) {
            let obj = self.obj();
            let from = self.in_from.property::<Place>("place");
            let to = self.in_to.property::<Place>("place");

            let time = self
                .pick_date_time
                .get()
                .get()
                .with_timezone(&chrono_tz::UTC);
            let time_type = self.pick_date_time.time_type();

            let main_context = MainContext::default();
            let window = self.obj().root().and_downcast::<Window>().expect(
                "search page must be mapped and realised when a template callback is called",
            );
            main_context.spawn_local(clone!(
                #[strong]
                from,
                #[strong]
                to,
                #[strong]
                obj,
                #[strong(rename_to = settings)]
                self.settings,
                #[strong]
                window,
                async move {
                    obj.set_searching(true);
                    let journeys = obj
                        .property::<Client>("client")
                        .journeys(
                            from,
                            to,
                            time_type,
                            JourneysOptions {
                                departure: if time_type == TimeType::Departure {
                                    Some(time)
                                } else {
                                    None
                                },
                                arrival: if time_type == TimeType::Arrival {
                                    Some(time)
                                } else {
                                    None
                                },
                                language: Some(Utility::language_code()),
                                stopovers: true,
                                loyalty_card: LoyaltyCard::from_id(
                                    settings
                                        .enum_("bahncard")
                                        .try_into()
                                        .expect("Failed to convert setting `bahncard` to u8"),
                                ),
                                bike_friendly: settings.boolean("bike-accessible"),
                                transfers: if settings.boolean("direct-only") {
                                    TransferOptions::Limited(0)
                                } else {
                                    TransferOptions::Unlimited
                                },
                                // Value clamped in the settings; default should never happen.
                                transfer_time: Duration::try_minutes(
                                    settings.int("transfer-time").into(),
                                )
                                .unwrap_or_default(),
                                tariff_class: if settings.boolean("first-class") {
                                    TariffClass::First
                                } else {
                                    TariffClass::Second
                                },
                                products: Utility::products_selection_from_setting(&settings),
                                ..Default::default()
                            },
                        )
                        .await;
                    obj.set_searching(false);
                    if journeys.is_err() {
                        window.display_error_toast(journeys.err().unwrap());
                        return;
                    }

                    obj.emit_by_name::<()>("search", &[&journeys.expect("Failed to get journeys")]);
                }
            ));
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SearchPage {
        const NAME: &'static str = "DBSearchPage";
        type Type = super::SearchPage;
        type ParentType = gtk::Box;

        fn new() -> Self {
            Self {
                settings: Settings::new(config::BASE_ID),
                in_from: Default::default(),
                in_to: Default::default(),
                pick_date_time: Default::default(),
                btn_search: Default::default(),
                stack_search_btn: Default::default(),
                box_journeys: Default::default(),
                box_searches: Default::default(),
                client: Default::default(),
                search_when_ready: Default::default(),
                searching: Default::default(),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            Self::bind_template_callbacks(klass);
            Utility::bind_template_callbacks(klass);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for SearchPage {
        fn constructed(&self) {
            self.parent_constructed();
            self.setup_search_when_ready();
            self.in_from
                .set_input(self.settings.string("search-from").to_string());
            self.in_to
                .set_input(self.settings.string("search-to").to_string());

            self.update_search_button();
            self.in_from.connect_notify_local(
                Some("set"),
                clone!(
                    #[weak(rename_to = search_page)]
                    self,
                    move |_, _| {
                        search_page.update_search_button();
                    }
                ),
            );
            self.in_to.connect_notify_local(
                Some("set"),
                clone!(
                    #[weak(rename_to = search_page)]
                    self,
                    move |_, _| {
                        search_page.update_search_button();
                    }
                ),
            );
            self.obj().connect_notify_local(
                Some("searching"),
                clone!(
                    #[weak(rename_to = search_page)]
                    self,
                    move |_, _| {
                        search_page.update_search_button();
                    }
                ),
            );
        }

        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![
                    Signal::builder("search")
                        .param_types([JourneysResult::static_type()])
                        .build(),
                    Signal::builder("details")
                        .param_types([Journey::static_type()])
                        .build(),
                ]
            });
            SIGNALS.as_ref()
        }
    }

    impl WidgetImpl for SearchPage {
        fn unmap(&self) {
            self.parent_unmap();
            self.settings
                .set_string("search-from", &self.in_from.input())
                .expect("Failed to save search-from");
            self.settings
                .set_string("search-to", &self.in_to.input())
                .expect("Failed to save search-to");
        }
    }
    impl BoxImpl for SearchPage {}
}
