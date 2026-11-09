"""BDK-05/08: current-root parity through Python, JSON CLI and real stdio MCP."""
from __future__ import annotations
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import unittest
from unittest import mock

from mcp import ClientSession, StdioServerParameters
from mcp.client.stdio import stdio_client
from _bilingual_support import (
    KIT, CACHE, EVIDENCE, assert_created_identity, companions, compare_created_bytes,
    fixture_for, materialize, selection, sha_bytes, snapshot, temporary_directory,
    updated_input, write_json,
)
from xdesign_doc_kit import core, DocumentKitError


def envelope(result):
    if not result.is_error:
        return result.structured_content
    text = ' '.join(getattr(c, 'text', '') for c in result.content)
    return json.loads(text[text.index('{'):])


class BilingualTransportTests(unittest.IsolatedAsyncioTestCase):
    def setUp(self):
        self.tmp = temporary_directory()
        self.root = Path(self.tmp.name)
        self.transcript = []
        self.env = {**os.environ, 'PYTHONPATH': str(KIT), 'PYTHONDONTWRITEBYTECODE': '1',
                    'TMPDIR': str(CACHE)}

    def tearDown(self):
        write_json(EVIDENCE / (self._testMethodName + '-transcript.json'), self.transcript)
        self.tmp.cleanup()

    def server(self, project, pack=KIT):
        return StdioServerParameters(command=sys.executable,
            args=['-m', 'xdesign_doc_kit.mcp_server', '--project-root', str(project), '--profile', str(pack)],
            env=self.env)

    async def call(self, client, name, args):
        result = await client.call_tool(name, args)
        data = envelope(result)
        self.transcript.append({'transport': 'mcp-stdio', 'tool': name, 'arguments': args,
                                'is_error': result.is_error, 'result': data})
        # A completed check with findings is a normal MCP response, not a ToolError.
        self.assertEqual(result.is_error, 'error' in data)
        return data

    def race_cases(self):
        return (
            ('guide', 'entry_resource', ('generic-requirements', 'zh-CN', 'guide'),
             'xdesign-doc-kit://entry/generic-requirements/zh-CN/guide'),
            ('template', 'language_template_resource', ('requirements', 'generic-requirements', 'zh-CN'),
             'xdesign-doc-kit://template/requirements/generic-requirements/zh-CN'),
        )

    def test_bdk05_verified_resource_snapshot_in_process(self):
        from xdesign_doc_kit import mcp_server as server
        for kind, handler, args, uri in self.race_cases():
            with self.subTest(uri=uri):
                pack = self.root / kind
                shutil.copytree(KIT, pack)
                contract = core.get_contract(pack)
                resource = pack / f'catalog/entries/generic-requirements/zh-CN/{kind}.md'
                original = resource.read_bytes()
                verified = server._resource_contract
                changed = False

                def mutate_after_verification():
                    nonlocal changed
                    snapshot = verified()
                    if not changed:
                        resource.write_bytes(original + b'\nConcurrent unreviewed resource revision.\n')
                        changed = True
                    return snapshot

                binding = server._Binding(self.root, pack, contract['contract_sha256'], '1.3')
                with mock.patch.object(server, '_binding', binding), \
                     mock.patch.object(server, '_resource_contract', mutate_after_verification):
                    returned = getattr(server, handler)(*args)
                    self.assertTrue(changed)
                    self.assertEqual(returned.encode(), original)
                    with self.assertRaises(Exception) as rejected:
                        getattr(server, handler)(*args)
                    self.assertIn('contract-drift', str(rejected.exception))
                self.transcript.append({
                    'transport': 'in-process', 'uri': uri, 'launch_digest': contract['contract_sha256'],
                    'verified_resource_sha256': sha_bytes(original), 'returned_sha256': sha_bytes(returned.encode()),
                    'mutated_resource_sha256': sha_bytes(resource.read_bytes()),
                    'changed_contract_digest': core.get_contract(pack)['contract_sha256'],
                    'next_request_error': str(rejected.exception),
                })

    async def test_bdk05_verified_resource_snapshot_actual_stdio(self):
        # Injection is confined to this test process wrapper; the production
        # server has no test hook or bypass. Mutate after pin verification.
        driver = self.root / 'race_server.py'
        driver.write_text('''import sys
from pathlib import Path
from xdesign_doc_kit import mcp_server as server
resource = Path(sys.argv[1])
verified = server._resource_contract
changed = False
def mutate_after_verification():
    global changed
    snapshot = verified()
    if not changed:
        resource.write_bytes(resource.read_bytes() + b'\\nConcurrent unreviewed resource revision.\\n')
        changed = True
    return snapshot
server._resource_contract = mutate_after_verification
raise SystemExit(server.main(sys.argv[2:]))
''', encoding='utf-8')
        for kind, _, _, uri in self.race_cases():
            with self.subTest(uri=uri):
                pack = self.root / kind
                shutil.copytree(KIT, pack)
                project = self.root / (kind + '-project')
                project.mkdir()
                contract = core.get_contract(pack)
                resource = pack / f'catalog/entries/generic-requirements/zh-CN/{kind}.md'
                original = resource.read_bytes()
                parameters = StdioServerParameters(command=sys.executable,
                    args=[str(driver), str(resource), '--project-root', str(project), '--profile', str(pack)],
                    env=self.env)
                async with stdio_client(parameters) as (read, write), ClientSession(read, write) as client:
                    initialized = await client.initialize()
                    returned = await client.read_resource(uri)
                    self.assertNotEqual(resource.read_bytes(), original)
                    self.assertEqual(returned.contents[0].text.encode(), original)
                    with self.assertRaises(Exception) as rejected:
                        await client.read_resource(uri)
                    self.assertIn('contract-drift', str(rejected.exception))
                self.transcript.append({
                    'transport': 'actual-mcp-stdio', 'uri': uri, 'command': parameters.args,
                    'injection_source_sha256': sha_bytes(driver.read_bytes()),
                    'initialize': initialized.model_dump(mode='json'),
                    'launch_digest': contract['contract_sha256'], 'verified_resource_sha256': sha_bytes(original),
                    'returned_sha256': sha_bytes(returned.contents[0].text.encode()),
                    'mutated_resource_sha256': sha_bytes(resource.read_bytes()),
                    'changed_contract_digest': core.get_contract(pack)['contract_sha256'],
                    'response': returned.model_dump(mode='json'), 'next_request_error': str(rejected.exception),
                })

    def cli(self, project, name, args, payload=None, pack=KIT, exit=0, extra=()):
        command = [sys.executable, '-m', 'xdesign_doc_kit', name.replace('_', '-'),
                   '--profile-root', str(pack)]
        if name != 'get_contract':
            command += ['--target-root', str(project)]
        for key, value in args.items():
            if value is not None:
                command += ['--'+{'document_class':'class', 'section_key':'section'}.get(key,key.replace('_','-')), str(value)]
        if payload is not None:
            path = self.root / 'caller-input.json'
            write_json(path, payload)
            command += ['--input', str(path)]
        command += list(extra)
        result = subprocess.run(command, env=self.env, capture_output=True, text=True, timeout=90)
        self.assertEqual(result.returncode, exit, result.stdout + result.stderr)
        data = json.loads(result.stdout)
        self.transcript.append({'transport':'json-cli', 'command':command, 'payload':payload,
                                'exit':result.returncode, 'result':data})
        return data

    async def test_bdk05_all_42_operations_exact_bytes(self):
        contract=core.get_contract(KIT)
        matrix=[]
        for entry in contract['catalog']['entries']:
            for language in ('en','zh-CN'):
                with self.subTest(template_id=entry['template_id'],language=language):
                    f=fixture_for(entry,language)
                    projects=[self.root/(entry['template_id']+'-'+language)/s for s in ('python','cli','mcp')]
                    for p in projects:
                        p.mkdir(parents=True);materialize(p,f);companions(p,f)
                    py,cli,mcp=projects
                    args={**selection(f),'template_id':f['template_id'],'language':language}
                    payload={k:f[k] for k in ('metadata','sections')}
                    async with stdio_client(self.server(mcp)) as (read,write), ClientSession(read,write) as client:
                        await client.initialize()
                        before=[snapshot(p) for p in projects]
                        planned=core.plan_document(py,**args,profile_root=KIT)
                        self.assertEqual(self.cli(cli,'plan_document',args),planned)
                        self.assertEqual(await self.call(client,'plan_document',args),planned)
                        self.assertEqual([snapshot(p) for p in projects],before)
                        created=core.create_document(py,**args,**payload,profile_root=KIT)
                        self.assertEqual(self.cli(cli,'create_document',args,payload),created)
                        self.assertEqual(await self.call(client,'create_document',{**args,**payload}),created)
                        for p in projects:
                            assert_created_identity(self,p,created,f,contract['contract_sha256'])
                            compare_created_bytes((KIT/f['expected_document']).read_bytes(),(p/created['path']).read_bytes(),contract['contract_sha256'],created['effective_identity'])
                        selected={'selection':selection(f),'language':language}
                        checked=core.check_documents(py,profile_root=KIT,**selected)
                        self.assertTrue(checked['ok'],checked)
                        self.assertEqual(self.cli(cli,'check_documents',{**selection(f),'language':language}),checked)
                        self.assertEqual(await self.call(client,'check_documents',selected),checked)
                        key,content,fields=updated_input(f)
                        route={**selection(f),'language':language}
                        update={'content':content,'fields':fields,'expected_sha256':created['sha256']}
                        updated=core.update_section(py,**route,section_key=key,**update,profile_root=KIT)
                        self.assertEqual(self.cli(cli,'update_section',{**route,'section_key':key},update),updated)
                        self.assertEqual(await self.call(client,'update_section',{**route,'section_key':key,**update}),updated)
                        # A supplied figure works for every class/language; this section has no selected concern.
                        # For entries whose first section carries an existing view concern, attach to an unconstrained section.
                        sections=entry['variants'][language]['required_sections']
                        concerned={v['section_key'] for v in entry['variants'][language]['view_concerns']}
                        attach_key=next(s['key'] for s in sections if s['key'] not in concerned)
                        for p in projects:
                            asset=(p/created['path']).parent/'assets/transport-context.svg'
                            asset.parent.mkdir(exist_ok=True)
                            asset.write_bytes((KIT/'catalog/shared/mixed/assets/system-context.svg').read_bytes())
                        fig={'asset':'assets/transport-context.svg','title':'虚构边界' if language=='zh-CN' else 'Fictional boundary',
                             'Question':'边界包含什么？' if language=='zh-CN' else 'What is inside the boundary?',
                             'Model view':'Example::SystemContext','Companions':'契约文字。' if language=='zh-CN' else 'Contract prose.',
                             'Status':'external-tool; designed-only','Reading':'仅表达边界。' if language=='zh-CN' else 'Boundary only.',
                             'Limits':'不证明执行结果。' if language=='zh-CN' else 'No execution evidence.'}
                        attach={'figure':fig,'expected_sha256':updated['sha256']}
                        attached=core.attach_figure(py,**route,section_key=attach_key,**attach,profile_root=KIT)
                        self.assertEqual(self.cli(cli,'attach_figure',{**route,'section_key':attach_key},attach),attached)
                        self.assertEqual(await self.call(client,'attach_figure',{**route,'section_key':attach_key,**attach}),attached)
                        final=core.check_documents(py,profile_root=KIT,**selected)
                        self.assertTrue(final['ok'],final)
                        self.assertEqual(await self.call(client,'check_documents',selected),final)
                        self.assertEqual(self.cli(cli,'check_documents',{**selection(f),'language':language}),final)
                        for p in projects:
                            self.assertEqual((p/created['path']).read_bytes(),(py/created['path']).read_bytes())
                        before=[snapshot(p) for p in projects]
                        for p in projects:
                            try:core.update_section(p,**route,section_key=key,**update,profile_root=KIT)
                            except DocumentKitError as error:self.assertEqual(error.code,'stale-document')
                            else:self.fail('Stale update was accepted')
                        self.assertEqual([snapshot(p) for p in projects],before)
                        matrix.append({'template_id':f['template_id'],'language':language,'created_sha256':created['sha256'],
                                       'updated_sha256':updated['sha256'],'attached_sha256':attached['sha256'],
                                       'fixture_sha256':sha_bytes((KIT/entry['variants'][language]['resources']['fixture']).read_bytes()),
                                       'final_snapshot':snapshot(py),'all_three_transports_equal':True,'stale_nonwrite':True})
        self.assertEqual(len(matrix),42)
        write_json(EVIDENCE/'bdk05-operation-matrix.json',{'contract_sha256':contract['contract_sha256'],'variants':matrix})

    async def test_bdk05_168_resource_reads_and_error_integrity(self):
        project=self.root/'project';project.mkdir()
        pack=self.root/'copied-pack';shutil.copytree(KIT,pack,ignore=shutil.ignore_patterns('__pycache__','*.egg-info'))
        contract=core.get_contract(pack)
        async with stdio_client(self.server(project,pack)) as (read,write), ClientSession(read,write) as client:
            await client.initialize()
            self.assertEqual(await self.call(client,'get_contract',{}),contract)
            self.assertEqual(self.cli(project,'get_contract',{},pack=pack),contract)
            catalog=await client.read_resource('xdesign-doc-kit://catalog')
            self.assertEqual(json.loads(catalog.contents[0].text),contract['catalog'])
            templates=await client.list_resource_templates()
            self.transcript.append({'transport':'mcp-stdio','resource_templates':templates.model_dump(mode='json')})
            matrix=[]
            for item in contract['resource_index']:
                uri=f"xdesign-doc-kit://entry/{item['template_id']}/{item['language']}/{item['kind']}"
                result=await client.read_resource(uri)
                text=result.contents[0].text
                self.assertEqual(text.encode(),(pack/item['path']).read_bytes())
                self.assertEqual(sha_bytes(text.encode()),item['sha256'])
                self.assertEqual(contract['resource_bytes'][item['path']],text)
                self.transcript.append({'transport':'mcp-stdio','uri':uri,'result':result.model_dump(mode='json')})
                matrix.append({**item,'actual_sha256':sha_bytes(text.encode())})
            for entry in contract['catalog']['entries']:
                for uri in (f"xdesign-doc-kit://template/{entry['document_class']}/{entry['template_id']}",
                            f"xdesign-doc-kit://template/{entry['document_class']}/{entry['template_id']}/en"):
                    r=await client.read_resource(uri)
                    self.assertEqual(r.contents[0].text,contract['templates'][entry['template_id']])
            base={'document_class':'requirements','name':'boundary'}
            self.assertEqual(await self.call(client,'plan_document',base),await self.call(client,'plan_document',{**base,'language':'en'}))
            before=snapshot(project)
            for language,code in [('zh','unknown-language'),('ZH-CN','unknown-language'),('zh_CN','unknown-language'),('zh--CN','invalid-request'),('中文','invalid-request'),('','invalid-request')]:
                args={**base,'language':language}
                try:core.plan_document(project,**args,profile_root=pack)
                except DocumentKitError as e:expected=e.as_dict()
                else:self.fail('Invalid selector accepted')
                self.assertEqual(expected['error']['code'],code)
                self.assertEqual(await self.call(client,'plan_document',args),expected)
                self.assertEqual(self.cli(project,'plan_document',args,pack=pack,exit=1),expected)
            for selector in ('target_root','project_root','profile_root','profile'):
                r=await self.call(client,'plan_document',{**base,selector:str(self.root),'language':'zh--CN'})
                self.assertEqual(r['error']['code'],'invalid-request')
            r=await self.call(client,'check_documents',{'selection':{**base,'layer':None,'language':'en'}})
            self.assertEqual(r['error']['code'],'invalid-request')
            r=self.cli(project,'plan_document',base,pack=pack,exit=2,extra=['--language','en','--language','zh-CN'])
            self.assertEqual(r['error']['code'],'invalid-request')
            for language,code in [('zh','unknown-language'),('中文','invalid-request')]:
                uri=f'xdesign-doc-kit://entry/generic-requirements/{language}/guide'
                try:await client.read_resource(uri)
                except Exception as e:
                    self.assertIn(code,str(e));self.transcript.append({'transport':'mcp-stdio','uri':uri,'error':str(e)})
                else:self.fail('Resource accepted invalid language')
            (pack/'README.md').write_text((pack/'README.md').read_text()+'\nDrift injection.\n')
            r=await self.call(client,'plan_document',base)
            self.assertEqual(r['error']['code'],'contract-drift')
            self.assertEqual(snapshot(project),before)
            write_json(EVIDENCE/'bdk05-resources.json',{'contract_sha256':contract['contract_sha256'],'resources':matrix,
                        'nonwrite_before':before,'nonwrite_after':snapshot(project),'real_mcp':True})


if __name__=='__main__':unittest.main()
