use std::str::FromStr;
use std::time::Duration;

use gdk::gio;
use gdk::glib::clone;
use gdk::prelude::{ObjectExt, SettingsExt};
use gdk::subclass::prelude::ObjectSubclassIsExt;
use gtk::glib::{self, Object};
use rcore::{JourneysOptions, LocationsOptions, RefreshJourneyOptions};

// Timeout in seconds.
const TIMEOUT: u64 = 30;

type ApiProvider = rapi::RailwayProvider;

use crate::Error;

use super::{Journey, JourneysResult, Place, Provider, TimeType};

fn providers() -> Vec<Provider> {
    vec![
        // Hafas

        // TODO: BVG, SNCB, SNCF, TPG
        Provider::new(
            "AVV",
            "AVV",
            Some("Aachener Verkehrsverbund"),
            &gettextrs::gettext("Germany"),
            true,
        ),
        Provider::new(
            "BART",
            "BART",
            Some("Bay Area Rapid Transit"),
            &gettextrs::gettext("North America"),
            true,
        ),
        Provider::new(
            "BLS",
            "BLS",
            Some("BLS AG"),
            &gettextrs::gettext("Europe"),
            true,
        ),
        Provider::new(
            "CFL",
            "CFL",
            Some("Société Nationale des Chemins de Fer Luxembourgeois"),
            &gettextrs::gettext("Europe"),
            true,
        ),
        Provider::new(
            "CMTA",
            "CapMetro",
            Some("Austin, Texas"),
            &gettextrs::gettext("North America"),
            true,
        ),
        Provider::new(
            "DART",
            "DART",
            Some("Des Moines Area Rapid Transit"),
            &gettextrs::gettext("North America"),
            true,
        ),
        // Always shows "no provider"
        // Provider::new(
        //     "DB-Busradar-Nrw",
        //     "DB Busradar NRW",
        //     // Translators: The state in germany, see https://en.wikipedia.org/wiki/North_Rhine-Westphalia.
        //     Some(&gettextrs::gettext("North Rhine-Westphalia")),
        //     &gettextrs::gettext("Germany"),
        //     true,
        // ),
        Provider::new(
            "DB",
            "DB",
            Some("Deutsche Bahn"),
            &gettextrs::gettext("Germany"),
            true,
        ),
        Provider::new(
            "VGI",
            "VGI",
            Some("Verkehrsgemeinschaft Region Ingolstadt"),
            &gettextrs::gettext("Germany"),
            true,
        ),
        Provider::new(
            "Irish-Rail",
            "Irish Rail",
            Some("Iarnród Éireann"),
            &gettextrs::gettext("Europe"),
            true,
        ),
        Provider::new(
            "IVB",
            "IVB",
            Some("Innsbrucker Verkehrsbetriebe"),
            &gettextrs::gettext("Europe"),
            true,
        ),
        Provider::new(
            "KVB",
            "KVB",
            Some("Kölner Verkehrs-Betriebe"),
            &gettextrs::gettext("Germany"),
            true,
        ),
        Provider::new(
            "Mobiliteit-Lu",
            "Mobiliteit",
            Some("Luxembourg"),
            &gettextrs::gettext("Europe"),
            true,
        ),
        // Migrated endpoint to EFA
        // Provider::new(
        //     "mobil-nrw",
        //     "mobil.nrw",
        //     // Translators: The state in germany, see https://en.wikipedia.org/wiki/North_Rhine-Westphalia.
        //     Some(&gettextrs::gettext("North Rhine-Westphalia")),
        //     &gettextrs::gettext("Germany"),
        //     true,
        // ),
        Provider::new(
            "NVV",
            "NVV",
            Some("Nordhessischer Verkehrsverbund"),
            &gettextrs::gettext("Germany"),
            true,
        ),
        Provider::new(
            "NahSH",
            "Nah.SH",
            // Translators: The state in germany, see https://en.wikipedia.org/wiki/Schleswig-Holstein.
            Some(&gettextrs::gettext("Schleswig-Holstein")),
            &gettextrs::gettext("Germany"),
            true,
        ),
        Provider::new(
            "ooevv",
            "OÖVV",
            Some("Oberösterreichischer Verkehrsverbund"),
            &gettextrs::gettext("Europe"),
            true,
        ),
        Provider::new(
            "OEBB",
            "ÖBB",
            Some("Österreichische Bundesbahnen"),
            &gettextrs::gettext("Europe"),
            true,
        ),
        Provider::new(
            "PKP",
            "PKP",
            Some("Polskie Koleje Państwowe"),
            &gettextrs::gettext("Europe"),
            true,
        ),
        Provider::new(
            "RMV",
            "RMV",
            Some("Rhein-Main-Verkehrsverbund"),
            &gettextrs::gettext("Germany"),
            true,
        ),
        Provider::new(
            "RSAG",
            "RSAG",
            Some("Rostocker Straßenbahn AG"),
            &gettextrs::gettext("Germany"),
            true,
        ),
        Provider::new(
            "Rejseplanen",
            "Rejseplanen",
            Some("Denmark"),
            &gettextrs::gettext("Europe"),
            true,
        ),
        Provider::new(
            "Resrobot",
            "Resrobot",
            Some("Sweden"),
            &gettextrs::gettext("Europe"),
            true,
        ),
        Provider::new(
            "SBahn-Muenchen",
            "S-Bahn München",
            // Translators: The country, see https://en.wikipedia.org/wiki/Germany
            Some(&gettextrs::gettext("Germany")),
            &gettextrs::gettext("Germany"),
            true,
        ),
        Provider::new(
            "Verbundlinie",
            "Verbundlinie",
            Some("Steirischer Verkehrsverbund"),
            &gettextrs::gettext("Europe"),
            true,
        ),
        Provider::new(
            "SVV",
            "SVV",
            Some("Salzburger Verkehrsverbund"),
            &gettextrs::gettext("Europe"),
            true,
        ),
        Provider::new(
            "saarvv",
            "saarVV",
            Some("Saarfahrplan/VGS Saarland"),
            &gettextrs::gettext("Germany"),
            true,
        ),
        Provider::new(
            "Salzburg",
            "Salzburg",
            // Translators: The country, see https://en.wikipedia.org/wiki/Austria
            Some(&gettextrs::gettext("Austria")),
            &gettextrs::gettext("Europe"),
            true,
        ),
        Provider::new(
            "VBB",
            "VBB",
            Some("Berlin & Brandenburg public transport"),
            &gettextrs::gettext("Germany"),
            true,
        ),
        Provider::new(
            "VBN",
            "VBN",
            Some("Verkehrsverbund Bremen/Niedersachsen"),
            &gettextrs::gettext("Germany"),
            true,
        ),
        Provider::new(
            "VKG",
            "VKG/VVK",
            Some("Kärntner Linien/Verkehrsverbund Kärnten"),
            &gettextrs::gettext("Europe"),
            true,
        ),
        Provider::new(
            "VMT",
            "VMT",
            Some("Verkehrsverbund Mittelthüringen"),
            &gettextrs::gettext("Germany"),
            true,
        ),
        Provider::new(
            "VOR",
            "VOR",
            Some("Verkehrsverbund Ost-Region"),
            &gettextrs::gettext("Europe"),
            true,
        ),
        Provider::new(
            "VOS",
            "VOS",
            Some("Verkehrsgemeinschaft Osnabrück"),
            &gettextrs::gettext("Germany"),
            true,
        ),
        Provider::new(
            "VRN",
            "VRN",
            Some("Verkehrsverbund Rhein-Neckar"),
            &gettextrs::gettext("Germany"),
            true,
        ),
        Provider::new(
            "VSN",
            "VSN",
            Some("Verkehrsverbund Süd-Niedersachsen"),
            &gettextrs::gettext("Germany"),
            true,
        ),
        Provider::new(
            "VVT",
            "VVT",
            Some("Verkehrsverbund Tirol"),
            &gettextrs::gettext("Europe"),
            true,
        ),
        Provider::new(
            "VVV",
            "VVV",
            Some("Verkehrsverbund Vorarlberg"),
            &gettextrs::gettext("Europe"),
            true,
        ),
        // Provider::new("ZVV", "ZVV", Some("Zürich public transport"), &gettextrs::gettext("Europe"), true),

        // Search.ch
        Provider::new(
            "search-ch",
            "SBB",
            Some("Switzerland"),
            &gettextrs::gettext("Europe"),
            true,
        ),
        // Transitous
        Provider::new(
            "transitous",
            &gettextrs::gettext("Worldwide (beta)"),
            // Translators: Transitous is a service (transitous.org) which should not be translated.
            Some(&gettextrs::gettext("Using Transitous")),
            "",
            true,
        ),
    ]
}

gtk::glib::wrapper! {
    pub struct Client(ObjectSubclass<imp::Client>);
}

impl std::default::Default for Client {
    fn default() -> Self {
        Self::new()
    }
}

impl Client {
    pub fn new() -> Client {
        let s: Self = Object::builder().build();
        let settings = &s.imp().settings;
        let profile_name = settings.string("search-provider");
        settings.connect_changed(
            Some("search-provider"),
            clone!(
                #[weak]
                s,
                move |settings, _| {
                    s.set_profile(settings.string("search-provider"));
                }
            ),
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
        *write = Some(rapi::RailwayProvider::new(
            rapi::RailwayProviderType::from_str(profile_name.as_ref())
                .unwrap_or(rapi::RailwayProviderType::Db),
            rcore::ReqwestRequesterBuilder::default(),
        ));
        self.emit_by_name::<()>("provider-changed", &[]);
    }

    fn internal(&self) -> ApiProvider {
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

impl Client {
    pub async fn locations(
        &self,
        opts: LocationsOptions,
    ) -> Result<impl Iterator<Item = Place>, Error> {
        use rcore::Provider;
        let client = self.internal();

        Ok(tspawn!(async move {
            tokio::time::timeout(Duration::from_secs(TIMEOUT), client.locations(opts))
                .await
                .map(|r| r.map_err(Error::from))
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
        use rcore::Provider;
        let client = self.internal();
        let from_place = from.place();
        let to_place = to.place();
        let requested_time = match time_type {
            TimeType::Departure => opts.departure,
            TimeType::Arrival => opts.arrival,
        };
        Ok(JourneysResult::new(
            tspawn!(async move {
                tokio::time::timeout(
                    Duration::from_secs(TIMEOUT),
                    client.journeys(from_place, to_place, opts),
                )
                .await
                .map(|r| r.map_err(Error::from))
                .unwrap_or(Err(Error::Timeout))
            })
            .await
            .expect("Failed to join tokio")?,
            from,
            to,
            requested_time,
            time_type,
            self.clone(),
        ))
    }

    pub async fn refresh_journey(
        &self,
        journey: &Journey,
        opts: RefreshJourneyOptions,
    ) -> Result<Journey, Error> {
        use rcore::Provider;
        let client = self.internal();
        let journey = journey.journey();
        Ok(self.get_journey(
            tspawn!(async move {
                tokio::time::timeout(
                    Duration::from_secs(TIMEOUT),
                    client.refresh_journey(&journey, opts),
                )
                .await
                .map(|r| r.map_err(Error::from))
                .unwrap_or(Err(Error::Timeout))
            })
            .await
            .expect("Failed to join tokio")?,
        ))
    }

    pub fn get_journey(&self, journey: rcore::Journey) -> Journey {
        let mut cache = self.imp().journey_cache.borrow_mut();
        if let Some(cached) = cache.get(&journey.id).and_then(|r| r.upgrade()) {
            cached.update(journey);
            cached
        } else {
            let id = journey.id.clone();
            let object = Journey::new(journey, self);
            cache.insert(id, object.downgrade());
            object
        }
    }
}

mod imp {
    use gdk::gio::{ListModel, ListStore};
    use gdk::glib::subclass::Signal;
    use gdk::glib::{ParamSpec, ParamSpecObject, Value, WeakRef};
    use gdk::prelude::{ParamSpecBuilderExt, ToValue};
    use gdk::subclass::prelude::{ObjectImpl, ObjectSubclass};
    use gtk::gio::Settings;
    use gtk::glib;
    use once_cell::sync::Lazy;
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::sync::RwLock;

    use crate::backend::Journey;
    use crate::config;

    pub struct Client {
        pub(super) internal: RwLock<Option<super::ApiProvider>>,

        pub(super) journey_cache: RefCell<HashMap<String, WeakRef<Journey>>>,

        pub(super) settings: Settings,
    }

    impl Default for Client {
        fn default() -> Self {
            Self {
                internal: Default::default(),
                journey_cache: Default::default(),
                settings: Settings::new(config::BASE_ID),
            }
        }
    }

    impl Client {
        pub(super) fn internal(&self) -> super::ApiProvider {
            self.internal
                .read()
                .expect("Failed to read internal client")
                .as_ref()
                .expect("Client internal not yet set")
                .clone()
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Client {
        const NAME: &'static str = "DBClient";
        type Type = super::Client;
    }

    impl ObjectImpl for Client {
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

#[cfg(test)]
mod test {
    #[test]
    fn provider_list_complete() {
        use std::collections::HashSet;

        let supported: HashSet<_> = rapi::RailwayProviderType::variants()
            .iter()
            .map(|p| p.to_string().to_lowercase().replace(['-', '_'], ""))
            .collect();

        let used: HashSet<_> = super::providers()
            .into_iter()
            .map(|p| p.id().to_lowercase().replace(['-', '_'], ""))
            .collect();

        let extra: HashSet<_> = used.difference(&supported).collect();
        let missing: HashSet<_> = supported.difference(&used).collect();

        if !(extra.is_empty() && missing.is_empty()) {
            println!("Extra Providers: {:#?}", extra);
            println!("Missing Providers: {:#?}", missing);
            panic!("Mismatched used and available providers");
        }
    }
}
