use gdk::{glib::Object, prelude::ObjectExt};

use crate::backend::Leg;

gtk::glib::wrapper! {
    pub struct LegItem(ObjectSubclass<imp::LegItem>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget;
}

impl LegItem {
    pub fn new(leg: &Leg) -> Self {
        Object::builder().property("leg", leg).build()
    }

    pub fn set_leg(&self, leg: &Leg) {
        self.set_property("leg", leg);
    }
}

pub mod imp {
    use std::cell::RefCell;

    use gdk::glib::JoinHandle;
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
    use crate::backend::Leg;
    use crate::backend::Place;
    use crate::backend::Remark;
    use crate::gui::intermediate_location_item::IntermediateLocationItem;
    use crate::gui::remark_item::RemarkItem;
    use crate::gui::utility::Utility;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/ui/leg_item.ui")]
    pub struct LegItem {
        #[template_child]
        box_intermediate_locations: TemplateChild<gtk::Box>,
        #[template_child]
        box_remarks: TemplateChild<gtk::Box>,
        #[template_child]
        reveal_stopovers: TemplateChild<gtk::Revealer>,
        #[template_child]
        label_num_stopovers: TemplateChild<gtk::Label>,
        #[template_child]
        stopover_button: TemplateChild<gtk::ToggleButton>,
        #[template_child]
        remarks_button: TemplateChild<gtk::ToggleButton>,
        #[template_child]
        size_group: TemplateChild<gtk::SizeGroup>,

        leg: RefCell<Option<Leg>>,

        load_handle: RefCell<Option<JoinHandle<()>>>,
    }

    #[gtk::template_callbacks]
    impl LegItem {
        #[template_callback(function)]
        fn format_train_direction(train: &str, destination: &str) -> String {
            // Translators: The formatting of the train going into what direction, i.e. "ICE 123 to Musterberg". Do not translate the strings in {}.
            let format = gettextrs::gettext("{train} in the direction of {destination}");
            format
                .replace("{train}", train)
                .replace("{destination}", destination)
        }

        fn format_trip_segment_description(
            start: &str,
            departure: &str,
            platform_start: &Option<String>,
            destination: &str,
            arrival: &str,
            platform_destination: &Option<String>,
        ) -> String {
            let format_departure = match platform_start {
                Some(_) => {
                    // Translators: Formatting for the segment's comprehensive description for screen readers. Do not translate the strings in {}.
                    gettextrs::gettext("Depart {start} from platform {platform} at {departure}.")
                }
                None => {
                    // Translators: Formatting for the segment's comprehensive description for screen readers. Do not translate the strings in {}.
                    gettextrs::gettext("Depart {start} at {departure}.")
                }
            };
            let format_arrival = match platform_destination {
                Some(_) => {
                    // Translators: Formatting for the segment's comprehensive description for screen readers. Do not translate the strings in {}.
                    gettextrs::gettext(
                        "Arrive at {destination} on platform {platform} at {arrival}.",
                    )
                }
                None => {
                    // Translators: Formatting for the segment's comprehensive description for screen readers. Do not translate the strings in {}.
                    gettextrs::gettext("Arrive at {destination} at {arrival}.")
                }
            };
            format!(
                "{} {}",
                format_departure
                    .replace("{start}", start)
                    .replace("{departure}", departure)
                    // uses fact that None format does not include "{platform}"
                    .replace(
                        "{platform}",
                        platform_start.as_ref().unwrap_or(&"".to_string())
                    ),
                format_arrival
                    .replace("{destination}", destination)
                    .replace("{arrival}", arrival)
                    // uses fact that None format does not include "{platform}"
                    .replace(
                        "{platform}",
                        platform_destination.as_ref().unwrap_or(&"".to_string())
                    )
            )
        }

        fn setup_sync(&self) {
            let (n_stopovers, n_intermediate_locations, remarks) = {
                let obj = self.leg.borrow();
                let (n_stopovers, n_intermediate_locations) = obj
                    .as_ref()
                    .map(|j| j.leg().intermediate_locations)
                    .map(|s| {
                        (
                            s.iter()
                                .filter(|l| matches!(l, rcore::IntermediateLocation::Stop(_)))
                                .count()
                                - 2,
                            s.len() - 2,
                        )
                    })
                    .unwrap_or_default();
                let remarks = obj.as_ref().map(|j| j.leg().remarks).unwrap_or_default();
                (n_stopovers, n_intermediate_locations, remarks)
            };

            if n_intermediate_locations > 0 {
                self.stopover_button.set_visible(true);
                let num_stopovers_fmt = gettextrs::ngettext(
                    "{} Stop",
                    "{} Stops",
                    n_stopovers.try_into().unwrap(),
                );
                let num_stopovers_str = num_stopovers_fmt.replace("{}", &n_stopovers.to_string());
                self.label_num_stopovers.set_label(&num_stopovers_str);
            } else {
                self.stopover_button.set_visible(false);
            }

            // Clear box_remarks
            while let Some(child) = self.box_remarks.first_child() {
                self.box_remarks.remove(&child);
            }

            // Fill box_remarks
            self.remarks_button.set_visible(!remarks.is_empty());
            for remark in remarks {
                self.box_remarks
                    .append(&RemarkItem::new(&Remark::new(remark.clone())));
            }
        }

        async fn setup_async(&self) {
            let stopovers = {
                let obj = self.leg.borrow();
                let mut stopovers = obj
                    .as_ref()
                    .map(|j| j.leg().intermediate_locations)
                    .unwrap_or_default();
                // Remove start and end. These are already shown as origin and destination.
                if !stopovers.is_empty() {
                    stopovers.pop();
                }
                if !stopovers.is_empty() {
                    stopovers.remove(0);
                }
                stopovers
            };

            let size_group = &self.size_group;

            let load_stopovers_async = !self.reveal_stopovers.is_child_revealed();

            let mut current_child = self.box_intermediate_locations.first_child();
            let mut i = 0;
            // Fill box_legs
            while i < stopovers.len() {
                if load_stopovers_async {
                    // Even though we are in glib runtime now, `yield_now` is runtime-agnostic and also seems to work with glib.
                    tokio::task::yield_now().await;
                }

                let stopover = IntermediateLocation::new(stopovers[i].clone());
                // Check if current child can be reused.
                if let Some(child) = current_child.and_downcast_ref::<IntermediateLocationItem>() {
                    child.set_property("intermediate-location", stopover);
                } else {
                    let widget = IntermediateLocationItem::new(&stopover);
                    self.box_intermediate_locations.append(&widget);
                    size_group.add_widget(&widget.arrival_label());
                }
                current_child = current_child.and_then(|c| c.next_sibling());
                i += 1;
            }
            while let Some(c) = current_child {
                current_child = c.next_sibling();
                self.box_intermediate_locations.remove(&c);
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for LegItem {
        const NAME: &'static str = "DBLegItem";
        type Type = super::LegItem;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            Self::bind_template_callbacks(klass);
            Utility::bind_template_callbacks(klass);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for LegItem {
        fn constructed(&self) {
            self.parent_constructed();

            self.obj().connect_notify_local(Some("leg"), |leg_item, _| {
                let leg = leg_item.property::<Leg>("leg");
                let origin = leg.property::<Place>("origin");
                let destination = leg.property::<Place>("destination");

                leg_item.update_property(&[gtk::accessible::Property::Description(
                    &LegItem::format_trip_segment_description(
                        &origin.name().expect("origin of leg must be set"),
                        &leg.property::<Option<String>>("departure")
                            .or(leg.property::<Option<String>>("planned-departure"))
                            .unwrap_or("".to_string()),
                        &leg.property::<Option<String>>("departure-platform")
                            .or(leg.property::<Option<String>>("planned-departure-platform")),
                        &destination.name().expect("destination of leg must be set"),
                        &leg.property::<Option<String>>("arrival")
                            .or(leg.property::<Option<String>>("planned-arrival"))
                            .unwrap_or("".to_string()),
                        &leg.property::<Option<String>>("arrival-platform")
                            .or(leg.property::<Option<String>>("planned-arrival-platform")),
                    ),
                )]);
            });
        }

        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> =
                Lazy::new(|| vec![ParamSpecObject::builder::<Leg>("leg").build()]);
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "leg" => {
                    let obj = value
                        .get::<Option<Leg>>()
                        .expect("Property `leg` of `LegItem` has to be of type `Leg`");

                    self.leg.replace(obj);
                    // Ensure the load is not called twice at the same time by aborting the old one if needed.
                    if let Some(handle) = self.load_handle.replace(None) {
                        handle.abort();
                    }

                    let o = self.obj().clone();
                    // First, setup sync UI which can change the layout.
                    o.imp().setup_sync();
                    // Afterwards, asynchronously add information which will not change the layout.
                    let handle = gspawn!(
                        async move { o.imp().setup_async().await },
                        glib::Priority::LOW
                    );

                    self.load_handle.replace(Some(handle));
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "leg" => self.leg.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for LegItem {
        fn focus(&self, direction: DirectionType) -> bool {
            Utility::move_focus_within_container(self, direction)
        }
    }

    impl BoxImpl for LegItem {}
}
