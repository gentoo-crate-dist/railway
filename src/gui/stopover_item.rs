use std::borrow::Borrow;

use gdk::{glib::Object, subclass::prelude::ObjectSubclassIsExt, prelude::Cast};
use gtk::Widget;

use crate::backend::Stopover;

gtk::glib::wrapper! {
    pub struct StopoverItem(ObjectSubclass<imp::StopoverItem>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget;
}

impl StopoverItem {
    pub fn new(stopover: &Stopover) -> Self {
        Object::builder::<Self>()
            .property("stopover", stopover)
            .build()
    }

    pub (crate) fn arrival_label(&self) -> Widget {
        let obj = self.imp();
        obj.alt_label_arrival.borrow()
            .dynamic_cast_ref::<gtk::Widget>()
            .expect("AltLabel to be a Widget")
            .clone()
    }
}

pub mod imp {
    use std::cell::RefCell;

    use gdk::glib::ParamSpec;
    use gdk::glib::ParamSpecObject;
    use gdk::glib::Value;
    use glib::subclass::InitializingObject;
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::CompositeTemplate;
    use gtk::DirectionType;
    use once_cell::sync::Lazy;

    use crate::backend::Place;
    use crate::backend::Stopover;
    use crate::gui::alt_label::AltLabel;
    use crate::gui::utility::Utility;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/ui/stopover_item.ui")]
    pub struct StopoverItem {
        #[template_child]
        pub(super) alt_label_arrival: TemplateChild<AltLabel>,

        stopover: RefCell<Option<Stopover>>,
    }

    impl StopoverItem {
        fn format_stopover_description(stop: &str, arrival: &Option<String>) -> String {
            // Translators: The formatting of the stopovers's description for screen readers. Do not translate the strings in {}.
            let format = gettextrs::gettext("{stop} at {arrival}");

            match arrival {
                Some(arrival) => format.replace("{stop}", stop).replace("{arrival}", &arrival),
                None => stop.to_string(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for StopoverItem {
        const NAME: &'static str = "DBStopoverItem";
        type Type = super::StopoverItem;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            WidgetClassExt::set_css_name(klass, "StopoverItem");
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for StopoverItem {
        fn constructed(&self) {
            self.parent_constructed();

            self.obj().connect_notify_local(Some("stopover"), |stopover_item, _| {
                let stopover = stopover_item.property::<Stopover>("stopover");
                let stop = stopover.property::<Place>("stop");

                stopover_item.update_property(&[
                    gtk::accessible::Property::Label(&StopoverItem::format_stopover_description(
                        &stop.name().unwrap_or_default(),
                        &stopover.property::<Option<String>>("arrival")
                            .or(stopover.property::<Option<String>>("planned-arrival")),
                    ))
                ]);
            });
        }

        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> =
                Lazy::new(|| vec![ParamSpecObject::builder::<Stopover>("stopover").build()]);
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "stopover" => {
                    let obj = value.get::<Option<Stopover>>().expect(
                        "Property `stopover` of `StopoverItem` has to be of type `Stopover`",
                    );

                    self.stopover.replace(obj);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "stopover" => self.stopover.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for StopoverItem {
        fn focus(&self, direction: DirectionType) -> bool {
            Utility::move_focus_within_container(self, direction)
        }
    }

    impl BoxImpl for StopoverItem {}
}
