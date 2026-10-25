package XDesignStorage {
    @XDesignCommon::DiagramPresentation { label = "Operational storage"; }
    @XDesignCommon::DocSection { document = "docs/design/storage.md"; section = "D86"; }
    private import ScalarValues::*;
    private import XDesignCommon::*;
    private import StandardViewDefinitions::*;
    private import Views::*;

    doc /* Logical model of the implemented local operational store. Git owns
       engineering definitions and products. The store owns new wish and run facts,
       immutable revision manifests, material and evidence bytes, and operation
       receipts. Legacy Git wish traces remain authoritative for their existing runs;
       this package neither migrates them nor supersedes their records. Cache copies
       are regenerable. Store-backed runs and recovery evidence exist within the
       acceptance boundary recorded for wish-0011; excluded capabilities and final
       human acceptance remain separate. */

    enum def OperationOutcome {
        @XDesignCommon::DocSection { document = "docs/design/storage.md"; section = "D86"; }
        doc /* Durable state of one exact save operation. Pending is recoverable but
           is not committed authority. Committed and conflict are terminal outcomes. */
        enum pending;
        enum committed;
        enum conflict;
    }

    item def Material {
        @XDesignCommon::DiagramPresentation { label = "Material"; summary = "Logical URI bound to exact bytes"; }
        @XDesignCommon::DocSection { document = "docs/design/storage.md"; section = "D86"; }
        doc /* One complete-manifest entry. The SHA-256 and byte length identify an
           immutable blob; the URI and role identify its use in this revision. The
           same bytes may be shared by revisions and items without changing identity. */
        attribute uri : String;
        attribute role : String;
        attribute sha256 : String;
        attribute byteLength : Integer;
    }

    item def SaveMaterial {
        @XDesignCommon::DiagramPresentation { label = "Save material"; summary = "Digest declaration and optional source"; }
        @XDesignCommon::DocSection { document = "docs/design/storage.md"; section = "D86"; }
        doc /* One requested manifest entry. A source is a normalized project-relative
           path whose exact bytes must match sha256. An absent source carries a verified
           blob by digest: a committed blob row, or a digest this exact operation
           verifiably published on an earlier attempt while its outcome was pending; an
           unrelated published or orphan blob is never adopted. Every entry is restated
           in the new revision's complete manifest. */
        attribute uri : String;
        attribute role : String;
        attribute sha256 : String;
        attribute source : String [0..1];
    }

    item def SaveRequest {
        @XDesignCommon::DiagramPresentation { label = "Save request"; summary = "Exact operation, expected head and complete manifest"; }
        @XDesignCommon::DocSection { document = "docs/design/storage.md"; section = "D86"; }
        doc /* Version-one request for one logical save. expectedRevision zero creates
           the item; a positive value appends only at that current head. metadata is a
           JSON object and result is any strict JSON value. The operation ID and exact
           request bytes, not object-level similarity, control idempotent retry. */
        attribute schemaVersion : Integer = 1;
        attribute operationId : String;
        attribute itemId : String;
        attribute kind : String;
        attribute expectedRevision : Integer;
        attribute metadata : String;
        item materials : SaveMaterial [0..*];
        attribute result : String;
    }

    item def OperationReceipt {
        @XDesignCommon::DiagramPresentation { label = "Operation receipt"; summary = "Pending, committed or conflict outcome"; }
        @XDesignCommon::DocSection { document = "docs/design/storage.md"; section = "D86"; }
        doc /* Durable reconciliation record for one operation ID and exact request
           SHA-256. revision is present only for a committed outcome. result preserves
           the request's JSON result; a conflict result is a proposed result, not facts
           applied to the item. Pending never authorizes a reader to use proposed facts. */
        attribute operationId : String;
        attribute requestSha256 : String;
        attribute outcome : OperationOutcome;
        attribute revision : Integer [0..1];
        attribute result : String;
    }

    item def ItemRevision {
        @XDesignCommon::DiagramPresentation { label = "Item revision"; summary = "Immutable metadata and complete URI manifest"; }
        @XDesignCommon::DocSection { document = "docs/design/storage.md"; section = "D86"; }
        doc /* One immutable, contiguously numbered item revision. metadata is the typed
           JSON object supplied by the save. manifest restates every logical URI retained
           by this revision, including unchanged carried materials. operationId links the
           committed receipt; earlier revisions are never overwritten. */
        attribute revision : Integer;
        attribute metadata : String;
        item manifest : Material [0..*];
        attribute operationId : String;
    }

    item def StoreItem {
        @XDesignCommon::DiagramPresentation { label = "Store item"; summary = "Stable identity, kind and append-only revisions"; }
        @XDesignCommon::DocSection { document = "docs/design/storage.md"; section = "D86"; }
        doc /* One generic operational item. A wish uses its wish ID as itemId and kind
           wish; other kinds may use the same record machinery. kind is immutable and
           headRevision equals the greatest committed revision. No wish-status flag is
           stored on this item. */
        attribute itemId : String;
        attribute kind : String;
        attribute headRevision : Integer;
        item revisions : ItemRevision [0..*] ordered;
    }

    item def WishProjection {
        @XDesignCommon::DiagramPresentation { label = "Wish projection"; summary = "Derived from flow registration and current run"; }
        @XDesignCommon::DocSection { document = "docs/design/storage.md"; section = "D86"; }
        doc /* Output-only projection at one selected revision. registryUri and runUri
           resolve through that revision's manifest. flowId comes from metadata and the
           matching registry; status maps the run's pending, active, blocked or completed
           state to received, active, blocked or delivered. currentStage comes from the
           run's currentVisit and is absent when completed. This is never an independent
           mutable wish-status record. */
        ref item revision : ItemRevision;
        attribute flowId : String;
        attribute registryUri : String;
        attribute runUri : String;
        attribute status : WishStatus;
        attribute currentStage : String [0..1];
    }

    part def ProjectStore {
        @XDesignCommon::DiagramPresentation { label = "Project store"; summary = "SQLite records and SHA-256 materials"; }
        @XDesignCommon::DocSection { document = "docs/design/storage.md"; section = "D86"; }
        doc /* Local POSIX operational store with a versioned root, SQLite record
           authority, content-addressed immutable blobs, and operation receipts. The
           default root is .xdesign with XDESIGN_STORE_ROOT as an override. Root and
           database identities, owner-only permissions and path safety are checked;
           there is no service, authentication or encryption boundary. */
        attribute rootUri : String = ".xdesign";
        attribute schemaVersion : Integer = 1;
        item items : StoreItem [0..*];
        item operations : OperationReceipt [0..*];

        perform action save : SaveOperation;

        perform action outcome {
            doc /* Return the durable pending, committed or conflict receipt for an
               operation ID. An unknown ID is not an empty successful outcome. */
            in attribute operationId : String;
            out item receipt : OperationReceipt [0..1];
        }

        perform action list {
            doc /* Read all current item heads in one consistent snapshot. Optional
               kind and derived wish-status filters are conjunctive. */
            in attribute kind : String [0..1];
            in attribute status : String [0..1];
            out item selectedItems : StoreItem [0..*];
        }

        perform action get {
            doc /* Return one complete revision; absent revision selects the current
               head in the same read snapshot. */
            in attribute itemId : String;
            in attribute revision : Integer [0..1];
            out item selectedRevision : ItemRevision [0..1];
        }

        perform action history {
            doc /* Return every complete revision in ascending revision order. */
            in attribute itemId : String;
            out item selectedRevisions : ItemRevision [0..*];
        }

        perform action readMaterial : ReadMaterialOperation;
        perform action checkout : CheckoutOperation;
        perform action doctor : DoctorOperation;
    }

    action def SaveOperation {
        @XDesignCommon::DiagramPresentation { label = "Save revision"; summary = "Reserve, publish, then commit or conflict"; }
        @XDesignCommon::DocSection { document = "docs/design/storage.md"; section = "D86"; }
        doc /* One exact-request save with expected-head protection. A stored terminal
           receipt for identical request bytes returns before project-root
           canonicalization or any source-path resolution. The pending request is durable
           before any source read or publication. All required blobs are published with
           fsync and no-replace semantics before one BEGIN IMMEDIATE transaction rechecks
           the receipt and head, writes the item on create, appends the revision and
           complete manifest, finalizes the receipt, and advances the head. A stale head
           commits a conflict receipt and no revision. Post-publication or post-commit
           uncertainty is reconciled through outcome, never a fresh operation ID. */

        in attribute projectRootUri : String;
        in item request : SaveRequest;
        in attribute requestSha256 : String;
        out item receipt : OperationReceipt;

        action validateRequest {
            @XDesignCommon::DiagramPresentation { label = "Validate request"; }
            doc /* Enforce strict JSON limits and schema, IDs, lowercase digests, safe
               unique URIs, metadata object shape, and per-material limits before any
               reservation; no project-root or source-path resolution happens here.
               Manifest names must be materializable: no name is both file and directory,
               and distinct URI and implied-directory names stay distinct under
               host-independent Unicode canonical decomposition (NFD) plus case folding.
               Open numbers are accepted only when the stored representation is an exact
               decode/re-encode fixed point under the single pinned numeric parser and
               serializer; overflowing integers, non-canonical decimals, and unsupported
               extreme exponent ranges are rejected without rounding or panic. Results
               and metadata must reserve room for the response envelope. The exact
               request bytes determine requestSha256. */
        }
        then action inspectReceipt {
            @XDesignCommon::DiagramPresentation { label = "Inspect receipt"; }
            doc /* Compare both exact request bytes and their digest with any existing
               operation before resolving the project root or sources. The same ID with
               different bytes is operation_mismatch. */
            out attribute returnImmediately : Boolean = false;
        }
        then decide receiptPath {
            @XDesignCommon::DiagramPresentation { label = "Existing outcome?"; }
        }
        action returnStoredReceipt {
            @XDesignCommon::DiagramPresentation { label = "Return stored receipt"; }
            doc /* Return an exact committed or conflict receipt before project-root
               canonicalization or any source-path resolution. There is no second
               revision, publication or other storage effect. */
        }
        action reservePending {
            @XDesignCommon::DiagramPresentation { label = "Reserve or resume"; }
            doc /* Durably reserve a new pending operation and its exact request, or
               resume the same pending request. Reservation precedes every source read
               and blob publication. */
        }
        then action readSources {
            @XDesignCommon::DiagramPresentation { label = "Reuse published, read sources"; }
            doc /* Resolve the project root and source paths only after reservation. Reuse
               and digest-verify content backed by a committed blob row or this operation's
               own earlier publication before reading any source, so a pending recovery
               completes even after its transient sources are gone; an unrelated published
               or orphan blob is never adopted for an omitted source. Only then read
               remaining regular files through safe project-relative paths with the 256
               MiB per-material cap, requiring byte-for-byte digest matches. A source is
               never inside the store root under any case or symlink alias. */
        }
        then action publishBlobs {
            @XDesignCommon::DiagramPresentation { label = "Publish blobs"; }
            doc /* Stage, fsync and publish every required blob with no-replace semantics,
               then fsync containing directories. Identical committed bytes deduplicate;
               missing or altered final bytes are never trusted. */
        }
        then action commitTransaction {
            @XDesignCommon::DiagramPresentation { label = "Commit or conflict"; }
            doc /* BEGIN IMMEDIATE; recheck operation identity and expected head. Commit
               item creation when needed, revision, complete manifest, blob references,
               committed receipt and head advance together, or commit only a durable
               conflict receipt. No revision row may reference an unpublished blob. */
        }
        merge receiptReady;

        first start then validateRequest;
        succession storedReceiptSelected first receiptPath
            if inspectReceipt.returnImmediately then returnStoredReceipt;
        succession reservationSelected first receiptPath
            if not inspectReceipt.returnImmediately then reservePending;
        succession committedReceiptReady first commitTransaction then receiptReady;
        succession storedReceiptReady first returnStoredReceipt then receiptReady;
        first receiptReady then done;
    }

    action def ReadMaterialOperation {
        @XDesignCommon::DocSection { document = "docs/design/storage.md"; section = "D86"; }
        doc /* Select a revision and URI in one consistent snapshot, then read the blob
           through end of file while checking exact byte length and SHA-256. Missing or
           altered bytes return no content. Only a blob referenced by the committed
           manifest is eligible; an orphan is not a successful read. */
        in attribute itemId : String;
        in attribute revision : Integer [0..1];
        in attribute uri : String;
        out item verifiedMaterial : Material [0..1];
    }

    item def CheckoutReport {
        @XDesignCommon::DocSection { document = "docs/design/storage.md"; section = "D86"; }
        doc /* Summary of one no-replace materialization of a complete selected revision. */
        attribute itemId : String;
        attribute revision : Integer;
        attribute destinationUri : String;
        attribute materialCount : Integer;
    }

    action def CheckoutOperation {
        @XDesignCommon::DocSection { document = "docs/design/storage.md"; section = "D86"; }
        doc /* Verify every blob in a selected complete manifest through end of file,
           materialize each URI beneath a previously absent destination using safe path
           components and no-replace publication, and report the exact revision. A failure
           does not expose a partial destination as a successful checkout. */
        in attribute itemId : String;
        in attribute revision : Integer [0..1];
        in attribute destinationUri : String;
        out item report : CheckoutReport;
    }

    enum def DoctorFindingKind {
        @XDesignCommon::DocSection { document = "docs/design/storage.md"; section = "D86"; }
        doc /* Findings are diagnostics, not repairs or successful trusted reads. */
        enum orphanBlob;
        enum pendingOperation;
        enum missingBlob;
        enum alteredBlob;
        enum stagingArtifact;
        enum invalidRecord;
        enum formatDefect;
        enum identityMismatch;
        enum permissionDefect;
    }

    item def DoctorFinding {
        @XDesignCommon::DocSection { document = "docs/design/storage.md"; section = "D86"; }
        attribute kind : DoctorFindingKind;
        attribute subjectUri : String;
        attribute detail : String;
    }

    item def DoctorReport {
        @XDesignCommon::DocSection { document = "docs/design/storage.md"; section = "D86"; }
        doc /* Complete read-only inspection result. verifyBytes selects full SHA-256
           verification in addition to record, reference, identity and length checks. */
        attribute verifyBytes : Boolean;
        item findings : DoctorFinding [0..*];
    }

    action def DoctorOperation {
        @XDesignCommon::DocSection { document = "docs/design/storage.md"; section = "D86"; }
        doc /* Inspect format and filesystem identity, owner permissions, record and
           manifest references, orphan blobs, pending operations and staging residue.
           Optionally hash every published blob through end of file. Never repair, delete,
           migrate, or convert findings into a clean result. */
        in attribute verifyBytes : Boolean;
        out item report : DoctorReport;

        action inspectRecords {
            doc /* Check schema and root identity, database identity, permissions,
               item/head/revision continuity, complete manifests, blob references and
               operation receipts without mutating them. */
        }
        then action inspectBytes {
            doc /* When verifyBytes is true, verify every published blob's length and
               SHA-256 through end of file; otherwise check presence and recorded length. */
        }
        then action reportFindings {
            doc /* Return every discovered finding, including orphan and staging state;
               perform no cleanup or implicit repair. */
        }
        first start then inspectRecords;
        succession recordsInspected first inspectRecords then inspectBytes;
        succession bytesInspected first inspectBytes then reportFindings;
    }

    view storageStructure : StandardViewDefinitions::GeneralView {
        @XDesignCommon::DiagramPresentation { label = "Storage structure"; question = "How do items, immutable revisions, materials and receipts relate?"; }
        @XDesignCommon::DocSection { document = "docs/design/storage.md"; section = "D86"; }
        @XDesignCommon::DocFigure { document = "docs/design/storage.md"; figure = "docs/design/assets/storage-structure.svg"; }
        doc /* Generic item history, complete material manifest, exact-operation
           request and receipt, and derived wish projection. ProjectStore operations
           are explained in text and the save view rather than mixed into this record
           structure projection. */
        expose XDesignStorage::StoreItem;
        expose XDesignStorage::ItemRevision;
        expose XDesignStorage::Material;
        expose XDesignStorage::SaveMaterial;
        expose XDesignStorage::SaveRequest;
        expose XDesignStorage::OperationReceipt;
        expose XDesignStorage::OperationOutcome;
        expose XDesignStorage::WishProjection;
    }

    view storageSave : StandardViewDefinitions::ActionFlowView {
        @XDesignCommon::DiagramPresentation { label = "Atomic save"; question = "How does a save reserve, publish and commit without duplicate effects?"; }
        @XDesignCommon::DocSection { document = "docs/design/storage.md"; section = "D86"; }
        @XDesignCommon::DocFigure { document = "docs/design/storage.md"; figure = "docs/design/assets/storage-save.svg"; }
        doc /* Exact-request validation and receipt inspection, early terminal return,
               pending reservation, source verification and blob publication before the
               atomic revision/head/receipt transaction. */
        expose XDesignStorage::SaveOperation;
        expose XDesignStorage::SaveOperation::*;
        render Views::asInterconnectionDiagram;
    }
}
