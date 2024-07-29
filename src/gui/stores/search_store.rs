use gdk::subclass::prelude::ObjectSubclassIsExt;

gtk::glib::wrapper! {
    pub struct SearchesStore(ObjectSubclass<imp::SearchesStore>);
}

impl SearchesStore {
    // XXX: Store Place instead of String.
    pub fn store(&self, origin: String, destination: String) {
        self.imp().store(origin, destination);
    }

    pub fn contains<S: AsRef<str>>(&self, origin: S, destination: S) -> bool {
        self.imp().contains(origin, destination)
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

    use gtk::glib;

    use gdk::{
        glib::subclass::Signal,
        prelude::{ObjectExt, StaticType},
        subclass::prelude::{ObjectImpl, ObjectSubclass, ObjectSubclassExt},
    };
    use once_cell::sync::Lazy;
    use serde::{Deserialize, Serialize};

    pub struct SearchesStore {
        path: PathBuf,
        stored: RefCell<Vec<Search>>,
    }

    impl SearchesStore {
        pub(super) fn load(&self) {
            log::debug!("Loading SearchesStore");
            let file = OpenOptions::new()
                .write(true)
                .read(true)
                .create(true)
                .truncate(false)
                .open(&self.path)
                .expect("Failed to open searches_store.json file");

            let searches: Vec<Search> = serde_json::from_reader(file).unwrap_or_default();
            for search in searches.into_iter().rev() {
                self.store(search.origin, search.destination);
            }
        }
    }

    impl Default for SearchesStore {
        fn default() -> Self {
            let mut path = gtk::glib::user_data_dir();
            path.push("diebahn");

            if !path.exists() {
                std::fs::create_dir_all(&path).expect("could not create user data dir");
            }
            path.push("searches_store.json");

            Self {
                path,
                stored: RefCell::new(vec![]),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SearchesStore {
        const NAME: &'static str = "DBSearchesStore";
        type Type = super::SearchesStore;
    }

    impl SearchesStore {
        pub(super) fn flush(&self) {
            log::debug!("Flushing SearchesStore");
            let searches = self.stored.borrow();

            let file = OpenOptions::new()
                .write(true)
                .read(false)
                .truncate(true)
                .create(true)
                .append(false)
                .open(&self.path)
                .expect("Failed to open searches_store.json file");

            serde_json::to_writer(file, &*searches).expect("Failed to write to file");
        }

        pub(super) fn contains<S: AsRef<str>>(&self, origin: S, destination: S) -> bool {
            let search = Search {
                origin: origin.as_ref().to_owned(),
                destination: destination.as_ref().to_owned(),
            };

            let stored = self.stored.borrow();
            stored.iter().any(|s| s == &search)
        }

        pub(super) fn store(&self, origin: String, destination: String) {
            let search = Search {
                origin,
                destination,
            };

            let mut stored = self.stored.borrow_mut();
            if let Some(idx) = stored.iter().position(|j| j == &search) {
                log::trace!("Removing Search {:?}", search);
                let s = stored.remove(idx);
                self.obj()
                    .emit_by_name::<()>("remove", &[&s.origin, &s.destination]);
            } else {
                log::trace!("Storing Journey {:?}", search);
                self.obj()
                    .emit_by_name::<()>("add", &[&search.origin, &search.destination]);
                stored.insert(0, search);
            }
        }
    }

    impl ObjectImpl for SearchesStore {
        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| -> Vec<Signal> {
                vec![
                    Signal::builder("add")
                        .param_types([String::static_type(), String::static_type()])
                        .build(),
                    Signal::builder("remove")
                        .param_types([String::static_type(), String::static_type()])
                        .build(),
                ]
            });
            SIGNALS.as_ref()
        }
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
    struct Search {
        origin: String,
        destination: String,
    }
}
