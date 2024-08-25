use gdk::glib::Object;

gtk::glib::wrapper! {
    pub struct SearchOptionsDialog(ObjectSubclass<imp::SearchOptionsDialog>)
        @extends libadwaita::PreferencesDialog, libadwaita::Dialog, gtk::Widget,
        @implements gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl Default for SearchOptionsDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchOptionsDialog {
    pub fn new() -> Self {
        Object::builder().build()
    }
}

pub mod imp {
    use crate::backend::DiscountCard;
    use crate::config;
    use gdk::gio::Settings;
    use gdk::gio::SettingsBindFlags;
    use gdk::glib::clone;
    use glib::subclass::InitializingObject;
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::CompositeTemplate;
    use gtk::Expression;
    use gtk::PropertyExpression;
    use libadwaita::prelude::ComboRowExt;
    use libadwaita::subclass::prelude::AdwDialogImpl;
    use libadwaita::subclass::prelude::PreferencesDialogImpl;

    #[derive(CompositeTemplate)]
    #[template(resource = "/ui/search_options_dialog.ui")]
    pub struct SearchOptionsDialog {
        #[template_child]
        dropdown_bahncard: TemplateChild<libadwaita::ComboRow>,

        #[template_child]
        toggle_first_class: TemplateChild<gtk::ToggleButton>,
        #[template_child]
        toggle_second_class: TemplateChild<gtk::ToggleButton>,

        #[template_child]
        switch_bike_accessible: TemplateChild<libadwaita::SwitchRow>,
        #[template_child]
        spin_transfer_time: TemplateChild<libadwaita::SpinRow>,
        #[template_child]
        switch_direct_only: TemplateChild<libadwaita::SwitchRow>,

        #[template_child]
        switch_national_express: TemplateChild<libadwaita::SwitchRow>,
        #[template_child]
        switch_regional: TemplateChild<libadwaita::SwitchRow>,
        #[template_child]
        switch_suburban: TemplateChild<libadwaita::SwitchRow>,
        #[template_child]
        switch_bus: TemplateChild<libadwaita::SwitchRow>,
        #[template_child]
        switch_ferry: TemplateChild<libadwaita::SwitchRow>,
        #[template_child]
        switch_subway: TemplateChild<libadwaita::SwitchRow>,
        #[template_child]
        switch_tram: TemplateChild<libadwaita::SwitchRow>,
        #[template_child]
        switch_cablecar: TemplateChild<libadwaita::SwitchRow>,
        #[template_child]
        switch_taxi: TemplateChild<libadwaita::SwitchRow>,

        settings: Settings,
    }

    #[gtk::template_callbacks]
    impl SearchOptionsDialog {
        fn init_settings(&self) {
            if self.settings.boolean("first-class") {
                self.toggle_first_class.set_active(true);
            } else {
                self.toggle_second_class.set_active(true);
            }

            let model_bahncard = gdk::gio::ListStore::new::<DiscountCard>();
            if let Some(settings_schema) = self.settings.settings_schema() {
                let bahncard_range = settings_schema.key("bahncard").range();

                assert!(bahncard_range.is_container());
                assert_eq!(
                    bahncard_range.child_value(0).get::<String>().expect(""),
                    "enum"
                );

                for card_id in bahncard_range
                    .child_value(1)
                    .as_variant()
                    .expect("bahncard's enum's values to be boxed")
                    .array_iter_str()
                    .expect("bahncard's enum's values to be of string type")
                {
                    model_bahncard.append(&DiscountCard::new(card_id))
                }
            }
            self.dropdown_bahncard
                .get()
                .set_model(Some(&model_bahncard));
            self.dropdown_bahncard
                .set_expression(Some(PropertyExpression::new(
                    DiscountCard::static_type(),
                    None::<Expression>,
                    "name",
                )));
            self.settings
                .bind("bahncard", &self.dropdown_bahncard.get(), "selected")
                .mapping(clone!(
                    #[weak]
                    model_bahncard,
                    #[upgrade_or_panic]
                    move |variant, value_type| {
                        assert_eq!(value_type, glib::types::Type::U32);

                        variant.str().map(|card_id| {
                            let position =
                                model_bahncard.iter::<glib::Object>().position(|entry| {
                                    card_id
                                        == entry
                                            .expect("our model to only contain GObjects")
                                            .downcast::<DiscountCard>()
                                            .expect("our model to only contain DiscountCards")
                                            .id()
                                });

                            assert!(position.is_some());

                            (position.unwrap() as u32).to_value()
                        })
                    }
                ))
                .set_mapping(clone!(
                    #[weak]
                    model_bahncard,
                    #[upgrade_or_panic]
                    move |value, variant_type| {
                        assert_eq!(variant_type.as_str(), "s");

                        match value.get::<u32>() {
                            Ok(position) => glib::variant::Variant::parse(
                                Some(&variant_type),
                                &format!(
                                    "'{}'",
                                    model_bahncard
                                        .item(position)
                                        .and_downcast::<DiscountCard>()?
                                        .id()
                                ),
                            )
                            .ok(),
                            _ => None,
                        }
                    }
                ))
                .flags(SettingsBindFlags::DEFAULT)
                .build();

            self.settings
                .bind(
                    "bike-accessible",
                    &self.switch_bike_accessible.get(),
                    "active",
                )
                .flags(SettingsBindFlags::DEFAULT)
                .build();
            self.settings
                .bind("transfer-time", &self.spin_transfer_time.get(), "value")
                .flags(SettingsBindFlags::DEFAULT)
                .build();
            self.settings
                .bind("direct-only", &self.switch_direct_only.get(), "active")
                .flags(SettingsBindFlags::DEFAULT)
                .build();

            self.settings
                .bind(
                    "include-national-express",
                    &self.switch_national_express.get(),
                    "active",
                )
                .flags(SettingsBindFlags::DEFAULT)
                .build();
            self.settings
                .bind("include-regional", &self.switch_regional.get(), "active")
                .flags(SettingsBindFlags::DEFAULT)
                .build();
            self.settings
                .bind("include-suburban", &self.switch_suburban.get(), "active")
                .flags(SettingsBindFlags::DEFAULT)
                .build();
            self.settings
                .bind("include-bus", &self.switch_bus.get(), "active")
                .flags(SettingsBindFlags::DEFAULT)
                .build();
            self.settings
                .bind("include-ferry", &self.switch_ferry.get(), "active")
                .flags(SettingsBindFlags::DEFAULT)
                .build();
            self.settings
                .bind("include-subway", &self.switch_subway.get(), "active")
                .flags(SettingsBindFlags::DEFAULT)
                .build();
            self.settings
                .bind("include-tram", &self.switch_tram.get(), "active")
                .flags(SettingsBindFlags::DEFAULT)
                .build();
            self.settings
                .bind("include-subway", &self.switch_subway.get(), "active")
                .flags(SettingsBindFlags::DEFAULT)
                .build();
            self.settings
                .bind("include-tram", &self.switch_tram.get(), "active")
                .flags(SettingsBindFlags::DEFAULT)
                .build();
            self.settings
                .bind("include-cablecar", &self.switch_cablecar.get(), "active")
                .flags(SettingsBindFlags::DEFAULT)
                .build();
            self.settings
                .bind("include-taxi", &self.switch_taxi.get(), "active")
                .flags(SettingsBindFlags::DEFAULT)
                .build();
        }

        #[template_callback]
        fn handle_first_class(&self, toggle: gtk::ToggleButton) {
            let active = toggle.is_active();
            self.settings
                .set_boolean("first-class", active)
                .expect("Failed to set first-class value");
        }

        #[template_callback]
        fn handle_transfer_time_output(&self, s: libadwaita::SpinRow) -> bool {
            // Translators: you might want to use a narrow no-breaking space (U+202F) in front of units
            s.set_text(&gettextrs::gettext("{} min").replace("{}", &s.value().to_string()));
            true
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SearchOptionsDialog {
        const NAME: &'static str = "DBSearchOptionsDialog";
        type Type = super::SearchOptionsDialog;
        type ParentType = libadwaita::PreferencesDialog;

        fn new() -> Self {
            Self {
                settings: Settings::new(config::BASE_ID),
                dropdown_bahncard: TemplateChild::default(),
                switch_bike_accessible: TemplateChild::default(),
                spin_transfer_time: TemplateChild::default(),
                switch_direct_only: TemplateChild::default(),
                toggle_first_class: TemplateChild::default(),
                toggle_second_class: TemplateChild::default(),
                switch_national_express: TemplateChild::default(),
                switch_regional: TemplateChild::default(),
                switch_suburban: TemplateChild::default(),
                switch_bus: TemplateChild::default(),
                switch_ferry: TemplateChild::default(),
                switch_subway: TemplateChild::default(),
                switch_tram: TemplateChild::default(),
                switch_cablecar: TemplateChild::default(),
                switch_taxi: TemplateChild::default(),
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

    impl ObjectImpl for SearchOptionsDialog {
        fn constructed(&self) {
            self.parent_constructed();
            self.init_settings();
        }
    }
    impl WidgetImpl for SearchOptionsDialog {}
    impl PreferencesDialogImpl for SearchOptionsDialog {}
    impl AdwDialogImpl for SearchOptionsDialog {}
}
