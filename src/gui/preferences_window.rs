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
    use gdk::gio::Settings;
    use gdk::gio::SettingsBindFlags;
    use glib::subclass::InitializingObject;
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::CompositeTemplate;
    use libadwaita::subclass::prelude::AdwWindowImpl;
    use libadwaita::subclass::prelude::PreferencesWindowImpl;
    use crate::config;

    #[derive(CompositeTemplate)]
    #[template(resource = "/ui/preferences_window.ui")]
    pub struct PreferencesWindow {
        #[template_child]
        switch_delete_old: TemplateChild<gtk::Switch>,
        #[template_child]
        spin_deletion_time: TemplateChild<gtk::SpinButton>,

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
                .bind("deletion-time", &self.spin_deletion_time.get(), "value")
                .flags(SettingsBindFlags::DEFAULT)
                .build();
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
