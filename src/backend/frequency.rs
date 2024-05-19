use std::cell::RefCell;

use gdk::glib::Object;
use gdk::subclass::prelude::ObjectSubclassIsExt;

gtk::glib::wrapper! {
    pub struct Frequency(ObjectSubclass<imp::Frequency>);
}

impl Frequency {
    pub fn new(frequency: rcore::Frequency) -> Self {
        let s: Self = Object::builder().build();
        s.imp().frequency.swap(&RefCell::new(Some(frequency)));
        s
    }

    pub fn frequency(&self) -> Option<rcore::Frequency> {
        self.imp().frequency.borrow().clone()
    }
}

mod imp {
    use gtk::glib;
    use std::cell::RefCell;

    use gdk::{glib::subclass::object::ObjectImpl, subclass::prelude::ObjectSubclass};

    #[derive(Default)]
    pub struct Frequency {
        pub(super) frequency: RefCell<Option<rcore::Frequency>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Frequency {
        const NAME: &'static str = "DBFrequency";
        type Type = super::Frequency;
    }

    impl ObjectImpl for Frequency {}
}
