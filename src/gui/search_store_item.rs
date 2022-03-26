use gdk::glib::Object;

gtk::glib::wrapper! {
    pub struct SearchStoreItem(ObjectSubclass<imp::SearchStoreItem>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget;
}

impl SearchStoreItem {
    pub fn new(origin: String, destination: String) -> Self {
        Object::new(&[("origin", &origin), ("destination", &destination)])
            .expect("Failed to create `SearchStoreItem`")
    }
}

pub mod imp {
    use std::cell::RefCell;

    use gdk::glib::subclass::Signal;
    use gdk::glib::ParamFlags;
    use gdk::glib::ParamSpec;
    use gdk::glib::ParamSpecString;
    use gdk::glib::Value;
    use glib::subclass::InitializingObject;
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::CompositeTemplate;
    use once_cell::sync::Lazy;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/ui/search_store_item.ui")]
    pub struct SearchStoreItem {
        origin: RefCell<Option<String>>,
        destination: RefCell<Option<String>>,
    }

    #[gtk::template_callbacks]
    impl SearchStoreItem {
        #[template_callback]
        fn handle_details(&self, _: gtk::Button) {
            self.instance().emit_by_name(
                "details",
                &[
                    self.origin
                        .borrow()
                        .as_ref()
                        .expect("`SearchStoreItem` to have a `origin`"),
                    self.destination
                        .borrow()
                        .as_ref()
                        .expect("`SearchStoreItem` to have a `destination`"),
                ],
            )
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SearchStoreItem {
        const NAME: &'static str = "DBSearchStoreItem";
        type Type = super::SearchStoreItem;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            Self::bind_template_callbacks(klass);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SearchStoreItem {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }

        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecString::new("origin", "origin", "origin", None, ParamFlags::READWRITE),
                    ParamSpecString::new(
                        "destination",
                        "destination",
                        "destination",
                        None,
                        ParamFlags::READWRITE,
                    ),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "origin" => {
                    let obj = value.get::<Option<String>>().expect(
                        "Property `origin` of `SearchStoreItem` has to be of type `String`",
                    );

                    self.origin.replace(obj);
                }
                "destination" => {
                    let obj = value.get::<Option<String>>().expect(
                        "Property `destination` of `SearchStoreItem` has to be of type `String`",
                    );

                    self.destination.replace(obj);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "origin" => self.origin.borrow().to_value(),
                "destination" => self.destination.borrow().to_value(),
                _ => unimplemented!(),
            }
        }

        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| -> Vec<Signal> {
                vec![Signal::builder(
                    "details",
                    &[String::static_type().into(), String::static_type().into()],
                    <()>::static_type().into(),
                )
                .build()]
            });
            SIGNALS.as_ref()
        }
    }

    impl WidgetImpl for SearchStoreItem {}
    impl BoxImpl for SearchStoreItem {}
}
