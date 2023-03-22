use gdk::subclass::prelude::ObjectSubclassIsExt;

gtk::glib::wrapper! {
    pub struct StationEntry(ObjectSubclass<imp::StationEntry>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget;
}

impl StationEntry {
    pub fn set_input(&self, input: String) {
        self.imp().set_input(input);
    }

    pub fn input(&self) -> String {
        self.imp().input()
    }
}

pub mod imp {
    use std::cell::{RefCell, Cell};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, Mutex};
    use std::time::{Instant, Duration};

    use gdk::glib::{clone, ParamSpec, ParamSpecObject, Value, ParamSpecBoolean, ParamSpecString};
    use gdk::glib::subclass::InitializingObject;
    use gdk::glib::MainContext;
    use gtk::{glib, ListStore};
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::CompositeTemplate;
    use hafas_rs::api::locations::LocationsOptions;
    use once_cell::sync::Lazy;

    use crate::backend::{HafasClient, Place};

    const REQUEST_DURATION: Duration = Duration::from_secs(1);

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/ui/station_entry.ui")]
    pub struct StationEntry {
        #[template_child]
        entry: TemplateChild<gtk::Entry>,

        last_request: Arc<Mutex<Option<Instant>>>,

        client: RefCell<Option<HafasClient>>,

        place: RefCell<Option<Place>>,
        set: Cell<bool>,
        placeholder_text: RefCell<Option<String>>,
        request_pending: Arc<AtomicBool>,
    }

    impl StationEntry {
        pub(super) fn set_input(&self, input: String) {
            self.entry.set_text(&input);
        }

        pub(super) fn input(&self) -> String {
            self.entry.text().to_string()
        }

        fn on_changed(&self) {
            let request_pending = &self.request_pending;
            let last_request = &self.last_request;
            let obj = self.obj();

            if request_pending.load(Ordering::SeqCst) {
                log::trace!("Station changed, but there is already a request pending");
                return;
            } else {
                log::trace!("Station changed. Block other requests");
                request_pending.store(true, Ordering::SeqCst);
            }

            let main_context = MainContext::default();
            main_context.spawn_local(clone!(@strong obj,
                                            @strong last_request, 
                                            @strong request_pending,
                                            => async move {
                let entry = &obj.imp().entry;
                let to_wait = {
                    let mut last_request = last_request.lock().expect("last_request to be lockable.");
                    if last_request.is_none() {
                        *last_request = Some(Instant::now() - REQUEST_DURATION);
                    }
                    let duration_from_last_request = Instant::now() - last_request.expect("last_request to be set");
                    if duration_from_last_request >= REQUEST_DURATION {
                        Duration::ZERO
                    } else {
                        REQUEST_DURATION - duration_from_last_request
                    }
                };

                log::trace!("Station entry changed. Wait {:?} for next request.", to_wait);

                tokio::time::sleep(to_wait).await;

                let text = entry.text();

                // Only request if text did not change since last time.
                if text.len() >= entry.completion().expect("Completion to be set up").minimum_key_length().try_into().unwrap() {
                    log::trace!("Request is allowed with text {}.", text);
                    {
                        // Update last_request
                        let mut last_request = last_request.lock().expect("last_request to be lockable.");
                        *last_request = Some(Instant::now());
                    }

                    let text = entry.text().to_string();
                    let places = obj.property::<HafasClient>("client").locations(LocationsOptions {query: text.clone(), ..Default::default()}).await;

                    if let Ok(places) = places {
                        let store = ListStore::new(&[String::static_type(), Place::static_type()]);

                        let mut exact_match: Option<Place> = None;
                        for place in places {
                            if place.id().is_none() {
                                // Skip things without ID
                                continue;
                            }
                            if place.name().as_ref() == Some(&text) {
                                exact_match = Some(place.clone());
                            }

                            let iter = store.append();
                            store.set_value(&iter, 0, &place.name().to_value());
                            store.set_value(&iter, 1, &place.to_value());
                        }
                        obj.set_property("place", exact_match);

                        entry.completion().expect("Completion to be set up").set_model(Some(&store));
                    }
                }
                log::trace!("Unlock pending request.");
                request_pending.store(false, Ordering::SeqCst);
            }));
        }

        fn connect_changed(&self, obj: &super::StationEntry) {
            self.entry.connect_changed(clone!(@strong obj => move |_entry| {
                    obj.imp().on_changed();
            }));
            self.on_changed();
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for StationEntry {
        const NAME: &'static str = "DBStationEntry";
        type Type = super::StationEntry;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for StationEntry {
        fn constructed(&self) {
            self.parent_constructed();
            self.connect_changed(&self.obj());

            self.obj().connect_notify_local(Some("place"), clone!(@strong self.entry as entry => move |obj, _| {
                let option: Option<Place> = obj.property("place");
                if option.is_some() {
                    obj.set_property("set", true);
                    // entry.add_css_class("success");
                    entry.remove_css_class("error");
                } else {
                    obj.set_property("set", false);
                    entry.add_css_class("error");
                    // entry.remove_css_class("success");
                }
            }));
        }

        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecObject::builder::<Place>("place").build(),
                    ParamSpecBoolean::builder("set").build(),
                    ParamSpecString::builder("placeholder-text").build(),
                    ParamSpecObject::builder::<HafasClient>("client").build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "place" => {self.place.replace(value.get().expect("Property place of StationEntry must be Place"));},
                "set" => {self.set.replace(value.get().expect("Property set of StationEntry must be bool"));},
                "placeholder-text" => {self.placeholder_text.replace(value.get().expect("Property placeholder-text of StationEntry must be String"));},
                "client" => {
                    let obj = value
                        .get::<Option<HafasClient>>()
                        .expect("Property `client` of `StationEntry` has to be of type `HafasClient`");

                    self.client.replace(obj);
                }
                _ => unimplemented!()
            }
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "place" => self
                    .place
                    .borrow()
                    .to_value(),
                "set" => self
                    .place
                    .borrow()
                    .is_some()
                    .to_value(),
                "placeholder-text" => self
                    .placeholder_text
                    .borrow()
                    .to_value(),
                "client" => self.client.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for StationEntry {}
    impl BoxImpl for StationEntry {}
}
