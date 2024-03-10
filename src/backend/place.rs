use std::cell::RefCell;

use gdk::glib::Object;
use gdk::prelude::ObjectExt;
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

    pub fn name(&self) -> Option<String> {
        self.property("name")
    }

    pub fn id(&self) -> Option<String> {
        self.property("id")
    }
}

mod imp {
    use gtk::glib;
    use std::cell::RefCell;

    use gdk::{
        glib::{ParamSpec, ParamSpecString, Value},
        prelude::{ParamSpecBuilderExt, ToValue},
        subclass::prelude::{ObjectImpl, ObjectSubclass},
    };
    use once_cell::sync::Lazy;

    #[derive(Default)]
    pub struct Place {
        pub(super) place: RefCell<Option<rcore::Place>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Place {
        const NAME: &'static str = "DBPlace";
        type Type = super::Place;
    }

    impl ObjectImpl for Place {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecString::builder("name").read_only().build(),
                    ParamSpecString::builder("id").read_only().build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, _value: &Value, _pspec: &ParamSpec) {}

        fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "name" => match self.place.borrow().as_ref() {
                    Some(rcore::Place::Station(s)) => s.name.as_ref().unwrap_or(&s.id).to_value(),
                    Some(rcore::Place::Location(l)) => match l {
                        rcore::Location::Address { address, .. } => address.to_value(),
                        rcore::Location::Point { name, id, .. } => name
                            .as_ref()
                            .unwrap_or_else(|| {
                                id.as_ref().expect("Either name of id for point set")
                            })
                            .to_value(),
                    },
                    _ => unimplemented!(),
                },
                "id" => match self.place.borrow().as_ref() {
                    Some(rcore::Place::Station(s)) => s.id.to_value(),
                    Some(rcore::Place::Location(l)) => match l {
                        rcore::Location::Address { .. } => None::<String>.to_value(),
                        rcore::Location::Point { id, .. } => id.as_ref().to_value(),
                    },
                    _ => unimplemented!(),
                },
                _ => unimplemented!(),
            }
        }
    }
}
