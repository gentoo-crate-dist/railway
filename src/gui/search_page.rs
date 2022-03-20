use gdk::subclass::prelude::ObjectSubclassIsExt;
use hafas_rest::Hafas;

use super::objects::JourneyObject;

gtk::glib::wrapper! {
    pub struct SearchPage(ObjectSubclass<imp::SearchPage>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget;
}

impl SearchPage {
    pub fn setup(&self, hafas: Hafas) {
        self.imp().setup(hafas);
    }

    pub fn add_journey_store(&self, journey: JourneyObject) {
        self.imp().add_journey_store(journey);
    }

    pub fn remove_journey_store(&self, journey: JourneyObject) {
        self.imp().remove_journey_store(journey);
    }
}

pub mod imp {
    use gdk::glib::clone;
    use gdk::glib::MainContext;
    use gdk::glib::closure_local;
    use gdk::glib::subclass::Signal;
    use glib::subclass::InitializingObject;
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::CompositeTemplate;
    use hafas_rest::JourneysQuery;
    use once_cell::sync::Lazy;

    use std::cell::RefCell;

    use hafas_rest::Hafas;

    use crate::gui::date_time_picker::DateTimePicker;
    use crate::gui::error::error_to_toast;
    use crate::gui::journey_store_item::JourneyStoreItem;
    use crate::gui::objects::JourneyObject;
    use crate::gui::objects::JourneysResultObject;
    use crate::gui::objects::StationObject;
    use crate::gui::station_entry::StationEntry;
    use crate::gui::utility::Utility;

    #[derive(CompositeTemplate, Default)]
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
        toast_errors: TemplateChild<libadwaita::ToastOverlay>,

        hafas: RefCell<Option<Hafas>>,
    }

    #[gtk::template_callbacks]
    impl SearchPage {
        pub(super) fn setup(&self, hafas: Hafas) {
            self.hafas.replace(Some(hafas.clone()));
            self.in_from.setup(hafas.clone());
            self.in_to.setup(hafas);
        }

        pub(super) fn add_journey_store(&self, journey: JourneyObject) {
            let item = JourneyStoreItem::new(journey);
            let obj = self.instance();
            item.connect_closure("details", false, 
                                 closure_local!(move |_item: JourneyStoreItem, journey: JourneyObject| {
                obj.emit_by_name::<()>("details", &[&journey]);
            }));
            self.carousel_journeys.append(&item);
        }

        pub(super) fn remove_journey_store(&self, journey: JourneyObject) {
            let mut child = self.carousel_journeys.first_child();

            while let Some(c) = child {
                if c.property::<JourneyObject>("journey").journey() == journey.journey() {
                    self.carousel_journeys.remove(&c);
                } 

                child = c.next_sibling();
            }
        }

        #[template_callback]
        fn handle_search(&self, _: gtk::Button) {
            let hafas = &self.hafas;
            let obj = self.instance();
            let from_obj = self.in_from.property::<Option<StationObject>>("station").expect("Input 'from' not set");
            let to_obj = self.in_to.property::<Option<StationObject>>("station").expect("Input 'to' not set");
            let from = from_obj.station();
            let to = to_obj.station();

            let departure = if self.expand_date_time.is_expanded() {
                Some(self.pick_date_time.get().get())
            } else {
                None
            };

            let main_context = MainContext::default();
            main_context.spawn_local(clone!(@strong from,
                                            @strong to,
                                            @strong hafas, 
                                            @strong obj, 
                                            @strong self.toast_errors as toast_errors => async move {
                let hafas_borrow = hafas.borrow();
                let hafas = hafas_borrow.as_ref().expect("Hafas has not yet been set up.");

                let journeys = hafas
                    .journey(&JourneysQuery {
                        from: Some(from.id.clone()),
                        to: Some(to.id.clone()),
                        departure,
                        stopovers: Some(true),
                        ..Default::default()
                    })
                    .await;

                if journeys.is_err() {
                    error_to_toast(&toast_errors, journeys.err().unwrap());
                    return;
                }

                obj.emit_by_name::<()>("search", &[&JourneysResultObject::new(journeys.expect("Journey not found").clone())]);
            }));
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SearchPage {
        const NAME: &'static str = "DBSearchPage";
        type Type = super::SearchPage;
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

    impl ObjectImpl for SearchPage {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }

        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![Signal::builder(
                    "search",
                    &[JourneysResultObject::static_type().into()],
                    <()>::static_type().into(),
                )
                .build(),
                Signal::builder(
                    "details",
                    &[JourneyObject::static_type().into()],
                    <()>::static_type().into(),
                )
                .build()]
            });
            SIGNALS.as_ref()
        }
    }

    impl WidgetImpl for SearchPage {}
    impl BoxImpl for SearchPage {}
}

