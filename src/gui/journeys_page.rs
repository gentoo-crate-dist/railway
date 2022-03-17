use gdk::subclass::prelude::ObjectSubclassIsExt;
use hafas_rest::Hafas;

gtk::glib::wrapper! {
    pub struct JourneysPage(ObjectSubclass<imp::JourneysPage>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget;
}

impl JourneysPage {
    pub fn setup(&self, hafas: Hafas) {
        self.imp().setup(hafas, self);
    }
}

pub mod imp {
    use std::cell::RefCell;

    use gdk::gio::ListStore;
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
    use hafas_rest::Hafas;
    use hafas_rest::JourneysQuery;
    use libadwaita::ToastOverlay;
    use once_cell::sync::Lazy;

    use crate::gui::error::error_to_toast;
    use crate::gui::journey_list_item::JourneyListItem;
    use crate::gui::objects::JourneyObject;
    use crate::gui::objects::JourneysResultObject;

    #[derive(CompositeTemplate, Default)]
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

        journeys_result: RefCell<Option<JourneysResultObject>>,

        hafas: RefCell<Option<Hafas>>,
    }

    impl JourneysPage {
        pub(super) fn setup(&self, hafas: Hafas, obj: &super::JourneysPage) {
            self.hafas.replace(Some(hafas));
            self.bind_btn_earlier_later(obj);
        }

        fn bind_btn_earlier_later(&self, obj: &super::JourneysPage) {
            let hafas_borrow = self.hafas.borrow();
            let hafas = hafas_borrow.as_ref().expect("Hafas should be set up");

            self.btn_earlier.connect_clicked(clone!(@strong hafas, 
                                                    @strong obj, 
                                                    @strong self.toast_errors as toast_errors => move |_| {
                let main_context = MainContext::default();
                main_context.spawn_local(
                    clone!(@strong hafas, 
                           @strong obj,
                           @strong toast_errors => async move {
                        let journeys_result_obj = obj.property::<JourneysResultObject>("journeys-result");
                        let journeys_result = journeys_result_obj.journeys_result();

                        let result_journeys_result = hafas
                            .journey( &JourneysQuery {
                                from: Some(journeys_result.journeys[0].legs[0].origin.id.clone()),
                                to: Some(journeys_result.journeys[0].legs.last().expect("Every journey should have at least one leg.").destination.id.clone()),
                                earlier_than: Some(journeys_result.earlier_ref.clone()),
                                stopovers: Some(true),
                                ..Default::default()
                            })
                            .await;
                        if let Ok(mut result_journeys_result) = result_journeys_result {
                            result_journeys_result.journeys.append(&mut journeys_result.journeys.clone());
                            result_journeys_result.later_ref = journeys_result.later_ref;
                            obj.set_property("journeys-result", JourneysResultObject::new(result_journeys_result));
                        } else {
                            error_to_toast(&toast_errors, result_journeys_result.err().expect("Error to be present"));
                        }
                }));
            }));

            self.btn_later.connect_clicked(clone!(@strong hafas, 
                                                  @strong obj,
                                                  @strong self.toast_errors as toast_errors => move |_| {
                let main_context = MainContext::default();
                main_context.spawn_local(
                    clone!(@strong hafas, 
                           @strong obj,
                           @strong toast_errors => async move {
                        let journeys_result_obj = obj.property::<JourneysResultObject>("journeys-result");
                        let journeys_result = journeys_result_obj.journeys_result();

                        let result_journeys_result = hafas
                            .journey( &JourneysQuery {
                                from: Some(journeys_result.journeys[0].legs[0].origin.id.clone()),
                                to: Some(journeys_result.journeys[0].legs.last().expect("Every journey should have at least one leg.").destination.id.clone()),
                                later_than: Some(journeys_result.later_ref.clone()),
                                stopovers: Some(true),
                                ..Default::default()
                            })
                            .await;
                        if let Ok(mut result_journeys_result) = result_journeys_result {
                            result_journeys_result.journeys.splice(0..0, journeys_result.journeys);
                            result_journeys_result.earlier_ref = journeys_result.earlier_ref;
                            obj.set_property("journeys-result", JourneysResultObject::new(result_journeys_result));
                        } else {
                            error_to_toast(&toast_errors, result_journeys_result.err().expect("Error to be present"));
                        }
                }));
            }));
        }

        fn setup_model(&self, obj: &super::JourneysPage) {
            let model = gtk::gio::ListStore::new(JourneyObject::static_type());
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
                        .downcast::<JourneyObject>()
                        .expect("The item has to be an `JourneyObject`.");

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
                vec![ParamSpecObject::new(
                    "journeys-result",
                    "journeys-result",
                    "journeys-result",
                    JourneysResultObject::static_type(),
                    ParamFlags::READWRITE,
                )]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "journeys-result" => {
                    let obj = value.get::<Option<JourneysResultObject>>()
                        .expect("Property `journeys-result` of `JourneysPage` has to be of type `JourneysResultObject`");

                    let model = self.model.borrow();
                    model.remove_all();

                    model.splice(
                        0,
                        0,
                        &obj.as_ref()
                            .map(|o| o.journeys_result().journeys)
                            .unwrap_or_default()
                            .iter()
                            .map(|j| JourneyObject::new(j.clone()))
                            .collect::<Vec<JourneyObject>>(),
                    );
                    self.journeys_result.replace(obj);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "journeys-result" => self.journeys_result.borrow().to_value(),
                _ => unimplemented!(),
            }
        }

        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![Signal::builder(
                    "select",
                    &[JourneyObject::static_type().into()],
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
