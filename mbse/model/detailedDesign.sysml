package XDesignDetailedDesign {
    @XDesignCommon::DiagramPresentation { label = "Design"; }
    @XDesignCommon::DocSection { document = "docs/details/simple_flow/01_run_design.md"; section = "D01"; }
    @XDesignCommon::DocFigure { document = "docs/details/simple_flow/03_structure_graphs.md"; figure = "docs/details/simple_flow/assets/03_logical.svg"; }
    private import ScalarValues::*;
    private import ControlFunctions::exists;
    private import SequenceFunctions::size;
    private import XDesignCommon::*;
    private import XDesignArchitecture::*;
    private import XDesignWorkflow::*;
    private import StandardViewDefinitions::*;
    private import Views::*;

    doc /* The detailed design of the simple flow: what one run of
       XDesignArchitecture::SimpleFlow is made into — the class of a run, the factory
       that creates it, the functions that evaluate its gates and advance it, the
       rules that determine each check, and the designed flow that gates every stage
       with checkpoints of exact content.
       This file details that flow; it changes neither the flow definition nor the
       runs already recorded against it. */

    part designedSimpleFlow : SimpleFlow {
        @XDesignCommon::DiagramPresentation { label = "Gated simple flow"; summary = "Four stages with explicit exit gates"; }
        @XDesignCommon::DocSection { document = "docs/details/simple_flow/01_run_design.md"; section = "D01"; }
        doc /* The simple flow as designed: the same four stages in the same order, with
           every stage's exit gated. The requirements, design and implementation stages
           are gated by fixed checks; the verification stage is gated by a record check
           and by the flow's inherited semantic judgment. A wish selects this usage at
           registration when it wants the designed gates; SimpleFlow itself is unchanged,
           so the runs already recorded against workflow.'flow' stay valid. */

        part redefines checkpoints : SimpleFlowCheckpoint [5];

        item simpleRequirementsCheck : Checker subsets checkers {
            @XDesignCommon::DocSection { document = "docs/details/simple_flow/02_gates_and_rules.md"; section = "D06"; }
            attribute redefines checkerId = "simpleRequirementsCheck";
            attribute redefines commandUri = "tools/checks/simple_requirements.py";
            attribute redefines establishes = (ConformanceKind::identifierConformance, ConformanceKind::traceConformance);
            item requirementsCarryRationale : CheckerRule subsets rules {
                attribute redefines ruleId = "requirements-carry-rationale";
                attribute redefines condition = "Every requirement declaration inside XDesignRequirements carries a nonempty rationale attribute.";
                attribute redefines reads = "mbse/model/requirements.sysml";
            }
            item satisfyLinksResolve : CheckerRule subsets rules {
                attribute redefines ruleId = "satisfy-links-resolve";
                attribute redefines condition = "Every satisfy link names a requirement declared inside XDesignRequirements.";
                attribute redefines reads = ("mbse/model/requirements.sysml", "mbse/model/traceability.sysml");
            }
        }

        item simpleDesignCheck : Checker subsets checkers {
            @XDesignCommon::DocSection { document = "docs/details/simple_flow/02_gates_and_rules.md"; section = "D06"; }
            attribute redefines checkerId = "simpleDesignCheck";
            attribute redefines commandUri = "tools/checks/simple_design.py";
            attribute redefines establishes = (ConformanceKind::artifactPresence, ConformanceKind::definitionCoverage);
            item designDeclarationsPresent : CheckerRule subsets rules {
                attribute redefines ruleId = "design-declarations-present";
                attribute redefines condition = "The run, factory, gate, advance, designed flow, checkpoint class, both adaptations, and checkpoint evaluation action are declared.";
                attribute redefines reads = "mbse/model/detailedDesign.sysml";
            }
            item everyStageGated : CheckerRule subsets rules {
                attribute redefines ruleId = "every-stage-gated";
                attribute redefines condition = "Every stage names nonempty exit gates declared by this flow and typed by SimpleFlowCheckpoint.";
                attribute redefines reads = "mbse/model/detailedDesign.sysml";
            }
            item designElementsCarryDocSections : CheckerRule subsets rules {
                attribute redefines ruleId = "design-elements-carry-doc-sections";
                attribute redefines condition = "Every top-level detailed design element has a DocSection that resolves to a document and section.";
                attribute redefines reads = ("mbse/model/detailedDesign.sysml", "docs/");
            }
            item checkerCommandsExist : CheckerRule subsets rules {
                attribute redefines ruleId = "checker-commands-exist";
                attribute redefines condition = "Every declared SimpleFlow checker command URI names an existing repository file.";
                attribute redefines reads = ("mbse/model/detailedDesign.sysml", "tools/checks/");
            }
            item checkerRulesAreDeclared : CheckerRule subsets rules {
                attribute redefines ruleId = "checker-rules-are-declared";
                attribute redefines condition = "Every SimpleFlow checker declares at least one rule with a nonempty ID, condition and reads value.";
                attribute redefines reads = "mbse/model/detailedDesign.sysml";
            }
            item ruleSetsAgreeWithCommands : CheckerRule subsets rules {
                attribute redefines ruleId = "rule-sets-agree-with-commands";
                attribute redefines condition = "Each checker's declared rule IDs exactly equal its command's rule ID list.";
                attribute redefines reads = ("mbse/model/detailedDesign.sysml", "tools/checks/");
            }
            item structureGraphsAreCarried : CheckerRule subsets rules {
                attribute redefines ruleId = "structure-graphs-are-carried";
                attribute redefines condition = "Every required SimpleFlow model figure exists and is embedded in the document named by its DocFigure annotation.";
                attribute redefines reads = ("mbse/model/detailedDesign.sysml", "docs/");
            }
        }

        item simpleImplementationCheck : Checker subsets checkers {
            @XDesignCommon::DocSection { document = "docs/details/simple_flow/02_gates_and_rules.md"; section = "D06"; }
            attribute redefines checkerId = "simpleImplementationCheck";
            attribute redefines commandUri = "tools/checks/simple_implementation.py";
            attribute redefines establishes = ConformanceKind::artifactPresence;
            item visitExists : CheckerRule subsets rules {
                attribute redefines ruleId = "visit-exists";
                attribute redefines condition = "The checked implementation stage has a current visit in the run record.";
                attribute redefines reads = "runtime run record JSON";
            }
            item artifactsOfCurrentVisitExist : CheckerRule subsets rules {
                attribute redefines ruleId = "artifacts-of-current-visit-exist";
                attribute redefines condition = "The current implementation visit records at least one artifact URI and every recorded artifact exists at that URI.";
                attribute redefines reads = ("runtime run record JSON", "recorded artifact URIs");
            }
        }

        item simpleRecordCheck : Checker subsets checkers {
            @XDesignCommon::DocSection { document = "docs/details/simple_flow/02_gates_and_rules.md"; section = "D06"; }
            attribute redefines checkerId = "simpleRecordCheck";
            attribute redefines commandUri = "tools/checks/simple_record.py";
            attribute redefines establishes = (ConformanceKind::traceConformance, ConformanceKind::artifactPresence);
            item flowIsSupported : CheckerRule subsets rules {
                attribute redefines ruleId = "flow-is-supported";
                attribute redefines condition = "The run record has plan_id designedSimpleFlow and plan_sha256 equal to the SHA-256 of the retained frozen plan serialization.";
                attribute redefines reads = "runtime run record JSON";
            }
            item everyStageVisited : CheckerRule subsets rules {
                attribute redefines ruleId = "every-stage-visited";
                attribute redefines condition = "Every planned stage has at least one visit and every visit names a stage in that plan.";
                attribute redefines reads = "runtime run record JSON";
            }
            item completedVisitsAuthorized : CheckerRule subsets rules {
                attribute redefines ruleId = "completed-visits-authorized";
                attribute redefines condition = "For every completed visit, the latest assessment of each required exit gate passed at that visit's artifact revision.";
                attribute redefines reads = "runtime run record JSON";
            }
            item assessmentsNameExitGates : CheckerRule subsets rules {
                attribute redefines ruleId = "assessments-name-exit-gates";
                attribute redefines condition = "Every passed assessment names an exit gate of its visit's stage.";
                attribute redefines reads = "runtime run record JSON";
            }
            item passedAssessmentsCarryEvidence : CheckerRule subsets rules {
                attribute redefines ruleId = "passed-assessments-carry-evidence";
                attribute redefines condition = "Every passed assessment names at least one evidence URI.";
                attribute redefines reads = "runtime run record JSON";
            }
            item evidenceExists : CheckerRule subsets rules {
                attribute redefines ruleId = "evidence-exists";
                attribute redefines condition = "Every evidence URI of every passed assessment on current or completed visits resolves to retained evidence.";
                attribute redefines reads = ("runtime run record JSON", "recorded evidence URIs");
            }
            item designedRunsAreTypedByDesignClass : CheckerRule subsets rules {
                attribute redefines ruleId = "designed-runs-are-typed-by-the-design-class";
                attribute redefines condition = "A run on the designed SimpleFlow records its execution as SimpleFlowRun.";
                attribute redefines reads = "runtime run record JSON";
            }
        }

        part simpleRequirementsAccepted : SimpleFlowSyntacticCheckpoint subsets checkpoints {
            @XDesignCommon::DocSection { document = "docs/details/simple_flow/02_gates_and_rules.md"; section = "D05"; }
            doc /* Exit gate of the requirements stage. */
            ref part redefines sourceStage = requirements;
            attribute redefines kind = CheckKind::syntactic;
            attribute redefines criteria = "Every requirement declared in the project model carries its rationale, and every satisfy link resolves to a requirement that is declared.";
            ref item redefines checkers = simpleRequirementsCheck;
                    }

        part simpleDesignAccepted : SimpleFlowSyntacticCheckpoint subsets checkpoints {
            @XDesignCommon::DocSection { document = "docs/details/simple_flow/02_gates_and_rules.md"; section = "D05"; }
            doc /* Exit gate of the design stage. */
            ref part redefines sourceStage = design;
            attribute redefines kind = CheckKind::syntactic;
            attribute redefines criteria = "The design declares the run class, the factory, the functions that evaluate gates and advance the run, and the checkpoint class with its syntactic and semantic adaptations and its evaluate operation; every exit checkpoint of this flow is an instance of that class; every stage of this flow declares its exit checkpoint; every design element names its documentation section; and all nine SysML diagram views are declared, drawn and carried by their document.";
            ref item redefines checkers = simpleDesignCheck;
                    }

        part simpleImplementationAccepted : SimpleFlowSyntacticCheckpoint subsets checkpoints {
            @XDesignCommon::DocSection { document = "docs/details/simple_flow/02_gates_and_rules.md"; section = "D05"; }
            doc /* Exit gate of the implementation stage. */
            ref part redefines sourceStage = implement;
            attribute redefines kind = CheckKind::syntactic;
            attribute redefines criteria = "The stage's current visit records at least one artifact, and every recorded artifact exists at its URI.";
            ref item redefines checkers = simpleImplementationCheck;
                    }

        part simpleRecordAccepted : SimpleFlowSyntacticCheckpoint subsets checkpoints {
            @XDesignCommon::DocSection { document = "docs/details/simple_flow/02_gates_and_rules.md"; section = "D05"; }
            doc /* The record gate of the verification stage: the mechanically checkable
               half of verification, split from the judgment that stays semantic, so a
               script decides the record and a judge decides the meaning. */
            ref part redefines sourceStage = 'verify';
            attribute redefines kind = CheckKind::syntactic;
            attribute redefines criteria = "The run records at least one visit per stage; every completed visit has a latest passed assessment of each exit gate at its artifact revision; and evidence of every passed assessment on current or completed visits is retained.";
            ref item redefines checkers = simpleRecordCheck;
                    }

        part redefines requirements {
            ref part redefines exitCheckpoints = simpleRequirementsAccepted;
        }
        part redefines design {
            ref part redefines exitCheckpoints = simpleDesignAccepted;
        }
        part redefines implement {
            ref part redefines exitCheckpoints = simpleImplementationAccepted;
        }
        part redefines 'verify' {
            ref part redefines exitCheckpoints = (simpleRecordAccepted, verificationAccepted);
        }
        part redefines verificationAccepted : SimpleFlowSemanticCheckpoint {
            @XDesignCommon::DocSection { document = "docs/details/simple_flow/02_gates_and_rules.md"; section = "D05"; }
            doc /* The flow's inherited semantic gate, kept and stated exactly: what a
               script cannot establish — that the evidence demonstrates the
               requirements are satisfied — stays a judgment, by an agent or a human,
               with the record and the evidence as what they read. */
            attribute redefines criteria = (
                "Verification evidence demonstrates that the implementation satisfies the requirements.",
                "The record and the evidence stand on their own: a reader can re-examine the judgment from them without re-asking.");
        }
    }

    part def SimpleFlowCheckpoint :> Checkpoint {
        @XDesignCommon::DiagramPresentation { label = "Checkpoint"; summary = "Evaluates one gate for one stage visit"; }
        @XDesignCommon::DocSection { document = "docs/details/simple_flow/02_gates_and_rules.md"; section = "D11"; }
        doc /* One checkpoint of the designed simple flow, abstracted as one class for both
           kinds of rule. The class adapts the flow's syntax rules and its semantic rules:
           a syntactic checkpoint binds the checker whose command fixes the check and
           the rules that decide it, declared here and documented in D06;
           a semantic checkpoint binds the criteria and the judge authorities. The class
           owns the evaluate operation and the assessments that operation produces, so a
           gate is judged through the class and the record points back to it — the loop
           closes on the class. */

        ref item assessments : CheckpointAssessment [0..*];

        perform action evaluate : EvaluateSimpleFlowCheckpoint;
    }

    part def SimpleFlowSyntacticCheckpoint :> SimpleFlowCheckpoint {
        @XDesignCommon::DiagramPresentation { label = "Script check"; summary = "Runs fixed rules and retains the report"; }
        @XDesignCommon::DocSection { document = "docs/details/simple_flow/02_gates_and_rules.md"; section = "D11"; }
        doc /* The syntactic adaptation of the checkpoint class: a fixed command
           applies its declared checker rules. The declared command must exist and
           return a revision-bound report with the exact rule set before this gate
           can pass. Its evaluate operation retains the per-rule report as evidence. */

        attribute redefines kind = CheckKind::syntactic;

        perform action redefines evaluate : RunSyntacticCheck;
    }

    part def SimpleFlowSemanticCheckpoint :> SimpleFlowCheckpoint {
        @XDesignCommon::DiagramPresentation { label = "Semantic judgment"; summary = "A permitted judge records a reasoned verdict"; }
        @XDesignCommon::DocSection { document = "docs/details/simple_flow/02_gates_and_rules.md"; section = "D11"; }
        doc /* The semantic adaptation of the checkpoint class: criteria a permitted judge
           reads, with no checker. Its evaluate operation presents the criteria and the
           visit's artifacts and evidence to the judge and records the rationale. The logic
           a criterion states stays prose for the judge; the design fixes who may judge it
           and what the run records. */

        attribute redefines kind = CheckKind::semantic;

        perform action redefines evaluate : TakeSemanticJudgment;

        assert constraint criteriaAreJudged {
            size(checkers) == 0 and (agentMayJudge or humanMayJudge)
        }
    }

    action def EvaluateSimpleFlowCheckpoint {
        @XDesignCommon::DiagramPresentation { label = "Evaluate checkpoint"; summary = "Choose scripted checks or semantic judgment"; }
        @XDesignCommon::DocSection { document = "docs/details/simple_flow/02_gates_and_rules.md"; section = "D11"; }
        doc /* The evaluate operation of a checkpoint class, for one gate and one visit:
           read the class's rules, take the path its adaptation fixes, and produce one
           assessment. A syntactic checkpoint's rules are its checker's CheckerRules,
           applied by the command and reported rule by rule; a semantic checkpoint's rules
           are its criteria, read by a permitted judge. A rule that cannot be settled by a
           command is stated in the documentation and judged, never encoded beside the
           model. */

        in part checkpoint : SimpleFlowCheckpoint;
        in part visit : StageRecord;
        out item assessment : CheckpointAssessment [0..1];

        action readRules {
            @XDesignCommon::DiagramPresentation { label = "Read rules"; }
            doc /* The checkpoint's rules: the syntactic adaptation reads its checker's
               CheckerRules and their conditions; the semantic adaptation reads its criteria
               and the judges its flags permit. */
            out item rules : CheckerRule [0..*];
            out attribute criteria : String [0..*];
        }
        then decide rulePath {
            @XDesignCommon::DiagramPresentation { label = "Check kind"; }
        }
        action applySyntacticRules {
            @XDesignCommon::DiagramPresentation { label = "Run fixed check"; }
            doc /* Run the checker command through the check port, verdict by verdict, and
               append an assessment judged by the script; the report for the visited
               revision is the evidence. */
        }
        action takeSemanticJudgment {
            @XDesignCommon::DiagramPresentation { label = "Request judgment"; }
            doc /* Present the criteria and the visit's artifacts and evidence through the
               judgment port to a permitted judge and append the judgment with its
               rationale and evidence. */
        }
        action appendAssessment {
            @XDesignCommon::DiagramPresentation { label = "Keep assessment"; }
            doc /* Append the assessment to the visit; its checkpoint is this class, and the
               class keeps the assessment as a member of its own evaluation history. */
        }
        merge assessmentReady;

        first start then readRules;
        succession syntacticRulesApplied first rulePath
            if checkpoint.kind == CheckKind::syntactic then applySyntacticRules {
            @XDesignCommon::DiagramPresentation { label = "Syntactic"; summary = "kind = syntactic"; }
        }
        succession semanticJudgmentTaken first rulePath
            if checkpoint.kind == CheckKind::semantic then takeSemanticJudgment {
            @XDesignCommon::DiagramPresentation { label = "Semantic"; summary = "kind = semantic"; }
        }
        succession syntacticAssessed first applySyntacticRules then assessmentReady;
        succession semanticAssessed first takeSemanticJudgment then assessmentReady;
        succession assessmentRecorded first assessmentReady then appendAssessment;
    }

    action def RunSyntacticCheck :> EvaluateSimpleFlowCheckpoint {
        @XDesignCommon::DiagramPresentation { label = "Run script check"; summary = "Apply fixed checker rules"; }
        @XDesignCommon::DocSection { document = "docs/details/simple_flow/02_gates_and_rules.md"; section = "D11"; }
        doc /* The syntactic implementation of the checkpoint class's evaluate operation:
           the checker command runs unchanged, its declared rules are applied one by one,
           and the assessment holds only when every rule holds. */

        in part redefines checkpoint : SimpleFlowSyntacticCheckpoint;
    }

    action def TakeSemanticJudgment :> EvaluateSimpleFlowCheckpoint {
        @XDesignCommon::DiagramPresentation { label = "Take semantic judgment"; summary = "Review criteria with artifacts and evidence"; }
        @XDesignCommon::DocSection { document = "docs/details/simple_flow/02_gates_and_rules.md"; section = "D11"; }
        doc /* The semantic implementation of the checkpoint class's evaluate operation: a
           permitted judge reads the criteria with the visit's artifacts and evidence, and
           the outcome and rationale are recorded. */

        in part redefines checkpoint : SimpleFlowSemanticCheckpoint;
    }

    part def SimpleFlowRun :> FlowExecution {
        @XDesignCommon::DiagramPresentation { label = "Flow run"; summary = "Visits, gate assessments and advancement"; }
        @XDesignCommon::DocSection { document = "docs/details/simple_flow/01_run_design.md"; section = "D02"; }
        doc /* One run of the designed simple flow: the class a wish's execution has.
           It refines the generic FlowExecution with the designed flow's plan — a visit
           to each of its four stages, more visits when a stage is reworked — and
           carries operations to evaluate the current visit's gates, advance when
           authorized, and explicitly reopen a completed terminal visit. Its ports are the interfaces
           a run is automated through: a check command it invokes, and a judge it
           presents to. The factory creates it; the controller keeps it in the wish's
           trace. */

        ref part redefines assignedFlow = designedSimpleFlow;
        ref part checkpoints : SimpleFlowCheckpoint [0..*] = designedSimpleFlow.checkpoints;
        part redefines stageRecords : StageRecord [4..*];

        port check : ~CheckCommandExchange {
            @XDesignCommon::DiagramPresentation { label = "check"; summary = "Invoke command and receive report"; }
        }
        port judgment : ~JudgmentExchange {
            @XDesignCommon::DiagramPresentation { label = "review"; summary = "Present criteria and receive verdict"; }
        }

        event occurrence created;
        event occurrence completed;

        exhibit state lifecycle : SimpleFlowRunLifecycle;
        bind lifecycle.run = self;

        perform action evaluateGate : EvaluateSimpleFlowGate;
        perform action advance : AdvanceSimpleFlowRun;
        perform action reopenStage : ReopenSimpleFlowStage;

        assert constraint visitsRealizeThePlannedStages {
            stageRecords->forAll { in record : StageRecord;
                designedSimpleFlow.stages->includes(record.stage)
            } and
            designedSimpleFlow.stages->forAll { in stage : Stage;
                stageRecords->exists { in record : StageRecord;
                    record.stage == stage
                }
            }
        }
    }

    part def SimpleFlowRunFactory {
        @XDesignCommon::DiagramPresentation { label = "Run factory"; summary = "Creates a run and its initial visits"; }
        @XDesignCommon::DocSection { document = "docs/details/simple_flow/01_run_design.md"; section = "D03"; }
        doc /* The factory of a run: given a wish and the designed flow's plan, it
           creates the run the controller will hold. It realizes registration for this
           flow — the part of Controller::receiveWish that assigns the flow, opens the
           run and enters its initial stage — so an implementation of the simple flow
           does not have to re-invent it from the generic controller. */

        perform action createRun {
            @XDesignCommon::DocSection { document = "docs/details/simple_flow/01_run_design.md"; section = "D03"; }
            doc /* Open a run of the designed simple flow for a wish: create the run
               bound to the plan, add one pending visit per stage, enter the
               requirements visit as current, and activate the run. The wish itself
               stays an item of the workflow; the trace that holds the run is the
               controller's, produced by receiveWish. */

            in part wish : Wish;
            out part run : SimpleFlowRun;

            action openRun {
                doc /* Create the run, bound to the designed flow's plan and without
                   visits yet. */
                out part run : SimpleFlowRun;
            }
            action addStageVisits {
                doc /* Add one pending visit per stage of the plan — requirements,
                   design, implement, verify — each bound to its stage. */
                in part redefines run = openRun.run;
            }
            action enterInitialStage {
                doc /* The requirements visit becomes the current visit and is marked
                   active at artifact revision 0; the other visits stay pending at
                   revision 0. Recording a content-change activity increments that
                   visit's revision before a gate assesses it. */
                in part redefines run = openRun.run;
            }
            action activateRun {
                doc /* The run becomes active, so the controller records activities
                   against its current visit and evaluates its gates. */
                in part redefines run = openRun.run;
            }

            first start then openRun;
            succession visitsAdded first openRun then addStageVisits;
            succession stageEntered first addStageVisits then enterInitialStage;
            succession runActivated first enterInitialStage then activateRun;
        }
    }

    action def EvaluateSimpleFlowGate {
        @XDesignCommon::DiagramPresentation { label = "Evaluate stage gates"; summary = "Record an assessment for each exit gate"; }
        @XDesignCommon::DocSection { document = "docs/details/simple_flow/01_run_design.md"; section = "D04"; }
        doc /* Evaluate exactly the exit checkpoints of this visit's stage, once
           per checkpoint, in plan order. Each checkpoint's evaluate operation
           takes the path its adaptation fixes. A syntactic checkpoint performs
           RunSyntacticCheck — the declared command executes through the run's check
           port, applies the declared
           rules, and its per-rule report for the visited revision is the evidence; the
           assessment is recorded as the script's judgment, holding only when every rule
           holds. A semantic checkpoint performs TakeSemanticJudgment — its criteria and
           the visit's artifacts and evidence go through the judgment port to a permitted
           judge, whose rationale is recorded. The per-gate loop visits both
           checkpoint kinds at the verification stage. Each evaluation appends one
           CheckpointAssessment to the visit; a gate with no current assessment
           authorizes nothing. */

        in part visit : StageRecord;
        ref part checkpoints : SimpleFlowCheckpoint [0..*];
        out item assessments : CheckpointAssessment [0..*];

        action readGates {
            doc /* The checkpoint class instances of the visit's stage, read from the
               assigned flow's plan. Other stages' gates are excluded. */
            out part gates : SimpleFlowCheckpoint [0..*] = checkpoints->select {
                in gate : SimpleFlowCheckpoint;
                visit.stage.exitCheckpoints->includes(gate)
            };
        }
        then for gate in readGates.gates {
            action evaluateCurrentGate : EvaluateSimpleFlowCheckpoint {
                doc /* The gate's own specialization selects the scripted or judged
                   path; this action is performed once for each selected exit gate. */
                in part redefines checkpoint = gate;
                in part redefines visit = EvaluateSimpleFlowGate::visit;
            }
        }

        first start then readGates;
    }

    action def AdvanceSimpleFlowRun {
        @XDesignCommon::DiagramPresentation { label = "Advance flow run"; summary = "Authorization selects the next visit or completion"; }
        @XDesignCommon::DocSection { document = "docs/details/simple_flow/01_run_design.md"; section = "D04"; }
        doc /* The successor rule of the designed flow, for one run: when the current
           visit is authorized — every exit gate passed and cited at the visited
           revision — decide where the run goes. A stage with a successor hands the
           run to that stage's pending visit, which becomes current. The verify stage
           has none: the run completes and the flow status becomes completed, which
           is what lets the controller deliver the trace. An unauthorized visit
           remains current and blocked for correction at a later artifact revision.
           Reopening a completed terminal stage is a separate action. */

        in part run : SimpleFlowRun;
        out part nextVisit : StageRecord [0..1];

        action checkAuthorization {
            doc /* completionAuthorized at the visited revision: every exit gate of
               the current visit has a passed assessment cited at it, and any subflow
               has completed. */
        }
        then decide advancePath;
        action enterNextStage {
            doc /* The pending visit of the stage that follows the current one becomes
               the current visit; the run stays active. */
        }
        action retainBlockedVisit {
            doc /* Keep the unauthorized current visit selected and mark both the
               visit and the run blocked. Corrective work records a content-change
               activity in this visit and increments its artifact revision. */
            in part blockedVisit : StageRecord = run.currentRecord;
            assert constraint sameVisitIsBlocked {
                run.currentRecord == blockedVisit and
                blockedVisit.status == StageStatus::blocked and
                run.status == FlowStatus::blocked
            }
        }
        action completeRun {
            doc /* The current visit is the verify visit, whose stage completes the
               flow: the run completes and stays its current record. */
        }

        first start then checkAuthorization;
        succession nextStageChosen first advancePath
            if run.currentRecord.completionAuthorized and
            not run.currentRecord.stage.completesFlow then enterNextStage;
        succession runCompleted first advancePath
            if run.currentRecord.completionAuthorized and
            run.currentRecord.stage.completesFlow then completeRun;
        succession gateBlocked first advancePath
            if not run.currentRecord.completionAuthorized then retainBlockedVisit;
    }

    action def ReopenSimpleFlowStage {
        @XDesignCommon::DocSection { document = "docs/details/simple_flow/01_run_design.md"; section = "D04"; }
        doc /* Explicit rework of a completed terminal visit. A completed run can
           reopen its terminal verification stage only on a recorded rework request.
           Keep the completed visit and its assessments immutable, append a new
           active visit for that stage at the next visit index with artifact revision
           0, and return the run to active. Reopening an earlier completed stage
           requires downstream invalidation rules and is outside this action. */

        in part run : SimpleFlowRun;
        in part completedVisit : StageRecord = run.currentRecord;
        out part newVisit : StageRecord;

        action requireCompletedTerminal {
            doc /* Reject unless the run is completed and the current visit is a
               completed terminal stage. */
            assert constraint terminalReopenPrecondition {
                run.status == FlowStatus::completed and
                completedVisit.status == StageStatus::completed and
                completedVisit.stage.completesFlow
            }
        }
        then action appendNewVisit {
            doc /* Preserve the prior visit and add a new active visit of the same
               stage with visitIndex one greater and artifactRevision 0. */
            assert constraint freshVisit {
                newVisit.stage == completedVisit.stage and
                newVisit.visitIndex == completedVisit.visitIndex + 1 and
                newVisit.artifactRevision == 0 and
                newVisit.status == StageStatus::active
            }
        }
        then action reactivateRun {
            doc /* Make the new visit current and the run active. */
            assert constraint newVisitIsCurrent {
                run.currentRecord == newVisit and
                run.status == FlowStatus::active
            }
        }

        first start then requireCompletedTerminal;
    }

    port def CheckCommandExchange {
        @XDesignCommon::DiagramPresentation { label = "Check exchange"; summary = "Arguments, rule report and verdict"; }
        @XDesignCommon::DocSection { document = "docs/details/simple_flow/03_structure_graphs.md"; section = "D10"; }
        doc /* The check command's interface: the independent SysML wish trace and
           the executable run-record snapshot are both named. The checker identity,
           frozen plan ID and hash, stage, artifact revision and artifact-set digest
           select the exact fixed rule set and input. The report returns the assessed
           revision, artifact-set digest and rule IDs so the caller can reject a
           stale, incomplete, duplicate or contradictory report before recording
           an assessment. */

        in attribute wishModel : String;
        in attribute recordUri : String;
        in attribute checkerId : String;
        in attribute planId : String;
        in attribute planSha256 : String;
        in attribute stage : String;
        in attribute artifactRevision : Integer;
        in attribute snapshotDigest : String;
        out attribute report : String;
        out attribute reportedRevision : Integer;
        out attribute reportedSnapshotDigest : String;
        out attribute reportedRuleIds : String [1..*];
        out attribute holds : Boolean;
    }

    interface def CheckCommandInterface {
        @XDesignCommon::DiagramPresentation { label = "Check interface"; summary = "Invokes a fixed checker command"; }
        @XDesignCommon::DocSection { document = "docs/details/simple_flow/03_structure_graphs.md"; section = "D10"; }
        doc /* The caller-command interface a syntactic gate is run through: the run
           invokes the named checker with the wish trace, run record, frozen plan,
           stage, revision and artifact-set digest, then validates the returned
           rule report and verdict. */

        end port caller : ~CheckCommandExchange;
        end port command : CheckCommandExchange;

        flow invocation from caller.wishModel to command.wishModel;
        flow recordArgument from caller.recordUri to command.recordUri;
        flow checkerArgument from caller.checkerId to command.checkerId;
        flow planArgument from caller.planId to command.planId;
        flow planHashArgument from caller.planSha256 to command.planSha256;
        flow stageArgument from caller.stage to command.stage;
        flow revisionArgument from caller.artifactRevision to command.artifactRevision;
        flow snapshotArgument from caller.snapshotDigest to command.snapshotDigest;
        flow ruleReport from command.report to caller.report;
        flow reportRevision from command.reportedRevision to caller.reportedRevision;
        flow reportSnapshot from command.reportedSnapshotDigest to caller.reportedSnapshotDigest;
        flow ruleIdentifiers from command.reportedRuleIds to caller.reportedRuleIds;
        flow verdict from command.holds to caller.holds;
    }

    port def JudgmentExchange {
        @XDesignCommon::DiagramPresentation { label = "Judgment exchange"; summary = "Criteria, artifacts, outcome and rationale"; }
        @XDesignCommon::DocSection { document = "docs/details/simple_flow/03_structure_graphs.md"; section = "D10"; }
        doc /* The judgment interface: the criteria and the artifacts go out to a
           permitted judge, the outcome and its rationale come back. What a script
           cannot settle crosses this interface. */

        out attribute criteria : String [1..*];
        out attribute artifactsUris : String [0..*];
        in attribute outcome : CheckpointOutcome;
        in attribute rationale : String;
    }

    interface def JudgmentInterface {
        @XDesignCommon::DiagramPresentation { label = "Judgment interface"; summary = "Requests a permitted semantic review"; }
        @XDesignCommon::DocSection { document = "docs/details/simple_flow/03_structure_graphs.md"; section = "D10"; }
        doc /* The run-judge interface a semantic gate is judged through. */

        end port judge : JudgmentExchange;
        end port requester : ~JudgmentExchange;

        flow criteriaPresented from requester.criteria to judge.criteria;
        flow artifactsPresented from requester.artifactsUris to judge.artifactsUris;
        flow outcomeReturned from judge.outcome to requester.outcome;
        flow rationaleReturned from judge.rationale to requester.rationale;
    }

    part def CheckerCommand {
        @XDesignCommon::DiagramPresentation { label = "Checker command"; summary = "Fixed script execution boundary"; }
        @XDesignCommon::DocSection { document = "docs/details/simple_flow/03_structure_graphs.md"; section = "D10"; }
        doc /* A checker's command as the design's part: the command implementation
           supplies later, reached through the check command interface. No command exists
           in the project yet; no implementation language has been selected. */
        port command : CheckCommandExchange {
            @XDesignCommon::DiagramPresentation { label = "command"; summary = "Arguments in; report and verdict out"; }
        }
    }

    part def Judge {
        @XDesignCommon::DiagramPresentation { label = "Judge"; summary = "Permitted agent or human"; }
        @XDesignCommon::DocSection { document = "docs/details/simple_flow/03_structure_graphs.md"; section = "D10"; }
        doc /* A permitted judge as the design's part: an agent or a human, reached
           through the judgment interface. */
        port judgment : JudgmentExchange {
            @XDesignCommon::DiagramPresentation { label = "review"; summary = "Criteria in; outcome and rationale out"; }
        }
    }

    part def SimpleFlowAutomation {
        @XDesignCommon::DiagramPresentation { label = "Simple flow automation"; summary = "Run, checker and judge connected by typed ports"; }
        @XDesignCommon::DocSection { document = "docs/details/simple_flow/03_structure_graphs.md"; section = "D10"; }
        doc /* The parts a run of the designed flow is automated by, wired by their
           interfaces: the factory creates the run, the run invokes a checker command
           through the check command interface, and presents its semantic gates to a
           judge through the judgment interface. */

        part runner : SimpleFlowRun {
            @XDesignCommon::DiagramPresentation { label = "Flow run"; summary = "Automates the current stage visit"; }
        }
        part factory : SimpleFlowRunFactory;
        ref part checkpoints : SimpleFlowCheckpoint [0..*] = runner.checkpoints;
        part command : CheckerCommand {
            @XDesignCommon::DiagramPresentation { label = "Checker"; summary = "Runs the fixed command"; }
        }
        part judge : Judge {
            @XDesignCommon::DiagramPresentation { label = "Judge"; summary = "Agent or human reviews evidence"; }
        }

        interface checking : CheckCommandInterface
            connect runner.check to command.command;
        interface judging : JudgmentInterface
            connect runner.judgment to judge.judgment;
    }

    occurrence def SimpleFlowRunTimeline {
        @XDesignCommon::DiagramPresentation { label = "Run interactions"; summary = "Controller events enter and assess the four stages"; }
        @XDesignCommon::DocSection { document = "docs/details/simple_flow/03_structure_graphs.md"; section = "D08"; }
        @XDesignCommon::DocFigure { document = "docs/details/simple_flow/03_structure_graphs.md"; figure = "docs/details/simple_flow/assets/03_timeline.svg"; }
        doc /* One run of the designed flow over time, whatever wish it answers: the
           run is created, each stage is entered and its gate assessed in order, and
           the terminal stage's completion finishes the run. The messages are the
           moments the flow defines; a particular wish's journey is the same shape
           with its own records. */

        ref part controller : Controller = workflow.controller;
        ref part run : SimpleFlowRun;
        ref part requirements : Stage = designedSimpleFlow.requirements;
        ref part design : Stage = designedSimpleFlow.design;
        ref part implement : Stage = designedSimpleFlow.implement;
        ref part 'verify' : Stage = designedSimpleFlow.'verify';

        message createRun from controller.entering to run.created;
        message enterRequirements from controller.entering to requirements.entered;
        message assessRequirements from controller.assessing to requirements.assessed;
        message enterDesign from controller.entering to design.entered;
        message assessDesign from controller.assessing to design.assessed;
        message enterImplementation from controller.entering to implement.entered;
        message assessImplementation from controller.assessing to implement.assessed;
        message enterVerification from controller.entering to 'verify'.entered;
        message assessVerification from controller.assessing to 'verify'.assessed;
        message completeRun from controller.delivering to run.completed;
    }

    view designedFlowTree : StandardViewDefinitions::BrowserView {
        @XDesignCommon::DocSection { document = "docs/details/simple_flow/01_run_design.md"; section = "D01"; }
        @XDesignCommon::DocFigure { document = "docs/details/simple_flow/01_run_design.md"; figure = "docs/details/simple_flow/assets/01_designed_flow.svg"; }
        doc /* What the designed flow is made of: its stages, the exit checkpoints it
           redefines onto them, and the checkers the syntactic gates name with the
           rules each checker declares. */

        expose XDesignDetailedDesign::designedSimpleFlow;
    }

    view designTree : StandardViewDefinitions::BrowserView {
        @XDesignCommon::DocSection { document = "docs/details/simple_flow/01_run_design.md"; section = "D02"; }
        @XDesignCommon::DocFigure { document = "docs/details/simple_flow/01_run_design.md"; figure = "docs/details/simple_flow/assets/01_design_classes.svg"; }
        doc /* The design's classes, factory and functions with their members: the
           run's visits, ports and operations, the factory's creation steps, and the
           gate function's paths. */

        expose XDesignDetailedDesign::SimpleFlowRun;
        expose XDesignDetailedDesign::SimpleFlowRunFactory;
        expose XDesignDetailedDesign::EvaluateSimpleFlowGate;
        expose XDesignDetailedDesign::AdvanceSimpleFlowRun;
        expose XDesignDetailedDesign::SimpleFlowCheckpoint;
        expose XDesignDetailedDesign::SimpleFlowSyntacticCheckpoint;
        expose XDesignDetailedDesign::SimpleFlowSemanticCheckpoint;
        expose XDesignDetailedDesign::EvaluateSimpleFlowCheckpoint;
    }

    view factoryFlow : StandardViewDefinitions::ActionFlowView {
        @XDesignCommon::DocSection { document = "docs/details/simple_flow/01_run_design.md"; section = "D03"; }
        @XDesignCommon::DocFigure { document = "docs/details/simple_flow/01_run_design.md"; figure = "docs/details/simple_flow/assets/01_factory.svg"; }
        doc /* The factory's creation steps in order: open the run, add the visits,
           enter the initial stage, activate. */

        expose XDesignDetailedDesign::SimpleFlowRunFactory::createRun;
        expose XDesignDetailedDesign::SimpleFlowRunFactory::createRun::*;

        render Views::asInterconnectionDiagram;
    }

    view gateEvaluationFlow : StandardViewDefinitions::ActionFlowView {
        @XDesignCommon::DocSection { document = "docs/details/simple_flow/01_run_design.md"; section = "D04"; }
        @XDesignCommon::DocFigure { document = "docs/details/simple_flow/01_run_design.md"; figure = "docs/details/simple_flow/assets/01_gate_evaluation.svg"; }
        doc /* The gate function's shape: select only the visit stage's exit gates,
           iterate over each gate, and delegate to its kind-specific evaluation
           path before recording one assessment per gate. */

        expose XDesignDetailedDesign::EvaluateSimpleFlowGate;
        expose XDesignDetailedDesign::EvaluateSimpleFlowGate::*;

        render Views::asInterconnectionDiagram;
    }

    view checkpointEvaluationFlow : StandardViewDefinitions::ActionFlowView {
        @XDesignCommon::DocSection { document = "docs/details/simple_flow/02_gates_and_rules.md"; section = "D11"; }
        @XDesignCommon::DocFigure { document = "docs/details/simple_flow/02_gates_and_rules.md"; figure = "docs/details/simple_flow/assets/02_checkpoint_evaluation.svg"; }
        doc /* The checkpoint class's evaluate operation: read the class's rules, take the
           syntactic or the semantic path its adaptation fixes, and append the assessment. */

        expose XDesignDetailedDesign::EvaluateSimpleFlowCheckpoint;
        expose XDesignDetailedDesign::EvaluateSimpleFlowCheckpoint::*;

        render Views::asInterconnectionDiagram;
    }

    item def SimpleFlowFileEntry {
        @XDesignCommon::DocSection { document = "docs/details/simple_flow/07_file_plan_and_verification.md"; section = "D53"; }
        doc /* One repository file in the SimpleFlow implementation plan. Its path
           names source, tests, or a checked input, never generated cache output. */
        attribute repositoryPath : String;
        attribute responsibility : String;
    }

    item simpleFlowFilePlan {
        @XDesignCommon::DiagramPresentation { label = "SimpleFlow file plan"; summary = "Source, checker and test files"; }
        @XDesignCommon::DocSection { document = "docs/details/simple_flow/07_file_plan_and_verification.md"; section = "D53"; }
        doc /* Planned repository layout for the first executable SimpleFlow slice.
           Rust modules follow the SimpleFlowRustUml module design; four Python
           commands implement the D06 syntactic checker rule sets. The test files
           exercise the same contracts independently. Generated reports and tool
           caches belong under cache/ and are not source entries here. */

        item application {
            item manifest : SimpleFlowFileEntry { attribute redefines repositoryPath = "Cargo.toml"; attribute redefines responsibility = "Rust crate manifest"; }
            item crateLibrary : SimpleFlowFileEntry { attribute redefines repositoryPath = "src/lib.rs"; attribute redefines responsibility = "Public crate modules and API"; }
            item commandLine : SimpleFlowFileEntry { attribute redefines repositoryPath = "src/main.rs"; attribute redefines responsibility = "Local run and check entry point"; }
            item status : SimpleFlowFileEntry { attribute redefines repositoryPath = "src/status.rs"; attribute redefines responsibility = "Flow and gate status vocabulary"; }
            item record : SimpleFlowFileEntry { attribute redefines repositoryPath = "src/record.rs"; attribute redefines responsibility = "Visits, activities and assessments"; }
            item plan : SimpleFlowFileEntry { attribute redefines repositoryPath = "src/plan.rs"; attribute redefines responsibility = "Stage, checkpoint and checker plan validation"; }
            item ports : SimpleFlowFileEntry { attribute redefines repositoryPath = "src/ports.rs"; attribute redefines responsibility = "Checker and judge boundaries"; }
            item gate : SimpleFlowFileEntry { attribute redefines repositoryPath = "src/gate.rs"; attribute redefines responsibility = "Per-checkpoint evaluation and report validation"; }
            item run : SimpleFlowFileEntry { attribute redefines repositoryPath = "src/run.rs"; attribute redefines responsibility = "Run state, authorization, advance and reopen"; }
            item factory : SimpleFlowFileEntry { attribute redefines repositoryPath = "src/factory.rs"; attribute redefines responsibility = "Create a validated run"; }
        }
        item checkCommands {
            item sharedContract : SimpleFlowFileEntry { attribute redefines repositoryPath = "tools/checks/common.py"; attribute redefines responsibility = "Shared report schema and input validation"; }
            item requirements : SimpleFlowFileEntry { attribute redefines repositoryPath = "tools/checks/simple_requirements.py"; attribute redefines responsibility = "Requirements gate rules"; }
            item design : SimpleFlowFileEntry { attribute redefines repositoryPath = "tools/checks/simple_design.py"; attribute redefines responsibility = "Design gate rules"; }
            item implementation : SimpleFlowFileEntry { attribute redefines repositoryPath = "tools/checks/simple_implementation.py"; attribute redefines responsibility = "Implementation gate rules"; }
            item record : SimpleFlowFileEntry { attribute redefines repositoryPath = "tools/checks/simple_record.py"; attribute redefines responsibility = "Verification record gate rules"; }
        }
        item testCases {
            item runCases : SimpleFlowFileEntry { attribute redefines repositoryPath = "tests/simple_flow.rs"; attribute redefines responsibility = "Plan, authorization, recovery and reopen cases"; }
            item checkerCases : SimpleFlowFileEntry { attribute redefines repositoryPath = "tests/test_simple_flow_checkers.py"; attribute redefines responsibility = "Rule and report contract cases"; }
        }
    }

    view simpleFlowFilePlanView : StandardViewDefinitions::BrowserView {
        @XDesignCommon::DocSection { document = "docs/details/simple_flow/07_file_plan_and_verification.md"; section = "D53"; }
        @XDesignCommon::DocFigure { document = "docs/details/simple_flow/07_file_plan_and_verification.md"; figure = "docs/details/simple_flow/assets/07_file_plan.svg"; }
        doc /* Which repository files realize the Rust run, fixed checker commands,
           and executable verification, with generated evidence kept in cache/. */
        expose XDesignDetailedDesign::simpleFlowFilePlan;
    }

    view interfaceStructure : StandardViewDefinitions::InterconnectionView {
        @XDesignCommon::DocSection { document = "docs/details/simple_flow/03_structure_graphs.md"; section = "D10"; }
        @XDesignCommon::DocFigure { document = "docs/details/simple_flow/03_structure_graphs.md"; figure = "docs/details/simple_flow/assets/03_interfaces.svg"; }
        doc /* The interfaces a run is automated through: the run's check port wired
           to a checker command by the check command interface, and its judgment port
           wired to a judge by the judgment interface. */

        expose XDesignDetailedDesign::SimpleFlowAutomation;
        expose XDesignDetailedDesign::SimpleFlowAutomation::*;
        expose XDesignDetailedDesign::CheckerCommand;
        expose XDesignDetailedDesign::Judge;
        expose XDesignDetailedDesign::CheckCommandInterface;
        expose XDesignDetailedDesign::JudgmentInterface;

        render Views::asInterconnectionDiagram;
    }

    constraint def GateAuthorization {
        @XDesignCommon::DiagramPresentation { label = "Gate authorization"; summary = "authorized = (passed = required) and subflow complete"; }
        @XDesignCommon::DocSection { document = "docs/details/simple_flow/05_module_design.md"; section = "D16"; }
        doc /* Count distinct required gates with a passed assessment at the visited
           revision. Authorization also requires the nested flow, if any, to finish.
           This equation is a design constraint, not an executed compiler check. */
        in attribute requiredGates : Integer;
        in attribute passedGates : Integer;
        in attribute subFlowComplete : Boolean;
        in attribute authorized : Boolean;
        authorized == (passedGates == requiredGates and subFlowComplete)
    }

    part def SimpleFlowAuthorization {
        @XDesignCommon::DiagramPresentation { label = "Visit authorization"; summary = "Bind current visit values to the authorization constraint"; }
        @XDesignCommon::DocSection { document = "docs/details/simple_flow/05_module_design.md"; section = "D16"; }
        doc /* Parametric context for a real stage visit. Counting gates rather than
           assessments prevents duplicate passes from compensating for a missing gate.
           A stale or failed assessment contributes nothing. */
        ref part visit : StageRecord;
        attribute requiredGates : Integer = size(visit.stage.exitCheckpoints) {
            @XDesignCommon::DiagramPresentation { label = "Required gates"; summary = "Count of distinct exit checkpoints"; }
        }
        attribute passedGates : Integer = size(visit.stage.exitCheckpoints->select {
            in gate : Checkpoint;
            visit.currentAssessments->exists { in assessment : CheckpointAssessment;
                assessment.checkpoint == gate and
                assessment.outcome == CheckpointOutcome::passed and
                assessment.evaluatedRevision == visit.artifactRevision
            }
        }) {
            @XDesignCommon::DiagramPresentation { label = "Passed gates"; summary = "Current revision; distinct required gates"; }
        }
        attribute subFlowComplete : Boolean = visit.subFlowSatisfied {
            @XDesignCommon::DiagramPresentation { label = "Subflow complete"; summary = "Any nested flow has completed"; }
        }
        attribute authorized : Boolean = visit.completionAuthorized {
            @XDesignCommon::DiagramPresentation { label = "Advance allowed"; summary = "Current visit completion is authorized"; }
        }
        constraint authorization : GateAuthorization {
            @XDesignCommon::DiagramPresentation { label = "Gate authorization"; summary = "authorized = (passed = required) and subflow complete"; }
        }
        bind requiredGates = authorization.requiredGates;
        bind passedGates = authorization.passedGates;
        bind subFlowComplete = authorization.subFlowComplete;
        bind authorized = authorization.authorized;
    }

    state def SimpleFlowRunLifecycle {
        @XDesignCommon::DiagramPresentation { label = "Run lifecycle"; summary = "Activation, blocking, correction and completion"; }
        @XDesignCommon::DocSection { document = "docs/details/simple_flow/05_module_design.md"; section = "D20"; }
        doc /* Lifecycle of one run, distinct from the four engineering stages.
           The factory activates the run. A failed, missing or stale gate blocks it;
           corrective work resumes the same current visit at a new artifact revision.
           Only authorization of a terminal visit permits completion. A completed
           terminal visit is reopened by an explicit rework action that appends a new
           visit without erasing history. */
        ref part run : SimpleFlowRun;
        state pending {
            @XDesignCommon::DiagramPresentation { label = "Pending"; }
        }
        state active {
            @XDesignCommon::DiagramPresentation { label = "Active"; }
        }
        state blocked {
            @XDesignCommon::DiagramPresentation { label = "Blocked"; }
        }
        state completed {
            @XDesignCommon::DiagramPresentation { label = "Completed"; }
        }
        first start then pending;
        transition activate first pending
            if run.status == FlowStatus::active then active;
        @XDesignCommon::DiagramPresentation about activate { label = "Activate"; summary = "run is active"; }
        transition gateBlocked first active
            if run.status == FlowStatus::blocked and
            not run.currentRecord.completionAuthorized then blocked;
        @XDesignCommon::DiagramPresentation about gateBlocked { label = "Block"; summary = "gate not authorized"; }
        transition resumeAfterCorrection first blocked
            if run.currentRecord.status == StageStatus::active then active;
        @XDesignCommon::DiagramPresentation about resumeAfterCorrection { label = "Resume"; summary = "corrected visit active"; }
        transition advanceStage first active
            if run.currentRecord.completionAuthorized and
            not run.currentRecord.stage.completesFlow then active;
        @XDesignCommon::DiagramPresentation about advanceStage { label = "Next stage"; summary = "nonterminal visit authorized"; }
        transition complete first active
            if run.currentRecord.completionAuthorized and
            run.currentRecord.stage.completesFlow then completed;
        @XDesignCommon::DiagramPresentation about complete { label = "Complete"; summary = "terminal visit authorized"; }
        transition reopenForRework first completed
            if run.status == FlowStatus::active and
            run.currentRecord.status == StageStatus::active then active;
        @XDesignCommon::DiagramPresentation about reopenForRework { label = "Rework"; summary = "run and new visit active"; }
    }

    use case def ProgressWish {
        @XDesignCommon::DiagramPresentation { label = "Progress wish"; summary = "Reach an evidence-backed delivery"; }
        @XDesignCommon::DocSection { document = "docs/requirements/01_baseline.md"; section = "D17"; }
        doc /* The requester's goal: progress a wish to an evidence-backed delivery,
           inspect its record and request corrective work when evidence is insufficient.
           Gate evaluation is internal behavior, not another external user goal. */
        subject automation : SimpleFlowAutomation;
        actor requester : WishRequester;
    }

    use case def ReviewEvidence {
        @XDesignCommon::DiagramPresentation { label = "Review evidence"; summary = "Return a reasoned semantic verdict"; }
        @XDesignCommon::DocSection { document = "docs/requirements/01_baseline.md"; section = "D17"; }
        doc /* A permitted agent or human judge reviews semantic criteria, artifacts
           and evidence, and returns a reasoned verdict through JudgmentInterface.
           The checker command remains an internal part of the automation. */
        subject automation : SimpleFlowAutomation;
        actor reviewer : Judge;
    }

    use case progressWish : ProgressWish {
        @XDesignCommon::DiagramPresentation { label = "Progress wish"; summary = "Reach an evidence-backed delivery"; }
        @XDesignCommon::DocSection { document = "docs/requirements/01_baseline.md"; section = "D17"; }
    }
    use case reviewEvidence : ReviewEvidence {
        @XDesignCommon::DiagramPresentation { label = "Review evidence"; summary = "Return a reasoned semantic verdict"; }
        @XDesignCommon::DocSection { document = "docs/requirements/01_baseline.md"; section = "D17"; }
    }

    part def DiagramDocumentationRenderer {
        @XDesignCommon::DocSection { document = "docs/details/04_diagram_presentation.md"; section = "D22"; }
        @XDesignCommon::DiagramPresentation { label = "Diagram renderer"; summary = "Focused model views become readable documentation figures"; }
        doc /* The documentation rendering utility, separate from runtime flow
           automation. It consumes native view selections and compiler exports,
           composes each diagram using its own notation, lays out nodes and edges
           through Graphviz, and writes SVGs with native source references. Short
           labels are presentation metadata; the source model retains full text.
           Figure generation does not establish engineering acceptance by itself. */

        perform action buildFigures {
            @XDesignCommon::DocSection { document = "docs/details/04_diagram_presentation.md"; section = "D23"; }
            in attribute sourceUris : String [1..*];
            in attribute selectedViewNames : String [1..*];
            out attribute figureUris : String [1..*];
            out attribute manifestUri : String;

            action exportModel {
                @XDesignCommon::DocSection { document = "docs/details/04_diagram_presentation.md"; section = "D23"; }
                doc /* Export native IDs, names, documentation, view selections,
                   metadata and relationships through the pinned compiler. Preserve
                   its diagnostics and disclose omitted export semantics. */
            }
            then action projectView {
                @XDesignCommon::DocSection { document = "docs/details/04_diagram_presentation.md"; section = "D23"; }
                doc /* Resolve the native view's selected elements and endpoints.
                   Apply short presentation labels and diagram-specific notation,
                   retaining source identities and full model details. Do not
                   fabricate relationships to improve composition. */
            }
            then action layoutGraph {
                @XDesignCommon::DocSection { document = "docs/details/04_diagram_presentation.md"; section = "D23"; }
                doc /* Use Graphviz rank placement, ports, clusters and routed
                   edges for focused graph views. Sequence diagrams use ordered
                   event-anchored lifelines. Preserve the meaning of each native
                   relationship, multiplicity, binding and control guard. */
            }
            then action renderFigure {
                @XDesignCommon::DocSection { document = "docs/details/04_diagram_presentation.md"; section = "D23"; }
                doc /* Write the final SVG with readable labels, source identity
                   references and full-text tooltips. Generate a raster preview
                   from that SVG, not from a previous layout or source revision. */
            }
            then action validateFinalFigure {
                @XDesignCommon::DocSection { document = "docs/details/04_diagram_presentation.md"; section = "D24"; }
                doc /* Check selected source identities, endpoints, displayed
                   font size and figure/document links. Record final SVG and
                   preview hashes and generation parameters in the manifest.
                   Visual review judges the final nine figures at normal article
                   size and records any residual limitation separately from
                   compiler and automated regression results. */
            }

            first start then exportModel;
            first validateFinalFigure then done;
        }
    }

    viewpoint def DocumentationViewpoint {
        @XDesignCommon::DocSection { document = "docs/details/05_documentation_contract.md"; section = "D29"; }
        doc /* Records a stakeholder concern for one declared documentation view.
           A concrete viewpoint usage names its subject view; that view selects
           native model elements with expose and locates its rendered figure with
           DocFigure. The concern text must agree with the view's presentation
           question, which is reviewed rather than evaluated by the compiler. */
        attribute concernText : String;
        attribute stakeholderRole : String;
    }

    view blockDefinitionDiagram : StandardViewDefinitions::GeneralView {
        @XDesignCommon::DiagramPresentation { label = "Run structure"; question = "Which reusable types make up a run?"; }
        @XDesignCommon::DocSection { document = "docs/details/simple_flow/05_module_design.md"; section = "D13"; }
        @XDesignCommon::DocFigure { document = "docs/details/simple_flow/05_module_design.md"; figure = "docs/details/simple_flow/assets/05_block_definition.svg"; }
        doc /* Which reusable types form a run, and how do they specialize and compose? */
        expose XDesignDetailedDesign::SimpleFlowRun;
        expose XDesignArchitecture::FlowExecution;
        expose XDesignDetailedDesign::SimpleFlowRunFactory;
        expose XDesignDetailedDesign::SimpleFlowCheckpoint;
        expose XDesignDetailedDesign::SimpleFlowSyntacticCheckpoint;
        expose XDesignDetailedDesign::SimpleFlowSemanticCheckpoint;
        expose XDesignArchitecture::StageRecord;
        expose XDesignArchitecture::CheckpointAssessment;
    }

    view internalBlockDiagram : StandardViewDefinitions::InterconnectionView {
        @XDesignCommon::DiagramPresentation { label = "Automation connections"; question = "How do the run, checker and judge connect?"; }
        @XDesignCommon::DocSection { document = "docs/details/simple_flow/05_module_design.md"; section = "D14"; }
        @XDesignCommon::DocFigure { document = "docs/details/simple_flow/05_module_design.md"; figure = "docs/details/simple_flow/assets/05_internal_block.svg"; }
        doc /* How do this automation's run, checker and judge connect through ports? */
        expose XDesignDetailedDesign::SimpleFlowAutomation;
        render Views::asInterconnectionDiagram;
    }

    view requirementsDiagram : StandardViewDefinitions::GeneralView {
        @XDesignCommon::DiagramPresentation { label = "Requirements coverage"; question = "Which requirements have satisfiers and planned verification?"; }
        @XDesignCommon::DocSection { document = "docs/requirements/01_baseline.md"; section = "D15"; }
        @XDesignCommon::DocFigure { document = "docs/requirements/01_baseline.md"; figure = "docs/requirements/assets/01_requirements.svg"; }
        doc /* Which requirements have design satisfiers and a planned review case?
           A verify relationship states coverage; it does not assert a passed test. */
        expose XDesignRequirements::flowRunIsRealizedByDesign;
        expose XDesignRequirements::designedFlowGatesEveryStage;
        expose XDesignRequirements::structureGraphsAreDocumented;
        expose XDesignDetailedDesign::SimpleFlowRun;
        expose XDesignDetailedDesign::designedSimpleFlow;
        expose XDesignCommon::DocFigure;
        expose XDesignVerification::SimpleFlowViewReview;
    }

    view parametricDiagram : StandardViewDefinitions::GeneralView {
        @XDesignCommon::DiagramPresentation { label = "Visit authorization"; question = "Which current visit values authorize advancement?"; }
        @XDesignCommon::DocSection { document = "docs/details/simple_flow/05_module_design.md"; section = "D16"; }
        @XDesignCommon::DocFigure { document = "docs/details/simple_flow/05_module_design.md"; figure = "docs/details/simple_flow/assets/05_parametric.svg"; }
        doc /* Which bound record values determine authorization at this revision? */
        expose XDesignDetailedDesign::SimpleFlowAuthorization;
        expose XDesignDetailedDesign::SimpleFlowAuthorization::*;
        expose XDesignDetailedDesign::GateAuthorization;
    }

    view useCaseDiagram : StandardViewDefinitions::GeneralView {
        @XDesignCommon::DiagramPresentation { label = "Actor goals"; question = "Which goals belong to each external actor?"; }
        @XDesignCommon::DocSection { document = "docs/requirements/01_baseline.md"; section = "D17"; }
        @XDesignCommon::DocFigure { document = "docs/requirements/01_baseline.md"; figure = "docs/requirements/assets/01_use_cases.svg"; }
        doc /* Which external actors have which goals at the automation boundary? */
        expose XDesignDetailedDesign::ProgressWish;
        expose XDesignDetailedDesign::ReviewEvidence;
        expose XDesignDetailedDesign::progressWish;
        expose XDesignDetailedDesign::reviewEvidence;
        expose XDesignDetailedDesign::SimpleFlowAutomation;
        expose XDesignWorkflow::WishRequester;
        expose XDesignDetailedDesign::Judge;
    }

    view activityDiagram : StandardViewDefinitions::ActionFlowView {
        @XDesignCommon::DiagramPresentation { label = "Checkpoint evaluation"; question = "How does one gate produce an assessment?"; }
        @XDesignCommon::DocSection { document = "docs/details/simple_flow/05_module_design.md"; section = "D18"; }
        @XDesignCommon::DocFigure { document = "docs/details/simple_flow/05_module_design.md"; figure = "docs/details/simple_flow/assets/05_activity.svg"; }
        doc /* How is one checkpoint evaluated, by a script or a permitted judge? */
        expose XDesignDetailedDesign::EvaluateSimpleFlowCheckpoint;
        expose XDesignDetailedDesign::EvaluateSimpleFlowCheckpoint::*;
        render Views::asInterconnectionDiagram;
    }

    view sequenceDiagram : StandardViewDefinitions::SequenceView {
        @XDesignCommon::DiagramPresentation { label = "Run interactions"; question = "In which order are the four stages entered and assessed?"; }
        @XDesignCommon::DocSection { document = "docs/details/simple_flow/05_module_design.md"; section = "D19"; }
        @XDesignCommon::DocFigure { document = "docs/details/simple_flow/05_module_design.md"; figure = "docs/details/simple_flow/assets/05_sequence.svg"; }
        doc /* In which order do the controller, run and stages exchange events? */
        expose XDesignDetailedDesign::SimpleFlowRunTimeline;
    }

    view stateMachineDiagram : StandardViewDefinitions::StateTransitionView {
        @XDesignCommon::DiagramPresentation { label = "Run lifecycle"; question = "When does a run activate, block, resume or complete?"; }
        @XDesignCommon::DocSection { document = "docs/details/simple_flow/05_module_design.md"; section = "D20"; }
        @XDesignCommon::DocFigure { document = "docs/details/simple_flow/05_module_design.md"; figure = "docs/details/simple_flow/assets/05_state_machine.svg"; }
        doc /* Which conditions activate, block, resume and complete a run? */
        expose XDesignDetailedDesign::SimpleFlowRunLifecycle;
        expose XDesignDetailedDesign::SimpleFlowRunLifecycle::*;
        render Views::asInterconnectionDiagram;
    }

    view packageDiagram : StandardViewDefinitions::GeneralView {
        @XDesignCommon::DiagramPresentation { label = "Model packages"; question = "How do the project model concerns depend on one another?"; }
        @XDesignCommon::DocSection { document = "docs/arch/02_design.md"; section = "D21"; }
        @XDesignCommon::DocFigure { document = "docs/arch/02_design.md"; figure = "docs/arch/assets/02_simple_flow_packages.svg"; }
        doc /* How are model concerns separated and which packages depend on which?
           Wish trace models are consumers outside this project-model boundary. */
        expose XDesignCommon;
        expose XDesignRequirements;
        expose XDesignArchitecture;
        expose XDesignWorkflow;
        expose XDesignDetailedDesign;
        expose XDesignVerification;
        expose XDesignTraceability;
    }

    viewpoint blockDefinitionConcern : DocumentationViewpoint {
        @XDesignCommon::DocSection { document = "docs/details/simple_flow/05_module_design.md"; section = "D13"; }
        subject blockDefinitionDiagram;
        attribute redefines concernText = "Which reusable types make up a run?";
        attribute redefines stakeholderRole = "flow designer";
    }

    viewpoint internalBlockConcern : DocumentationViewpoint {
        @XDesignCommon::DocSection { document = "docs/details/simple_flow/05_module_design.md"; section = "D14"; }
        subject internalBlockDiagram;
        attribute redefines concernText = "How do the run, checker and judge connect?";
        attribute redefines stakeholderRole = "integrator";
    }

    viewpoint requirementsConcern : DocumentationViewpoint {
        @XDesignCommon::DocSection { document = "docs/requirements/01_baseline.md"; section = "D15"; }
        subject requirementsDiagram;
        attribute redefines concernText = "Which requirements have satisfiers and planned verification?";
        attribute redefines stakeholderRole = "requirements reviewer";
    }

    viewpoint parametricConcern : DocumentationViewpoint {
        @XDesignCommon::DocSection { document = "docs/details/simple_flow/05_module_design.md"; section = "D16"; }
        subject parametricDiagram;
        attribute redefines concernText = "Which current visit values authorize advancement?";
        attribute redefines stakeholderRole = "verification engineer";
    }

    viewpoint useCaseConcern : DocumentationViewpoint {
        @XDesignCommon::DocSection { document = "docs/requirements/01_baseline.md"; section = "D17"; }
        subject useCaseDiagram;
        attribute redefines concernText = "Which goals belong to each external actor?";
        attribute redefines stakeholderRole = "requester";
    }

    viewpoint activityConcern : DocumentationViewpoint {
        @XDesignCommon::DocSection { document = "docs/details/simple_flow/05_module_design.md"; section = "D18"; }
        subject activityDiagram;
        attribute redefines concernText = "How does one gate produce an assessment?";
        attribute redefines stakeholderRole = "implementer";
    }

    viewpoint sequenceConcern : DocumentationViewpoint {
        @XDesignCommon::DocSection { document = "docs/details/simple_flow/05_module_design.md"; section = "D19"; }
        subject sequenceDiagram;
        attribute redefines concernText = "In which order are the four stages entered and assessed?";
        attribute redefines stakeholderRole = "integrator";
    }

    viewpoint stateMachineConcern : DocumentationViewpoint {
        @XDesignCommon::DocSection { document = "docs/details/simple_flow/05_module_design.md"; section = "D20"; }
        subject stateMachineDiagram;
        attribute redefines concernText = "When does a run activate, block, resume or complete?";
        attribute redefines stakeholderRole = "maintainer";
    }

    viewpoint packageConcern : DocumentationViewpoint {
        @XDesignCommon::DocSection { document = "docs/arch/02_design.md"; section = "D21"; }
        subject packageDiagram;
        attribute redefines concernText = "How do the project model concerns depend on one another?";
        attribute redefines stakeholderRole = "model maintainer";
    }
}
