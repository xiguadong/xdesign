"""Focused CLI documentation views derived from declared SysML selections.

These projections show action order and documented responsibilities, not native
Message usages, allocations or executed runs. No persistent export IDs or cache
artifacts are required. Source identities and expression evidence travel in SVG.
"""

from __future__ import annotations

import json
from html import escape

from .declared_views import DeclaredViews
from .documentation_views import DocumentationProjection
from .renderer import _ids, _name, _one

CLI_KINDS = {"cli_packages", "cli_timeline", "cli_swimlane"}


class CliProjection:
    def __init__(self, raw, guards, target):
        self.p = DocumentationProjection(raw, "activity")
        self.key = self.p.target(target)
        self.selection = DeclaredViews(raw, guards).selection(self.key)
        self.scope = set().union(*(self.p.descendants(k) for k in self.selection["exposed"]))
        self.used = {}
        self.relationships = []
        self.presentation = self.p.presentation(self.key)
        if not all(self.presentation.get(key) for key in ("label", "summary", "question")):
            raise ValueError("CLI views require label, summary and question presentation metadata.")

    def keep(self, key, context=None):
        if key not in self.scope:
            raise ValueError(f"CLI source {key} is outside the declared view selection.")
        e = self.p.index[key]
        self.used.setdefault(key, {"native_id": key, "qualifiedName": e.get("qualifiedName"),
                                   "declaredName": e.get("declaredName"), "type": e["@type"],
                                   "source_context": context})
        return key

    def source(self, qname, name=None):
        key = self.p.target(qname)
        if name:
            candidates = [k for k in self.p.descendants(key)
                          if self.p.index[k].get("declaredName") == name
                          and self.p.index[k]["@type"] in {"ActionUsage", "PerformActionUsage"}]
            if len(candidates) != 1:
                raise ValueError(f"Expected one declared action {qname} / {name}.")
            key = candidates[0]
        return self.keep(key, qname)

    def expression(self, qname):
        key = self.source(qname)
        values = self.p.text(key)
        if len(values) != 1:
            raise ValueError(f"Expected one exported expression for {qname}.")
        self.used[key]["source_expression"] = values[0]
        return " ".join(values[0].split())

    def literal(self, qname):
        key = self.source(qname)
        values = [self.p.index[k].get("value") for k in self.p.descendants(key)
                  if self.p.index[k]["@type"].startswith("Literal")]
        if len(values) != 1:
            raise ValueError(f"Expected one exported literal for {qname}.")
        self.used[key]["source_value"] = values[0]
        return values[0]

    def guard(self, qname, expected):
        root = self.source(qname)
        candidates = [k for k in self.p.descendants(root)
                      if self.p.index[k]["@type"] == "IfActionUsage"
                      and [" ".join(x.split())
                           for child in self.p.children[k]
                           if self.p.index[child]["@type"] == "Expression"
                           for x in self.p.text(child)] == [expected]]
        if len(candidates) != 1:
            raise ValueError(f"Expected CLI guard {expected!r} under {qname}.")
        key = self.keep(candidates[0], qname)
        self.used[key]["source_expression"] = expected
        return key

    def succession(self, source, target):
        candidates = [k for k in self.scope
                      if self.p.index[k]["@type"] == "SuccessionAsUsage"
                      and _ids(self.p.index[k].get("relatedElement")) == [source, target]]
        if len(candidates) != 1:
            raise ValueError("Expected one native succession between projected CLI actions.")
        self.keep(candidates[0])
        return candidates[0]

    def validate_interactions(self):
        """Fail instead of presenting stale action order or action-call targets."""
        agent = "XDesignWorkflow::AgentStageWork"
        check = "XDesignWorkflow::StageCli::check"
        progress = "XDesignArchitecture::Controller::progressWish"
        agent_guard = self.guard(agent, "not observe.completed")
        check_guard = self.guard(check, "not observe.completed")
        authorized = self.guard(progress, "selectCurrentExecution.currentRecord.completionAuthorized")
        lookup = lambda owner, name: self.source(owner, name)
        paths = [
            [lookup(agent, "observe"), agent_guard],
            [lookup(agent, "performWork"), lookup(agent, "submit")],
            [lookup(check, "observe"), check_guard, lookup(check, "returnWork")],
            [lookup(check, "collectOutputs"), lookup(check, "record"), lookup(check, "assessAndAdvance")],
            [lookup(progress, "selectCurrentExecution"), lookup(progress, "evaluateCheckpoints"),
             authorized, lookup(progress, "updateTraceStatus")],
            [lookup(progress, "decideNextStep"), lookup(progress, "updateExecutionState")],
        ]
        for path in paths:
            for source, target in zip(path, path[1:]):
                native = self.succession(source, target)
                self.relationships.append({"native_id": native, "type": "SuccessionAsUsage",
                                           "source": source, "target": target})
        for owner, name, target in (
            (agent, "observe", "XDesignWorkflow::StageCli::next"),
            (agent, "submit", check), (check, "observe", "XDesignWorkflow::StageCli::next"),
            (check, "returnWork", "XDesignWorkflow::StageCli::next"),
            (check, "record", "XDesignArchitecture::Controller::recordActivity"),
            (check, "assessAndAdvance", progress),
        ):
            source = lookup(owner, name)
            expected = self.source(target)
            references = []
            for relation in _ids(self.p.index[source].get("ownedSubsetting")):
                entry = self.p.index[relation]
                if entry.get("isImplied"):
                    continue
                target_id = _one(entry.get("subsettedFeature"))
                chain = _ids(self.p.index.get(target_id, {}).get("chainingFeature"))
                if target_id == expected or (chain and chain[-1] == expected):
                    references.append(relation)
            if len(references) != 1:
                raise ValueError(f"Expected declared CLI call {owner} / {name} to {target}.")
            self.keep(references[0], owner)
            self.relationships.append({"native_id": references[0], "type": "Subsetting",
                                       "source": source, "target": expected})

    def flow_scope(self):
        base = "XDesignArchitecture::WishVerifyFlow"
        designed = "XDesignDetailedDesign::DesignedWishVerifyFlow"
        self.source(base)
        self.source(designed)
        initial = self.expression(base + "::initialStage").strip("'")
        successor = self.expression(base + "::" + initial + "::nextStages").strip("'")
        terminal = base + "::'" + successor + "'"
        if self.expression(terminal + "::nextStages") != "()" or not self.literal(terminal + "::completesFlow"):
            raise ValueError("CLI example requires the selected two-stage terminal flow.")
        gates = [self.expression(designed + "::" + initial + "::exitCheckpoints"),
                 self.expression(designed + "::'" + successor + "'::exitCheckpoints")]
        for gate, owner in zip(gates, (designed, base)):
            for authority in ("agentMayJudge", "humanMayJudge"):
                if self.literal(owner + "::" + gate + "::" + authority) is not True:
                    raise ValueError("WishVerifyFlow scope requires the declared agent/human judging permission.")
        # The first gate is typed by the designed semantic checkpoint; the
        # inherited terminal gate supplies its classification in the base flow.
        kind = self.expression("XDesignDetailedDesign::WishVerifySemanticCheckpoint::kind")
        if kind != "CheckKind::semantic" or self.expression(base + "::" + gates[1] + "::kind") != "CheckKind::semantic":
            raise ValueError("WishVerifyFlow CLI example requires semantic gates.")
        return (f"{initial}  — {gates[0]} →  {successor}  — {gates[1]} →  completed",
                "Same iteration at both stages · semantic judgment by a permitted agent or human")

    def metadata(self, kind):
        return {"name": self.p.index[self.key].get("qualifiedName"), "rendering": kind,
                "origin": "declared GeneralView with focused action projection",
                "exposedIds": sorted(self.selection["exposed"]),
                "source_elements": list(self.used.values()),
                "source_relations": self.relationships,
                "native_sequence_messages": False, "native_allocations": False}


def _scope_strip(parts, context, width):
    lines = context.flow_scope()
    parts.append(f'<g font-family="Arial, sans-serif"><rect x="40" y="126" width="{width-80}" height="94" rx="12" fill="#fff" stroke="#b9cae5"/>')
    for y, line, size in ((158, lines[0], 23), (192, lines[1], 20)):
        parts.append(f'<text x="{width/2}" y="{y}" text-anchor="middle" fill="#172b43" font-size="{size}">{escape(line)}</text>')
    parts.append('</g>')


def render_cli_view(raw, guards, target, kind):
    if kind not in CLI_KINDS:
        raise ValueError(f"Unsupported CLI view kind {kind!r}.")
    context = CliProjection(raw, guards, target)
    if kind != "cli_packages":
        context.validate_interactions()
    svg = {"cli_packages": _packages, "cli_timeline": _timeline,
           "cli_swimlane": _swimlane}[kind](context)
    info = context.metadata(kind)
    metadata = '<metadata>' + escape(json.dumps(info, ensure_ascii=False)) + '</metadata>'
    return svg.replace('</svg>', metadata + '</svg>'), info


def _packages(context):
    DATA = list(context.p.index.values())
    BYQ = {e.get('qualifiedName'): e for e in DATA if e.get('qualifiedName')}
    selected={}
    def esc(s):return escape(str(s),quote=True)
    def pick(q):
     e=BYQ[q];context.source(q);selected[q]={'native_id':e['@id'],'native_type':e['@type'],'owner_id':e.get('owner',{}).get('@id')};return e
    W='XDesignWorkflow';A='XDesignArchitecture';C='XDesignComposition';V='XDesignVerification'
    if BYQ[W+'::StageCli']['@id'] not in _ids(BYQ[C+'::xdesign::cli']['type']): raise ValueError('xdesign.cli must be typed by StageCli')
    if BYQ[W+'::StageCli']['@id'] not in _ids(BYQ[V+'::StageCliInteractionVerification::cli']['type']): raise ValueError('CLI verification subject must be typed by StageCli')
    relationships=[]
    def imp(src,dst):
     es=[e for e in DATA if e['@type']=='NamespaceImport' and e.get('owner',{}).get('@id')==BYQ[src]['@id'] and e.get('importedNamespace',{}).get('@id')==BYQ[dst]['@id']]
     if len(es)!=1: raise ValueError('Expected one native package import')
     relationships.append({'kind':'private namespace import','native_id':es[0]['@id'],'source':src,'target':dst})
     return es[0]['@id']
    for src,dst in [(C,W),(C,A),(W,A)]:imp(src,dst)
    relationships.append({'kind':'verification subject type','source':V+'::StageCliInteractionVerification::cli','target':W+'::StageCli','native_id':pick(V+'::StageCliInteractionVerification::cli')['@id']})
    for q in [A+'::Stage::goal',A+'::Stage::workInstructions',A+'::Stage::inputArtifacts',A+'::Stage::expectedOutputs',A+'::Stage::exitCheckpoints',A+'::Controller::recordActivity',A+'::Controller::progressWish',A+'::StageRecord',A+'::ActivityRecord',A+'::CheckpointAssessment',A+'::FlowExecution',W+'::StageCli::controller',W+'::StageCli::next',W+'::StageCli::check',C+'::xdesign::cli']:
     pick(q)
    # Bind connector provenance, retained without claiming an extra package dependency.
    bindings = []
    for e in DATA:
        if e['@type'] != 'BindingConnectorAsUsage' or _one(e.get('owner')) != BYQ[C+'::xdesign']['@id']:
            continue
        ends = [context.p.index[k] for k in _ids(e.get('endFeature'))]
        if len(ends) != 2:
            continue
        chain = _ids(ends[0].get('chainingFeature'))
        refs = [context.p.index[k] for k in _ids(ends[1].get('ownedReferenceSubsetting'))]
        if (chain == [BYQ[C+'::xdesign::cli']['@id'], BYQ[W+'::StageCli::controller']['@id']]
                and any(_one(r.get('referencedFeature')) == BYQ[A+'::XDesign::controller']['@id'] for r in refs)):
            bindings.append(e['@id'])
    if len(bindings) != 1:
        raise ValueError('Expected the native cli.controller = controller binding.')
    context.keep(bindings[0], C+'::xdesign')
    relationships.append({'kind':'binding connector','native_id':bindings[0],
                          'source_expression':'bind cli.controller = controller;'})
    S=['<svg xmlns="http://www.w3.org/2000/svg" width="1200" height="1030" viewBox="0 0 1200 1030">','<title>Current SysML integration of the two-command stage CLI</title>','<desc>Focused package projection. Ownership is shown by containment. Arrows are source import or subject-type dependencies. Designed-only CLI; verification is planned.</desc>', '<defs><marker id="arrow" markerWidth="10" markerHeight="10" refX="9" refY="5" orient="auto"><path d="M1 1 L9 5 L1 9" fill="none" stroke="#52647a" stroke-width="1.5"/></marker></defs>','<rect width="1200" height="1030" fill="#f8fafc"/>','<g font-family="Arial, sans-serif" fill="#172b43">']
    def text(x,y,s,size=19,weight='normal',fill='#172b43',anchor='start'):
     S.append(f'<text x="{x}" y="{y}" font-size="{size}" font-weight="{weight}" fill="{fill}" text-anchor="{anchor}">{esc(s)}</text>')
    def group(q):
     e=pick(q);S.append(f'<g id="n-{e["@id"]}" data-native-id="{e["@id"]}"><title>{esc(q)} | {e["@id"]}</title>')
    def end():S.append('</g>')
    def package(x,y,w,h,q,color,stroke):
     group(q);S.append(f'<path d="M{x} {y+34} V{y} H{x+330} V{y+34} H{x+w} V{y+h} H{x} Z" fill="{color}" stroke="{stroke}" stroke-width="2"/>');text(x+18,y+25,q,23,'bold');end()
    def panel(x,y,w,h,q,heading,lines,color='#ffffff',stroke='#cbd5e1'):
     group(q);S.append(f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="9" fill="{color}" stroke="{stroke}"/>');text(x+16,y+29,heading,21,'bold')
     for i,line in enumerate(lines):text(x+16,y+59+i*27,line,18)
     end()
    def edge(path,native_id,label,x,y):
     S.append(f'<g data-native-id="{native_id}"><title>{native_id}</title><path d="{path}" fill="none" stroke="#52647a" stroke-width="2" stroke-dasharray="7 5" marker-end="url(#arrow)"/>')
     text(x,y,label,18,anchor='middle');end()
    text(40,44,context.presentation['label'],31,'bold')
    text(40,78,context.presentation['summary'],19,fill='#52647a')
    package(40,120,500,295,C,'#f1eef9','#8770b2')
    panel(60,176,460,216,C+'::xdesign','xdesign : XDesign',[
     'cli : StageCli',
     'bind cli.controller = controller;',
     'Existing controller and flow portfolio',
     'remain owned by xdesign.'
    ],color='#ffffff',stroke='#c9c0e0')
    package(660,120,500,500,W,'#eaf2ff','#527bb6')
    panel(680,176,460,132,W+'::StageCli','StageCli',[
     'next(wishId) → current work / complete',
     'check(wishId) → record, assess, advance',
     'controller : Controller'
    ],stroke='#b9cae5')
    panel(680,324,460,132,W+'::StageWorkOrder','StageWorkOrder',[
     'Goal, instructions, inputs and outputs',
     'Stage visit, revision and exit checkpoints',
     'Current assessments and corrections'
    ],stroke='#b9cae5')
    panel(680,472,460,125,W+'::AgentStageWork','AgentStageWork',[
     'observe → performWork → submit',
     'Agent produces workspace artifacts',
     'xdesign owns records and progression'
    ],color='#edf7ef',stroke='#6c9876')
    package(40,565,500,410,A,'#f1eef9','#8770b2')
    panel(60,621,460,125,A+'::Stage','Stage',[
     'goal · workInstructions',
     'inputArtifacts · expectedOutputs',
    ],stroke='#c9c0e0')
    # Include source field on its own line below the source-backed panel heading.
    text(76,734,'exitCheckpoints : Checkpoint [0..*]',18)
    panel(60,760,460,102,A+'::Controller','Controller',[
     'recordActivity(trace, activity)',
     'progressWish(trace)'
    ],stroke='#c9c0e0')
    panel(60,876,460,81,A+'::StageRecord','Execution records',[
     'StageRecord · ActivityRecord · Assessment'
    ],stroke='#c9c0e0')
    package(660,740,500,225,V,'#f5f6f8','#93a0b2')
    panel(680,796,460,146,V+'::StageCliInteractionVerification','StageCliInteractionVerification',[
     'subject cli : StageCli',
     'Two-flow interaction check scope',
     'Executed results live in wish records'
    ],stroke='#cbd5e1')
    edge('M540 249 H660',relationships[0]['native_id'],'«import»',600,235)
    edge('M290 415 V565',relationships[1]['native_id'],'«import»',344,494)
    edge('M660 530 H598 V655 H540',relationships[2]['native_id'],'«import»',602,512)
    edge('M910 740 V620',relationships[3]['native_id'],'subject type',985,682)
    text(40,1003,'Containment = source ownership. Dashed arrows = declared import or typed-subject dependency.',18,fill='#52647a')
    S.extend(['</g>','</svg>'])

    context.relationships.extend(relationships)
    return '\n'.join(S)


def _timeline(context):
    p = context.p
    used = context.used
    source = context.source
    context.guard("XDesignWorkflow::StageCli::check", "not observe.completed")
    context.guard("XDesignArchitecture::Controller::progressWish",
                  "selectCurrentExecution.currentRecord.completionAuthorized")
    agent = source('XDesignWorkflow::AgentStageWork')
    cli = source('XDesignWorkflow::StageCli')
    controller = source('XDesignArchitecture::Controller')
    next_action = source('XDesignWorkflow::StageCli::next')
    check = source('XDesignWorkflow::StageCli::check')
    observe = source('XDesignWorkflow::AgentStageWork::observe')
    perform = source('XDesignWorkflow::AgentStageWork', 'performWork')
    submit = source('XDesignWorkflow::AgentStageWork', 'submit')
    observe_check = source('XDesignWorkflow::StageCli::check::observe')
    collect = source('XDesignWorkflow::StageCli::check', 'collectOutputs')
    record_call = source('XDesignWorkflow::StageCli::check', 'record')
    progress_call = source('XDesignWorkflow::StageCli::check', 'assessAndAdvance')
    record = source('XDesignArchitecture::Controller::recordActivity')
    progress = source('XDesignArchitecture::Controller::progressWish')
    select = source('XDesignArchitecture::Controller::progressWish::selectCurrentExecution')
    evaluate = source('XDesignArchitecture::Controller::progressWish::evaluateCheckpoints')
    decide = source('XDesignArchitecture::Controller::progressWish', 'decideNextStep')
    update = source('XDesignArchitecture::Controller::progressWish', 'updateExecutionState')
    retain = source('XDesignArchitecture::Controller::progressWish', 'retainCurrentStage')
    status = source('XDesignArchitecture::Controller::progressWish::updateTraceStatus')
    return_work = source('XDesignWorkflow::StageCli::check::returnWork')
    next_work = source('XDesignWorkflow::StageCli::next::work')
    assessments = source('XDesignWorkflow::StageCli::check::assessments')
    check_work = source('XDesignWorkflow::StageCli::check::work')
    completed = source('XDesignWorkflow::StageCli::check::completed')

    relations = []
    for action in (observe, submit, observe_check, record_call, progress_call, return_work):
        for k in _ids(p.index[action].get('ownedSubsetting')):
            r = p.index[k]
            relations.append({'native_id': k, 'type': r['@type'], 'source': action,
                              'subsettedFeature': r.get('subsettedFeature')})
    for k in p.descendants(agent) | p.descendants(check) | p.descendants(progress):
        e = p.index[k]
        if e['@type'] == 'SuccessionAsUsage':
            relations.append({'native_id': k, 'type': e['@type'],
                              'source': e.get('source'), 'target': e.get('target'),
                              'relatedElement': e.get('relatedElement')})

    INK = '#172b43'
    MUTED = '#52657b'
    BLUE = '#527bb6'
    GREEN = '#6c9876'
    PURPLE = '#8770b2'
    parts = []
    def add(s): parts.append(s)
    def text(x, y, value, size=21, weight=400, anchor='start', color=INK):
        add(f'<text x="{x}" y="{y}" font-size="{size}" font-weight="{weight}" text-anchor="{anchor}" fill="{color}">{escape(value)}</text>')
    def group(keys, note):
        if isinstance(keys, str): keys = [keys]
        add(f'<g data-native-id="{keys[0]}" data-native-ids="{escape(" ".join(keys))}"><title>{escape(note)}')
        for k in keys:
            e = used[k]
            add(escape('\n' + (e['qualifiedName'] or (e['source_context'] + ' / ' + str(e['declaredName']) + ' (native qualifiedName absent)')) + ' [' + k + ']'))
        add('</title>')
    def end(): add('</g>')
    def rect(x,y,w,h,fill='#fff',stroke='#cbd5e1',radius=12):
        add(f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="{radius}" fill="{fill}" stroke="{stroke}" stroke-width="1.6"/>')
    def path(d,color=INK,dashed=False,arrow=True):
        dash = ' stroke-dasharray="8 6"' if dashed else ''
        marker = ' marker-end="url(#arrow)"' if arrow else ''
        add(f'<path d="{d}" stroke="{color}" stroke-width="2.2" fill="none"{dash}{marker}/>')
    def call(x1,x2,y,label,keys,size=21,dashed=False):
        group(keys, 'Projected action call' if not dashed else 'Projected output/control return; not a native Message usage')
        path(f'M{x1} {y} H{x2}', dashed=dashed)
        text((x1+x2)/2, y-14, label, size=size, weight=600 if not dashed else 400, anchor='middle')
        end()
    def box(x,y,w,h,lines,keys,fill,stroke):
        group(keys, 'Declared action or parameter projection')
        rect(x,y,w,h,fill,stroke)
        for i,line in enumerate(lines):
            text(x+w/2,y+30+i*28,line,22 if i==0 else 18,600 if i==0 else 400,'middle')
        end()

    add('<svg xmlns="http://www.w3.org/2000/svg" width="1200" height="1700" viewBox="0 0 1200 1700" role="img" aria-labelledby="title desc">')
    add('<title id="title">CLI integration timeline: one agent iteration</title>')
    add('<desc id="desc">Action-order projection of current SysML actions. Two public commands call an internal controller. No native CLI sequence messages or runtime execution are claimed.</desc>')
    add('<defs><marker id="arrow" markerWidth="10" markerHeight="10" refX="8" refY="4" orient="auto" markerUnits="strokeWidth"><path d="M0 0 L8 4 L0 8" fill="none" stroke="#172b43" stroke-width="1.2"/></marker></defs>')
    add('<g font-family="Helvetica, Arial, sans-serif">')
    add('<rect width="1200" height="1700" fill="#f8fafc"/>')
    text(44,51,context.presentation['label'] + ' · one stage iteration',32,700)
    text(44,84,context.presentation['summary'],20,color=MUTED)
    text(44,114,'One active visit; an already-completed run returns without mutations.',18,color=MUTED)
    _scope_strip(parts, context, 1200)
    add('<g transform="translate(0 130)">')

    rect(548,140,620,1158,'#f1f5fa','#cbd5e1',14)
    text(858,167,'xdesign system',19,600,'middle',MUTED)
    box(70,182,260,72,['Agent','AgentStageWork role'],agent,'#edf7ef',GREEN)
    box(570,182,260,72,['cli : StageCli','public command boundary'],cli,'#eaf2ff',BLUE)
    box(875,182,260,72,['controller','existing Controller'],controller,'#f1eef9',PURPLE)
    for x in (200,700,1005):
        path(f'M{x} 254 V1278','#9babbd',dashed=True,arrow=False)

    call(200,700,310,'1  xdesign next WISH_ID',[observe,next_action])
    call(700,200,375,'Current StageWorkOrder',[next_work],20,True)
    box(70,414,340,85,['performWork','Agent edits workspace artifacts'],perform,'#edf7ef',GREEN)
    text(70,525,'Continue only when work is present.',18,color=MUTED)
    call(200,700,578,'2  xdesign check WISH_ID [--result FILE]',[submit,check],20)

    box(570,616,260,78,['observe → next','Re-read the active visit'],[observe_check,next_action],'#eaf2ff',BLUE)
    text(864,645,'If not completed:',18,600,color=MUTED)
    text(864,671,'collect, record,',18,color=MUTED)
    text(864,697,'and progress.',18,color=MUTED)
    path('M700 694 V720',BLUE)
    box(570,720,260,78,['collectOutputs','Capture declared outputs'],collect,'#eaf2ff',BLUE)
    call(700,1005,851,'recordActivity(trace, activity)',[record_call,record],18)
    call(700,1005,916,'progressWish(trace)',[progress_call,progress],20)
    box(866,946,278,138,['Controller actions','Select current execution','Evaluate checkpoints','Advance or retain visit'],[select,evaluate,decide,update,retain],'#f1eef9',PURPLE)
    text(1005,1112,'Then updateTraceStatus',18,600,'middle',PURPLE)
    call(1005,700,1147,'Control returns; trace updated',[progress,status],18,True)
    box(570,1170,260,72,['returnWork → next','Read the resulting state'],[return_work,next_action],'#eaf2ff',BLUE)
    call(700,200,1288,'Assessed-visit results + work / completion',[assessments,check_work,completed],20,True)

    text(44,1320,'What check returns',22,700)
    box(44,1343,354,100,['Blocked','Same visit + correction reasons','Agent corrects its artifacts'],[retain,check_work],'#fff4e8','#bf8a51')
    box(422,1343,354,100,['Advanced','Successor work order','Agent performs the next work'],[update,check_work],'#eaf2ff',BLUE)
    box(800,1343,354,100,['Completed','No work order','End of this stage loop'],[completed],'#edf7ef',GREEN)
    group(agent, 'Repetition is stated in AgentStageWork documentation; there is no native loop back-edge.')
    text(44,1480,'Repeat with returned work is documented intent; only one iteration is structurally declared.',18,600,color='#97601e')
    end()
    text(44,1514,'Action-order projection, not a native event timeline or execution log. Dashed arrows show returns.',18,color=MUTED)
    text(44,1544,'Source: AgentStageWork · StageCli.check · Controller.progressWish. Design projection; see the local CLI contract.',18,color=MUTED)
    add('</g></g></svg>')
    context.relationships.extend(sorted(relations, key=lambda relation: relation['native_id']))
    return '\n'.join(parts)



def _swimlane(context):
    P = context.p
    N = {}
    def action(alias, qname, name=None):
        N[alias] = P.index[context.source(qname, name)]
    agent = "XDesignWorkflow::AgentStageWork"
    check = "XDesignWorkflow::StageCli::check"
    progress = "XDesignArchitecture::Controller::progressWish"
    action("agentObserve", agent + "::observe")
    action("checkObserve", check + "::observe")
    action("check", check)
    action("next", "XDesignWorkflow::StageCli::next")
    action("recordActivity", "XDesignArchitecture::Controller::recordActivity")
    for name in ("performWork", "submit"):
        action(name, agent, name)
    for name in ("collectOutputs", "record", "assessAndAdvance", "returnWork"):
        action(name, check, name)
    for name in ("selectCurrentExecution", "evaluateCheckpoints", "decideNextStep", "updateExecutionState", "retainCurrentStage", "updateTraceStatus"):
        action(name, progress, name)
    N["checkGuard"] = P.index[context.guard(check, "not observe.completed")]
    N["authorization"] = P.index[context.guard(progress, "selectCurrentExecution.currentRecord.completionAuthorized")]
    COL={'agent':('#edf7ef','#6c9876'),'cli':('#eaf2ff','#527bb6'),'controller':('#f1eef9','#8770b2')}
    INK='#172b43';MUTED='#53677b';ORANGE='#bf721e'
    W,H=1260,2010
    s=[f'<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{H}" viewBox="0 0 {W} {H}" role="img" aria-labelledby="title desc">', '<title id="title">Two-command CLI responsibility swimlane</title>', '<desc id="desc">Source-bound action and responsibility projection from the current xdesign SysML design. Native action order is expanded across Agent, StageCli and Controller lanes. Orange repeat is documented intent only.</desc>', '<defs><marker id="arrow" markerWidth="10" markerHeight="8" refX="9" refY="4" orient="auto"><path d="M0,0 L10,4 L0,8" fill="#53677b"/></marker><marker id="orangeArrow" markerWidth="10" markerHeight="8" refX="9" refY="4" orient="auto"><path d="M0,0 L10,4 L0,8" fill="#bf721e"/></marker></defs>',f'<rect width="{W}" height="{H}" fill="#f8fafc"/>']
    manifest_nodes=[];manifest_edges=[]
    esc=lambda x:escape(str(x),quote=True)
    def txt(x,y,lines,size=22,fill=INK,anchor='middle',weight=400,gap=27):
        if isinstance(lines,str):lines=[lines]
        s.append(f'<text x="{x}" y="{y}" text-anchor="{anchor}" fill="{fill}" font-family="Arial, sans-serif" font-size="{size}" font-weight="{weight}">')
        for i,line in enumerate(lines):s.append(f'<tspan x="{x}" dy="{0 if i==0 else gap}">{esc(line)}</tspan>')
        s.append('</text>')
    def native(aliases,label):
        if isinstance(aliases,str):aliases=[aliases]
        ids=[N[a]['@id'] for a in aliases]
        records=[]
        for a in aliases:
            n=N[a];q=n.get('qualifiedName');name=n.get('declaredName') or n.get('name');source_context=None
            if q is None:
                cur=n
                while cur.get('owner') and not source_context:
                    cur=P.index[cur['owner']['@id']];source_context=cur.get('qualifiedName')
            records.append({'id':n['@id'],'qualifiedName':q,'declaredName':name,'enclosingNamedContext':source_context})
        manifest_nodes.append({'label':label,'nativeElements':records})
        detail=' | '.join((r['qualifiedName'] or (str(r['enclosingNamedContext'])+' / '+str(r['declaredName'])+' [qualifiedName=null]')) for r in records)
        detail += ''.join(' | ' + context.used[key]['source_expression'] for key in ids
                          if context.used[key].get('source_expression'))
        s.append(f'<g data-native-id="{esc(ids[0])}" data-native-ids="{esc(" ".join(ids))}"><title>{esc(detail)}</title>')
    def box(x,y,w,h,lines,lane,aliases,size=22):
        native(aliases,' / '.join(lines));fill,stroke=COL[lane]
        s.append(f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="13" fill="{fill}" stroke="{stroke}" stroke-width="2"/>')
        txt(x+w/2,y+h/2-(len(lines)-1)*14+8,lines,size=size,gap=28)
        s.append('</g>')
    def edge(path,label=None,x=None,y=None,kind='action-projection',refs=None,orange=False,dashed=False):
        color=ORANGE if orange else MUTED
        dash=' stroke-dasharray="8 7"' if dashed or orange else ''
        marker='orangeArrow' if orange else 'arrow'
        s.append(f'<g data-projection-basis="{esc(kind)}" data-native-ids="{esc(" ".join(refs or []))}"><title>{esc(kind)}</title><path d="{path}" fill="none" stroke="{color}" stroke-width="2.5"{dash} marker-end="url(#{marker})"/></g>')
        if label:txt(x,y,label,size=20,fill=color)
        manifest_edges.append({'path':path,'label':label,'basis':kind,'sourceIds':refs or []})
    def diamond(cx,cy,w,h,lines,alias):
        native(alias,' / '.join(lines))
        s.append(f'<path d="M{cx},{cy-h/2} L{cx+w/2},{cy} L{cx},{cy+h/2} L{cx-w/2},{cy} Z" fill="#fff" stroke="#8770b2" stroke-width="2"/>')
        txt(cx,cy-(len(lines)-1)*13+7,lines,size=20,gap=26);s.append('</g>')
    # Reading surface and boundaries.
    txt(40,51,context.presentation['label'] + ' · one stage iteration',size=33,anchor='start',weight=700)
    txt(40,87,context.presentation['summary'],size=21,anchor='start',fill=MUTED)
    _scope_strip(s, context, W)
    s.append('<g transform="translate(0 130)">')
    s.append('<rect x="400" y="117" width="830" height="1593" rx="18" fill="#fff" stroke="#8396a9" stroke-width="2"/>')
    txt(815,149,'xdesign system',size=23,weight=700)
    for x,w,label,lane in [(40,350,'Agent','agent'),(410,360,'StageCli','cli'),(780,440,'Controller','controller')]:
        fill,stroke=COL[lane]
        s.append(f'<rect x="{x}" y="172" width="{w}" height="1510" rx="10" fill="{fill}" fill-opacity="0.38"/>')
        s.append(f'<rect x="{x}" y="172" width="{w}" height="48" rx="10" fill="{fill}" stroke="{stroke}"/>')
        txt(x+w/2,204,label,size=24,weight=700)
    # All connectors are presentation expansions, each backed by source action links/outputs.
    edge('M365,277 L435,277',refs=[N['agentObserve']['@id'],N['next']['@id']],kind='action subsetting: observe :> cli.next')
    edge('M590,314 L590,346 L215,346 L215,370','work (if active)',400,336,kind='parameter binding: performWork.workOrder = observe.work',dashed=True)
    edge('M215,460 L215,510',refs=[context.succession(N['performWork']['@id'], N['submit']['@id'])],kind='native succession: performWork -> submit')
    edge('M365,555 L435,555',refs=[N['submit']['@id'],N['check']['@id']],kind='action subsetting: submit :> cli.check')
    edge('M590,600 L590,627',refs=[context.succession(N['checkObserve']['@id'], N['checkGuard']['@id'])],kind='native succession: observe -> if')
    edge('M590,737 L590,765','true',618,754,kind='native guard and ordered then-body: not observe.completed')
    edge('M480,682 L420,682 L420,1538 L435,1538',kind='empty conditional branch -> returnWork; control shortcut when complete')
    txt(451,719,'false',size=20,fill=MUTED)
    edge('M590,845 L590,890 L835,890','record',718,878,refs=[context.succession(N['collectOutputs']['@id'], N['record']['@id'])],kind='native succession + action subsetting: collectOutputs -> record :> controller.recordActivity')
    edge('M1000,929 L1000,975',refs=[context.succession(N['record']['@id'], N['assessAndAdvance']['@id'])],kind='native succession + action subsetting: record -> assessAndAdvance :> controller.progressWish')
    edge('M1000,1050 L1000,1080',refs=[context.succession(N['selectCurrentExecution']['@id'], N['evaluateCheckpoints']['@id'])],kind='native succession: selectCurrentExecution -> evaluateCheckpoints')
    edge('M1000,1155 L1000,1177',refs=[context.succession(N['evaluateCheckpoints']['@id'], N['authorization']['@id'])],kind='native succession: evaluateCheckpoints -> if')
    edge('M880,1232 L880,1325','true',848,1295,kind='guard then-body: completionAuthorized')
    edge('M1120,1232 L1120,1325','false',1151,1295,kind='guard else-body: not completionAuthorized')
    edge('M880,1430 L880,1470 L1000,1470 L1000,1500',refs=[context.succession(N['authorization']['@id'], N['updateTraceStatus']['@id'])],kind='native if join -> updateTraceStatus')
    edge('M1120,1430 L1120,1470 L1000,1470',refs=[context.succession(N['authorization']['@id'], N['updateTraceStatus']['@id'])],kind='native if join -> updateTraceStatus')
    edge('M835,1538 L745,1538',refs=[context.succession(N['checkGuard']['@id'], N['returnWork']['@id'])],kind='check conditional completes -> returnWork; expanded controller returns control')
    edge('M590,1575 L590,1640 L365,1640',kind='check output binding: work, completed, assessed visit assessments',dashed=True)
    edge('M65,1640 L20,1640 L20,415 L65,415',kind='documented intent only: repeat while work is returned; stop on completion; no native back-edge',orange=True)
    # Action surfaces.
    box(65,240,300,74,['xdesign next WISH_ID'],'agent','agentObserve',size=22)
    box(435,240,310,74,['next · read current work'],'cli','next',size=22)
    box(65,370,300,90,['performWork','Edit workspace artifacts'],'agent','performWork',size=22)
    box(65,510,300,90,['xdesign check WISH_ID','[--result FILE]'],'agent','submit',size=22)
    box(435,510,310,90,['observe → next','Read current visit again'],'cli','checkObserve',size=22)
    diamond(590,682,220,110,['not','completed?'],'checkGuard')
    box(435,765,310,80,['collectOutputs','Capture artifact evidence'],'cli','collectOutputs',size=22)
    box(835,850,330,79,['recordActivity','Append activity record'],'controller',['record','recordActivity'],size=22)
    # ProgressWish owns the controller's selection, evaluation and alternatives.
    s.append('<rect x="795" y="951" width="410" height="637" rx="15" fill="none" stroke="#8770b2" stroke-width="2" stroke-dasharray="5 5"/>')
    s.append('<rect x="850" y="937" width="300" height="28" fill="#f6f4fa"/>')
    txt(1000,959,'progressWish (internal)',size=20,fill='#715a9a',weight=700)
    box(835,975,330,75,['selectCurrentExecution'],'controller','selectCurrentExecution',size=22)
    box(835,1080,330,75,['evaluateCheckpoints'],'controller','evaluateCheckpoints',size=22)
    diamond(1000,1232,240,110,['completion','authorized?'],'authorization')
    box(805,1325,190,105,['Choose successor','Advance or','complete execution'],'controller',['decideNextStep','updateExecutionState'],size=20)
    box(1015,1325,190,105,['Retain current','visit as blocked'],'controller','retainCurrentStage',size=20)
    box(835,1500,330,75,['updateTraceStatus'],'controller','updateTraceStatus',size=22)
    box(435,1500,310,75,['returnWork → next'],'cli','returnWork',size=22)
    box(65,1595,300,90,['Current / successor work','or completion'],'agent','check',size=21)
    # Compact contract notes; these do not add actions or branches.
    txt(215,1030,['Repeat only when','work is returned.','Stop on completion.'],size=20,fill=ORANGE,gap=27)
    txt(590,397,['Read-only. Completed run:','return completion and stop.'],size=20,fill=MUTED,gap=26)
    txt(590,1090,['If already complete, check','skips mutation and still','returns through next.'],size=20,fill=MUTED,gap=27)
    txt(590,1215,['Authorization requires','subflow completion and','satisfied checkpoints.'],size=20,fill=MUTED,gap=27)
    txt(40,1744,'Orange dashed loop: documented repeat only; no native back-edge exists in AgentStageWork.',size=21,fill=ORANGE,anchor='start')
    txt(40,1780,'Projection: action order and responsibility; lanes are not declared SysML allocations.',size=20,fill=MUTED,anchor='start')
    txt(40,1811,'Guards come from model expressions. Internal controller calls are expanded for inspection.',size=20,fill=MUTED,anchor='start')
    txt(40,1842,'Design projection. Registration and final human acceptance are outside this stage interaction.',size=20,fill=MUTED,anchor='start')
    s.append('</g></svg>')
    context.relationships.extend(manifest_edges)
    return '\n'.join(s)
