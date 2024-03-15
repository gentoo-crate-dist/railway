use gdk::subclass::prelude::ObjectSubclassIsExt;

use crate::gui::window::Window;
use crate::backend::Journey;

#[derive(PartialEq)]
enum StoreMode {
    Toggle,
    Add,
    Remove,
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

    pub fn flush(&self) {
        self.imp().flush();
    }

    pub fn setup(&self, window: Window) {
        self.imp().window.replace(Some(window));
        self.imp().load();
    }

    pub fn reload(&self) {
        self.imp().load();
    }
}

pub mod imp {
    use std::{
        cell::RefCell,
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

    use crate::config;
    use crate::{backend::Journey, gui::stores::migrate_journey_store::import_old_store};
    use crate::gui::window::Window;
    use crate::gui::stores::journey_store::StoreMode;

    pub struct JourneysStore {
        path: PathBuf,
        stored: RefCell<Vec<Journey>>,
        pub(super) window: RefCell<Option<Window>>,

        settings: Settings,
    }

    impl JourneysStore {
        pub(super) fn load(&self) {
            log::debug!("Loading JourneyStore");
            let mut file = OpenOptions::new()
                .write(true)
                .read(true)
                .create(true)
                .open(&self.path)
                .expect("Failed to open journey_store.json file");

            let mut some_deletable = false;

            let journeys: Vec<rcore::Journey> = serde_json::from_reader(&file)
                // Note: The migration will be removed once it is decided it will not be needed anymore.
                .or_else(|_| {
                    // Seek back file such that the same will be read again.
                    let _ = file.seek(SeekFrom::Start(0));
                    import_old_store(file)
                })
                .unwrap_or_default();
            for journey in journeys.into_iter().rev() {
                // Note: We limit the deletion time in the settings; the conversion to Duration should never fail.
                let deletion_time = Duration::try_hours(self.settings.int("deletion-time").into()).unwrap_or_default();
                let could_be_deleted = journey.legs.last().and_then(|l| {
                    l.arrival.or(l.planned_arrival)
                }).map(|arrival| {
                     arrival + deletion_time < Local::now()
                }).unwrap_or(true);

                if could_be_deleted {
                    if self.settings.boolean("delete-old") {
                        self.settings.set_boolean("bookmark-purge-suggested", true)
                            .expect("setting bookmark-purge-suggested must exist as a boolean");
                        self.store(Journey::new(journey), StoreMode::Remove);
                        continue;
                    } else if !self.settings.boolean("bookmark-purge-suggested") {
                        some_deletable = true;
                    }
                }

                self.store(Journey::new(journey), StoreMode::Add);
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
                            .build()
                    )
                    .button_label(&gettextrs::gettext("Always _Delete"))
                    .timeout(0)
                    .build();

                let store = self.obj();
                toast.connect_button_clicked(clone!(@strong self.settings as settings, @strong store => move |_| {
                    settings.set_boolean("delete-old", true)
                            .expect("setting delete-old must exist as a boolean");
                }));

                toast.connect_dismissed(clone!(@strong self.settings as settings => move |_| {
                    settings.set_boolean("bookmark-purge-suggested", true)
                            .expect("setting bookmark-purge-suggested must exist as a boolean");
                }));

                self.window.borrow().as_ref()
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
                window: RefCell::new(None),
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

        pub(super) fn store(&self, journey: Journey, store_mode: StoreMode) {
            let mut stored = self.stored.borrow_mut();
            if let Some(idx) = stored.iter().position(|j| j.id() == journey.id()) {
                if store_mode != StoreMode::Add {
                    log::trace!("Removing Journey {:?}", journey.journey());
                    let s = stored.remove(idx);
                    self.obj().emit_by_name::<()>("remove", &[&s]);
                }
            } else {
                if store_mode != StoreMode::Remove {
                    log::trace!("Storing Journey {:?}", journey.journey());
                    self.obj().emit_by_name::<()>("add", &[&journey]);
                    stored.insert(0, journey);
                }
            }
        }

        pub(super) fn contains(&self, journey: &Journey) -> bool {
            let stored = self.stored.borrow();
            stored.iter().any(|j| j.id() == journey.id())
        }
    }

    impl ObjectImpl for JourneysStore {
        fn constructed(&self) {
            self.parent_constructed();

            let store = self.obj();
            self.settings.connect_changed(Some("delete-old"), clone!(@weak store => move |_, _| {
                store.reload();
            }));
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
