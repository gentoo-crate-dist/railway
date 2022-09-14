use std::cell::RefCell;

use gdk::glib::Object;
use gdk::subclass::prelude::ObjectSubclassIsExt;

use super::Journey;

gtk::glib::wrapper! {
    pub struct JourneysResult(ObjectSubclass<imp::JourneysResult>);
}

impl JourneysResult {
    pub fn new(journeys_response: hafas_rs::api::journeys::JourneysResponse) -> Self {
        let s: Self = Object::new(&[]).expect("Failed to create `JourneysResult`.");
        s.imp()
            .journeys_response
            .swap(&RefCell::new(Some(journeys_response)));
        s
    }

    pub fn journeys(&self) -> Vec<Journey> {
        self.imp()
            .journeys_response
            .borrow()
            .as_ref()
            .map(|j| j.journeys.clone())
            .unwrap_or_default()
            .into_iter()
            .map(Journey::new)
            .collect()
    }
    pub fn journeys_response(&self) -> hafas_rs::api::journeys::JourneysResponse {
        self.imp()
            .journeys_response
            .borrow()
            .as_ref()
            .expect("JourneysResult has not yet been set up")
            .clone()
    }

    pub fn merge_prepend(&self, other: &Self) {
        let mut journeys_response = self.journeys_response();
        let other_journeys_response = other.journeys_response();
        journeys_response
            .journeys
            .splice(0..0, other_journeys_response.journeys);
        journeys_response.earlier_ref = other_journeys_response.earlier_ref;
        self.imp()
            .journeys_response
            .replace(Some(journeys_response));
    }

    pub fn merge_append(&self, other: &Self) {
        let mut journeys_response = self.journeys_response();
        let mut other_journeys_response = other.journeys_response();
        journeys_response
            .journeys
            .append(&mut other_journeys_response.journeys);
        journeys_response.later_ref = other_journeys_response.later_ref;
        self.imp()
            .journeys_response
            .replace(Some(journeys_response));
    }
}

mod imp {
    use gtk::glib;
    use std::cell::RefCell;

    use gdk::subclass::prelude::{ObjectImpl, ObjectSubclass};

    #[derive(Default, Clone)]
    pub struct JourneysResult {
        pub(super) journeys_response: RefCell<Option<hafas_rs::api::journeys::JourneysResponse>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for JourneysResult {
        const NAME: &'static str = "DBJourneysResult";
        type Type = super::JourneysResult;
    }

    impl ObjectImpl for JourneysResult {}
}
