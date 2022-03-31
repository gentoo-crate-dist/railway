use gdk::glib::Object;

gtk::glib::wrapper! {
    pub struct PreferencesWindow(ObjectSubclass<imp::PreferencesWindow>)
        @extends libadwaita::PreferencesWindow, libadwaita::Window, gtk::Window, gtk::Widget,
        @implements gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl PreferencesWindow {
    pub fn new() -> Self {
        Object::new(&[]).expect("Failed to create PreferencesWindow")
    }
}

pub mod imp {
    use gdk::gio::Settings;
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
        dropdown_bahncard: TemplateChild<gtk::ComboBox>,

        settings: Settings,
    }

    #[gtk::template_callbacks]
    impl PreferencesWindow {
        fn init_settings(&self) {
            self.dropdown_bahncard
                .set_active_id(Some(&self.settings.enum_("bahncard").to_string()));
        }

        #[template_callback]
        fn handle_bahncard_dropdown(&self, dropdown: gtk::ComboBox) {
            let id_str = dropdown.property::<String>("active-id");
            let id = id_str.parse::<i32>().expect("active-id must be i32");
            self.settings
                .set_enum("bahncard", id)
                .expect("Failed to set enum value");
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PreferencesWindow {
        const NAME: &'static str = "DBPreferencesWindow";
        type Type = super::PreferencesWindow;
        type ParentType = libadwaita::PreferencesWindow;

        fn new() -> Self {
            Self {
                settings: Settings::new("de.schmidhuberj.DieBahn"),
                dropdown_bahncard: TemplateChild::default(),
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
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            self.init_settings();
        }
    }
    impl WidgetImpl for PreferencesWindow {}
    impl WindowImpl for PreferencesWindow {}
    impl PreferencesWindowImpl for PreferencesWindow {}
    impl AdwWindowImpl for PreferencesWindow {}
}
