use std::cell::RefCell;

use gdk::glib::Object;
use gdk::subclass::prelude::ObjectSubclassIsExt;
use rcore::RemarkAssociation;

gtk::glib::wrapper! {
    pub struct Remark(ObjectSubclass<imp::Remark>);
}

impl Remark {
    pub fn new(remark: rcore::Remark) -> Self {
        let s: Self = Object::builder().build();
        s.imp().remark.swap(&RefCell::new(Some(remark)));
        s
    }
}

fn association_to_icon(association: &RemarkAssociation) -> &'static str {
    match association {
        RemarkAssociation::Bike => "cycling-symbolic",
        RemarkAssociation::Accessibility => "wheelchair-symbolic",
        RemarkAssociation::Ticket => "ticket-symbolic",

        RemarkAssociation::Power => "power-symbolic",
        RemarkAssociation::AirConditioning => "thermometer-symbolic",
        RemarkAssociation::WiFi => "network-wireless-signal-excellent-symbolic",

        RemarkAssociation::OnlySecondClass => "ticket-second-class-symbolic",
        _ => "dialog-information-symbolic",
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
    pub struct Remark {
        pub(super) remark: RefCell<Option<rcore::Remark>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Remark {
        const NAME: &'static str = "DBRemark";
        type Type = super::Remark;
    }

    impl ObjectImpl for Remark {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecString::builder("text").read_only().build(),
                    ParamSpecString::builder("icon-name").read_only().build(),
                    ParamSpecString::builder("code").read_only().build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, _value: &Value, _pspec: &ParamSpec) {}

        fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "text" => self.remark.borrow().as_ref().map(|r| &r.text).to_value(),
                "code" => self.remark.borrow().as_ref().map(|r| &r.code).to_value(),
                "icon-name" => self
                    .remark
                    .borrow()
                    .as_ref()
                    .map(|r| super::association_to_icon(&r.association))
                    .to_value(),
                _ => unimplemented!(),
            }
        }
    }
}
