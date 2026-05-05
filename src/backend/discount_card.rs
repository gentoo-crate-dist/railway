use gdk::glib::Object;

gtk::glib::wrapper! {
    pub struct DiscountCard(ObjectSubclass<imp::DiscountCard>);
}

impl DiscountCard {
    pub fn new(id: &str) -> DiscountCard {
        Object::builder::<Self>().property("id", id).build()
    }
}

mod imp {
    use std::cell::RefCell;

    use gtk::glib;

    use gdk::{
        glib::Properties,
        prelude::ObjectExt,
        subclass::prelude::{DerivedObjectProperties, ObjectImpl, ObjectSubclass},
    };

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::DiscountCard)]
    pub struct DiscountCard {
        #[property(get, set)]
        #[property(name = "name", type = String, get = |s: &Self| gettextrs::gettext(s.id.borrow().as_str()))]
        pub(super) id: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for DiscountCard {
        const NAME: &'static str = "DBDiscountCard";
        type Type = super::DiscountCard;
    }

    #[glib::derived_properties]
    impl ObjectImpl for DiscountCard {}
}
