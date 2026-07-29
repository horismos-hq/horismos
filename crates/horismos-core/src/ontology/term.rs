//! Interned IRIs and the term/value model: `Iri`, `Ref`, `Value`.
//!
//! `Value` mirrors the XSD subset of the RDFS-plus profile (`xsd:decimal`,
//! `xsd:integer`, `xsd:string`, `xsd:boolean`, `xsd:dateTime`,
//! `xsd:duration`) plus `Ref` for node-valued properties. IRIs are stored as
//! `u32`-newtype tokens interned in an arena (decision D2,
//! `PRPs/PRP-001-ontology-core/02-tasks-and-gaps.md`); adjacency-map instance
//! storage arrives with the World in U6.
//!
//! Spec: `docs/framework/02-ontology.md`, `docs/framework/12-rust-api.md`
//! (§ Core types) · Owner: PRP-001

use std::collections::HashMap;
use std::fmt;

use chrono::{DateTime, FixedOffset};
use rust_decimal::Decimal;

use crate::error::OntologyError;

/// An interned IRI: a `Copy` token identifying a string held once in an
/// [`Interner`]. Two `Iri`s compare equal iff their source strings were
/// equal at intern time (decision D2, `u32`-newtype interned IRIs in an
/// arena).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Iri(u32);

/// An arena of interned IRI strings.
///
/// `intern` deduplicates by string equality; the returned token depends only
/// on insertion order (first-seen index), never on the internal `HashMap`'s
/// randomized hasher or its iteration order — so behavior is deterministic
/// across runs, satisfying the no-iteration-order-dependence rule
/// (CLAUDE.md).
/// Cloning is a deep copy of the arena. That is exactly what
/// [`crate::world::World::new`] does: a world starts from its ontology's
/// interner — so every schema `Iri` keeps its token — and interns instance
/// IRIs into its own copy from there, leaving the (immutable, shareable)
/// ontology untouched.
#[derive(Debug, Default, Clone)]
pub struct Interner {
    strings: Vec<String>,
    ids: HashMap<String, Iri>,
}

impl Interner {
    /// Creates an empty interner.
    pub fn new() -> Self {
        Self::default()
    }

    /// Interns `iri`, returning its token. Interning an equal string again —
    /// in this call or a prior one — returns the same token.
    pub fn intern(&mut self, iri: &str) -> Iri {
        if let Some(&id) = self.ids.get(iri) {
            return id;
        }
        let id = Iri(self.strings.len() as u32);
        self.strings.push(iri.to_owned());
        self.ids.insert(iri.to_owned(), id);
        id
    }

    /// The token for `iri` if it has already been interned — the lookup-only
    /// counterpart of [`Interner::intern`], for callers that must not mutate
    /// the arena (an immutable [`crate::Ontology`], say).
    pub fn get(&self, iri: &str) -> Option<Iri> {
        self.ids.get(iri).copied()
    }

    /// Resolves a token back to its interned string.
    ///
    /// # Panics
    /// Panics if `iri` was not produced by this `Interner`.
    pub fn resolve(&self, iri: Iri) -> &str {
        &self.strings[iri.0 as usize]
    }
}

/// A reference to a node — an instance/subject, identified by its interned
/// IRI. Blank-node skolemization is out of scope here; it arrives with the
/// World in U6. Today every `Ref` wraps an already-named IRI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Ref(Iri);

impl Ref {
    /// Wraps an interned IRI as a node reference.
    pub fn new(iri: Iri) -> Self {
        Self(iri)
    }

    /// The wrapped IRI.
    pub fn iri(self) -> Iri {
        self.0
    }
}

/// The XSD datatype subset of the RDFS-plus profile
/// (`docs/framework/02-ontology.md`). Does not include a `Ref` variant — a
/// node reference is not an XSD literal type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Datatype {
    Decimal,
    Integer,
    String,
    Boolean,
    DateTime,
    Duration,
}

impl fmt::Display for Datatype {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Datatype::Decimal => "xsd:decimal",
            Datatype::Integer => "xsd:integer",
            Datatype::String => "xsd:string",
            Datatype::Boolean => "xsd:boolean",
            Datatype::DateTime => "xsd:dateTime",
            Datatype::Duration => "xsd:duration",
        };
        f.write_str(name)
    }
}

/// An exact `xsd:duration` value: sign plus component magnitudes. Years and
/// months are folded into a single `months` count; seconds keep exact
/// (decimal) precision. Equality is component-wise; ordering is not
/// implemented (not required by U1 — `xsd:duration` ordering is partial in
/// general, since e.g. months-vs-days isn't comparable without a calendar).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct XsdDuration {
    pub negative: bool,
    pub months: i64,
    pub days: i64,
    pub hours: i64,
    pub minutes: i64,
    pub seconds: Decimal,
}

impl XsdDuration {
    /// Parses the XSD lexical form of `xsd:duration`:
    /// `-?P(nY)?(nM)?(nD)?(T(nH)?(nM)?(nS)?)?`, with at least one component
    /// present.
    pub fn parse(lexical: &str) -> Result<Self, OntologyError> {
        let invalid = || invalid_lexical(Datatype::Duration, lexical);

        let mut rest = lexical;
        let negative = if let Some(r) = rest.strip_prefix('-') {
            rest = r;
            true
        } else {
            false
        };
        let rest = rest.strip_prefix('P').ok_or_else(invalid)?;

        let (date_part, time_part) = match rest.split_once('T') {
            Some((d, t)) => (d, Some(t)),
            None => (rest, None),
        };

        let (years, date_rest) = take_int(date_part, 'Y').map_err(|()| invalid())?;
        let (months_part, date_rest) = take_int(date_rest, 'M').map_err(|()| invalid())?;
        let (days, date_rest) = take_int(date_rest, 'D').map_err(|()| invalid())?;
        if !date_rest.is_empty() {
            return Err(invalid());
        }

        let mut had_component = years.is_some() || months_part.is_some() || days.is_some();

        let mut hours = 0i64;
        let mut minutes = 0i64;
        let mut seconds = Decimal::ZERO;
        if let Some(t) = time_part {
            let (h, t_rest) = take_int(t, 'H').map_err(|()| invalid())?;
            let (mi, t_rest) = take_int(t_rest, 'M').map_err(|()| invalid())?;
            let (sec, t_rest) = take_decimal(t_rest, 'S').map_err(|()| invalid())?;
            if !t_rest.is_empty() {
                return Err(invalid());
            }
            if h.is_none() && mi.is_none() && sec.is_none() {
                // "T" present but with nothing after it — not a valid duration.
                return Err(invalid());
            }
            hours = h.unwrap_or(0);
            minutes = mi.unwrap_or(0);
            seconds = sec.unwrap_or(Decimal::ZERO);
            had_component = true;
        }

        if !had_component {
            return Err(invalid());
        }

        Ok(XsdDuration {
            negative,
            months: years.unwrap_or(0) * 12 + months_part.unwrap_or(0),
            days: days.unwrap_or(0),
            hours,
            minutes,
            seconds,
        })
    }

    /// Serializes to the canonical XSD lexical form.
    pub fn to_lexical(&self) -> String {
        let years = self.months / 12;
        let months = self.months % 12;

        let mut body = String::new();
        if years != 0 {
            body.push_str(&years.to_string());
            body.push('Y');
        }
        if months != 0 {
            body.push_str(&months.to_string());
            body.push('M');
        }
        if self.days != 0 {
            body.push_str(&self.days.to_string());
            body.push('D');
        }

        let mut time = String::new();
        if self.hours != 0 {
            time.push_str(&self.hours.to_string());
            time.push('H');
        }
        if self.minutes != 0 {
            time.push_str(&self.minutes.to_string());
            time.push('M');
        }
        if !self.seconds.is_zero() {
            time.push_str(&self.seconds.to_string());
            time.push('S');
        }
        if !time.is_empty() {
            body.push('T');
            body.push_str(&time);
        }

        if body.is_empty() {
            body.push_str("T0S");
        }

        format!("{}P{body}", if self.negative { "-" } else { "" })
    }
}

/// Consumes a leading `<digits><unit>` component from `s`, if present.
/// Returns `(None, s)` unchanged if `unit` doesn't occur in `s`; errors if
/// `unit` occurs but is not preceded purely by ASCII digits (out-of-order or
/// malformed input).
fn take_int(s: &str, unit: char) -> Result<(Option<i64>, &str), ()> {
    let Some(idx) = s.find(unit) else {
        return Ok((None, s));
    };
    let digits = &s[..idx];
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return Err(());
    }
    let value: i64 = digits.parse().map_err(|_| ())?;
    Ok((Some(value), &s[idx + unit.len_utf8()..]))
}

/// Like [`take_int`], but the component is an XSD decimal (used for
/// fractional seconds).
fn take_decimal(s: &str, unit: char) -> Result<(Option<Decimal>, &str), ()> {
    let Some(idx) = s.find(unit) else {
        return Ok((None, s));
    };
    let digits = &s[..idx];
    if digits.is_empty() {
        return Err(());
    }
    let value: Decimal = digits.parse().map_err(|_| ())?;
    Ok((Some(value), &s[idx + unit.len_utf8()..]))
}

fn invalid_lexical(datatype: Datatype, lexical: &str) -> OntologyError {
    OntologyError::invalid_lexical_form(datatype.to_string(), lexical)
}

/// A typed value: the XSD subset the RDFS-plus profile supports, plus `Ref`
/// for node-valued (object) properties.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Decimal(Decimal),
    Integer(i64),
    String(String),
    Boolean(bool),
    DateTime(DateTime<FixedOffset>),
    Duration(XsdDuration),
    Ref(Ref),
}

impl Value {
    /// Parses `lexical` as the XSD lexical form of `datatype`.
    pub fn parse_typed(lexical: &str, datatype: Datatype) -> Result<Value, OntologyError> {
        match datatype {
            Datatype::Decimal => lexical
                .parse::<Decimal>()
                .map(Value::Decimal)
                .map_err(|_| invalid_lexical(datatype, lexical)),
            Datatype::Integer => lexical
                .parse::<i64>()
                .map(Value::Integer)
                .map_err(|_| invalid_lexical(datatype, lexical)),
            Datatype::String => Ok(Value::String(lexical.to_owned())),
            Datatype::Boolean => match lexical {
                "true" | "1" => Ok(Value::Boolean(true)),
                "false" | "0" => Ok(Value::Boolean(false)),
                _ => Err(invalid_lexical(datatype, lexical)),
            },
            Datatype::DateTime => DateTime::parse_from_rfc3339(lexical)
                .map(Value::DateTime)
                .map_err(|_| invalid_lexical(datatype, lexical)),
            Datatype::Duration => XsdDuration::parse(lexical).map(Value::Duration),
        }
    }

    /// Serializes to the XSD lexical form. Returns `None` for `Ref` — a node
    /// reference isn't an XSD literal; resolve its IRI text via
    /// [`Interner::resolve`] instead.
    pub fn lexical_form(&self) -> Option<String> {
        match self {
            Value::Decimal(d) => Some(d.to_string()),
            Value::Integer(i) => Some(i.to_string()),
            Value::String(s) => Some(s.clone()),
            Value::Boolean(b) => Some(if *b { "true" } else { "false" }.to_owned()),
            Value::DateTime(dt) => Some(dt.to_rfc3339()),
            Value::Duration(d) => Some(d.to_lexical()),
            Value::Ref(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // -- Interner --------------------------------------------------------

    #[test]
    fn intern_same_string_returns_same_token() {
        let mut interner = Interner::new();
        let a = interner.intern("https://example.org/a");
        let b = interner.intern("https://example.org/a");
        assert_eq!(a, b);
    }

    #[test]
    fn intern_distinct_strings_returns_distinct_tokens() {
        let mut interner = Interner::new();
        let a = interner.intern("https://example.org/a");
        let b = interner.intern("https://example.org/b");
        assert_ne!(a, b);
    }

    #[test]
    fn resolve_returns_the_interned_string() {
        let mut interner = Interner::new();
        let id = interner.intern("https://example.org/Patient");
        assert_eq!(interner.resolve(id), "https://example.org/Patient");
    }

    proptest! {
        #[test]
        fn intern_resolve_roundtrip(s in "https://[a-z]{1,10}\\.example/[a-zA-Z0-9_#-]{0,20}") {
            let mut interner = Interner::new();
            let id = interner.intern(&s);
            prop_assert_eq!(interner.resolve(id), s.as_str());
        }

        #[test]
        fn interning_twice_dedups(s in "https://[a-z]{1,10}\\.example/[a-zA-Z0-9_#-]{0,20}") {
            let mut interner = Interner::new();
            let a = interner.intern(&s);
            let b = interner.intern(&s);
            prop_assert_eq!(a, b);
        }
    }

    // -- Ref ---------------------------------------------------------------

    #[test]
    fn ref_wraps_and_returns_its_iri() {
        let mut interner = Interner::new();
        let id = interner.intern("https://example.org/p1");
        let r = Ref::new(id);
        assert_eq!(r.iri(), id);
    }

    // -- Value: decimal ------------------------------------------------------

    #[test]
    fn parses_valid_decimal() {
        let v = Value::parse_typed("123.45", Datatype::Decimal).expect("valid decimal");
        assert_eq!(v, Value::Decimal(rust_decimal::Decimal::new(12345, 2)));
    }

    #[test]
    fn rejects_invalid_decimal() {
        let err = Value::parse_typed("not-a-decimal", Datatype::Decimal)
            .expect_err("invalid decimal lexical form must be rejected");
        let text = err.domain_text();
        assert!(text.contains("xsd:decimal"), "{text}");
        assert!(text.contains("not-a-decimal"), "{text}");
    }

    // -- Value: integer ------------------------------------------------------

    #[test]
    fn parses_valid_integer() {
        let v = Value::parse_typed("-42", Datatype::Integer).expect("valid integer");
        assert_eq!(v, Value::Integer(-42));
    }

    #[test]
    fn rejects_invalid_integer() {
        assert!(Value::parse_typed("4.5", Datatype::Integer).is_err());
    }

    // -- Value: boolean ------------------------------------------------------

    #[test]
    fn parses_valid_booleans() {
        assert_eq!(
            Value::parse_typed("true", Datatype::Boolean).expect("valid"),
            Value::Boolean(true)
        );
        assert_eq!(
            Value::parse_typed("0", Datatype::Boolean).expect("valid"),
            Value::Boolean(false)
        );
    }

    #[test]
    fn rejects_invalid_boolean() {
        assert!(Value::parse_typed("yes", Datatype::Boolean).is_err());
    }

    // -- Value: string ---------------------------------------------------------

    #[test]
    fn string_lexical_form_is_identity() {
        let v = Value::parse_typed("hello world", Datatype::String).expect("valid");
        assert_eq!(v.lexical_form(), Some("hello world".to_owned()));
    }

    // -- Value: dateTime -------------------------------------------------------

    #[test]
    fn parses_valid_datetime() {
        let v = Value::parse_typed("2024-04-12T13:20:00-05:00", Datatype::DateTime)
            .expect("valid dateTime");
        assert!(matches!(v, Value::DateTime(_)));
    }

    #[test]
    fn rejects_invalid_datetime() {
        assert!(Value::parse_typed("not-a-date", Datatype::DateTime).is_err());
    }

    // -- Value: duration ---------------------------------------------------------

    #[test]
    fn parses_full_duration() {
        let v = Value::parse_typed("P1Y2M3DT4H5M6.5S", Datatype::Duration).expect("valid");
        let Value::Duration(d) = v else {
            panic!("expected Value::Duration")
        };
        assert!(!d.negative);
        assert_eq!(d.months, 14);
        assert_eq!(d.days, 3);
        assert_eq!(d.hours, 4);
        assert_eq!(d.minutes, 5);
        assert_eq!(d.seconds, rust_decimal::Decimal::new(65, 1));
    }

    #[test]
    fn parses_negative_zero_duration() {
        let v = Value::parse_typed("-PT0S", Datatype::Duration).expect("valid");
        let Value::Duration(d) = v else {
            panic!("expected Value::Duration")
        };
        assert!(d.negative);
        assert_eq!(d.months, 0);
        assert_eq!(d.days, 0);
        assert_eq!(d.hours, 0);
        assert_eq!(d.minutes, 0);
        assert!(d.seconds.is_zero());
    }

    #[test]
    fn rejects_malformed_durations() {
        for bad in ["", "P", "-P", "PT", "1Y2M3D", "garbage", "P1Y2M3DTX"] {
            assert!(
                Value::parse_typed(bad, Datatype::Duration).is_err(),
                "expected {bad:?} to be rejected"
            );
        }
    }

    // -- Round trips -------------------------------------------------------------

    fn arb_decimal() -> impl Strategy<Value = rust_decimal::Decimal> {
        (-1_000_000i64..=1_000_000i64, 0u32..=6u32)
            .prop_map(|(num, scale)| rust_decimal::Decimal::new(num, scale))
    }

    fn arb_nonneg_decimal() -> impl Strategy<Value = rust_decimal::Decimal> {
        (0i64..=1_000_000i64, 0u32..=6u32)
            .prop_map(|(num, scale)| rust_decimal::Decimal::new(num, scale))
    }

    fn arb_datetime() -> impl Strategy<Value = chrono::DateTime<chrono::FixedOffset>> {
        use chrono::{DateTime, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime};
        (
            1970i32..2100,
            1u32..=12,
            1u32..=28,
            0u32..24,
            0u32..60,
            0u32..60,
            0u32..1_000_000_000u32,
            // RFC3339 (and XSD dateTime) timezone designators carry only
            // minute precision (`+HH:MM`) — sub-minute offsets can't survive
            // a lexical round trip, so generate whole minutes only.
            -14 * 60i32..=14 * 60i32,
        )
            .prop_map(|(year, month, day, hour, min, sec, nano, offset_minutes)| {
                let date = NaiveDate::from_ymd_opt(year, month, day)
                    .expect("day <= 28 is valid for every month/year");
                let time = NaiveTime::from_hms_nano_opt(hour, min, sec, nano)
                    .expect("hour/min/sec/nano within valid ranges by construction");
                let naive = NaiveDateTime::new(date, time);
                let offset = FixedOffset::east_opt(offset_minutes * 60)
                    .expect("offset within +/-14:00 by construction");
                DateTime::from_naive_utc_and_offset(naive, offset)
            })
    }

    fn arb_duration() -> impl Strategy<Value = XsdDuration> {
        (
            any::<bool>(),
            0i64..=100_000,
            0i64..=100_000,
            0i64..=100_000,
            0i64..=100_000,
            arb_nonneg_decimal(),
        )
            .prop_map(
                |(negative, months, days, hours, minutes, seconds)| XsdDuration {
                    negative,
                    months,
                    days,
                    hours,
                    minutes,
                    seconds,
                },
            )
    }

    fn arb_value_with_datatype() -> impl Strategy<Value = (Datatype, Value)> {
        prop_oneof![
            arb_decimal().prop_map(|d| (Datatype::Decimal, Value::Decimal(d))),
            any::<i64>().prop_map(|i| (Datatype::Integer, Value::Integer(i))),
            ".*".prop_map(|s| (Datatype::String, Value::String(s))),
            any::<bool>().prop_map(|b| (Datatype::Boolean, Value::Boolean(b))),
            arb_datetime().prop_map(|dt| (Datatype::DateTime, Value::DateTime(dt))),
            arb_duration().prop_map(|d| (Datatype::Duration, Value::Duration(d))),
        ]
    }

    proptest! {
        #[test]
        fn value_lexical_round_trip((datatype, value) in arb_value_with_datatype()) {
            let lexical = value.lexical_form().expect("literal values always have a lexical form");
            let parsed = Value::parse_typed(&lexical, datatype).expect("our own lexical form must parse");
            prop_assert_eq!(parsed, value);
        }

        #[test]
        fn duration_lexical_round_trip(d in arb_duration()) {
            let lexical = d.to_lexical();
            let parsed = XsdDuration::parse(&lexical).expect("our own lexical form must parse");
            prop_assert_eq!(parsed, d);
        }
    }
}
