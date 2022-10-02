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
    use gdk::glib::ParamFlags;
    use gdk::glib::ParamSpec;
    use gdk::glib::ParamSpecObject;
    use gdk::glib::Value;
    use gdk::glib::clone;
    use gdk::glib::MainContext;
    use gdk::glib::closure_local;
    use gdk::glib::subclass::Signal;
    use glib::subclass::InitializingObject;
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::CompositeTemplate;
    use hafas_rs::LoyaltyCard;
    use hafas_rs::ProductsSelection;
    use hafas_rs::TariffClass;
    use hafas_rs::api::journeys::JourneysOptions;
    use once_cell::sync::Lazy;

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

    #[derive(CompositeTemplate)]
    #[template(resource = "/ui/search_page.ui")]
    pub struct SearchPage {
        #[template_child]
        in_from: TemplateChild<StationEntry>,
        #[template_child]
        in_to: TemplateChild<StationEntry>,

        #[template_child]
        expand_date_time: TemplateChild<gtk::Expander>,
        #[template_child]
        pick_date_time: TemplateChild<DateTimePicker>,

        #[template_child]
        btn_search: TemplateChild<gtk::Button>,

        #[template_child]
        carousel_journeys: TemplateChild<libadwaita::Carousel>,
        #[template_child]
        carousel_searches: TemplateChild<libadwaita::Carousel>,

        #[template_child]
        toast_errors: TemplateChild<libadwaita::ToastOverlay>,

        settings: Settings,
        client: RefCell<Option<HafasClient>>,
    }

    #[gtk::template_callbacks]
    impl SearchPage {
        pub(super) fn add_journey_store(&self, journey: Journey) {
            let item = JourneyStoreItem::new(journey);
            let obj = self.instance();
            item.connect_closure("details", false, 
                                 closure_local!(move |_item: JourneyStoreItem, journey: Journey| {
                obj.emit_by_name::<()>("details", &[&journey]);
            }));
            self.carousel_journeys.append(&item);
        }

        pub(super) fn remove_journey_store(&self, journey: Journey) {
            let mut child = self.carousel_journeys.first_child();

            while let Some(c) = child {
                if c.property::<Journey>("journey").journey() == journey.journey() {
                    self.carousel_journeys.remove(&c);
                } 

                child = c.next_sibling();
            }
        }

        pub(super) fn add_search_store(&self, origin: String, destination: String) {
            let item = SearchStoreItem::new(origin, destination);
            let obj = self.instance();
            item.connect_closure("details", false, 
                                 closure_local!(move |_item: SearchStoreItem, origin: String, destination: String| {
                                     let s = obj.imp();
                                     s.in_from.set_input(origin);
                                     s.in_to.set_input(destination);
            }));
            self.carousel_searches.append(&item);
        }

        pub(super) fn remove_search_store(&self, origin: String, destination: String) {
            let mut child = self.carousel_searches.first_child();

            while let Some(c) = child {
                if c.property::<Option<String>>("origin") == Some(origin.clone()) 
                    && c.property::<Option<String>>("destination") == Some(destination.clone()) {
                    self.carousel_searches.remove(&c);
                } 

                child = c.next_sibling();
            }
        }

        #[template_callback]
        fn handle_swap(&self, _: gtk::Button) {
            let from = self.in_from.input();
            let to = self.in_to.input();
            self.in_from.set_input(to);
            self.in_to.set_input(from);
        }

        #[template_callback]
        fn handle_search(&self, _: gtk::Button) {
            let obj = self.instance();
            let from = self.in_from.property::<Place>("place");
            let to = self.in_to.property::<Place>("place");

            let departure = if self.expand_date_time.is_expanded() {
                Some(self.pick_date_time.get().get())
            } else {
                None
            };

            let main_context = MainContext::default();
            main_context.spawn_local(clone!(@strong from,
                                            @strong to,
                                            @strong obj, 
                                            @strong self.settings as settings,
                                            @strong self.toast_errors as toast_errors => async move {
                let journeys = obj.property::<HafasClient>("client").journeys(from, to, JourneysOptions {
                    departure: departure.map(|d| d.timestamp()),
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
                settings: Settings::new("de.schmidhuberj.DieBahn"),
                in_from: Default::default(),
                in_to: Default::default(),
                expand_date_time: Default::default(),
                pick_date_time: Default::default(),
                btn_search: Default::default(),
                carousel_journeys: Default::default(),
                carousel_searches: Default::default(),
                toast_errors: Default::default(),
                client: Default::default()
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

    impl ObjectImpl for SearchPage {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            self.in_from.set_input(self.settings.string("search-from").to_string());
            self.in_to.set_input(self.settings.string("search-to").to_string());
        }

        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![Signal::builder(
                    "search",
                    &[JourneysResult::static_type().into()],
                    <()>::static_type().into(),
                )
                .build(),
                Signal::builder(
                    "details",
                    &[Journey::static_type().into()],
                    <()>::static_type().into(),
                )
                .build()]
            });
            SIGNALS.as_ref()
        }

        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![ParamSpecObject::new(
                    "client",
                    "client",
                    "client",
                    HafasClient::static_type(),
                    ParamFlags::READWRITE,
                )]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "client" => {
                    let obj = value
                        .get::<Option<HafasClient>>()
                        .expect("Property `client` of `SearchPage` has to be of type `HafasClient`");

                    self.client.replace(obj);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "client" => self.client.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for SearchPage {
        fn unmap(&self, widget: &Self::Type) {
            self.parent_unmap(widget);
            self.settings.set_string("search-from", &self.in_from.input()).expect("Failed to save search-from");
            self.settings.set_string("search-to", &self.in_to.input()).expect("Failed to save search-to");
        }
    }
    impl BoxImpl for SearchPage {}
}

