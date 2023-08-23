use gdk::glib::Object;

use crate::backend::Leg;

gtk::glib::wrapper! {
    pub struct LegItem(ObjectSubclass<imp::LegItem>)
        @extends gtk::Grid, gtk::Widget,
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
    use gtk::SizeGroup;
    use gtk::SizeGroupMode;
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::CompositeTemplate;
    use once_cell::sync::Lazy;

    use crate::backend::Leg;
    use crate::backend::Remark;
    use crate::backend::Stopover;
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

        leg: RefCell<Option<Leg>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for LegItem {
        const NAME: &'static str = "DBLegItem";
        type Type = super::LegItem;
        type ParentType = gtk::Grid;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
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

                    // Fill box_legs
                    for stopover in &stopovers {
                        let widget = StopoverItem::new(&Stopover::new(stopover.clone()));
                        self.box_stopovers
                            .append(&widget);
                        for w in widget.alt_labels() {
                            size_group.add_widget(&w);
                        }
                    }


                    // Fill box_remarks
                    for remark in remarks {
                        self.box_remarks
                            .append(&RemarkItem::new(&Remark::new(remark.clone())));
                    }

                    let num_stopovers_fmt = gettextrs::gettext("{} stopovers");
                    let num_stopovers_str = num_stopovers_fmt.replace("{}", &stopovers.len().to_string());
                    self.label_num_stopovers.set_label(&num_stopovers_str);

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
    impl GridImpl for LegItem {}
}
