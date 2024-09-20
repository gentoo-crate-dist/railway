use gdk::prelude::ObjectExt;
use gdk::subclass::prelude::ObjectSubclassIsExt;
use serde::{Deserialize, Serialize};

use crate::backend::Journey;
use crate::gui::window::Window;

#[derive(PartialEq)]
enum StoreMode {
    Toggle,
    Add,
    Remove,
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct StorageData {
    journeys: Vec<rcore::Journey>,
    watched: Vec<String>,
}

gtk::glib::wrapper! {
    pub struct JourneysStore(ObjectSubclass<imp::JourneysStore>);
}

impl JourneysStore {
    pub fn store(&self, journey: Journey) {
        self.imp().store(journey, StoreMode::Toggle);
    }

    pub fn contains(&self, journey: &Journey) -> bool {
        self.imp().contains(journey)
    }

    pub fn toggle_watch(&self, journey_id: String) {
        self.imp().toggle_watch(journey_id);
    }

    pub fn is_watched<S: AsRef<str>>(&self, journey_id: S) -> bool {
        self.imp().is_watched(journey_id)
    }

    pub fn flush(&self) {
        self.imp().flush();
    }

    pub fn setup(&self, window: Window) {
        self.imp().client.replace(Some(window.property("client")));
        self.imp().window.replace(Some(window));
        self.imp().load();
    }

    pub fn reload(&self) {
        self.imp().load();
    }

    fn on_minutely(&self) {
        let watched = self.imp().watched.borrow();
        for journey in &*self.imp().stored.borrow() {
            if watched.contains(&journey.id()) {
                // TODO: Store notification status for journey?
                journey.background_tasks();
            }
        }
    }
}

pub mod imp {
    use std::{
        cell::RefCell,
        collections::HashSet,
        fs::OpenOptions,
        io::{Seek, SeekFrom},
        path::PathBuf,
    };

    use chrono::{Duration, Local};
    use gtk::glib;
    use gtk::glib::clone;

    use gtk::pango;

    use gdk::{
        gio::Settings,
        glib::subclass::Signal,
        prelude::{ObjectExt, SettingsExt, StaticType},
        subclass::prelude::{ObjectImpl, ObjectImplExt, ObjectSubclass, ObjectSubclassExt},
    };
    use once_cell::sync::Lazy;

    use crate::gui::window::Window;
    use crate::{
        backend::{Client, Journey, Timer},
        config,
        gui::stores::{journey_store::StoreMode, migrate_journey_store::import_old_store},
    };

    use super::StorageData;

    pub struct JourneysStore {
        path: PathBuf,
        pub(super) watched: RefCell<HashSet<String>>,
        pub(super) stored: RefCell<Vec<Journey>>,
        pub(super) window: RefCell<Option<Window>>,
        pub(super) client: RefCell<Option<Client>>,

        pub(super) timer: RefCell<Timer>,
        settings: Settings,
    }

    impl JourneysStore {
        pub(super) fn load(&self) {
            log::debug!("Loading JourneyStore");
            let binding = self.client.borrow();
            let client = binding.as_ref().expect("Client to be set up");

            let mut file = OpenOptions::new()
                .write(true)
                .read(true)
                .create(true)
                .truncate(false)
                .open(&self.path)
                .expect("Failed to open journey_store.json file");

            let mut some_deletable = false;

            let data: StorageData = serde_json::from_reader(&file)
                .or_else(|_| {
                    let _ = file.seek(SeekFrom::Start(0));
                    let journeys: Vec<rcore::Journey> = serde_json::from_reader(&file)?;
                    Ok::<_, serde_json::Error>(StorageData {
                        journeys,
                        watched: vec![],
                    })
                })
                // Note: The migration will be removed once it is decided it will not be needed anymore.
                .or_else(|_| {
                    // Seek back file such that the same will be read again.
                    let _ = file.seek(SeekFrom::Start(0));
                    Ok::<_, serde_json::Error>(StorageData {
                        journeys: import_old_store(file)?,
                        watched: vec![],
                    })
                })
                .unwrap_or_default();
            for journey in data.journeys.into_iter().rev() {
                // Note: We limit the deletion time in the settings; the conversion to Duration should never fail.
                let deletion_time = Duration::try_hours(self.settings.int("deletion-time").into())
                    .unwrap_or_default();
                let could_be_deleted = journey
                    .legs
                    .last()
                    .and_then(|l| l.arrival.or(l.planned_arrival))
                    .map(|arrival| arrival + deletion_time < Local::now())
                    .unwrap_or(true);

                if could_be_deleted {
                    if self.settings.boolean("delete-old") {
                        self.settings
                            .set_boolean("bookmark-purge-suggested", true)
                            .expect("setting bookmark-purge-suggested must exist as a boolean");
                        self.store(client.get_journey(journey), StoreMode::Remove);
                        continue;
                    } else if !self.settings.boolean("bookmark-purge-suggested") {
                        some_deletable = true;
                    }
                }

                self.store(client.get_journey(journey), StoreMode::Add);
            }

            for journey in data.watched.into_iter() {
                self.toggle_watch(journey);
            }

            if some_deletable {
                let toast = libadwaita::Toast::builder()
                    .custom_title(
                        &gtk::Label::builder()
                            .label(gettextrs::gettext("A bookmarked trip ended recently."))
                            .wrap(true)
                            .lines(2)
                            /*
                             * follows libadwaita's default title widget:
                             * <https://gitlab.gnome.org/GNOME/libadwaita/-/blob/main/src/adw-toast-widget.c#L129-132>
                             */
                            .ellipsize(pango::EllipsizeMode::End)
                            .xalign(0.0)
                            .css_classes(vec!["heading"])
                            .build(),
                    )
                    .button_label(gettextrs::gettext("Always _Delete"))
                    .timeout(0)
                    .build();

                toast.connect_button_clicked(clone!(
                    #[strong(rename_to = settings)]
                    self.settings,
                    move |_| {
                        settings
                            .set_boolean("delete-old", true)
                            .expect("setting delete-old must exist as a boolean");
                    }
                ));

                toast.connect_dismissed(clone!(
                    #[strong(rename_to = settings)]
                    self.settings,
                    move |_| {
                        settings
                            .set_boolean("bookmark-purge-suggested", true)
                            .expect("setting bookmark-purge-suggested must exist as a boolean");
                    }
                ));

                self.window
                    .borrow()
                    .as_ref()
                    .expect("JourneyStore needs an associated window")
                    .display_custom_toast(toast);
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
                watched: Default::default(),
                window: RefCell::new(None),
                client: RefCell::new(None),
                timer: Default::default(),
                settings: Settings::new(config::BASE_ID),
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
            let journeys: Vec<rcore::Journey> =
                self.stored.borrow().iter().map(|j| j.journey()).collect();

            let watched = self.watched.borrow().clone();

            let data = StorageData {
                journeys,
                watched: watched.into_iter().collect(),
            };

            let file = OpenOptions::new()
                .write(true)
                .read(false)
                .truncate(true)
                .create(true)
                .append(false)
                .open(&self.path)
                .expect("Failed to open journey_store.json file");

            serde_json::to_writer(file, &data).expect("Failed to write to file");
        }

        pub(super) fn store(&self, journey: Journey, store_mode: StoreMode) {
            let mut stored = self.stored.borrow_mut();
            if let Some(idx) = stored.iter().position(|j| j.id() == journey.id()) {
                if store_mode != StoreMode::Add {
                    log::trace!("Removing Journey {:?}", journey.journey());
                    let s = stored.remove(idx);
                    self.obj().emit_by_name::<()>("remove", &[&s]);

                    let journey_id = s.id();
                    if self.is_watched(&journey_id) {
                        self.toggle_watch(journey_id);
                    }
                }
            } else if store_mode != StoreMode::Remove {
                log::trace!("Storing Journey {:?}", journey.journey());
                self.obj().emit_by_name::<()>("add", &[&journey]);
                stored.insert(0, journey);
            }
        }

        pub(super) fn toggle_watch(&self, journey_id: String) {
            let mut watched = self.watched.borrow_mut();
            if watched.contains(&journey_id) {
                log::trace!("Removing Watch {:?}", journey_id);
                watched.remove(&journey_id);
            } else {
                log::trace!("Adding Watch {:?}", journey_id);
                watched.insert(journey_id);
                drop(watched);
                self.obj().on_minutely();
            }
        }

        pub(super) fn contains(&self, journey: &Journey) -> bool {
            let stored = self.stored.borrow();
            stored.iter().any(|j| j.id() == journey.id())
        }

        pub(super) fn is_watched<S: AsRef<str>>(&self, journey_id: S) -> bool {
            let stored = self.watched.borrow();
            stored.iter().any(|j| j == journey_id.as_ref())
        }
    }

    impl ObjectImpl for JourneysStore {
        fn constructed(&self) {
            self.parent_constructed();

            let store = self.obj();
            self.settings.connect_changed(
                Some("delete-old"),
                clone!(
                    #[weak]
                    store,
                    move |_, _| {
                        store.reload();
                    }
                ),
            );

            let obj = self.obj();
            self.timer.borrow().connect_local(
                "minutely",
                false,
                clone!(
                    #[weak]
                    obj,
                    #[upgrade_or_default]
                    move |_| {
                        obj.on_minutely();
                        None
                    }
                ),
            );
        }

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
