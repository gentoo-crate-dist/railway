use gdk::glib::Object;
use gdk::subclass::prelude::ObjectSubclassIsExt;

gtk::glib::wrapper! {
    pub struct DiscountCard(ObjectSubclass<imp::DiscountCard>);
}

impl DiscountCard {
    pub fn new(id: &str) -> DiscountCard {
        Object::builder::<Self>()
            .property("id", id)
            .build()
    }

    pub fn id(&self) -> String {
        self.imp().id.borrow().clone()
    }
}

mod imp {
    use gtk::glib;
    use once_cell::sync::Lazy;
    use std::cell::RefCell;

    use gdk::{
        glib::{
            ParamSpec, ParamSpecString, Value,
        },
        prelude::{ParamSpecBuilderExt, ToValue},
        subclass::prelude::{ObjectImpl, ObjectSubclass},
    };

    #[derive(Default)]
    pub struct DiscountCard {
        pub(super) id: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for DiscountCard {
        const NAME: &'static str = "DBDiscountCard";
        type Type = super::DiscountCard;
    }

    impl ObjectImpl for DiscountCard {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecString::builder("id").construct_only().build(),
                    ParamSpecString::builder("name").read_only().build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "id" => {
                    let obj = value
                        .get::<String>().expect("Property `id` of `DiscountCard` has to be of type `String`");
                    self.id.replace(obj);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "id" => self.id.borrow().to_value(),
                "name" => gettextrs::gettext(self.id.borrow().as_str()).to_value(),
                _ => unimplemented!(),
            }
        }
    }
}
