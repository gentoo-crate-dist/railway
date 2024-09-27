use std::time::Duration;

use gdk::{
    glib::{self, clone, Object},
    subclass::prelude::ObjectSubclassIsExt,
};

use super::Journey;

gtk::glib::wrapper! {
    pub struct Timer(ObjectSubclass<imp::Timer>);
}

impl Default for Timer {
    fn default() -> Self {
        Object::builder().build()
    }
}

impl Timer {
    pub fn register_background(&self, journey: Journey) {
        let journey_id = journey.id();
        let handle = gspawn!(clone!(
            #[strong(rename_to = s)]
            self,
            async move {
                loop {
                    glib::timeout_future(
                        journey
                            .next_background_tasks_in()
                            .to_std()
                            .unwrap_or_default(),
                    )
                    .await;
                    // Don't do background tasks if already in minutely batch.
                    if !s.has_in_minutely(journey.id()) {
                        journey.background_tasks();
                    }
                }
            }
        ));
        self.imp()
            .background_handles
            .borrow_mut()
            .insert(journey_id, handle);
    }

    pub fn unregister_background(&self, journey: Journey) {
        if let Some(handle) = self
            .imp()
            .background_handles
            .borrow_mut()
            .remove(&journey.id())
        {
            handle.abort();
        }
    }

    pub fn has_in_minutely(&self, journey_id: String) -> bool {
        self.imp()
            .minutely_handles
            .borrow()
            .get(&journey_id)
            .is_some()
    }

    pub fn register_minutely(&self, journey: Journey) {
        let journey_id = journey.id();
        let handle = gspawn!(async move {
            loop {
                glib::timeout_future(Duration::from_secs(60)).await;
                journey.background_tasks();
            }
        });
        self.imp()
            .minutely_handles
            .borrow_mut()
            .insert(journey_id, handle);
    }

    pub fn unregister_minutely(&self, journey: Journey) {
        if let Some(handle) = self
            .imp()
            .minutely_handles
            .borrow_mut()
            .remove(&journey.id())
        {
            handle.abort();
        }
    }
}

mod imp {
    use std::cell::RefCell;
    use std::collections::HashMap;

    use gdk::glib::subclass::types::ObjectSubclass;
    use gdk::glib::{self, JoinHandle};
    use gdk::subclass::prelude::ObjectImpl;

    #[derive(Default)]
    pub struct Timer {
        pub(super) background_handles: RefCell<HashMap<String, JoinHandle<()>>>,
        pub(super) minutely_handles: RefCell<HashMap<String, JoinHandle<()>>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Timer {
        const NAME: &'static str = "DBTimer";
        type Type = super::Timer;
    }

    impl ObjectImpl for Timer {}
}
