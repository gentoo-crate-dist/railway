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
    use gtk::AnyFilter;
    use gtk::CompositeTemplate;
    use gtk::Expression;
    use gtk::FilterListModel;
    use gtk::ListItem;
    use gtk::PropertyExpression;
    use gtk::SignalListItemFactory;
    use gtk::Widget;
    use once_cell::sync::Lazy;

    use crate::backend::Client;
    use crate::backend::Provider;
    use crate::config;
    use crate::gui::provider_list_item::ProviderListItem;
    use crate::gui::utility::Utility;

    #[derive(CompositeTemplate)]
    #[template(resource = "/ui/provider_popover.ui")]
    pub struct ProviderPopover {
        #[template_child]
        list_providers: TemplateChild<gtk::ListView>,
        #[template_child]
        entry_search: TemplateChild<gtk::SearchEntry>,

        current_selection: RefCell<Option<Provider>>,

        settings: Settings,
        client: RefCell<Option<Client>>,
    }

    impl Default for ProviderPopover {
        fn default() -> Self {
            Self {
                list_providers: Default::default(),
                entry_search: Default::default(),
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

            let filter_short = gtk::StringFilter::new(Some(PropertyExpression::new(
                Provider::static_type(),
                None::<Expression>,
                "short-name",
            )));
            let filter_long = gtk::StringFilter::new(Some(PropertyExpression::new(
                Provider::static_type(),
                None::<Expression>,
                "name",
            )));

            self.entry_search
                .bind_property("text", &filter_short, "search")
                .build();
            self.entry_search
                .bind_property("text", &filter_long, "search")
                .build();

            let filter = AnyFilter::new();
            filter.append(filter_short);
            filter.append(filter_long);

            let filter_model = FilterListModel::new(Some(model), Some(filter));

            let selection_model = gtk::SingleSelection::builder()
                .autoselect(false)
                .model(&filter_model)
                .build();
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

            selection_model.bind_property("selected-item", self.obj().as_ref(), "current-selection")
                .sync_create()
                .build();
            selection_model.connect_selected_item_notify(
                clone!(@strong obj => move |_| {
                    obj.popdown();
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
            let obj = self.obj();
            self.parent_constructed();

            self.entry_search.set_key_capture_widget(Some(obj.as_ref()));

            let escape_controller = gtk::EventControllerKey::new();

            escape_controller.connect_key_pressed(clone!(@weak obj as popover => @default-return glib::Propagation::Proceed, move |_, key, _, _| {
                match key {
                    gdk::Key::Escape => {
                         popover.popdown();
                    }
                    _ => (),
                }
                glib::Propagation::Proceed
            }));

            self.entry_search.add_controller(escape_controller);

            let entry_search = self.entry_search.get();
            obj.connect_closed(clone!(@weak entry_search => move |_| {
                entry_search.set_text("");
            }));
        }

        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecObject::builder::<Client>("client").build(),
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

                    if let Some(selection_model) = self.list_providers.model() {
                        let selection_model = selection_model.downcast_ref::<gtk::SingleSelection>()
                            .expect("selection model of the provider selection has to be a single selection");

                        let position = selection_model
                            .iter::<glib::Object>()
                            .position(|entry| {
                                let entry_provider = entry.ok().and_then(|object| {
                                    object.downcast::<Provider>().ok()
                                });
                                match (entry_provider, obj.clone()) {
                                    (Some(a), Some(b)) => a.id() == b.id(),
                                    (_, _) => false,
                                }
                            })
                            .map(|position| position as u32)
                            .unwrap_or(gtk::INVALID_LIST_POSITION);
                            selection_model.set_selected(position);

                        if let Some(item) = selection_model.selected_item() {
                            let provider = item.downcast_ref::<Provider>()
                                .expect("selection has to be for a provider");
                            self.settings.set_string("search-provider", &provider.id())
                                .expect("Failed to set setting `search-provider`");
                        }
                    }

                    self.current_selection.replace(obj);
                }
                "client" => {
                    let obj = value.get::<Option<Client>>().expect(
                        "Property `client` of `ProviderPopover` has to be of type `Client`",
                    );

                    self.client.replace(obj.clone());

                    if let Some(obj) = &obj {
                        self.setup_model(&self.obj());

                        self.obj()
                            .set_property("current-selection", obj.current_provider());
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
