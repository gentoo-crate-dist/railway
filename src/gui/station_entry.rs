use gdk::{prelude::ObjectExt, subclass::prelude::ObjectSubclassIsExt};
use libadwaita::prelude::EditableExt;

use crate::backend::Place;

gtk::glib::wrapper! {
    pub struct StationEntry(ObjectSubclass<imp::StationEntry>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget, gtk::Editable;
}

impl StationEntry {
    pub fn set_input(&self, input: String) {
        self.imp().text.set_text(&input);
    }

    pub fn input(&self) -> String {
        self.imp().text.text().to_string()
    }

    pub fn set_place(&self, place: Option<&Place>) {
        self.set_property("place", place);
        if let Some(place) = place {
            // When something is selected, set the text of the input and clear all completion suggestions.
            self.imp().text.set_text(&place.name().unwrap_or_default());
            self.imp().completions.borrow().remove_all();
        }
    }
}

pub mod imp {
    use std::borrow::Borrow;
    use std::cell::RefCell;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, Mutex};
    use std::time::{Duration, Instant};

    use gdk::gio;
    use gdk::glib::subclass::InitializingObject;
    use gdk::glib::MainContext;
    use gdk::glib::{clone, ParamSpec, ParamSpecBoolean, ParamSpecObject, ParamSpecString, Value};
    use gtk::subclass::prelude::*;
    use gtk::{gio::ListStore, glib};
    use gtk::{prelude::*, ListItem, SignalListItemFactory, Text, Widget};
    use gtk::{CompositeTemplate, Popover};
    use hafas_rs::api::locations::LocationsOptions;
    use once_cell::sync::Lazy;

    use crate::backend::{HafasClient, Place};
    use crate::gui::place_list_item::PlaceListItem;

    const REQUEST_DURATION: Duration = Duration::from_secs(1);
    const MIN_REQUEST_LEN: usize = 3;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/ui/station_entry.ui")]
    pub struct StationEntry {
        #[template_child]
        pub(super) text: TemplateChild<Text>,
        #[template_child]
        popover: TemplateChild<Popover>,
        #[template_child]
        list_completions: TemplateChild<gtk::ListView>,

        pub(super) completions: RefCell<ListStore>,

        last_request: Arc<Mutex<Option<Instant>>>,

        client: RefCell<Option<HafasClient>>,

        place: RefCell<Option<Place>>,

        request_pending: Arc<AtomicBool>,

        title: RefCell<String>,
    }

    impl StationEntry {
        fn on_changed(&self) {
            let request_pending = &self.request_pending;
            let last_request = &self.last_request;
            let completions = &self.completions;
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
                                            @strong completions,
                                            => async move {
                let entry = &obj.imp().text;
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
                if text.len() >= MIN_REQUEST_LEN {
                    log::trace!("Request is allowed with text {}.", text);
                    {
                        // Update last_request
                        let mut last_request = last_request.lock().expect("last_request to be lockable.");
                        *last_request = Some(Instant::now());
                    }

                    let text = entry.text().to_string();
                    let places = obj.property::<HafasClient>("client").locations(LocationsOptions {query: text.clone(), ..Default::default()}).await;

                    if let Ok(places) = places {
                        let mut exact_match: Option<Place> = None;
                        let completions = completions.borrow();
                        completions.remove_all();
                        for place in places {
                            if place.id().is_none() {
                                // Skip things without ID
                                continue;
                            }
                            if place.name().as_ref() == Some(&text) {
                                exact_match = Some(place.clone());
                            }

                            completions.append(&place);
                        }
                        drop(completions);
                        obj.set_place(exact_match.as_ref());
                    }
                }
                log::trace!("Unlock pending request.");
                request_pending.store(false, Ordering::SeqCst);
            }));
        }

        fn connect_changed(&self, obj: &super::StationEntry) {
            self.text
                .connect_changed(clone!(@strong obj => move |_entry| {
                    obj.imp().on_changed();
                }));
            self.on_changed();
        }

        fn update_popover_visible(&self) {
            self.popover.set_visible(
                self.completions.borrow().n_items() != 0
                    && self.text.has_focus()
                    && self.place.borrow().is_none(),
            );
        }

        fn setup_model(&self, obj: &super::StationEntry) {
            let model = &self.completions.borrow();
            let model: &gio::ListModel = model
                .dynamic_cast_ref()
                .expect("ListStore to be a ListModel");
            let selection_model = gtk::NoSelection::new(Some(model.clone()));
            self.list_completions
                .get()
                .set_model(Some(&selection_model));

            let factory = SignalListItemFactory::new();
            factory.connect_setup(move |_, list_item| {
                let place_item = PlaceListItem::new();
                let list_item = list_item
                    .downcast_ref::<ListItem>()
                    .expect("The factory item to be a `ListItem`");

                list_item.set_child(Some(&place_item));
                list_item
                    .property_expression("item")
                    .bind(&place_item, "place", Widget::NONE);
            });
            self.list_completions.set_factory(Some(&factory));
            self.list_completions.set_single_click_activate(true);

            self.list_completions.connect_activate(
                clone!(@strong obj => move |list_view, position| {
                    let model = list_view.model().expect("The model has to exist.");
                    let place_model = model
                        .item(position)
                        .expect("The item has to exist.")
                        .downcast::<Place>()
                        .expect("The item has to be an `Place`.");

                    obj.set_place(Some(&place_model));
                    obj.imp().popover.popdown();
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
            Self::bind_template_callbacks(klass);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[gtk::template_callbacks]
    impl StationEntry {
        #[template_callback]
        fn handle_focus(&self) {
            self.text.grab_focus();
        }
    }

    impl ObjectImpl for StationEntry {
        fn constructed(&self) {
            let obj = self.obj();
            self.parent_constructed();
            self.connect_changed(&obj);
            self.setup_model(&obj);

            self.completions.borrow().connect_notify_local(
                Some("n-items"),
                clone!(@strong obj => move |_, _| {
                    obj.imp().update_popover_visible();
                }),
            );

            self.text.borrow().connect_notify_local(
                Some("has-focus"),
                clone!(@strong obj => move |_, _| {
                    obj.imp().update_popover_visible();
                }),
            );

            obj.connect_notify_local(
                Some("place"),
                clone!(@strong obj as entry => move |obj, _| {
                    let option: Option<Place> = obj.property("place");
                    if option.is_some() {
                        entry.remove_css_class("error");
                    } else {
                        entry.add_css_class("error");
                    }
                    obj.notify("set");
                }),
            );
        }

        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecObject::builder::<Place>("place").build(),
                    ParamSpecBoolean::builder("set").read_only().build(),
                    ParamSpecObject::builder::<HafasClient>("client").build(),
                    ParamSpecObject::builder::<ListStore>("completions")
                        .read_only()
                        .build(),
                    ParamSpecString::builder("title").build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "place" => {
                    self.place.replace(
                        value
                            .get()
                            .expect("Property place of StationEntry must be Place"),
                    );
                }
                "client" => {
                    let obj = value.get::<Option<HafasClient>>().expect(
                        "Property `client` of `StationEntry` has to be of type `HafasClient`",
                    );

                    if let Some(obj) = &obj {
                        let s = self.obj();
                        obj.connect_local("provider-changed", true, clone!(@weak s => @default-return None, move |_| {
                            log::trace!("Station-entry got provider change from hafas_client. Restting");
                            s.set_place(None);
                            s.imp().on_changed();
                            None
                        }));
                    }

                    self.client.replace(obj);
                }
                "title" => {
                    self.title.replace(
                        value
                            .get()
                            .expect("Property title of StationEntry must be String"),
                    );
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "place" => self.place.borrow().to_value(),
                "set" => self.place.borrow().is_some().to_value(),
                "client" => self.client.borrow().to_value(),
                "completions" => self.completions.borrow().to_value(),
                "title" => self.title.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for StationEntry {}
    impl BoxImpl for StationEntry {}
}
