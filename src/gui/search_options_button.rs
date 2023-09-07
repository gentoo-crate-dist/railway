gtk::glib::wrapper! {
    pub struct SearchOptionsButton(ObjectSubclass<imp::SearchOptionsButton>)
        @extends gtk::Button, gtk::Widget,
        @implements gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget;
}

pub mod imp {
    use std::collections::HashMap;

    use gdk::gio::Settings;
    use gdk::glib::clone;
    use gdk::glib::ParamSpec;
    use gdk::glib::ParamSpecObject;
    use gdk::glib::Value;
    use glib::subclass::InitializingObject;
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::CompositeTemplate;
    use once_cell::sync::Lazy;

    use crate::backend::Remark;
    use crate::gui::search_options_window::SearchOptionsWindow;
    use crate::config;

    #[derive(CompositeTemplate)]
    #[template(resource = "/ui/search_options_button.ui")]
    pub struct SearchOptionsButton {
        settings: Settings,
    }

    impl Default for SearchOptionsButton {
        fn default() -> Self {
            Self {
                settings: Settings::new(config::BASE_ID),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SearchOptionsButton {
        const NAME: &'static str = "DBSearchOptionsButton";
        type Type = super::SearchOptionsButton;
        type ParentType = gtk::Button;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            Self::bind_template_callbacks(klass);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }
    #[gtk::template_callbacks]
    impl SearchOptionsButton {
        fn window(&self) -> crate::gui::window::Window {
            self.obj()
                .root()
                .expect("`SearchPage` to have a root")
                .dynamic_cast::<crate::gui::window::Window>()
                .expect("Root of `SearchPage` to be a `Window`.")
        }

        #[template_callback]
        fn handle_clicked(&self, _: gtk::Button) {
            let settings = SearchOptionsWindow::new(&self.window());
            settings.present();
        }

        fn bahncards() -> HashMap<usize, String> {
            HashMap::from([
                (0, gettextrs::gettext("None")),
                (1, gettextrs::gettext("BahnCard 25, 1st class")),
                (2, gettextrs::gettext("BahnCard 25, 2nd class")),
                (3, gettextrs::gettext("BahnCard 50, 1st class")),
                (4, gettextrs::gettext("BahnCard 50, 2nd class")),
                (9, gettextrs::gettext("A - VORTEILScard (with RAILPLUS)")),
                (10, gettextrs::gettext("CH - HalbtaxAbo (with RAILPLUS)")),
                (11, gettextrs::gettext("CH - HalbtaxAbo (without RAILPLUS)")),
                (
                    12,
                    gettextrs::gettext("NL - Voordeelurenabo (with RAILPLUS)"),
                ),
                (
                    13,
                    gettextrs::gettext("NL - Voordeelurenabo (without RAILPLUS)"),
                ),
                (14, gettextrs::gettext("SH-Card")),
                (15, gettextrs::gettext("CH - General-Abonnement")),
            ])
        }

        fn bahncard(number: usize) -> String {
            Self::bahncards()
                .get(&number)
                .unwrap_or(&gettextrs::gettext("Unknown"))
                .clone()
        }
    }

    impl ObjectImpl for SearchOptionsButton {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();

            // XXX: Maybe only listen for interesting things?
            self.settings.connect_changed(
                None,
                clone!(@weak obj => move |_, _| {
                    obj.notify("extra-label");
                }),
            );
        }

        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![ParamSpecObject::builder::<Remark>("extra-label")
                    .read_only()
                    .build()]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, _value: &Value, _pspec: &ParamSpec) {
            unimplemented!()
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "extra-label" => {
                    let first_class = self.settings.boolean("first-class");
                    let first_class_string = if first_class {
                        Some(gettextrs::gettext("1st class"))
                    } else {
                        Some(gettextrs::gettext("2nd class"))
                    };

                    let bahncard = self.settings.enum_("bahncard");
                    let bahncard_string = if bahncard == 0 {
                        None
                    } else {
                        Some(
                            Self::bahncard(
                                bahncard
                                    .try_into()
                                    .expect("bahncard enum to fit into usize"),
                            )
                            .to_string(),
                        )
                    };

                    let include_national_express =
                        self.settings.boolean("include-national-express");
                    let include_national = self.settings.boolean("include-national");
                    let include_regional_express =
                        self.settings.boolean("include-regional-express");
                    let include_regional = self.settings.boolean("include-regional");
                    let include_suburban = self.settings.boolean("include-suburban");
                    let include_bus = self.settings.boolean("include-bus");
                    let include_ferry = self.settings.boolean("include-ferry");
                    let include_subway = self.settings.boolean("include-subway");
                    let include_tram = self.settings.boolean("include-tram");
                    let include_taxi = self.settings.boolean("include-taxi");

                    let regional = [
                        include_regional_express,
                        include_regional,
                        include_suburban,
                        include_bus,
                        include_ferry,
                        include_subway,
                        include_tram,
                        include_taxi,
                    ];
                    let ic = [include_national_express, include_national];

                    let types_string = if regional.iter().all(|b| *b) && ic.iter().all(|b| *b) {
                        None
                    } else if regional.iter().all(|b| *b) && ic.iter().all(|b| !*b) {
                        Some(gettextrs::gettext("Regional only"))
                    } else {
                        Some(gettextrs::gettext("Custom selection"))
                    };

                    let bike_accessible = self.settings.boolean("bike-accessible");
                    let bike_accessible_string = if bike_accessible {
                        Some(gettextrs::gettext("Bike accessible"))
                    } else {
                        None
                    };

                    let direct_only = self.settings.boolean("direct-only");
                    let direct_only_string = if direct_only {
                        Some(gettextrs::gettext("Direct connection"))
                    } else {
                        None
                    };

                    [
                        first_class_string,
                        bahncard_string,
                        types_string,
                        bike_accessible_string,
                        direct_only_string,
                    ]
                    .into_iter()
                    .flatten()
                    .reduce(|s1, s2| format!("{s1}\u{00A0}\u{00B7} {s2}")) // non-breaking space and centered dot.
                    .to_value()
                }
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for SearchOptionsButton {}
    impl ButtonImpl for SearchOptionsButton {}
}
