gtk::glib::wrapper! {
    pub struct FrequencyLabel(ObjectSubclass<imp::FrequencyLabel>)
        @extends libadwaita::Bin, gtk::Widget,
        @implements gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget;
}

pub mod imp {
    use std::cell::RefCell;

    use gdk::glib::Properties;
    use glib::subclass::InitializingObject;
    use gtk::glib;
    use gtk::CompositeTemplate;
    use libadwaita::prelude::*;
    use libadwaita::subclass::prelude::*;

    use crate::backend::Frequency;
    use crate::gui::utility::Utility;

    #[derive(CompositeTemplate, Default, Properties)]
    #[properties(wrapper_type = super::FrequencyLabel)]
    #[template(resource = "/ui/frequency_label.ui")]
    pub struct FrequencyLabel {
        #[property(get, set = Self::set_frequency)]
        #[property(name = "label", type = Option<String>, get = Self::label)]
        frequency: RefCell<Option<Frequency>>,
    }

    impl FrequencyLabel {
        fn set_frequency(&self, frequency: Option<Frequency>) {
            let obj = self.obj();
            obj.set_visible(frequency.is_some());
            self.frequency.replace(frequency);
            obj.notify("label");
        }

        fn label(&self) -> Option<String> {
            self.frequency
                .borrow()
                .as_ref()
                .and_then(Frequency::frequency)
                .and_then(|f| match (f.minimum, f.maximum) {
                    (Some(min), Some(max)) => Some((min + max) / 2),
                    (Some(d), _) | (_, Some(d)) => Some(d),
                    _ => None,
                })
                .map(Utility::format_duration_inline)
                // Translators: Formatting of frequency of trains. The {} will already contain the duration format (most likely min). E.g. `every ~10 min`.
                .map(|x| gettextrs::gettext("every ~{}").replace("{}", &x))
        }
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

    #[glib::derived_properties]
    impl ObjectImpl for FrequencyLabel {}

    impl WidgetImpl for FrequencyLabel {}
    impl BinImpl for FrequencyLabel {}
}
