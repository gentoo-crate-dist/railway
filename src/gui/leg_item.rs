use gdk::glib::Object;

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
    use gtk::SizeGroup;
    use gtk::SizeGroupMode;
    use once_cell::sync::Lazy;

    use crate::backend::Leg;
    use crate::backend::Remark;
    use crate::backend::Stopover;
    use crate::gui::alt_label::AltLabel;
    use crate::gui::remark_item::RemarkItem;
    use crate::gui::stopover_item::StopoverItem;
    use crate::gui::utility::Utility;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/ui/leg_item.ui")]
    pub struct LegItem {
        #[template_child]
        box_stopovers: TemplateChild<gtk::Box>,
        #[template_child]
        box_remarks: TemplateChild<gtk::Box>,
        #[template_child]
        label_num_stopovers: TemplateChild<gtk::Label>,
        #[template_child]
        stopover_button: TemplateChild<gtk::ToggleButton>,
        #[template_child]
        remarks_button: TemplateChild<gtk::ToggleButton>,
        #[template_child]
        start_departure_label: TemplateChild<AltLabel>,
        #[template_child]
        destination_arrival_label: TemplateChild<AltLabel>,
        #[template_child]
        spacing: TemplateChild<libadwaita::Bin>,

        leg: RefCell<Option<Leg>>,
    }

    #[gtk::template_callbacks]
    impl LegItem {
        /// Every time when the page is not yet filled with the journeys, load more.
        #[template_callback(function)]
        fn format_train_direction(train: &str, destination: &str) -> String {
            // Translators: The formattign of the train going into what direction, i.e. "ICE 123 to Musterberg". Do not translate the strings in {}.
            let format = gettextrs::gettext("{train} to {destination}");
            format
                .replace("{train}", train)
                .replace("{destination}", destination)
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

                    // Clear box_legs
                    while let Some(child) = self.box_stopovers.first_child() {
                        self.box_stopovers.remove(&child);
                    }
                    // Clear box_remarks
                    while let Some(child) = self.box_remarks.first_child() {
                        self.box_remarks.remove(&child);
                    }

                    let mut stopovers = obj
                        .as_ref()
                        .and_then(|j| j.leg().stopovers)
                        .unwrap_or_default();
                    let remarks = obj
                        .as_ref()
                        .and_then(|j| j.leg().remarks)
                        .unwrap_or_default();
                    // Remove start and end. These are already shown as origin and destination.
                    if !stopovers.is_empty() {
                        stopovers.pop();
                    }
                    if !stopovers.is_empty() {
                        stopovers.remove(0);
                    }

                    let size_group = SizeGroup::new(SizeGroupMode::Horizontal);

                    size_group.add_widget(&self.start_departure_label.get());
                    size_group.add_widget(&self.destination_arrival_label.get());
                    size_group.add_widget(&self.spacing.get());

                    // Fill box_legs
                    for stopover in &stopovers {
                        let widget = StopoverItem::new(&Stopover::new(stopover.clone()));
                        self.box_stopovers.append(&widget);
                        size_group.add_widget(&widget.arrival_label());
                    }

                    // Fill box_remarks
                    self.remarks_button.set_visible(remarks.len() > 0);
                    for remark in remarks {
                        self.box_remarks
                            .append(&RemarkItem::new(&Remark::new(remark.clone())));
                    }

                    let n_stopovers = stopovers.len();
                    if n_stopovers > 0 {
                        self.stopover_button.set_visible(true);
                        let num_stopovers_fmt = gettextrs::ngettext(
                            "{} stopover",
                            "{} stopovers",
                            n_stopovers.try_into().unwrap(),
                        );
                        let num_stopovers_str =
                            num_stopovers_fmt.replace("{}", &n_stopovers.to_string());
                        self.label_num_stopovers.set_label(&num_stopovers_str);
                    } else {
                        self.stopover_button.set_visible(false);
                    }

                    self.leg.replace(obj);
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

    impl WidgetImpl for LegItem {}
    impl BoxImpl for LegItem {}
}
