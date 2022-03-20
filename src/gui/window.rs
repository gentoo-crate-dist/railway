use gtk::glib::Object;

use super::{
    alt_label::AltLabel, date_time_picker::DateTimePicker, station_entry::StationEntry,
    stores::journey_store::JourneysStore,
};

gtk::glib::wrapper! {
    pub struct Window(ObjectSubclass<imp::Window>)
        @extends libadwaita::ApplicationWindow, gtk::ApplicationWindow, libadwaita::Window, gtk::Window, gtk::Widget,
        @implements gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl Window {
    pub fn new(app: &gtk::Application) -> Self {
        let _: AltLabel = Object::new(&[]).expect("Failed to initialize `DBAltLabel`");
        let _: DateTimePicker = Object::new(&[]).expect("Failed to initialize `DBDateTimePicker`");
        let _: StationEntry = Object::new(&[]).expect("Failed to initialize `DBStationEntry`");
        let _: JourneysStore = Object::new(&[]).expect("Failed to initialize `DBJourneysStore`");
        Object::new(&[("application", app)]).expect("Failed to create Window")
    }
}

pub mod imp {
    use std::cell::RefCell;

    use glib::subclass::InitializingObject;
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::CompositeTemplate;
    use gtk::Inhibit;
    use hafas_rest::Hafas;
    use libadwaita::subclass::prelude::AdwApplicationWindowImpl;
    use libadwaita::subclass::prelude::AdwWindowImpl;
    use rrw::RestConfig;

    use crate::gui::journey_detail_page::JourneyDetailPage;
    use crate::gui::journeys_page::JourneysPage;
    use crate::gui::objects::JourneyObject;
    use crate::gui::objects::JourneysResultObject;
    use crate::gui::search_page::SearchPage;
    use crate::gui::stores::journey_store::JourneysStore;
    use crate::gui::utility::Utility;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/ui/window.ui")]
    pub struct Window {
        #[template_child]
        leaflet: TemplateChild<libadwaita::Leaflet>,

        #[template_child]
        search_page: TemplateChild<SearchPage>,
        #[template_child]
        journeys_page: TemplateChild<JourneysPage>,
        #[template_child]
        journey_detail_page: TemplateChild<JourneyDetailPage>,

        #[template_child]
        btn_reload_detail: TemplateChild<gtk::Button>,

        #[template_child]
        store_journeys: TemplateChild<JourneysStore>,

        hafas: RefCell<Option<Hafas>>,
    }

    #[gtk::template_callbacks]
    impl Window {
        fn setup(&self) {
            let hafas = Hafas::new(RestConfig::new("https://v5.db.transport.rest"));
            self.hafas.replace(Some(hafas.clone()));
            self.search_page.setup(hafas.clone());
            self.journeys_page.setup(hafas.clone());
            self.journey_detail_page.setup(hafas);
            self.store_journeys.setup();
        }

        #[template_callback]
        fn handle_go_back(&self) {
            self.leaflet.navigate(libadwaita::NavigationDirection::Back);
        }

        #[template_callback]
        fn handle_details(&self, journey: JourneyObject) {
            self.leaflet
                .navigate(libadwaita::NavigationDirection::Forward);
            self.leaflet
                .navigate(libadwaita::NavigationDirection::Forward);
            self.journey_detail_page.set_property("journey", journey);
            self.journey_detail_page.reload();
        }

        #[template_callback]
        fn handle_search_page(&self, journeys_result: JourneysResultObject) {
            self.journeys_page
                .set_property("journeys-result", journeys_result);
            self.leaflet
                .navigate(libadwaita::NavigationDirection::Forward);
        }

        #[template_callback]
        fn handle_journey_reload(&self, _: gtk::Button) {
            self.journey_detail_page.reload();
        }

        #[template_callback]
        fn handle_journeys_page(&self, journey: JourneyObject) {
            self.journey_detail_page.set_property("journey", journey);
            self.leaflet
                .navigate(libadwaita::NavigationDirection::Forward);
        }

        #[template_callback]
        fn handle_journey_store(&self, _: gtk::Button) {
            if let Some(journey) = self
                .journey_detail_page
                .property::<Option<JourneyObject>>("journey")
            {
                self.store_journeys.store(journey)
            }
        }

        #[template_callback]
        fn handle_journey_store_add(&self, journey: JourneyObject) {
            self.search_page.add_journey_store(journey);
        }

        #[template_callback]
        fn handle_journey_store_remove(&self, journey: JourneyObject) {
            self.search_page.remove_journey_store(journey);
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Window {
        const NAME: &'static str = "DBWindow";
        type Type = super::Window;
        type ParentType = libadwaita::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            Self::bind_template_callbacks(klass);
            Utility::bind_template_callbacks(klass);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Window {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            self.setup();
        }
    }

    impl WidgetImpl for Window {}
    impl WindowImpl for Window {
        fn close_request(&self, _obj: &Self::Type) -> Inhibit {
            self.store_journeys.get().flush();
            Inhibit(false)
        }
    }
    impl ApplicationWindowImpl for Window {}
    impl AdwWindowImpl for Window {}
    impl AdwApplicationWindowImpl for Window {}
}
