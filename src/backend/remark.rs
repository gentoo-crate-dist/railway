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
        glib::Properties,
        prelude::ObjectExt,
        subclass::prelude::{DerivedObjectProperties, ObjectImpl, ObjectSubclass},
    };

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::Remark)]
    pub struct Remark {
        #[property(name = "text", type = Option<String>, get = |s: &Self| s.remark.borrow().as_ref().map(|r| r.text.clone()))]
        #[property(name = "code", type = Option<String>, get = |s: &Self| s.remark.borrow().as_ref().map(|r| r.code.clone()))]
        #[property(name = "icon-name", type = Option<String>, get = |s: &Self| s.remark.borrow().as_ref().map(|r| super::association_to_icon(&r.association).to_owned()))]
        pub(super) remark: RefCell<Option<rcore::Remark>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Remark {
        const NAME: &'static str = "DBRemark";
        type Type = super::Remark;
    }

    #[glib::derived_properties]
    impl ObjectImpl for Remark {}
}
