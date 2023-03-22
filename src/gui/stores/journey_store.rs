use gdk::subclass::prelude::ObjectSubclassIsExt;

use crate::backend::Journey;

gtk::glib::wrapper! {
    pub struct JourneysStore(ObjectSubclass<imp::JourneysStore>);
}

impl JourneysStore {
    pub fn store(&self, journey: Journey) {
        self.imp().store(journey);
    }

    pub fn flush(&self) {
        self.imp().flush();
    }

    pub fn setup(&self) {
        self.imp().load();
    }
}

pub mod imp {
    use std::{cell::RefCell, fs::OpenOptions, path::PathBuf};

    use chrono::{Duration, Local};
    use gtk::glib;

    use gdk::{
        gio::Settings,
        glib::subclass::Signal,
        prelude::{ObjectExt, SettingsExt, StaticType},
        subclass::prelude::{ObjectImpl, ObjectSubclass, ObjectSubclassExt},
    };
    use once_cell::sync::Lazy;

    use crate::backend::Journey;

    pub struct JourneysStore {
        path: PathBuf,
        stored: RefCell<Vec<Journey>>,

        settings: Settings,
    }

    impl JourneysStore {
        pub(super) fn load(&self) {
            log::debug!("Loading JourneyStore");
            let file = OpenOptions::new()
                .write(true)
                .read(true)
                .create(true)
                .open(&self.path)
                .expect("Failed to open journey_store.json file");

            let journeys: Vec<hafas_rs::Journey> =
                serde_json::from_reader(file).unwrap_or_default();
            for journey in journeys.into_iter().rev() {
                if self.settings.boolean("delete-old") {
                    if let Some(arrival) = journey.legs.last().and_then(|l| l.planned_arrival) {
                        let deletion_time = self.settings.int("deletion-time");
                        let deletion = arrival + Duration::hours(deletion_time.into());
                        if deletion < Local::now() {
                            continue;
                        }
                    } else {
                        continue;
                    }
                }
                self.store(Journey::new(journey));
            }
        }
    }

    impl Default for JourneysStore {
        fn default() -> Self {
            let mut path = gtk::glib::user_data_dir();
            path.push("diebahn");

            if !path.exists() {
                std::fs::create_dir_all(&path).expect("could not create user data dir");
            }
            path.push("journeys_store.json");

            Self {
                path,
                stored: RefCell::new(vec![]),
                settings: Settings::new("de.schmidhuberj.DieBahn"),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for JourneysStore {
        const NAME: &'static str = "DBJourneysStore";
        type Type = super::JourneysStore;
    }

    impl JourneysStore {
        pub(super) fn flush(&self) {
            log::debug!("Flushing JourneyStore");
            let journeys: Vec<hafas_rs::Journey> =
                self.stored.borrow().iter().map(|j| j.journey()).collect();

            let file = OpenOptions::new()
                .write(true)
                .read(false)
                .truncate(true)
                .create(true)
                .append(false)
                .open(&self.path)
                .expect("Failed to open journey_store.json file");

            serde_json::to_writer(file, &journeys).expect("Failed to write to file");
        }

        pub(super) fn store(&self, journey: Journey) {
            let mut stored = self.stored.borrow_mut();
            if let Some(idx) = stored
                .iter()
                .position(|j| j.journey().refresh_token == journey.journey().refresh_token)
            {
                log::trace!("Removing Journey {:?}", journey.journey());
                let s = stored.remove(idx);
                self.obj().emit_by_name::<()>("remove", &[&s]);
            } else {
                log::trace!("Storing Journey {:?}", journey.journey());
                self.obj().emit_by_name::<()>("add", &[&journey]);
                stored.insert(0, journey);
            }
        }
    }

    impl ObjectImpl for JourneysStore {
        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| -> Vec<Signal> {
                vec![
                    Signal::builder("add")
                        .param_types([Journey::static_type()])
                        .build(),
                    Signal::builder("remove")
                        .param_types([Journey::static_type()])
                        .build(),
                ]
            });
            SIGNALS.as_ref()
        }
    }
}
