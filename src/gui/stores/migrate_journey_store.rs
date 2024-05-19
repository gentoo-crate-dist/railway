//! Note: This file will be removed in the future after it is decided that most people will have migrated already.

use std::fs::File;

use chrono::FixedOffset;
use chrono_tz::Tz;

/// Try to import the journeys store in the old format, converting it to the new format.
pub fn import_old_store(file: File) -> Result<Vec<rcore::Journey>, serde_json::Error> {
    let old_journeys: Vec<hafas_rs::Journey> = serde_json::from_reader(file)?;

    let new_journeys = old_journeys.into_iter().map(convert_journey).collect();

    Ok(new_journeys)
}

// Note: Not a perfect mapping; not sure how this can be improved.
fn convert_time(t: chrono::DateTime<FixedOffset>) -> chrono::DateTime<Tz> {
    t.with_timezone(&chrono_tz::Tz::UTC)
}

fn convert_journey(j: hafas_rs::Journey) -> rcore::Journey {
    rcore::Journey {
        id: j.refresh_token.unwrap_or_default(),
        legs: j.legs.into_iter().map(convert_leg).collect(),
        price: j.price.map(convert_price),
    }
}

fn convert_price(p: hafas_rs::Price) -> rcore::Price {
    rcore::Price {
        amount: p.amount,
        currency: p.currency,
    }
}

fn convert_leg(l: hafas_rs::Leg) -> rcore::Leg {
    rcore::Leg {
        origin: convert_place(l.origin),
        destination: convert_place(l.destination),
        departure: l.departure.map(convert_time),
        planned_departure: l.planned_departure.map(convert_time),
        arrival: l.arrival.map(convert_time),
        planned_arrival: l.planned_arrival.map(convert_time),
        reachable: l.reachable.unwrap_or_default(),
        trip_id: l.trip_id,
        line: l.line.map(convert_line),
        direction: l.direction,
        arrival_platform: l.arrival_platform,
        planned_arrival_platform: l.planned_arrival_platform,
        departure_platform: l.departure_platform,
        planned_departure_platform: l.planned_departure_platform,
        frequency: l.frequency.map(convert_frequency),
        cancelled: l.cancelled.unwrap_or_default(),
        intermediate_locations: l
            .stopovers
            .unwrap_or_default()
            .into_iter()
            .map(convert_stopover)
            .collect(),
        load_factor: l.load_factor.map(convert_load_factor),
        remarks: l
            .remarks
            .unwrap_or_default()
            .into_iter()
            .map(convert_remark)
            .collect(),
        walking: l.walking.unwrap_or_default(),
        transfer: l.transfer.unwrap_or_default(),
        distance: l.distance,
    }
}

fn convert_place(p: hafas_rs::Place) -> rcore::Place {
    match p {
        hafas_rs::Place::Stop(s) => rcore::Place::Station(rcore::Station {
            id: s.id,
            name: s.name,
            location: s.location.map(convert_location),
            products: s
                .products
                .unwrap_or_default()
                .into_iter()
                .map(convert_product)
                .collect(),
        }),
        hafas_rs::Place::Location(l) => rcore::Place::Location(convert_location(l)),
    }
}

fn convert_location(l: hafas_rs::Location) -> rcore::Location {
    match l {
        hafas_rs::Location::Address {
            address,
            latitude,
            longitude,
        } => rcore::Location::Address {
            address,
            latitude,
            longitude,
        },
        hafas_rs::Location::Point {
            id,
            name,
            poi,
            latitude,
            longitude,
        } => rcore::Location::Point {
            id,
            name,
            poi,
            latitude,
            longitude,
        },
    }
}

fn convert_product(p: hafas_rs::Product) -> rcore::Product {
    rcore::Product {
        mode: convert_mode(p.mode),
        name: p.name,
        short: p.short,
    }
}

fn convert_mode(m: hafas_rs::Mode) -> rcore::Mode {
    match m {
        hafas_rs::Mode::Train => rcore::Mode::RegionalTrain, // Note: Not a perfect mapping.
        hafas_rs::Mode::Bus => rcore::Mode::Bus,
        hafas_rs::Mode::Watercraft => rcore::Mode::Ferry,
        hafas_rs::Mode::Taxi => rcore::Mode::OnDemand,
        hafas_rs::Mode::Walking => rcore::Mode::Unknown, // Note: Not a perfect mapping.
        hafas_rs::Mode::Gondola => rcore::Mode::Tram,
    }
}

fn convert_line(l: hafas_rs::Line) -> rcore::Line {
    rcore::Line {
        name: l.name,
        fahrt_nr: l.fahrt_nr,
        mode: convert_mode(l.mode),
        product: convert_product(l.product),
        operator: l.operator.map(convert_operator),
        product_name: l.product_name,
    }
}

fn convert_operator(o: hafas_rs::Operator) -> rcore::Operator {
    rcore::Operator {
        id: o.id,
        name: o.name,
    }
}

fn convert_frequency(f: hafas_rs::Frequency) -> rcore::Frequency {
    rcore::Frequency {
        minimum: f.minimum,
        maximum: f.maximum,
        iterations: f.iterations,
    }
}

fn convert_stopover(s: hafas_rs::Stopover) -> rcore::IntermediateLocation {
    rcore::IntermediateLocation::Stop(rcore::Stop {
        place: convert_place(s.stop),
        departure: s.departure.map(convert_time),
        planned_departure: s.planned_departure.map(convert_time),
        arrival: s.arrival.map(convert_time),
        planned_arrival: s.planned_arrival.map(convert_time),
        arrival_platform: s.arrival_platform,
        planned_arrival_platform: s.planned_arrival_platform,
        departure_platform: s.departure_platform,
        planned_departure_platform: s.planned_departure_platform,
        cancelled: s.cancelled.unwrap_or_default(),
        remarks: s
            .remarks
            .unwrap_or_default()
            .into_iter()
            .map(convert_remark)
            .collect(),
    })
}

fn convert_remark(r: hafas_rs::Remark) -> rcore::Remark {
    rcore::Remark {
        code: r.code,
        text: r.text,
        r#type: match r.r#type {
            hafas_rs::RemarkType::Hint => rcore::RemarkType::Hint,
            hafas_rs::RemarkType::Status => rcore::RemarkType::Hint,
        },
        association: rcore::RemarkAssociation::Unknown, // Note: Not perfect.
        summary: r.summary,
        trip_id: r.trip_id,
    }
}

fn convert_load_factor(l: hafas_rs::LoadFactor) -> rcore::LoadFactor {
    match l {
        hafas_rs::LoadFactor::LowToMedium => rcore::LoadFactor::LowToMedium,
        hafas_rs::LoadFactor::High => rcore::LoadFactor::High,
        hafas_rs::LoadFactor::VeryHigh => rcore::LoadFactor::VeryHigh,
        hafas_rs::LoadFactor::ExceptionallyHigh => rcore::LoadFactor::ExceptionallyHigh,
    }
}

/// The types hafas_rs used previously.
mod hafas_rs {
    pub(crate) mod duration {
        use chrono::Duration;
        use serde::{Deserialize, Deserializer, Serializer};

        pub(crate) fn serialize<S>(v: &Option<Duration>, s: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            if let Some(d) = v {
                s.serialize_some(&d.num_minutes())
            } else {
                s.serialize_none()
            }
        }

        pub(crate) fn deserialize<'de, D>(d: D) -> Result<Option<Duration>, D::Error>
        where
            D: Deserializer<'de>,
        {
            let v = Option::<i64>::deserialize(d)?;
            #[allow(deprecated)]
            Ok(v.map(Duration::minutes))
        }
    }

    use std::borrow::Cow;

    use chrono::DateTime;
    use chrono::Duration;
    use chrono::FixedOffset;
    use serde::{Deserialize, Serialize};

    /* Types */

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(untagged)]
    pub enum Location {
        Address {
            address: String,
            latitude: f32,
            longitude: f32,
        },
        Point {
            #[serde(skip_serializing_if = "Option::is_none")]
            id: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            name: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            poi: Option<bool>,
            latitude: f32,
            longitude: f32,
        },
    }

    impl PartialEq for Location {
        fn eq(&self, other: &Self) -> bool {
            match (&self, other) {
                (Location::Address { address: a, .. }, Location::Address { address: b, .. }) => {
                    a == b
                }
                (Location::Point { id: Some(a), .. }, Location::Point { id: Some(b), .. }) => {
                    a == b
                }
                (_, _) => false,
            }
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    #[serde(tag = "type")]
    #[serde(rename_all = "lowercase")]
    pub enum Place {
        Stop(Stop),
        Location(Location),
    }

    #[derive(Debug, Default, Clone, Serialize, Deserialize)]
    pub struct Stop {
        pub id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub name: Option<String>,
        pub location: Option<Location>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub products: Option<Vec<Product>>,
        //station: Option<Station>,
    }

    impl PartialEq for Stop {
        fn eq(&self, other: &Self) -> bool {
            self.id == other.id
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    #[serde(rename_all = "camelCase")]
    pub struct Product {
        pub id: Cow<'static, str>,
        pub mode: Mode,
        pub bitmasks: Cow<'static, [u16]>,
        pub name: Cow<'static, str>,
        pub short: Cow<'static, str>,
        // TODO: default?
    }

    #[derive(Serialize, Debug, Clone, Deserialize, PartialEq, Eq)]
    #[serde(rename_all = "lowercase")]
    pub enum Mode {
        Train,
        Bus,
        Watercraft,
        Taxi,
        Walking,
        Gondola,
    }

    #[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
    #[serde(rename_all = "kebab-case")]
    pub enum LoadFactor {
        LowToMedium,
        High,
        VeryHigh,
        ExceptionallyHigh,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    #[serde(rename_all = "camelCase")]
    pub struct Line {
        pub name: Option<String>,
        pub fahrt_nr: Option<String>,
        pub mode: Mode,
        pub product: Product,
        pub operator: Option<Operator>,
        pub product_name: Option<String>,
    }

    #[derive(Debug, Serialize, Clone, Deserialize, PartialEq)]
    #[serde(rename_all = "camelCase")]
    pub struct Frequency {
        #[serde(with = "crate::gui::stores::migrate_journey_store::hafas_rs::duration")]
        pub minimum: Option<Duration>,
        #[serde(with = "crate::gui::stores::migrate_journey_store::hafas_rs::duration")]
        pub maximum: Option<Duration>,
        pub iterations: Option<u64>,
    }

    #[derive(Debug, Serialize, Clone, Deserialize, PartialEq)]
    #[serde(rename_all = "camelCase")]
    pub struct Leg {
        pub origin: Place,
        pub destination: Place,
        pub departure: Option<DateTime<FixedOffset>>,
        pub planned_departure: Option<DateTime<FixedOffset>>,
        pub departure_delay: Option<i64>,
        pub arrival: Option<DateTime<FixedOffset>>,
        pub planned_arrival: Option<DateTime<FixedOffset>>,
        pub arrival_delay: Option<i64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub reachable: Option<bool>,
        pub trip_id: Option<String>,
        pub line: Option<Line>,
        pub direction: Option<String>,
        pub arrival_platform: Option<String>,
        pub planned_arrival_platform: Option<String>,
        pub departure_platform: Option<String>,
        pub planned_departure_platform: Option<String>,
        pub frequency: Option<Frequency>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub cancelled: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub stopovers: Option<Vec<Stopover>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub load_factor: Option<LoadFactor>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub remarks: Option<Vec<Remark>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub walking: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub transfer: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub distance: Option<u64>,
    }

    #[derive(Debug, Serialize, Clone, Deserialize, PartialEq)]
    #[serde(rename_all = "camelCase")]
    pub struct Stopover {
        pub stop: Place,
        pub departure: Option<DateTime<FixedOffset>>,
        pub planned_departure: Option<DateTime<FixedOffset>>,
        pub departure_delay: Option<i64>,
        pub arrival: Option<DateTime<FixedOffset>>,
        pub planned_arrival: Option<DateTime<FixedOffset>>,
        pub arrival_delay: Option<i64>,
        pub arrival_platform: Option<String>,
        pub planned_arrival_platform: Option<String>,
        pub departure_platform: Option<String>,
        pub planned_departure_platform: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub cancelled: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub remarks: Option<Vec<Remark>>,
    }

    #[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
    pub struct Price {
        pub amount: f64,
        pub currency: String,
    }

    #[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
    #[serde(rename_all = "camelCase")]
    pub struct Journey {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub refresh_token: Option<String>,
        pub legs: Vec<Leg>,
        pub price: Option<Price>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub struct Operator {
        pub id: String,
        pub name: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    #[serde(rename_all = "lowercase")]
    pub enum RemarkType {
        Hint,
        Status,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub struct Remark {
        pub code: String,
        pub text: String,
        pub r#type: RemarkType,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub summary: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub trip_id: Option<String>,
    }
}
