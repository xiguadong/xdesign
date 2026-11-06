"""Public migration workflow with genuine old-binary history and account isolation.

XDESIGN_OLD_FIXTURE_BINARY must name an independently built pre-migration binary.
On macOS the injected dylib replaces only libc's account lookup; no product API
or test-only request generator supplies migration bytes. Linux uses the isolated
VM's account, whose entire root filesystem is a disposable cache-local disk.
"""
import hashlib
import json
import os
import pathlib
import shutil
import sqlite3
import subprocess
import sys
import unittest
import uuid

ROOT = pathlib.Path(__file__).resolve().parents[1]
EVIDENCE = pathlib.Path(os.environ.get('XDESIGN_TEST_EVIDENCE', ROOT / 'cache/public-cli-evidence')).resolve()
BINARY = pathlib.Path(os.environ.get('XDESIGN_FIXTURE_BINARY', ROOT / 'cache/target/debug/xdesign')).resolve()


class PublicMigration(unittest.TestCase):
    def test_public_old_history_and_recovery(self):
        self.assertTrue(EVIDENCE.is_relative_to((ROOT / 'cache').resolve()))
        old = pathlib.Path(os.environ['XDESIGN_OLD_FIXTURE_BINARY']).resolve()
        self.assertTrue(old.is_relative_to((ROOT / 'cache').resolve()))
        case = EVIDENCE / 'public-cli' / str(uuid.uuid4())
        case.mkdir(parents=True)
        home = case / 'home'
        home.mkdir(mode=0o700)
        env = dict(os.environ)
        env.pop('XDESIGN_STORE_ROOT', None)
        env.update(HOME=str(case / 'misleading-home'), TMPDIR=str(case), TMP=str(case), TEMP=str(case))
        if sys.platform == 'darwin':
            shim = case / 'account.c'
            shim.write_text(r'''
#include <pwd.h>
#include <errno.h>
#include <string.h>
#include <unistd.h>
#include <stdlib.h>
static int fixture_account(uid_t uid, struct passwd *p, char *b, size_t n, struct passwd **r) {
    if (uid != geteuid() || !getenv("XDESIGN_FIXTURE_ACCOUNT_HOME")) { *r = NULL; return ENOENT; }
    memset(p, 0, sizeof(*p));
    p->pw_uid = uid;
    p->pw_gid = getegid();
    p->pw_name = "fixture";
    p->pw_shell = "/bin/sh";
    p->pw_dir = getenv("XDESIGN_FIXTURE_ACCOUNT_HOME");
    *r = p;
    return 0;
}
__attribute__((used)) static struct { const void *replacement; const void *original; }
interpose __attribute__((section("__DATA,__interpose"))) = { (void *)fixture_account, (void *)getpwuid_r };
''')
            subprocess.run(['clang', '-dynamiclib', str(shim), '-o', str(case / 'account.dylib')], check=True, env=env)
            env.update(DYLD_INSERT_LIBRARIES=str(case / 'account.dylib'), XDESIGN_FIXTURE_ACCOUNT_HOME=str(home))
        else:
            # Only an explicitly authorized disposable Linux guest may use this.
            self.assertEqual(env.get('XDESIGN_ISOLATED_LINUX_GUEST'), '1')
            import pwd
            home = pathlib.Path(pwd.getpwuid(os.geteuid()).pw_dir)
        def call(binary, args, cwd, expected=0, extra=None):
            actual = dict(env, **(extra or {}))
            command = [str(binary), *map(str, args)]
            r = subprocess.run(command, cwd=cwd, env=actual, capture_output=True)
            with (case / 'commands.jsonl').open('a') as log:
                log.write(json.dumps(dict(argv=command, cwd=str(cwd), exit_code=r.returncode,
                    stdout=r.stdout.decode(errors='replace'), stderr=r.stderr.decode(errors='replace'))) + '\n')
            self.assertEqual(r.returncode, expected, r.stderr.decode(errors='replace'))
            return json.loads(r.stdout if expected == 0 else r.stderr)
        primary, project = case / 'primary', case / 'linked'
        primary.mkdir()
        for args in [['init', '-q'], ['-c', 'user.name=Fixture', '-c', 'user.email=fixture@example.invalid', 'commit', '--allow-empty', '-qm', 'fixture'], ['worktree', 'add', '-q', '--detach', str(project)]]:
            subprocess.run(['git', '-C', str(primary), *args], env=env, check=True, capture_output=True)
        source = project / '.xdesign'
        help_text = call(BINARY, ['store', '--help'], project)['data']['help']
        for verb in ['location inspect', 'location enroll', 'migrate prepare', 'migrate apply', 'migrate status', 'migrate request', 'migrate abort', 'migrate discard-prepared']:
            self.assertIn(verb, help_text)
        call(old, ['store', 'init', '--root', source], project)
        requests, receipts = [], []
        for rev in range(3):
            material = ('old material %s\n' % rev).encode()
            (project / 'bytes.txt').write_bytes(material)
            request = dict(schema_version=1, operation_id='old-save-%s' % rev, item_id='history', kind='record', expected_revision=rev, metadata={'exact': 'old', 'revision': rev}, materials=[dict(uri='evidence/history.txt', role='evidence', sha256=hashlib.sha256(material).hexdigest(), source='bytes.txt')], result={'old_result': rev})
            raw = json.dumps(request, indent=2).encode() + b'\n'
            path = case / ('save-%s.json' % rev)
            path.write_bytes(raw)
            requests.append(path)
            receipts.append(call(old, ['store', 'save', '--root', source, '--request', path, '--project-root', project], project))
        conflict = dict(request, operation_id='old-conflict', expected_revision=0)
        (case / 'conflict.json').write_text(json.dumps(conflict, indent=3))
        conflict_receipt = call(old, ['store', 'save', '--root', source, '--request', case / 'conflict.json'], project, 4)
        def attributes(path):
            if hasattr(os, 'listxattr'):
                return {k: os.getxattr(path, k).hex() for k in os.listxattr(path)}
            import ctypes
            lib = ctypes.CDLL(None, use_errno=True)
            raw = os.fsencode(path)
            n = lib.listxattr(raw, None, 0, 0)
            if n < 0: raise OSError(ctypes.get_errno(), 'listxattr')
            names = ctypes.create_string_buffer(n)
            if lib.listxattr(raw, names, n, 0) != n: raise OSError('listxattr changed')
            result = {}
            for name in names.raw.split(b'\0'):
                if not name: continue
                size = lib.getxattr(raw, name, None, 0, 0, 0)
                if size < 0: raise OSError(ctypes.get_errno(), 'getxattr')
                value = ctypes.create_string_buffer(size)
                if lib.getxattr(raw, name, value, size, 0, 0) != size: raise OSError('getxattr changed')
                result[name.decode()] = value.raw.hex()
            return result
        def snapshot(paths):
            result = {}
            for root in paths:
                for path in [root, *sorted(root.rglob('*'))] if root.exists() else []:
                    m = path.lstat()
                    result[str(path)] = [m.st_dev, m.st_ino, m.st_mode, hashlib.sha256(path.read_bytes()).hexdigest() if path.is_file() else None, attributes(path)]
            return result
        # Inspect must not create directories, install attributes or write SQLite.
        before = snapshot([home, primary, project])
        inspected = call(BINARY, ['store', 'location', 'inspect'], project, 4)['data']
        self.assertEqual(inspected['failed_predicates'][0]['code'], 'migration_required')
        self.assertIn('physical_identity', inspected['source'])
        self.assertTrue(inspected['next_commands'])
        self.assertEqual(snapshot([home, primary, project]), before)
        enrolled = call(BINARY, ['store', 'location', 'enroll', '--operation-id', 'public-enrollment'], project)['data']
        destination = pathlib.Path(enrolled['store_root'])
        self.assertTrue(destination.is_relative_to(home))
        self.assertFalse(destination.exists())
        before = snapshot([home, primary, project])
        inspected = call(BINARY, ['store', 'location', 'inspect'], project, 4)['data']
        self.assertEqual(inspected['binding_sha256'], enrolled['binding_sha256'])
        self.assertEqual(snapshot([home, primary, project]), before)
        request_path = case / 'migration.json'
        self.assertEqual(call(BINARY, ['store', 'migrate', 'prepare', 'public-migration', '--output', request_path], project, 2)['error']['code'], 'invalid_input')
        self.assertFalse(request_path.exists())
        call(BINARY, ['store', 'migrate', 'prepare', 'public-migration', '--output', request_path, '--older-clients-stopped'], project)
        raw_request = request_path.read_bytes()
        request = json.loads(raw_request)
        self.assertEqual(request['source_root'], str(source))
        self.assertEqual(request['destination_root'], str(destination))
        self.assertTrue(request['older_clients_stopped'])
        def logical(root):
            with sqlite3.connect('file:' + str(root / 'store.db') + '?immutable=1', uri=True) as db:
                tables = {t: db.execute('SELECT * FROM ' + t + ' ORDER BY 1,2').fetchall() for t in ['items', 'revisions', 'materials', 'blobs', 'operations', 'pending_blobs']}
            return tables
        baseline = logical(source)
        def blob_bytes(root):
            return {str(p.relative_to(root)): hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted((root / 'blobs').rglob('*')) if p.is_file()}
        before_blobs = blob_bytes(source)
        source_database_sha = hashlib.sha256((source / 'store.db').read_bytes()).hexdigest()
        (case / 'independent-baseline.json').write_text(json.dumps(dict(tables=baseline, blobs=before_blobs, source_database_sha256=source_database_sha), default=lambda b: {'hex': b.hex()}, indent=2))
        applied = call(BINARY, ['store', 'migrate', 'apply', '--request', request_path], project)
        self.assertEqual(applied['data']['outcome'], 'committed')
        self.assertEqual(logical(destination), baseline)
        self.assertEqual(logical(source), baseline)
        self.assertEqual(blob_bytes(destination), before_blobs)
        self.assertEqual(blob_bytes(source), before_blobs)
        self.assertEqual(hashlib.sha256((source / 'store.db').read_bytes()).hexdigest(), source_database_sha)
        self.assertEqual((source / 'format.json').read_bytes(), (destination / 'format.json').read_bytes())
        self.assertNotEqual(source.stat().st_ino, destination.stat().st_ino)
        self.assertNotEqual((source / 'store.db').stat().st_ino, (destination / 'store.db').stat().st_ino)
        self.assertEqual(call(BINARY, ['store', 'location', 'inspect'], project)['data']['authority_state'], 'committed_destination')
        for i, root in enumerate([source, destination]):
            self.assertEqual(call(BINARY, ['store', 'migrate', 'status', 'public-migration', '--root', root], project)['data'], applied['data'])
            recovered = case / ('recovered-%s.json' % i)
            call(BINARY, ['store', 'migrate', 'request', 'public-migration', '--root', root, '--output', recovered], project)
            self.assertEqual(recovered.read_bytes(), raw_request)
            self.assertEqual(call(BINARY, ['store', 'migrate', 'apply', '--request', recovered], project), applied)
        duplicate = case / 'prepared-retry.json'
        call(BINARY, ['store', 'migrate', 'prepare', 'public-migration', '--output', duplicate, '--older-clients-stopped'], project)
        self.assertEqual(duplicate.read_bytes(), raw_request)
        changed = case / 'changed.json'
        changed.write_bytes(raw_request + b'\n')
        self.assertEqual(call(BINARY, ['store', 'migrate', 'apply', '--request', changed], project, 4)['error']['code'], 'operation_mismatch')
        self.assertEqual(call(BINARY, ['store', 'save', '--root', destination, '--request', requests[-1]], project), receipts[-1])
        self.assertEqual(call(BINARY, ['store', 'save', '--root', destination, '--request', case / 'conflict.json'], project, 4), conflict_receipt)
        self.assertEqual(logical(destination), baseline)
        if sys.platform == 'darwin':
            unsafe_parent = case / 'replaceable-parent'
            unsafe_parent.mkdir()
            unsafe_home = unsafe_parent / 'home'
            unsafe_home.mkdir(mode=0o700)
            unsafe_parent.chmod(0o777)
            unsafe_env = {'XDESIGN_FIXTURE_ACCOUNT_HOME': str(unsafe_home)}
            error = call(BINARY, ['store', 'location', 'enroll', '--operation-id', 'unsafe-enroll'], project, 5, unsafe_env)
            self.assertEqual(error['error']['code'], 'unsafe_path')
            self.assertFalse((unsafe_home / '.xdesign').exists())
            explicit = call(BINARY, ['store', 'location', 'inspect', '--root', source], project, extra=unsafe_env)
            self.assertEqual(explicit['data']['selected_by'], 'explicit')
            self.assertEqual(explicit['data']['authority_state'], 'retained_source')
        (case / 'result.json').write_text(json.dumps(dict(binary_sha256=hashlib.sha256(BINARY.read_bytes()).hexdigest(), old_binary_sha256=hashlib.sha256(old.read_bytes()).hexdigest(), request_sha256=hashlib.sha256(raw_request).hexdigest(), source=str(source), destination=str(destination), public_request_generation=True, exact_history_and_retries=True), indent=2))


if __name__ == '__main__':
    unittest.main()
