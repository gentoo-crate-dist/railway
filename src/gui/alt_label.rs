gtk::glib::wrapper! {
    pub struct AltLabel(ObjectSubclass<imp::AltLabel>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget;
}

pub mod imp {
    use std::cell::RefCell;

    use gdk::glib::ParamFlags;
    use gdk::glib::ParamSpec;
    use gdk::glib::ParamSpecString;
    use gdk::glib::Value;
    use gdk::glib::clone;
    use glib::subclass::InitializingObject;
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::CompositeTemplate;
    use once_cell::sync::Lazy;

    use crate::gui::utility::Utility;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/ui/alt_label.ui")]
    pub struct AltLabel {
        #[template_child]
        label_main: TemplateChild<gtk::Label>,
        #[template_child]
        label_alt: TemplateChild<gtk::Label>,

        main: RefCell<Option<String>>,
        alt: RefCell<Option<String>>,
    }

    impl AltLabel {
        fn connect_equal(&self, obj: &super::AltLabel) {
            obj.connect_notify_local(None, 
                                     clone!(@strong self.label_main as label_main, 
                                            @strong self.label_alt as label_alt => move |obj, _| {
                let main = obj.property::<Option<String>>("main");
                let alt = obj.property::<Option<String>>("alt");
                if main == alt {
                    label_alt.add_css_class("success");
                    label_alt.remove_css_class("error");
                } else {
                    label_alt.add_css_class("error");
                    label_alt.remove_css_class("success");
                }
            }));
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

    impl ObjectImpl for AltLabel {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            self.connect_equal(obj);
        }

        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecString::new("main", "main", "main", None, ParamFlags::READWRITE),
                    ParamSpecString::new("alt", "alt", "alt", None, ParamFlags::READWRITE),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "main" => {
                    let obj = value
                        .get::<Option<String>>()
                        .expect("Property `main` of `AltLabel` has to be of type `String`");

                    self.main.replace(obj);
                }
                "alt" => {
                    let obj = value
                        .get::<Option<String>>()
                        .expect("Property `alt` of `AltLabel` has to be of type `String`");

                    self.alt.replace(obj);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "main" => self.main.borrow().to_value(),
                "alt" => self.alt.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for AltLabel {}
    impl BoxImpl for AltLabel {}
}
