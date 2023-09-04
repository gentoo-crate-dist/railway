use gdk::glib::Object;

use crate::gui::utility::Utility;

gtk::glib::wrapper! {
    pub struct Transition(ObjectSubclass<imp::Transition>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget;
}

impl Transition {
    pub fn new(duration: &Option<chrono::Duration>) -> Self {
        let label = if let Some(duration) = duration {
            Utility::format_duration(*duration)
        } else {
            gettextrs::gettext("Unknown")
        };
        Object::builder::<Self>()
            .property("duration", label)
            .build()
    }
}

pub mod imp {
    use std::cell::RefCell;

    use gdk::glib::ParamSpec;
    use gdk::glib::ParamSpecString;
    use gdk::glib::Value;
    use glib::subclass::InitializingObject;
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::CompositeTemplate;
    use once_cell::sync::Lazy;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/ui/transition.ui")]
    pub struct Transition {
        duration: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Transition {
        const NAME: &'static str = "DBTransition";
        type Type = super::Transition;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Transition {
        fn constructed(&self) {
            self.parent_constructed();
        }

        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> =
                Lazy::new(|| vec![ParamSpecString::builder("duration").build()]);
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "duration" => {
                    let obj = value.get::<String>().expect(
                        "Property `duration` of `Transition` has to be of type `String`",
                    );

                    self.duration.replace(obj);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "duration" => self.duration.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for Transition {}
    impl BoxImpl for Transition {}
}
