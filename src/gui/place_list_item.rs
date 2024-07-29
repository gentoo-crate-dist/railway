use gdk::glib::prelude::ObjectExt;
use gdk::glib::Object;

use crate::backend::Place;

gtk::glib::wrapper! {
    pub struct PlaceListItem(ObjectSubclass<imp::PlaceListItem>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget;
}

impl PlaceListItem {
    pub fn new() -> Self {
        Object::builder().build()
    }

    pub fn place(&self) -> Option<Place> {
        self.property("place")
    }
}

impl Default for PlaceListItem {
    fn default() -> Self {
        Self::new()
    }
}

pub mod imp {
    use std::cell::RefCell;

    use gdk::glib::prelude::ObjectExt;
    use gdk::glib::subclass::Signal;
    use gdk::glib::ParamSpec;
    use gdk::glib::ParamSpecObject;
    use gdk::glib::Value;
    use gdk::prelude::ToValue;
    use glib::subclass::InitializingObject;
    use gtk::glib;
    use gtk::subclass::prelude::*;
    use gtk::CompositeTemplate;
    use once_cell::sync::Lazy;

    use crate::backend::Place;
    use crate::gui::utility::Utility;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/ui/place_list_item.ui")]
    pub struct PlaceListItem {
        place: RefCell<Option<Place>>,
    }

    #[gtk::template_callbacks]
    impl PlaceListItem {
        #[template_callback]
        fn handle_pressed(&self) {
            self.obj().emit_by_name::<()>("pressed", &[]);
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PlaceListItem {
        const NAME: &'static str = "DBPlaceListItem";
        type Type = super::PlaceListItem;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            Self::bind_template_callbacks(klass);
            Utility::bind_template_callbacks(klass);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PlaceListItem {
        fn constructed(&self) {
            self.parent_constructed();
        }

        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> =
                Lazy::new(|| vec![ParamSpecObject::builder::<Place>("place").build()]);
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "place" => {
                    let obj = value
                        .get::<Option<Place>>()
                        .expect("Property `place` of `DBPlaceListItem` has to be of type `Place`");

                    self.place.replace(obj);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "place" => self.place.borrow().to_value(),
                _ => unimplemented!(),
            }
        }

        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> =
                Lazy::new(|| -> Vec<Signal> { vec![Signal::builder("pressed").build()] });
            SIGNALS.as_ref()
        }
    }

    impl WidgetImpl for PlaceListItem {}
    impl BoxImpl for PlaceListItem {}
}
