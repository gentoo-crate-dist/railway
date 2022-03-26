use std::cell::RefCell;

use gdk::glib::Object;
use gdk::subclass::prelude::ObjectSubclassIsExt;
use hafas_rest::Remark;

gtk::glib::wrapper! {
    pub struct RemarkObject(ObjectSubclass<imp::RemarkObject>);
}

impl RemarkObject {
    pub fn new(remark: Remark) -> Self {
        let s: Self = Object::new(&[]).expect("Failed to create `RemarkObject`.");
        s.imp().remark.swap(&RefCell::new(Some(remark)));
        s
    }
}

mod imp {
    use gtk::glib;
    use hafas_rest::Remark;
    use std::cell::RefCell;

    use gdk::{
        glib::{ParamFlags, ParamSpec, ParamSpecString, Value},
        prelude::ToValue,
        subclass::prelude::{ObjectImpl, ObjectSubclass},
    };
    use once_cell::sync::Lazy;

    #[derive(Default, Clone)]
    pub struct RemarkObject {
        pub(super) remark: RefCell<Option<Remark>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RemarkObject {
        const NAME: &'static str = "DBRemarkObject";
        type Type = super::RemarkObject;
    }

    impl ObjectImpl for RemarkObject {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![ParamSpecString::new(
                    "text",
                    "text",
                    "text",
                    None,
                    ParamFlags::READABLE,
                )]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _obj: &Self::Type, _id: usize, _value: &Value, _pspec: &ParamSpec) {}

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "text" => self.remark.borrow().as_ref().map(|r| &r.text).to_value(),
                _ => unimplemented!(),
            }
        }
    }
}
