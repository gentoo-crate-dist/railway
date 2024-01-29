use gdk::glib::Object;
use gtk::{gdk, glib, prelude::*, CompositeTemplate};
use libadwaita::subclass::prelude::*;

glib::wrapper! {
    pub struct TimeDivider(ObjectSubclass<imp::TimeDivider>)
        @extends gtk::Box, gtk::Widget, @implements gtk::Accessible;
}

impl Default for TimeDivider {
    fn default() -> Self {
        Object::builder::<Self>().build()
    }
}

mod imp {
    use glib::subclass::InitializingObject;
    use gtk::subclass::box_::BoxImpl;
    use once_cell::sync::Lazy;

    use crate::backend::Journey;

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/ui/time_divider.ui")]
    pub struct TimeDivider {
        #[template_child]
        label_date: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TimeDivider {
        const NAME: &'static str = "DBTimeDivider";
        type Type = super::TimeDivider;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for TimeDivider {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpecObject::builder::<Journey>("item")
                    .write_only()
                    .build(),
                    glib::ParamSpecUInt::builder("start")
                    .write_only()
                    .build()]
            });

            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "item" => {
                    let v = value
                        .get::<Option<Journey>>()
                        .expect("TimeDivider to only get Journey");

                    let formatted = v.map(|v| v.departure_day());
                    self.label_date.set_text(&formatted.unwrap_or_default());
                }
                "start" => {
                    let v = value
                        .get::<u32>()
                        .expect("TimeDivider to only get an unsigned integer");
                    self.obj().set_visible(v != 0);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, _pspec: &glib::ParamSpec) -> glib::Value {
            unimplemented!();
        }
    }

    impl WidgetImpl for TimeDivider {}
    impl BoxImpl for TimeDivider {}
}
