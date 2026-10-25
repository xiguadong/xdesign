mod blobs;
pub mod cli;
mod db;
mod ops;

use rusqlite::Connection;
use serde::{
    de::{self, DeserializeSeed},
    Deserialize, Deserializer, Serialize,
};
use serde_json::{Map, Number, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    path::{Path, PathBuf},
};

pub(crate) const SCHEMA_VERSION: u32 = 1;
pub(crate) const APPLICATION_ID: i64 = 0x58445331;
pub(crate) const MAX_JSON_BYTES: usize = 16 * 1024 * 1024;
pub(crate) const MAX_JSON_DEPTH: usize = 64;
pub(crate) const MAX_MATERIAL_BYTES: u64 = 256 * 1024 * 1024;
// Receipts and revision echoes re-emit request JSON inside command envelopes.
// Saves reserve this much nesting and byte capacity up front so an accepted
// request can never commit a receipt the CLI is then unable to return.
pub(crate) const RESPONSE_DEPTH_MARGIN: usize = 6;
pub(crate) const RESPONSE_BYTES_MARGIN: usize = 2048;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RsSaveRequest {
    pub schema_version: u32,
    pub operation_id: String,
    pub item_id: String,
    pub kind: String,
    pub expected_revision: u64,
    pub metadata: Map<String, Value>,
    pub materials: Vec<RsSaveMaterial>,
    pub result: Value,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RsSaveMaterial {
    pub uri: String,
    pub role: String,
    pub sha256: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<PathBuf>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RsMaterial {
    pub uri: String,
    pub role: String,
    pub sha256: String,
    pub byte_length: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RsOperationOutcome {
    Pending,
    Committed,
    Conflict,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RsOperationReceipt {
    pub operation_id: String,
    pub request_sha256: String,
    pub outcome: RsOperationOutcome,
    pub revision: Option<u64>,
    pub result: Value,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RsItemRevision {
    pub item_id: String,
    pub kind: String,
    pub revision: u64,
    pub metadata: Map<String, Value>,
    pub materials: Vec<RsMaterial>,
    pub operation_id: String,
    pub projection: Option<RsWishProjection>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RsItemSummary {
    pub item_id: String,
    pub kind: String,
    pub head_revision: u64,
    pub projection: Option<RsWishProjection>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RsWishStatus {
    Received,
    Active,
    Blocked,
    Delivered,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RsWishProjection {
    pub revision: u64,
    pub flow_id: String,
    pub registry_uri: String,
    pub run_uri: String,
    pub status: RsWishStatus,
    pub current_stage: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RsCheckoutReport {
    pub item_id: String,
    pub revision: u64,
    pub destination: PathBuf,
    pub material_count: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RsDoctorFindingKind {
    OrphanBlob,
    PendingOperation,
    MissingBlob,
    AlteredBlob,
    StagingArtifact,
    InvalidRecord,
    FormatDefect,
    IdentityMismatch,
    PermissionDefect,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RsDoctorFinding {
    pub kind: RsDoctorFindingKind,
    pub subject_uri: String,
    pub detail: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RsDoctorReport {
    pub verify_bytes: bool,
    pub findings: Vec<RsDoctorFinding>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RsStoreError {
    InvalidInput(String),
    UnsupportedFormat(String),
    NotFound(String),
    OperationMismatch {
        operation_id: String,
    },
    Integrity(String),
    UnsafePath(String),
    Io {
        operation_id: Option<String>,
        message: String,
    },
    Database {
        operation_id: Option<String>,
        message: String,
    },
    UnknownOutcome {
        operation_id: String,
    },
}

impl fmt::Display for RsStoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput(message)
            | Self::UnsupportedFormat(message)
            | Self::NotFound(message)
            | Self::Integrity(message)
            | Self::UnsafePath(message) => formatter.write_str(message),
            Self::OperationMismatch { .. } => formatter.write_str(
                "The operation ID is reserved for different request bytes.",
            ),
            Self::Io { message, .. } | Self::Database { message, .. } => {
                formatter.write_str(message)
            }
            Self::UnknownOutcome { operation_id } => write!(
                formatter,
                "The outcome of operation {operation_id} is unknown; query outcome or retry the exact request."
            ),
        }
    }
}

impl std::error::Error for RsStoreError {}

impl From<rusqlite::Error> for RsStoreError {
    fn from(error: rusqlite::Error) -> Self {
        database_error(error)
    }
}

impl From<std::io::Error> for RsStoreError {
    fn from(error: std::io::Error) -> Self {
        io_error(error)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct RsIdentity {
    pub root_device: u64,
    pub root_inode: u64,
    pub database_device: u64,
    pub database_inode: u64,
}

pub struct RsStore {
    pub(crate) root: PathBuf,
    pub(crate) conn: Connection,
    pub(crate) identity: RsIdentity,
    pub(crate) store_id: String,
}

#[derive(Serialize)]
pub(crate) struct RsStoreInfo {
    pub store_id: String,
    pub schema_version: u32,
    pub item_count: u64,
    pub revision_count: u64,
    pub blob_count: u64,
    pub pending_operation_count: u64,
}

impl RsStore {
    pub fn init(root: &Path) -> Result<Self, RsStoreError> {
        db::init(root)
    }

    pub fn open(root: &Path) -> Result<Self, RsStoreError> {
        db::open(root)
    }

    pub(crate) fn info(&self) -> Result<RsStoreInfo, RsStoreError> {
        db::info(self)
    }

    pub(crate) fn verify_live(&self) -> Result<(), RsStoreError> {
        db::verify_live(self)
    }
}

pub(crate) fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub(crate) fn valid_id(value: &str) -> bool {
    let mut bytes = value.bytes();
    value.len() <= 128
        && bytes
            .next()
            .is_some_and(|byte| byte.is_ascii_alphanumeric())
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')
}

pub(crate) fn valid_token(value: &str) -> bool {
    let mut bytes = value.bytes();
    value.len() <= 64
        && bytes.next().is_some_and(|byte| byte.is_ascii_lowercase())
        && bytes.all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

pub(crate) fn valid_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

pub(crate) fn valid_uri(value: &str) -> bool {
    if value.is_empty()
        || value.len() > 4096
        || value.starts_with('/')
        || value.ends_with('/')
        || value.contains(['\\', '?', '#'])
        || value.chars().any(char::is_control)
    {
        return false;
    }
    let mut segments = value.split('/');
    let Some(first) = segments.next() else {
        return false;
    };
    if first.contains(':') || !valid_segment(first) {
        return false;
    }
    segments.all(valid_segment)
}

fn valid_segment(segment: &str) -> bool {
    !segment.is_empty() && segment.len() <= 255 && segment != "." && segment != ".."
}

fn case_fold(path: &str) -> String {
    path.chars().flat_map(char::to_lowercase).collect()
}

/// Filesystem-aware manifest name rules. Every material URI and every
/// intermediate directory it implies is a directory entry; a manifest is
/// materializable only when no two entries collide, either because one name is
/// a path prefix of another (file and directory at once) or because distinct
/// names fold to the same case-insensitive entry.
pub(crate) fn manifest_entry_collision(uris: &BTreeSet<String>) -> Option<(String, String)> {
    for uri in uris {
        let prefix = format!("{uri}/");
        if let Some(child) = uris
            .range(prefix.clone()..)
            .next()
            .filter(|child| child.starts_with(&prefix))
        {
            return Some((uri.clone(), child.clone()));
        }
    }
    let mut folded_entries: BTreeMap<String, String> = BTreeMap::new();
    for uri in uris {
        let mut entry = uri.as_str();
        loop {
            let fold = case_fold(entry);
            match folded_entries.get(&fold) {
                Some(existing) if existing != entry => {
                    return Some((existing.clone(), entry.to_owned()));
                }
                Some(_) => {}
                None => {
                    folded_entries.insert(fold, entry.to_owned());
                }
            }
            match entry.rfind('/') {
                Some(index) => entry = &entry[..index],
                None => break,
            }
        }
    }
    None
}

pub(crate) fn valid_revision(value: u64) -> bool {
    value <= i64::MAX as u64
}

pub(crate) fn to_i64(value: u64, name: &str) -> Result<i64, RsStoreError> {
    i64::try_from(value)
        .map_err(|_| RsStoreError::InvalidInput(format!("{name} exceeds SQLite's integer range")))
}

pub(crate) fn from_i64(value: i64, name: &str) -> Result<u64, RsStoreError> {
    u64::try_from(value).map_err(|_| RsStoreError::Integrity(format!("stored {name} is negative")))
}

pub(crate) fn database_error(error: rusqlite::Error) -> RsStoreError {
    RsStoreError::Database {
        operation_id: None,
        message: error.to_string(),
    }
}

pub(crate) fn io_error(error: std::io::Error) -> RsStoreError {
    RsStoreError::Io {
        operation_id: None,
        message: error.to_string(),
    }
}

struct StrictValueSeed(usize);

impl<'de> de::DeserializeSeed<'de> for StrictValueSeed {
    type Value = Value;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct StrictVisitor(usize);

        impl<'de> de::Visitor<'de> for StrictVisitor {
            type Value = Value;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a strict JSON value")
            }

            fn visit_bool<E>(self, value: bool) -> Result<Value, E> {
                Ok(Value::Bool(value))
            }

            fn visit_i64<E>(self, value: i64) -> Result<Value, E> {
                Ok(Value::Number(value.into()))
            }

            fn visit_u64<E>(self, value: u64) -> Result<Value, E> {
                Ok(Value::Number(value.into()))
            }

            fn visit_f64<E>(self, value: f64) -> Result<Value, E>
            where
                E: de::Error,
            {
                Number::from_f64(value)
                    .map(Value::Number)
                    .ok_or_else(|| E::custom("non-finite JSON number"))
            }

            fn visit_str<E>(self, value: &str) -> Result<Value, E>
            where
                E: de::Error,
            {
                Ok(Value::String(value.to_owned()))
            }

            fn visit_string<E>(self, value: String) -> Result<Value, E> {
                Ok(Value::String(value))
            }

            fn visit_none<E>(self) -> Result<Value, E> {
                Ok(Value::Null)
            }

            fn visit_unit<E>(self) -> Result<Value, E> {
                Ok(Value::Null)
            }

            fn visit_seq<A>(self, mut sequence: A) -> Result<Value, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                if self.0 > MAX_JSON_DEPTH {
                    return Err(de::Error::custom("JSON nesting exceeds 64 levels"));
                }
                let mut values = Vec::new();
                while let Some(value) = sequence.next_element_seed(StrictValueSeed(self.0 + 1))? {
                    values.push(value);
                }
                Ok(Value::Array(values))
            }

            fn visit_map<A>(self, mut object: A) -> Result<Value, A::Error>
            where
                A: de::MapAccess<'de>,
            {
                if self.0 > MAX_JSON_DEPTH {
                    return Err(de::Error::custom("JSON nesting exceeds 64 levels"));
                }
                let mut values = Map::new();
                while let Some(key) = object.next_key::<String>()? {
                    if values.contains_key(&key) {
                        return Err(de::Error::custom(format!(
                            "duplicate JSON object member: {key}"
                        )));
                    }
                    let value = object.next_value_seed(StrictValueSeed(self.0 + 1))?;
                    values.insert(key, value);
                }
                Ok(Value::Object(values))
            }
        }

        deserializer.deserialize_any(StrictVisitor(self.0))
    }
}

pub(crate) fn strict_json(raw: &[u8]) -> Result<Value, String> {
    if raw.len() > MAX_JSON_BYTES {
        return Err("JSON document exceeds 16 MiB".into());
    }
    validate_numeric_tokens(raw)?;
    let mut deserializer = serde_json::Deserializer::from_slice(raw);
    let value = StrictValueSeed(1)
        .deserialize(&mut deserializer)
        .map_err(|error| error.to_string())?;
    deserializer.end().map_err(|error| error.to_string())?;
    Ok(value)
}

/// A decimal number reduced to its significant digits and a power-of-ten
/// exponent so two numeric tokens can be compared for mathematical equality
/// without floating point or arbitrary-precision arithmetic.
struct DecimalParts {
    negative: bool,
    digits: String,
    exponent: i32,
}

fn decimal_parts(token: &str) -> Option<DecimalParts> {
    let (negative, unsigned) = match token.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, token.strip_prefix('+').unwrap_or(token)),
    };
    let (mantissa, exponent) = match unsigned.split_once(['e', 'E']) {
        Some((mantissa, exponent)) => (mantissa, exponent.parse::<i32>().ok()?),
        None => (unsigned, 0),
    };
    let (integral, fractional) = match mantissa.split_once('.') {
        Some((integral, fractional)) => (integral, fractional),
        None => (mantissa, ""),
    };
    if integral.is_empty() && fractional.is_empty() || !integral.bytes().all(|b| b.is_ascii_digit())
        || !fractional.bytes().all(|b| b.is_ascii_digit())
    {
        return None;
    }
    let mut digits = format!("{integral}{fractional}");
    let mut exponent = exponent - fractional.len() as i32;
    let significant = digits.trim_start_matches('0');
    let leading = digits.len() - significant.len();
    digits.drain(..leading);
    while digits.ends_with('0') {
        digits.pop();
        exponent += 1;
    }
    if digits.is_empty() {
        digits.push('0');
    }
    Some(DecimalParts {
        negative,
        digits,
        exponent,
    })
}

fn decimal_equal(left: &DecimalParts, right: &DecimalParts) -> bool {
    (left.digits == "0" && right.digits == "0")
        || (left.negative == right.negative
            && left.digits == right.digits
            && left.exponent == right.exponent)
}

/// Numbers are stored as exact 64-bit integers or IEEE-754 doubles. A token
/// whose value survives neither path unchanged is rejected instead of being
/// silently rounded in metadata and receipts.
fn check_numeric_token(token: &str) -> Result<(), String> {
    let integer_form = token
        .strip_prefix('-')
        .unwrap_or(token)
        .bytes()
        .all(|byte| byte.is_ascii_digit());
    if integer_form {
        let fits = if token.starts_with('-') {
            token.parse::<i64>().is_ok()
        } else {
            token.parse::<u64>().is_ok()
        };
        if !fits {
            return Err(format!(
                "number {token} exceeds the 64-bit integer range and cannot be stored without losing precision"
            ));
        }
        return Ok(());
    }
    let Ok(value) = token.parse::<f64>() else {
        return Ok(());
    };
    if !value.is_finite() {
        return Err(format!(
            "number {token} is not a finite value the store can retain"
        ));
    }
    let Some(parsed) = decimal_parts(token) else {
        return Ok(());
    };
    let shortest = format!("{value}");
    let Some(round_trip) = decimal_parts(&shortest) else {
        return Ok(());
    };
    if !decimal_equal(&parsed, &round_trip) {
        return Err(format!(
            "number {token} exceeds float64 precision and would be stored as {shortest}"
        ));
    }
    Ok(())
}

fn validate_numeric_tokens(raw: &[u8]) -> Result<(), String> {
    let mut token = String::new();
    let mut in_string = false;
    let mut escaped = false;
    for &byte in raw {
        if in_string {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                in_string = false;
            }
            continue;
        }
        match byte {
            b'"' => in_string = true,
            b'-' | b'0'..=b'9' => token.push(byte as char),
            b'.' | b'e' | b'E' | b'+' if !token.is_empty() => token.push(byte as char),
            _ => {
                if !token.is_empty() {
                    check_numeric_token(&token)?;
                    token.clear();
                }
            }
        }
    }
    if !token.is_empty() {
        check_numeric_token(&token)?;
    }
    Ok(())
}

pub(crate) fn json_depth(value: &Value) -> usize {
    let mut maximum = 0;
    let mut pending = vec![(value, 0)];
    while let Some((value, parent_depth)) = pending.pop() {
        match value {
            Value::Array(values) => {
                let depth = parent_depth + 1;
                maximum = maximum.max(depth);
                pending.extend(values.iter().map(|value| (value, depth)));
            }
            Value::Object(values) => {
                let depth = parent_depth + 1;
                maximum = maximum.max(depth);
                pending.extend(values.values().map(|value| (value, depth)));
            }
            _ => maximum = maximum.max(parent_depth),
        }
    }
    maximum
}

pub(crate) fn compact_json<T: Serialize>(value: &T) -> Result<Vec<u8>, RsStoreError> {
    serde_json::to_vec(value)
        .map_err(|error| RsStoreError::InvalidInput(format!("cannot serialize JSON: {error}")))
}
