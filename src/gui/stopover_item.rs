use gdk::glib::Object;

use super::objects::StopoverObject;

gtk::glib::wrapper! {
    pub struct StopoverItem(ObjectSubclass<imp::StopoverItem>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget;
}

impl StopoverItem {
    pub fn new(stopover: &StopoverObject) -> Self {
        Object::new(&[("stopover", stopover)]).expect("Failed to create StopoverItem")
    }
}

pub mod imp {
    use std::cell::RefCell;

    use gdk::glib::ParamFlags;
    use gdk::glib::ParamSpec;
    use gdk::glib::ParamSpecObject;
    use gdk::glib::Value;
    use glib::subclass::InitializingObject;
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::CompositeTemplate;
    use once_cell::sync::Lazy;

    use crate::gui::objects::StopoverObject;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/ui/stopover_item.ui")]
    pub struct StopoverItem {
        stopover: RefCell<Option<StopoverObject>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for StopoverItem {
        const NAME: &'static str = "DBStopoverItem";
        type Type = super::StopoverItem;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for StopoverItem {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }

        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![ParamSpecObject::new(
                    "stopover",
                    "stopover",
                    "stopover",
                    StopoverObject::static_type(),
                    ParamFlags::READWRITE,
                )]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "stopover" => {
                    let obj = value.get::<Option<StopoverObject>>().expect(
                        "Property `stopover` of `StopoverItem` has to be of type `StopoverObject`",
                    );

                    self.stopover.replace(obj);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "stopover" => self.stopover.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for StopoverItem {}
    impl BoxImpl for StopoverItem {}
}
