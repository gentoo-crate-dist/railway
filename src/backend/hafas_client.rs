use std::time::Duration;

use gdk::gio;
use gdk::glib::clone;
use gdk::prelude::{ObjectExt, SettingsExt};
use gdk::subclass::prelude::ObjectSubclassIsExt;
use gtk::glib::{self, Object};
use hafas_rs::api::{
    journeys::JourneysOptions, locations::LocationsOptions, refresh_journey::RefreshJourneyOptions,
};
use hafas_rs::profile::db::DbProfile;
use hafas_rs::profile::profile_from_name;

// Timeout in seconds.
const TIMEOUT: u64 = 30;

use crate::Error;

use super::{Journey, JourneysResult, Place, Provider, TimeType};

fn providers() -> Vec<Provider> {
    vec![
        // TODO: BVG, KVB, SNCB, PKP, SNCF, TPG
        Provider::new("AVV", "AVV", Some("Aachener Verkehrsverbund"), true),
        // Seems to use a custom certificate now.
        // Provider::new("BART", "BART", Some("Bay Area Rapid Transit"), true),
        Provider::new("BLS", "BLS", Some("BLS AG"), true),
        Provider::new(
            "CFL",
            "CFL",
            Some("Société Nationale des Chemins de Fer Luxembourgeois"),
            true,
        ),
        Provider::new("CMTA", "CapMetro", Some("Austin, Texas"), true),
        Provider::new("DART", "DART", Some("Des Moines Area Rapid Transit"), true),
        // Always shows "no provider"
        // Provider::new(
        //     "DB-Busradar-Nrw",
        //     "DB Busradar NRW",
        //     // Translators: The state in germany, see https://en.wikipedia.org/wiki/North_Rhine-Westphalia.
        //     Some(&gettextrs::gettext("Nordrhein-Westfalen")),
        //     true,
        // ),
        Provider::new("DB", "DB", Some("Deutsche Bahn"), true),
        Provider::new("INSA", "NASA", Some("Nahverkehr Sachsen-Anhalt"), true),
        Provider::new(
            "VGI",
            "VGI",
            Some("Verkehrsgemeinschaft Region Ingolstadt"),
            true,
        ),
        Provider::new("Irish-Rail", "Irish Rail", Some("Iarnród Éireann"), true),
        Provider::new("Mobiliteit-Lu", "Mobiliteit", Some("Luxembourg"), true),
        Provider::new(
            "mobil-nrw",
            "mobil.nrw",
            // Translators: The state in germany, see https://en.wikipedia.org/wiki/North_Rhine-Westphalia.
            Some(&gettextrs::gettext("Nordrhein-Westfalen")),
            true,
        ),
        Provider::new("NVV", "NVV", Some("Nordhessischer Verkehrsverbund"), true),
        Provider::new(
            "NahSH",
            "Nah.SH",
            // Translators: The state in germany, see https://en.wikipedia.org/wiki/Schleswig-Holstein.
            Some(&gettextrs::gettext("Schleswig-Holstein")),
            true,
        ),
        Provider::new(
            "ooevv",
            "OÖVV",
            Some("Oberösterreichischer Verkehrsverbund"),
            true,
        ),
        Provider::new("OEBB", "ÖBB", Some("Österreichische Bundesbahnen"), true),
        Provider::new("RMV", "RMV", Some("Rhein-Main-Verkehrsverbund"), true),
        Provider::new("RSAG", "RSAG", Some("Rostocker Straßenbahn AG"), true),
        Provider::new("Rejseplanen", "Rejseplanen", Some("Denmark"), true),
        Provider::new(
            "SBahn-Muenchen",
            "S-Bahn München",
            // Translators: The country, see https://en.wikipedia.org/wiki/Germany
            Some(&gettextrs::gettext("Germany")),
            true,
        ),
        Provider::new(
            "Verbundlinie",
            "Verbundlinie",
            Some("Steirischer Verkehrsverbund"),
            true,
        ),
        Provider::new("SVV", "SVV", Some("Salzburger Verkehrsverbund"), true),
        Provider::new("saarvv", "saarVV", Some("Saarfahrplan/VGS Saarland"), true),
        Provider::new(
            "Salzburg",
            "Salzburg",
            // Translators: The country, see https://en.wikipedia.org/wiki/Austria
            Some(&gettextrs::gettext("Austria")),
            true,
        ),
        Provider::new(
            "VBB",
            "VBB",
            Some("Berlin &amp; Brandenburg public transport"),
            true,
        ),
        Provider::new(
            "VBN",
            "VBN",
            Some("Verkehrsverbund Bremen/Niedersachsen"),
            true,
        ),
        Provider::new(
            "VKG",
            "VKG/VVK",
            Some("Kärntner Linien/Verkehrsverbund Kärnten"),
            true,
        ),
        Provider::new("VMT", "VMT", Some("Verkehrsverbund Mittelthüringen"), true),
        Provider::new("VOR", "VOR", Some("Verkehrsverbund Ost-Region"), true),
        Provider::new("VOS", "VOS", Some("Verkehrsgemeinschaft Osnabrück"), true),
        Provider::new("VRN", "VRN", Some("Verkehrsverbund Rhein-Neckar"), true),
        Provider::new(
            "VSN",
            "VSN",
            Some("Verkehrsverbund Süd-Niedersachsen"),
            true,
        ),
        Provider::new("VVT", "VVT", Some("Verkehrsverbund Tirol"), true),
        Provider::new("VVV", "VVV", Some("Verkehrsverbund Vorarlberg"), true),
        // Provider::new("ZVV", "ZVV", Some("Zürich public transport"), true),
    ]
}

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
        settings.connect_changed(
            Some("search-provider"),
            clone!(@weak s => move |settings, _| {
                s.set_profile(settings.string("search-provider"));
            }),
        );
        s.set_profile(profile_name);
        s
    }

    pub fn set_profile<S: AsRef<str>>(&self, profile_name: S) {
        log::info!(
            "The hafas client was changed to: {}.",
            profile_name.as_ref()
        );
        let mut write = self
            .imp()
            .internal
            .write()
            .expect("Profile to be writeable");
        *write = Some(hafas_rs::client::HafasClient::new(
            profile_from_name(profile_name.as_ref()).unwrap_or(Box::new(DbProfile {})),
            hafas_rs::requester::hyper::HyperRustlsRequester::new(),
        ));
        self.emit_by_name::<()>("provider-changed", &[]);
    }

    fn internal(&self) -> hafas_rs::client::HafasClient {
        self.imp().internal()
    }

    pub fn providers(&self) -> gio::ListModel {
        self.property("providers")
    }

    pub fn current_provider(&self) -> Option<Provider> {
        let value = self.imp().settings.string("search-provider");
        providers().into_iter().find(|p| p.id() == value)
    }
}

impl HafasClient {
    pub async fn locations(
        &self,
        opts: LocationsOptions,
    ) -> Result<impl Iterator<Item = Place>, Error> {
        let client = self.internal();

        Ok(tspawn!(async move {
            tokio::time::timeout(Duration::from_secs(TIMEOUT), client.locations(opts))
                .await
                .map(|r| r.map_err(Error::Hafas))
                .unwrap_or(Err(Error::Timeout))
        })
        .await
        .expect("Failed to join tokio")?
        .into_iter()
        .map(Place::new))
    }

    pub async fn journeys(
        &self,
        from: Place,
        to: Place,
        time_type: TimeType,
        opts: JourneysOptions,
    ) -> Result<JourneysResult, Error> {
        let client = self.internal();
        let from_place = from.place();
        let to_place = to.place();
        Ok(JourneysResult::new(
            tspawn!(async move {
                tokio::time::timeout(
                    Duration::from_secs(TIMEOUT),
                    client.journeys(from_place, to_place, opts),
                )
                .await
                .map(|r| r.map_err(Error::Hafas))
                .unwrap_or(Err(Error::Timeout))
            })
            .await
            .expect("Failed to join tokio")?,
            from,
            to,
            time_type,
        ))
    }

    pub async fn refresh_journey<S: AsRef<str>>(
        &self,
        refresh_token: S,
        opts: RefreshJourneyOptions,
    ) -> Result<Journey, Error> {
        let client = self.internal();
        let refresh_token = refresh_token.as_ref().to_owned();
        Ok(Journey::new(
            tspawn!(async move {
                tokio::time::timeout(
                    Duration::from_secs(TIMEOUT),
                    client.refresh_journey(&refresh_token, opts),
                )
                .await
                .map(|r| r.map_err(Error::Hafas))
                .unwrap_or(Err(Error::Timeout))
            })
            .await
            .expect("Failed to join tokio")?,
        ))
    }
}

mod imp {
    use gdk::gio::{ListModel, ListStore};
    use gdk::glib::subclass::Signal;
    use gdk::glib::{ParamSpec, ParamSpecObject, Value};
    use gdk::prelude::{ParamSpecBuilderExt, ToValue};
    use gdk::subclass::prelude::{ObjectImpl, ObjectSubclass};
    use gtk::gio::Settings;
    use gtk::glib;
    use once_cell::sync::Lazy;
    use std::sync::RwLock;

    use crate::config;

    pub struct HafasClient {
        pub(super) internal: RwLock<Option<hafas_rs::client::HafasClient>>,

        pub(super) settings: Settings,
    }

    impl Default for HafasClient {
        fn default() -> Self {
            Self {
                internal: Default::default(),
                settings: Settings::new(config::BASE_ID),
            }
        }
    }

    impl HafasClient {
        pub(super) fn internal(&self) -> hafas_rs::client::HafasClient {
            self.internal
                .read()
                .expect("Failed to read internal client")
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
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![ParamSpecObject::builder::<ListModel>("providers")
                    .read_only()
                    .build()]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, _value: &Value, _pspec: &ParamSpec) {}

        fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "providers" => {
                    let list = ListStore::new::<super::Provider>();
                    list.extend_from_slice(&super::providers());
                    list.to_value()
                }
                _ => unimplemented!(),
            }
        }
        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> =
                Lazy::new(|| -> Vec<Signal> { vec![Signal::builder("provider-changed").build()] });
            SIGNALS.as_ref()
        }
    }
}
