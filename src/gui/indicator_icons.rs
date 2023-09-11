gtk::glib::wrapper! {
    pub struct IndicatorIcons(ObjectSubclass<imp::IndicatorIcons>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget;
}

pub mod imp {
    use gdk::glib::ParamSpec;
    use gdk::glib::ParamSpecBoolean;
    use gdk::glib::ParamSpecEnum;
    use gdk::glib::Value;
    use gdk::prelude::ParamSpecBuilderExt;
    use glib::subclass::InitializingObject;
    use gtk::accessible::Property;
    use gtk::glib;
    use gtk::prelude::AccessibleExtManual;
    use gtk::subclass::prelude::*;
    use gtk::CompositeTemplate;
    use libadwaita::prelude::WidgetExt;
    use once_cell::sync::Lazy;

    use crate::backend::LateFactor;
    use crate::backend::LoadFactor;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/ui/indicator_icons.ui")]
    pub struct IndicatorIcons {
        #[template_child]
        pub(super) img_load_factor: TemplateChild<gtk::Image>,
        #[template_child]
        pub(super) img_late_factor: TemplateChild<gtk::Image>,
        #[template_child]
        pub(super) img_change_platform: TemplateChild<gtk::Image>,
        #[template_child]
        pub(super) img_unreachable: TemplateChild<gtk::Image>,
        #[template_child]
        pub(super) img_cancelled: TemplateChild<gtk::Image>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for IndicatorIcons {
        const NAME: &'static str = "DBIndicatorIcons";
        type Type = super::IndicatorIcons;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl IndicatorIcons {
        fn images(&self) -> Vec<gtk::Image> {
            vec![
                self.img_load_factor.get(),
                self.img_late_factor.get(),
                self.img_change_platform.get(),
                self.img_unreachable.get(),
                self.img_cancelled.get(),
            ]
        }

        fn recompute_visible(&self) {
            self.obj()
                .set_visible(self.images().iter().any(|i| i.get_visible()));
        }

        fn set_icon(
            &self,
            img: &gtk::Image,
            visible: bool,
            icon_name: &str,
            tooltip: &str,
            css_classes: &[&str],
        ) {
            img.set_visible(visible);
            img.set_icon_name(Some(icon_name));
            img.set_css_classes(css_classes);
            img.set_tooltip_text(Some(tooltip));
            img.update_property(&[Property::Label(tooltip)]);

            self.recompute_visible();
        }
    }

    impl ObjectImpl for IndicatorIcons {
        fn constructed(&self) {
            self.parent_constructed();
        }

        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecEnum::builder::<LoadFactor>("load-factor")
                        .write_only()
                        .build(),
                    ParamSpecEnum::builder::<LateFactor>("late-factor")
                        .write_only()
                        .build(),
                    ParamSpecBoolean::builder("change-platform")
                        .write_only()
                        .build(),
                    ParamSpecBoolean::builder("is-unreachable")
                        .write_only()
                        .build(),
                    ParamSpecBoolean::builder("is-cancelled")
                        .write_only()
                        .build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "load-factor" => {
                    let obj = value.get::<LoadFactor>().expect(
                        "Property `load-factor` of `IndicatorIcons` has to be of type `LoadFactor`",
                    );
                    let img = &self.img_load_factor;

                    match obj {
                        LoadFactor::Unknown => self.set_icon(img, false, "", "", &["load-unknown"]),
                        LoadFactor::LowToMedium => {
                            self.set_icon(img, false, "", "", &["load-low-to-medium"])
                        }
                        LoadFactor::High => self.set_icon(
                            img,
                            true,
                            "train-load-high-symbolic",
                            &gettextrs::gettext("High load"),
                            &["load-high"],
                        ),
                        LoadFactor::VeryHigh => self.set_icon(
                            img,
                            true,
                            "train-load-veryhigh-symbolic",
                            &gettextrs::gettext("Very high load"),
                            &["load-very-high"],
                        ),
                        LoadFactor::ExceptionallyHigh => self.set_icon(
                            img,
                            true,
                            "train-load-extreme-symbolic",
                            &gettextrs::gettext("Exceptionally high load"),
                            &["load-exceptionally-high"],
                        ),
                    }
                }
                "late-factor" => {
                    let obj = value.get::<LateFactor>().expect(
                        "Property `late-factor` of `IndicatorIcons` has to be of type `LateFactor`",
                    );
                    let img = &self.img_late_factor;

                    match obj {
                        LateFactor::OnTime => self.set_icon(
                            img,
                            false,
                            "",
                            &gettextrs::gettext("On time"),
                            &["late-on-time"],
                        ),
                        LateFactor::LittleLate => self.set_icon(
                            img,
                            true,
                            "delay-small-symbolic",
                            &gettextrs::gettext("Minor delays"),
                            &["late-little-late"],
                        ),
                        LateFactor::Late => self.set_icon(
                            img,
                            true,
                            "delay-small-symbolic",
                            &gettextrs::gettext("Delayed"),
                            &["late-late"],
                        ),
                        LateFactor::VeryLate => self.set_icon(
                            img,
                            true,
                            "delay-long-small-symbolic",
                            &gettextrs::gettext("Very delayed"),
                            &["late-very-late"],
                        ),
                        LateFactor::ExtremelyLate => self.set_icon(
                            img,
                            true,
                            "delay-extreme-small-symbolic",
                            &gettextrs::gettext("Extremely delayed"),
                            &["late-extremely-late"],
                        ),
                    }
                }
                "change-platform" => {
                    let obj = value.get::<bool>().expect(
                        "Property `change-platform` of `IndicatorIcons` has to be of type `bool`",
                    );
                    let img = &self.img_change_platform;

                    self.set_icon(
                        img,
                        obj,
                        "change-symbolic",
                        &if obj {
                            gettextrs::gettext("Platform changed")
                        } else {
                            "".to_string()
                        },
                        if obj { &["change-platform"] } else { &[] },
                    );
                }
                "is-unreachable" => {
                    let obj = value.get::<bool>().expect(
                        "Property `is-unreachable` of `IndicatorIcons` has to be of type `bool`",
                    );
                    let img = &self.img_unreachable;

                    self.set_icon(
                        img,
                        obj,
                        "dialog-warning-symbolic",
                        &if obj {
                            gettextrs::gettext("Unreachable")
                        } else {
                            "".to_string()
                        },
                        if obj { &["unreachable"] } else { &[] },
                    );
                }
                "is-cancelled" => {
                    let obj = value.get::<bool>().expect(
                        "Property `is-cancelled` of `IndicatorIcons` has to be of type `bool`",
                    );
                    let img = &self.img_cancelled;

                    self.set_icon(
                        img,
                        obj,
                        "dialog-error-symbolic",
                        &if obj {
                            gettextrs::gettext("Cancelled")
                        } else {
                            "".to_string()
                        },
                        if obj { &["cancelled"] } else { &[] },
                    );
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, _pspec: &ParamSpec) -> Value {
            unimplemented!()
        }
    }

    impl WidgetImpl for IndicatorIcons {}
    impl BoxImpl for IndicatorIcons {}
}
