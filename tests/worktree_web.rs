use std::{
    fs,
    io::{BufRead, BufReader, Read, Write},
    net::TcpStream,
    path::PathBuf,
    process::{Command, Stdio},
};
fn request(address: &str, method: &str, path: &str, body: &str) -> (u16, serde_json::Value) {
    let mut stream = TcpStream::connect(address).unwrap();
    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(30)))
        .unwrap();
    write!(stream,"{method} {path} HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len()).unwrap();
    let mut raw = String::new();
    stream.read_to_string(&mut raw).unwrap();
    let (headers, body) = raw.split_once("\r\n\r\n").unwrap();
    (
        headers.split_whitespace().nth(1).unwrap().parse().unwrap(),
        serde_json::from_str(body).unwrap(),
    )
}
#[test]
fn actual_web_child_uses_explicit_store_instead_of_inherited_root() {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("cache/implementation/worktree-web")
        .join(uuid::Uuid::new_v4().to_string());
    fs::create_dir_all(&p).unwrap();
    let chosen = p.join("chosen");
    let inherited = p.join("wrong");
    let mut child = Command::new(env!("CARGO_BIN_EXE_xdesign"))
        .args(["web", "--port", "0", "--project-root"])
        .arg(&p)
        .arg("--store-root")
        .arg(&chosen)
        .env("XDESIGN_STORE_ROOT", &inherited)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let stdout = child.stdout.take().unwrap();
    let mut reader = BufReader::new(stdout);
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    let address = line
        .trim()
        .strip_prefix("xdesign web listening on http://")
        .unwrap()
        .trim_end_matches('/')
        .to_owned();
    let body = r#"{"wish_id":"web-child","title":"Child root selection","description":"Explicit Web root reaches the child","flow_id":"wish-verify"}"#;
    let result = (|| {
        let mut stream = TcpStream::connect(&address).unwrap();
        stream
            .set_read_timeout(Some(std::time::Duration::from_secs(30)))
            .unwrap();
        write!(stream,"POST /api/wishes HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",body.len(),body).unwrap();
        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();
        response
    })();
    assert!(
        result.starts_with("HTTP/1.1 201") || result.starts_with("HTTP/1.1 200"),
        "{result}"
    );
    let (code, health) = request(&address, "GET", "/api/health", "");
    assert_eq!(code, 200);
    assert_eq!(health["data"]["store_root"], chosen.to_str().unwrap());
    for path in ["/api/wishes/web-child", "/api/wishes/web-child/work"] {
        let (code, value) = request(&address, "GET", path, "");
        assert_eq!(code, 200, "{value}");
    }
    let (code, checked) = request(&address, "POST", "/api/wishes/web-child/check", "{}");
    assert_eq!(code, 200, "{checked}");
    assert_eq!(checked["data"]["exit_code"], 1);
    let store = xdesign::storage::RsStore::open(&chosen).unwrap();
    assert_eq!(store.get("web-child", None).unwrap().revision, 2);
    drop(store);
    fs::write(chosen.join("format.json"), b"broken marker").unwrap();
    let (code, broken) = request(&address, "GET", "/api/wishes", "");
    assert_ne!(code, 200, "corrupt store must not look empty: {broken}");
    let (_, health) = request(&address, "GET", "/api/health", "");
    assert_eq!(health["data"]["store_present"], false);
    assert!(!health["data"]["location_diagnostic"].is_null());
    child.kill().unwrap();
    child.wait().unwrap();
    assert!(!inherited.exists());
    assert!(!p.join(".xdesign").exists());
}
