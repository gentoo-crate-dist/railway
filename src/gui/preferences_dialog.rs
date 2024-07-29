use gdk::glib::Object;

gtk::glib::wrapper! {
    pub struct PreferencesDialog(ObjectSubclass<imp::PreferencesDialog>)
        @extends libadwaita::PreferencesDialog, libadwaita::Dialog, gtk::Widget,
        @implements gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl Default for PreferencesDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl PreferencesDialog {
    pub fn new() -> Self {
        Object::builder().build()
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
    use libadwaita::subclass::prelude::AdwDialogImpl;
    use libadwaita::subclass::prelude::PreferencesDialogImpl;

    #[derive(CompositeTemplate)]
    #[template(resource = "/ui/preferences_dialog.ui")]
    pub struct PreferencesDialog {
        #[template_child]
        switch_delete_old: TemplateChild<libadwaita::SwitchRow>,
        #[template_child]
        spin_deletion_time: TemplateChild<libadwaita::SpinRow>,

        settings: Settings,
    }

    #[gtk::template_callbacks]
    impl PreferencesDialog {
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
            // Translators: duration in hours, standalone in preferences, you might want to use a narrow no-breaking space (U+202F) in front of units
            s.set_text(
                &gettextrs::ngettext("{} h", "{} h", s.value() as u32)
                    .replace("{}", &s.value().to_string()),
            );
            true
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PreferencesDialog {
        const NAME: &'static str = "DBPreferencesDialog";
        type Type = super::PreferencesDialog;
        type ParentType = libadwaita::PreferencesDialog;

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

    impl ObjectImpl for PreferencesDialog {
        fn constructed(&self) {
            self.parent_constructed();
            self.init_settings();
        }
    }
    impl WidgetImpl for PreferencesDialog {}
    impl PreferencesDialogImpl for PreferencesDialog {}
    impl AdwDialogImpl for PreferencesDialog {}
}
