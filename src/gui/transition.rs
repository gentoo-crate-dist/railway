use gdk::glib::Object;

use crate::gui::utility::Utility;

use crate::backend::Place;

gtk::glib::wrapper! {
    pub struct Transition(ObjectSubclass<imp::Transition>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget;
}

impl Transition {
    pub fn new(walking_time: &Option<chrono::Duration>,
        waiting_time: &Option<chrono::Duration>,
        has_walk: bool,
        is_last_mile: bool,
        final_destination: &Option<Place>) -> Self {
        let walking_time_label = walking_time.map(|duration| Utility::format_duration(duration));
        let final_destination_label = final_destination.clone().and_then(|stop| stop.name());
        let waiting_time_label = waiting_time.map(|duration| Utility::format_duration(duration));
        Object::builder::<Self>()
            .property("walking-time", walking_time_label)
            .property("waiting-time", waiting_time_label)
            .property("is-last-mile", is_last_mile)
            .property("has-walk", has_walk)
            .property("final-destination", final_destination_label)
            .build()
    }
}

pub mod imp {
    use std::cell::Cell;
    use std::cell::RefCell;

    use gdk::glib::object::ObjectExt;
    use gdk::glib::ParamSpec;
    use gdk::glib::ParamSpecString;
    use gdk::glib::ParamSpecBoolean;
    use gdk::glib::Value;
    use glib::subclass::InitializingObject;
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::CompositeTemplate;
    use once_cell::sync::Lazy;
    use crate::gui::utility::Utility;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/ui/transition.ui")]
    pub struct Transition {
        walking_time: RefCell<Option<String>>,
        waiting_time: RefCell<Option<String>>,
        is_last_mile: Cell<bool>,
        has_walk: Cell<bool>,
        final_destination: RefCell<Option<String>>,

        #[template_child]
        destination_box: TemplateChild<gtk::Box>,
        #[template_child]
        destination_label: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Transition {
        const NAME: &'static str = "DBTransition";
        type Type = super::Transition;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            Utility::bind_template_callbacks(klass);
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
                Lazy::new(|| {
                vec![
                    ParamSpecString::builder("walking-time").build(),
                    ParamSpecString::builder("waiting-time").build(),
                    ParamSpecBoolean::builder("is-last-mile").build(),
                    ParamSpecBoolean::builder("has-walk").build(),
                    ParamSpecString::builder("final-destination").build(),

                    ParamSpecString::builder("icon").read_only().build(),
                    ParamSpecString::builder("label").read_only().build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "walking-time" => {
                    let obj = value.get::<Option<String>>().expect(
                        "Property `walking-time` of `Transition` has to be of type `String`",
                    );

                    self.obj().notify("label");
                    self.walking_time.replace(obj);
                }
                "waiting-time" => {
                    let obj = value.get::<Option<String>>().expect(
                        "Property `waiting-time` of `Transition` has to be of type `String`",
                    );

                    self.obj().notify("label");
                    self.waiting_time.replace(obj);
                }
                "is-last-mile" => {
                    let obj = value.get::<bool>().expect(
                        "Property `is-last-mile` of `Transition` has to be of type `bool`",
                    );

                    self.obj().notify("icon");
                    self.is_last_mile.replace(obj);
                }
                "has-walk" => {
                    let obj = value.get::<bool>().expect(
                        "Property `has-walk` of `Transition` has to be of type `bool`",
                    );

                    self.obj().notify("icon");
                    self.has_walk.replace(obj);
                }
                "final-destination" => {
                    let obj = value.get::<Option<String>>().expect(
                        "Property `final-destination` of `Transition` has to be of type `String`",
                    );

                    self.destination_box.set_visible(obj.is_some());
                    self.destination_label.set_label(&obj.clone().unwrap_or_default());

                    self.final_destination.replace(obj);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "walking-time" => self.walking_time.borrow().to_value(),
                "waiting-time" => self.waiting_time.borrow().to_value(),
                "is-last-mile" => self.is_last_mile.get().to_value(),
                "has-walk" => self.has_walk.get().to_value(),
                "final-destination" => self.final_destination.borrow().to_value(),

                "icon" => {
                    (if !self.has_walk.get() && !self.is_last_mile.get() {
                        "change-symbolic"
                    } else {
                        "walking-symbolic"
                    }).to_value()
                },
                "label" => {
                    (match (self.walking_time.borrow().clone(),
                        self.waiting_time.borrow().clone()) {
                        (Some(walking), Some(waiting)) => gettextrs::gettext!("{} walk, {} waiting", walking, waiting),
                        (None, Some(waiting)) => gettextrs::gettext!("Transition: {}", waiting),
                        (Some(walking), None) => gettextrs::gettext!("{} walk", walking),
                        (None, None) => gettextrs::gettext("Transition")
                    }).to_value()
                },

                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for Transition {}
    impl BoxImpl for Transition {}
}
