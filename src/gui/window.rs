use gdk::gio::Settings;
use gdk::glib;
use gdk::subclass::prelude::ObjectSubclassIsExt;
use gdk::prelude::SettingsExt;
use gtk::glib::Object;
use gtk::prelude::{GtkApplicationExt, GtkWindowExt};

use crate::Error;
use crate::gui::error::error_to_toast;
use crate::config::BASE_ID;

gtk::glib::wrapper! {
    pub struct Window(ObjectSubclass<imp::Window>)
        @extends libadwaita::ApplicationWindow, gtk::ApplicationWindow, libadwaita::Window, gtk::Window, gtk::Widget,
        @implements gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl Window {
    pub fn new(app: &libadwaita::Application) -> Self {
        app.set_accels_for_action("win.settings", &["<Control>comma"]);
        app.set_accels_for_action("win.show-help-overlay", &["<Control>question"]);
        app.set_accels_for_action("window.close", &["<Control>q"]);

        app.set_accels_for_action("journey-list.bookmark", &["<Control>s"]);
        app.set_accels_for_action("journey-details.bookmark", &["<Control>d"]);
        app.set_accels_for_action("journey-details.reload", &["<Control>r"]);

        Object::builder::<Self>()
            .property("application", app)
            .build()
    }

    fn save_window_size(&self) -> Result<(), glib::BoolError> {
        let settings = Settings::new(BASE_ID);

        let (width, height) = self.default_size();

        settings.set_int("window-width", width)?;
        settings.set_int("window-height", height)?;

        settings.set_boolean("is-maximized", self.is_maximized())?;

        Ok(())
    }

    fn load_window_size(&self) {
        let settings = Settings::new(BASE_ID);

        let width = settings.int("window-width");
        let height = settings.int("window-height");
        let is_maximized = settings.boolean("is-maximized");

        self.set_default_size(width, height);

        if is_maximized {
            self.maximize();
        }
    }

    pub fn display_error_toast(&self, err: Error) {
        let toast_overlay = self.imp().toast_overlay.get();
        error_to_toast(&toast_overlay, err);
    }
}

pub mod imp {
    use std::cell::RefCell;

    use gdk::gio::SimpleAction;
    use gdk::gio::SimpleActionGroup;
    use gdk::glib::ParamSpec;
    use gdk::glib::ParamSpecObject;
    use gdk::glib::Value;
    use glib::signal::Propagation;
    use glib::subclass::InitializingObject;
    use gtk::glib;
    use gtk::glib::clone;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::Builder;
    use gtk::CompositeTemplate;
    use gtk::ShortcutsWindow;
    use gtk::ToggleButton;
    use libadwaita::subclass::prelude::AdwApplicationWindowImpl;
    use libadwaita::subclass::prelude::AdwWindowImpl;
    use once_cell::sync::Lazy;

    use crate::backend::DiscountCard;
    use crate::backend::HafasClient;
    use crate::backend::Journey;
    use crate::backend::JourneysResult;
    use crate::backend::Place;
    use crate::config;
    use crate::gui::alt_label::AltLabel;
    use crate::gui::date_time_picker::DateTimePicker;
    use crate::gui::frequency_label::FrequencyLabel;
    use crate::gui::indicator_icons::IndicatorIcons;
    use crate::gui::journey_detail_page::JourneyDetailPage;
    use crate::gui::journeys_page::JourneysPage;
    use crate::gui::preferences_window::PreferencesWindow;
    use crate::gui::provider_popover::ProviderPopover;
    use crate::gui::refresh_button::RefreshButton;
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
        search_view: TemplateChild<libadwaita::NavigationSplitView>,
        #[template_child]
        result_view: TemplateChild<libadwaita::NavigationSplitView>,

        #[template_child]
        search_page: TemplateChild<SearchPage>,
        #[template_child]
        journeys_page: TemplateChild<JourneysPage>,
        #[template_child]
        journey_detail_page: TemplateChild<JourneyDetailPage>,

        #[template_child]
        btn_bookmark_search: TemplateChild<gtk::ToggleButton>,
        #[template_child]
        btn_bookmark_journey: TemplateChild<gtk::ToggleButton>,

        #[template_child]
        store_journeys: TemplateChild<JourneysStore>,
        #[template_child]
        store_searches: TemplateChild<SearchesStore>,

        #[template_child]
        pub toast_overlay: TemplateChild<libadwaita::ToastOverlay>,

        client: RefCell<HafasClient>,
    }

    #[gtk::template_callbacks]
    impl Window {
        fn setup(&self) {
            self.store_journeys.setup();
            self.store_searches.setup();

            if config::PROFILE == "Devel" {
                self.obj().add_css_class("devel");
            }

            self.obj().load_window_size();
        }

        fn setup_actions(&self, obj: &super::Window) {
            let action_settings = SimpleAction::new("settings", None);
            action_settings.connect_activate(clone!(@weak obj as window => move |_, _| {
                let settings = PreferencesWindow::new(&window);
                settings.present();
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
                    .application_icon(config::APP_ID)
                    .application_name("Railway")
                    .translator_credits(gettextrs::gettext("translators"))
                    .version(env!("CARGO_PKG_VERSION"))
                    .website(env!("CARGO_PKG_HOMEPAGE"))
                    .build();
                about_dialog.add_link("GitLab", "https://gitlab.com/schmiddi-on-mobile/railway");
                about_dialog.present();
            }));

            let action_show_help_overlay = SimpleAction::new("show-help-overlay", None);
            action_show_help_overlay.connect_activate(clone!(@weak obj as window => move |_, _| {
                let builder = Builder::from_resource("/ui/shortcuts.ui");
                let shortcuts_window: ShortcutsWindow = builder
                    .object("help_overlay")
                    .expect("shortcuts.ui to have at least one object help_overlay");
                shortcuts_window.set_transient_for(Some(&window));
                shortcuts_window.present();
            }));

            let actions = SimpleActionGroup::new();
            obj.insert_action_group("win", Some(&actions));
            actions.add_action(&action_settings);
            actions.add_action(&action_about);
            actions.add_action(&action_show_help_overlay);

            let action_journey_list_bookmark = SimpleAction::new("bookmark", None);
            action_journey_list_bookmark.connect_activate(clone!(@weak self as s => move |_, _| {
                s.handle_searches_store();
            }));

            let actions_journey_list = SimpleActionGroup::new();
            obj.insert_action_group("journey-list", Some(&actions_journey_list));
            actions_journey_list.add_action(&action_journey_list_bookmark);

            let action_journey_details_bookmark = SimpleAction::new("bookmark", None);
            action_journey_details_bookmark.connect_activate(
                clone!(@weak self as s => move |_, _| {
                    s.handle_journey_store();
                }),
            );
            let action_journey_details_reload = SimpleAction::new("reload", None);
            action_journey_details_reload.connect_activate(clone!(@weak self as s => move |_, _| {
                s.handle_journey_reload();
            }));

            let actions_journey_details = SimpleActionGroup::new();
            obj.insert_action_group("journey-details", Some(&actions_journey_details));
            actions_journey_details.add_action(&action_journey_details_bookmark);
            actions_journey_details.add_action(&action_journey_details_reload);
        }

        #[template_callback]
        fn handle_details(&self, journey: Journey) {
            self.search_view.set_show_content(true);
            self.result_view.set_show_content(true);
            self.journey_detail_page.set_property("journey", journey);
            self.journey_detail_page.reload();
        }

        #[template_callback]
        fn handle_search_page(&self, journeys_result: JourneysResult) {
            self.journeys_page
                .set_property("journeys-result", journeys_result);
            self.search_view.set_show_content(true);
            self.result_view.set_show_content(false);
        }

        #[template_callback]
        fn handle_journey_reload(&self) {
            self.journey_detail_page.reload();
        }

        #[template_callback]
        fn handle_journeys_page(&self, journey: Journey) {
            self.journey_detail_page.set_property("journey", journey);
            self.search_view.set_show_content(true);
            self.result_view.set_show_content(true);
        }

        #[template_callback]
        fn handle_journey_store(&self) {
            if let Some(journey) = self
                .journey_detail_page
                .property::<Option<Journey>>("journey")
            {
                self.store_journeys.store(journey.clone());
                self.btn_bookmark_journey
                    .set_active(self.store_journeys.contains(&journey));
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
        fn has_journey_stored(&self, journey: Option<Journey>) -> bool {
            if let Some(journey) = journey {
                self.store_journeys.contains(&journey)
            } else {
                false
            }
        }

        #[template_callback]
        fn has_search_stored(&self, source: Option<Place>, destination: Option<Place>) -> bool {
            if let (Some(source), Some(destination)) = (
                source.and_then(|p| p.name()),
                destination.and_then(|p| p.name()),
            ) {
                self.store_searches.contains(&source, &destination)
            } else {
                false
            }
        }

        #[template_callback]
        fn handle_searches_store(&self) {
            if let Some(journeys_result) = self
                .journeys_page
                .property::<Option<JourneysResult>>("journeys-result")
            {
                let origin = journeys_result
                    .source()
                    .and_then(|p| p.name())
                    .unwrap_or_default();
                let destination = journeys_result
                    .destination()
                    .and_then(|p| p.name())
                    .unwrap_or_default();
                self.store_searches
                    .store(origin.clone(), destination.clone());
                self.btn_bookmark_search
                    .set_active(self.store_searches.contains(origin, destination));
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

        #[template_callback]
        fn handle_bookmark_button_icon(&self, _param: ParamSpec, button: ToggleButton) {
            if button.is_active() {
                button.set_icon_name("bookmark-toggled-symbolic");
            } else {
                button.set_icon_name("bookmark-untoggled-symbolic");
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Window {
        const NAME: &'static str = "DBWindow";
        type Type = super::Window;
        type ParentType = libadwaita::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            DiscountCard::ensure_type();
            AltLabel::ensure_type();
            ProviderPopover::ensure_type();
            DateTimePicker::ensure_type();
            StationEntry::ensure_type();
            JourneysStore::ensure_type();
            SearchesStore::ensure_type();
            SearchOptionsButton::ensure_type();
            IndicatorIcons::ensure_type();
            RefreshButton::ensure_type();
            FrequencyLabel::ensure_type();
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
            let obj = self.obj();
            self.parent_constructed();
            self.setup_actions(&obj);
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
        fn close_request(&self) -> Propagation {
            if let Err(err) = self.obj().save_window_size() {
                log::warn!("Failed to save window state, {}", &err);
            }

            self.store_journeys.get().flush();
            self.store_searches.get().flush();
            Propagation::Proceed
        }
    }
    impl ApplicationWindowImpl for Window {}
    impl AdwWindowImpl for Window {}
    impl AdwApplicationWindowImpl for Window {}
}
