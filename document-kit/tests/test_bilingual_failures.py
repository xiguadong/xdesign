"""New-root I/O/publication outcomes over all three real consumer surfaces."""
from __future__ import annotations
import json,os,subprocess,sys,unittest
from pathlib import Path
from mcp import ClientSession,StdioServerParameters
from mcp.client.stdio import stdio_client
from _faults import fault
from _bilingual_support import KIT,CACHE,EVIDENCE,fixture_for,materialize,snapshot,temporary_directory,write_json
from xdesign_doc_kit import core,DocumentKitError


class BilingualFailureTests(unittest.IsolatedAsyncioTestCase):
    async def test_bdk05_new_root_failure_integrity_and_publication(self):
        contract=core.get_contract(KIT);entry=next(x for x in contract['catalog']['entries'] if x['template_id']=='generic-api-design')
        records=[]
        env={**os.environ,'PYTHONPATH':str(KIT),'PYTHONDONTWRITEBYTECODE':'1','TMPDIR':str(CACHE)}
        for language in ('en','zh-CN'):
            for mode in ('pack-read','model-read','staging','page-publish','orphan','unknown-cleanup'):
                with self.subTest(language=language,mode=mode),temporary_directory() as d:
                    root=Path(d);f=fixture_for(entry,language)
                    args={k:f[k] for k in ('document_class','name','layer','template_id','language','metadata','sections')}
                    projects=[root/s for s in ('python','cli','mcp')]
                    for p in projects:p.mkdir();materialize(p,f)
                    before=[snapshot(p) for p in projects]
                    try:
                        with fault(mode):core.create_document(projects[0],**args,profile_root=KIT)
                    except DocumentKitError as e:py=e.as_dict()
                    else:self.fail('Injected failure did not reject')
                    payload=root/'payload.json';write_json(payload,{k:f[k] for k in ('metadata','sections')})
                    cli_command=[sys.executable,str(KIT/'tests/_fault_driver.py'),mode,'cli','create-document',
                                 '--profile-root',str(KIT),'--target-root',str(projects[1]),'--class',f['document_class'],
                                 '--name',f['name'],'--layer',f['layer'],'--template-id',f['template_id'],'--language',language,'--input',str(payload)]
                    result=subprocess.run(cli_command,env=env,capture_output=True,text=True,timeout=90)
                    self.assertEqual(result.returncode,1,result.stdout+result.stderr)
                    cli=json.loads(result.stdout)
                    server=StdioServerParameters(command=sys.executable,args=[str(KIT/'tests/_fault_driver.py'),mode,'mcp',
                        '--project-root',str(projects[2]),'--profile',str(KIT)],env=env)
                    async with stdio_client(server) as (read,write),ClientSession(read,write) as client:
                        await client.initialize();response=await client.call_tool('create_document',args)
                        self.assertTrue(response.is_error)
                        raw=' '.join(getattr(c,'text','') for c in response.content);mcp=json.loads(raw[raw.index('{'):])
                    self.assertEqual(py,cli);self.assertEqual(py,mcp)
                    expected_state='page-only' if mode=='orphan' else 'unknown' if mode=='unknown-cleanup' else 'none'
                    self.assertEqual(py['outcome']['publication_state'],expected_state)
                    if expected_state=='none':self.assertEqual([snapshot(p) for p in projects],before)
                    else:
                        for p in projects:
                            self.assertTrue((p/py['outcome']['path']).is_file())
                            self.assertFalse(core.check_documents(p,profile_root=KIT)['ok'])
                    records.append({'language':language,'fault':mode,'arguments':args,'python':py,'cli_command':cli_command,
                                    'cli_exit':result.returncode,'cli':cli,'mcp_stdio':mcp,'before':before,
                                    'after':[snapshot(p) for p in projects]})
        write_json(EVIDENCE/'bdk05-current-failure-transcript.json',records)


if __name__=='__main__':unittest.main()
