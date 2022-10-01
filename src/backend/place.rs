use std::cell::RefCell;

use gdk::glib::Object;
use gdk::prelude::ObjectExt;
use gdk::subclass::prelude::ObjectSubclassIsExt;

gtk::glib::wrapper! {
    pub struct Place(ObjectSubclass<imp::Place>);
}

impl Place {
    pub fn new(place: hafas_rs::Place) -> Self {
        let s: Self = Object::new(&[]).expect("Failed to create `Place`.");
        s.imp().place.swap(&RefCell::new(Some(place)));
        s
    }

    pub fn place(&self) -> hafas_rs::Place {
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
        glib::{ParamFlags, ParamSpec, ParamSpecString, Value},
        prelude::ToValue,
        subclass::prelude::{ObjectImpl, ObjectSubclass},
    };
    use once_cell::sync::Lazy;

    #[derive(Default, Clone)]
    pub struct Place {
        pub(super) place: RefCell<Option<hafas_rs::Place>>,
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
                    ParamSpecString::new("name", "name", "name", None, ParamFlags::READABLE),
                    ParamSpecString::new("id", "id", "id", None, ParamFlags::READABLE),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _obj: &Self::Type, _id: usize, _value: &Value, _pspec: &ParamSpec) {}

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "name" => match self.place.borrow().as_ref() {
                    Some(hafas_rs::Place::Stop(s)) => s.name.as_ref().unwrap_or(&s.id).to_value(),
                    Some(hafas_rs::Place::Location(l)) => match l {
                        hafas_rs::Location::Address { address, .. } => address.to_value(),
                        hafas_rs::Location::Point { name, id, .. } => name
                            .as_ref()
                            .unwrap_or_else(|| {
                                id.as_ref().expect("Either name of id for point set")
                            })
                            .to_value(),
                    },
                    _ => unimplemented!(),
                },
                "id" => match self.place.borrow().as_ref() {
                    Some(hafas_rs::Place::Stop(s)) => s.id.to_value(),
                    Some(hafas_rs::Place::Location(l)) => match l {
                        hafas_rs::Location::Address { .. } => None::<String>.to_value(),
                        hafas_rs::Location::Point { id, .. } => id.as_ref().to_value(),
                    },
                    _ => unimplemented!(),
                },
                _ => unimplemented!(),
            }
        }
    }
}
