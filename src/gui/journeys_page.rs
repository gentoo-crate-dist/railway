gtk::glib::wrapper! {
    pub struct JourneysPage(ObjectSubclass<imp::JourneysPage>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget;
}

pub mod imp {
    use std::cell::RefCell;

    use gdk::gio::ListStore;
    use gdk::gio::Settings;
    use gdk::glib::clone;
    use gdk::glib::subclass::Signal;
    use gdk::glib::MainContext;
    use gdk::glib::ParamFlags;
    use gdk::glib::ParamSpec;
    use gdk::glib::ParamSpecObject;
    use gdk::glib::Value;
    use glib::subclass::InitializingObject;
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::CompositeTemplate;
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

    #[derive(CompositeTemplate)]
    #[template(resource = "/ui/journeys_page.ui")]
    pub struct JourneysPage {
        #[template_child]
        list_journeys: TemplateChild<gtk::ListView>,
        #[template_child]
        btn_earlier: TemplateChild<gtk::Button>,
        #[template_child]
        btn_later: TemplateChild<gtk::Button>,

        #[template_child]
        toast_errors: TemplateChild<ToastOverlay>,

        model: RefCell<ListStore>,

        journeys_result: RefCell<Option<JourneysResult>>,

        settings: Settings,
        client: RefCell<Option<HafasClient>>,
    }

    impl Default for JourneysPage {
        fn default() -> Self {
            Self {
                list_journeys: Default::default(),
                btn_earlier: Default::default(),
                btn_later: Default::default(),
                toast_errors: Default::default(),
                model: Default::default(),
                journeys_result: Default::default(),
                settings: Settings::new("de.schmidhuberj.DieBahn"),
                client: Default::default(),
            }
        }
    }

    #[gtk::template_callbacks]
    impl JourneysPage {
        #[template_callback]
        fn handle_earlier(&self, _: gtk::Button) {
            let obj = self.instance();

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
                        error_to_toast(&toast_errors, result_journeys_result.err().expect("Error to be present"));
                    }
            }));
        }

        #[template_callback]
        fn handle_later(&self, _: gtk::Button) {
            let obj = self.instance();

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
                        error_to_toast(&toast_errors, result_journeys_result.err().expect("Error to be present"));
                    }
            }));
        }

        fn setup_model(&self, obj: &super::JourneysPage) {
            let model = gtk::gio::ListStore::new(Journey::static_type());
            let selection_model = gtk::NoSelection::new(Some(&model));
            self.list_journeys.get().set_model(Some(&selection_model));

            self.model.replace(model);

            let factory = SignalListItemFactory::new();
            factory.connect_setup(move |_, list_item| {
                let journey_item = JourneyListItem::new();
                list_item.set_child(Some(&journey_item));

                list_item
                    .property_expression("item")
                    .bind(&journey_item, "journey", Widget::NONE);
            });
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
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            self.setup_model(obj);
        }

        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecObject::new(
                        "journeys-result",
                        "journeys-result",
                        "journeys-result",
                        JourneysResult::static_type(),
                        ParamFlags::READWRITE,
                    ),
                    ParamSpecObject::new(
                        "client",
                        "client",
                        "client",
                        HafasClient::static_type(),
                        ParamFlags::READWRITE,
                    ),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
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
                        "Property `client` of `SearchPage` has to be of type `HafasClient`",
                    );

                    self.client.replace(obj);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "journeys-result" => self.journeys_result.borrow().to_value(),
                "client" => self.client.borrow().to_value(),
                _ => unimplemented!(),
            }
        }

        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![Signal::builder(
                    "select",
                    &[Journey::static_type().into()],
                    <()>::static_type().into(),
                )
                .build()]
            });
            SIGNALS.as_ref()
        }
    }

    impl WidgetImpl for JourneysPage {}
    impl BoxImpl for JourneysPage {}
}
