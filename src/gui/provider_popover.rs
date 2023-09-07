gtk::glib::wrapper! {
    pub struct ProviderPopover(ObjectSubclass<imp::ProviderPopover>)
        @extends gtk::Popover, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Native, gtk::ShortcutManager;
}

pub mod imp {
    use std::cell::RefCell;

    use gdk::gio::Settings;
    use gdk::glib::clone;
    use gdk::glib::ParamSpec;
    use gdk::glib::ParamSpecObject;
    use gdk::glib::Value;
    use glib::subclass::InitializingObject;
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::CompositeTemplate;
    use gtk::ListItem;
    use gtk::SignalListItemFactory;
    use gtk::Widget;
    use once_cell::sync::Lazy;

    use crate::backend::HafasClient;
    use crate::backend::Provider;
    use crate::gui::provider_list_item::ProviderListItem;
    use crate::gui::utility::Utility;
    use crate::config;

    #[derive(CompositeTemplate)]
    #[template(resource = "/ui/provider_popover.ui")]
    pub struct ProviderPopover {
        #[template_child]
        list_providers: TemplateChild<gtk::ListView>,

        current_selection: RefCell<Option<Provider>>,

        settings: Settings,
        client: RefCell<Option<HafasClient>>,
    }

    impl Default for ProviderPopover {
        fn default() -> Self {
            Self {
                list_providers: Default::default(),
                settings: Settings::new(config::BASE_ID),
                current_selection: RefCell::new(None),
                client: Default::default(),
            }
        }
    }

    #[gtk::template_callbacks]
    impl ProviderPopover {
        fn setup_model(&self, obj: &super::ProviderPopover) {
            let model = self
                .client
                .borrow()
                .as_ref()
                .expect("The client to be set up")
                .providers();
            let selection_model = gtk::NoSelection::new(Some(model));
            self.list_providers.get().set_model(Some(&selection_model));

            let factory = SignalListItemFactory::new();
            factory.connect_setup(move |_, list_item| {
                let provider_item = ProviderListItem::new();
                let list_item = list_item
                    .downcast_ref::<ListItem>()
                    .expect("The factory item to be a `ListItem`");

                list_item.set_child(Some(&provider_item));
                list_item.property_expression("item").bind(
                    &provider_item,
                    "provider",
                    Widget::NONE,
                );
            });
            self.list_providers.set_factory(Some(&factory));
            self.list_providers.set_single_click_activate(true);

            self.list_providers.connect_activate(
                clone!(@strong obj, @weak self.settings as settings => move |list_view, position| {
                    let model = list_view.model().expect("The model has to exist.");
                    let provider_object = model
                        .item(position)
                        .expect("The item has to exist.")
                        .downcast::<Provider>()
                        .expect("The item has to be an `Provider`.");

                    settings.set_string("search-provider", &provider_object.id()).expect("Failed to set setting `search-provider`");
                    obj.popdown();
                    obj.set_property("current-selection", provider_object);
                }),
            );
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ProviderPopover {
        const NAME: &'static str = "DBProviderPopover";
        type Type = super::ProviderPopover;
        type ParentType = gtk::Popover;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            Self::bind_template_callbacks(klass);
            Utility::bind_template_callbacks(klass);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ProviderPopover {
        fn constructed(&self) {
            self.parent_constructed();
        }

        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecObject::builder::<HafasClient>("client").build(),
                    ParamSpecObject::builder::<Provider>("current-selection").build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "current-selection" => {
                    let obj = value.get::<Option<Provider>>().expect(
                        "Property `current-selection` of `ProviderPopover` has to be of type `Provider`",
                    );

                    self.current_selection.replace(obj);
                }
                "client" => {
                    let obj = value.get::<Option<HafasClient>>().expect(
                        "Property `client` of `ProviderPopover` has to be of type `HafasClient`",
                    );

                    let set = obj.is_some();

                    if let Some(obj) = &obj {
                        self.obj()
                            .set_property("current-selection", obj.current_provider());
                    }

                    self.client.replace(obj);

                    if set {
                        self.setup_model(&self.obj());
                    }
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "client" => self.client.borrow().to_value(),
                "current-selection" => self.current_selection.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for ProviderPopover {}
    impl PopoverImpl for ProviderPopover {}
}
