use gdk::subclass::prelude::ObjectSubclassIsExt;
use hafas_rest::{Hafas, Station, StationsQuery};
use rrw::{Error, StandardRestError};

gtk::glib::wrapper! {
    pub struct StationEntry(ObjectSubclass<imp::StationEntry>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget;
}

impl StationEntry {
    pub fn setup(&self, hafas: Hafas) {
        self.imp().setup(hafas, self);
    }

    pub fn set_input(&self, input: String) {
        self.imp().set_input(input);
    }
}

pub mod imp {
    use std::cell::{RefCell, Cell};
    use std::sync::{Arc, Mutex};
    use std::time::{Instant, Duration};

    use gdk::glib::{clone, ParamSpec, ParamSpecObject, ParamFlags, Value, ParamSpecBoolean, ParamSpecString};
    use gdk::glib::subclass::InitializingObject;
    use gdk::glib::MainContext;
    use gtk::{glib, ListStore};
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::CompositeTemplate;
    use hafas_rest::Hafas;
    use once_cell::sync::Lazy;

    use crate::gui::objects::StationObject;

    const REQUEST_DURATION: Duration = Duration::from_secs(1);

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/ui/station_entry.ui")]
    pub struct StationEntry {
        #[template_child]
        entry: TemplateChild<gtk::Entry>,

        last_request: Arc<Mutex<Option<Instant>>>,

        hafas: RefCell<Option<Hafas>>,

        station: RefCell<Option<StationObject>>,
        set: Cell<bool>,
        placeholder_text: RefCell<Option<String>>
    }

    impl StationEntry {
        pub(super) fn set_input(&self, input: String) {
            self.entry.set_text(&input);
        }

        pub(super) fn setup(&self, hafas: Hafas, obj: &super::StationEntry) {
            self.hafas.replace(Some(hafas));
            self.connect_changed(obj);
        }

        fn connect_changed(&self, obj: &super::StationEntry) {
            self.entry.connect_changed(
                clone!(@strong obj,
                       @strong self.hafas as hafas, 
                       @strong self.last_request as last_request => move |entry| {

                    let main_context = MainContext::default();
                    main_context.spawn_local(clone!(@strong obj,
                                                    @strong hafas, 
                                                    @strong last_request, 
                                                    @strong entry => async move {
                        let text = entry.text();

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

                        // Only request if text did not change since last time.
                        if text.len() >= entry.completion().expect("Completion to be set up").minimum_key_length().try_into().unwrap() && entry.text() == text {
                            log::trace!("Request is allowed with text {}.", text);
                            {
                                // Update last_request
                                let mut last_request = last_request.lock().expect("last_request to be lockable.");
                                *last_request = Some(Instant::now());
                            }

                            let hafas_borrow = hafas.borrow();
                            let hafas = hafas_borrow.as_ref().expect("Hafas has not yet been set up.");

                            let stations = super::get_stations_by_search(hafas, entry.text()).await;

                            if let Ok(stations) = stations {
                                let store = ListStore::new(&[String::static_type(), StationObject::static_type()]);

                                let mut exact_match: Option<StationObject> = None;
                                for station in stations {
                                    if station.name == text {
                                        exact_match = Some(StationObject::new(station.clone()));
                                    }

                                    let iter = store.append();
                                    store.set_value(&iter, 0, &station.name.to_value());
                                    store.set_value(&iter, 1, &StationObject::new(station).to_value());
                                }
                                obj.set_property("station", exact_match);

                                entry.completion().expect("Completion to be set up").set_model(Some(&store));
                            }
                        }
                    }));
                }),
            );
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
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            obj.connect_notify_local(Some("station"), clone!(@strong self.entry as entry => move |obj, _| {
                let option: Option<StationObject> = obj.property("station");
                if option.is_some() {
                    obj.set_property("set", true);
                    entry.add_css_class("success");
                    entry.remove_css_class("error");
                } else {
                    obj.set_property("set", false);
                    entry.add_css_class("error");
                    entry.remove_css_class("success");
                }
            }));
        }

        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![ParamSpecObject::new(
                    "station",
                    "station",
                    "station",
                    StationObject::static_type(),
                    ParamFlags::READWRITE,
                ), ParamSpecBoolean::new(
                    "set",
                    "set",
                    "set",
                    false,
                    ParamFlags::READWRITE,
                ), ParamSpecString::new(
                    "placeholder-text",
                    "placeholder-text",
                    "placeholder-text",
                    None,
                    ParamFlags::READWRITE,
                )]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "station" => {self.station.replace(value.get().expect("Property station of StationEntry must be StationObject"));},
                "set" => {self.set.replace(value.get().expect("Property set of StationEntry must be bool"));},
                "placeholder-text" => {self.placeholder_text.replace(value.get().expect("Property placeholder-text of StationEntry must be String"));},
                _ => unimplemented!()
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "station" => self
                    .station
                    .borrow()
                    .to_value(),
                "set" => self
                    .station
                    .borrow()
                    .is_some()
                    .to_value(),
                "placeholder-text" => self
                    .placeholder_text
                    .borrow()
                    .to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for StationEntry {}
    impl BoxImpl for StationEntry {}
}

async fn get_stations_by_search<S: AsRef<str>>(
    hafas: &Hafas,
    query: S,
) -> Result<Vec<Station>, Error<StandardRestError>> {
    let stations = hafas
        .stations(&StationsQuery {
            query: query.as_ref().to_string(),
            limit: Some(10),
            ..Default::default()
        })
        .await?;

    let mut stations_vec = stations.into_values().collect::<Vec<_>>();
    stations_vec.sort_by(|s1, s2| s1.weight.partial_cmp(&s2.weight).unwrap());

    Ok(stations_vec)
}
