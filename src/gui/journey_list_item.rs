use gdk::glib::subclass::prelude::ObjectSubclassIsExt;
use gdk::glib::Object;
use std::borrow::Borrow;

gtk::glib::wrapper! {
    pub struct JourneyListItem(ObjectSubclass<imp::JourneyListItem>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget;
}

impl JourneyListItem {
    pub fn new() -> Self {
        Object::builder().build()
    }

    pub fn get_destination_box(&self) -> gtk::Box {
        self.imp().destination_box.borrow().get().clone()
    }
}

impl Default for JourneyListItem {
    fn default() -> Self {
        Self::new()
    }
}

pub mod imp {
    use std::cell::RefCell;

    use gdk::glib::ParamSpec;
    use gdk::glib::ParamSpecBoolean;
    use gdk::glib::ParamSpecObject;
    use gdk::glib::Value;
    use glib::subclass::InitializingObject;
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::CompositeTemplate;
    use once_cell::sync::Lazy;

    use crate::backend::Journey;
    use crate::gui::utility::Utility;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/ui/journey_list_item.ui")]
    pub struct JourneyListItem {
        #[template_child]
        pub(super) destination_box: TemplateChild<gtk::Box>,
        #[template_child]
        from_time: TemplateChild<gtk::Box>,
        #[template_child]
        to_time: TemplateChild<gtk::Box>,

        journey: RefCell<Option<Journey>>,
    }

    impl JourneyListItem {
        fn set_compact(&self, compact: bool) {
            let (orientation, spacing) = if compact {
                (gtk::Orientation::Vertical, 0)
            } else {
                (gtk::Orientation::Horizontal, 6)
            };
            self.from_time.set_orientation(orientation);
            self.from_time.set_spacing(spacing);
            self.to_time.set_orientation(orientation);
            self.to_time.set_spacing(spacing);
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for JourneyListItem {
        const NAME: &'static str = "DBJourneyListItem";
        type Type = super::JourneyListItem;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            Utility::bind_template_callbacks(klass);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for JourneyListItem {
        fn constructed(&self) {
            self.parent_constructed();
        }

        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecObject::builder::<Journey>("journey").build(),
                    ParamSpecBoolean::builder("compact").write_only().build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "journey" => {
                    let obj = value.get::<Option<Journey>>().expect(
                        "Property `journey` of `JourneyListItem` has to be of type `Journey`",
                    );

                    if obj
                        .as_ref()
                        .is_some_and(|j| j.is_unreachable() || j.is_cancelled())
                    {
                        self.obj().add_css_class("dim-label");
                    } else {
                        self.obj().remove_css_class("dim-label");
                    }

                    self.journey.replace(obj);
                }
                "compact" => {
                    let obj = value
                        .get::<bool>()
                        .expect("Property `compact` of `JourneyListItem` has to be of type `bool`");

                    self.set_compact(obj);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "journey" => self.journey.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for JourneyListItem {}
    impl BoxImpl for JourneyListItem {}
}
