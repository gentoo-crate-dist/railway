use std::cell::RefCell;

use gdk::glib::clone;
use gdk::prelude::{SettingsExt, ObjectExt};
use gdk::subclass::prelude::ObjectSubclassIsExt;
use gtk::glib::{Object, self};
use hafas_rs::api::{
    journeys::JourneysOptions, locations::LocationsOptions, refresh_journey::RefreshJourneyOptions,
};
use hafas_rs::profile::db::DbProfile;
use hafas_rs::profile::profile_from_name;

use crate::Error;

use super::{Journey, JourneysResult, Place};

gtk::glib::wrapper! {
    pub struct HafasClient(ObjectSubclass<imp::HafasClient>);
}

impl std::default::Default for HafasClient {
    fn default() -> Self {
        Self::new()
    }
}

impl HafasClient {
    pub fn new() -> HafasClient {
        let s: Self = Object::builder().build();
        let settings = &s.imp().settings;
        let profile_name = settings.string("search-provider");
        settings.connect_changed(Some("search-provider"), clone!(@weak s => move |settings, _| {
            s.set_profile(settings.string("search-provider"));
        }));
        s.set_profile(profile_name);
        s
    }

    pub fn set_profile<S: AsRef<str>>(&self, profile_name: S) {
        log::info!("The hafas client was changed to: {}.", profile_name.as_ref());
        self.imp()
            .internal
            .swap(&RefCell::new(Some(hafas_rs::client::HafasClient::new(
                profile_from_name(profile_name.as_ref()).unwrap_or(Box::new(DbProfile {})),
                hafas_rs::requester::hyper::HyperRustlsRequester::new(),
            ))));
        self.emit_by_name::<()>("provider-changed", &[]);
    }

    fn internal(&self) -> hafas_rs::client::HafasClient {
        self.imp().internal()
    }
}

impl HafasClient {
    pub async fn locations(
        &self,
        opts: LocationsOptions,
    ) -> Result<impl Iterator<Item = Place>, Error> {
        Ok(self
            .internal()
            .locations(opts)
            .await?
            .into_iter()
            .map(Place::new))
    }

    pub async fn journeys(
        &self,
        from: Place,
        to: Place,
        opts: JourneysOptions,
    ) -> Result<JourneysResult, Error> {
        Ok(JourneysResult::new(
            self.internal()
                .journeys(from.place(), to.place(), opts)
                .await?,
        ))
    }

    pub async fn refresh_journey<S: AsRef<str>>(
        &self,
        refresh_token: S,
        opts: RefreshJourneyOptions,
    ) -> Result<Journey, Error> {
        Ok(Journey::new(
            self.internal()
                .refresh_journey(refresh_token.as_ref(), opts)
                .await?,
        ))
    }
}

mod imp {
    use gdk::glib::subclass::Signal;
    use gdk::subclass::prelude::{ObjectImpl, ObjectSubclass};
    use gtk::gio::Settings;
    use gtk::glib;
    use once_cell::sync::Lazy;
    use std::cell::RefCell;

    pub struct HafasClient {
        pub(super) internal: RefCell<Option<hafas_rs::client::HafasClient>>,

        pub(super) settings: Settings,
    }

    impl Default for HafasClient {
        fn default() -> Self {
            Self {
                internal: Default::default(),
                settings: Settings::new("de.schmidhuberj.DieBahn"),
            }
        }
    }

    impl HafasClient {
        pub(super) fn internal(&self) -> hafas_rs::client::HafasClient {
            self.internal
                .borrow()
                .as_ref()
                .expect("HafasClient internal not yet set")
                .clone()
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for HafasClient {
        const NAME: &'static str = "DBHafasClient";
        type Type = super::HafasClient;
    }

    impl ObjectImpl for HafasClient {
        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| -> Vec<Signal> {
                vec![Signal::builder("provider-changed")
                    .build()]
            });
            SIGNALS.as_ref()
        }
    }
}
