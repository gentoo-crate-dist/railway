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
        fn set_icon(
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
                        LoadFactor::Unknown => Self::set_icon(
                            img,
                            false,
                            "network-cellular-signal-none-symbolic",
                            &gettextrs::gettext("Unknown load"),
                            &["load-unknown"],
                        ),
                        LoadFactor::LowToMedium => Self::set_icon(
                            img,
                            true,
                            "network-cellular-signal-weak-symbolic",
                            &gettextrs::gettext("Low or medium load"),
                            &["load-low-to-medium"],
                        ),
                        LoadFactor::High => Self::set_icon(
                            img,
                            true,
                            "network-cellular-signal-ok-symbolic",
                            &gettextrs::gettext("High load"),
                            &["load-high"],
                        ),
                        LoadFactor::VeryHigh => Self::set_icon(
                            img,
                            true,
                            "network-cellular-signal-good-symbolic",
                            &gettextrs::gettext("Very high load"),
                            &["load-very-high"],
                        ),
                        LoadFactor::ExceptionallyHigh => Self::set_icon(
                            img,
                            true,
                            "network-cellular-signal-excellent-symbolic",
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
                        LateFactor::OnTime => Self::set_icon(
                            img,
                            false,
                            "face-angel-symbolic",
                            &gettextrs::gettext("On time"),
                            &["late-on-time"],
                        ),
                        LateFactor::LittleLate => Self::set_icon(
                            img,
                            true,
                            "face-plain-symbolic",
                            &gettextrs::gettext("Minor delays"),
                            &["late-little-late"],
                        ),
                        LateFactor::Late => Self::set_icon(
                            img,
                            true,
                            "face-sad-symbolic",
                            &gettextrs::gettext("Delayed"),
                            &["late-late"],
                        ),
                        LateFactor::VeryLate => Self::set_icon(
                            img,
                            true,
                            "face-angry-symbolic",
                            &gettextrs::gettext("Very delayed"),
                            &["late-very-late"],
                        ),
                        LateFactor::ExtremelyLate => Self::set_icon(
                            img,
                            true,
                            "face-monkey-symbolic",
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

                    Self::set_icon(
                        img,
                        obj,
                        "change-symbolic",
                        &if obj {
                            gettextrs::gettext("Platform changed")
                        } else {
                            gettextrs::gettext("No platform changes")
                        },
                        if obj { &["change-platform"] } else { &[] },
                    );
                }
                "is-unreachable" => {
                    let obj = value.get::<bool>().expect(
                        "Property `is-unreachable` of `IndicatorIcons` has to be of type `bool`",
                    );
                    let img = &self.img_unreachable;

                    Self::set_icon(
                        img,
                        obj,
                        "dialog-warning-symbolic",
                        &if obj {
                            gettextrs::gettext("Reachable")
                        } else {
                            gettextrs::gettext("Unreachable")
                        },
                        if obj { &["unreachable"] } else { &[] },
                    );
                }
                "is-cancelled" => {
                    let obj = value.get::<bool>().expect(
                        "Property `is-cancelled` of `IndicatorIcons` has to be of type `bool`",
                    );
                    let img = &self.img_cancelled;

                    Self::set_icon(
                        img,
                        obj,
                        "dialog-warning-symbolic",
                        &if obj {
                            gettextrs::gettext("Reachable")
                        } else {
                            gettextrs::gettext("cancelled")
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
