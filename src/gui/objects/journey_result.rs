use std::cell::RefCell;

use gdk::glib::Object;
use gdk::subclass::prelude::ObjectSubclassIsExt;
use hafas_rest::JourneysResult;

gtk::glib::wrapper! {
    pub struct JourneysResultObject(ObjectSubclass<imp::JourneysResultObject>);
}

impl JourneysResultObject {
    pub fn new(journeys_result: JourneysResult) -> Self {
        let s: Self = Object::new(&[]).expect("Failed to create `JourneysResultObject`.");
        s.imp()
            .journeys_result
            .swap(&RefCell::new(Some(journeys_result)));
        s
    }

    pub fn journeys_result(&self) -> JourneysResult {
        self.imp()
            .journeys_result
            .borrow()
            .as_ref()
            .expect("JourneysResultObject has not yet been set up")
            .clone()
    }
}

mod imp {
    use gtk::glib;
    use hafas_rest::JourneysResult;
    use std::cell::RefCell;

    use gdk::subclass::prelude::{ObjectImpl, ObjectSubclass};

    #[derive(Default, Clone)]
    pub struct JourneysResultObject {
        pub(super) journeys_result: RefCell<Option<JourneysResult>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for JourneysResultObject {
        const NAME: &'static str = "DBJourneysResultObject";
        type Type = super::JourneysResultObject;
    }

    impl ObjectImpl for JourneysResultObject {}
}
