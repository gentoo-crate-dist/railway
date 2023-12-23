use gdk::{prelude::ObjectExt, subclass::prelude::ObjectSubclassIsExt};
use libadwaita::prelude::EditableExt;

use crate::backend::Place;

gtk::glib::wrapper! {
    pub struct StationEntry(ObjectSubclass<imp::StationEntry>)
        @extends libadwaita::EntryRow, libadwaita::PreferencesRow, gtk::ListBoxRow, gtk::Widget,
        @implements gtk::Accessible, gtk::Actionable, gtk::Buildable,
            gtk::ConstraintTarget, gtk::Editable;
}

impl StationEntry {
    pub fn set_input(&self, input: String) {
        self.set_text(&input);
        self.set_place(None);
    }

    pub fn input(&self) -> String {
        self.text().to_string()
    }

    pub fn set_place(&self, place: Option<&Place>) {
        self.set_property("place", place);
        if let Some(place) = place {
            // When something is selected, set the text of the input and clear all completion suggestions.
            let name = place.name().unwrap_or_default();
            if self.text() != name {
                self.set_text(&name);
            }
            self.imp().completions.borrow().remove_all();
        }
    }
}

pub mod imp {
    use std::cell::{Cell, RefCell};
    use std::time::Duration;

    use gdk::gio;
    use gdk::glib::subclass::{InitializingObject, Signal};
    use gdk::glib::{clone, ParamSpec, ParamSpecBoolean, ParamSpecObject, ParamSpecString, Value};
    use gdk::glib::{MainContext, Properties};
    use gdk::prelude::ObjectExt;
    use gtk::subclass::prelude::*;
    use gtk::{gio::ListStore, glib};
    use gtk::{prelude::*, ListItem, SignalListItemFactory, Widget};
    use gtk::{CompositeTemplate, Popover};
    use hafas_rs::api::locations::LocationsOptions;
    use libadwaita::subclass::prelude::{EntryRowImpl, PreferencesRowImpl};
    use once_cell::sync::Lazy;

    use crate::backend::{HafasClient, Place, RequestLimiter};
    use crate::gui::place_list_item::PlaceListItem;

    const REQUEST_DURATION: Duration = Duration::from_secs(1);
    const MIN_REQUEST_LEN: usize = 3;

    #[derive(CompositeTemplate, Properties)]
    #[template(resource = "/ui/station_entry.ui")]
    #[properties(wrapper_type=super::StationEntry)]
    pub struct StationEntry {
        #[template_child]
        popover: TemplateChild<Popover>,
        #[template_child]
        list_completions: TemplateChild<gtk::ListView>,

        pub(super) completions: RefCell<ListStore>,

        client: RefCell<Option<HafasClient>>,

        place: RefCell<Option<Place>>,

        title: RefCell<String>,

        request_limiter: RequestLimiter<String>,

        #[property(get, set)]
        show_swap_button: Cell<bool>,
    }

    impl Default for StationEntry {
        fn default() -> Self {
            StationEntry {
                popover: Default::default(),
                list_completions: Default::default(),
                completions: RefCell::new(ListStore::new::<Place>()),
                client: Default::default(),
                place: Default::default(),
                title: Default::default(),
                show_swap_button: Default::default(),
                request_limiter: RequestLimiter::new(REQUEST_DURATION),
            }
        }
    }

    #[gtk::template_callbacks]
    impl StationEntry {
        #[template_callback]
        fn handle_swapped_clicked(&self) {
            self.obj().emit_by_name::<()>("swap", &[]);
        }

        fn try_fill_exact_match(&self, search: &str) {
            self.obj().set_place(
                self.completions
                    .borrow()
                    .into_iter()
                    .flatten()
                    .flat_map(|p| p.dynamic_cast::<Place>().ok())
                    .find(|p| p.name() == Some(search.to_owned()))
                    .as_ref(),
            );
        }

        fn on_changed(&self) {
            let obj = self.obj();
            let completions = &self.completions;

            let request_limiter = &self.request_limiter;

            let main_context = MainContext::default();
            main_context.spawn_local(clone!(@strong obj,
                                            @strong completions,
                                            @strong request_limiter
                                            => async move {
                let entry = &obj;
                let text = entry.text().to_string();

                if text.len() < MIN_REQUEST_LEN {
                    return;
                }

                obj.imp().try_fill_exact_match(&text);

                let request = request_limiter.request(text).await;

                if let Some(request) = request {
                    let places = obj.property::<HafasClient>("client").locations(LocationsOptions {query: request.clone(), ..Default::default()}).await;

                    // XXX: Handle error case.
                    if let Ok(places) = places {
                        log::trace!("Got results back. Filling up completions.");
                        let completions = completions.borrow();
                        completions.splice(0, completions.n_items(), &places.into_iter().filter(|p| p.id().is_some()).collect::<Vec<_>>());
                        drop(completions);
                        obj.imp().try_fill_exact_match(&request);
                    }
                } else {
                    log::trace!("No request needed");
                }
            }));
        }

        fn connect_changed(&self, obj: &super::StationEntry) {
            self.obj()
                .connect_changed(clone!(@strong obj => move |_entry| {
                    obj.imp().on_changed();
                }));
            self.on_changed();
        }

        fn update_popover_visible(&self) {
            // Position the popover at the bottom-left of the entry.
            self.popover.set_pointing_to(Some(&gdk::Rectangle::new(
                self.popover.width_request() / 2, // Half the width of the popover.
                self.obj().height(),              // The height of the entry
                0,
                0,
            )));

            self.popover.set_visible(
                self.completions.borrow().n_items() != 0
                    && self.obj().has_css_class("focused")
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

        // Libadwaita has an `document-edit-symbolic. This does not fit in this use case.
        // Upstream does not want to introduce a feature for this. See <https://gitlab.gnome.org/GNOME/libadwaita/-/issues/727>.
        fn hide_edit_icon(&self) {
            let mut child = self.obj().first_child();
            while let Some(c) = child {
                if c.has_css_class("edit-icon") {
                    c.set_visible(false);
                    c.unparent();
                    break;
                } else if c.has_css_class("header") {
                    child = c.first_child();
                } else {
                    child = c.next_sibling();
                }
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for StationEntry {
        const NAME: &'static str = "DBStationEntry";
        type Type = super::StationEntry;
        type ParentType = libadwaita::EntryRow;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            Self::bind_template_callbacks(klass);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for StationEntry {
        fn constructed(&self) {
            let obj = self.obj();
            self.parent_constructed();

            self.popover.set_parent(obj.as_ref());
            self.connect_changed(&obj);
            self.setup_model(&obj);

            self.hide_edit_icon();

            self.completions.borrow().connect_notify_local(
                Some("n-items"),
                clone!(@strong obj => move |_, _| {
                    obj.imp().update_popover_visible();
                }),
            );

            self.obj()
                .connect_css_classes_notify(clone!(@strong obj => move |_| {
                    // Update when e.g. focus changed.
                    obj.imp().update_popover_visible();
                }));

            obj.connect_notify_local(Some("place"), |obj, _| {
                obj.notify("set");
            });
        }

        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                let mut v = StationEntry::derived_properties().to_owned();
                v.extend(vec![
                    ParamSpecObject::builder::<Place>("place").build(),
                    ParamSpecBoolean::builder("set").read_only().build(),
                    ParamSpecObject::builder::<HafasClient>("client").build(),
                    ParamSpecObject::builder::<ListStore>("completions")
                        .read_only()
                        .build(),
                    ParamSpecString::builder("title").build(),
                ]);
                v
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
                            log::trace!("Station-entry got provider change from hafas_client. Resetting");
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
                _ => self.derived_set_property(_id, value, pspec),
            }
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "place" => self.place.borrow().to_value(),
                "set" => self.place.borrow().is_some().to_value(),
                "client" => self.client.borrow().to_value(),
                "completions" => self.completions.borrow().to_value(),
                "title" => self.title.borrow().to_value(),
                _ => self.derived_property(_id, pspec),
            }
        }

        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> =
                Lazy::new(|| -> Vec<Signal> { vec![Signal::builder("swap").build()] });
            SIGNALS.as_ref()
        }
    }

    impl WidgetImpl for StationEntry {}
    impl ListBoxRowImpl for StationEntry {}
    impl PreferencesRowImpl for StationEntry {}
    impl EntryRowImpl for StationEntry {}
}
