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
    use gdk::gio::Settings;
    use gdk::glib::Properties;
    use gdk::glib::clone;
    use gdk::glib::MainContext;
    use gdk::glib::subclass::Signal;
    use glib::subclass::InitializingObject;
    use gtk::ListBoxRow;
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::CompositeTemplate;
    use hafas_rs::LoyaltyCard;
    use hafas_rs::ProductsSelection;
    use hafas_rs::TariffClass;
    use hafas_rs::api::journeys::JourneysOptions;
    use once_cell::sync::Lazy;

    use std::cell::Cell;
    use std::cell::RefCell;

    use crate::backend::HafasClient;
    use crate::backend::Journey;
    use crate::backend::JourneysResult;
    use crate::backend::Place;
    use crate::gui::date_time_picker::DateTimePicker;
    use crate::gui::error::error_to_toast;
    use crate::gui::journey_store_item::JourneyStoreItem;
    use crate::gui::search_store_item::SearchStoreItem;
    use crate::gui::station_entry::StationEntry;
    use crate::gui::utility::Utility;
    use crate::config;

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
        box_journeys: TemplateChild<gtk::ListBox>,
        #[template_child]
        box_searches: TemplateChild<gtk::ListBox>,

        #[template_child]
        toast_errors: TemplateChild<libadwaita::ToastOverlay>,

        settings: Settings,
        #[property(get, set)]
        client: RefCell<Option<HafasClient>>,
        #[property(get, set)]
        search_when_ready: Cell<bool>,
        #[property(get, set)]
        searching: Cell<bool>,
    }

    #[gtk::template_callbacks]
    impl SearchPage {
        #[template_callback]
        fn handle_journeys_row_activated(&self, row: ListBoxRow) {
            self.obj().emit_by_name::<()>("details", &[&row.first_child().expect("Activated row to have a child").property::<Journey>("journey")]);
        }

        #[template_callback]
        fn handle_searches_row_activated(&self, row: ListBoxRow) {
            let search = &row.first_child().expect("Activated row to have a child");
            self.in_from.set_input(search.property::<String>("origin"));
            self.in_to.set_input(search.property::<String>("destination"));
            self.obj().set_search_when_ready(true);
        }

        fn update_search_button(&self) {
            let from_set = self.in_from.property::<bool>("set");
            let to_set = self.in_to.property::<bool>("set");
            let searching = self.searching.get();

            self.btn_search.set_tooltip_text(match (from_set, to_set, searching) {
                (false, false, _) => Some("Start and destination are missing"),
                (false, true, _) => Some("Start is missing"),
                (true, false, _) => Some("Destination is missing"),
                (true, true, true) => Some("Search ongoing"),
                (_, _, _) => None,
            });
        }

        fn setup_search_when_ready(&self) {
            let obj = self.obj();
            self.btn_search.connect_sensitive_notify(clone!(@weak obj => move |_| {
                let s = obj.imp();
                if s.btn_search.is_sensitive() && obj.search_when_ready() {
                    s.handle_search();
                }
                // XXX: Not the best place to unset search when ready. It would be better to do that when the station entry changed manually, but that will not work as it also changes when directly set.
                obj.set_search_when_ready(false);
            }));
        }

        pub(super) fn add_journey_store(&self, journey: Journey) {
            let item = JourneyStoreItem::new(journey);
            self.box_journeys.append(&item);
            self.box_journeys.set_visible(true);
        }

        pub(super) fn remove_journey_store(&self, journey: Journey) {
            let mut child = self.box_journeys.first_child();

            while let Some(c) = child {
                if c.first_child().expect("ListBoxRow to have a JourneyStoreItem child").property::<Journey>("journey").journey() == journey.journey() {
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
                    && s.property::<Option<String>>("destination") == Some(destination.clone()) {
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

            let departure = Some(self.pick_date_time.get().get().naive_local());

            let main_context = MainContext::default();
            main_context.spawn_local(clone!(@strong from,
                                            @strong to,
                                            @strong obj, 
                                            @strong self.settings as settings,
                                            @strong self.toast_errors as toast_errors => async move {
                obj.set_searching(true);
                let journeys = obj.property::<HafasClient>("client").journeys(from, to, JourneysOptions {
                    departure,
                    language: Some(gettextrs::gettext("language")),
                    stopovers: Some(true),
                    loyalty_card: LoyaltyCard::from_id(settings.enum_("bahncard").try_into().expect("Failed to convert setting `bahncard` to u8")),
                    bike_friendly: Some(settings.boolean("bike-accessible")),
                    start_with_walking: Some(false),
                    transfers: if settings.boolean("direct-only") {Some(0)} else {None},
                    transfer_time: Some(settings.int("transfer-time").try_into().unwrap_or_default()),
                    tariff_class: Some(if settings.boolean("first-class") {
                        TariffClass::First
                    } else {
                        TariffClass::Second
                    }),
                    products: ProductsSelection {
                        national_express: Some(settings.boolean("include-national-express")),
                        national: Some(settings.boolean("include-national")),
                        regional_exp: Some(settings.boolean("include-regional-express")),
                        regional: Some(settings.boolean("include-regional")),
                        suburban: Some(settings.boolean("include-suburban")),
                        bus: Some(settings.boolean("include-bus")),
                        ferry: Some(settings.boolean("include-ferry")),
                        subway: Some(settings.boolean("include-subway")),
                        tram: Some(settings.boolean("include-tram")),
                        taxi: Some(settings.boolean("include-taxi")),
                    },
                    ..Default::default()
                }).await;
                obj.set_searching(false);
                if journeys.is_err() {
                    error_to_toast(&toast_errors, journeys.err().unwrap());
                    return;
                }

                obj.emit_by_name::<()>("search", &[&journeys.expect("Failed to get journeys")]);
            }));
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
                box_journeys: Default::default(),
                box_searches: Default::default(),
                toast_errors: Default::default(),
                client: Default::default(),
                search_when_ready: Default::default(),
                searching: Default::default()
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
            self.in_from.set_input(self.settings.string("search-from").to_string());
            self.in_to.set_input(self.settings.string("search-to").to_string());

            self.update_search_button();
            self.in_from.connect_notify_local(Some("set"), clone!(@weak self as search_page => move |_, _| {
                search_page.update_search_button();
            }));
            self.in_to.connect_notify_local(Some("set"), clone!(@weak self as search_page => move |_, _| {
                search_page.update_search_button();
            }));
            self.obj().connect_notify_local(Some("searching"), clone!(@weak self as search_page => move |_, _| {
                search_page.update_search_button();
            }));
        }

        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![
                    Signal::builder("search")
                        .param_types([JourneysResult::static_type()])
                        .build(),
                    Signal::builder("details")
                        .param_types([Journey::static_type()])
                        .build()
                ]
            });
            SIGNALS.as_ref()
        }
    }

    impl WidgetImpl for SearchPage {
        fn unmap(&self) {
            self.parent_unmap();
            self.settings.set_string("search-from", &self.in_from.input()).expect("Failed to save search-from");
            self.settings.set_string("search-to", &self.in_to.input()).expect("Failed to save search-to");
        }
    }
    impl BoxImpl for SearchPage {}
}

