gtk::glib::wrapper! {
    pub struct AltLabel(ObjectSubclass<imp::AltLabel>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget;
}

pub mod imp {
    use std::cell::RefCell;
    use std::marker::PhantomData;

    use gdk::glib::Properties;
    use gdk::glib::clone;
    use glib::subclass::InitializingObject;
    use gtk::CompositeTemplate;
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;

    use crate::gui::utility::Utility;

    #[derive(CompositeTemplate, Default, Properties)]
    #[properties(wrapper_type = super::AltLabel)]
    #[template(resource = "/ui/alt_label.ui")]
    pub struct AltLabel {
        #[template_child]
        label_main: TemplateChild<gtk::Label>,
        #[template_child]
        label_alt: TemplateChild<gtk::Label>,

        #[property(get, set)]
        main: RefCell<Option<String>>,
        #[property(get, set)]
        alt: RefCell<Option<String>>,

        #[property(name = "is-different", get = Self::is_different)]
        _is_different: PhantomData<bool>,
    }

    impl AltLabel {
        fn is_different(&self) -> bool {
            let main = self.main.borrow();
            let alt = self.alt.borrow();
            main.is_some() && alt.is_some() && main.as_ref() != alt.as_ref()
        }

        fn connect_equal(&self, obj: &super::AltLabel) {
            obj.connect_notify_local(
                Some("main"),
                clone!(
                    #[strong(rename_to = label_main)]
                    self.label_main,
                    #[strong(rename_to = label_alt)]
                    self.label_alt,
                    move |obj, _| {
                        let main = obj.property::<Option<String>>("main");
                        let alt = obj.property::<Option<String>>("alt");
                        if main == alt {
                            label_main.add_css_class("main-label-on-time");
                            label_alt.add_css_class("alt-label-on-time");
                            label_main.remove_css_class("main-label-late");
                            label_alt.remove_css_class("alt-label-late");
                        } else {
                            label_main.add_css_class("main-label-late");
                            label_alt.add_css_class("alt-label-late");
                            label_main.remove_css_class("main-label-on-time");
                            label_alt.remove_css_class("alt-label-on-time");
                        }
                        obj.notify("is-different");
                    }
                ),
            );
            obj.connect_notify_local(
                Some("alt"),
                clone!(
                    #[strong(rename_to = label_main)]
                    self.label_main,
                    #[strong(rename_to = label_alt)]
                    self.label_alt,
                    move |obj, _| {
                        let main = obj.property::<Option<String>>("main");
                        let alt = obj.property::<Option<String>>("alt");
                        if main == alt {
                            label_main.add_css_class("main-label-on-time");
                            label_alt.add_css_class("alt-label-on-time");
                            label_main.remove_css_class("main-label-late");
                            label_alt.remove_css_class("alt-label-late");
                        } else {
                            label_main.add_css_class("main-label-late");
                            label_alt.add_css_class("alt-label-late");
                            label_main.remove_css_class("main-label-on-time");
                            label_alt.remove_css_class("alt-label-on-time");
                        }
                        obj.notify("is-different");
                    }
                ),
            );
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AltLabel {
        const NAME: &'static str = "DBAltLabel";
        type Type = super::AltLabel;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            Utility::bind_template_callbacks(klass);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for AltLabel {
        fn constructed(&self) {
            self.parent_constructed();
            self.connect_equal(&self.obj());
        }
    }

    impl WidgetImpl for AltLabel {}
    impl BoxImpl for AltLabel {}
}
