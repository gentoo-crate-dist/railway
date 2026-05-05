use gdk::glib::Object;

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
}

impl Default for PlaceListItem {
    fn default() -> Self {
        Self::new()
    }
}

pub mod imp {
    use std::cell::RefCell;

    use gdk::glib::Properties;
    use gdk::glib::prelude::ObjectExt;
    use gdk::glib::subclass::Signal;
    use glib::subclass::InitializingObject;
    use gtk::CompositeTemplate;
    use gtk::glib;
    use gtk::subclass::prelude::*;
    use once_cell::sync::Lazy;

    use crate::backend::Place;
    use crate::gui::utility::Utility;

    #[derive(CompositeTemplate, Default, Properties)]
    #[template(resource = "/ui/place_list_item.ui")]
    #[properties(wrapper_type = super::PlaceListItem)]
    pub struct PlaceListItem {
        #[property(get, set)]
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

    #[glib::derived_properties]
    impl ObjectImpl for PlaceListItem {
        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> =
                Lazy::new(|| -> Vec<Signal> { vec![Signal::builder("pressed").build()] });
            SIGNALS.as_ref()
        }
    }

    impl WidgetImpl for PlaceListItem {}
    impl BoxImpl for PlaceListItem {}
}
