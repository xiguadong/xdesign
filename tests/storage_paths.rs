//! Filesystem-path and bounded-materialization acceptance tests for S5/S7a.
//!
//! These tests exercise the public store API and CLI only. The direct
//! `published_by_attempt` / `cleanup_operation_staging` cases live as unit
//! tests inside `src/storage/blobs.rs` because those primitives are
//! `pub(crate)`.

use serde_json::{json, Map};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
};
use xdesign::storage::{cli, RsSaveMaterial, RsSaveRequest, RsStore, RsStoreError};

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

struct Fixture {
    base: PathBuf,
    project: PathBuf,
    root: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let base = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("cache/implementation/storage-paths-tests")
            .join(uuid::Uuid::new_v4().to_string());
        let project = base.join("project");
        fs::create_dir_all(&project).unwrap();
        Self {
            root: base.join("store"),
            base,
            project,
        }
    }

    fn write(&self, uri: &str, bytes: &[u8]) -> PathBuf {
        let path = self.project.join(uri);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, bytes).unwrap();
        path
    }

    fn material(&self, uri: &str, bytes: &[u8]) -> RsSaveMaterial {
        self.write(uri, bytes);
        RsSaveMaterial {
            uri: uri.into(),
            role: "artifact".into(),
            sha256: digest(bytes),
            source: Some(PathBuf::from(uri)),
        }
    }

    fn request(
        &self,
        operation: &str,
        item: &str,
        expected_revision: u64,
        materials: Vec<RsSaveMaterial>,
    ) -> RsSaveRequest {
        RsSaveRequest {
            schema_version: 1,
            operation_id: operation.into(),
            item_id: item.into(),
            kind: "artifact".into(),
            expected_revision,
            metadata: json!({"operation": operation})
                .as_object()
                .cloned()
                .unwrap_or_else(Map::new),
            materials,
            result: json!({"operation": operation}),
        }
    }

    fn store(&self) -> RsStore {
        RsStore::init(&self.root).unwrap()
    }

    fn blob(&self, sha256: &str) -> PathBuf {
        self.root
            .join("blobs/sha256")
            .join(&sha256[..2])
            .join(&sha256[2..])
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.base);
    }
}

#[cfg(unix)]
fn mode(path: impl AsRef<Path>, mode: u32) {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(mode)).unwrap();
}

#[cfg(not(unix))]
fn mode(_path: impl AsRef<Path>, _mode: u32) {}

fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}

/// S5: an earlier symlink ancestor above an existing directory must be refused
/// by both `read --output` and `checkout --destination`, and neither may create
/// anything beneath the symlink target.
#[cfg(unix)]
#[test]
fn existing_parent_under_symlink_ancestor_is_refused() {
    use std::os::unix::fs::symlink;
    let fixture = Fixture::new();
    let store = fixture.store();
    let request = fixture.request(
        "ancestor-save",
        "ancestor-item",
        0,
        vec![fixture.material("through.txt", b"through an earlier symlink")],
    );
    store.save(&fixture.project, &request).unwrap();

    // alias -> actual, with actual/nested already existing. `alias/nested`
    // therefore stats as a real directory even though an earlier component of
    // the destination path is a symlink.
    let actual = fixture.base.join("actual");
    fs::create_dir_all(actual.join("nested")).unwrap();
    symlink(&actual, fixture.base.join("alias")).unwrap();

    let output = fixture.base.join("alias/nested/read.txt");
    let (response, code) = cli::dispatch(&strings(&[
        "read",
        "ancestor-item",
        "--uri",
        "through.txt",
        "--out",
        &output.to_string_lossy(),
        "--root",
        &fixture.root.to_string_lossy(),
    ]));
    assert_eq!(code, 5, "{response}");
    assert_eq!(response["error"]["code"], "unsafe_path");
    assert!(!actual.join("nested/read.txt").exists());

    let destination = fixture.base.join("alias/nested/checkout");
    assert!(matches!(
        store.checkout("ancestor-item", None, &destination),
        Err(RsStoreError::UnsafePath(_))
    ));
    assert!(!actual.join("nested/checkout").exists());
    // Nothing at all was created through the alias.
    assert_eq!(fs::read_dir(actual.join("nested")).unwrap().count(), 0);

    // Plain missing destination parents keep reporting NotFound, not
    // UnsafePath, so the stricter ancestor walk does not change that contract.
    let missing_output = fixture.base.join("no-such-parent/read.txt");
    let (response, code) = cli::dispatch(&strings(&[
        "read",
        "ancestor-item",
        "--uri",
        "through.txt",
        "--out",
        &missing_output.to_string_lossy(),
        "--root",
        &fixture.root.to_string_lossy(),
    ]));
    assert_eq!(code, 3, "{response}");
    assert_eq!(response["error"]["code"], "not_found");
    assert!(matches!(
        store.checkout(
            "ancestor-item",
            None,
            &fixture.base.join("no-such-parent/checkout"),
        ),
        Err(RsStoreError::NotFound(_))
    ));
}

/// S7a: checkout streams a deduplicated manifest byte-exactly. The payload is
/// larger than the 64 KiB copy buffer, and every URI shares one published blob.
#[test]
fn checkout_streams_deduplicated_manifest_byte_exactly() {
    let fixture = Fixture::new();
    let store = fixture.store();
    let payload: Vec<u8> = (0..150_000usize).map(|index| (index % 251) as u8).collect();
    let request = fixture.request(
        "stream-save",
        "stream-item",
        0,
        vec![
            fixture.material("tree/a.bin", &payload),
            fixture.material("tree/b.bin", &payload),
            fixture.material("deep/dir/c.bin", &payload),
            fixture.material("top.bin", &payload),
        ],
    );
    store.save(&fixture.project, &request).unwrap();
    assert!(fixture.blob(&digest(&payload)).exists());

    let destination = fixture.base.join("checkout");
    let report = store.checkout("stream-item", None, &destination).unwrap();
    assert_eq!(report.material_count, 4);
    assert_eq!(report.destination, fs::canonicalize(&destination).unwrap());
    for uri in ["tree/a.bin", "tree/b.bin", "deep/dir/c.bin", "top.bin"] {
        let entry = destination.join(uri);
        let metadata = fs::metadata(&entry).unwrap();
        assert!(metadata.is_file());
        assert_eq!(metadata.len(), payload.len() as u64);
        assert_eq!(fs::read(&entry).unwrap(), payload, "uri {uri}");
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let entry = destination.join("tree/a.bin");
        assert_eq!(
            entry.metadata().unwrap().permissions().mode() & 0o777,
            0o600
        );
    }
    // The destination is published whole and the store stays consistent.
    assert!(fs::read_dir(&destination).unwrap().count() > 0);
    assert!(store.doctor(true).unwrap().findings.is_empty());
}

/// S7a: a checkout or read that streams altered or missing published bytes
/// fails with an integrity refusal and leaves no partial destination.
#[test]
fn checkout_and_read_reject_altered_and_missing_blobs() {
    let fixture = Fixture::new();
    let store = fixture.store();
    let first = b"first published blob";
    let second = b"second published blob";
    let request = fixture.request(
        "integrity-save",
        "integrity-item",
        0,
        vec![
            fixture.material("tree/a.txt", first),
            fixture.material("tree/b.txt", second),
        ],
    );
    store.save(&fixture.project, &request).unwrap();

    // Alter the first blob in place with the same length.
    let altered = fixture.blob(&digest(first));
    mode(&altered, 0o600);
    fs::write(&altered, vec![b'z'; first.len()]).unwrap();
    mode(&altered, 0o400);
    let altered_destination = fixture.base.join("altered-checkout");
    assert!(matches!(
        store.checkout("integrity-item", None, &altered_destination),
        Err(RsStoreError::Integrity(_))
    ));
    assert!(!altered_destination.exists());
    assert!(matches!(
        store.read_material("integrity-item", None, "tree/a.txt"),
        Err(RsStoreError::Integrity(_))
    ));

    // Restore the first blob, then remove the second one.
    mode(&altered, 0o600);
    fs::write(&altered, first).unwrap();
    mode(&altered, 0o400);
    let missing = fixture.blob(&digest(second));
    fs::remove_file(&missing).unwrap();
    let missing_destination = fixture.base.join("missing-checkout");
    assert!(matches!(
        store.checkout("integrity-item", None, &missing_destination),
        Err(RsStoreError::Integrity(_))
    ));
    assert!(!missing_destination.exists());
    assert!(matches!(
        store.read_material("integrity-item", None, "tree/b.txt"),
        Err(RsStoreError::Integrity(_))
    ));
}
