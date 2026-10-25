use super::{
    blobs::{
        canonical_existing, canonical_for_creation, create_store_dir, fsync_dir, identity,
        read_store_file, store_dir, store_file, sync_durable,
    },
    compact_json, database_error, from_i64, io_error, strict_json, valid_id, RsIdentity, RsStore,
    RsStoreError, RsStoreInfo, APPLICATION_ID, SCHEMA_VERSION,
};
use rusqlite::{Connection, OpenFlags, OptionalExtension};
use serde::Deserialize;
use serde_json::json;
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
    time::Duration,
};

#[cfg(unix)]
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

const FORMAT: &str = "xdesign.store";
const REQUIRED_TABLES: [&str; 7] = [
    "store_identity",
    "items",
    "revisions",
    "materials",
    "blobs",
    "operations",
    "pending_blobs",
];
const ROOT_ENTRIES: [&str; 7] = [
    "format.json",
    "store.db",
    "store.db-wal",
    "store.db-shm",
    "blobs",
    "staging",
    "locks",
];
// macOS Finder writes .DS_Store into directories it browses. Entries in this
// closed allowlist are inert to every store semantic, are never read or
// written by the store, and keep a browsed store openable; any other name
// remains foreign content refused at open.
const IGNORED_ROOT_ENTRIES: [&str; 1] = [".DS_Store"];

#[allow(dead_code)]
#[derive(Debug)]
pub(crate) struct FormatRecord {
    pub format: String,
    pub schema_version: u32,
    pub store_id: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FormatDocument {
    format: String,
    schema_version: u32,
    store_id: String,
}

pub(crate) struct Transaction<'a> {
    pub conn: &'a Connection,
    active: bool,
}

impl<'a> Transaction<'a> {
    pub fn begin(conn: &'a Connection, immediate: bool) -> Result<Self, RsStoreError> {
        conn.execute_batch(if immediate {
            "BEGIN IMMEDIATE"
        } else {
            "BEGIN"
        })
        .map_err(database_error)?;
        Ok(Self { conn, active: true })
    }

    pub fn commit(mut self) -> Result<(), RsStoreError> {
        self.conn.execute_batch("COMMIT").map_err(database_error)?;
        self.active = false;
        Ok(())
    }

    pub fn rollback(mut self) -> Result<(), RsStoreError> {
        if self.active {
            self.conn
                .execute_batch("ROLLBACK")
                .map_err(database_error)?;
            self.active = false;
        }
        Ok(())
    }
}

impl Drop for Transaction<'_> {
    fn drop(&mut self) {
        if self.active {
            let _ = self.conn.execute_batch("ROLLBACK");
        }
    }
}

const SCHEMA: &str = r#"
CREATE TABLE store_identity (
    singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
    store_id TEXT NOT NULL,
    root_device TEXT NOT NULL,
    root_inode TEXT NOT NULL,
    database_device TEXT NOT NULL,
    database_inode TEXT NOT NULL
);
CREATE TABLE operations (
    operation_id TEXT PRIMARY KEY,
    request_bytes BLOB NOT NULL,
    request_sha256 TEXT NOT NULL,
    outcome TEXT NOT NULL CHECK (outcome IN ('pending', 'committed', 'conflict')),
    item_id TEXT NOT NULL,
    expected_revision INTEGER NOT NULL CHECK (expected_revision >= 0),
    revision INTEGER CHECK (revision > 0),
    result_json TEXT NOT NULL,
    CHECK ((outcome = 'committed' AND revision IS NOT NULL)
        OR (outcome IN ('pending', 'conflict') AND revision IS NULL))
);
CREATE TABLE blobs (
    sha256 TEXT PRIMARY KEY,
    byte_length INTEGER NOT NULL CHECK (byte_length >= 0 AND byte_length <= 268435456)
);
CREATE TABLE items (
    item_id TEXT PRIMARY KEY,
    kind TEXT NOT NULL,
    head_revision INTEGER NOT NULL CHECK (head_revision >= 0)
);
CREATE TABLE revisions (
    item_id TEXT NOT NULL,
    revision INTEGER NOT NULL CHECK (revision > 0),
    metadata_json TEXT NOT NULL,
    operation_id TEXT NOT NULL,
    PRIMARY KEY (item_id, revision),
    FOREIGN KEY (item_id) REFERENCES items(item_id),
    FOREIGN KEY (operation_id) REFERENCES operations(operation_id)
);
CREATE TABLE materials (
    item_id TEXT NOT NULL,
    revision INTEGER NOT NULL,
    uri TEXT NOT NULL,
    role TEXT NOT NULL,
    sha256 TEXT NOT NULL,
    byte_length INTEGER NOT NULL CHECK (byte_length >= 0 AND byte_length <= 268435456),
    PRIMARY KEY (item_id, revision, uri),
    FOREIGN KEY (item_id, revision) REFERENCES revisions(item_id, revision),
    FOREIGN KEY (sha256) REFERENCES blobs(sha256)
);
CREATE TABLE pending_blobs (
    operation_id TEXT NOT NULL,
    sha256 TEXT NOT NULL,
    PRIMARY KEY (operation_id, sha256),
    FOREIGN KEY (operation_id) REFERENCES operations(operation_id)
);
CREATE INDEX materials_by_blob ON materials(sha256);
CREATE INDEX revisions_by_operation ON revisions(operation_id);

CREATE TRIGGER store_identity_no_update BEFORE UPDATE ON store_identity
BEGIN SELECT RAISE(ABORT, 'store identity is immutable'); END;
CREATE TRIGGER store_identity_no_delete BEFORE DELETE ON store_identity
BEGIN SELECT RAISE(ABORT, 'store identity is immutable'); END;
CREATE TRIGGER item_insert_at_zero BEFORE INSERT ON items
WHEN NEW.head_revision <> 0
BEGIN SELECT RAISE(ABORT, 'new item head must be zero'); END;
CREATE TRIGGER item_kind_immutable BEFORE UPDATE OF kind ON items
WHEN NEW.kind <> OLD.kind
BEGIN SELECT RAISE(ABORT, 'item kind is immutable'); END;
CREATE TRIGGER item_id_immutable BEFORE UPDATE OF item_id ON items
WHEN NEW.item_id <> OLD.item_id
BEGIN SELECT RAISE(ABORT, 'item identity is immutable'); END;
CREATE TRIGGER item_head_append_only BEFORE UPDATE OF head_revision ON items
WHEN NEW.head_revision <> OLD.head_revision + 1
 OR NOT EXISTS (SELECT 1 FROM revisions r
    WHERE r.item_id = NEW.item_id AND r.revision = NEW.head_revision)
BEGIN SELECT RAISE(ABORT, 'item head must advance to an appended revision'); END;
CREATE TRIGGER item_no_delete BEFORE DELETE ON items
BEGIN SELECT RAISE(ABORT, 'items cannot be deleted'); END;
CREATE TRIGGER revision_contiguous BEFORE INSERT ON revisions
WHEN NEW.revision <> COALESCE((SELECT MAX(revision) FROM revisions
    WHERE item_id = NEW.item_id), 0) + 1
BEGIN SELECT RAISE(ABORT, 'revisions must be contiguous'); END;
CREATE TRIGGER revision_no_update BEFORE UPDATE ON revisions
BEGIN SELECT RAISE(ABORT, 'revisions are immutable'); END;
CREATE TRIGGER revision_no_delete BEFORE DELETE ON revisions
BEGIN SELECT RAISE(ABORT, 'revisions are immutable'); END;
CREATE TRIGGER material_no_update BEFORE UPDATE ON materials
BEGIN SELECT RAISE(ABORT, 'materials are immutable'); END;
CREATE TRIGGER material_no_delete BEFORE DELETE ON materials
BEGIN SELECT RAISE(ABORT, 'materials are immutable'); END;
CREATE TRIGGER blob_no_update BEFORE UPDATE ON blobs
BEGIN SELECT RAISE(ABORT, 'blobs are immutable'); END;
CREATE TRIGGER blob_no_delete BEFORE DELETE ON blobs
BEGIN SELECT RAISE(ABORT, 'blobs are immutable'); END;
CREATE TRIGGER operation_guard_update BEFORE UPDATE ON operations
WHEN OLD.outcome <> 'pending' OR NEW.outcome = 'pending'
 OR NEW.operation_id <> OLD.operation_id
 OR NEW.request_bytes <> OLD.request_bytes
 OR NEW.request_sha256 <> OLD.request_sha256
 OR NEW.item_id <> OLD.item_id
 OR NEW.expected_revision <> OLD.expected_revision
 OR NEW.result_json <> OLD.result_json
BEGIN SELECT RAISE(ABORT, 'only a pending receipt outcome can change'); END;
CREATE TRIGGER operation_no_delete BEFORE DELETE ON operations
BEGIN SELECT RAISE(ABORT, 'operations cannot be deleted'); END;
CREATE TRIGGER pending_blob_insert_only_pending BEFORE INSERT ON pending_blobs
WHEN NOT EXISTS (SELECT 1 FROM operations o
    WHERE o.operation_id = NEW.operation_id AND o.outcome = 'pending')
BEGIN SELECT RAISE(ABORT, 'pending blobs require a pending operation'); END;
CREATE TRIGGER pending_blob_no_update BEFORE UPDATE ON pending_blobs
BEGIN SELECT RAISE(ABORT, 'pending blob reservations are immutable'); END;
"#;

pub(crate) fn init(root: &Path) -> Result<RsStore, RsStoreError> {
    match fs::symlink_metadata(root) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() {
                return Err(RsStoreError::UnsafePath(format!(
                    "store root is a symbolic link: {}",
                    root.display()
                )));
            }
            if !metadata.is_dir() {
                return Err(RsStoreError::UnsupportedFormat(
                    "store root is not a directory".into(),
                ));
            }
            // Only entries outside the closed OS-generated allowlist make the
            // root nonempty for creation purposes; a browsed empty directory
            // stays a safe empty root.
            let foreign = fs::read_dir(root)
                .map_err(io_error)?
                .filter_map(|entry| entry.ok())
                .any(|entry| {
                    entry
                        .file_name()
                        .to_str()
                        .is_some_and(|name| !IGNORED_ROOT_ENTRIES.contains(&name))
                });
            if foreign {
                return open(root);
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(io_error(error)),
    }

    let root = match fs::symlink_metadata(root) {
        Ok(_) => canonical_existing(root)?,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => canonical_for_creation(root)?,
        Err(error) => return Err(io_error(error)),
    };
    create_store_dir(&root)?;
    create_store_dir(&root.join("blobs"))?;
    create_store_dir(&root.join("blobs/sha256"))?;
    create_store_dir(&root.join("staging"))?;
    create_store_dir(&root.join("locks"))?;
    create_store_dir(&root.join("locks/items"))?;

    let store_id = uuid::Uuid::new_v4().to_string();
    let marker = compact_json(&json!({
        "format": FORMAT,
        "schema_version": SCHEMA_VERSION,
        "store_id": store_id,
    }))?;
    let marker_path = root.join("format.json");
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    options.mode(0o600);
    let mut file = options.open(&marker_path).map_err(io_error)?;
    #[cfg(unix)]
    fs::set_permissions(&marker_path, fs::Permissions::from_mode(0o600)).map_err(io_error)?;
    file.write_all(&marker).map_err(io_error)?;
    sync_durable(&file).map_err(io_error)?;
    drop(file);
    fsync_dir(&root)?;

    let database_path = root.join("store.db");
    let conn = Connection::open_with_flags(
        &database_path,
        OpenFlags::SQLITE_OPEN_READ_WRITE
            | OpenFlags::SQLITE_OPEN_CREATE
            | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(database_error)?;
    #[cfg(unix)]
    fs::set_permissions(&database_path, fs::Permissions::from_mode(0o600)).map_err(io_error)?;
    configure_writer(&conn)?;
    conn.execute_batch(
        "PRAGMA journal_mode = WAL; PRAGMA application_id = 0x58445331; PRAGMA user_version = 1;",
    )
    .map_err(database_error)?;
    conn.execute_batch(SCHEMA).map_err(database_error)?;
    let root_metadata = store_dir(&root)?;
    let database_metadata = store_file(&database_path, 0o600)?;
    let (root_device, root_inode) = identity(&root_metadata);
    let (database_device, database_inode) = identity(&database_metadata);
    conn.execute(
        "INSERT INTO store_identity
         (singleton, store_id, root_device, root_inode, database_device, database_inode)
         VALUES (1, ?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![
            store_id,
            root_device.to_string(),
            root_inode.to_string(),
            database_device.to_string(),
            database_inode.to_string(),
        ],
    )
    .map_err(database_error)?;
    fsync_dir(&root)?;
    for sidecar in ["store.db-wal", "store.db-shm"] {
        let path = root.join(sidecar);
        if path.exists() {
            #[cfg(unix)]
            fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).map_err(io_error)?;
        }
    }
    drop(conn);
    open(&root)
}

fn configure_writer(conn: &Connection) -> Result<(), RsStoreError> {
    let timeout = std::env::var("XDESIGN_STORE_BUSY_TIMEOUT_MS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .map_or(5_000, |millis| millis.clamp(1, 600_000));
    conn.busy_timeout(Duration::from_millis(timeout))
        .map_err(database_error)?;
    // fullfsync is a macOS-only pragma; elsewhere SQLite ignores the request.
    conn.execute_batch(
        "PRAGMA foreign_keys = ON; PRAGMA synchronous = FULL; PRAGMA fullfsync = ON;",
    )
    .map_err(database_error)
}

pub(crate) fn open(root: &Path) -> Result<RsStore, RsStoreError> {
    let root = canonical_existing(root)?;
    store_dir(&root)?;
    check_root_entries(&root)?;
    let format = load_format(&root)?;
    for directory in ["blobs", "blobs/sha256", "staging", "locks", "locks/items"] {
        let path = root.join(directory);
        store_dir(&path).map_err(|error| match &error {
            RsStoreError::Io { .. } if !path.exists() => RsStoreError::UnsupportedFormat(format!(
                "store is missing required directory: {directory}"
            )),
            _ => error,
        })?;
    }
    let database_path = root.join("store.db");
    store_file(&database_path, 0o600).map_err(|error| match &error {
        RsStoreError::Io { .. } if !database_path.exists() => {
            RsStoreError::UnsupportedFormat("store database is missing".into())
        }
        _ => error,
    })?;
    check_sqlite_header(&database_path)?;
    let conn = Connection::open_with_flags(
        &database_path,
        OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(database_error)?;
    configure_writer(&conn)?;
    let application_id: i64 = conn
        .query_row("PRAGMA application_id", [], |row| row.get(0))
        .map_err(|_| RsStoreError::UnsupportedFormat("cannot read SQLite application ID".into()))?;
    let user_version: i64 = conn
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .map_err(|_| RsStoreError::UnsupportedFormat("cannot read SQLite user version".into()))?;
    if application_id != APPLICATION_ID || user_version != 1 {
        return Err(RsStoreError::UnsupportedFormat(format!(
            "unsupported SQLite identity: application_id={application_id}, user_version={user_version}"
        )));
    }
    let journal_mode: String = conn
        .query_row("PRAGMA journal_mode", [], |row| row.get(0))
        .map_err(database_error)?;
    if !journal_mode.eq_ignore_ascii_case("wal") {
        return Err(RsStoreError::UnsupportedFormat(format!(
            "store database is not in WAL mode: {journal_mode}"
        )));
    }
    require_tables(&conn)?;
    let (recorded_id, recorded_identity) = read_identity(&conn)?;
    if recorded_id != format.store_id {
        return Err(RsStoreError::Integrity(
            "format.json and SQLite store identities differ".into(),
        ));
    }
    let root_metadata = store_dir(&root)?;
    let database_metadata = store_file(&database_path, 0o600)?;
    let (root_device, root_inode) = identity(&root_metadata);
    let (database_device, database_inode) = identity(&database_metadata);
    let live = RsIdentity {
        root_device,
        root_inode,
        database_device,
        database_inode,
    };
    if live != recorded_identity {
        return Err(RsStoreError::Integrity(
            "live root/database identity differs from the recorded identity".into(),
        ));
    }
    let store = RsStore {
        root,
        conn,
        identity: live,
        store_id: recorded_id,
    };
    store.verify_live()?;
    Ok(store)
}

pub(crate) fn read_only_connection(path: &Path) -> Result<Connection, RsStoreError> {
    let conn = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(database_error)?;
    conn.busy_timeout(Duration::from_secs(5))
        .map_err(database_error)?;
    conn.execute_batch("PRAGMA query_only = ON; PRAGMA foreign_keys = ON;")
        .map_err(database_error)?;
    Ok(conn)
}

fn check_sqlite_header(path: &Path) -> Result<(), RsStoreError> {
    use std::io::Read;
    let mut file = fs::File::open(path).map_err(io_error)?;
    let mut header = [0_u8; 16];
    file.read_exact(&mut header)
        .map_err(|_| RsStoreError::UnsupportedFormat("truncated SQLite database header".into()))?;
    if &header != b"SQLite format 3\0" {
        return Err(RsStoreError::UnsupportedFormat(
            "store.db is not a SQLite 3 database".into(),
        ));
    }
    Ok(())
}

fn check_root_entries(root: &Path) -> Result<(), RsStoreError> {
    for entry in fs::read_dir(root).map_err(io_error)? {
        let entry = entry.map_err(io_error)?;
        let name = entry.file_name();
        let name = name.to_str().ok_or_else(|| {
            RsStoreError::UnsupportedFormat("store root contains a non-UTF-8 entry".into())
        })?;
        if !ROOT_ENTRIES.contains(&name) && !IGNORED_ROOT_ENTRIES.contains(&name) {
            return Err(RsStoreError::UnsupportedFormat(format!(
                "store root contains foreign content: {name}"
            )));
        }
    }
    Ok(())
}

pub(crate) fn load_format(root: &Path) -> Result<FormatRecord, RsStoreError> {
    let path = root.join("format.json");
    let raw = read_store_file(&path, 0o600).map_err(|error| match error {
        RsStoreError::Io { .. } if !path.exists() => {
            RsStoreError::UnsupportedFormat("store format marker is missing".into())
        }
        other => other,
    })?;
    let value = strict_json(&raw).map_err(|error| {
        RsStoreError::UnsupportedFormat(format!("invalid format.json: {error}"))
    })?;
    let document: FormatDocument = serde_json::from_value(value).map_err(|error| {
        RsStoreError::UnsupportedFormat(format!("invalid format.json schema: {error}"))
    })?;
    if document.format != FORMAT || document.schema_version != SCHEMA_VERSION {
        return Err(RsStoreError::UnsupportedFormat(
            "unsupported store format or schema version".into(),
        ));
    }
    if !valid_id(&document.store_id) {
        return Err(RsStoreError::UnsupportedFormat(
            "format.json contains an invalid store_id".into(),
        ));
    }
    Ok(FormatRecord {
        format: document.format,
        schema_version: document.schema_version,
        store_id: document.store_id,
    })
}

pub(crate) fn require_tables(conn: &Connection) -> Result<(), RsStoreError> {
    for table in REQUIRED_TABLES {
        let present: Option<i64> = conn
            .query_row(
                "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1",
                [table],
                |row| row.get(0),
            )
            .optional()
            .map_err(database_error)?;
        if present.is_none() {
            return Err(RsStoreError::UnsupportedFormat(format!(
                "store database is missing required table: {table}"
            )));
        }
    }
    Ok(())
}

pub(crate) fn read_identity(conn: &Connection) -> Result<(String, RsIdentity), RsStoreError> {
    let row = conn
        .query_row(
            "SELECT store_id, root_device, root_inode, database_device, database_inode
             FROM store_identity WHERE singleton = 1",
            [],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                ))
            },
        )
        .optional()
        .map_err(database_error)?
        .ok_or_else(|| RsStoreError::UnsupportedFormat("store identity row is missing".into()))?;
    if !valid_id(&row.0) {
        return Err(RsStoreError::Integrity(
            "store identity contains an invalid store_id".into(),
        ));
    }
    let parse = |value: &str, name: &str| {
        value.parse::<u64>().map_err(|_| {
            RsStoreError::Integrity(format!("stored {name} is not a lossless unsigned integer"))
        })
    };
    Ok((
        row.0,
        RsIdentity {
            root_device: parse(&row.1, "root device")?,
            root_inode: parse(&row.2, "root inode")?,
            database_device: parse(&row.3, "database device")?,
            database_inode: parse(&row.4, "database inode")?,
        },
    ))
}

pub(crate) fn verify_live(store: &RsStore) -> Result<(), RsStoreError> {
    if !store.root.is_dir() {
        return Err(RsStoreError::Integrity(
            "store root is missing or no longer a directory".into(),
        ));
    }
    check_root_entries(&store.root)
        .map_err(|error| RsStoreError::Integrity(format!("store root changed: {error}")))?;
    let root_metadata = store_dir(&store.root)?;
    let database_path = store.root.join("store.db");
    if !database_path.is_file() {
        return Err(RsStoreError::Integrity(
            "store database is missing or no longer regular".into(),
        ));
    }
    let database_metadata = store_file(&database_path, 0o600)?;
    let (root_device, root_inode) = identity(&root_metadata);
    let (database_device, database_inode) = identity(&database_metadata);
    let live = RsIdentity {
        root_device,
        root_inode,
        database_device,
        database_inode,
    };
    if live != store.identity {
        return Err(RsStoreError::Integrity(
            "store root or database was replaced".into(),
        ));
    }
    let format = load_format(&store.root)
        .map_err(|error| RsStoreError::Integrity(format!("invalid live store format: {error}")))?;
    if format.store_id != store.store_id {
        return Err(RsStoreError::Integrity(
            "format.json store_id changed".into(),
        ));
    }
    for directory in ["blobs", "blobs/sha256", "staging", "locks", "locks/items"] {
        store_dir(&store.root.join(directory))?;
    }
    for sidecar in ["store.db-wal", "store.db-shm"] {
        let path = store.root.join(sidecar);
        match fs::symlink_metadata(&path) {
            Ok(_) => {
                store_file(&path, 0o600)?;
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(io_error(error)),
        }
    }
    let application_id: i64 = store
        .conn
        .query_row("PRAGMA application_id", [], |row| row.get(0))
        .map_err(database_error)?;
    let user_version: i64 = store
        .conn
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .map_err(database_error)?;
    if application_id != APPLICATION_ID || user_version != 1 {
        return Err(RsStoreError::Integrity(
            "SQLite application or schema version changed".into(),
        ));
    }
    let (recorded_id, recorded_identity) = read_identity(&store.conn)?;
    if recorded_id != store.store_id || recorded_identity != store.identity {
        return Err(RsStoreError::Integrity(
            "recorded store identity changed".into(),
        ));
    }
    Ok(())
}

pub(crate) fn info(store: &RsStore) -> Result<RsStoreInfo, RsStoreError> {
    store.verify_live()?;
    let transaction = Transaction::begin(&store.conn, false)?;
    let count = |sql: &str| -> Result<u64, RsStoreError> {
        let value = transaction
            .conn
            .query_row(sql, [], |row| row.get::<_, i64>(0))
            .map_err(database_error)?;
        from_i64(value, "count")
    };
    let result = RsStoreInfo {
        store_id: store.store_id.clone(),
        schema_version: SCHEMA_VERSION,
        item_count: count("SELECT COUNT(*) FROM items")?,
        revision_count: count("SELECT COUNT(*) FROM revisions")?,
        blob_count: count("SELECT COUNT(*) FROM blobs")?,
        pending_operation_count: count(
            "SELECT COUNT(*) FROM operations WHERE outcome = 'pending'",
        )?,
    };
    transaction.commit()?;
    Ok(result)
}
