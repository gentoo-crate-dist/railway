use gdk::glib::Object;

use crate::backend::Remark;

gtk::glib::wrapper! {
    pub struct RemarkItem(ObjectSubclass<imp::RemarkItem>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget;
}

impl RemarkItem {
    pub fn new(remark: &Remark) -> Self {
        Object::new(&[("remark", remark)]).expect("Failed to create RemarkItem")
    }
}

pub mod imp {
    use std::cell::RefCell;

    use gdk::glib::ParamFlags;
    use gdk::glib::ParamSpec;
    use gdk::glib::ParamSpecObject;
    use gdk::glib::Value;
    use glib::subclass::InitializingObject;
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::CompositeTemplate;
    use once_cell::sync::Lazy;

    use crate::backend::Remark;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/ui/remark_item.ui")]
    pub struct RemarkItem {
        remark: RefCell<Option<Remark>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RemarkItem {
        const NAME: &'static str = "DBRemarkItem";
        type Type = super::RemarkItem;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for RemarkItem {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }

        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![ParamSpecObject::new(
                    "remark",
                    "remark",
                    "remark",
                    Remark::static_type(),
                    ParamFlags::READWRITE,
                )]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "remark" => {
                    let obj = value
                        .get::<Option<Remark>>()
                        .expect("Property `remark` of `RemarkItem` has to be of type `Remark`");

                    self.remark.replace(obj);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "remark" => self.remark.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for RemarkItem {}
    impl BoxImpl for RemarkItem {}
}
