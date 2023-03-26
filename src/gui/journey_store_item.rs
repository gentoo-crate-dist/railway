use gdk::glib::Object;

use crate::backend::Journey;

gtk::glib::wrapper! {
    pub struct JourneyStoreItem(ObjectSubclass<imp::JourneyStoreItem>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget;
}

impl JourneyStoreItem {
    pub fn new(journey: Journey) -> Self {
        Object::builder().property("journey", &journey).build()
    }
}

pub mod imp {
    use std::cell::RefCell;

    use gdk::glib::subclass::Signal;
    use gdk::glib::ParamSpec;
    use gdk::glib::ParamSpecObject;
    use gdk::glib::Value;
    use glib::subclass::InitializingObject;
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::CompositeTemplate;
    use once_cell::sync::Lazy;

    use crate::backend::Journey;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/ui/journey_store_item.ui")]
    pub struct JourneyStoreItem {
        journey: RefCell<Option<Journey>>,
    }

    #[gtk::template_callbacks]
    impl JourneyStoreItem {
        #[template_callback]
        fn handle_details(&self, _: gtk::Button) {
            self.obj().emit_by_name(
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
        fn constructed(&self) {
            self.parent_constructed();
        }

        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> =
                Lazy::new(|| vec![ParamSpecObject::builder::<Journey>("journey").build()]);
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "journey" => {
                    let obj = value.get::<Option<Journey>>().expect(
                        "Property `journey` of `JourneyStoreItem` has to be of type `Journey`",
                    );

                    self.journey.replace(obj);
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

        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| -> Vec<Signal> {
                vec![Signal::builder("details")
                    .param_types([Journey::static_type()])
                    .build()]
            });
            SIGNALS.as_ref()
        }
    }

    impl WidgetImpl for JourneyStoreItem {}
    impl BoxImpl for JourneyStoreItem {}
}
