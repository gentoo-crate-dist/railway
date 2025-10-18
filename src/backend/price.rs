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
        glib::Properties,
        prelude::ObjectExt,
        subclass::prelude::{DerivedObjectProperties, ObjectImpl, ObjectSubclass},
    };

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::Price)]
    pub struct Price {
        #[property(name="formatted", type=Option<String>, get=Self::formatted)]
        pub(super) price: RefCell<Option<rcore::Price>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Price {
        const NAME: &'static str = "DBPrice";
        type Type = super::Price;
    }

    impl Price {
        fn formatted(&self) -> Option<String> {
            let price = self.price.borrow();
            let price = price.as_ref()?;

            Some(match price.currency.as_str() {
                "EUR" => {
                    // Translators: How to format the currency "Euro". Do not translate in {}.
                    gettextrs::gettext("€{amount}")
                        .replace("{amount}", &format!("{:.2}", price.amount))
                }
                "USD" => {
                    // Translators: How to format the currency "Dollar (US)". Do not translate in {}.
                    gettextrs::gettext("${amount}")
                        .replace("{amount}", &format!("{:.2}", price.amount))
                }
                // XXX: Add other currencies here
                s => {
                    // Translators: How to format unknown currency "currency". Do not translate in {}.
                    gettextrs::gettext("{currency} {amount}")
                        .replace("{amount}", &format!("{:.2}", price.amount))
                        .replace("{currency}", s)
                }
            })
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for Price {}
}
