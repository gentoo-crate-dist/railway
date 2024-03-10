use std::borrow::Borrow;

use gdk::{glib::Object, prelude::Cast, subclass::prelude::ObjectSubclassIsExt};
use gtk::Widget;

use crate::backend::IntermediateLocation;

gtk::glib::wrapper! {
    pub struct IntermediateLocationItem(ObjectSubclass<imp::IntermediateLocationItem>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget;
}

impl IntermediateLocationItem {
    pub fn new(intermediate_location: &IntermediateLocation) -> Self {
        Object::builder::<Self>()
            .property("intermediate-location", intermediate_location)
            .build()
    }

    pub(crate) fn arrival_label(&self) -> Widget {
        let obj = self.imp();
        obj.alt_label_arrival
            .borrow()
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

    use crate::backend::IntermediateLocation;
    use crate::backend::Place;
    use crate::gui::alt_label::AltLabel;
    use crate::gui::utility::Utility;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/ui/intermediate_location_item.ui")]
    pub struct IntermediateLocationItem {
        #[template_child]
        pub(super) alt_label_arrival: TemplateChild<AltLabel>,

        intermediate_location: RefCell<Option<IntermediateLocation>>,
    }

    impl IntermediateLocationItem {
        fn format_intermediate_location_description(
            stop: &str,
            arrival: &Option<String>,
            platform: &Option<String>,
        ) -> String {
            // Translators: The formatting of the intermediate_locations's description for screen readers. Do not translate the strings in {}.
            let format_full = gettextrs::gettext("{stop} at {arrival} on platform {platform}");

            // Translators: The formatting of the intermediate_locations's description for screen readers. Do not translate the strings in {}.
            let format_no_platform = gettextrs::gettext("{stop} at {arrival}");

            match (arrival, platform) {
                (Some(arrival), Some(platform)) => format_full
                    .replace("{stop}", stop)
                    .replace("{arrival}", &arrival)
                    .replace("{platform}", &platform),
                (Some(arrival), None) => format_no_platform
                    .replace("{stop}", stop)
                    .replace("{arrival}", &arrival),
                (_, _) => stop.to_string(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for IntermediateLocationItem {
        const NAME: &'static str = "DBIntermediateLocationItem";
        type Type = super::IntermediateLocationItem;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            WidgetClassExt::set_css_name(klass, "IntermediateLocationItem");
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for IntermediateLocationItem {
        fn constructed(&self) {
            self.parent_constructed();

            self.obj().connect_notify_local(
                Some("intermediate-location"),
                |intermediate_location_item, _| {
                    let intermediate_location = intermediate_location_item
                        .property::<IntermediateLocation>("intermediate_location");
                    let stop = intermediate_location.property::<Place>("place");

                    intermediate_location_item.update_property(&[
                        gtk::accessible::Property::Label(
                            &IntermediateLocationItem::format_intermediate_location_description(
                                &stop.name().unwrap_or_default(),
                                &intermediate_location
                                    .property::<Option<String>>("arrival")
                                    .or(intermediate_location
                                        .property::<Option<String>>("planned-arrival")),
                                &intermediate_location
                                    .property::<Option<String>>("arrival-platform")
                                    .or(intermediate_location
                                        .property::<Option<String>>("planned-arrival-platform")),
                            ),
                        ),
                    ]);
                },
            );
        }

        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecObject::builder::<IntermediateLocation>("intermediate-location")
                        .build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "intermediate-location" => {
                    let obj = value.get::<Option<IntermediateLocation>>().expect(
                        "Property `intermediate-location` of `IntermediateLocationItem` has to be of type `IntermediateLocation`",
                    );

                    self.intermediate_location.replace(obj);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "intermediate-location" => self.intermediate_location.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for IntermediateLocationItem {
        fn focus(&self, direction: DirectionType) -> bool {
            Utility::move_focus_within_container(self, direction)
        }
    }

    impl BoxImpl for IntermediateLocationItem {}
}
