use gdk::glib::Object;

use crate::gui::window::Window;

gtk::glib::wrapper! {
    pub struct PreferencesWindow(ObjectSubclass<imp::PreferencesWindow>)
        @extends libadwaita::PreferencesWindow, libadwaita::Window, gtk::Window, gtk::Widget,
        @implements gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl PreferencesWindow {
    pub fn new(window: &Window) -> Self {
        Object::builder().property("transient-for", window).build()
    }
}

pub mod imp {
    use crate::config;
    use gdk::gio::Settings;
    use gdk::gio::SettingsBindFlags;
    use glib::subclass::InitializingObject;
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::CompositeTemplate;
    use libadwaita::subclass::prelude::AdwWindowImpl;
    use libadwaita::subclass::prelude::PreferencesWindowImpl;

    #[derive(CompositeTemplate)]
    #[template(resource = "/ui/preferences_window.ui")]
    pub struct PreferencesWindow {
        #[template_child]
        switch_delete_old: TemplateChild<libadwaita::SwitchRow>,
        #[template_child]
        spin_deletion_time: TemplateChild<libadwaita::SpinRow>,

        settings: Settings,
    }

    #[gtk::template_callbacks]
    impl PreferencesWindow {
        fn init_settings(&self) {
            self.settings
                .bind("delete-old", &self.switch_delete_old.get(), "active")
                .flags(SettingsBindFlags::DEFAULT)
                .build();
            self.settings
                .bind("delete-old", &self.spin_deletion_time.get(), "sensitive")
                .flags(SettingsBindFlags::DEFAULT)
                .build();
            self.settings
                .bind("deletion-time", &self.spin_deletion_time.get(), "value")
                .flags(SettingsBindFlags::NO_SENSITIVITY)
                .build();
        }

        #[template_callback]
        fn handle_deletion_time_output(&self, s: libadwaita::SpinRow) -> bool {
            // Translators: duration in hours, standalone in preferences
            s.set_text(&gettextrs::ngettext(
                "{}\u{202F}h",
                "{}\u{202F}h",
                s.value() as u32
            ).replace("{}", &s.value().to_string()));
            true
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PreferencesWindow {
        const NAME: &'static str = "DBPreferencesWindow";
        type Type = super::PreferencesWindow;
        type ParentType = libadwaita::PreferencesWindow;

        fn new() -> Self {
            Self {
                settings: Settings::new(config::BASE_ID),
                switch_delete_old: TemplateChild::default(),
                spin_deletion_time: TemplateChild::default(),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            Self::bind_template_callbacks(klass);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PreferencesWindow {
        fn constructed(&self) {
            self.parent_constructed();
            self.init_settings();
        }
    }
    impl WidgetImpl for PreferencesWindow {}
    impl WindowImpl for PreferencesWindow {}
    impl PreferencesWindowImpl for PreferencesWindow {}
    impl AdwWindowImpl for PreferencesWindow {}
}
