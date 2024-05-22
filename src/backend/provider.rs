use gdk::{glib::Object, prelude::ObjectExt};

gtk::glib::wrapper! {
    pub struct Provider(ObjectSubclass<imp::Provider>);
}

impl Provider {
    pub fn new(id: &'static str, short_name: &str, name: Option<&str>, has_icon: bool) -> Provider {
        let icon_name = if has_icon {
            id.to_lowercase()
        } else {
            crate::config::APP_ID.to_string()
        };
        Object::builder::<Self>()
            .property("id", id)
            .property("short-name", short_name)
            .property("name", name)
            .property("icon-name", &icon_name)
            .build()
    }

    pub fn id(&self) -> String {
        self.property("id")
    }
}

mod imp {
    use gtk::glib;
    use std::cell::RefCell;

    use gdk::{
        glib::{ParamSpec, ParamSpecString, Value},
        prelude::{ParamSpecBuilderExt, ToValue},
        subclass::prelude::{ObjectImpl, ObjectSubclass},
    };
    use once_cell::sync::Lazy;

    #[derive(Default)]
    pub struct Provider {
        id: RefCell<String>,
        short_name: RefCell<String>,
        name: RefCell<Option<String>>,
        icon_name: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Provider {
        const NAME: &'static str = "DBProvider";
        type Type = super::Provider;
    }

    impl ObjectImpl for Provider {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecString::builder("id").construct_only().build(),
                    ParamSpecString::builder("name").construct_only().build(),
                    ParamSpecString::builder("short-name")
                        .construct_only()
                        .build(),
                    ParamSpecString::builder("icon-name")
                        .construct_only()
                        .build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "id" => {
                    let obj = value
                        .get::<String>()
                        .expect("Property `id` of `Provider` has to be of type `String`");
                    self.id.replace(obj);
                }
                "short-name" => {
                    let obj = value
                        .get::<String>()
                        .expect("Property `short-name` of `Provider` has to be of type `String`");
                    self.short_name.replace(obj);
                }
                "name" => {
                    let obj = value
                        .get::<Option<String>>()
                        .expect("Property `name` of `Provider` has to be of type `String`");
                    self.name.replace(obj);
                }
                "icon-name" => {
                    let obj = value
                        .get::<String>()
                        .expect("Property `icon-name` of `Provider` has to be of type `String`");
                    self.icon_name.replace(obj);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "id" => self.id.borrow().to_value(),
                "short-name" => self.short_name.borrow().to_value(),
                "name" => self.name.borrow().to_value(),
                "icon-name" => self.icon_name.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}
