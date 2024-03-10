use std::collections::HashSet;

use gdk::prelude::{Cast, ListModelExt};
use gdk::subclass::prelude::ObjectSubclassIsExt;
use gdk::{gio, glib::Object};
use gtk::glib;

use super::{Journey, Place};

#[derive(Debug, Copy, Clone, PartialEq, Eq, glib::Enum)]
#[enum_type(name = "DBTimeType")]
pub enum TimeType {
    Arrival,
    Departure,
}

impl Default for TimeType {
    fn default() -> Self {
        Self::Departure
    }
}

gtk::glib::wrapper! {
    pub struct JourneysResult(ObjectSubclass<imp::JourneysResult>)
        @implements gio::ListModel, gtk::SectionModel, gtk::SelectionModel;
}

impl JourneysResult {
    pub fn new(
        journeys_response: rcore::JourneysResponse,
        source: Place,
        destination: Place,
        time_type: TimeType,
    ) -> Self {
        let s: Self = Object::builder()
            .property("source", source)
            .property("destination", destination)
            .property("earlier-ref", journeys_response.earlier_ref)
            .property("later-ref", journeys_response.later_ref)
            .property("time-type", time_type)
            .build();
        let mut to_insert: Vec<_> = journeys_response
            .journeys
            .into_iter()
            .map(Journey::new)
            .collect();
        let insert_len = to_insert.len();
        s.imp().journeys.borrow_mut().append(&mut to_insert);
        s.items_changed(0, 0, insert_len.try_into().unwrap_or_default());
        s
    }

    pub fn journeys(&self) -> Vec<Journey> {
        self.imp().journeys.borrow().clone()
    }

    pub fn merge_prepend(&self, other: &Self) {
        let current_ids = self.current_ids();
        let mut to_insert = other.journeys();
        // Don't insert duplicate journey IDs.
        to_insert.retain(|j| !current_ids.contains(&j.id()));
        let insert_len = to_insert.len();
        let mut journeys = self.imp().journeys.borrow_mut();
        journeys.splice(0..0, to_insert);
        self.set_earlier_ref(other.earlier_ref());

        drop(journeys);
        self.items_changed(0, 0, insert_len.try_into().unwrap_or_default());
    }

    pub fn merge_append(&self, other: &Self) {
        let current_ids = self.current_ids();
        let mut to_insert = other.journeys();
        // Don't insert duplicate journey IDs.
        to_insert.retain(|j| !current_ids.contains(&j.id()));
        let mut journeys = self.imp().journeys.borrow_mut();
        let prev_length = journeys.len();
        let insert_len = to_insert.len();
        journeys.extend(to_insert);
        self.set_later_ref(other.later_ref());

        drop(journeys);
        self.items_changed(
            prev_length.try_into().unwrap_or_default(),
            0,
            insert_len.try_into().unwrap_or_default(),
        );
    }

    fn current_ids(&self) -> HashSet<String> {
        self.imp()
            .journeys
            .borrow()
            .iter()
            .map(|j| j.id())
            .collect()
    }

    fn items_changed(&self, position: u32, removed: u32, added: u32) {
        self.upcast_ref::<gio::ListModel>()
            .items_changed(position, removed, added);
    }
}

mod imp {
    use gdk::gio;
    use gdk::glib::subclass::types::ObjectSubclassExt;
    use gdk::prelude::{Cast, StaticType};
    use gtk::glib;
    use gtk::prelude::SelectionModelExt;
    use gtk::subclass::section_model::SectionModelImpl;
    use gtk::subclass::selection_model::SelectionModelImpl;
    use std::cell::RefCell;

    use gdk::subclass::prelude::{DerivedObjectProperties, ListModelImpl};
    use gdk::{
        glib::Properties,
        prelude::ObjectExt,
        subclass::prelude::{ObjectImpl, ObjectSubclass},
    };

    use crate::backend::{Journey, Place};

    use super::TimeType;

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::JourneysResult)]
    pub struct JourneysResult {
        pub(super) journeys: RefCell<Vec<Journey>>,

        #[property(get, set, construct_only)]
        source: RefCell<Option<Place>>,
        #[property(get, set, construct_only)]
        destination: RefCell<Option<Place>>,
        #[property(get, set, nullable)]
        earlier_ref: RefCell<Option<String>>,
        #[property(get, set, nullable)]
        later_ref: RefCell<Option<String>>,

        #[property(get, set = Self::set_selected, nullable)]
        selected: RefCell<Option<Journey>>,

        #[property(get, set, construct_only, builder(TimeType::Departure))]
        time_type: RefCell<TimeType>,
    }

    impl JourneysResult {
        fn set_selected(&self, selected: Option<Journey>) {
            self.selected.replace(selected);
            // Don't know where exactly it changed; emit that it changed everywhere.
            self.obj()
                .selection_changed(0, self.journeys.borrow().len() as u32);
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for JourneysResult {
        const NAME: &'static str = "DBJourneysResult";
        type Type = super::JourneysResult;
        type Interfaces = (gio::ListModel, gtk::SectionModel, gtk::SelectionModel);
    }

    #[glib::derived_properties]
    impl ObjectImpl for JourneysResult {}

    impl ListModelImpl for JourneysResult {
        fn item_type(&self) -> glib::Type {
            Journey::static_type()
        }

        fn n_items(&self) -> u32 {
            self.journeys.borrow().len().try_into().unwrap_or_default()
        }

        fn item(&self, position: u32) -> Option<glib::Object> {
            let list = self.journeys.borrow();

            list.get(position as usize)
                .map(|o| o.clone().upcast::<glib::Object>())
        }
    }

    impl SectionModelImpl for JourneysResult {
        fn section(&self, pos: u32) -> (u32, u32) {
            let list = self.journeys.borrow();

            // As per the doc: <https://docs.gtk.org/gtk4/method.SectionModel.get_section.html>
            // > If the position is larger than the number of items, a single range from n_items to G_MAXUINT will be returned.
            if pos >= TryInto::<u32>::try_into(list.len()).unwrap_or_default() {
                return (list.len().try_into().unwrap_or_default(), u32::MAX);
            }

            // Unwrap should never happen due to previous check.
            let day = list
                .get::<usize>(pos.try_into().unwrap_or_default())
                .map(|t: &Journey| t.day_timestamp())
                .unwrap_or_default();

            let first_in = list.partition_point(|t| t.day_timestamp() < day);
            let first_out = list.partition_point(|t| t.day_timestamp() < day + 1);

            (
                first_in.try_into().unwrap_or_default(),
                first_out.try_into().unwrap_or_default(),
            )
        }
    }

    impl SelectionModelImpl for JourneysResult {
        fn is_selected(&self, position: u32) -> bool {
            let list = self.journeys.borrow();
            let selection = self.selected.borrow();
            // Compare by ID.
            list.get(position as usize)
                .is_some_and(|j| Some(j.id()) == selection.as_ref().map(|j| j.id()))
        }

        fn selection_in_range(&self, position: u32, n_items: u32) -> gtk::Bitset {
            let result = gtk::Bitset::new_range(position, n_items);
            let list = self.journeys.borrow();
            let selection = self.selected.borrow();

            let index: Option<u32> = list
                .iter()
                .position(|j| Some(j.id()) == selection.as_ref().map(|j| j.id()))
                .and_then(|v| v.try_into().ok());

            if let Some(index) = index {
                if position <= index && index < position + n_items {
                    result.add(position);
                }
            }
            result
        }
    }
}
