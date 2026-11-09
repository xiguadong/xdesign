"""Current-release negative cases preserve target, registry and outside bytes."""
from __future__ import annotations
import copy
from pathlib import Path
import shutil
import unittest
from _bilingual_support import (KIT,EVIDENCE,create_from_fixture,error_code,fixture_for,
    sha_path,selection,snapshot,temporary_directory,write_json)
from xdesign_doc_kit import core


class BilingualSecurityTests(unittest.TestCase):
    def test_bdk04_identity_sentinel_and_assertion_precedence_nonwrite(self):
        contract=core.get_contract(KIT);entry=contract['catalog']['entries'][0];evidence=[]
        for language in ('en','zh-CN'):
            for mode in ('marker-zero','registry-zero','registry-disagreement','stale-before-assertion','selection-extra','payload-extra'):
                with self.subTest(language=language,mode=mode),temporary_directory() as d:
                    target=Path(d);f=fixture_for(entry,language);created=create_from_fixture(target,f)
                    page=target/created['path'];reg=target/'docs/.document-kit-managed.json'
                    if mode=='marker-zero':page.write_bytes(page.read_bytes().replace(created['contract_sha256'].encode(),b'0'*64,1))
                    if mode in ('registry-zero','registry-disagreement'):
                        import json
                        value=json.loads(reg.read_text())
                        value['entries'][0]['contract_sha256' if mode=='registry-zero' else 'language']='0'*64 if mode=='registry-zero' else ('en' if language=='zh-CN' else 'zh-CN')
                        write_json(reg,value)
                    before=snapshot(target);key=next(iter(f['sections']))
                    if mode=='selection-extra':
                        error_code(self,'invalid-request',core.check_documents,target,profile_root=KIT,selection={**selection(f),'language':language})
                    elif mode=='payload-extra':
                        bad=copy.deepcopy(f['metadata']);bad['language']=language
                        error_code(self,'missing-metadata',core.create_document,target,f['document_class'],'another',bad,f['sections'],profile_root=KIT,language=language)
                    else:
                        stale=mode=='stale-before-assertion'
                        error_code(self,'stale-document' if stale else 'registry-mismatch',core.update_section,target,f['document_class'],f['name'],key,'Caller content.', 'f'*64 if stale else sha_path(page),profile_root=KIT,language=('en' if language=='zh-CN' else 'zh-CN') if stale else None)
                    self.assertEqual(snapshot(target),before)
                    evidence.append({'language':language,'mode':mode,'before':before,'after':snapshot(target)})
        write_json(EVIDENCE/'bdk04-nonwrite-negatives.json',evidence)

    def test_bdk01_malformed_variant_types_remain_structured_errors(self):
        import json
        for mode in ("label-list", "label-object", "class-list", "figure-path-list"):
            with self.subTest(mode=mode),temporary_directory() as d:
                root=Path(d);pack=root/"pack";shutil.copytree(KIT,pack)
                target=root/"target";target.mkdir()
                if mode=="figure-path-list":
                    p=pack/"profile.json";value=json.loads(p.read_text());value["figure_block"]["templates"]["zh-CN"]=[]
                else:
                    p=pack/"catalog/manifest.json";value=json.loads(p.read_text());entry=value["entries"][0]
                    if mode=="class-list":entry["document_class"]=[]
                    else:
                        labels=entry["variants"]["zh-CN"]["required_sections"][0]["field_labels"]
                        labels[next(iter(labels))]=[] if mode=="label-list" else {}
                # Preserve declaration order; this case targets types, not label ordering.
                p.write_text(json.dumps(value,ensure_ascii=False,indent=2)+"\n")
                before=snapshot(target)
                error_code(self,"invalid-profile" if mode=="figure-path-list" else "invalid-catalog",core.plan_document,target,"requirements","boundary",profile_root=pack)
                self.assertEqual(snapshot(target),before)

    def test_bdk01_current_resource_security_and_symlink_binding(self):
        evidence=[]
        for mode in ('template-html','guide-url','example-zero','fixture-extra','symlink','extra-declaration'):
            with self.subTest(mode=mode),temporary_directory() as d:
                root=Path(d);pack=root/'pack';shutil.copytree(KIT,pack)
                target=root/'target';target.mkdir();outside=root/'outside';outside.write_text('Outside bytes.\n')
                resource=pack/'catalog/entries/generic-requirements/zh-CN'
                if mode=='template-html':
                    p=resource/'template.md';p.write_text(p.read_text()+'\n<script>bad()</script>\n')
                elif mode=='guide-url':
                    p=resource/'guide.md';p.write_text(p.read_text()+'\n[unsafe](javascript:alert(1))\n')
                elif mode=='example-zero':
                    p=resource/'example.md';p.write_text(p.read_text()+'\n'+'0'*64+'\n')
                elif mode=='fixture-extra':
                    import json
                    p=resource/'fixture.json';f=json.loads(p.read_text());f['language_override']='en';write_json(p,f)
                elif mode=='symlink':
                    p=resource/'guide.md';p.unlink();p.symlink_to(outside)
                else:
                    import json
                    p=pack/'catalog/manifest.json';manifest=json.loads(p.read_text())
                    manifest['entries'][0]['variants']['zh-CN']['resources']['unlisted']='catalog/entries/generic-requirements/zh-CN/unlisted.md'
                    write_json(p,manifest)
                before=snapshot(target);outside_hash=sha_path(outside)
                error_code(self,{'template-html':'unsafe-markdown','guide-url':'unsafe-url'}.get(mode,'invalid-catalog'),core.plan_document,target,'requirements','boundary',profile_root=pack,language='zh-CN')
                self.assertEqual(snapshot(target),before);self.assertEqual(sha_path(outside),outside_hash)
                evidence.append({'mode':mode,'before':before,'after':snapshot(target),'outside_sha256':outside_hash})
        write_json(EVIDENCE/'bdk01-security-negatives.json',evidence)


if __name__=='__main__':unittest.main()
