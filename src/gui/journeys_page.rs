use std::time::Duration;

use gdk::{
    glib::{self, clone},
    prelude::ObjectExt,
    subclass::prelude::ObjectSubclassIsExt,
};
use gtk::traits::AdjustmentExt;

gtk::glib::wrapper! {
    pub struct JourneysPage(ObjectSubclass<imp::JourneysPage>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget;
}

impl JourneysPage {
    fn set_loading_earlier(&self, is: bool) {
        self.set_property("is-loading-earlier", is)
    }

    fn set_loading_later(&self, is: bool) {
        self.set_property("is-loading-later", is)
    }

    fn is_loading_earlier(&self) -> bool {
        self.property("is-loading-earlier")
    }

    fn is_loading_later(&self) -> bool {
        self.property("is-loading-later")
    }

    fn is_auto_scroll(&self) -> bool {
        self.property("auto-scroll")
    }

    fn set_auto_scroll(&self, val: bool) {
        self.set_property("auto-scroll", val)
    }

    fn scroll_down(&self) {
        if self.is_auto_scroll() {
            gspawn!(clone!(@weak self as obj => async move  {
                // Need to sleep a little to make sure the scrolled window saw the changed
                // child.
                glib::timeout_future(Duration::from_millis(50)).await;
                let adjustment = obj.imp().scrolled_window.vadjustment();
                adjustment.set_value(adjustment.upper());
            }));
        }
    }

    fn scroll_up(&self) {
        if self.is_auto_scroll() {
            gspawn!(clone!(@weak self as obj => async move  {
                // Need to sleep a little to make sure the scrolled window saw the changed
                // child.
                glib::timeout_future(Duration::from_millis(50)).await;
                let adjustment = obj.imp().scrolled_window.vadjustment();
                adjustment.set_value(adjustment.lower());
            }));
        }
    }
}

pub mod imp {
    use std::cell::Cell;
    use std::cell::RefCell;

    use gdk::gio::ListStore;
    use gdk::gio::Settings;
    use gdk::glib::clone;
    use gdk::glib::subclass::Signal;
    use gdk::glib::MainContext;
    use gdk::glib::ParamSpec;
    use gdk::glib::ParamSpecBoolean;
    use gdk::glib::ParamSpecObject;
    use gdk::glib::Value;
    use glib::subclass::InitializingObject;
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::CompositeTemplate;
    use gtk::ListItem;
    use gtk::PositionType;
    use gtk::SignalListItemFactory;
    use gtk::Widget;
    use hafas_rs::api::journeys::JourneysOptions;
    use hafas_rs::LoyaltyCard;
    use hafas_rs::ProductsSelection;
    use hafas_rs::TariffClass;
    use libadwaita::ToastOverlay;
    use once_cell::sync::Lazy;

    use crate::backend::HafasClient;
    use crate::backend::Journey;
    use crate::backend::JourneysResult;
    use crate::backend::Place;
    use crate::gui::error::error_to_toast;
    use crate::gui::journey_list_item::JourneyListItem;
    use crate::gui::utility::Utility;
    use crate::config;

    #[derive(CompositeTemplate)]
    #[template(resource = "/ui/journeys_page.ui")]
    pub struct JourneysPage {
        #[template_child]
        pub(super) scrolled_window: TemplateChild<gtk::ScrolledWindow>,

        #[template_child]
        list_journeys: TemplateChild<gtk::ListView>,
        #[template_child]
        toast_errors: TemplateChild<ToastOverlay>,

        destination_alignment_group: gtk::SizeGroup,

        model: RefCell<ListStore>,

        journeys_result: RefCell<Option<JourneysResult>>,

        settings: Settings,
        client: RefCell<Option<HafasClient>>,

        loading_earlier: Cell<bool>,
        loading_later: Cell<bool>,
        auto_scroll: Cell<bool>,
    }

    impl Default for JourneysPage {
        fn default() -> Self {
            Self {
                scrolled_window: Default::default(),
                list_journeys: Default::default(),
                toast_errors: Default::default(),
                destination_alignment_group: gtk::SizeGroup::new(gtk::SizeGroupMode::Horizontal),
                model: RefCell::new(gtk::gio::ListStore::new::<Journey>()),
                journeys_result: Default::default(),
                settings: Settings::new(config::BASE_ID),
                client: Default::default(),
                loading_earlier: Default::default(),
                loading_later: Default::default(),
                auto_scroll: Cell::new(true),
            }
        }
    }

    #[gtk::template_callbacks]
    impl JourneysPage {
        /// Every time when the page is not yet filled with the journeys, load more.
        fn connect_initial_loading(&self) {
            let obj = self.obj();
            self.scrolled_window
                .vadjustment()
                .connect_changed(clone!(@weak obj => move |adj| {
                    // This means the page is not yet filled.
                    if adj.upper() <= adj.page_size() {
                        // Do not scroll for the initial loading.
                        obj.set_auto_scroll(false);
                        obj.imp().handle_later()
                    } else {
                        // Scroll if the page is already filled and more is manually requested.
                        obj.set_auto_scroll(true);
                    }
                }));
        }

        #[template_callback]
        fn handle_edge_reached(&self, position: PositionType) {
            match position {
                PositionType::Top => self.handle_earlier(),
                PositionType::Bottom => self.handle_later(),
                _ => (),
            }
        }

        #[template_callback]
        fn handle_earlier(&self) {
            let obj = self.obj();

            // Skip if already loading.
            if obj.is_loading_earlier() {
                return;
            }
            obj.set_loading_earlier(true);

            let main_context = MainContext::default();
            main_context.spawn_local(
                clone!(
                       @strong obj,
                       @strong self.settings as settings,
                       @strong self.toast_errors as toast_errors => async move {
                    let journeys_result_obj = obj.property::<JourneysResult>("journeys-result");
                    let journeys_result = journeys_result_obj.journeys_response();

                    let result_journeys_result = obj.property::<HafasClient>("client")
                        .journeys(Place::new(journeys_result.journeys[0].legs[0].origin.clone()), Place::new(journeys_result.journeys[0].legs.last().expect("Every journey should have at least one leg.").destination.clone()), JourneysOptions {
                            earlier_than: journeys_result.earlier_ref.clone(),
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
                        })
                        .await;
                    if let Ok(result_journeys_result) = result_journeys_result {
                        result_journeys_result.merge_append(&journeys_result_obj);
                        obj.set_property("journeys-result", result_journeys_result);
                    } else {
                        error_to_toast(&toast_errors, result_journeys_result.expect_err("Error to be present"));
                    }
                    obj.set_loading_earlier(false);
                    obj.scroll_up();
            }));
        }

        #[template_callback]
        fn handle_later(&self) {
            let obj = self.obj();

            // Skip if already loading.
            if obj.is_loading_later() {
                return;
            }
            obj.set_loading_later(true);

            let main_context = MainContext::default();
            main_context.spawn_local(
                clone!(
                       @strong obj,
                       @strong self.settings as settings,
                       @strong self.toast_errors as toast_errors => async move {
                    let journeys_result_obj = obj.property::<JourneysResult>("journeys-result");
                    let journeys_result = journeys_result_obj.journeys_response();

                    let result_journeys_result = obj.property::<HafasClient>("client")
                        .journeys(Place::new(journeys_result.journeys[0].legs[0].origin.clone()), Place::new(journeys_result.journeys[0].legs.last().expect("Every journey should have at least one leg.").destination.clone()), JourneysOptions {
                            later_than: journeys_result.later_ref.clone(),
                            language: Some(gettextrs::gettext("language")),
                            stopovers: Some(true),
                            loyalty_card: LoyaltyCard::from_id(settings.enum_("bahncard").try_into().expect("Failed to convert setting `bahncard` to u8")),
                            bike_friendly: Some(settings.boolean("bike-accessible")),
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
                        })
                        .await;
                    if let Ok(result_journeys_result) = result_journeys_result {
                        result_journeys_result.merge_prepend(&journeys_result_obj);
                        obj.set_property("journeys-result", result_journeys_result);
                    } else {
                        error_to_toast(&toast_errors, result_journeys_result.expect_err("Error to be present"));
                    }
                    obj.set_loading_later(false);
                    obj.scroll_down();
            }));
        }

        fn setup_model(&self, obj: &super::JourneysPage) {
            let model = gtk::gio::ListStore::new::<Journey>();
            let selection_model = gtk::NoSelection::new(Some(model.clone()));
            self.list_journeys.get().set_model(Some(&selection_model));

            self.model.replace(model);

            let factory = SignalListItemFactory::new();
            factory.connect_setup(
                clone!(@weak self.destination_alignment_group as size_group => move |_, list_item| {
                    let journey_item = JourneyListItem::new();
                    let list_item = list_item
                        .downcast_ref::<ListItem>()
                        .expect("The factory item to be a `ListItem`");

                    list_item.set_child(Some(&journey_item));
                    list_item
                        .property_expression("item")
                        .bind(&journey_item, "journey", Widget::NONE);

                    size_group.add_widget(&journey_item.get_destination_box());
                }),
            );
            self.list_journeys.set_factory(Some(&factory));
            self.list_journeys.set_single_click_activate(true);

            self.list_journeys
                .connect_activate(clone!(@strong obj => move |list_view, position| {
                    let model = list_view.model().expect("The model has to exist.");
                    let journey_object = model
                        .item(position)
                        .expect("The item has to exist.")
                        .downcast::<Journey>()
                        .expect("The item has to be an `Journey`.");

                    obj.emit_by_name::<()>("select", &[&journey_object]);
                }));
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for JourneysPage {
        const NAME: &'static str = "DBJourneysPage";
        type Type = super::JourneysPage;
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

    impl ObjectImpl for JourneysPage {
        fn constructed(&self) {
            self.parent_constructed();
            self.setup_model(&self.obj());
            self.connect_initial_loading();
        }

        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecObject::builder::<JourneysResult>("journeys-result").build(),
                    ParamSpecObject::builder::<HafasClient>("client").build(),
                    ParamSpecBoolean::builder("is-loading-earlier").build(),
                    ParamSpecBoolean::builder("is-loading-later").build(),
                    ParamSpecBoolean::builder("auto-scroll").build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "journeys-result" => {
                    let obj = value.get::<Option<JourneysResult>>()
                        .expect("Property `journeys-result` of `JourneysPage` has to be of type `JourneysResult`");

                    let model = self.model.borrow();
                    model.remove_all();

                    model.splice(
                        0,
                        0,
                        &obj.as_ref().map(|o| o.journeys()).unwrap_or_default(),
                    );
                    self.journeys_result.replace(obj);
                }
                "client" => {
                    let obj = value.get::<Option<HafasClient>>().expect(
                        "Property `client` of `JourneysPage` has to be of type `HafasClient`",
                    );

                    self.client.replace(obj);
                }
                "is-loading-earlier" => {
                    let obj = value.get::<bool>().expect(
                        "Property `is-loading-earlier` of `JourneysPage` has to be of type `bool`",
                    );

                    self.loading_earlier.replace(obj);
                }
                "is-loading-later" => {
                    let obj = value.get::<bool>().expect(
                        "Property `is-loading-later` of `JourneysPage` has to be of type `bool`",
                    );

                    self.loading_later.replace(obj);
                }
                "auto-scroll" => {
                    let obj = value.get::<bool>().expect(
                        "Property `auto-scroll` of `JourneysPage` has to be of type `bool`",
                    );

                    self.auto_scroll.replace(obj);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "journeys-result" => self.journeys_result.borrow().to_value(),
                "client" => self.client.borrow().to_value(),
                "is-loading-earlier" => self.loading_earlier.get().to_value(),
                "is-loading-later" => self.loading_later.get().to_value(),
                "auto-scroll" => self.auto_scroll.get().to_value(),
                _ => unimplemented!(),
            }
        }

        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![Signal::builder("select")
                    .param_types([Journey::static_type()])
                    .build()]
            });
            SIGNALS.as_ref()
        }
    }

    impl WidgetImpl for JourneysPage {}
    impl BoxImpl for JourneysPage {}
}
