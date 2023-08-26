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

    use gdk::glib::ParamSpec;
    use gdk::glib::ParamSpecObject;
    use gdk::glib::Value;
    use gdk::prelude::ToValue;
    use glib::subclass::InitializingObject;
    use gtk::glib;
    use gtk::subclass::prelude::*;
    use gtk::CompositeTemplate;
    use once_cell::sync::Lazy;

    use crate::backend::Provider;
    use crate::gui::utility::Utility;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/ui/provider_list_item.ui")]
    pub struct ProviderListItem {
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

    impl ObjectImpl for ProviderListItem {
        fn constructed(&self) {
            self.parent_constructed();
        }

        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> =
                Lazy::new(|| vec![ParamSpecObject::builder::<Provider>("provider").build()]);
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "provider" => {
                    let obj = value.get::<Option<Provider>>().expect(
                        "Property `provider` of `DBProviderListItem` has to be of type `Provider`",
                    );

                    self.provider.replace(obj);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "provider" => self.provider.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for ProviderListItem {}
    impl BoxImpl for ProviderListItem {}
}
