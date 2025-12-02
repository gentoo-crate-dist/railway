use gdk::glib::Object;

gtk::glib::wrapper! {
    pub struct ProviderListItem(ObjectSubclass<imp::ProviderListItem>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget;
}

impl ProviderListItem {
    pub fn new() -> Self {
        Object::builder().build()
    }
}

impl Default for ProviderListItem {
    fn default() -> Self {
        Self::new()
    }
}

pub mod imp {
    use std::cell::RefCell;

    use gdk::glib::Properties;
    use glib::subclass::InitializingObject;
    use gtk::CompositeTemplate;
    use gtk::glib;
    use gtk::prelude::ObjectExt;
    use gtk::subclass::prelude::*;

    use crate::backend::Provider;
    use crate::gui::utility::Utility;

    #[derive(CompositeTemplate, Default, Properties)]
    #[properties(wrapper_type = super::ProviderListItem)]
    #[template(resource = "/ui/provider_list_item.ui")]
    pub struct ProviderListItem {
        #[property(get, set)]
        provider: RefCell<Option<Provider>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ProviderListItem {
        const NAME: &'static str = "DBProviderListItem";
        type Type = super::ProviderListItem;
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
    impl ObjectImpl for ProviderListItem {}

    impl WidgetImpl for ProviderListItem {}
    impl BoxImpl for ProviderListItem {}
}
