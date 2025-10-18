use std::cell::RefCell;

use gdk::glib::Object;
use gdk::subclass::prelude::ObjectSubclassIsExt;

gtk::glib::wrapper! {
    pub struct Place(ObjectSubclass<imp::Place>);
}

impl Place {
    pub fn new(place: rcore::Place) -> Self {
        let s: Self = Object::builder().build();
        s.imp().place.swap(&RefCell::new(Some(place)));
        s
    }

    pub fn place(&self) -> rcore::Place {
        self.imp()
            .place
            .borrow()
            .clone()
            .expect("Station not yet set up")
    }
}

mod imp {
    use gtk::glib;
    use std::cell::RefCell;

    use gdk::{
        glib::Properties,
        prelude::ObjectExt,
        subclass::prelude::{DerivedObjectProperties, ObjectImpl, ObjectSubclass},
    };

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::Place)]
    pub struct Place {
        #[property(name = "name", type = Option<String>, get = Self::name)]
        #[property(name = "id", type = Option<String>, get = Self::id)]
        pub(super) place: RefCell<Option<rcore::Place>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Place {
        const NAME: &'static str = "DBPlace";
        type Type = super::Place;
    }

    impl Place {
        fn name(&self) -> Option<String> {
            match self.place.borrow().as_ref() {
                Some(rcore::Place::Station(s)) => Some(s.name.as_ref().unwrap_or(&s.id).to_owned()),
                Some(rcore::Place::Location(l)) => match l {
                    rcore::Location::Address { address, .. } => Some(address.to_owned()),
                    rcore::Location::Point { name, id, .. } => Some(
                        name.as_ref()
                            .unwrap_or_else(|| {
                                id.as_ref().expect("Either name of id for point set")
                            })
                            .to_owned(),
                    ),
                },
                _ => unimplemented!(),
            }
        }

        fn id(&self) -> Option<String> {
            match self.place.borrow().as_ref() {
                Some(rcore::Place::Station(s)) => Some(s.id.to_owned()),
                Some(rcore::Place::Location(l)) => match l {
                    rcore::Location::Address { .. } => None::<String>,
                    rcore::Location::Point { id, .. } => id.to_owned(),
                },
                _ => unimplemented!(),
            }
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for Place {}
}
