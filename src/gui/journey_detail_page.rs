use gdk::subclass::prelude::ObjectSubclassIsExt;
use hafas_rest::Hafas;

gtk::glib::wrapper! {
    pub struct JourneyDetailPage(ObjectSubclass<imp::JourneyDetailPage>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget;
}

impl JourneyDetailPage {
    pub fn setup(&self, hafas: Hafas) {
        self.imp().setup(hafas);
    }

    pub fn reload(&self) {
        self.imp().reload(self);
    }
}

pub mod imp {
    use std::cell::RefCell;

    use gdk::glib::clone;
    use gdk::glib::MainContext;
    use gdk::glib::ParamFlags;
    use gdk::glib::ParamSpec;
    use gdk::glib::ParamSpecObject;
    use gdk::glib::Value;
    use glib::subclass::InitializingObject;
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::CompositeTemplate;
    use hafas_rest::Hafas;
    use hafas_rest::JourneysRefreshQuery;
    use libadwaita::ToastOverlay;
    use once_cell::sync::Lazy;

    use crate::gui::error::error_to_toast;
    use crate::gui::leg_item::LegItem;
    use crate::gui::objects::JourneyObject;
    use crate::gui::objects::LegObject;
    use crate::gui::utility::Utility;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/ui/journey_detail_page.ui")]
    pub struct JourneyDetailPage {
        #[template_child]
        box_legs: TemplateChild<gtk::Box>,

        #[template_child]
        toast_errors: TemplateChild<ToastOverlay>,

        journey: RefCell<Option<JourneyObject>>,

        hafas: RefCell<Option<Hafas>>,
    }

    impl JourneyDetailPage {
        pub(super) fn setup(&self, hafas: Hafas) {
            self.hafas.replace(Some(hafas));
        }

        pub(super) fn reload(&self, obj: &super::JourneyDetailPage) {
            let hafas_borrow = self.hafas.borrow();
            let hafas = hafas_borrow.as_ref().expect("Hafas should be set up");

            let main_context = MainContext::default();
            main_context.spawn_local(clone!(@strong hafas,
                       @strong obj,
                       @strong self.toast_errors as toast_errors,
                       @strong self.journey as journey => async move {
                let journey_borrow = journey.borrow();
                let journey_obj = journey_borrow.as_ref();

                if let Some(journey) = journey_obj {
                    let result_journey = hafas
                        .journey_refresh(&journey.journey().refresh_token, &JourneysRefreshQuery {
                            stopovers: Some(true),
                            language: Some(gettextrs::gettext("language")),
                            ..Default::default()
                        }).await;
                    if let Ok(result_journey) = result_journey {
                        obj.set_property("journey", JourneyObject::new(result_journey));
                    } else {
                        error_to_toast(&toast_errors, result_journey.err().expect("A error"));
                    }
                }
            }));
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for JourneyDetailPage {
        const NAME: &'static str = "DBJourneyDetailPage";
        type Type = super::JourneyDetailPage;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            Utility::bind_template_callbacks(klass);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for JourneyDetailPage {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }

        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![ParamSpecObject::new(
                    "journey",
                    "journey",
                    "journey",
                    JourneyObject::static_type(),
                    ParamFlags::READWRITE,
                )]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "journey" => {
                    let obj = value.get::<Option<JourneyObject>>()
                        .expect("Property `journey` of `JourneyDetailPage` has to be of type `JourneyObject`");

                    // Clear box_legs
                    while let Some(child) = self.box_legs.first_child() {
                        self.box_legs.remove(&child);
                    }

                    // Fill box_legs
                    let legs = obj.as_ref().map(|j| j.journey().legs).unwrap_or_default();
                    for i in 0..legs.len() {
                        if i != 0 {
                            let from = &legs[i - 1];
                            let to = &legs[i];
                            let time = to.departure - from.arrival;

                            let minutes_fmt = gettextrs::gettext("{} Minutes");
                            let minutes =
                                minutes_fmt.replace("{}", &time.num_minutes().to_string());

                            self.box_legs.append(&gtk::Label::new(Some(&minutes)));
                        }
                        self.box_legs
                            .append(&LegItem::new(&LegObject::new(legs[i].clone())));
                    }

                    self.journey.replace(obj);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "journey" => self.journey.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for JourneyDetailPage {}
    impl BoxImpl for JourneyDetailPage {}
}
