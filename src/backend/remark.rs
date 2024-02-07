use std::cell::RefCell;

use gdk::glib::Object;
use gdk::subclass::prelude::ObjectSubclassIsExt;

gtk::glib::wrapper! {
    pub struct Remark(ObjectSubclass<imp::Remark>);
}

impl Remark {
    pub fn new(remark: hafas_rs::Remark) -> Self {
        let s: Self = Object::builder().build();
        s.imp().remark.swap(&RefCell::new(Some(remark)));
        s
    }
}

// TODO: This is probably DB-specific.
// Move this to future profile backend.
fn code_to_icon<S: AsRef<str>>(code: S) -> &'static str {
    match code.as_ref() {
        // Bikes limited
        "FB" |
        // Bike free
        "KF" |
        // Bike times limited
        "FS" => "cycling-symbolic",

        // Places for wheelchairs
        "RO" |
        // Accessible equipment
        "RG" | "EA" | "ER" |
        // Ramp for wheelchairs
        "EH" |
        // Boarding aid at center of train
        "ZM" |
        // Accessible only at limited stations
        "SI" => "wheelchair-symbolic",

        // Ticket machine in train
        "FM" | "FZ" |
        // Reservation upfront at service points and vending machines possible
        "RC" => "ticket-symbolic",

        // Power sockers
        "LS" => "power-symbolic",
        // Air conditioning
        "KL" => "thermometer-symbolic",
        // WiFi
        "WV" => "network-wireless-signal-excellent-symbolic",

        // Only second class
        "K2" => "ticket-second-class-symbolic",


        // Hamburg mobility info link
        "HM" |
        // Schleswig-Holstein mobility / accesibility info link
        "SM" |
        // RRX Rhein-Ruhr-Express
        "N " |


        // Not specified
        "" => "dialog-information-symbolic",
        c => {
            log::debug!("Found unknown remark code: {}", c);
            "dialog-information-symbolic"
        }
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
                    .map(|r| super::code_to_icon(&r.code))
                    .to_value(),
                _ => unimplemented!(),
            }
        }
    }
}
