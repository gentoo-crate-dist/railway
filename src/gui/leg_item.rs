use gdk::glib::Object;

use super::objects::LegObject;

gtk::glib::wrapper! {
    pub struct LegItem(ObjectSubclass<imp::LegItem>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget;
}

impl LegItem {
    pub fn new(leg: &LegObject) -> Self {
        Object::new(&[("leg", leg)]).expect("Failed to create LegItem")
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

    use crate::gui::objects::LegObject;
    use crate::gui::objects::StopoverObject;
    use crate::gui::stopover_item::StopoverItem;
    use crate::gui::utility::Utility;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/ui/leg_item.ui")]
    pub struct LegItem {
        #[template_child]
        box_stopovers: TemplateChild<gtk::Box>,

        leg: RefCell<Option<LegObject>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for LegItem {
        const NAME: &'static str = "DBLegItem";
        type Type = super::LegItem;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            Utility::bind_template_callbacks(klass);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for LegItem {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }

        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![ParamSpecObject::new(
                    "leg",
                    "leg",
                    "leg",
                    LegObject::static_type(),
                    ParamFlags::READWRITE,
                )]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "leg" => {
                    let obj = value
                        .get::<Option<LegObject>>()
                        .expect("Property `leg` of `LegItem` has to be of type `LegObject`");

                    // Clear box_legs
                    while let Some(child) = self.box_stopovers.first_child() {
                        self.box_stopovers.remove(&child);
                    }

                    let mut stopovers = obj
                        .as_ref()
                        .map(|j| j.leg().stopovers)
                        .flatten()
                        .unwrap_or_default();
                    // Remove start and end. These are already shown as origin and destination.
                    if !stopovers.is_empty() {
                        stopovers.pop();
                    }
                    if !stopovers.is_empty() {
                        stopovers.remove(0);
                    }

                    // Fill box_legs
                    for stopover in stopovers {
                        self.box_stopovers
                            .append(&StopoverItem::new(&StopoverObject::new(stopover.clone())));
                    }

                    self.leg.replace(obj);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "leg" => self.leg.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for LegItem {}
    impl BoxImpl for LegItem {}
}
