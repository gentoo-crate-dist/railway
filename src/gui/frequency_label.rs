gtk::glib::wrapper! {
    pub struct FrequencyLabel(ObjectSubclass<imp::FrequencyLabel>)
        @extends libadwaita::Bin, gtk::Widget,
        @implements gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget;
}

pub mod imp {
    use std::cell::RefCell;

    use gdk::glib::ParamSpec;
    use gdk::glib::ParamSpecObject;
    use gdk::glib::ParamSpecString;
    use gdk::glib::Value;
    use glib::subclass::InitializingObject;
    use gtk::glib;
    use gtk::CompositeTemplate;
    use libadwaita::prelude::*;
    use libadwaita::subclass::prelude::*;
    use once_cell::sync::Lazy;

    use crate::backend::Frequency;
    use crate::gui::utility::Utility;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/ui/frequency_label.ui")]
    pub struct FrequencyLabel {
        frequency: RefCell<Option<Frequency>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FrequencyLabel {
        const NAME: &'static str = "DBFrequencyLabel";
        type Type = super::FrequencyLabel;
        type ParentType = libadwaita::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for FrequencyLabel {
        fn constructed(&self) {
            self.parent_constructed();
        }

        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecObject::builder::<Frequency>("frequency").build(),
                    ParamSpecString::builder("label").read_only().build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "frequency" => {
                    let obj = value.get::<Option<Frequency>>().expect(
                        "Property `frequency` of `FrequencyLabel` has to be of type `Frequency`",
                    );

                    self.obj().set_visible(obj.is_some());
                    self.frequency.replace(obj);
                    self.obj().notify("label");
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "frequency" => self.frequency.borrow().to_value(),
                "label" => self
                    .frequency
                    .borrow()
                    .as_ref()
                    .and_then(Frequency::frequency)
                    .and_then(|f| match (f.minimum, f.maximum) {
                        (Some(min), Some(max)) => Some((min + max) / 2),
                        (Some(d), _) | (_, Some(d)) => Some(d),
                        _ => None,
                    })
                    .map(Utility::format_duration)
                    // Translators: Formatting of frequency of trains. The {} will already contain the duration format (most likely min). E.g. `every ~10 min`.
                    .map(|x| gettextrs::gettext!("every ~{}", x))
                    .to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for FrequencyLabel {}
    impl BinImpl for FrequencyLabel {}
}
