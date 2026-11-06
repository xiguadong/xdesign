"""Isolated real-binary migration acceptance; detailed process transcripts are retained."""
import hashlib,json,os,pathlib,subprocess,unittest
ROOT=pathlib.Path(__file__).resolve().parents[1]
BINARY=ROOT/'cache/target/debug/xdesign'
EVIDENCE=pathlib.Path(os.environ.get('XDESIGN_TEST_EVIDENCE',ROOT/'cache/implementation/worktree-process-evidence')).resolve()
assert EVIDENCE.is_relative_to((ROOT/'cache').resolve()), 'test evidence must stay in the project cache'
class MigrationTests(unittest.TestCase):
 def test_old_schema_history_two_flows_exact_requests_and_source_change(self):
  import uuid
  case=EVIDENCE/'fixtures'/str(uuid.uuid4());case.mkdir(parents=True)
  old=ROOT/'cache/evidence/wish-worktree-global-store-migration-20261105/coordinator/xdesign-baseline'
  self.assertTrue(old.is_file(),'explicit archived old binary is required for this acceptance case')
  self.assertEqual(hashlib.sha256(old.read_bytes()).hexdigest(),'e36fd275fa169a6d55a5dc4263d3f3bb417f23d9394f8a3937e2fd72f0c22735')
  env=dict(os.environ);env.pop('XDESIGN_STORE_ROOT',None)
  def call(binary,args,cwd,expected=0,extra=None):
   actual=dict(env);actual.update(extra or {})
   result=subprocess.run([str(binary),*map(str,args)],cwd=cwd,env=actual,capture_output=True)
   with (case/'commands.jsonl').open('a') as f:f.write(json.dumps({'argv':[str(binary),*map(str,args)],'cwd':str(cwd),'exit_code':result.returncode,'stdout':result.stdout.decode(errors='replace'),'stderr':result.stderr.decode(errors='replace')})+'\n')
   self.assertEqual(result.returncode,expected,result.stderr.decode(errors='replace'))
   return json.loads(result.stdout if expected==0 else result.stderr)
  runners=[]
  for p in (ROOT/'cache/target/debug/deps').glob('xdesign-*'):
   if p.is_file() and os.access(p,os.X_OK):
    output=subprocess.run([str(p),'--list'],capture_output=True)
    if b'storage::migration::tests::real_worktree_migration' in output.stdout:runners.append(p)
  self.assertEqual(len(runners),1)
  request=case/'migration.json'
  result=subprocess.run([str(runners[0]),'--exact','storage::migration::tests::real_worktree_migration_preserves_history_and_receipts','--nocapture'],env=dict(env,XDESIGN_TEST_EXPORT_FIXTURE=str(request),XDESIGN_TEST_OLD_BINARY=str(old)),cwd=ROOT,capture_output=True)
  (case/'fixture.stdout').write_bytes(result.stdout);(case/'fixture.stderr').write_bytes(result.stderr);self.assertEqual(result.returncode,0,result.stderr.decode())
  request_value=json.loads(request.read_bytes());project=pathlib.Path(request_value['project_root']);source=request_value['source_root'];destination=request_value['destination_root']
  import shutil
  shutil.copytree(ROOT/'mbse/model',project/'mbse/model',dirs_exist_ok=True)
  (project/'wish.sysml').write_text('package FixtureWish {}\n')
  before={}
  for flow in ['simple-flow','wish-verify']:
   wish='fixture-'+flow
   call(old,['init',wish,'--flow',flow,'--wish','wish.sysml'],project,extra={'XDESIGN_STORE_ROOT':source})
   before[wish]=call(old,['next',wish],project,extra={'XDESIGN_STORE_ROOT':source})
  # Preserve blocked and completed histories as well as active heads. These
  # are real old-binary flow transitions, not hand-edited terminal records.
  call(old,['init','fixture-completed','--flow','wish-verify','--wish','wish.sysml'],project,extra={'XDESIGN_STORE_ROOT':source})
  current=call(old,['next','fixture-completed'],project,extra={'XDESIGN_STORE_ROOT':source})
  for phase in ['wish','verify']:
   work=current['work'];self.assertEqual(work['stage_id'],phase)
   artifacts=[]
   if phase=='verify':
    (project/'verification.txt').write_text('Retained verification fixture')
    artifacts=[{'uri':'verification.txt','sha256':hashlib.sha256((project/'verification.txt').read_bytes()).hexdigest()}]
   candidate={a['uri']:a for a in work['candidate_artifacts']};candidate.update({a['uri']:a for a in artifacts})
   payload=b''.join(a['uri'].encode()+b'\0'+a['sha256'].encode()+b'\n' for a in [candidate[k] for k in sorted(candidate)])
   result_body={'wish_id':'fixture-completed','run_id':current['run_id'],'stage_id':phase,'visit_index':work['visit_index'],'base_revision':work['artifact_revision'],'state_token':current['state_token'],'description':'Retained completion fixture','performed_by':'fixture','artifacts':artifacts,'judgment':{'authority':'agent','identity':'fixture-reviewer','outcome':'passed','rationale':'Fixture evidence exercises restoration of a completed history.','artifact_snapshot_digest':hashlib.sha256(payload).hexdigest()}}
   (project/'completion.json').write_text(json.dumps(result_body))
   call(old,['check','fixture-completed','--result','completion.json'],project,extra={'XDESIGN_STORE_ROOT':source})
   current=call(old,['next','fixture-completed'],project,extra={'XDESIGN_STORE_ROOT':source})
  self.assertTrue(current['completed']);before['fixture-completed']=current
  call(old,['init','fixture-blocked','--flow','wish-verify','--wish','wish.sysml'],project,extra={'XDESIGN_STORE_ROOT':source})
  r=subprocess.run([str(old),'check','fixture-blocked'],cwd=project,env=dict(env,XDESIGN_STORE_ROOT=source),capture_output=True)
  with (case/'commands.jsonl').open('a') as f:f.write(json.dumps({'argv':[str(old),'check','fixture-blocked'],'exit_code':r.returncode,'stdout':r.stdout.decode(),'stderr':r.stderr.decode()})+'\n')
  self.assertEqual(r.returncode,1,r.stderr.decode());before['fixture-blocked']=call(old,['next','fixture-blocked'],project,extra={'XDESIGN_STORE_ROOT':source})
  old_history=call(old,['store','history','generic-record','--root',source],project)
  import sqlite3
  def logical(root):
   with sqlite3.connect('file:'+str(pathlib.Path(root)/'store.db')+'?immutable=1',uri=True) as db:
    return {table:db.execute('SELECT * FROM '+table+' ORDER BY 1,2').fetchall() for table in ['items','revisions','materials','blobs','operations','pending_blobs']}
  exact_source=logical(source)
  result=call(BINARY,['store','migrate','apply','--request',request],project)
  self.assertEqual(logical(destination),exact_source)
  self.assertEqual((pathlib.Path(source)/'format.json').read_bytes(),(pathlib.Path(destination)/'format.json').read_bytes())
  self.assertEqual(result['data']['outcome'],'committed')
  self.assertEqual(call(BINARY,['store','migrate','apply','--request',request],project),result)
  self.assertEqual(call(BINARY,['store','history','generic-record','--root',destination],project),old_history)
  for wish,work in before.items():
   after=call(BINARY,['next',wish],project,extra={'XDESIGN_STORE_ROOT':destination});self.assertEqual(after,work)
  import shutil
  shutil.rmtree(project/'cache/xdesign');shutil.rmtree(project/'cache/wishes')
  self.assertEqual(call(BINARY,['next','fixture-completed'],project,extra={'XDESIGN_STORE_ROOT':source}),before['fixture-completed'])
  self.assertFalse((project/'cache/xdesign').exists())
  r=subprocess.run([str(BINARY),'next','fixture-simple-flow'],cwd=project,env=dict(env,XDESIGN_STORE_ROOT=source),capture_output=True)
  (case/'retired-active-next.stdout').write_bytes(r.stdout);(case/'retired-active-next.stderr').write_bytes(r.stderr)
  self.assertEqual(r.returncode,2);self.assertIn(b'location_reason=source_retired;',r.stderr);self.assertFalse((project/'cache/xdesign').exists())
  for wish,work in before.items():self.assertEqual(call(BINARY,['next',wish],project,extra={'XDESIGN_STORE_ROOT':destination}),work)
  terminal_before=logical(destination)
  call(BINARY,['check','fixture-completed'],project,extra={'XDESIGN_STORE_ROOT':destination})
  self.assertEqual(logical(destination),terminal_before)
  recovered=case/'recovered.json';call(BINARY,['store','migrate','request','migration-test','--root',source,'--output',recovered],project);self.assertEqual(recovered.read_bytes(),request.read_bytes())
  old_request=project/'old-request.json'
  call(BINARY,['store','save','--root',destination,'--request',old_request],project)
  self.assertEqual(call(BINARY,['store','save','--root',source,'--request',old_request],project,4)['error']['code'],'source_retired')
  changed=json.loads(old_request.read_bytes());(project/'bytes.txt').write_bytes(b'historical bytes');changed['operation_id']='old-client-later';changed['expected_revision']=3;(case/'old-change.json').write_text(json.dumps(changed))
  call(old,['store','save','--root',source,'--request',case/'old-change.json'],project)
  self.assertEqual(call(BINARY,['store','save','--root',destination,'--request',old_request],project,4)['error']['code'],'source_changed')
  self.assertEqual(call(BINARY,['store','migrate','status','migration-test','--root',destination],project,4)['error']['code'],'source_changed')
  self.assertEqual(call(BINARY,['store','migrate','apply','--request',request],project,4)['error']['code'],'source_changed')
 def test_process_crash_boundaries_recover_or_replay(self):
  import uuid,shutil
  case=EVIDENCE/'crash-fixtures'/str(uuid.uuid4());case.mkdir(parents=True)
  env=dict(os.environ);env.pop('XDESIGN_STORE_ROOT',None)
  runner=None
  for p in (ROOT/'cache/target/debug/deps').glob('xdesign-*'):
   if p.is_file() and os.access(p,os.X_OK) and b'storage::migration::migration_process_driver' in subprocess.run([str(p),'--list'],capture_output=True).stdout:runner=p;break
  self.assertIsNotNone(runner)
  for point in ['reservation-draft-written','reservation-draft-synced','reservation-published','reserved','destination-pointer','destination-allocated','source-pointer','fenced','database-created','blob-published','import-committed','prepared','before-decision','decision-published','decision-durable']:
   with self.subTest(point=point):
    directory=case/point;directory.mkdir();request=directory/'request.json'
    command=[str(runner),'--exact','storage::migration::tests::real_worktree_migration_preserves_history_and_receipts','--nocapture']
    r=subprocess.run(command,cwd=ROOT,env=dict(env,XDESIGN_TEST_EXPORT_FIXTURE=str(request)),capture_output=True)
    (directory/'prepare.stdout').write_bytes(r.stdout);(directory/'prepare.stderr').write_bytes(r.stderr);self.assertEqual(r.returncode,0,r.stderr.decode())
    value=json.loads(request.read_bytes())
    driver=[str(runner),'--exact','storage::migration::migration_process_driver','--nocapture']
    r=subprocess.run(driver,cwd=ROOT,env=dict(env,XDESIGN_TEST_MIGRATION_REQUEST=str(request),XDESIGN_TEST_MIGRATION_CRASH=point),capture_output=True)
    (directory/'crash.stdout').write_bytes(r.stdout);(directory/'crash.stderr').write_bytes(r.stderr);self.assertEqual(r.returncode,86,r.stderr.decode())
    # Publication is the only durable decision: source absence is allowed only
    # after a visible complete decision is confirmed under the leases.
    if point in ['decision-published','decision-durable']:shutil.rmtree(value['source_root'])
    observations=[]
    for root in [value['source_root'],value['destination_root']]:
     cmd=[str(BINARY),'store','migrate','status',value['migration_id'],'--root',root]
     r=subprocess.run(cmd,cwd=value['project_root'],env=env,capture_output=True)
     observations.append({'argv':cmd,'exit':r.returncode,'stdout':r.stdout.decode(),'stderr':r.stderr.decode()})
     if point in ['reservation-draft-written','reservation-draft-synced']:self.assertEqual(r.returncode,3,r.stderr.decode())
     else:self.assertEqual(r.returncode,0,r.stderr.decode())
    r=subprocess.run(driver,cwd=ROOT,env=dict(env,XDESIGN_TEST_MIGRATION_REQUEST=str(request)),capture_output=True)
    (directory/'retry.stdout').write_bytes(r.stdout);(directory/'retry.stderr').write_bytes(r.stderr);(directory/'status.json').write_text(json.dumps(observations,indent=2));self.assertEqual(r.returncode,0,r.stderr.decode())
    status=subprocess.run([str(BINARY),'store','migrate','status',value['migration_id'],'--root',value['destination_root']],cwd=value['project_root'],env=env,capture_output=True)
    self.assertEqual(status.returncode,0,status.stderr.decode());self.assertEqual(json.loads(status.stdout)['data']['outcome'],'committed')
    (directory/'result.json').write_text(json.dumps({'crash_point':point,'crash_exit':86,'retry_exit':r.returncode,'terminal':json.loads(status.stdout)},indent=2))
 def fixture(self, label, routes=True):
  import uuid
  case=EVIDENCE/label/str(uuid.uuid4());case.mkdir(parents=True)
  runner=None
  for p in (ROOT/'cache/target/debug/deps').glob('xdesign-*'):
   if p.is_file() and os.access(p,os.X_OK) and b'storage::migration::migration_process_driver' in subprocess.run([str(p),'--list'],capture_output=True).stdout:runner=p;break
  self.assertIsNotNone(runner);request=case/'request.json'
  env=dict(os.environ);env.pop('XDESIGN_STORE_ROOT',None)
  setup=dict(env,XDESIGN_TEST_EXPORT_FIXTURE=str(request))
  if not routes:setup['XDESIGN_TEST_REMOVE_ROUTES']='1'
  r=subprocess.run([str(runner),'--exact','storage::migration::tests::real_worktree_migration_preserves_history_and_receipts','--nocapture'],cwd=ROOT,env=setup,capture_output=True)
  (case/'prepare.stdout').write_bytes(r.stdout);(case/'prepare.stderr').write_bytes(r.stderr);self.assertEqual(r.returncode,0,r.stderr.decode())
  counter=0
  def driver(action='apply',side='source',expected=0,**extra):
   nonlocal counter
   counter+=1
   e=dict(env,XDESIGN_TEST_MIGRATION_REQUEST=str(request),XDESIGN_TEST_MIGRATION_ACTION=action,XDESIGN_TEST_MIGRATION_SIDE=side,**extra)
   r=subprocess.run([str(runner),'--exact','storage::migration::migration_process_driver','--nocapture'],cwd=ROOT,env=e,capture_output=True)
   (case/(str(counter)+'.stdout')).write_bytes(r.stdout);(case/(str(counter)+'.stderr')).write_bytes(r.stderr)
   with (case/'commands.jsonl').open('a') as f:f.write(json.dumps({'action':action,'side':side,'environment':{k:v for k,v in e.items() if k.startswith('XDESIGN_TEST_MIGRATION_')},'exit':r.returncode,'stdout':r.stdout.decode(),'stderr':r.stderr.decode()})+'\n')
   self.assertEqual(r.returncode,expected,r.stderr.decode());return r
  return case,request,json.loads(request.read_bytes()),driver

 def test_first_route_publication_is_recoverable_without_accepted_request(self):
  for point in ['access-directory-created','access-marker-written','access-marker-synced','access-directory-published','access-directory-durable','route-draft-written','route-draft-synced','route-published','source-route-published','destination-route-published']:
   with self.subTest(point=point):
    case,request,value,driver=self.fixture('routing-crashes',routes=False)
    driver(expected=86,XDESIGN_TEST_MIGRATION_CRASH=point)
    parents=[pathlib.Path(value[k]).parent for k in ['source_root','destination_root']]
    drafts={str(p):p.read_bytes() for parent in parents for p in parent.glob('.xdesign-access-draft-*/*') if p.is_file()}
    drafts.update({str(p):p.read_bytes() for parent in parents for p in parent.glob('.xdesign-access-v1/reservation-drafts/*') if p.is_file()})
    for side in ['source','destination']:
     for action in ['status','request','abort','discard']:
      self.assertIn(b'location_reason=not_found;',driver(action,side,87).stderr)
    self.assertFalse(pathlib.Path(value['destination_root']).exists())
    driver()
    for name,raw in drafts.items():self.assertEqual(pathlib.Path(name).read_bytes(),raw)

 def test_decision_uncertainty_precedes_source_absence(self):
  import shutil
  for phase in ['before-decision','decision-published']:
   with self.subTest(phase=phase):
    case,request,value,driver=self.fixture('uncertainty')
    driver(expected=86,XDESIGN_TEST_MIGRATION_CRASH=phase)
    shutil.rmtree(value['source_root'])
    for side in ['source','destination']:
     for failure in ['decision-read','decision-sync']:
      for action in ['status','apply','abort','discard']:
       r=driver(action,side,87,XDESIGN_TEST_MIGRATION_IO_FAILURE=failure)
       self.assertIn(b'location_reason=migration_outcome_unknown;',r.stderr)
      r=driver('request',side,0,XDESIGN_TEST_MIGRATION_IO_FAILURE=failure)
      self.assertIn(request.read_bytes().hex().encode(),r.stdout)
    if phase=='decision-published':driver()
    else:self.assertIn(b'location_reason=identity_unprovable;',driver(expected=87).stderr)

 def test_initialization_retry_requires_recorded_physical_proof(self):
  import shutil
  for phase in ['initialization-reserved','initialization-constructed','initialization-verified','initialization-witness-installed']:
   with self.subTest(phase=phase):
    case,request,value,driver=self.fixture('initialization')
    shutil.rmtree(value['source_root'])
    driver('initialize',expected=86,XDESIGN_TEST_MIGRATION_CRASH=phase)
    path=pathlib.Path(value['destination_root']).parent/'initialization.json'
    before=json.loads(path.read_bytes())
    if phase=='initialization-constructed':
     self.assertIn(b'location_reason=identity_unprovable;',driver('initialize',expected=87).stderr)
     self.assertEqual(json.loads(path.read_bytes()),before)
    else:
     driver('initialize')
     self.assertEqual(json.loads(path.read_bytes())['initialization_id'],before['initialization_id'])

 def test_records_helper_preserves_explicit_and_environment_root_selection(self):
  import shutil
  case,request,value,driver=self.fixture('records-helper')
  project=pathlib.Path(value['project_root']);(project/'tools').mkdir()
  shutil.copy2(ROOT/'tools/records.py',project/'tools/records.py')
  executable=project/'cache/target/debug/xdesign';executable.parent.mkdir(parents=True)
  executable.symlink_to(BINARY)
  def call(args,root):
   env=dict(os.environ,XDESIGN_STORE_ROOT=str(root));r=subprocess.run(['python3','-B',str(project/'tools/records.py'),*args],cwd=case,env=env,capture_output=True)
   with (case/'records.jsonl').open('a') as f:f.write(json.dumps({'args':args,'environment_root':str(root),'exit':r.returncode,'stdout':r.stdout.decode(),'stderr':r.stderr.decode()})+'\n')
   self.assertEqual(r.returncode,0,r.stderr.decode());return json.loads(r.stdout)
  before=call(['show','generic-record','--revision','1'],value['source_root'])
  self.assertEqual(before['revision'],1)
  driver()
  self.assertEqual(call(['--root',value['destination_root'],'show','generic-record','--revision','1'],case/'wrong-root'),before)
  self.assertEqual(call(['list'],value['destination_root']),[{'item_id':'generic-record','head_revision':3}])

 def test_aborted_owned_targets_discard_but_foreign_residue_survives(self):
  for phase in ['database-created','blob-published','prepared']:
   for foreign in [False,True]:
    with self.subTest(phase=phase,foreign=foreign):
     case,request,value,driver=self.fixture('discard')
     driver(expected=86,XDESIGN_TEST_MIGRATION_CRASH=phase)
     destination=pathlib.Path(value['destination_root'])
     driver('abort')
     if foreign:
      (destination/'foreign.txt').write_bytes(b'preserve foreign residue')
      driver('discard',expected=87)
      self.assertEqual((destination/'foreign.txt').read_bytes(),b'preserve foreign residue')
     else:
      self.assertIn(b'"outcome":"discarded"',driver('discard').stdout)
      self.assertFalse(destination.exists())
      self.assertIn(b'"outcome":"discarded"',driver('discard').stdout)

 def test_reserved_abort_without_roots_and_foreign_slots(self):
  import shutil
  case,request,value,driver=self.fixture('abort')
  driver(expected=86,XDESIGN_TEST_MIGRATION_CRASH='reserved')
  shutil.rmtree(value['source_root'])
  raw=request.read_bytes()
  for side in ['source','destination']:
   self.assertIn(b'"phase":"reserved"',driver('status',side).stdout)
   self.assertIn(raw.hex().encode(),driver('request',side).stdout)
  self.assertIn(b'"outcome":"aborted"',driver('abort').stdout)
  self.assertIn(b'location_reason=migration_aborted;',driver(expected=87).stderr)
  driver('discard','destination');driver('abort','destination')
  case,request,value,driver=self.fixture('foreign-slot')
  slot=pathlib.Path(value['destination_root']).parent/'.xdesign-access-v1/migrations/foreign'
  slot.mkdir(mode=0o700);(slot/'reservation.pending').write_bytes(b'foreign unaccepted bytes');(slot/'reservation.pending').chmod(0o600)
  self.assertIn(b'location_reason=coordination_corrupt;',driver(expected=87).stderr)
  self.assertEqual((slot/'reservation.pending').read_bytes(),b'foreign unaccepted bytes')
  self.assertFalse(pathlib.Path(value['destination_root']).exists())

 def test_refusal_preserves_source_and_unrelated_bytes(self):
  import sqlite3
  for defect in ['unknown_schema','stale_lock','orphan_blob','occupied_destination','false_quiescence']:
   with self.subTest(defect=defect):
    case,request,value,driver=self.fixture('refusals')
    source=pathlib.Path(value['source_root']);destination=pathlib.Path(value['destination_root'])
    if defect=='unknown_schema':
     with sqlite3.connect(source/'store.db') as db:db.execute('CREATE TABLE foreign_extension(value TEXT)')
    if defect=='stale_lock':(source/'locks/items/stale.lock').write_text('pid=0 token=stale\n')
    if defect=='orphan_blob':
     blob=source/'blobs/sha256/00'/('0'*62);blob.parent.mkdir(exist_ok=True,mode=0o700);blob.write_bytes(b'orphan');blob.chmod(0o400)
    if defect=='occupied_destination':destination.mkdir(mode=0o700);(destination/'unrelated').write_bytes(b'preserve')
    if defect=='false_quiescence':value['older_clients_stopped']=False;request.write_text(json.dumps(value))
    with sqlite3.connect(source/'store.db') as db:before=db.execute('SELECT * FROM operations ORDER BY operation_id').fetchall()
    driver(expected=87)
    with sqlite3.connect(source/'store.db') as db:self.assertEqual(db.execute('SELECT * FROM operations ORDER BY operation_id').fetchall(),before)
    self.assertFalse(list(destination.parent.glob('.xdesign-access-v1/migrations/*/decision.json')))
    if defect=='occupied_destination':self.assertEqual((destination/'unrelated').read_bytes(),b'preserve')
if __name__=='__main__':unittest.main()
