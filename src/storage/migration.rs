//! Explicit location transfer. No public save is used to import old history.
use super::{
    access::{self, diagnostic, AccessLease},
    blobs, db, io_error,
    location::{self, Binding},
    sha256, strict_json, RsStore, RsStoreError,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::os::unix::{
    ffi::OsStrExt,
    fs::{OpenOptionsExt, PermissionsExt},
};
use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::Arc,
};

const ACCESS: &str = ".xdesign-access-v1";
fn corrupt(s: impl std::fmt::Display) -> RsStoreError {
    diagnostic("coordination_corrupt", s)
}
fn absent(path: &Path) -> Result<bool, RsStoreError> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(false),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(true),
        Err(e) => Err(io_error(e)),
    }
}
fn key(root: &Path) -> Result<String, RsStoreError> {
    Ok(sha256(&location::domain(
        "access-root-key",
        &location::encode(
            6,
            &json!({"schema_version":1,"owner_uid":unsafe{libc::geteuid()}.to_string(),"parent":location::hex(root.parent().ok_or_else(||corrupt("no parent"))?.as_os_str().as_bytes()),"child":location::hex(root.file_name().ok_or_else(||corrupt("no child"))?.as_bytes())}),
        )?,
    )))
}
fn access_dir(root: &Path) -> PathBuf {
    root.parent().unwrap().join(ACCESS)
}
fn pointer(root: &Path) -> Result<PathBuf, RsStoreError> {
    Ok(access_dir(root)
        .join("roots")
        .join(format!("{}.json", key(root)?)))
}
fn marker(parent: &Path, create: bool) -> Result<(), RsStoreError> {
    let dir = parent.join(ACCESS);
    if absent(&dir)? {
        if !create {
            return Ok(());
        }
        // Publish a complete directory. An interrupted unpublished draft never
        // occupies the reserved name and is never adopted or removed on retry.
        let draft = parent.join(format!(".xdesign-access-draft-{}", location::random()?));
        location::mkdir(&draft)?;
        fault("access-directory-created");
        let bytes=serde_json::to_vec(&json!({"format":"xdesign.store-access","schema_version":1,"owner_uid":unsafe{libc::geteuid()}.to_string(),"parent_path":parent,"parent_identity":location::observe(parent,None)?,"epoch":location::random()?})).map_err(corrupt)?;
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(draft.join("format.json"))?;
        file.write_all(&bytes)?;
        fault("access-marker-written");
        blobs::sync_durable(&file)?;
        drop(file);
        fault("access-marker-synced");
        for d in ["roots", "routes", "migrations", "reservation-drafts"] {
            location::mkdir(&draft.join(d))?;
        }
        blobs::fsync_dir(&draft)?;
        location::rename_exclusive(&draft, &dir)?;
        fault("access-directory-published");
        blobs::fsync_dir(parent)?;
        fault("access-directory-durable");
    }
    location::safe_dir(&dir)?;
    let value = location::json_file(&dir.join("format.json"))?;
    if value.as_object().map(|o| o.len()) != Some(6)
        || value["format"] != "xdesign.store-access"
        || value["schema_version"] != 1
        || value["owner_uid"] != unsafe { libc::geteuid() }.to_string()
        || value["parent_path"] != json!(parent)
        || value["parent_identity"] != json!(location::observe(parent, None)?)
    {
        return Err(corrupt("access namespace identity mismatch"));
    }
    for name in ["roots", "routes", "migrations", "reservation-drafts"] {
        location::safe_dir(&dir.join(name))?;
    }
    for item in fs::read_dir(&dir)? {
        let item = item?;
        if ![
            "format.json",
            "roots",
            "routes",
            "migrations",
            "reservation-drafts",
        ]
        .iter()
        .any(|n| item.file_name() == *n)
        {
            return Err(corrupt("foreign access namespace entry"));
        }
    }
    Ok(())
}
#[derive(Clone)]
struct Route {
    binding: Binding,
    payload: Value,
    bytes: Vec<u8>,
}
impl Route {
    fn source(&self) -> PathBuf {
        PathBuf::from(self.payload["source_root"].as_str().unwrap())
    }
    fn destination(&self) -> PathBuf {
        self.binding.root()
    }
    fn base(&self) -> PathBuf {
        PathBuf::from(self.payload["journal_base"].as_str().unwrap())
    }
    fn digest(&self) -> String {
        sha256(&self.bytes)
    }
    fn parents(&self) -> Vec<PathBuf> {
        vec![
            self.binding.namespace.clone(),
            self.binding.project(),
            self.source().parent().unwrap().to_owned(),
            self.destination().parent().unwrap().to_owned(),
        ]
    }
    fn verify(&self) -> Result<(), RsStoreError> {
        self.verify_setup(true)
    }
    fn verify_read(&self, root: &Path) -> Result<(), RsStoreError> {
        if root != self.destination() {
            return self.verify();
        }
        self.binding.verify_archive()?;
        let parent = root.parent().unwrap();
        if self.payload["destination_parent"] != json!(location::observe(parent, None)?) {
            return Err(diagnostic(
                "identity_unprovable",
                "destination parent changed",
            ));
        }
        marker(parent, false)?;
        let path = access_dir(root)
            .join("routes")
            .join(key(root)?)
            .join(format!("{}.json", self.binding.key()));
        if blobs::read_store_file(&path, 0o600)? != self.bytes {
            return Err(corrupt("destination route changed"));
        }
        verify_pointer(self, root)
    }
    fn verify_setup(&self, complete: bool) -> Result<(), RsStoreError> {
        self.binding.verify()?;
        for (role, path) in [
            ("source_parent", self.source()),
            ("destination_parent", self.destination()),
        ] {
            let parent = path.parent().unwrap();
            if self.payload[role] != json!(location::observe(parent, None)?) {
                return Err(diagnostic("identity_unprovable", "route parent changed"));
            }
            marker(parent, false)?;
            let p = access_dir(&path)
                .join("routes")
                .join(key(&path)?)
                .join(format!("{}.json", self.binding.key()));
            if !complete && absent(&p)? {
                continue;
            }
            if blobs::read_store_file(&p, 0o600)? != self.bytes {
                return Err(corrupt("permanent route copies differ"));
            }
            verify_pointer(self, &path)?;
        }
        Ok(())
    }
}
fn read_route(bytes: Vec<u8>) -> Result<Route, RsStoreError> {
    let record = strict_json(&bytes).map_err(corrupt)?;
    let payload = &record["payload"];
    let ns = payload["namespace_path"]
        .as_str()
        .ok_or_else(|| corrupt("route namespace"))?;
    let key = payload["binding_key"]
        .as_str()
        .ok_or_else(|| corrupt("route binding"))?;
    let binding = location::load_binding(Path::new(ns), key)?;
    let payload = location::authenticate(&binding.key_bytes, "route", 7, &record)?;
    if payload["namespace_epoch"] != binding.payload["namespace_epoch"]
        || payload["owner_uid"] != binding.payload["owner_uid"]
        || payload["binding_sha256"] != binding.digest()
        || payload["source_root"] != json!(binding.project().join(".xdesign"))
        || payload["destination_root"] != json!(binding.root())
        || payload["journal_base"] != json!(access_dir(&binding.root()).join("migrations"))
    {
        return Err(corrupt("route differs from binding"));
    }
    Ok(Route {
        binding,
        payload,
        bytes,
    })
}
fn route(root: &Path) -> Result<Option<Route>, RsStoreError> {
    marker(root.parent().unwrap(), false)?;
    let bucket = access_dir(root).join("routes").join(key(root)?);
    let mut found = None;
    if !absent(&bucket)? {
        location::safe_dir(&bucket)?;
        for entry in fs::read_dir(bucket)? {
            let entry = entry?;
            let r = read_route(blobs::read_store_file(&entry.path(), 0o600)?)?;
            if root != r.source() && root != r.destination() {
                return Err(corrupt("route bucket collision"));
            }
            if found.is_some() {
                return Err(corrupt("multiple bindings route one root"));
            }
            found = Some(r);
        }
    }
    // The independent attribute survives renames and loss of parent coordination.
    if !absent(root)? {
        if let Some(raw) = location::attribute(root, location::ACCESS_ATTRIBUTE)? {
            let a = strict_json(&raw).map_err(corrupt)?;
            if a.get("initialization_path").is_some() {
                if a["initialization_path"]
                    != json!(root.parent().unwrap().join("initialization.json"))
                {
                    return Err(diagnostic(
                        "source_moved",
                        "initialized root moved from its bound location",
                    ));
                }
                let r = found
                    .as_ref()
                    .ok_or_else(|| corrupt("initialization reference has no permanent route"))?;
                let init = initialization_record(&r.binding)?;
                if a != initialization_reference(&r.binding, &init) {
                    return Err(corrupt("initialization reference mismatch"));
                }
                return Ok(found);
            }
            if a["pointer_path"] != json!(pointer(root)?) {
                return Err(diagnostic(
                    "source_moved",
                    "access reference no longer matches canonical root",
                ));
            }
            if found.is_none() || absent(&pointer(root)?)? {
                return Err(corrupt(
                    "access reference has no retained pointer and route",
                ));
            }
        }
    }
    Ok(found)
}
pub(crate) fn peers(root: &Path) -> Result<Vec<PathBuf>, RsStoreError> {
    if let Some(r) = route(root)? {
        // A retired/moved project may be absent during explicit archive reads.
        // Mutation still requires the entire live binding in guard().
        if root == r.destination() && absent(&r.binding.project())? {
            r.binding.verify_archive()?;
            return Ok(vec![
                r.binding.namespace.clone(),
                root.parent().unwrap().to_owned(),
            ]);
        }
        return Ok(r.parents());
    }
    if let Some(b) = adjacent_binding(root)? {
        return Ok(vec![
            b.namespace.clone(),
            b.project(),
            root.parent().unwrap().to_owned(),
        ]);
    }
    Ok(vec![])
}

pub(crate) fn related(root: &Path) -> Result<(Vec<PathBuf>, Option<PathBuf>), RsStoreError> {
    let binding = if let Some(r) = route(root)? {
        Some(r.binding)
    } else {
        adjacent_binding(root)?
    };
    if let Some(b) = binding {
        let project = b.project();
        let mut roots = vec![b.root()];
        let project = if !absent(&project)? {
            roots.push(project.join(".xdesign"));
            Some(project)
        } else {
            None
        };
        Ok((roots, project))
    } else {
        Ok((vec![], None))
    }
}

pub(crate) fn install_routes(binding: &Binding) -> Result<(), RsStoreError> {
    let source = binding.project().join(".xdesign");
    let destination = binding.root();
    let paths = vec![
        binding.namespace.clone(),
        binding.project(),
        destination.parent().unwrap().to_owned(),
    ];
    let _lease = AccessLease::acquire(&paths)?;
    binding.verify()?;
    for root in [&source, &destination] {
        marker(root.parent().unwrap(), true)?;
        location::mkdir(&access_dir(root).join("routes").join(key(root)?))?;
    }
    let payload = json!({"schema_version":1,"namespace_epoch":binding.payload["namespace_epoch"],"owner_uid":binding.payload["owner_uid"],"binding_key":binding.key(),"binding_sha256":binding.digest(),"source_root":source,"destination_root":destination,"source_parent":location::observe(source.parent().unwrap(),None)?,"destination_parent":location::observe(destination.parent().unwrap(),None)?,"journal_base":access_dir(&destination).join("migrations"),"namespace_path":binding.namespace});
    let bytes = serde_json::to_vec(&location::sign(&binding.key_bytes, "route", 7, payload)?)
        .map_err(corrupt)?;
    for root in [&source, &destination] {
        location::publish(
            &access_dir(root)
                .join("routes")
                .join(key(root)?)
                .join(format!("{}.json", binding.key())),
            &bytes,
        )?;
        fault(if root == &source {
            "source-route-published"
        } else {
            "destination-route-published"
        });
    }
    let r = read_route(bytes)?;
    r.verify()?;
    Ok(())
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub(crate) struct Request {
    schema_version: u32,
    pub migration_id: String,
    project_root: PathBuf,
    binding_key: String,
    binding_sha256: String,
    source_root: PathBuf,
    source_store_id: String,
    source_identity: Physical,
    destination_root: PathBuf,
    older_clients_stopped: bool,
}
#[derive(Deserialize, Serialize, Clone, PartialEq, Eq, Debug)]
#[serde(deny_unknown_fields)]
struct Physical {
    root_device: String,
    root_inode: String,
    database_device: String,
    database_inode: String,
}
fn physical(store: &RsStore) -> Physical {
    Physical {
        root_device: store.identity.root_device.to_string(),
        root_inode: store.identity.root_inode.to_string(),
        database_device: store.identity.database_device.to_string(),
        database_inode: store.identity.database_inode.to_string(),
    }
}
fn parse_request(raw: &[u8]) -> Result<Request, RsStoreError> {
    if raw.len() > 1024 * 1024 {
        return Err(diagnostic(
            "invalid_input",
            "migration request exceeds 1 MiB",
        ));
    }
    strict_json(raw).map_err(|e| diagnostic("invalid_input", e))?;
    let r: Request = serde_json::from_slice(raw).map_err(|e| diagnostic("invalid_input", e))?;
    if r.schema_version != 1
        || !super::valid_id(&r.migration_id)
        || !super::valid_digest(&r.binding_key)
        || !super::valid_digest(&r.binding_sha256)
        || !r.older_clients_stopped
    {
        return Err(diagnostic(
            "invalid_input",
            "invalid migration identity or missing old-client quiescence attestation",
        ));
    }
    Ok(r)
}
struct Attempt {
    route: Route,
    id: String,
    journal: PathBuf,
    reservation: Value,
    bytes: Vec<u8>,
}
impl Attempt {
    fn nonce(&self) -> &str {
        self.reservation["attempt_nonce"].as_str().unwrap()
    }
    fn digest(&self) -> String {
        sha256(&self.bytes)
    }
    fn request_digest(&self) -> &str {
        self.reservation["request_sha256"].as_str().unwrap()
    }
    fn common(&self) -> Value {
        json!({"schema_version":1,"migration_id":self.id,"attempt_nonce":self.nonce(),"reservation_sha256":self.digest(),"request_sha256":self.request_digest()})
    }
    fn record(&self, name: &str) -> Result<Option<Value>, RsStoreError> {
        let p = self.journal.join(name);
        if absent(&p)? {
            return Ok(None);
        }
        let v = location::json_file(&p)?;
        let extra: &[&str] = match name {
            "state.json" => &[
                "generation",
                "baseline_sha256",
                "prepared_generation",
                "diagnostics",
                "phase",
                "source_identity",
                "destination_identity",
            ],
            "baseline.json" => &["inventory", "source_identity"],
            "verification.json" => &[
                "source_inventory",
                "destination_inventory",
                "destination_identity",
                "owned_objects",
            ],
            "decision.json" => &[
                "binding_key",
                "binding_sha256",
                "source_identity",
                "destination_identity",
                "source_baseline_sha256",
                "verification_sha256",
                "authority_root",
                "outcome",
            ],
            "abort.json" => &["outcome", "invalidated_generation", "residue_identity"],
            "discard.json" => &["discarded"],
            _ => return Err(corrupt("unknown migration record")),
        };
        let common = self.common();
        if !v.as_object().is_some_and(|o| {
            o.len() == common.as_object().unwrap().len() + extra.len()
                && o.keys()
                    .all(|k| common.get(k).is_some() || extra.contains(&k.as_str()))
        }) {
            return Err(corrupt(format!("{name} has unknown or missing fields")));
        }
        for field in [
            "schema_version",
            "migration_id",
            "attempt_nonce",
            "reservation_sha256",
            "request_sha256",
        ] {
            if v[field] != self.common()[field] {
                return Err(corrupt(format!("{name} disagrees with reservation")));
            }
        }
        Ok(Some(v))
    }
    fn decision(&self) -> Result<Option<Value>, RsStoreError> {
        failure("decision-read")?;
        let decision = self.record("decision.json").map_err(|e| match e {
            RsStoreError::Io { .. } => diagnostic("migration_outcome_unknown", e),
            _ => e,
        })?;
        if decision.is_some() && self.record("abort.json")?.is_some() {
            return Err(corrupt("contradictory terminal records"));
        }
        if let Some(d) = &decision {
            if d["binding_key"] != self.route.binding.key()
                || d["binding_sha256"] != self.route.binding.digest()
                || d["authority_root"] != json!(self.route.destination())
                || d["outcome"] != "committed"
            {
                return Err(corrupt("invalid cutover decision"));
            }
            let verification = self
                .record("verification.json")?
                .ok_or_else(|| corrupt("decision verification is missing"))?;
            if d["verification_sha256"]
                != sha256(&blobs::read_store_file(
                    &self.journal.join("verification.json"),
                    0o600,
                )?)
                || d["destination_identity"] != verification["destination_identity"]
                || d["source_baseline_sha256"] != verification["source_inventory"]["logical_sha256"]
                || verification["source_inventory"] != verification["destination_inventory"]
            {
                return Err(corrupt("decision and verified inventories disagree"));
            }
            let file = File::open(self.journal.join("decision.json"))
                .map_err(|e| diagnostic("migration_outcome_unknown", e))?;
            blobs::sync_durable(&file).map_err(|e| diagnostic("migration_outcome_unknown", e))?;
            failure("decision-sync")?;
            blobs::fsync_dir(&self.journal)
                .map_err(|e| diagnostic("migration_outcome_unknown", e))?;
            verify_object(&d["destination_identity"], &self.route.destination(), false)?;
        } else {
            failure("decision-sync")?;
            blobs::fsync_dir(&self.journal)
                .map_err(|e| diagnostic("migration_outcome_unknown", e))?;
        }
        Ok(decision)
    }
    fn state(&self) -> Result<Option<Value>, RsStoreError> {
        self.record("state.json")
    }
    fn set_state(&self, phase: &str, destination: Option<Value>) -> Result<(), RsStoreError> {
        let mut v = self.common();
        v["generation"] = json!("1");
        v["baseline_sha256"] = Value::Null;
        v["prepared_generation"] = if phase == "prepared" {
            json!("1")
        } else {
            Value::Null
        };
        v["diagnostics"] = json!({});
        v["phase"] = json!(phase);
        v["source_identity"] = json!({"root":self.reservation["source_root_identity"],"database":self.reservation["source_database_identity"]});
        v["destination_identity"] = destination.unwrap_or(Value::Null);
        location::replace_json(&self.journal.join("state.json"), &v)
    }
}
fn load_attempt(route: &Route, id: &str) -> Result<Option<Attempt>, RsStoreError> {
    if !super::valid_id(id) {
        return Err(diagnostic("invalid_input", "invalid migration ID"));
    }
    let journal = route.base().join(id);
    if absent(&journal)? {
        return Ok(None);
    }
    location::safe_dir(&journal)?;
    let allowed = [
        "reservation.json",
        "request.json",
        "state.json",
        "baseline.json",
        "verification.json",
        "decision.json",
        "abort.json",
        "discard.json",
    ];
    for entry in fs::read_dir(&journal)? {
        if !allowed
            .iter()
            .any(|n| entry.as_ref().is_ok_and(|e| e.file_name() == *n))
        {
            return Err(corrupt("unexpected migration slot content"));
        }
    }
    let path = journal.join("reservation.json");
    if absent(&path)? {
        if fs::read_dir(&journal)?.next().is_some() {
            return Err(corrupt("slot effects without reservation"));
        }
        return Ok(None);
    }
    let bytes = blobs::read_store_file(&path, 0o600)?;
    let value = strict_json(&bytes).map_err(corrupt)?;
    let reservation = location::authenticate(&route.binding.key_bytes, "reservation", 8, &value)?;
    let raw = location::unhex(
        reservation["request_bytes"]
            .as_str()
            .ok_or_else(|| corrupt("request bytes"))?,
    )?;
    let request = parse_request(&raw)?;
    if reservation["migration_id"] != id
        || request.migration_id != id
        || reservation["binding_key"] != route.binding.key()
        || reservation["binding_sha256"] != route.binding.digest()
        || reservation["route_sha256"] != route.digest()
        || reservation["request_sha256"] != sha256(&raw)
        || reservation["journal_identity"] != json!(location::observe(&journal, None)?)
        || reservation["namespace_epoch"] != route.binding.payload["namespace_epoch"]
        || reservation["owner_uid"] != route.binding.payload["owner_uid"]
    {
        return Err(corrupt("reservation identity differs"));
    }
    if request.source_root != route.source()
        || request.destination_root != route.destination()
        || request.project_root != route.binding.project()
        || request.binding_key != route.binding.key()
        || request.binding_sha256 != route.binding.digest()
    {
        return Err(corrupt("request routing differs"));
    }
    blobs::sync_durable(&File::open(&path)?)
        .map_err(|e| diagnostic("migration_outcome_unknown", e))?;
    blobs::fsync_dir(&journal).map_err(|e| diagnostic("migration_outcome_unknown", e))?;
    if !absent(&journal.join("request.json"))?
        && blobs::read_store_file(&journal.join("request.json"), 0o600)? != raw
    {
        return Err(corrupt("derivative request differs"));
    }
    Ok(Some(Attempt {
        route: route.clone(),
        id: id.to_owned(),
        journal,
        reservation,
        bytes,
    }))
}

fn verify_pointer(route: &Route, root: &Path) -> Result<(), RsStoreError> {
    let path = pointer(root)?;
    if absent(&path)? {
        return Ok(());
    }
    let value = location::json_file(&path)?;
    let fields = [
        "schema_version",
        "generation",
        "root_path",
        "parent_identity",
        "root_identity",
        "role",
        "migration_id",
        "journal_path",
        "journal_identity",
        "request_sha256",
        "attempt_nonce",
        "reservation_sha256",
        "attempts",
    ];
    if !value
        .as_object()
        .is_some_and(|o| o.len() == fields.len() && o.keys().all(|k| fields.contains(&k.as_str())))
    {
        return Err(corrupt("invalid root pointer fields"));
    }
    let id = value["migration_id"]
        .as_str()
        .ok_or_else(|| corrupt("pointer migration ID"))?;
    let a = load_attempt(route, id)?.ok_or_else(|| corrupt("root pointer lost its reservation"))?;
    if value["schema_version"] != 1
        || value["root_path"] != json!(root)
        || value["parent_identity"] != json!(location::observe(root.parent().unwrap(), None)?)
        || value["journal_path"] != json!(a.journal)
        || value["journal_identity"] != a.reservation["journal_identity"]
        || value["attempt_nonce"] != a.nonce()
        || value["reservation_sha256"] != a.digest()
        || value["request_sha256"] != a.request_digest()
        || value["role"]
            != if root == route.source() {
                "source"
            } else {
                "destination"
            }
    {
        return Err(corrupt(
            "root pointer disagrees with authenticated reservation",
        ));
    }
    if !absent(root)? {
        if let Some(raw) = location::attribute(root, location::ACCESS_ATTRIBUTE)? {
            let attribute = strict_json(&raw).map_err(corrupt)?;
            let marker = location::json_file(&access_dir(root).join("format.json"))?;
            let expected = json!({"schema_version":1,"pointer_path":path,"access_epoch":marker["epoch"],"migration_id":id,"request_sha256":a.request_digest(),"attempt_nonce":a.nonce(),"reservation_sha256":a.digest()});
            // An aborted attempt may retain its prior attribute until an exact
            // new apply replaces it after publication of the new pointer.
            if attribute != expected {
                let old_id = attribute["migration_id"]
                    .as_str()
                    .ok_or_else(|| corrupt("access reference lacks attempt"))?;
                let old = load_attempt(route, old_id)?
                    .ok_or_else(|| corrupt("access reference lost its reservation"))?;
                if old.record("abort.json")?.is_none() || old.decision()?.is_some() {
                    return Err(corrupt("root access reference differs"));
                }
            }
        }
    }
    let prior = value["attempts"]
        .as_array()
        .ok_or_else(|| corrupt("pointer attempts must be an array"))?;
    for old in prior {
        let old_id = old["migration_id"]
            .as_str()
            .ok_or_else(|| corrupt("prior pointer ID"))?;
        let old_attempt = load_attempt(route, old_id)?
            .ok_or_else(|| corrupt("prior pointer reservation lost"))?;
        if old["attempt_nonce"] != old_attempt.nonce()
            || old["reservation_sha256"] != old_attempt.digest()
            || old["request_sha256"] != old_attempt.request_digest()
            || old["journal_path"] != json!(old_attempt.journal)
            || old["journal_identity"] != old_attempt.reservation["journal_identity"]
        {
            return Err(corrupt("prior root pointer differs"));
        }
    }
    Ok(())
}
fn attempts(route: &Route) -> Result<Vec<Attempt>, RsStoreError> {
    let mut out = vec![];
    for entry in fs::read_dir(route.base())? {
        let e = entry?;
        let id = e
            .file_name()
            .to_str()
            .ok_or_else(|| corrupt("non-UTF-8 migration ID"))?
            .to_owned();
        if let Some(a) = load_attempt(route, &id)? {
            out.push(a);
        }
    }
    out.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(out)
}
fn object_pair(root: &Path) -> Result<Value, RsStoreError> {
    Ok(
        json!({"root":location::observe(root,None)?,"database":location::observe(&root.join("store.db"),None)?}),
    )
}
fn verify_object(value: &Value, root: &Path, source: bool) -> Result<(), RsStoreError> {
    let live = object_pair(root).map_err(|e| {
        diagnostic(
            if source {
                "source_changed"
            } else {
                "identity_unprovable"
            },
            e,
        )
    })?;
    if &live != value {
        return Err(diagnostic(
            if source {
                "source_changed"
            } else {
                "identity_unprovable"
            },
            "store physical identity differs from retained mapping",
        ));
    }
    Ok(())
}
fn source_check(a: &Attempt, decision: &Value) -> Result<(), RsStoreError> {
    if absent(&a.route.source())? {
        return Ok(());
    }
    verify_object(&decision["source_identity"], &a.route.source(), true)?;
    let source = db::open(&a.route.source()).map_err(|e| diagnostic("source_changed", e))?;
    let inv = inventory(&source).map_err(|e| diagnostic("source_changed", e))?;
    if inv["logical_sha256"] != decision["source_baseline_sha256"] {
        return Err(diagnostic("source_changed","retained source history changed after cutover; stop older clients and inspect both histories"));
    }
    Ok(())
}

pub(crate) fn guard(root: &Path, mutation: bool) -> Result<(), RsStoreError> {
    let Some(route) = route(root)? else {
        if adjacent_binding(root)?.is_some() {
            return Err(corrupt("bound root is missing permanent routes"));
        }
        return Ok(());
    };
    if mutation {
        route.verify()?;
    } else {
        route.verify_read(root)?;
    }
    let list = attempts(&route)?;
    let has_decision = list.iter().try_fold(false, |seen, a| {
        Ok::<_, RsStoreError>(seen || a.decision()?.is_some())
    })?;
    let mut committed = 0;
    for a in list {
        if let Some(d) = a.decision()? {
            committed += 1;
            if committed > 1 {
                return Err(corrupt("multiple migration decisions"));
            }
            if root == route.source() {
                if mutation {
                    return Err(diagnostic(
                        "source_retired",
                        format!("authority moved to {}", route.destination().display()),
                    ));
                }
                if !absent(root)? {
                    verify_object(&d["source_identity"], root, true)?;
                }
            } else if mutation {
                source_check(&a, &d)?;
            }
        } else if a.record("abort.json")?.is_some() {
            if root == route.destination() && !has_decision && a.record("discard.json")?.is_none() {
                return Err(diagnostic(
                    "migration_aborted",
                    "aborted destination is not authoritative",
                ));
            }
        } else if root == route.destination()
            || (!absent(&pointer(&route.source())?)?
                && location::json_file(&pointer(&route.source())?)?["migration_id"] == a.id)
        {
            return Err(diagnostic(
                "migration_busy",
                format!("migration {} is unfinished", a.id),
            ));
        }
    }
    if root == route.destination() && committed == 0 {
        let init = route
            .destination()
            .parent()
            .unwrap()
            .join("initialization.json");
        if !absent(root)? {
            if absent(&init)? {
                return Err(diagnostic(
                    "identity_unprovable",
                    "global root lacks verified creation provenance",
                ));
            }
            let v = initialization_record(&route.binding)?;
            if v["phase"] != "verified" || v["binding_sha256"] != route.binding.digest() {
                return Err(diagnostic(
                    "identity_unprovable",
                    "global initialization is incomplete",
                ));
            }
            verify_object(&initialization_identity(&v), root, false)?;
            let expected = serde_json::to_vec(&initialization_reference(&route.binding, &v))
                .map_err(corrupt)?;
            if location::attribute(root, location::ACCESS_ATTRIBUTE)?.as_deref() != Some(&expected)
            {
                return Err(diagnostic(
                    "identity_unprovable",
                    "initialization witness is missing",
                ));
            }
        }
    }
    Ok(())
}
pub(crate) fn select(binding: &Binding, local: bool) -> Result<(), RsStoreError> {
    let Some(route) = route(&binding.root())? else {
        return Err(corrupt("binding has no permanent route"));
    };
    route.verify()?;
    let list = attempts(&route)?;
    for a in &list {
        if let Some(d) = a.decision()? {
            source_check(a, &d)?;
            return Ok(());
        }
        if a.record("abort.json")?.is_none() {
            return Err(diagnostic("migration_busy", "migration is unfinished"));
        }
    }
    if local {
        return Err(diagnostic(
            if binding.root().exists() {
                "history_conflict"
            } else {
                "migration_required"
            },
            "local candidate requires explicit migration",
        ));
    }
    guard(&binding.root(), false)
}

pub(crate) fn select_initialization(binding: &Binding, local: bool) -> Result<(), RsStoreError> {
    if !local {
        let path = binding.root().parent().unwrap().join("initialization.json");
        if !absent(&path)? {
            let r =
                route(&binding.root())?.ok_or_else(|| corrupt("initialization has no route"))?;
            r.verify()?;
            let v = initialization_record(binding)?;
            if attempts(&r)?.is_empty()
                && v["schema_version"] == 1
                && v["binding_sha256"] == binding.digest()
                && v["binding_key"] == binding.key()
                && ["reserved", "verified"]
                    .iter()
                    .any(|phase| v["phase"] == *phase)
            {
                return Ok(());
            }
        }
    }
    select(binding, local)
}

/// An acquired diagnostic lease may describe preparation, never its partial
/// SQLite history. Normal construction/query paths still use the strict guard.
pub(crate) fn preparation_diagnostic(root: &Path) -> Result<Option<Value>, RsStoreError> {
    let Some(route) = route(root)? else {
        return Ok(None);
    };
    route.verify_read(root)?;
    for a in attempts(&route)? {
        if a.decision()?.is_some() {
            continue;
        }
        let aborted = a.record("abort.json")?.is_some();
        if root == route.destination() || (!aborted && !absent(&pointer(&route.source())?)?) {
            return Ok(Some(
                json!({"source_root":route.source(),"destination_root":route.destination(),"authority_state":"non_authoritative_preparation","migration":outcome(&a)?,"history_inspected":false}),
            ));
        }
    }
    Ok(None)
}

/// The constructor cannot be called without this private owned lease.
pub(crate) struct MigrationLease {
    lease: Arc<AccessLease>,
    journal: PathBuf,
}
fn table_rows(
    conn: &rusqlite::Connection,
    table: &str,
) -> Result<Vec<Vec<rusqlite::types::Value>>, RsStoreError> {
    let order = match table {
        "operations" => "operation_id",
        "items" | "revisions" => "item_id, 2",
        "materials" => "item_id, revision, uri",
        "blobs" => "sha256",
        "pending_blobs" => "operation_id, sha256",
        _ => "type,name",
    };
    let sql = if table == "sqlite_schema" {
        "SELECT type,name,tbl_name,sql FROM sqlite_schema ORDER BY type,name".into()
    } else {
        format!("SELECT * FROM {table} ORDER BY {order}")
    };
    let mut stmt = conn.prepare(&sql)?;
    let count = stmt.column_count();
    let mut rows = stmt.query([])?;
    let mut out = vec![];
    while let Some(row) = rows.next()? {
        out.push(
            (0..count)
                .map(|i| row.get(i))
                .collect::<Result<Vec<_>, _>>()?,
        );
    }
    Ok(out)
}
fn hash_rows(rows: &[Vec<rusqlite::types::Value>]) -> String {
    use rusqlite::types::Value::*;
    let mut h = Sha256::new();
    h.update((rows.len() as u64).to_be_bytes());
    for row in rows {
        h.update((row.len() as u64).to_be_bytes());
        for v in row {
            let (tag, b) = match v {
                Null => (0, vec![]),
                Integer(n) => (1, n.to_be_bytes().to_vec()),
                Real(n) => (2, n.to_bits().to_be_bytes().to_vec()),
                Text(s) => (3, s.as_bytes().to_vec()),
                Blob(b) => (4, b.clone()),
            };
            h.update([tag]);
            h.update((b.len() as u64).to_be_bytes());
            h.update(b);
        }
    }
    location::hex(&h.finalize())
}
fn inventory(store: &RsStore) -> Result<Value, RsStoreError> {
    db::verify_live(store)?;
    let report = super::ops::inspect_root(&store.root, true, Some(store))?;
    if !report.findings.is_empty() {
        return Err(diagnostic(
            "integrity",
            format!("source has doctor findings: {:?}", report.findings),
        ));
    }
    for p in ["locks/items", "staging"] {
        if fs::read_dir(store.root.join(p))?.next().is_some() {
            return Err(diagnostic("migration_busy", format!("{p} is not empty")));
        }
    }
    let tx = db::Transaction::begin(&store.conn, false)?;
    super::ops::validate_migration_flows(store)?;
    let integrity: String = store
        .conn
        .query_row("PRAGMA integrity_check", [], |r| r.get(0))?;
    if integrity != "ok" {
        return Err(diagnostic("integrity", integrity));
    }
    if store
        .conn
        .prepare("PRAGMA foreign_key_check")?
        .query([])?
        .next()?
        .is_some()
    {
        return Err(diagnostic("integrity", "foreign key check failed"));
    }
    let reference = rusqlite::Connection::open_in_memory()?;
    reference.execute_batch(db::SCHEMA)?;
    let ddl = table_rows(&store.conn, "sqlite_schema")?;
    if ddl != table_rows(&reference, "sqlite_schema")? {
        return Err(diagnostic(
            "unsupported_format",
            "source schema objects differ from supported schema 1",
        ));
    }
    let mut tables = vec![];
    let mut blob_summary = Value::Null;
    let mut h = Sha256::new();
    h.update(b"XDWS-INVENTORY-1");
    let format = blobs::read_store_file(&store.root.join("format.json"), 0o600)?;
    h.update((format.len() as u64).to_be_bytes());
    h.update(&format);
    let ddl_digest = hash_rows(&ddl);
    h.update(ddl_digest.as_bytes());
    for table in [
        "items",
        "revisions",
        "materials",
        "blobs",
        "operations",
        "pending_blobs",
    ] {
        let rows = table_rows(&store.conn, table)?;
        if table == "pending_blobs" && !rows.is_empty() {
            return Err(diagnostic("migration_busy", "pending blob reservations"));
        }
        if table == "operations"
            && rows
                .iter()
                .any(|r| r[3] == rusqlite::types::Value::Text("pending".into()))
        {
            return Err(diagnostic(
                "migration_busy",
                "pending store operations require original-root recovery",
            ));
        }
        if table == "items"
            && rows
                .iter()
                .any(|r| r[2] == rusqlite::types::Value::Integer(0))
        {
            return Err(diagnostic("integrity", "head zero has unresolved history"));
        }
        let statement = store
            .conn
            .prepare(&format!("SELECT * FROM {table} LIMIT 0"))?;
        let columns: Vec<_> = statement
            .column_names()
            .iter()
            .map(|name| name.to_string())
            .collect();
        let mut table_hash = Sha256::new();
        for field in std::iter::once(table).chain(columns.iter().map(String::as_str)) {
            table_hash.update((field.len() as u64).to_be_bytes());
            table_hash.update(field.as_bytes());
        }
        table_hash.update(location::unhex(&hash_rows(&rows))?);
        let digest = location::hex(&table_hash.finalize());
        h.update((table.len() as u64).to_be_bytes());
        h.update(table.as_bytes());
        h.update(location::unhex(&digest)?);
        if table == "blobs" {
            let mut bytes = 0u64;
            for row in &rows {
                let rusqlite::types::Value::Integer(length) = row[1] else {
                    return Err(corrupt("blob length type"));
                };
                bytes = bytes
                    .checked_add(
                        length
                            .try_into()
                            .map_err(|_| corrupt("negative blob length"))?,
                    )
                    .ok_or_else(|| corrupt("blob length overflow"))?;
            }
            blob_summary = json!({"count":rows.len(),"byte_count":bytes.to_string(),"inventory_digest":digest});
        }
        tables.push(json!({"name":table,"columns":columns,"count":rows.len(),"sha256":digest}));
    }
    tx.commit()?;
    db::verify_live(store)?;
    Ok(
        json!({"inventory_version":1,"verifier_version":"xdesign-schema1-v1","store_id":store.store_id,"ddl_sha256":ddl_digest,"format_sha256":sha256(&format),"tables":tables,"blobs":blob_summary,"logical_sha256":location::hex(&h.finalize())}),
    )
}
fn insert_rows(
    conn: &rusqlite::Connection,
    table: &str,
    rows: &[Vec<rusqlite::types::Value>],
) -> Result<(), RsStoreError> {
    for row in rows {
        let sql = format!(
            "INSERT INTO {table} VALUES ({})",
            vec!["?"; row.len()].join(",")
        );
        conn.execute(&sql, rusqlite::params_from_iter(row.iter()))?;
    }
    Ok(())
}
fn copy(
    source: &RsStore,
    destination: &Path,
    lease: &MigrationLease,
) -> Result<RsStore, RsStoreError> {
    lease.lease.verify()?;
    let format = blobs::read_store_file(&source.root.join("format.json"), 0o600)?;
    let target = db::migration_construct(destination, &format, &source.store_id, lease)?;
    checkpoint_ownership(&lease.journal, destination)?;
    fault("database-created");
    let blobs_rows = table_rows(&source.conn, "blobs")?;
    for row in &blobs_rows {
        let rusqlite::types::Value::Text(digest) = &row[0] else {
            return Err(corrupt("blob digest type"));
        };
        let rusqlite::types::Value::Integer(length) = row[1] else {
            return Err(corrupt("blob length type"));
        };
        let from = blobs::blob_path(&source.root, digest)?;
        let to = blobs::blob_path(destination, digest)?;
        location::mkdir(to.parent().unwrap())?;
        let mut input = File::open(&from)?;
        let mut output = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&to)?;
        let copied = std::io::copy(&mut input, &mut output)?;
        if copied != length as u64 {
            return Err(diagnostic("integrity", "blob length changed during copy"));
        }
        output.set_permissions(fs::Permissions::from_mode(0o400))?;
        blobs::sync_durable(&output)?;
        drop(output);
        blobs::fsync_dir(to.parent().unwrap())?;
        blobs::verify_published(destination, digest, length as u64)?;
        checkpoint_ownership(&lease.journal, destination)?;
        fault("blob-published");
    }
    let tx = db::Transaction::begin(&target.conn, true)?;
    insert_rows(
        &target.conn,
        "operations",
        &table_rows(&source.conn, "operations")?,
    )?;
    insert_rows(&target.conn, "blobs", &blobs_rows)?;
    for mut row in table_rows(&source.conn, "items")? {
        row[2] = rusqlite::types::Value::Integer(0);
        insert_rows(&target.conn, "items", &[row])?;
    }
    let materials = table_rows(&source.conn, "materials")?;
    for row in table_rows(&source.conn, "revisions")? {
        insert_rows(&target.conn, "revisions", std::slice::from_ref(&row))?;
        for m in materials
            .iter()
            .filter(|m| m[0] == row[0] && m[1] == row[1])
        {
            insert_rows(&target.conn, "materials", std::slice::from_ref(m))?;
        }
        target.conn.execute(
            "UPDATE items SET head_revision=? WHERE item_id=?",
            rusqlite::params_from_iter([&row[1], &row[0]]),
        )?;
    }
    tx.commit()?;
    checkpoint_ownership(&lease.journal, destination)?;
    fault("import-committed");
    target
        .conn
        .execute_batch("PRAGMA wal_checkpoint(TRUNCATE)")?;
    blobs::sync_durable(&File::open(destination.join("store.db"))?)?;
    blobs::fsync_dir(destination)?;
    Ok(target)
}

fn publish_pointer(
    a: &Attempt,
    root: &Path,
    role: &str,
    identity: Value,
) -> Result<(), RsStoreError> {
    let p = pointer(root)?;
    let previous = if p.exists() {
        Some(location::json_file(&p)?)
    } else {
        None
    };
    let mut attempts = previous
        .as_ref()
        .and_then(|p| p["attempts"].as_array())
        .cloned()
        .unwrap_or_default();
    if let Some(old) = &previous {
        if old["migration_id"] != a.id {
            attempts.push(json!({"migration_id":old["migration_id"],"attempt_nonce":old["attempt_nonce"],"reservation_sha256":old["reservation_sha256"],"request_sha256":old["request_sha256"],"journal_path":old["journal_path"],"journal_identity":old["journal_identity"]}));
        }
    }
    let v = json!({"schema_version":1,"generation":"1","root_path":root,"parent_identity":location::observe(root.parent().unwrap(),None)?,"root_identity":identity,"role":role,"migration_id":a.id,"journal_path":a.journal,"journal_identity":a.reservation["journal_identity"],"request_sha256":a.request_digest(),"attempt_nonce":a.nonce(),"reservation_sha256":a.digest(),"attempts":attempts});
    location::replace_json(&p, &v)
}
fn install_reference(a: &Attempt, root: &Path) -> Result<(), RsStoreError> {
    let marker = location::json_file(&access_dir(root).join("format.json"))?;
    let v = json!({"schema_version":1,"pointer_path":pointer(root)?,"access_epoch":marker["epoch"],"migration_id":a.id,"request_sha256":a.request_digest(),"attempt_nonce":a.nonce(),"reservation_sha256":a.digest()});
    let bytes = serde_json::to_vec(&v).map_err(corrupt)?;
    if let Some(old) = location::attribute(root, location::ACCESS_ATTRIBUTE)? {
        if old == bytes {
            return Ok(());
        }
        let previous = strict_json(&old).map_err(corrupt)?;
        let id = previous["migration_id"]
            .as_str()
            .ok_or_else(|| corrupt("old access reference has no attempt"))?;
        let prior = load_attempt(&a.route, id)?
            .ok_or_else(|| corrupt("old reference reservation missing"))?;
        if prior.decision()?.is_some() || prior.record("abort.json")?.is_none() {
            return Err(corrupt("cannot replace a non-aborted access reference"));
        }
        location::replace_attribute(root, location::ACCESS_ATTRIBUTE, &old, &bytes)
    } else {
        location::install_attribute(root, location::ACCESS_ATTRIBUTE, &bytes)
    }
}
fn outcome(a: &Attempt) -> Result<Value, RsStoreError> {
    if let Some(d) = a.decision()? {
        source_check(a, &d)?;
        return Ok(
            json!({"migration_id":a.id,"phase":"committed","attempt_nonce":a.nonce(),"request_sha256":a.request_digest(),"reservation_sha256":a.digest(),"outcome":"committed","decision":d}),
        );
    }
    if let Some(abort) = a.record("abort.json")? {
        return Ok(
            json!({"migration_id":a.id,"phase":"aborted","attempt_nonce":a.nonce(),"request_sha256":a.request_digest(),"reservation_sha256":a.digest(),"outcome":"aborted","abort":abort}),
        );
    }
    let state = a.state()?;
    Ok(
        json!({"migration_id":a.id,"phase":state.as_ref().map(|s|s["phase"].clone()).unwrap_or(json!("reserved")),"attempt_nonce":a.nonce(),"request_sha256":a.request_digest(),"reservation_sha256":a.digest(),"state":state}),
    )
}
fn resolve_attempt(root: &Path, id: &str) -> Result<(Arc<AccessLease>, Attempt), RsStoreError> {
    let root = access::absolute_root(root)?;
    let lease = access::acquire(&root, None)?;
    let route =
        route(&root)?.ok_or_else(|| diagnostic("not_found", "no permanent migration route"))?;
    let a = match load_attempt(&route, id)? {
        Some(a) => a,
        None => {
            route.verify_setup(false)?;
            return Err(diagnostic("not_found", "no accepted migration reservation"));
        }
    };
    route.verify()?;
    Ok((lease, a))
}
/// Read metadata only; never open an unfinished destination as store history.
pub(crate) fn inspect(root: &Path) -> Result<Value, RsStoreError> {
    let mut data = json!({"authority_state":"absent", "store_id":null, "physical_identity":null, "migrations":[]});
    if let Some(route) = route(root)? {
        route.verify_read(root)?;
        data["source_root"] = json!(route.source());
        data["destination_root"] = json!(route.destination());
        data["binding_key"] = json!(route.binding.key());
        data["binding_sha256"] = json!(route.binding.digest());
        data["namespace_epoch"] = route.binding.payload["namespace_epoch"].clone();
        for a in attempts(&route)? {
            let state = outcome(&a)?;
            data["migrations"]
                .as_array_mut()
                .unwrap()
                .push(state.clone());
            if state["phase"] == "committed" {
                data["authority_state"] = json!(if root == route.source() {
                    "retained_source"
                } else {
                    "committed_destination"
                });
            } else if state["phase"] != "aborted" {
                data["authority_state"] = json!("non_authoritative_preparation");
            }
        }
    }
    if data["authority_state"] == "non_authoritative_preparation" {
        return Ok(data);
    }
    guard(root, false)?;
    if !absent(root)? {
        let store = db::open(root)?;
        data["store_id"] = json!(store.store_id);
        data["physical_identity"] = serde_json::to_value(physical(&store)).map_err(corrupt)?;
        if data["authority_state"] == "absent" {
            data["authority_state"] = json!("unmigrated_store");
        }
    }
    Ok(data)
}

/// Generate the exact public request from verified live observations. This only
/// writes the requested external file; enrollment remains an explicit command.
pub(crate) fn prepare(
    project: &Path,
    id: &str,
    output: &Path,
    older_clients_stopped: bool,
) -> Result<Value, RsStoreError> {
    if !super::valid_id(id) || !older_clients_stopped {
        return Err(diagnostic("invalid_input", "prepare requires a valid migration ID and --older-clients-stopped; stop all older clients first"));
    }
    let ctx = location::context(project)?
        .ok_or_else(|| diagnostic("invalid_input", "migration requires a linked worktree"))?;
    let binding = location::binding_for_project(&ctx, &location::account_home()?)?
        .ok_or_else(|| diagnostic("binding_missing", "run store location enroll first"))?;
    let source_root = binding.project().join(".xdesign");
    let _lease = access::acquire(&source_root, Some(project))?;
    binding.verify()?;
    let route = route(&source_root)?
        .ok_or_else(|| corrupt("binding has no permanent routes; retry its enrollment"))?;
    route.verify()?;
    // A published reservation owns the original bytes forever, even if its
    // source has since disappeared. Never mint a different retry request.
    if let Some(a) = load_attempt(&route, id)? {
        let bytes = location::unhex(
            a.reservation["request_bytes"]
                .as_str()
                .ok_or_else(|| corrupt("request bytes"))?,
        )?;
        return output_request(&route, id, output, &bytes);
    }
    for a in attempts(&route)? {
        if a.decision()?.is_some() {
            return Err(diagnostic(
                "migration_committed",
                "use status/request for the committed migration ID",
            ));
        }
        if a.record("abort.json")?.is_none() || a.record("discard.json")?.is_none() {
            return Err(diagnostic("migration_busy", "recover the existing attempt with status/request, or abort and discard before a new ID"));
        }
    }
    if !absent(&binding.root())? {
        return Err(diagnostic(
            "history_conflict",
            "destination is occupied; adoption and merge are not supported",
        ));
    }
    guard(&source_root, false)?;
    let store = db::open(&source_root)?;
    inventory(&store)?;
    let request = Request {
        schema_version: 1,
        migration_id: id.to_owned(),
        project_root: binding.project(),
        binding_key: binding.key().to_owned(),
        binding_sha256: binding.digest(),
        source_root,
        source_store_id: store.store_id.clone(),
        source_identity: physical(&store),
        destination_root: binding.root(),
        older_clients_stopped,
    };
    let bytes = serde_json::to_vec_pretty(&request).map_err(corrupt)?;
    parse_request(&bytes)?;
    output_request(&route, id, output, &bytes)
}
fn output_request(
    route: &Route,
    id: &str,
    output: &Path,
    bytes: &[u8],
) -> Result<Value, RsStoreError> {
    let path = blobs::canonical_for_creation(output)?;
    if path.starts_with(route.source())
        || path.starts_with(route.destination())
        || path.starts_with(access_dir(&route.source()))
        || path.starts_with(&route.binding.namespace)
    {
        return Err(diagnostic(
            "unsafe_path",
            "request output cannot be inside a store or its coordination namespace",
        ));
    }
    let output = blobs::write_new_external(output, bytes)?;
    Ok(
        json!({"migration_id":id,"request_sha256":sha256(bytes),"byte_length":bytes.len(),"output":output,"source_root":route.source(),"destination_root":route.destination(),"next_commands":[["xdesign","store","migrate","apply","--request",output]]}),
    )
}

pub(crate) fn status(root: &Path, id: &str) -> Result<Value, RsStoreError> {
    let (_lease, a) = resolve_attempt(root, id)?;
    outcome(&a)
}
pub(crate) fn request_bytes(root: &Path, id: &str) -> Result<Vec<u8>, RsStoreError> {
    let (_lease, a) = resolve_attempt(root, id)?;
    location::unhex(
        a.reservation["request_bytes"]
            .as_str()
            .ok_or_else(|| corrupt("request bytes"))?,
    )
}
pub(crate) fn request_output(root: &Path, id: &str, output: &Path) -> Result<Value, RsStoreError> {
    let (_lease, a) = resolve_attempt(root, id)?;
    let bytes = location::unhex(
        a.reservation["request_bytes"]
            .as_str()
            .ok_or_else(|| corrupt("request bytes"))?,
    )?;
    let path = blobs::canonical_for_creation(output)?;
    if path.starts_with(a.route.source())
        || path.starts_with(a.route.destination())
        || path.starts_with(access_dir(&a.route.source()))
        || path.starts_with(a.route.binding.namespace.clone())
    {
        return Err(diagnostic(
            "unsafe_path",
            "recovery output cannot be inside a store or its coordination namespace",
        ));
    }
    let output = blobs::write_new_external(output, &bytes)?;
    Ok(
        json!({"migration_id":id,"request_sha256":sha256(&bytes),"byte_length":bytes.len(),"output":output}),
    )
}
pub(crate) fn abort(root: &Path, id: &str) -> Result<Value, RsStoreError> {
    let (_lease, a) = resolve_attempt(root, id)?;
    if a.decision()?.is_some() {
        return Err(diagnostic(
            "migration_committed",
            "a cutover decision cannot be aborted",
        ));
    }
    if a.record("abort.json")?.is_none() {
        let mut v = a.common();
        v["outcome"] = json!("aborted");
        v["invalidated_generation"] = json!("all");
        v["residue_identity"] = a
            .state()?
            .map(|s| s["destination_identity"].clone())
            .unwrap_or(Value::Null);
        location::publish_json(&a.journal.join("abort.json"), &v)?;
    }
    outcome(&a)
}
pub(crate) fn discard(root: &Path, id: &str) -> Result<Value, RsStoreError> {
    let (_lease, a) = resolve_attempt(root, id)?;
    if a.decision()?.is_some() {
        return Err(diagnostic(
            "migration_committed",
            "committed history cannot be discarded",
        ));
    }
    let abort = a.record("abort.json")?.ok_or_else(|| {
        diagnostic(
            "migration_busy",
            "abort the attempt before disposing prepared bytes",
        )
    })?;
    if !absent(&a.route.destination())? {
        // Removing anything requires a complete per-object ownership inventory. A
        // partial constructor with no such evidence is preserved for inspection.
        if let Some(verification) = a.record("verification.json")? {
            verify_object(
                &verification["destination_identity"],
                &a.route.destination(),
                false,
            )?;
            let target = db::open(&a.route.destination())?;
            if inventory(&target)? != verification["destination_inventory"] {
                return Err(diagnostic(
                    "identity_unprovable",
                    "prepared target changed; preserved",
                ));
            }
            drop(target);
            let owned = tree_objects(&a.route.destination())?;
            if owned != verification["owned_objects"] {
                return Err(diagnostic(
                    "identity_unprovable",
                    "prepared target ownership changed; preserved",
                ));
            }
            remove_owned_contents(&a.route.destination(), &verification["owned_objects"])?;
        } else {
            let state = a.state()?.ok_or_else(|| {
                diagnostic(
                    "identity_unprovable",
                    "partial target lacks recorded ownership",
                )
            })?;
            let owned = &state["diagnostics"]["owned_objects"];
            if owned.is_null() {
                return Err(diagnostic(
                    "identity_unprovable",
                    "partial target lacks proven per-object ownership; preserved",
                ));
            }
            remove_owned_contents(&a.route.destination(), owned)?;
        }
        fs::remove_dir(a.route.destination())?;
        blobs::fsync_dir(a.route.destination().parent().unwrap())?;
    }
    let mut v = a.common();
    v["discarded"] = json!(true);
    location::publish_json(&a.journal.join("discard.json"), &v)?;
    Ok(json!({"migration_id":a.id,"outcome":"discarded","discard":v,"abort":abort}))
}
fn tree_objects(root: &Path) -> Result<Value, RsStoreError> {
    let mut pending = vec![root.to_owned()];
    let mut entries = vec![];
    while let Some(p) = pending.pop() {
        let m = fs::symlink_metadata(&p)?;
        if m.file_type().is_symlink() {
            return Err(diagnostic("unsafe_path", "prepared target symlink"));
        }
        let name = p
            .strip_prefix(root)
            .unwrap()
            .to_str()
            .ok_or_else(|| corrupt("non-UTF-8 target entry"))?
            .to_owned();
        if m.is_file() && std::os::unix::fs::MetadataExt::nlink(&m) != 1 {
            return Err(diagnostic("unsafe_path", "hard-linked migration object"));
        }
        entries.push(
            json!({"path":name,"identity":location::observe(&p,None)?,"directory":m.is_dir()}),
        );
        if m.is_dir() {
            for e in fs::read_dir(p)? {
                pending.push(e?.path());
            }
        }
    }
    entries.sort_by(|a, b| a["path"].as_str().cmp(&b["path"].as_str()));
    Ok(json!(entries))
}
fn reserve(
    route: &Route,
    request: &Request,
    raw: &[u8],
    source: &RsStore,
) -> Result<Attempt, RsStoreError> {
    let journal = route.base().join(&request.migration_id);
    location::mkdir(&journal)?;
    if fs::read_dir(&journal)?.next().is_some() {
        return Err(corrupt("new attempt slot is occupied"));
    }
    let payload = json!({"schema_version":1,"namespace_epoch":route.binding.payload["namespace_epoch"],"owner_uid":route.binding.payload["owner_uid"],"migration_id":request.migration_id,"attempt_nonce":location::random()?,"binding_key":route.binding.key(),"binding_sha256":route.binding.digest(),"request_bytes":location::hex(raw),"request_sha256":sha256(raw),"route_sha256":route.digest(),"journal_identity":location::observe(&journal,None)?,"source_root_identity":location::observe(&source.root,None)?,"source_database_identity":location::observe(&source.root.join("store.db"),None)?});
    let record = location::sign(&route.binding.key_bytes, "reservation", 8, payload)?;
    let bytes = serde_json::to_vec(&record).map_err(corrupt)?;
    let pool = access_dir(&route.destination()).join("reservation-drafts");
    let draft = pool.join(location::random()?);
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&draft)?;
    file.write_all(&bytes)?;
    fault("reservation-draft-written");
    blobs::sync_durable(&file)?;
    drop(file);
    fault("reservation-draft-synced");
    location::rename_exclusive(&draft, &journal.join("reservation.json"))
        .map_err(|e| diagnostic("migration_outcome_unknown", e))?;
    fault("reservation-published");
    for p in [&pool, &journal, &route.base()] {
        blobs::fsync_dir(p).map_err(|e| diagnostic("migration_outcome_unknown", e))?;
    }
    load_attempt(route, &request.migration_id)?.ok_or_else(|| {
        diagnostic(
            "migration_outcome_unknown",
            "reservation publication could not be confirmed",
        )
    })
}
pub(crate) fn apply(raw: &[u8]) -> Result<Value, RsStoreError> {
    let request = parse_request(raw)?;
    let ns = request
        .destination_root
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .ok_or_else(|| diagnostic("invalid_input", "invalid binding destination"))?;
    let binding = location::load_binding(ns, &request.binding_key)?;
    binding.verify()?;
    if request.project_root != binding.project()
        || request.source_root != binding.project().join(".xdesign")
        || request.destination_root != binding.root()
        || request.binding_sha256 != binding.digest()
    {
        return Err(diagnostic(
            "invalid_input",
            "request does not name the enrolled project and exact roots",
        ));
    }
    install_routes(&binding)?;
    let lease = access::acquire(&request.source_root, None)?;
    let private = MigrationLease {
        lease,
        journal: route(&request.source_root)?
            .ok_or_else(|| corrupt("missing route"))?
            .base()
            .join(&request.migration_id),
    };
    let route = route(&request.source_root)?.ok_or_else(|| corrupt("missing permanent route"))?;
    route.verify()?;
    let all = attempts(&route)?;
    for a in &all {
        if a.id != request.migration_id {
            if a.decision()?.is_some() {
                return Err(diagnostic(
                    "migration_committed",
                    "binding already has a cutover decision",
                ));
            }
            if a.record("abort.json")?.is_none() || a.record("discard.json")?.is_none() {
                return Err(diagnostic(
                    "migration_busy",
                    "another attempt must finish or be aborted and discarded",
                ));
            }
        }
    }
    let existing = load_attempt(&route, &request.migration_id)?;
    if let Some(a) = &existing {
        if location::unhex(a.reservation["request_bytes"].as_str().unwrap())? != raw {
            return Err(diagnostic(
                "operation_mismatch",
                "same migration ID has different request bytes",
            ));
        }
        // Decide cutover before checking source presence. A durable decision remains
        // the answer when the retained source has subsequently been removed.
        if a.decision()?.is_some() {
            return outcome(a);
        }
        if a.record("abort.json")?.is_some() {
            return Err(diagnostic(
                "migration_aborted",
                "original attempt was aborted; status returns its retained outcome",
            ));
        }
    }
    if absent(&request.source_root)? {
        return Err(diagnostic(
            "identity_unprovable",
            "source is absent before cutover",
        ));
    }
    let source = db::open(&request.source_root)?;
    if physical(&source) != request.source_identity || source.store_id != request.source_store_id {
        return Err(diagnostic(
            "identity_unprovable",
            "source identity does not match request",
        ));
    }
    let before = inventory(&source)?;
    let a = match existing {
        Some(a) => a,
        None => {
            if !absent(&request.destination_root)? {
                return Err(diagnostic(
                    "history_conflict",
                    "destination is occupied; adoption and merge are not supported",
                ));
            }
            reserve(&route, &request, raw, &source)?
        }
    };
    fault("reserved");
    location::publish(&a.journal.join("request.json"), raw)?;
    let state = a.state()?;
    if !absent(&request.destination_root)? && state.is_none() {
        if fs::read_dir(&request.destination_root)?.next().is_some() {
            return Err(diagnostic(
                "identity_unprovable",
                "unowned destination residue",
            ));
        }
        blobs::store_dir(&request.destination_root)?;
    }
    publish_pointer(
        &a,
        &request.destination_root,
        "destination",
        state
            .as_ref()
            .map(|s| s["destination_identity"].clone())
            .unwrap_or(Value::Null),
    )?;
    fault("destination-pointer");
    if absent(&request.destination_root)? {
        location::mkdir(&request.destination_root)?;
        fault("destination-allocated");
    }
    if state.is_none() {
        a.set_state(
            "reserved",
            Some(
                json!({"root":location::observe(&request.destination_root,None)?,"database":null}),
            ),
        )?;
    }
    let source_identity = object_pair(&request.source_root)?;
    publish_pointer(&a, &request.source_root, "source", source_identity.clone())?;
    fault("source-pointer");
    install_reference(&a, &request.source_root)?;
    install_reference(&a, &request.destination_root)?;
    fault("fenced");
    if request.destination_root.join("store.db").exists()
        && a.record("verification.json")?.is_none()
    {
        let state = a
            .state()?
            .ok_or_else(|| corrupt("partial target has no state"))?;
        let owned = state["diagnostics"]["owned_objects"].clone();
        if owned.is_null() {
            return Err(diagnostic(
                "identity_unprovable",
                "partial database lacks proven ownership; preserve and inspect",
            ));
        }
        remove_owned_contents(&request.destination_root, &owned)?;
    }
    if request.destination_root.join("store.db").exists() {
        let verification = a
            .record("verification.json")?
            .ok_or_else(|| corrupt("missing verification"))?;
        verify_object(
            &verification["destination_identity"],
            &request.destination_root,
            false,
        )?;
        let target = db::open(&request.destination_root)?;
        if inventory(&target)? != verification["destination_inventory"]
            || before != verification["source_inventory"]
        {
            return Err(diagnostic(
                "source_changed",
                "source or prepared target changed; new baseline required",
            ));
        }
    } else {
        a.set_state(
            "fenced",
            Some(
                json!({"root":location::observe(&request.destination_root,None)?,"database":null}),
            ),
        )?;
        let mut baseline = a.common();
        baseline["inventory"] = before.clone();
        baseline["source_identity"] = source_identity.clone();
        location::publish_json(&a.journal.join("baseline.json"), &baseline)?;
        a.set_state(
            "preparing",
            Some(
                json!({"root":location::observe(&request.destination_root,None)?,"database":null}),
            ),
        )?;
        let target = copy(&source, &request.destination_root, &private)?;
        let target_inventory = inventory(&target)?;
        if target_inventory != before {
            return Err(diagnostic(
                "integrity",
                "destination logical inventory differs",
            ));
        }
        drop(target);
        let dest_identity = object_pair(&request.destination_root)?;
        let mut verification = a.common();
        verification["source_inventory"] = before.clone();
        verification["destination_inventory"] = target_inventory;
        verification["destination_identity"] = dest_identity.clone();
        verification["owned_objects"] = tree_objects(&request.destination_root)?;
        location::publish_json(&a.journal.join("verification.json"), &verification)?;
        a.set_state("prepared", Some(dest_identity))?;
        fault("prepared");
    }
    if inventory(&source)? != before {
        return Err(diagnostic(
            "source_changed",
            "source changed before decision",
        ));
    }
    verify_object(&source_identity, &request.source_root, true)?;
    private.lease.verify()?;
    a.route.verify()?;
    let verification = a
        .record("verification.json")?
        .ok_or_else(|| corrupt("missing verification"))?;
    let mut decision = a.common();
    decision["binding_key"] = json!(binding.key());
    decision["binding_sha256"] = json!(binding.digest());
    decision["source_identity"] = source_identity;
    decision["destination_identity"] = verification["destination_identity"].clone();
    decision["source_baseline_sha256"] = before["logical_sha256"].clone();
    decision["verification_sha256"] = json!(sha256(&blobs::read_store_file(
        &a.journal.join("verification.json"),
        0o600
    )?));
    decision["authority_root"] = json!(request.destination_root);
    decision["outcome"] = json!("committed");
    fault("before-decision");
    location::publish_json(&a.journal.join("decision.json"), &decision)
        .map_err(|e| diagnostic("migration_outcome_unknown", e))?;
    fault("decision-durable");
    outcome(&a)
}

pub(crate) fn initialize(root: &Path) -> Result<RsStore, RsStoreError> {
    let Some(route) = route(root)? else {
        guard(root, true)?;
        return db::init(root);
    };
    route.verify()?;
    if root != route.destination() {
        guard(root, true)?;
        return db::init(root);
    }
    let init = root.parent().unwrap().join("initialization.json");
    let mut v = if !absent(&init)? && attempts(&route)?.is_empty() && absent(&route.source())? {
        let value = initialization_record(&route.binding)?;
        if value["schema_version"] != 1
            || value["binding_key"] != route.binding.key()
            || value["binding_sha256"] != route.binding.digest()
            || !value["initialization_id"]
                .as_str()
                .is_some_and(super::valid_digest)
        {
            return Err(corrupt("initialization journal differs from binding"));
        }
        if value["phase"] == "verified" {
            verify_object(&initialization_identity(&value), root, false)?;
            location::install_attribute(
                root,
                location::ACCESS_ATTRIBUTE,
                &serde_json::to_vec(&initialization_reference(&route.binding, &value))
                    .map_err(corrupt)?,
            )?;
            guard(root, true)?;
            return db::open(root);
        }
        if value["phase"] != "reserved" || !absent(root)? {
            return Err(diagnostic(
                "identity_unprovable",
                "initialization has unproved residue; preserve it for inspection",
            ));
        }
        value
    } else {
        select(&route.binding, !absent(&route.source())?)?;
        if !absent(root)? {
            guard(root, true)?;
            return db::open(root);
        }
        let value = json!({"schema_version":1,"binding_key":route.binding.key(),"binding_sha256":route.binding.digest(),"initialization_id":location::random()?,"phase":"reserved","root_identity":null,"database_identity":null,"store_id":null});
        location::publish_json(&init, &value)?;
        value
    };
    fault("initialization-reserved");
    let store = db::init(root)?;
    fault("initialization-constructed");
    v["phase"] = json!("verified");
    let physical = object_pair(root)?;
    v["root_identity"] = physical["root"].clone();
    v["database_identity"] = physical["database"].clone();
    v["store_id"] = json!(store.store_id);
    location::replace_json(&init, &v)?;
    fault("initialization-verified");
    location::install_attribute(
        root,
        location::ACCESS_ATTRIBUTE,
        &serde_json::to_vec(&initialization_reference(&route.binding, &v)).map_err(corrupt)?,
    )?;
    fault("initialization-witness-installed");
    Ok(store)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn git(cwd: &Path, args: &[&str]) {
        let o = std::process::Command::new("git")
            .current_dir(cwd)
            .args(args)
            .output()
            .unwrap();
        assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    }
    #[test]
    fn real_worktree_migration_preserves_history_and_receipts() {
        let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("cache/implementation/worktree-migration")
            .join(uuid::Uuid::new_v4().to_string());
        fs::create_dir_all(&base).unwrap();
        let home = base.join("home");
        let primary = base.join("primary");
        let worktree = base.join("linked");
        for p in [&home, &primary] {
            fs::create_dir(p).unwrap();
        }
        git(&primary, &["init", "-q"]);
        git(
            &primary,
            &[
                "-c",
                "user.name=Fixture",
                "-c",
                "user.email=fixture@example.invalid",
                "commit",
                "--allow-empty",
                "-qm",
                "fixture",
            ],
        );
        git(
            &primary,
            &[
                "worktree",
                "add",
                "-q",
                "--detach",
                worktree.to_str().unwrap(),
            ],
        );
        assert!(location::context(&primary).unwrap().is_none());
        let ctx = location::context(&worktree).unwrap().unwrap();
        let root = worktree.join(".xdesign");
        if let Some(old) = std::env::var_os("XDESIGN_TEST_OLD_BINARY") {
            let out = std::process::Command::new(old)
                .current_dir(&worktree)
                .args(["store", "init", "--root"])
                .arg(&root)
                .output()
                .unwrap();
            assert!(
                out.status.success(),
                "{}",
                String::from_utf8_lossy(&out.stderr)
            );
        }
        let store = RsStore::init(&root).unwrap();
        fs::write(worktree.join("bytes.txt"), b"historical bytes").unwrap();
        let raw=serde_json::to_vec_pretty(&json!({"schema_version":1,"operation_id":"old-save","item_id":"generic-record","kind":"record","expected_revision":0,"metadata":{"old":true},"materials":[{"uri":"evidence/old.txt","role":"evidence","sha256":sha256(b"historical bytes"),"source":"bytes.txt"}],"result":{"ok":"old result"}})).unwrap();
        let old = std::env::var_os("XDESIGN_TEST_OLD_BINARY").map(PathBuf::from);
        if let Some(old) = &old {
            let req = worktree.join("old-request.json");
            fs::write(&req, &raw).unwrap();
            let out = std::process::Command::new(old)
                .current_dir(&worktree)
                .args(["store", "save", "--root"])
                .arg(&root)
                .arg("--request")
                .arg(req)
                .output()
                .unwrap();
            assert!(
                out.status.success(),
                "{}",
                String::from_utf8_lossy(&out.stderr)
            );
        }
        let receipt = store.save_bytes(&worktree, &raw).unwrap();
        let mut conflict: Value = serde_json::from_slice(&raw).unwrap();
        conflict["operation_id"] = json!("old-conflict");
        let conflict = serde_json::to_vec_pretty(&conflict).unwrap();
        if let Some(old) = &old {
            let req = worktree.join("old-conflict.json");
            fs::write(&req, &conflict).unwrap();
            let out = std::process::Command::new(old)
                .current_dir(&worktree)
                .args(["store", "save", "--root"])
                .arg(&root)
                .arg("--request")
                .arg(req)
                .output()
                .unwrap();
            assert_eq!(out.status.code(), Some(4));
        }
        let old_conflict = store.save_bytes(&worktree, &conflict).unwrap();
        for rev in 1..3 {
            let bytes = format!("historical revision {rev}");
            fs::write(worktree.join("bytes.txt"), bytes.as_bytes()).unwrap();
            let mut request: Value = serde_json::from_slice(&raw).unwrap();
            request["operation_id"] = json!(format!("history-{rev}"));
            request["expected_revision"] = json!(rev);
            request["materials"][0]["sha256"] = json!(sha256(bytes.as_bytes()));
            let request = serde_json::to_vec_pretty(&request).unwrap();
            if let Some(old) = &old {
                let req = worktree.join(format!("history-{rev}.json"));
                fs::write(&req, &request).unwrap();
                let out = std::process::Command::new(old)
                    .current_dir(&worktree)
                    .args(["store", "save", "--root"])
                    .arg(&root)
                    .arg("--request")
                    .arg(req)
                    .output()
                    .unwrap();
                assert!(
                    out.status.success(),
                    "{}",
                    String::from_utf8_lossy(&out.stderr)
                );
            } else {
                store.save_bytes(&worktree, &request).unwrap();
            }
        }
        let binding = location::enroll(&ctx, &home, "enrollment-test", None).unwrap();
        let request = json!({"schema_version":1,"migration_id":"migration-test","project_root":worktree,"binding_key":binding.key(),"binding_sha256":binding.digest(),"source_root":root,"source_store_id":store.store_id,"source_identity":physical(&store),"destination_root":binding.root(),"older_clients_stopped":true});
        let request = serde_json::to_vec_pretty(&request).unwrap();
        if let Some(export) = std::env::var_os("XDESIGN_TEST_EXPORT_FIXTURE") {
            let export = PathBuf::from(export);
            let allowed = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("cache")
                .canonicalize()
                .unwrap();
            assert!(export
                .parent()
                .unwrap()
                .canonicalize()
                .unwrap()
                .starts_with(allowed));
            if std::env::var_os("XDESIGN_TEST_REMOVE_ROUTES").is_some() {
                fs::remove_dir_all(access_dir(&root)).unwrap();
                fs::remove_dir_all(access_dir(&binding.root())).unwrap();
            }
            fs::write(&export, &request).unwrap();
            println!("fixture_request={}", export.display());
            return;
        }
        let result = apply(&request).unwrap();
        assert_eq!(result["outcome"], "committed");
        assert_eq!(apply(&request).unwrap(), result);
        assert_eq!(status(&root, "migration-test").unwrap(), result);
        assert_eq!(
            request_bytes(&binding.root(), "migration-test").unwrap(),
            request
        );
        assert_eq!(
            access::reason(&store.save_bytes(&worktree, &raw).unwrap_err()),
            Some("source_retired")
        );
        let target = RsStore::open(&binding.root()).unwrap();
        assert_eq!(target.outcome("old-save").unwrap(), receipt);
        assert_eq!(target.outcome("old-conflict").unwrap(), old_conflict);
        assert_eq!(
            super::super::cli::request_bytes(&target, "old-save")
                .unwrap()
                .unwrap(),
            raw
        );
        assert_eq!(
            target
                .read_material("generic-record", Some(1), "evidence/old.txt")
                .unwrap(),
            b"historical bytes"
        );
        assert_eq!(
            target
                .save_bytes(&base.join("missing-project"), &raw)
                .unwrap(),
            receipt
        );
        assert_eq!(
            store.history("generic-record").unwrap(),
            target.history("generic-record").unwrap()
        );
        assert_eq!(
            access::reason(&abort(&root, "migration-test").unwrap_err()),
            Some("migration_committed")
        );
        // Moving the checkout retires its default binding, but the unchanged
        // global history remains explicitly inspectable through held/fresh APIs.
        let moved = base.join("moved-linked");
        git(
            &primary,
            &[
                "worktree",
                "move",
                worktree.to_str().unwrap(),
                moved.to_str().unwrap(),
            ],
        );
        assert_eq!(target.history("generic-record").unwrap().len(), 3);
        let archive = RsStore::open(&binding.root()).unwrap();
        assert_eq!(archive.outcome("old-save").unwrap(), receipt);
        assert!(archive.save_bytes(&moved, &raw).is_err());
        assert!(binding.verify().is_err());
    }
}

fn checkpoint_ownership(journal: &Path, root: &Path) -> Result<(), RsStoreError> {
    let path = journal.join("state.json");
    let mut state = location::json_file(&path)?;
    state["diagnostics"] = json!({"owned_objects":tree_objects(root)?});
    location::replace_json(&path, &state)
}
fn remove_owned_contents(root: &Path, owned: &Value) -> Result<(), RsStoreError> {
    let actual = tree_objects(root)?;
    let expected = owned
        .as_array()
        .ok_or_else(|| corrupt("missing ownership inventory"))?;
    for entry in actual.as_array().unwrap() {
        if !expected.contains(entry) {
            return Err(diagnostic(
                "identity_unprovable",
                "unrecognized or replaced prepared object; preserved",
            ));
        }
    }
    // Validate transient SQLite sidecars through the native closed-root checker
    // before deleting them along with an explicitly invalidated partial database.
    if root.join("store.db").exists() {
        let checked = db::open(root)?;
        drop(checked);
    }
    let mut objects = actual.as_array().unwrap().clone();
    objects.sort_by_key(|v| std::cmp::Reverse(v["path"].as_str().unwrap().len()));
    for v in objects {
        let relative = v["path"].as_str().unwrap();
        if relative.is_empty() {
            continue;
        }
        let p = root.join(relative);
        if ["store.db-wal", "store.db-shm"].contains(&relative) && absent(&p)? {
            continue;
        }
        if v["directory"] == true {
            fs::remove_dir(p)?;
        } else {
            fs::remove_file(p)?;
        }
    }
    for sidecar in ["store.db-wal", "store.db-shm"] {
        let p = root.join(sidecar);
        if p.exists() {
            return Err(diagnostic(
                "identity_unprovable",
                format!("unrecorded SQLite sidecar {}; preserved", p.display()),
            ));
        }
    }
    blobs::fsync_dir(root)
}
#[cfg(not(test))]
pub(crate) fn fault(_point: &str) {}
#[cfg(not(test))]
fn failure(_point: &str) -> Result<(), RsStoreError> {
    Ok(())
}
#[cfg(test)]
fn failure(point: &str) -> Result<(), RsStoreError> {
    if std::env::var("XDESIGN_TEST_MIGRATION_IO_FAILURE").as_deref() == Ok(point) {
        return Err(diagnostic(
            "migration_outcome_unknown",
            format!("injected {point} I/O failure"),
        ));
    }
    Ok(())
}
#[cfg(test)]
pub(crate) fn fault(point: &str) {
    if std::env::var("XDESIGN_TEST_MIGRATION_CRASH").as_deref() == Ok(point) {
        std::process::exit(86);
    }
}

#[cfg(test)]
#[test]
fn migration_process_driver() {
    let Ok(path) = std::env::var("XDESIGN_TEST_MIGRATION_REQUEST") else {
        return;
    };
    let path = PathBuf::from(path).canonicalize().unwrap();
    assert!(path.starts_with(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("cache")
            .canonicalize()
            .unwrap()
    ));
    let raw = fs::read(path).unwrap();
    let request = match parse_request(&raw) {
        Ok(request) => request,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(87);
        }
    };
    let root = if std::env::var("XDESIGN_TEST_MIGRATION_SIDE").as_deref() == Ok("destination") {
        &request.destination_root
    } else {
        &request.source_root
    };
    let action = std::env::var("XDESIGN_TEST_MIGRATION_ACTION").unwrap_or_else(|_| "apply".into());
    let result = match action.as_str() {
        "initialize" => {
            RsStore::init(&request.destination_root).map(|store| json!({"store_id":store.store_id}))
        }
        "apply" => apply(&raw),
        "status" => status(root, &request.migration_id),
        "abort" => abort(root, &request.migration_id),
        "discard" => discard(root, &request.migration_id),
        "request" => request_bytes(root, &request.migration_id)
            .map(|bytes| json!({"request_hex":location::hex(&bytes)})),
        _ => panic!("unknown fixture action"),
    };
    match result {
        Ok(v) => println!("{}", v),
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(87);
        }
    }
}

fn initialization_reference(binding: &Binding, journal: &Value) -> Value {
    json!({"schema_version":1,"initialization_path":binding.root().parent().unwrap().join("initialization.json"),"binding_key":binding.key(),"binding_sha256":binding.digest(),"initialization_id":journal["initialization_id"]})
}
fn initialization_record(binding: &Binding) -> Result<Value, RsStoreError> {
    let value = location::json_file(&binding.root().parent().unwrap().join("initialization.json"))?;
    let fields = [
        "schema_version",
        "binding_key",
        "binding_sha256",
        "initialization_id",
        "phase",
        "root_identity",
        "database_identity",
        "store_id",
    ];
    if !value
        .as_object()
        .is_some_and(|o| o.len() == fields.len() && o.keys().all(|k| fields.contains(&k.as_str())))
        || value["schema_version"] != 1
        || value["binding_key"] != binding.key()
        || value["binding_sha256"] != binding.digest()
        || !value["initialization_id"]
            .as_str()
            .is_some_and(super::valid_digest)
        || !["reserved", "creating", "verified"]
            .iter()
            .any(|phase| value["phase"] == *phase)
    {
        return Err(corrupt("invalid initialization journal"));
    }
    for field in ["root_identity", "database_identity"] {
        if !value[field].is_null() {
            serde_json::from_value::<location::Object>(value[field].clone()).map_err(corrupt)?;
        }
    }
    if value["phase"] == "verified"
        && (value["root_identity"].is_null()
            || value["database_identity"].is_null()
            || value["store_id"]
                .as_str()
                .and_then(|s| uuid::Uuid::parse_str(s).ok())
                .is_none())
    {
        return Err(corrupt("verified initialization lacks physical identity"));
    }
    Ok(value)
}
fn initialization_identity(value: &Value) -> Value {
    json!({"root":value["root_identity"],"database":value["database_identity"]})
}

fn adjacent_binding(root: &Path) -> Result<Option<Binding>, RsStoreError> {
    let parent = root.parent().ok_or_else(|| corrupt("missing parent"))?;
    if absent(&parent.join("binding.json"))? {
        return Ok(None);
    }
    let key = parent
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| corrupt("binding parent key"))?;
    let ns = parent
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| corrupt("binding namespace path"))?;
    Ok(Some(location::load_binding(ns, key)?))
}
pub(crate) fn observation(root: &Path) -> Result<String, RsStoreError> {
    let mut hash = Sha256::new();
    hash.update(b"xdesign-access-probe-v1");
    let mut paths = vec![
        pointer(root)?,
        access_dir(root).join("format.json"),
        root.parent().unwrap().join("initialization.json"),
    ];
    if let Some(r) = route(root)? {
        paths.extend([
            pointer(&r.source())?,
            pointer(&r.destination())?,
            r.binding.namespace.join("namespace.json"),
            r.binding.namespace.join("key"),
            r.binding.root().parent().unwrap().join("binding.json"),
        ]);
        hash.update(&r.bytes);
        let entries = match fs::read_dir(r.base()) {
            Ok(entries) => Some(entries),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
            Err(e) => return Err(e.into()),
        };
        for entry in entries.into_iter().flatten() {
            let entry = entry?;
            location::safe_dir(&entry.path())?;
            paths.push(entry.path());
            for record in fs::read_dir(entry.path())? {
                paths.push(record?.path());
            }
        }
    }
    paths.sort();
    paths.dedup();
    for path in paths {
        hash.update(path.as_os_str().as_bytes());
        match fs::symlink_metadata(&path) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => hash.update([0]),
            Err(e) => return Err(io_error(e)),
            Ok(m) => {
                let identity = blobs::identity(&m);
                hash.update(identity.0.to_be_bytes());
                hash.update(identity.1.to_be_bytes());
                if m.file_type().is_symlink() {
                    return Err(corrupt("symbolic coordination record"));
                }
                if m.is_file() {
                    hash.update(blobs::read_store_file(&path, 0o600)?);
                } else if !m.is_dir() {
                    return Err(corrupt("special coordination entry"));
                }
            }
        }
    }
    if !absent(root)? {
        if let Some(attr) = location::attribute(root, location::ACCESS_ATTRIBUTE)? {
            hash.update(attr);
        }
    }
    Ok(location::hex(&hash.finalize()))
}
