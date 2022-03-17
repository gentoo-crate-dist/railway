use gtk::glib::Object;

use super::{alt_label::AltLabel, date_time_picker::DateTimePicker, station_entry::StationEntry};

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
        Object::new(&[("application", app)]).expect("Failed to create Window")
    }
}

pub mod imp {
    use std::cell::RefCell;

    use gdk::glib::clone;
    use gdk::glib::closure_local;
    use glib::subclass::InitializingObject;
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::CompositeTemplate;
    use hafas_rest::Hafas;
    use libadwaita::subclass::prelude::AdwApplicationWindowImpl;
    use libadwaita::subclass::prelude::AdwWindowImpl;
    use rrw::RestConfig;

    use crate::gui::journey_detail_page::JourneyDetailPage;
    use crate::gui::journeys_page::JourneysPage;
    use crate::gui::objects::JourneyObject;
    use crate::gui::objects::JourneysResultObject;
    use crate::gui::search_page::SearchPage;

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
        btn_go_back: TemplateChild<gtk::Button>,
        #[template_child]
        btn_go_back_2: TemplateChild<gtk::Button>,

        #[template_child]
        btn_reload_detail: TemplateChild<gtk::Button>,

        hafas: RefCell<Option<Hafas>>,
    }

    impl Window {
        fn setup(&self) {
            let hafas = Hafas::new(RestConfig::new("https://v5.db.transport.rest"));
            self.hafas.replace(Some(hafas.clone()));
            self.search_page.setup(hafas.clone());
            self.journeys_page.setup(hafas.clone());
            self.journey_detail_page.setup(hafas);
        }

        fn bind_btn_go_back(&self) {
            self.btn_go_back
                .connect_clicked(clone!(@strong self.leaflet as leaflet => move |_| {
                    leaflet.navigate(libadwaita::NavigationDirection::Back);
                }));
            self.btn_go_back_2.connect_clicked(
                clone!(@strong self.leaflet as leaflet => move |_| {
                    leaflet.navigate(libadwaita::NavigationDirection::Back);
                }),
            );
        }

        fn bind_search_page(&self) {
            let leaflet = self.leaflet.clone();
            let journeys_page = self.journeys_page.clone();
            self.search_page.connect_closure(
                "search",
                false,
                closure_local!(
                    move |_: SearchPage, journeys_result: JourneysResultObject| {
                        journeys_page.set_property("journeys-result", journeys_result);
                        leaflet.navigate(libadwaita::NavigationDirection::Forward);
                    }
                ),
            );
        }

        fn bind_btn_reload_detail(&self) {
            self.btn_reload_detail.connect_clicked(
                clone!(@strong self.journey_detail_page as journey_detail_page => move |_| {
                    journey_detail_page.reload();
                }),
            );
        }

        fn bind_journeys_page(&self) {
            let leaflet = self.leaflet.clone();
            let journey_detail_page = self.journey_detail_page.clone();
            self.journeys_page.connect_closure(
                "select",
                false,
                closure_local!(move |_: JourneysPage, journey: JourneyObject| {
                    journey_detail_page.set_property("journey", journey);
                    leaflet.navigate(libadwaita::NavigationDirection::Forward);
                }),
            );
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Window {
        const NAME: &'static str = "DBWindow";
        type Type = super::Window;
        type ParentType = libadwaita::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Window {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            self.setup();
            self.bind_btn_go_back();
            self.bind_btn_reload_detail();
            self.bind_search_page();
            self.bind_journeys_page();
        }
    }

    impl WidgetImpl for Window {}
    impl WindowImpl for Window {}
    impl ApplicationWindowImpl for Window {}
    impl AdwWindowImpl for Window {}
    impl AdwApplicationWindowImpl for Window {}
}
