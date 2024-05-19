use std::cell::RefCell;

use gdk::glib::Object;
use gdk::subclass::prelude::ObjectSubclassIsExt;

gtk::glib::wrapper! {
    pub struct Price(ObjectSubclass<imp::Price>);
}

impl Price {
    pub fn new(price: rcore::Price) -> Self {
        let s: Self = Object::builder().build();
        s.imp().price.swap(&RefCell::new(Some(price)));
        s
    }

    pub fn price(&self) -> rcore::Price {
        self.imp()
            .price
            .borrow()
            .clone()
            .expect("Station not yet set up")
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
    pub struct Price {
        pub(super) price: RefCell<Option<rcore::Price>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Price {
        const NAME: &'static str = "DBPrice";
        type Type = super::Price;
    }

    impl ObjectImpl for Price {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> =
                Lazy::new(|| vec![ParamSpecString::builder("formatted").read_only().build()]);
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, _value: &Value, _pspec: &ParamSpec) {}

        fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "formatted" => {
                    let price = self.price.borrow();
                    let Some(price) = price.as_ref() else {
                        return None::<String>.to_value();
                    };

                    match price.currency.as_str() {
                        "EUR" => {
                            // Translators: How to format the currency "Euro". Do not translate in {}.
                            gettextrs::gettext("€{amount}")
                                .replace("{amount}", &format!("{:.2}", price.amount))
                                .to_value()
                        }
                        "USD" => {
                            // Translators: How to format the currency "Dollar (US)". Do not translate in {}.
                            gettextrs::gettext("${amount}")
                                .replace("{amount}", &format!("{:.2}", price.amount))
                                .to_value()
                        }
                        // XXX: Add other currencies here
                        s => {
                            // Translators: How to format unknown currency "currency". Do not translate in {}.
                            gettextrs::gettext("{currency} {amount}")
                                .replace("{amount}", &format!("{:.2}", price.amount))
                                .replace("{currency}", s)
                                .to_value()
                        }
                    }
                }
                _ => unimplemented!(),
            }
        }
    }
}
