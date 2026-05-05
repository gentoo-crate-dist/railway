use gdk::glib::Object;

use crate::gui::utility::Utility;

use crate::backend::Place;

gtk::glib::wrapper! {
    pub struct Transition(ObjectSubclass<imp::Transition>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget;
}

impl Transition {
    pub fn new(
        walking_time: &Option<chrono::Duration>,
        waiting_time: &Option<chrono::Duration>,
        has_walk: bool,
        is_last_mile: bool,
        final_destination: &Option<Place>,
    ) -> Self {
        let s: Self = Object::builder().build();
        s.setup(
            walking_time,
            waiting_time,
            has_walk,
            is_last_mile,
            final_destination,
        );
        s
    }

    pub fn setup(
        &self,
        walking_time: &Option<chrono::Duration>,
        waiting_time: &Option<chrono::Duration>,
        has_walk: bool,
        is_last_mile: bool,
        final_destination: &Option<Place>,
    ) {
        let walking_time_label = walking_time.map(Utility::format_duration_inline);
        let final_destination_label = final_destination.as_ref().map(Place::name);
        let waiting_time_label = waiting_time.map(Utility::format_duration_inline);
        self.set_walking_time(walking_time_label);
        self.set_waiting_time(waiting_time_label);
        self.set_is_last_mile(is_last_mile);
        self.set_has_walk(has_walk);
        self.set_final_destination(final_destination_label);
    }
}

pub mod imp {
    use std::cell::Cell;
    use std::cell::RefCell;
    use std::marker::PhantomData;

    use crate::gui::utility::Utility;
    use gdk::glib::Properties;
    use gdk::glib::object::ObjectExt;
    use glib::subclass::InitializingObject;
    use gtk::CompositeTemplate;
    use gtk::DirectionType;
    use gtk::glib;
    use gtk::glib::clone;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;

    #[derive(CompositeTemplate, Default, Properties)]
    #[properties(wrapper_type = super::Transition)]
    #[template(resource = "/ui/transition.ui")]
    pub struct Transition {
        #[property(get, set = Self::set_walking_time, nullable)]
        walking_time: RefCell<Option<String>>,
        #[property(get, set = Self::set_waiting_time, nullable)]
        waiting_time: RefCell<Option<String>>,
        #[property(get, set = Self::set_is_last_mile)]
        is_last_mile: Cell<bool>,
        #[property(get, set = Self::set_has_walk)]
        has_walk: Cell<bool>,
        #[property(get, set = Self::set_final_destination, nullable)]
        final_destination: RefCell<Option<String>>,
        #[property(name = "icon", get = Self::icon)]
        _icon: PhantomData<String>,
        #[property(name = "label", get = Self::label)]
        _label: PhantomData<String>,

        #[template_child]
        destination_box: TemplateChild<gtk::Box>,
        #[template_child]
        destination_label: TemplateChild<gtk::Label>,
    }

    impl Transition {
        fn set_walking_time(&self, walking_time: Option<String>) {
            self.walking_time.replace(walking_time);
            self.obj().notify_label();
        }

        fn set_waiting_time(&self, waiting_time: Option<String>) {
            self.waiting_time.replace(waiting_time);
            self.obj().notify_label();
        }

        fn set_is_last_mile(&self, is_last_mile: bool) {
            self.is_last_mile.set(is_last_mile);
            self.obj().notify_icon();
        }

        fn set_has_walk(&self, has_walk: bool) {
            self.has_walk.set(has_walk);
            self.obj().notify_icon();
        }

        fn set_final_destination(&self, final_destination: Option<String>) {
            self.destination_box
                .set_visible(final_destination.is_some());
            self.destination_label
                .set_label(final_destination.as_deref().unwrap_or_default());
            self.final_destination.replace(final_destination);
        }

        fn icon(&self) -> String {
            if !self.has_walk.get() && !self.is_last_mile.get() {
                "change-symbolic".to_owned()
            } else {
                "walking-symbolic".to_owned()
            }
        }

        fn label(&self) -> String {
            match (
                self.walking_time.borrow().clone(),
                self.waiting_time.borrow().clone(),
            ) {
                (Some(walking), Some(waiting)) => gettextrs::gettext("Walk {walk} Wait {wait}")
                    .replace("{walk}", &walking)
                    .replace("{wait}", &waiting),
                (None, Some(waiting)) => {
                    gettextrs::gettext("Transfer Time {}").replace("{}", &waiting)
                }
                (Some(walking), None) => gettextrs::gettext("Walk {}").replace("{}", &walking),
                (None, None) => gettextrs::gettext("Transfer"),
            }
        }

        fn format_transfer_description(
            transfer_description: &str,
            destination: &Option<&String>,
        ) -> String {
            // Translators: Do not translate the strings in {}.
            let format = gettextrs::gettext("Arrive at {destination}.");
            match destination {
                Some(destination) => format!(
                    "{} {}",
                    transfer_description,
                    format.replace("{destination}", destination)
                ),
                None => transfer_description.to_string(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Transition {
        const NAME: &'static str = "DBTransition";
        type Type = super::Transition;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            Utility::bind_template_callbacks(klass);
            WidgetClassExt::set_css_name(klass, "TransferItem");
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for Transition {
        fn constructed(&self) {
            self.parent_constructed();

            self.obj().connect_notify_local(
                None,
                clone!(
                    #[weak(rename_to = transition)]
                    self,
                    move |obj, _| {
                        obj.update_property(&[gtk::accessible::Property::Label(
                            &Transition::format_transfer_description(
                                &obj.property::<String>("label"),
                                &transition.final_destination.borrow().as_ref(),
                            ),
                        )]);
                    }
                ),
            );
        }
    }

    impl WidgetImpl for Transition {
        fn focus(&self, direction: DirectionType) -> bool {
            Utility::move_focus_within_container(self, direction)
        }
    }

    impl BoxImpl for Transition {}
}
