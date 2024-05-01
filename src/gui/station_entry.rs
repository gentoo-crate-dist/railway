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
                self.set_position(-1);
            }
            self.imp().completions.borrow().remove_all();
        }
    }
}

pub mod imp {
    use std::cell::{Cell, RefCell};
    use std::time::Duration;

    use gdk::glib::subclass::{InitializingObject, Signal};
    use gdk::glib::{
        clone, ParamSpec, ParamSpecBoolean, ParamSpecObject, ParamSpecString, Propagation, Value,
    };
    use gdk::glib::{MainContext, Properties};
    use gdk::prelude::ObjectExt;
    use gdk::{gio, Key, ModifierType};
    use gtk::subclass::prelude::*;
    use gtk::{gio::ListStore, glib};
    use gtk::{
        prelude::*, ListItem, ListScrollFlags, SignalListItemFactory, SingleSelection, Widget,
        INVALID_LIST_POSITION,
    };
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
        selection: RefCell<SingleSelection>,

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
                selection: RefCell::new(SingleSelection::builder().autoselect(false).build()),
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

        #[template_callback]
        fn handle_activated(&self) {
            if let Some(selected) = self
                .selection
                .borrow()
                .selected_item()
                .and_downcast::<Place>()
            {
                self.obj().set_place(Some(&selected));
            }
        }

        #[template_callback]
        fn handle_completion_activate(&self, pos: u32) {
            if let Some(selected) = self.completions.borrow().item(pos).and_downcast::<Place>() {
                self.obj().set_place(Some(&selected))
            }
        }

        // Adapted from <https://gitlab.gnome.org/GNOME/epiphany/-/blob/b6203597637b4b7725372750983f6e38ca92c2ac/src/ephy-location-entry.c#L436-512>.
        #[template_callback]
        fn handle_key_pressed(&self, key: Key, _: u32, modifier: ModifierType) -> Propagation {
            let obj = self.obj();

            if modifier.intersects(
                gdk::ModifierType::SHIFT_MASK
                    .union(gdk::ModifierType::ALT_MASK)
                    .union(gdk::ModifierType::CONTROL_MASK),
            ) {
                return Propagation::Proceed;
            }

            if key != gdk::Key::Up
                && key != gdk::Key::KP_Up
                && key != gdk::Key::Down
                && key != gdk::Key::KP_Down
            {
                return Propagation::Proceed;
            }

            let selection = self.selection.borrow();
            let model = self.completions.borrow();

            let selected = selection.selected();
            let selected = if selected == INVALID_LIST_POSITION {
                None
            } else {
                Some(selected)
            };
            let matches = model.n_items();

            if matches == 0 {
                return Propagation::Stop;
            }

            let new_selected = match (selected, key) {
                (None, gdk::Key::Up) | (None, gdk::Key::KP_Up) => Some(matches - 1),
                (Some(0), gdk::Key::Up) | (Some(0), gdk::Key::KP_Up) => None,
                (Some(s), gdk::Key::Up) | (Some(s), gdk::Key::KP_Up) => Some(s - 1),
                (None, gdk::Key::Down) | (None, gdk::Key::KP_Down) => Some(0),
                (Some(s), gdk::Key::Down) | (Some(s), gdk::Key::KP_Down) if s == matches - 1 => {
                    None
                }
                (Some(s), gdk::Key::Down) | (Some(s), gdk::Key::KP_Down) => Some(s + 1),
                _ => {
                    // All other cases should be impossible.
                    log::error!("Station entry keyboard selection match incomplete. This should be impossible.");
                    None
                }
            };

            if let Some(new_selected_item_name) = new_selected
                .and_then(|i| model.item(i))
                .and_downcast_ref::<Place>()
                .and_then(|p| p.name())
            {
                // Translators: Text that will be announced by the screen reader when a suggestion was selected.
                let format = gettextrs::gettext("Suggestion {} selected")
                    .replace("{}", &new_selected_item_name);
                obj.announce(&format, gtk::AccessibleAnnouncementPriority::Low);
            } else {
                // Translators: Text that will be announced by the screen reader when no selection was selected.
                let format = gettextrs::gettext("No suggestion selected");
                obj.announce(&format, gtk::AccessibleAnnouncementPriority::Low);
            }

            selection.set_selected(new_selected.unwrap_or(INVALID_LIST_POSITION));
            self.list_completions.scroll_to(
                new_selected.unwrap_or_default(),
                ListScrollFlags::NONE,
                None,
            );

            Propagation::Stop
        }

        fn try_fill_exact_match(&self, search: &str) -> bool {
            let exact = self
                .completions
                .borrow()
                .into_iter()
                .flatten()
                .flat_map(|p| p.dynamic_cast::<Place>().ok())
                .find(|p| p.name() == Some(search.to_owned()));
            self.obj().set_place(exact.as_ref());
            exact.is_some()
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
                    obj.set_place(None);
                    return;
                }

                // Try fill any exact match if available. If it could be filled, don't request.
                if obj.imp().try_fill_exact_match(&text) {
                    return;
                }

                let request = request_limiter.request(text).await;

                if let Some(request) = request {
                    let places = obj.property::<HafasClient>("client").locations(LocationsOptions {query: request.clone(), ..Default::default()}).await;

                    // XXX: Handle error case.
                    if let Ok(places) = places {
                        let places = places.into_iter().filter(|p| p.id().is_some()).collect::<Vec<_>>();
                        log::trace!("Got results back. Filling up completions.");
                        let exact = places.iter()
                            .find(|p| p.name().as_ref() == Some(&request));

                        let completions = completions.borrow();

                        if exact.is_some() {
                            obj.set_place(exact);
                            completions.remove_all();
                        } else {
                            completions.splice(0, completions.n_items(), &places);
                        }
                        drop(completions);
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

            self.popover.set_width_request(self.obj().width());
        }

        fn setup_model(&self) {
            let obj = self.obj();
            let model = &self.completions.borrow();
            let model: &gio::ListModel = model
                .dynamic_cast_ref()
                .expect("ListStore to be a ListModel");
            let selection = self.selection.borrow();
            selection.set_model(Some(model));
            self.list_completions
                .get()
                .set_model(Some(&selection.clone()));

            let factory = SignalListItemFactory::new();
            factory.connect_setup(clone!(@weak obj => move |_, list_item| {
                let place_item = PlaceListItem::new();
                let list_item = list_item
                    .downcast_ref::<ListItem>()
                    .expect("The factory item to be a `ListItem`");

                list_item.set_child(Some(&place_item));
                list_item
                    .property_expression("item")
                    .bind(&place_item, "place", Widget::NONE);

                place_item.connect_local("pressed", false, clone!(@weak obj, @weak place_item => @default-return None, move |_| {
                    obj.set_place(place_item.place().as_ref());
                    None
                }));
            }));
            self.list_completions.set_factory(Some(&factory));
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
            self.setup_model();

            self.hide_edit_icon();

            self.completions.borrow().connect_notify_local(
                Some("n-items"),
                clone!(@strong obj => move |list, _| {
                    obj.imp().update_popover_visible();

                    let n = list.n_items();
                    if n > 0 {
                        // Translators: Text that will be announced by the screen reader when suggestions changed in the station entry.
                        let format = gettextrs::ngettext("one suggestion, not selected", "{n} suggestions, none selected", n).replace("{n}", &n.to_string());
                        obj.announce(&format, gtk::AccessibleAnnouncementPriority::Low);
                    }
                }),
            );

            self.obj()
                .connect_css_classes_notify(clone!(@strong obj => move |_| {
                    // Update when e.g. focus changed.
                    obj.imp().update_popover_visible();
                }));

            obj.connect_notify_local(Some("place"), |obj, _| {
                obj.notify("set");
                obj.imp().update_popover_visible();
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
