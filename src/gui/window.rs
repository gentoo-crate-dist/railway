use gtk::glib::Object;

gtk::glib::wrapper! {
    pub struct Window(ObjectSubclass<imp::Window>)
        @extends libadwaita::ApplicationWindow, gtk::ApplicationWindow, libadwaita::Window, gtk::Window, gtk::Widget,
        @implements gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl Window {
    pub fn new(app: &libadwaita::Application) -> Self {
        Object::builder::<Self>()
            .property("application", app)
            .build()
    }
}

pub mod imp {
    use std::cell::RefCell;

    use gdk::gio::SimpleAction;
    use gdk::gio::SimpleActionGroup;
    use gdk::glib::ParamSpec;
    use gdk::glib::ParamSpecObject;
    use gdk::glib::Value;
    use glib::subclass::InitializingObject;
    use gtk::glib;
    use gtk::glib::clone;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::CompositeTemplate;
    use gtk::Inhibit;
    use libadwaita::subclass::prelude::AdwApplicationWindowImpl;
    use libadwaita::subclass::prelude::AdwWindowImpl;
    use once_cell::sync::Lazy;

    use crate::backend::HafasClient;
    use crate::backend::Journey;
    use crate::backend::JourneysResult;
    use crate::backend::Leg;
    use crate::backend::Place;
    use crate::gui::alt_label::AltLabel;
    use crate::gui::date_time_picker::DateTimePicker;
    use crate::gui::journey_detail_page::JourneyDetailPage;
    use crate::gui::journeys_page::JourneysPage;
    use crate::gui::preferences_window::PreferencesWindow;
    use crate::gui::provider_popover::ProviderPopover;
    use crate::gui::search_options_button::SearchOptionsButton;
    use crate::gui::search_page::SearchPage;
    use crate::gui::station_entry::StationEntry;
    use crate::gui::stores::journey_store::JourneysStore;
    use crate::gui::stores::search_store::SearchesStore;
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
        #[template_child]
        store_searches: TemplateChild<SearchesStore>,

        client: RefCell<HafasClient>,
    }

    #[gtk::template_callbacks]
    impl Window {
        fn setup(&self) {
            self.store_journeys.setup();
            self.store_searches.setup();
        }

        fn setup_actions(&self, obj: &super::Window) {
            let action_settings = SimpleAction::new("settings", None);
            action_settings.connect_activate(clone!(@weak obj as window => move |_, _| {
                let settings = PreferencesWindow::new(&window);
                settings.show();
            }));
            let action_about = SimpleAction::new("about", None);
            action_about.connect_activate(clone!(@weak obj as window => move |_, _| {
                let about_dialog = libadwaita::AboutWindow::builder()
                    .transient_for(&window)
                    .developers(
                        env!("CARGO_PKG_AUTHORS")
                            .split(';')
                            .map(|s| s.to_string())
                            .collect::<Vec<_>>(),
                    )
                    .comments(env!("CARGO_PKG_DESCRIPTION"))
                    .copyright(glib::markup_escape_text(
                        include_str!("../../NOTICE")
                            .to_string()
                            .lines()
                            .next()
                            .unwrap_or_default(),
                    ))
                    .license_type(gtk::License::Gpl30)
                    .application_icon("icon")
                    .application_name("DieBahn")
                    .translator_credits(gettextrs::gettext("translators"))
                    .version(env!("CARGO_PKG_VERSION"))
                    .website(env!("CARGO_PKG_HOMEPAGE"))
                    .build();
                about_dialog.add_link("GitLab", "https://gitlab.com/schmiddi-on-mobile/diebahn");
                about_dialog.show();
            }));

            let actions = SimpleActionGroup::new();
            obj.insert_action_group("win", Some(&actions));
            actions.add_action(&action_settings);
            actions.add_action(&action_about);
        }

        #[template_callback]
        fn handle_go_back(&self) {
            self.leaflet.navigate(libadwaita::NavigationDirection::Back);
        }

        #[template_callback]
        fn handle_details(&self, journey: Journey) {
            self.leaflet
                .navigate(libadwaita::NavigationDirection::Forward);
            self.leaflet
                .navigate(libadwaita::NavigationDirection::Forward);
            self.journey_detail_page.set_property("journey", journey);
            self.journey_detail_page.reload();
        }

        #[template_callback]
        fn handle_search_page(&self, journeys_result: JourneysResult) {
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
        fn handle_journeys_page(&self, journey: Journey) {
            self.journey_detail_page.set_property("journey", journey);
            self.leaflet
                .navigate(libadwaita::NavigationDirection::Forward);
        }

        #[template_callback]
        fn handle_journey_store(&self, _: gtk::Button) {
            if let Some(journey) = self
                .journey_detail_page
                .property::<Option<Journey>>("journey")
            {
                self.store_journeys.store(journey)
            }
        }

        #[template_callback]
        fn handle_journey_store_add(&self, journey: Journey) {
            self.search_page.add_journey_store(journey);
        }

        #[template_callback]
        fn handle_journey_store_remove(&self, journey: Journey) {
            self.search_page.remove_journey_store(journey);
        }

        #[template_callback]
        fn handle_searches_store(&self, _: gtk::Button) {
            if let Some(journeys_result) = self
                .journeys_page
                .property::<Option<JourneysResult>>("journeys-result")
            {
                let origin = journeys_result.journeys()[0]
                    .property::<Leg>("first-leg")
                    .property::<Place>("origin")
                    .name()
                    .unwrap_or_default();
                let destination = journeys_result.journeys()[0]
                    .property::<Leg>("last-leg")
                    .property::<Place>("destination")
                    .name()
                    .unwrap_or_default();
                self.store_searches.store(origin, destination);
            }
        }

        #[template_callback]
        fn handle_searches_store_add(&self, origin: String, destination: String) {
            self.search_page.add_search_store(origin, destination);
        }

        #[template_callback]
        fn handle_searches_store_remove(&self, origin: String, destination: String) {
            self.search_page.remove_search_store(origin, destination);
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Window {
        const NAME: &'static str = "DBWindow";
        type Type = super::Window;
        type ParentType = libadwaita::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            AltLabel::ensure_type();
            ProviderPopover::ensure_type();
            DateTimePicker::ensure_type();
            StationEntry::ensure_type();
            JourneysStore::ensure_type();
            SearchesStore::ensure_type();
            SearchOptionsButton::ensure_type();
            Self::bind_template(klass);
            Self::bind_template_callbacks(klass);
            Utility::bind_template_callbacks(klass);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Window {
        fn constructed(&self) {
            self.parent_constructed();
            self.setup_actions(&self.obj());
            self.setup();
        }

        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![ParamSpecObject::builder::<HafasClient>("client")
                    .read_only()
                    .build()]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, _value: &Value, _pspec: &ParamSpec) {}

        fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "client" => self.client.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for Window {}
    impl WindowImpl for Window {
        fn close_request(&self) -> Inhibit {
            self.store_journeys.get().flush();
            self.store_searches.get().flush();
            Inhibit(false)
        }
    }
    impl ApplicationWindowImpl for Window {}
    impl AdwWindowImpl for Window {}
    impl AdwApplicationWindowImpl for Window {}
}
