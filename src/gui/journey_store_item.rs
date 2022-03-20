use gdk::glib::Object;

use super::objects::JourneyObject;

gtk::glib::wrapper! {
    pub struct JourneyStoreItem(ObjectSubclass<imp::JourneyStoreItem>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget;
}

impl JourneyStoreItem {
    pub fn new(journey: JourneyObject) -> Self {
        Object::new(&[("journey", &journey)]).expect("Failed to create `JourneyStoreItem`")
    }
}

pub mod imp {
    use std::cell::RefCell;

    use gdk::glib::subclass::Signal;
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

    use crate::gui::objects::JourneyObject;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/ui/journey_store_item.ui")]
    pub struct JourneyStoreItem {
        journey: RefCell<Option<JourneyObject>>,
    }

    #[gtk::template_callbacks]
    impl JourneyStoreItem {
        #[template_callback]
        fn handle_details(&self, _: gtk::Button) {
            self.instance().emit_by_name(
                "details",
                &[self
                    .journey
                    .borrow()
                    .as_ref()
                    .expect("`JourneyStoreItem` to have a `journey`")],
            )
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for JourneyStoreItem {
        const NAME: &'static str = "DBJourneyStoreItem";
        type Type = super::JourneyStoreItem;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            Self::bind_template_callbacks(klass);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for JourneyStoreItem {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }

        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![ParamSpecObject::new(
                    "journey",
                    "journey",
                    "journey",
                    JourneyObject::static_type(),
                    ParamFlags::READWRITE,
                )]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "journey" => {
                    let obj = value.get::<Option<JourneyObject>>().expect(
                        "Property `journey` of `JourneyStoreItem` has to be of type `JourneyObject`",
                    );

                    self.journey.replace(obj);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "journey" => self.journey.borrow().to_value(),
                _ => unimplemented!(),
            }
        }

        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| -> Vec<Signal> {
                vec![Signal::builder(
                    "details",
                    &[JourneyObject::static_type().into()],
                    <()>::static_type().into(),
                )
                .build()]
            });
            SIGNALS.as_ref()
        }
    }

    impl WidgetImpl for JourneyStoreItem {}
    impl BoxImpl for JourneyStoreItem {}
}
