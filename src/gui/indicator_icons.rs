gtk::glib::wrapper! {
    pub struct IndicatorIcons(ObjectSubclass<imp::IndicatorIcons>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget;
}

pub mod imp {
    use gdk::glib::Properties;
    use glib::subclass::InitializingObject;
    use gtk::accessible::Property;
    use gtk::glib;
    use gtk::prelude::AccessibleExtManual;
    use gtk::prelude::ObjectExt;
    use gtk::subclass::prelude::*;
    use gtk::CompositeTemplate;
    use libadwaita::prelude::WidgetExt;

    use crate::backend::LateFactor;
    use crate::backend::LoadFactor;

    #[derive(CompositeTemplate, Default, Properties)]
    #[properties(wrapper_type = super::IndicatorIcons)]
    #[template(resource = "/ui/indicator_icons.ui")]
    pub struct IndicatorIcons {
        #[template_child]
        #[property(name = "load-factor", type = LoadFactor, set = Self::set_load_factor, builder(LoadFactor::default()))]
        pub(super) img_load_factor: TemplateChild<gtk::Image>,
        #[template_child]
        #[property(name = "late-factor", type = LateFactor, set = Self::set_late_factor, builder(LateFactor::default()))]
        pub(super) img_late_factor: TemplateChild<gtk::Image>,
        #[template_child]
        #[property(name = "change-platform", type = bool, set = Self::set_change_platform)]
        pub(super) img_change_platform: TemplateChild<gtk::Image>,
        #[template_child]
        #[property(name = "is-unreachable", type = bool, set = Self::set_unreachable)]
        pub(super) img_unreachable: TemplateChild<gtk::Image>,
        #[template_child]
        #[property(name = "is-cancelled", type = bool, set = Self::set_cancelled)]
        pub(super) img_cancelled: TemplateChild<gtk::Image>,
    }

    impl IndicatorIcons {
        fn set_load_factor(&self, obj: LoadFactor) {
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
                    &gettextrs::gettext("High Load"),
                    &["load-high"],
                ),
                LoadFactor::VeryHigh => self.set_icon(
                    img,
                    true,
                    "train-load-veryhigh-symbolic",
                    &gettextrs::gettext("Very High Load"),
                    &["load-very-high"],
                ),
                LoadFactor::ExceptionallyHigh => self.set_icon(
                    img,
                    true,
                    "train-load-extreme-symbolic",
                    &gettextrs::gettext("Exceptionally High Load"),
                    &["load-exceptionally-high"],
                ),
            }
        }

        fn set_late_factor(&self, obj: LateFactor) {
            let img = &self.img_late_factor;

            match obj {
                LateFactor::OnTime => self.set_icon(
                    img,
                    false,
                    "",
                    &gettextrs::gettext("On Time"),
                    &["late-on-time"],
                ),
                LateFactor::LittleLate => self.set_icon(
                    img,
                    true,
                    "delay-small-symbolic",
                    &gettextrs::gettext("Minor Delays"),
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
                    &gettextrs::gettext("Very Delayed"),
                    &["late-very-late"],
                ),
                LateFactor::ExtremelyLate => self.set_icon(
                    img,
                    true,
                    "delay-extreme-small-symbolic",
                    &gettextrs::gettext("Extremely Delayed"),
                    &["late-extremely-late"],
                ),
            }
        }

        fn set_change_platform(&self, obj: bool) {
            let img = &self.img_change_platform;

            self.set_icon(
                img,
                obj,
                "change-symbolic",
                &if obj {
                    gettextrs::gettext("Platform Changes")
                } else {
                    "".to_string()
                },
                if obj { &["change-platform"] } else { &[] },
            );
        }

        fn set_unreachable(&self, obj: bool) {
            let img = &self.img_unreachable;

            self.set_icon(
                img,
                // Do not show if cancelled.
                obj && !self.img_cancelled.get_visible(),
                "dialog-warning-symbolic",
                &if obj {
                    gettextrs::gettext("Connection Not Possible")
                } else {
                    "".to_string()
                },
                if obj { &["unreachable"] } else { &[] },
            );
        }

        fn set_cancelled(&self, obj: bool) {
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

            if obj {
                self.img_unreachable.set_visible(false);
            }
        }

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
            img.update_property(&[Property::Description(tooltip)]);

            self.recompute_visible();
        }
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

    #[glib::derived_properties]
    impl ObjectImpl for IndicatorIcons {}

    impl WidgetImpl for IndicatorIcons {}
    impl BoxImpl for IndicatorIcons {}
}
