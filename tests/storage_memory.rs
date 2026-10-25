//! S7b peak-heap regression: checkout must verify and materialize blobs with
//! bounded streaming buffers instead of expanding the whole manifest in memory.
//! The N2 regression below holds the save path to the same bound: publishing
//! per material instead of buffering the whole new-material set.
//!
//! The review measured roughly 73 MB RSS for 64 aliases of one 1 MiB blob
//! while the request itself was only ~9 KiB. This target runs each test with
//! a counting global allocator so the peak is attributable per
//! integration-test process; verification of the written bytes happens only
//! after the measurement window closes.

use serde_json::{json, Map};
use sha2::{Digest, Sha256};
use std::{
    alloc::{GlobalAlloc, Layout, System},
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
};
use xdesign::storage::{RsOperationOutcome, RsSaveMaterial, RsSaveRequest, RsStore};

struct CountingAllocator;

static LIVE: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let pointer = System.alloc(layout);
        if !pointer.is_null() {
            let live = LIVE.fetch_add(layout.size(), Ordering::Relaxed) + layout.size();
            PEAK.fetch_max(live, Ordering::Relaxed);
        }
        pointer
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        System.dealloc(pointer, layout);
        LIVE.fetch_sub(layout.size(), Ordering::Relaxed);
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let replacement = System.realloc(pointer, layout, new_size);
        if !replacement.is_null() {
            LIVE.fetch_add(new_size, Ordering::Relaxed);
            LIVE.fetch_sub(layout.size(), Ordering::Relaxed);
            PEAK.fetch_max(LIVE.load(Ordering::Relaxed), Ordering::Relaxed);
        }
        replacement
    }
}

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn reset_peak() -> usize {
    let live = LIVE.load(Ordering::SeqCst);
    PEAK.store(live, Ordering::SeqCst);
    live
}

fn peak() -> usize {
    PEAK.load(Ordering::SeqCst)
}

const BLOB_BYTES: usize = 1024 * 1024;
const ALIAS_COUNT: usize = 48;

fn request(
    operation: &str,
    expected_revision: u64,
    materials: Vec<RsSaveMaterial>,
) -> RsSaveRequest {
    RsSaveRequest {
        schema_version: 1,
        operation_id: operation.into(),
        item_id: "memory-item".into(),
        kind: "artifact".into(),
        expected_revision,
        metadata: Map::new(),
        materials,
        result: json!({"operation": operation}),
    }
}

#[test]
fn checkout_peak_heap_stays_bounded_for_many_aliases() {
    let base = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("cache/implementation/storage-memory-tests")
        .join(uuid::Uuid::new_v4().to_string());
    let project = base.join("project");
    fs::create_dir_all(&project).unwrap();
    let root = base.join("store");
    let store = RsStore::init(&root).unwrap();

    // Deterministic ~1 MiB material, saved once through a source.
    let content: Vec<u8> = (0..BLOB_BYTES).map(|index| (index % 251) as u8).collect();
    let sha256 = digest(&content);
    fs::write(project.join("blob.bin"), &content).unwrap();
    let create = request(
        "memory-create",
        0,
        vec![RsSaveMaterial {
            uri: "blob.bin".into(),
            role: "artifact".into(),
            sha256: sha256.clone(),
            source: Some(PathBuf::from("blob.bin")),
        }],
    );
    let receipt = store.save(&project, &create).unwrap();
    assert_eq!(receipt.outcome, RsOperationOutcome::Committed);
    assert_eq!(receipt.revision, Some(1));

    // A later revision where 48 distinct URIs alias the same committed digest.
    let aliases: Vec<RsSaveMaterial> = (0..ALIAS_COUNT)
        .map(|index| RsSaveMaterial {
            uri: format!("alias-{index:02}.bin"),
            role: "artifact".into(),
            sha256: sha256.clone(),
            source: None,
        })
        .collect();
    let append = request("memory-append", 1, aliases);
    let receipt = store.save(&project, &append).unwrap();
    assert_eq!(receipt.outcome, RsOperationOutcome::Committed);
    assert_eq!(receipt.revision, Some(2));

    let destination = base.join("checkout");
    let baseline = reset_peak();
    let report = store
        .checkout("memory-item", Some(2), &destination)
        .unwrap();
    let measured_peak = peak();
    let growth = measured_peak.saturating_sub(baseline);

    // Reading the 48 files below allocates its own buffers; it must not be part
    // of the measurement window.
    assert_eq!(report.material_count, ALIAS_COUNT);
    assert_eq!(report.revision, 2);
    eprintln!(
        "checkout peak heap growth: {growth} bytes over baseline {baseline} ({ALIAS_COUNT} aliases of {BLOB_BYTES} bytes)"
    );
    assert!(
        growth < 16 * 1024 * 1024,
        "checkout grew peak heap by {growth} bytes (baseline {baseline}, peak {measured_peak}); \
         the baseline implementation held ~{ALIAS_COUNT} MiB of material buffers"
    );
    for index in 0..ALIAS_COUNT {
        let written = fs::read(destination.join(format!("alias-{index:02}.bin"))).unwrap();
        assert_eq!(written.len(), BLOB_BYTES, "alias {index} is truncated");
        assert_eq!(written, content, "alias {index} is not byte-correct");
    }
}

/// N2 peak-heap regression: saving many distinct new materials must publish
/// each one as it is read instead of accumulating every material's bytes
/// before publication. The pre-fix implementation buffered all new bytes in
/// one vector, so a 32 x 1 MiB save peaked above 32 MiB of heap.
#[test]
fn save_peak_heap_stays_bounded_for_many_new_materials() {
    let base = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("cache/implementation/storage-memory-tests")
        .join(uuid::Uuid::new_v4().to_string());
    let project = base.join("project");
    fs::create_dir_all(&project).unwrap();
    let root = base.join("store");
    let store = RsStore::init(&root).unwrap();

    const NEW_MATERIALS: usize = 32;
    let content = |index: usize| {
        (0..BLOB_BYTES)
            .map(|byte| ((index.wrapping_mul(37) + byte) % 251) as u8)
            .collect::<Vec<u8>>()
    };
    let materials: Vec<_> = (0..NEW_MATERIALS)
        .map(|index| {
            let bytes = content(index);
            let uri = format!("new-{index:02}.bin");
            fs::write(project.join(&uri), &bytes).unwrap();
            RsSaveMaterial {
                uri,
                role: "artifact".into(),
                sha256: digest(&bytes),
                source: Some(PathBuf::from(format!("new-{index:02}.bin"))),
            }
        })
        .collect();
    let create = request("memory-save-create", 0, materials);

    let baseline = reset_peak();
    let receipt = store.save(&project, &create).unwrap();
    let measured_peak = peak();
    let growth = measured_peak.saturating_sub(baseline);
    assert_eq!(receipt.outcome, RsOperationOutcome::Committed);
    assert_eq!(receipt.revision, Some(1));
    eprintln!(
        "save peak heap growth: {growth} bytes over baseline {baseline} \
         ({NEW_MATERIALS} new materials of {BLOB_BYTES} bytes)"
    );
    assert!(
        growth < 16 * 1024 * 1024,
        "save grew peak heap by {growth} bytes (baseline {baseline}, peak {measured_peak}); \
         the pre-fix implementation buffered roughly {NEW_MATERIALS} MiB of new material bytes"
    );

    // Byte correctness is verified outside the measurement window.
    for index in 0..NEW_MATERIALS {
        let read = store
            .read_material("memory-item", Some(1), &format!("new-{index:02}.bin"))
            .unwrap();
        assert_eq!(read, content(index), "material {index} is not byte-correct");
    }
}
