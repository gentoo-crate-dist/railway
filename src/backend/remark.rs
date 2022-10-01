use std::cell::RefCell;

use gdk::glib::Object;
use gdk::subclass::prelude::ObjectSubclassIsExt;

gtk::glib::wrapper! {
    pub struct Remark(ObjectSubclass<imp::Remark>);
}

impl Remark {
    pub fn new(remark: hafas_rs::Remark) -> Self {
        let s: Self = Object::new(&[]).expect("Failed to create `Remark`.");
        s.imp().remark.swap(&RefCell::new(Some(remark)));
        s
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
    pub struct Remark {
        pub(super) remark: RefCell<Option<hafas_rs::Remark>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Remark {
        const NAME: &'static str = "DBRemark";
        type Type = super::Remark;
    }

    impl ObjectImpl for Remark {
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
