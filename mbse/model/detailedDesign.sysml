package XDesignDetailedDesign {
    @XDesignCommon::DiagramPresentation { label = "Design"; }
    @XDesignCommon::DocSection { document = "docs/design/flows/simple-flow/definition.md"; section = "D01"; }
    @XDesignCommon::DocFigure { document = "docs/design/flows/simple-flow/views.md"; figure = "docs/design/flows/simple-flow/assets/logical.svg"; }
    private import ScalarValues::*;
    private import ControlFunctions::exists;
    private import SequenceFunctions::size;
    private import XDesignCommon::*;
    private import XDesignArchitecture::*;
    private import XDesignWorkflow::*;
    private import StandardViewDefinitions::*;
    private import Views::*;

    doc /* The detailed design of the simple flow: the gated flow specialization,
       run class, factory, gate and advance functions, and exact checker rules.
       XDesignComposition owns the concrete selectable usage; this file defines
       its design without changing historical four-stage runs. */

    item def FrozenCheckBinding {
        @XDesignCommon::DocSection { document = "docs/design/checkpoint-library.md"; section = "D107"; }
        doc /* As-built projection across the frozen plan and work definition.
           libraryId is optional origin text, not a versioned package identity.
           Rules, criteria, permissions, runner and selected input paths are copied
           at init; new stored runs additionally retain the declared executor bundle.
           Library input selection replaces checker selectors only for the
           requirements and design stages in the current implementation.
           cli/model fields remain for historical-plan decoding only. */
        attribute checkpointId : String;
        attribute libraryId : String [0..1];
        attribute executorMode : String;
        attribute commandUri : String [0..1];
        attribute cli : String [0..1];
        attribute model : String [0..1];
        ref item rules : CheckerRule [0..*];
        attribute criteria : String [0..*];
        attribute agentMayJudge : Boolean;
        attribute humanMayJudge : Boolean;
        attribute selectedInputPaths : String [0..*];
    }

    part def CheckpointLibraryService {
        @XDesignCommon::DocSection { document = "docs/design/checkpoint-library.md"; section = "D107"; }
        doc /* Load/validate source, save definitions with ten previous partial
           snapshots, optionally scaffold a script, and resolve init bindings.
           Saving source alone never evaluates a gate. */
        ref part source : CheckpointLibrarySource;
    }

    action def ReadDocumentContract {
        @XDesignCommon::DocSection { document = "docs/design/document-kit.md"; section = "D113"; }
        in attribute profileRoot : String;
        out attribute contractDigest : String;
    }

    action def PlanContractDocument {
        @XDesignCommon::DocSection { document = "docs/design/document-kit.md"; section = "D113"; }
        in attribute documentClass : String;
        in attribute name : String;
        out attribute documentPath : String;
        out attribute requiredSections : String;
    }

    action def CreateContractDocument {
        @XDesignCommon::DocSection { document = "docs/design/document-kit.md"; section = "D113"; }
        in attribute documentClass : String;
        in attribute name : String;
        in attribute callerContent : String;
        out attribute documentSha256 : String;
    }

    action def UpdateContractSection {
        @XDesignCommon::DocSection { document = "docs/design/document-kit.md"; section = "D113"; }
        in attribute documentPath : String;
        in attribute sectionKey : String;
        in attribute callerContent : String;
        in attribute expectedSha256 : String;
        out attribute documentSha256 : String;
    }

    action def AttachContractFigure {
        @XDesignCommon::DocSection { document = "docs/design/document-kit.md"; section = "D113"; }
        in attribute documentPath : String;
        in attribute sectionKey : String;
        in attribute canonicalSvg : String;
        in attribute expectedSha256 : String;
        out attribute documentSha256 : String;
    }

    action def CheckContractDocuments {
        @XDesignCommon::DocSection { document = "docs/design/document-kit.md"; section = "D113"; }
        in attribute projectRoot : String;
        out attribute diagnosticReport : String;
    }

    part def DocumentContractEngine :> DocumentAuthoringBoundary {
        @XDesignCommon::DocSection { document = "docs/design/document-kit.md"; section = "D113"; }
        doc /* Resolve a document class and slug through a selected profile,
           render its versioned template from caller-supplied section content,
           update anchored sections and attach a model figure, then validate
           the authored document set. The CLI and MCP adapter invoke the same
           operations. Structural diagnostics do not assert design truth or
           visual readability. */
        attribute profileVersion : String;
        attribute projectRoot : String;
        attribute contractDigest : String;
        attribute supportedDocumentClasses : String [1..*];
        ref part contract : DocumentContractSource;
        perform action readContract : ReadDocumentContract;
        perform action planDocument : PlanContractDocument;
        perform action createDocument : CreateContractDocument;
        perform action updateSection : UpdateContractSection;
        perform action attachFigure : AttachContractFigure;
        perform action checkDocuments : CheckContractDocuments;
    }

    part def DocumentCliAdapter :> DocumentCliBoundary {
        @XDesignCommon::DocSection { document = "docs/design/document-kit.md"; section = "D113"; }
        doc /* Deterministic JSON command line over DocumentContractEngine. */
        ref part engine : DocumentContractEngine;
    }

    part def DocumentMcpAdapter :> DocumentMcpBoundary {
        @XDesignCommon::DocSection { document = "docs/design/document-kit.md"; section = "D113"; }
        doc /* Stdio MCP tool/resource adapter over DocumentContractEngine. */
        ref part engine : DocumentContractEngine;
    }

    package ProposedProjectChecks {
        @XDesignCommon::DocSection { document = "docs/design/checkpoint-library.md"; section = "D108"; }
        doc /* Discussion proposal only. No runtime, serialized contract, project
           identity, package import, federation or store relocation is implemented
           by these declarations. */
        item def DefinitionOrigin {
            @XDesignCommon::DocSection { document = "docs/design/checkpoint-library.md"; section = "D108"; }
            attribute namespace : String;
            attribute checkpointId : String;
            attribute upstreamRevision : String;
            attribute definitionSha256 : String;
            attribute executorBundleSha256 : String;
        }
        item def ExecutionReference {
            @XDesignCommon::DocSection { document = "docs/design/checkpoint-library.md"; section = "D108"; }
            attribute projectId : String;
            attribute storeId : String;
            attribute itemId : String;
            attribute storeRevision : Integer;
            attribute runId : String;
            attribute stageId : String;
            attribute visitIndex : Integer;
            attribute artifactRevision : Integer;
            attribute assessmentIndex : Integer;
        }
    }

    part def DesignedSimpleFlow :> SimpleFlow {
        @XDesignCommon::DiagramPresentation { label = "Gated simple flow"; summary = "Four stages with explicit exit gates"; }
        @XDesignCommon::DocSection { document = "docs/design/flows/simple-flow/definition.md"; section = "D01"; }
        doc /* The simple flow specialization: the same four stages in the same
           order, with every stage's exit gated. The first three stages use fixed
           checks; verification uses a record check and inherited semantic judgment.
           xdesign.designedSimpleFlow is the selectable usage of this definition. */

        part redefines checkpoints : SimpleFlowCheckpoint [5];

        item simpleRequirementsCheck : Checker subsets checkers {
            @XDesignCommon::DocSection { document = "docs/design/flows/simple-flow/definition.md"; section = "D06"; }
            attribute redefines checkerId = "simpleRequirementsCheck";
            attribute redefines commandUri = "tools/checks/simple_requirements.py";
            attribute redefines establishes = (ConformanceKind::identifierConformance, ConformanceKind::traceConformance);
            item requirementsCarryRationale : CheckerRule subsets rules {
                attribute redefines ruleId = "requirements-carry-rationale";
                attribute redefines condition = "XDesignRequirements declares at least one requirement, and every requirement declaration inside it carries a nonempty rationale attribute.";
                attribute redefines reads = "mbse/model/requirements.sysml";
            }
            item satisfyLinksResolve : CheckerRule subsets rules {
                attribute redefines ruleId = "satisfy-links-resolve";
                attribute redefines condition = "The declared traceability source has at least one satisfy link, and every satisfy link names a requirement declared inside XDesignRequirements.";
                attribute redefines reads = ("mbse/model/requirements.sysml", "mbse/model/traceability.sysml");
            }
        }

        item simpleDesignCheck : Checker subsets checkers {
            @XDesignCommon::DocSection { document = "docs/design/flows/simple-flow/definition.md"; section = "D06"; }
            attribute redefines checkerId = "simpleDesignCheck";
            attribute redefines commandUri = "tools/checks/simple_design.py";
            attribute redefines establishes = (ConformanceKind::artifactPresence, ConformanceKind::definitionCoverage);
            item designDeclarationsPresent : CheckerRule subsets rules {
                attribute redefines ruleId = "design-declarations-present";
                attribute redefines condition = "The run, factory, gate, advance, designed flow definition and xdesign-owned usage, checkpoint class, both adaptations, and checkpoint evaluation action are declared.";
                attribute redefines reads = ("mbse/model/detailedDesign.sysml", "mbse/model/composition.sysml");
            }
            item everyStageGated : CheckerRule subsets rules {
                attribute redefines ruleId = "every-stage-gated";
                attribute redefines condition = "Every stage names nonempty exit gates declared by this flow and typed by SimpleFlowCheckpoint; each syntactic exit binds its designated one of the four fixed SimpleFlow checkers.";
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
                attribute redefines condition = "All four fixed SimpleFlow checkers are declared, each with at least one rule with a nonempty ID, condition and reads value.";
                attribute redefines reads = "mbse/model/detailedDesign.sysml";
            }
            item ruleSetsAgreeWithCommands : CheckerRule subsets rules {
                attribute redefines ruleId = "rule-sets-agree-with-commands";
                attribute redefines condition = "Each of the four fixed checkers' declared rule IDs exactly equal its command's rule ID list.";
                attribute redefines reads = ("mbse/model/detailedDesign.sysml", "tools/checks/");
            }
            item structureGraphsAreCarried : CheckerRule subsets rules {
                attribute redefines ruleId = "structure-graphs-are-carried";
                attribute redefines condition = "Each of the nine primary SimpleFlow views names a distinct model figure, and every required figure exists and is embedded in the document named by its DocFigure annotation.";
                attribute redefines reads = ("mbse/model/detailedDesign.sysml", "docs/");
            }
        }

        item simpleImplementationCheck : Checker subsets checkers {
            @XDesignCommon::DocSection { document = "docs/design/flows/simple-flow/definition.md"; section = "D06"; }
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
            @XDesignCommon::DocSection { document = "docs/design/flows/simple-flow/definition.md"; section = "D06"; }
            attribute redefines checkerId = "simpleRecordCheck";
            attribute redefines commandUri = "tools/checks/simple_record.py";
            attribute redefines establishes = (ConformanceKind::traceConformance, ConformanceKind::artifactPresence);
            item flowIsSupported : CheckerRule subsets rules {
                attribute redefines ruleId = "flow-is-supported";
                attribute redefines condition = "The run record has plan_id designedSimpleFlow and plan_sha256 equal to the SHA-256 of the retained frozen plan serialization.";
                attribute redefines reads = ("runtime run record JSON", "retained plan serialization");
            }
            item everyStageVisited : CheckerRule subsets rules {
                attribute redefines ruleId = "every-stage-visited";
                attribute redefines condition = "Every planned stage has at least one visit and every visit names a stage in that plan.";
                attribute redefines reads = "runtime run record JSON";
            }
            item completedVisitsAuthorized : CheckerRule subsets rules {
                attribute redefines ruleId = "completed-visits-authorized";
                attribute redefines condition = "For every completed visit, the latest assessment of each required exit gate passed at that visit's artifact revision and snapshot digest, and every recorded artifact still matches its file digest.";
                attribute redefines reads = ("runtime run record JSON", "recorded artifact URIs");
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
                attribute redefines condition = "A run on the designed SimpleFlow records its execution as a FlowRun of plan designedSimpleFlow.";
                attribute redefines reads = "runtime run record JSON";
            }
        }

        part simpleRequirementsAccepted : SimpleFlowSyntacticCheckpoint subsets checkpoints {
            @XDesignCommon::DocSection { document = "docs/design/flows/simple-flow/definition.md"; section = "D05"; }
            doc /* Exit gate of the requirements stage. */
            ref part redefines sourceStage = requirements;
            attribute redefines kind = CheckKind::syntactic;
            attribute redefines criteria = "Every requirement declared in the project model carries its rationale, and every satisfy link resolves to a requirement that is declared.";
            ref item redefines checkers = simpleRequirementsCheck;
                    }

        part simpleDesignAccepted : SimpleFlowSyntacticCheckpoint subsets checkpoints {
            @XDesignCommon::DocSection { document = "docs/design/flows/simple-flow/definition.md"; section = "D05"; }
            doc /* Exit gate of the design stage. */
            ref part redefines sourceStage = design;
            attribute redefines kind = CheckKind::syntactic;
            attribute redefines criteria = "The design declares the run class, the factory, the functions that evaluate gates and advance the run, and the checkpoint class with its syntactic and semantic adaptations and its evaluate operation; every exit checkpoint of this flow is an instance of that class; every stage of this flow declares its exit checkpoint; every design element names its documentation section; and all nine SysML diagram views are declared, drawn and carried by their document.";
            ref item redefines checkers = simpleDesignCheck;
                    }

        part simpleImplementationAccepted : SimpleFlowSyntacticCheckpoint subsets checkpoints {
            @XDesignCommon::DocSection { document = "docs/design/flows/simple-flow/definition.md"; section = "D05"; }
            doc /* Exit gate of the implementation stage. */
            ref part redefines sourceStage = implement;
            attribute redefines kind = CheckKind::syntactic;
            attribute redefines criteria = "The stage's current visit records at least one artifact, and every recorded artifact exists at its URI.";
            ref item redefines checkers = simpleImplementationCheck;
                    }

        part simpleRecordAccepted : SimpleFlowSyntacticCheckpoint subsets checkpoints {
            @XDesignCommon::DocSection { document = "docs/design/flows/simple-flow/definition.md"; section = "D05"; }
            doc /* The record gate of the verification stage: the mechanically checkable
               half of verification, split from the judgment that stays semantic, so a
               script decides the record and a judge decides the meaning. */
            ref part redefines sourceStage = 'verify';
            attribute redefines kind = CheckKind::syntactic;
            attribute redefines criteria = "The run records at least one visit per stage; every completed visit has a latest passed assessment of each exit gate at its artifact revision; and evidence of every passed assessment on current or completed visits is retained.";
            ref item redefines checkers = simpleRecordCheck;
                    }

        part redefines requirements {
            attribute redefines goal = "Define requirements and their rationale.";
            attribute redefines workInstructions = "State the requirements, rationale and satisfaction relationships in the project model.";
            attribute redefines inputArtifacts = "wish:model";
            attribute redefines expectedOutputs = "checker:simpleRequirementsCheck";
            ref part redefines exitCheckpoints = simpleRequirementsAccepted;
        }
        part redefines design {
            attribute redefines goal = "Design the solution and its verification.";
            attribute redefines workInstructions = "Update the SysML design, API contracts and documentation; keep model views and checker declarations consistent.";
            attribute redefines inputArtifacts = "wish:model";
            attribute redefines expectedOutputs = "checker:simpleDesignCheck";
            ref part redefines exitCheckpoints = simpleDesignAccepted;
        }
        part redefines implement {
            attribute redefines goal = "Implement the reviewed design.";
            attribute redefines workInstructions = "Implement the source and tests required by the design; retain source artifacts for checking.";
            attribute redefines inputArtifacts = "wish:model";
            attribute redefines expectedOutputs = "source:src";
            ref part redefines exitCheckpoints = simpleImplementationAccepted;
        }
        part redefines 'verify' {
            attribute redefines goal = "Demonstrate that the implementation satisfies the requirements.";
            attribute redefines workInstructions = "Run the planned tests, retain verification evidence and supply a permitted semantic judgment.";
            attribute redefines inputArtifacts = "wish:model";
            attribute redefines expectedOutputs = "result:artifacts";
            ref part redefines exitCheckpoints = (simpleRecordAccepted, verificationAccepted);
        }
        part redefines verificationAccepted : SimpleFlowSemanticCheckpoint {
            @XDesignCommon::DocSection { document = "docs/design/flows/simple-flow/definition.md"; section = "D05"; }
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
        @XDesignCommon::DocSection { document = "docs/design/flows/simple-flow/definition.md"; section = "D11"; }
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
        @XDesignCommon::DocSection { document = "docs/design/flows/simple-flow/definition.md"; section = "D11"; }
        doc /* The syntactic adaptation of the checkpoint class: a fixed command
           applies its declared checker rules. The declared command must exist and
           return a revision-bound report with the exact rule set before this gate
           can pass. Its evaluate operation retains the per-rule report as evidence. */

        attribute redefines kind = CheckKind::syntactic;

        perform action redefines evaluate : RunSyntacticCheck;
    }

    part def SimpleFlowSemanticCheckpoint :> SimpleFlowCheckpoint {
        @XDesignCommon::DiagramPresentation { label = "Semantic judgment"; summary = "A permitted judge records a reasoned verdict"; }
        @XDesignCommon::DocSection { document = "docs/design/flows/simple-flow/definition.md"; section = "D11"; }
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
        @XDesignCommon::DocSection { document = "docs/design/flows/simple-flow/definition.md"; section = "D11"; }
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
        @XDesignCommon::DocSection { document = "docs/design/flows/simple-flow/definition.md"; section = "D11"; }
        doc /* The syntactic implementation of the checkpoint class's evaluate operation:
           the checker command runs unchanged, its declared rules are applied one by one,
           and the assessment holds only when every rule holds. */

        in part redefines checkpoint : SimpleFlowSyntacticCheckpoint;
    }

    action def TakeSemanticJudgment :> EvaluateSimpleFlowCheckpoint {
        @XDesignCommon::DiagramPresentation { label = "Take semantic judgment"; summary = "Review criteria with artifacts and evidence"; }
        @XDesignCommon::DocSection { document = "docs/design/flows/simple-flow/definition.md"; section = "D11"; }
        doc /* The semantic implementation of the checkpoint class's evaluate operation: a
           permitted judge reads the criteria with the visit's artifacts and evidence, and
           the outcome and rationale are recorded. */

        in part redefines checkpoint : SimpleFlowSemanticCheckpoint;
    }

    part def SimpleFlowRun :> FlowExecution {
        @XDesignCommon::DiagramPresentation { label = "Flow run"; summary = "Visits, gate assessments and advancement"; }
        @XDesignCommon::DocSection { document = "docs/design/flows/simple-flow/implementation.md"; section = "D02"; }
        doc /* One run of the designed simple flow: the class a wish's execution has.
           It refines the generic FlowExecution with the designed flow's plan — a visit
           to each of its four stages, more visits when a stage is reworked — and
           carries operations to evaluate the current visit's gates, advance when
           authorized, and explicitly reopen a completed terminal visit. Its ports are the interfaces
           a run is automated through: a check command it invokes, and a judge it
           presents to. The factory creates it; the controller keeps it in the wish's
           trace. */

        ref part redefines assignedFlow : DesignedSimpleFlow;
        ref part checkpoints : SimpleFlowCheckpoint [0..*] = assignedFlow.checkpoints;
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
                assignedFlow.stages->includes(record.stage)
            } and
            assignedFlow.stages->forAll { in stage : Stage;
                stageRecords->exists { in record : StageRecord;
                    record.stage == stage
                }
            }
        }
    }

    part def SimpleFlowRunFactory {
        @XDesignCommon::DiagramPresentation { label = "Run factory"; summary = "Creates a run and its initial visits"; }
        @XDesignCommon::DocSection { document = "docs/design/flows/simple-flow/implementation.md"; section = "D03"; }
        doc /* The factory of a run: given a wish and the designed flow's plan, it
           creates the run the controller will hold. It realizes registration for this
           flow — the part of Controller::receiveWish that assigns the flow, opens the
           run and enters its initial stage — so an implementation of the simple flow
           does not have to re-invent it from the generic controller. */

        perform action createRun {
            @XDesignCommon::DocSection { document = "docs/design/flows/simple-flow/implementation.md"; section = "D03"; }
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
        @XDesignCommon::DocSection { document = "docs/design/flows/simple-flow/implementation.md"; section = "D04"; }
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
        @XDesignCommon::DocSection { document = "docs/design/flows/simple-flow/implementation.md"; section = "D04"; }
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
        @XDesignCommon::DocSection { document = "docs/design/flows/simple-flow/implementation.md"; section = "D04"; }
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
        @XDesignCommon::DocSection { document = "docs/design/flows/simple-flow/views.md"; section = "D10"; }
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
        @XDesignCommon::DocSection { document = "docs/design/flows/simple-flow/views.md"; section = "D10"; }
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
        @XDesignCommon::DocSection { document = "docs/design/flows/simple-flow/views.md"; section = "D10"; }
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
        @XDesignCommon::DocSection { document = "docs/design/flows/simple-flow/views.md"; section = "D10"; }
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
        @XDesignCommon::DocSection { document = "docs/design/flows/simple-flow/views.md"; section = "D10"; }
        doc /* A checker's command as the design's part, reached through the check
           command interface. The four designed SimpleFlow commands are implemented
           as Python scripts under tools/checks/. The default V-model commands remain
           separate work. */
        port command : CheckCommandExchange {
            @XDesignCommon::DiagramPresentation { label = "command"; summary = "Arguments in; report and verdict out"; }
        }
    }

    part def Judge {
        @XDesignCommon::DiagramPresentation { label = "Judge"; summary = "Permitted agent or human"; }
        @XDesignCommon::DocSection { document = "docs/design/flows/simple-flow/views.md"; section = "D10"; }
        doc /* A permitted judge as the design's part: an agent or a human, reached
           through the judgment interface. */
        port judgment : JudgmentExchange {
            @XDesignCommon::DiagramPresentation { label = "review"; summary = "Criteria in; outcome and rationale out"; }
        }
    }

    part def SimpleFlowAutomation {
        @XDesignCommon::DiagramPresentation { label = "Simple flow automation"; summary = "Run, checker and judge connected by typed ports"; }
        @XDesignCommon::DocSection { document = "docs/design/flows/simple-flow/views.md"; section = "D10"; }
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
        @XDesignCommon::DocSection { document = "docs/design/flows/simple-flow/views.md"; section = "D08"; }
        @XDesignCommon::DocFigure { document = "docs/design/flows/simple-flow/views.md"; figure = "docs/design/flows/simple-flow/assets/timeline.svg"; }
        doc /* One run of the designed flow over time, whatever wish it answers: the
           run is created, each stage is entered and its gate assessed in order, and
           the terminal stage's completion finishes the run. The messages are the
           moments the flow defines; a particular wish's journey is the same shape
           with its own records. */

        ref part controller : Controller;
        ref part run : SimpleFlowRun;
        ref part 'flow' : DesignedSimpleFlow = run.assignedFlow;
        ref part requirements : Stage = 'flow'.requirements;
        ref part design : Stage = 'flow'.design;
        ref part implement : Stage = 'flow'.implement;
        ref part 'verify' : Stage = 'flow'.'verify';

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
        @XDesignCommon::DocSection { document = "docs/design/flows/simple-flow/definition.md"; section = "D01"; }
        @XDesignCommon::DocFigure { document = "docs/design/flows/simple-flow/definition.md"; figure = "docs/design/flows/simple-flow/assets/designed-flow.svg"; }
        doc /* What the designed flow is made of: its stages, the exit checkpoints it
           redefines onto them, and the checkers the syntactic gates name with the
           rules each checker declares. */

        expose XDesignDetailedDesign::DesignedSimpleFlow;
    }

    view designTree : StandardViewDefinitions::BrowserView {
        @XDesignCommon::DocSection { document = "docs/design/flows/simple-flow/implementation.md"; section = "D02"; }
        @XDesignCommon::DocFigure { document = "docs/design/flows/simple-flow/implementation.md"; figure = "docs/design/flows/simple-flow/assets/design-classes.svg"; }
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
        @XDesignCommon::DocSection { document = "docs/design/flows/simple-flow/implementation.md"; section = "D03"; }
        @XDesignCommon::DocFigure { document = "docs/design/flows/simple-flow/implementation.md"; figure = "docs/design/flows/simple-flow/assets/factory.svg"; }
        doc /* The factory's creation steps in order: open the run, add the visits,
           enter the initial stage, activate. */

        expose XDesignDetailedDesign::SimpleFlowRunFactory::createRun;
        expose XDesignDetailedDesign::SimpleFlowRunFactory::createRun::*;

        render Views::asInterconnectionDiagram;
    }

    view gateEvaluationFlow : StandardViewDefinitions::ActionFlowView {
        @XDesignCommon::DocSection { document = "docs/design/flows/simple-flow/implementation.md"; section = "D04"; }
        @XDesignCommon::DocFigure { document = "docs/design/flows/simple-flow/implementation.md"; figure = "docs/design/flows/simple-flow/assets/gate-evaluation.svg"; }
        doc /* The gate function's shape: select only the visit stage's exit gates,
           iterate over each gate, and delegate to its kind-specific evaluation
           path before recording one assessment per gate. */

        expose XDesignDetailedDesign::EvaluateSimpleFlowGate;
        expose XDesignDetailedDesign::EvaluateSimpleFlowGate::*;

        render Views::asInterconnectionDiagram;
    }

    view checkpointEvaluationFlow : StandardViewDefinitions::ActionFlowView {
        @XDesignCommon::DocSection { document = "docs/design/flows/simple-flow/definition.md"; section = "D11"; }
        @XDesignCommon::DocFigure { document = "docs/design/flows/simple-flow/definition.md"; figure = "docs/design/flows/simple-flow/assets/checkpoint-evaluation.svg"; }
        doc /* The checkpoint class's evaluate operation: read the class's rules, take the
           syntactic or the semantic path its adaptation fixes, and append the assessment. */

        expose XDesignDetailedDesign::EvaluateSimpleFlowCheckpoint;
        expose XDesignDetailedDesign::EvaluateSimpleFlowCheckpoint::*;

        render Views::asInterconnectionDiagram;
    }

    item def SimpleFlowFileEntry {
        @XDesignCommon::DocSection { document = "docs/records/plans/simple-flow-file-plan.md"; section = "D53"; }
        doc /* One repository file in the SimpleFlow implementation plan. Its path
           names source, tests, or a checked input, never generated cache output. */
        attribute repositoryPath : String;
        attribute responsibility : String;
    }

    item simpleFlowFilePlan {
        @XDesignCommon::DiagramPresentation { label = "SimpleFlow file plan"; summary = "Source, checker and test files"; }
        @XDesignCommon::DocSection { document = "docs/records/plans/simple-flow-file-plan.md"; section = "D53"; }
        doc /* Planned repository layout for the first executable SimpleFlow slice.
           Rust modules follow the XDesignRustUml module design; four Python
           commands implement the D06 syntactic checker rule sets. The test files
           exercise the same contracts independently. Generated reports and tool
           caches belong under cache/ and are not source entries here. */

        item application {
            item stageCli : SimpleFlowFileEntry { attribute redefines repositoryPath = "src/stage_cli/mod.rs"; attribute redefines responsibility = "Top-level stage command dispatch and interaction"; }
            item stageCliWork : SimpleFlowFileEntry { attribute redefines repositoryPath = "src/stage_cli/work.rs"; attribute redefines responsibility = "Frozen stage work definitions and artifact collection"; }
            item stageCliStore : SimpleFlowFileEntry { attribute redefines repositoryPath = "src/stage_cli/store.rs"; attribute redefines responsibility = "Wish registry, locks and state integrity"; }
            item stageCliPersistent : SimpleFlowFileEntry { attribute redefines repositoryPath = "src/stage_cli/persistent.rs"; attribute redefines responsibility = "Stored-head stage adapter and atomic revision commits"; }
            item storageModule : SimpleFlowFileEntry { attribute redefines repositoryPath = "src/storage/mod.rs"; attribute redefines responsibility = "Public storage records, errors and Store API"; }
            item storageDb : SimpleFlowFileEntry { attribute redefines repositoryPath = "src/storage/db.rs"; attribute redefines responsibility = "SQLite format, schema and consistent queries"; }
            item storageBlobs : SimpleFlowFileEntry { attribute redefines repositoryPath = "src/storage/blobs.rs"; attribute redefines responsibility = "Content-addressed publication, verified reads and checkout"; }
            item storageOps : SimpleFlowFileEntry { attribute redefines repositoryPath = "src/storage/ops.rs"; attribute redefines responsibility = "Strict saves, operation receipts, recovery and doctor"; }
            item storageCli : SimpleFlowFileEntry { attribute redefines repositoryPath = "src/storage/cli.rs"; attribute redefines responsibility = "Versioned store command envelope and exit codes"; }
            item storageUnicode : SimpleFlowFileEntry { attribute redefines repositoryPath = "src/storage/unicode.rs"; attribute redefines responsibility = "Generated Unicode canonical-decomposition and case-folding tables for manifest-name equivalence"; }
            item restoreRun : SimpleFlowFileEntry { attribute redefines repositoryPath = "src/core/restore.rs"; attribute redefines responsibility = "Validated read-only run restoration"; }
            item manifest : SimpleFlowFileEntry { attribute redefines repositoryPath = "Cargo.toml"; attribute redefines responsibility = "Rust crate manifest"; }
            item crateLibrary : SimpleFlowFileEntry { attribute redefines repositoryPath = "src/lib.rs"; attribute redefines responsibility = "xdesign application module boundary"; }
            item commandLine : SimpleFlowFileEntry { attribute redefines repositoryPath = "src/main.rs"; attribute redefines responsibility = "Local run and check entry point"; }
            item commandLineTests : SimpleFlowFileEntry { attribute redefines repositoryPath = "src/main_wish_tests.rs"; attribute redefines responsibility = "Command-line wish validation cases"; }
            item coreModule : SimpleFlowFileEntry { attribute redefines repositoryPath = "src/core/mod.rs"; attribute redefines responsibility = "Core public API within xdesign"; }
            item status : SimpleFlowFileEntry { attribute redefines repositoryPath = "src/core/status.rs"; attribute redefines responsibility = "Flow and gate status vocabulary"; }
            item plan : SimpleFlowFileEntry { attribute redefines repositoryPath = "src/core/plan.rs"; attribute redefines responsibility = "Stage, checkpoint and checker plan structural validation"; }
            item record : SimpleFlowFileEntry { attribute redefines repositoryPath = "src/core/record.rs"; attribute redefines responsibility = "Visits, activities and assessments"; }
            item ports : SimpleFlowFileEntry { attribute redefines repositoryPath = "src/core/ports.rs"; attribute redefines responsibility = "Checker and judge boundaries"; }
            item gate : SimpleFlowFileEntry { attribute redefines repositoryPath = "src/core/gate.rs"; attribute redefines responsibility = "Per-checkpoint evaluation and report validation"; }
            item run : SimpleFlowFileEntry { attribute redefines repositoryPath = "src/core/run.rs"; attribute redefines responsibility = "Run state, authorization, advance and reopen"; }
            item coreFactory : SimpleFlowFileEntry { attribute redefines repositoryPath = "src/core/factory.rs"; attribute redefines responsibility = "Create a run from any structurally valid plan"; }
            item simpleFlowModule : SimpleFlowFileEntry { attribute redefines repositoryPath = "src/simple_flow/mod.rs"; attribute redefines responsibility = "SimpleFlow public API within xdesign"; }
            item checkpointModule : SimpleFlowFileEntry { attribute redefines repositoryPath = "src/checkpoint/mod.rs"; attribute redefines responsibility = "Checkpoint library public API within xdesign"; }
            item checkpointLibrary : SimpleFlowFileEntry { attribute redefines repositoryPath = "src/checkpoint/library.rs"; attribute redefines responsibility = "Checkpoint library model, validation, built-in seeds, atomic storage and scaffold generation"; }
            item checkpointCli : SimpleFlowFileEntry { attribute redefines repositoryPath = "src/checkpoint/cli.rs"; attribute redefines responsibility = "Checkpoint list, show and save command envelope"; }
            item blueprint : SimpleFlowFileEntry { attribute redefines repositoryPath = "src/simple_flow/blueprint.rs"; attribute redefines responsibility = "The designed four-stage plan as data and its blueprint conformance"; }
            item simpleFlowFactory : SimpleFlowFileEntry { attribute redefines repositoryPath = "src/simple_flow/factory.rs"; attribute redefines responsibility = "Create a conforming designed run"; }
            item processCheck : SimpleFlowFileEntry { attribute redefines repositoryPath = "src/simple_flow/check.rs"; attribute redefines responsibility = "Fixed four-checker process adapter"; }
        }
        item checkCommands {
            item sharedContract : SimpleFlowFileEntry { attribute redefines repositoryPath = "tools/checks/common.py"; attribute redefines responsibility = "Shared report schema and input validation"; }
            item requirements : SimpleFlowFileEntry { attribute redefines repositoryPath = "tools/checks/simple_requirements.py"; attribute redefines responsibility = "Requirements gate rules"; }
            item design : SimpleFlowFileEntry { attribute redefines repositoryPath = "tools/checks/simple_design.py"; attribute redefines responsibility = "Design gate rules"; }
            item implementation : SimpleFlowFileEntry { attribute redefines repositoryPath = "tools/checks/simple_implementation.py"; attribute redefines responsibility = "Implementation gate rules"; }
            item record : SimpleFlowFileEntry { attribute redefines repositoryPath = "tools/checks/simple_record.py"; attribute redefines responsibility = "Verification record gate rules"; }
        }
        item testCases {
            item stageCliTests : SimpleFlowFileEntry { attribute redefines repositoryPath = "tests/stage_cli.rs"; attribute redefines responsibility = "CLI process, restart and failure integration cases"; }
            item storageTests : SimpleFlowFileEntry { attribute redefines repositoryPath = "tests/storage.rs"; attribute redefines responsibility = "Storage schema, materials, idempotency, conflict and crash recovery cases"; }
            item storageCliTests : SimpleFlowFileEntry { attribute redefines repositoryPath = "tests/storage_cli.rs"; attribute redefines responsibility = "Store command, query, checkout and cache-loss process cases"; }
            item storagePathsTests : SimpleFlowFileEntry { attribute redefines repositoryPath = "tests/storage_paths.rs"; attribute redefines responsibility = "Destination-ancestry and bounded streaming checkout cases"; }
            item storageProtocolTests : SimpleFlowFileEntry { attribute redefines repositoryPath = "tests/storage_protocol.rs"; attribute redefines responsibility = "Terminal-receipt retry and source-omission attribution cases"; }
            item storageMemoryTests : SimpleFlowFileEntry { attribute redefines repositoryPath = "tests/storage_memory.rs"; attribute redefines responsibility = "Bounded checkout peak-heap regression case"; }
            item checkerTimeoutTests : SimpleFlowFileEntry { attribute redefines repositoryPath = "tests/checker_timeout.rs"; attribute redefines responsibility = "Checker process-tree deadline, group-kill and retained-output cases"; }
            item restoreTests : SimpleFlowFileEntry { attribute redefines repositoryPath = "tests/run_restore.rs"; attribute redefines responsibility = "Run restoration corruption and history cases"; }
            item runCases : SimpleFlowFileEntry { attribute redefines repositoryPath = "tests/simple_flow.rs"; attribute redefines responsibility = "Plan, authorization, recovery and reopen cases"; }
            item modelPlanCases : SimpleFlowFileEntry { attribute redefines repositoryPath = "tests/plan_model_sync.rs"; attribute redefines responsibility = "Frozen rule and model declaration agreement"; }
            item processCheckCases : SimpleFlowFileEntry { attribute redefines repositoryPath = "tests/process_check_contract.rs"; attribute redefines responsibility = "Process adapter report contract cases"; }
            item checkerCases : SimpleFlowFileEntry { attribute redefines repositoryPath = "tests/test_simple_flow_checkers.py"; attribute redefines responsibility = "Rule and report contract cases"; }
            item designLayoutCases : SimpleFlowFileEntry { attribute redefines repositoryPath = "tests/test_simple_flow_design_layout.py"; attribute redefines responsibility = "Design declaration layout cases"; }
        }
        item checkerResources : SimpleFlowFileEntry { attribute redefines repositoryPath = "src/simple_flow/resources.rs"; attribute redefines responsibility = "Embedded resources, executor freezing and project Python context"; }
        item checkerIsolationCases : SimpleFlowFileEntry { attribute redefines repositoryPath = "tests/checker_isolation.rs"; attribute redefines responsibility = "Released-binary checker ownership, dependency and project isolation acceptance"; }
    }

    view simpleFlowFilePlanView : StandardViewDefinitions::BrowserView {
        @XDesignCommon::DocSection { document = "docs/records/plans/simple-flow-file-plan.md"; section = "D53"; }
        @XDesignCommon::DocFigure { document = "docs/records/plans/simple-flow-file-plan.md"; figure = "docs/records/plans/assets/simple-flow-file-plan.svg"; }
        doc /* Which repository files realize the Rust run, fixed checker commands,
           and executable verification, with generated evidence kept in cache/. */
        expose XDesignDetailedDesign::simpleFlowFilePlan;
    }

    view interfaceStructure : StandardViewDefinitions::InterconnectionView {
        @XDesignCommon::DocSection { document = "docs/design/flows/simple-flow/views.md"; section = "D10"; }
        @XDesignCommon::DocFigure { document = "docs/design/flows/simple-flow/views.md"; figure = "docs/design/flows/simple-flow/assets/interfaces.svg"; }
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
        @XDesignCommon::DocSection { document = "docs/design/flows/simple-flow/views.md"; section = "D16"; }
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
        @XDesignCommon::DocSection { document = "docs/design/flows/simple-flow/views.md"; section = "D16"; }
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
        @XDesignCommon::DocSection { document = "docs/design/flows/simple-flow/views.md"; section = "D20"; }
        doc /* Lifecycle of one run, distinct from the four engineering stages.
           The factory activates the run. A failed, missing or stale gate blocks it;
           corrective work resumes the same current visit at a new artifact revision.
           Only authorization of a terminal visit permits completion. Advancing an
           already completed run revalidates its retained artifacts and passed
           assessment evidence; loss blocks it, and restoration permits completion
           again. An explicit rework action appends a new terminal visit without
           erasing history. */
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
        transition revalidateCompleted first completed
            if run.status == FlowStatus::completed and
            run.currentRecord.completionAuthorized then completed;
        @XDesignCommon::DiagramPresentation about revalidateCompleted { label = "Revalidate"; summary = "retained evidence still available"; }
        transition completedEvidenceLost first completed
            if run.status == FlowStatus::blocked and
            run.currentRecord.status == StageStatus::blocked then blocked;
        @XDesignCommon::DiagramPresentation about completedEvidenceLost { label = "Evidence lost"; summary = "completed run is blocked"; }
        transition restoredCompletion first blocked
            if run.status == FlowStatus::completed and
            run.currentRecord.status == StageStatus::completed and
            run.currentRecord.stage.completesFlow then completed;
        @XDesignCommon::DiagramPresentation about restoredCompletion { label = "Restore"; summary = "retained evidence restored"; }
        transition reopenForRework first completed
            if run.status == FlowStatus::active and
            run.currentRecord.status == StageStatus::active then active;
        @XDesignCommon::DiagramPresentation about reopenForRework { label = "Rework"; summary = "run and new visit active"; }
    }

    use case def ProgressWish {
        @XDesignCommon::DiagramPresentation { label = "Progress wish"; summary = "Reach an evidence-backed delivery"; }
        @XDesignCommon::DocSection { document = "docs/requirements/requirements.md"; section = "D17"; }
        doc /* The requester's goal: progress a wish to an evidence-backed delivery,
           inspect its record and request corrective work when evidence is insufficient.
           Gate evaluation is internal behavior, not another external user goal. */
        subject automation : SimpleFlowAutomation;
        actor requester : WishRequester;
    }

    use case def ReviewEvidence {
        @XDesignCommon::DiagramPresentation { label = "Review evidence"; summary = "Return a reasoned semantic verdict"; }
        @XDesignCommon::DocSection { document = "docs/requirements/requirements.md"; section = "D17"; }
        doc /* A permitted agent or human judge reviews semantic criteria, artifacts
           and evidence, and returns a reasoned verdict through JudgmentInterface.
           The checker command remains an internal part of the automation. */
        subject automation : SimpleFlowAutomation;
        actor reviewer : Judge;
    }

    use case progressWish : ProgressWish {
        @XDesignCommon::DiagramPresentation { label = "Progress wish"; summary = "Reach an evidence-backed delivery"; }
        @XDesignCommon::DocSection { document = "docs/requirements/requirements.md"; section = "D17"; }
    }
    use case reviewEvidence : ReviewEvidence {
        @XDesignCommon::DiagramPresentation { label = "Review evidence"; summary = "Return a reasoned semantic verdict"; }
        @XDesignCommon::DocSection { document = "docs/requirements/requirements.md"; section = "D17"; }
    }

    part def DiagramDocumentationRenderer {
        @XDesignCommon::DocSection { document = "docs/interfaces/documentation-contract.md"; section = "D22"; }
        @XDesignCommon::DiagramPresentation { label = "Diagram renderer"; summary = "Focused model views become readable documentation figures"; }
        doc /* The documentation rendering utility, separate from runtime flow
           automation. It consumes native view selections and compiler exports,
           composes each diagram using its own notation, lays out nodes and edges
           through Graphviz, and writes SVGs with native source references. Short
           labels are presentation metadata; the source model retains full text.
           Figure generation does not establish engineering acceptance by itself. */

        perform action buildFigures {
            @XDesignCommon::DocSection { document = "docs/interfaces/documentation-contract.md"; section = "D23"; }
            in attribute sourceUris : String [1..*];
            in attribute selectedViewNames : String [1..*];
            out attribute figureUris : String [1..*];
            out attribute manifestUri : String;

            action exportModel {
                @XDesignCommon::DocSection { document = "docs/interfaces/documentation-contract.md"; section = "D23"; }
                doc /* Export native IDs, names, documentation, view selections,
                   metadata and relationships through the pinned compiler. Preserve
                   its diagnostics and disclose omitted export semantics. */
            }
            then action projectView {
                @XDesignCommon::DocSection { document = "docs/interfaces/documentation-contract.md"; section = "D23"; }
                doc /* Resolve the native view's selected elements and endpoints.
                   Apply short presentation labels and diagram-specific notation,
                   retaining source identities and full model details. Do not
                   fabricate relationships to improve composition. */
            }
            then action layoutGraph {
                @XDesignCommon::DocSection { document = "docs/interfaces/documentation-contract.md"; section = "D23"; }
                doc /* Use Graphviz rank placement, ports, clusters and routed
                   edges for focused graph views. Sequence diagrams use ordered
                   event-anchored lifelines. Preserve the meaning of each native
                   relationship, multiplicity, binding and control guard. */
            }
            then action renderFigure {
                @XDesignCommon::DocSection { document = "docs/interfaces/documentation-contract.md"; section = "D23"; }
                doc /* Write the final SVG with readable labels, source identity
                   references and full-text tooltips. Generate a raster preview
                   from that SVG, not from a previous layout or source revision. */
            }
            then action validateFinalFigure {
                @XDesignCommon::DocSection { document = "docs/interfaces/documentation-contract.md"; section = "D24"; }
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
        @XDesignCommon::DocSection { document = "docs/interfaces/documentation-contract.md"; section = "D29"; }
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
        @XDesignCommon::DocSection { document = "docs/design/flows/simple-flow/views.md"; section = "D13"; }
        @XDesignCommon::DocFigure { document = "docs/design/flows/simple-flow/views.md"; figure = "docs/design/flows/simple-flow/assets/block-definition.svg"; }
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
        @XDesignCommon::DocSection { document = "docs/design/flows/simple-flow/views.md"; section = "D14"; }
        @XDesignCommon::DocFigure { document = "docs/design/flows/simple-flow/views.md"; figure = "docs/design/flows/simple-flow/assets/internal-block.svg"; }
        doc /* How do this automation's run, checker and judge connect through ports? */
        expose XDesignDetailedDesign::SimpleFlowAutomation;
        render Views::asInterconnectionDiagram;
    }

    view requirementsDiagram : StandardViewDefinitions::GeneralView {
        @XDesignCommon::DiagramPresentation { label = "Requirements coverage"; question = "Which requirements have satisfiers and planned verification?"; }
        @XDesignCommon::DocSection { document = "docs/verification/requirements.md"; section = "D15"; }
        @XDesignCommon::DocFigure { document = "docs/verification/requirements.md"; figure = "docs/verification/assets/requirements.svg"; }
        doc /* Which requirements have design satisfiers and a planned review case?
           A verify relationship states coverage; it does not assert a passed test. */
        expose XDesignRequirements::flowRunIsRealizedByDesign;
        expose XDesignRequirements::designedFlowGatesEveryStage;
        expose XDesignRequirements::structureGraphsAreDocumented;
        expose XDesignDetailedDesign::SimpleFlowRun;
        expose XDesignDetailedDesign::DesignedSimpleFlow;
        expose XDesignCommon::DocFigure;
        expose XDesignVerification::SimpleFlowViewReview;
    }

    view parametricDiagram : StandardViewDefinitions::GeneralView {
        @XDesignCommon::DiagramPresentation { label = "Visit authorization"; question = "Which current visit values authorize advancement?"; }
        @XDesignCommon::DocSection { document = "docs/design/flows/simple-flow/views.md"; section = "D16"; }
        @XDesignCommon::DocFigure { document = "docs/design/flows/simple-flow/views.md"; figure = "docs/design/flows/simple-flow/assets/parametric.svg"; }
        doc /* Which bound record values determine authorization at this revision? */
        expose XDesignDetailedDesign::SimpleFlowAuthorization;
        expose XDesignDetailedDesign::SimpleFlowAuthorization::*;
        expose XDesignDetailedDesign::GateAuthorization;
    }

    view useCaseDiagram : StandardViewDefinitions::GeneralView {
        @XDesignCommon::DiagramPresentation { label = "Actor goals"; question = "Which goals belong to each external actor?"; }
        @XDesignCommon::DocSection { document = "docs/requirements/requirements.md"; section = "D17"; }
        @XDesignCommon::DocFigure { document = "docs/requirements/requirements.md"; figure = "docs/requirements/assets/use-cases.svg"; }
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
        @XDesignCommon::DocSection { document = "docs/design/flows/simple-flow/views.md"; section = "D18"; }
        @XDesignCommon::DocFigure { document = "docs/design/flows/simple-flow/views.md"; figure = "docs/design/flows/simple-flow/assets/activity.svg"; }
        doc /* How is one checkpoint evaluated, by a script or a permitted judge? */
        expose XDesignDetailedDesign::EvaluateSimpleFlowCheckpoint;
        expose XDesignDetailedDesign::EvaluateSimpleFlowCheckpoint::*;
        render Views::asInterconnectionDiagram;
    }

    view sequenceDiagram : StandardViewDefinitions::SequenceView {
        @XDesignCommon::DiagramPresentation { label = "Run interactions"; question = "In which order are the four stages entered and assessed?"; }
        @XDesignCommon::DocSection { document = "docs/design/flows/simple-flow/views.md"; section = "D19"; }
        @XDesignCommon::DocFigure { document = "docs/design/flows/simple-flow/views.md"; figure = "docs/design/flows/simple-flow/assets/sequence.svg"; }
        doc /* In which order do the controller, run and stages exchange events? */
        expose XDesignDetailedDesign::SimpleFlowRunTimeline;
    }

    view stateMachineDiagram : StandardViewDefinitions::StateTransitionView {
        @XDesignCommon::DiagramPresentation { label = "Run lifecycle"; question = "When does a run activate, block, resume or complete?"; }
        @XDesignCommon::DocSection { document = "docs/design/flows/simple-flow/views.md"; section = "D20"; }
        @XDesignCommon::DocFigure { document = "docs/design/flows/simple-flow/views.md"; figure = "docs/design/flows/simple-flow/assets/state-machine.svg"; }
        doc /* Which conditions activate, block, resume and complete a run? */
        expose XDesignDetailedDesign::SimpleFlowRunLifecycle;
        expose XDesignDetailedDesign::SimpleFlowRunLifecycle::*;
        render Views::asInterconnectionDiagram;
    }

    view packageDiagram : StandardViewDefinitions::GeneralView {
        @XDesignCommon::DiagramPresentation { label = "Model packages"; question = "How do the project model concerns depend on one another?"; }
        @XDesignCommon::DocSection { document = "docs/architecture/system-composition.md"; section = "D21"; }
        @XDesignCommon::DocFigure { document = "docs/architecture/system-composition.md"; figure = "docs/architecture/assets/simple-flow-packages.svg"; }
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
        @XDesignCommon::DocSection { document = "docs/design/flows/simple-flow/views.md"; section = "D13"; }
        subject blockDefinitionDiagram;
        attribute redefines concernText = "Which reusable types make up a run?";
        attribute redefines stakeholderRole = "flow designer";
    }

    viewpoint internalBlockConcern : DocumentationViewpoint {
        @XDesignCommon::DocSection { document = "docs/design/flows/simple-flow/views.md"; section = "D14"; }
        subject internalBlockDiagram;
        attribute redefines concernText = "How do the run, checker and judge connect?";
        attribute redefines stakeholderRole = "integrator";
    }

    viewpoint requirementsConcern : DocumentationViewpoint {
        @XDesignCommon::DocSection { document = "docs/verification/requirements.md"; section = "D15"; }
        subject requirementsDiagram;
        attribute redefines concernText = "Which requirements have satisfiers and planned verification?";
        attribute redefines stakeholderRole = "requirements reviewer";
    }

    viewpoint parametricConcern : DocumentationViewpoint {
        @XDesignCommon::DocSection { document = "docs/design/flows/simple-flow/views.md"; section = "D16"; }
        subject parametricDiagram;
        attribute redefines concernText = "Which current visit values authorize advancement?";
        attribute redefines stakeholderRole = "verification engineer";
    }

    viewpoint useCaseConcern : DocumentationViewpoint {
        @XDesignCommon::DocSection { document = "docs/requirements/requirements.md"; section = "D17"; }
        subject useCaseDiagram;
        attribute redefines concernText = "Which goals belong to each external actor?";
        attribute redefines stakeholderRole = "requester";
    }

    viewpoint activityConcern : DocumentationViewpoint {
        @XDesignCommon::DocSection { document = "docs/design/flows/simple-flow/views.md"; section = "D18"; }
        subject activityDiagram;
        attribute redefines concernText = "How does one gate produce an assessment?";
        attribute redefines stakeholderRole = "implementer";
    }

    viewpoint sequenceConcern : DocumentationViewpoint {
        @XDesignCommon::DocSection { document = "docs/design/flows/simple-flow/views.md"; section = "D19"; }
        subject sequenceDiagram;
        attribute redefines concernText = "In which order are the four stages entered and assessed?";
        attribute redefines stakeholderRole = "integrator";
    }

    viewpoint stateMachineConcern : DocumentationViewpoint {
        @XDesignCommon::DocSection { document = "docs/design/flows/simple-flow/views.md"; section = "D20"; }
        subject stateMachineDiagram;
        attribute redefines concernText = "When does a run activate, block, resume or complete?";
        attribute redefines stakeholderRole = "maintainer";
    }

    viewpoint packageConcern : DocumentationViewpoint {
        @XDesignCommon::DocSection { document = "docs/architecture/system-composition.md"; section = "D21"; }
        subject packageDiagram;
        attribute redefines concernText = "How do the project model concerns depend on one another?";
        attribute redefines stakeholderRole = "model maintainer";
    }

    part def DesignedWishVerifyFlow :> WishVerifyFlow {
        @XDesignCommon::DiagramPresentation { label = "Gated wish-verify flow"; summary = "Two judged stages with explicit exit gates"; }
        @XDesignCommon::DocSection { document = "docs/design/flows/wish-verify/definition.md"; section = "D57"; }
        doc /* The wish-verify flow specialization: the same two stages in the
           same order, with every stage's exit gated. Both gates are semantic —
           the architecture states that nothing in this flow's outcome is
           established by a fixed check, so the designed flow declares no
           checker, and a permitted agent or human judges each exit.
           xdesign.designedWishVerifyFlow is the selectable usage of this
           definition. */

        part redefines checkpoints : WishVerifyCheckpoint [2];

        part wishAccepted : WishVerifySemanticCheckpoint subsets checkpoints {
            @XDesignCommon::DocSection { document = "docs/design/flows/wish-verify/definition.md"; section = "D61"; }
            doc /* Exit gate of the wish stage. The wish stage has no exit gate
               in the architecture's flow; a designed flow gates every stage, so
               the judgment that the request and its answering artifacts are
               recorded is made explicit here. */
            ref part redefines sourceStage = wish;
            attribute redefines criteria = (
                "The wish statement is recorded, and the artifacts that answer it are named in the current visit.",
                "A reader can see from the recorded artifacts what the wish asked and what was produced for it, without re-asking the requester.");
            attribute redefines agentMayJudge = true;
            attribute redefines humanMayJudge = true;
        }

        part redefines wish {
            attribute redefines goal = "Record the wish and the artifacts that answer it.";
            attribute redefines workInstructions = "Record the requested outcome and its answering artifacts, then supply a permitted semantic judgment.";
            attribute redefines inputArtifacts = "wish:model";
            attribute redefines expectedOutputs = "wish:model";
            ref part redefines exitCheckpoints = wishAccepted;
        }
        part redefines 'verify' {
            attribute redefines goal = "Demonstrate that the implementation satisfies the requirements.";
            attribute redefines workInstructions = "Run the planned tests, retain verification evidence and supply a permitted semantic judgment.";
            attribute redefines inputArtifacts = "wish:model";
            attribute redefines expectedOutputs = "result:artifacts";
            ref part redefines exitCheckpoints = outcomeAccepted;
        }
        part redefines outcomeAccepted : WishVerifySemanticCheckpoint {
            @XDesignCommon::DocSection { document = "docs/design/flows/wish-verify/definition.md"; section = "D61"; }
            doc /* The flow's inherited terminal gate, retyped to the designed
               checkpoint class so the run evaluates it through the class. Its
               criteria, kind and judge flags stay exactly as the architecture
               states them. */
        }
    }

    part def WishVerifyCheckpoint :> Checkpoint {
        @XDesignCommon::DiagramPresentation { label = "Checkpoint"; summary = "Presents one gate's criteria to a permitted judge"; }
        @XDesignCommon::DocSection { document = "docs/design/flows/wish-verify/definition.md"; section = "D63"; }
        doc /* One checkpoint of the designed wish-verify flow. Unlike the simple
           flow's class it needs no syntactic adaptation: this flow's gates are
           all judged. The class owns the evaluate operation and the assessments
           that operation produces, so a gate is judged through the class and
           the record points back to it — the loop closes on the class. */

        ref item assessments : CheckpointAssessment [0..*];

        perform action evaluate : EvaluateWishVerifyCheckpoint;
    }

    part def WishVerifySemanticCheckpoint :> WishVerifyCheckpoint {
        @XDesignCommon::DiagramPresentation { label = "Semantic judgment"; summary = "A permitted judge records a reasoned verdict"; }
        @XDesignCommon::DocSection { document = "docs/design/flows/wish-verify/definition.md"; section = "D63"; }
        doc /* The only adaptation of the wish-verify checkpoint class: criteria
           a permitted judge reads, with no checker. Its evaluate operation
           presents the criteria and the visit's artifacts and evidence to the
           judge and records the rationale. The logic a criterion states stays
           prose for the judge; the design fixes who may judge it and what the
           run records. */

        attribute redefines kind = CheckKind::semantic;

        perform action redefines evaluate : TakeWishVerifyJudgment;

        assert constraint criteriaAreJudged {
            size(checkers) == 0 and (agentMayJudge or humanMayJudge)
        }
    }

    action def EvaluateWishVerifyCheckpoint {
        @XDesignCommon::DiagramPresentation { label = "Evaluate checkpoint"; summary = "Present criteria to a permitted judge"; }
        @XDesignCommon::DocSection { document = "docs/design/flows/wish-verify/definition.md"; section = "D63"; }
        doc /* The evaluate operation of the wish-verify checkpoint class, for one
           gate and one visit: read the gate's criteria and the judges its flags
           permit, take the judgment, and append one assessment. The designed
           flow declares no syntactic gate, so the operation has a single judged
           path. */

        in part checkpoint : WishVerifyCheckpoint;
        in part visit : StageRecord;
        out item assessment : CheckpointAssessment [0..1];

        action readCriteria {
            @XDesignCommon::DiagramPresentation { label = "Read criteria"; }
            doc /* The gate's criteria and the judge authorities its flags
               permit. */
            out attribute criteria : String [0..*];
        }
        then action takeJudgment {
            @XDesignCommon::DiagramPresentation { label = "Request judgment"; }
            doc /* Present the criteria and the visit's artifacts and evidence
               through the judgment port to a permitted judge; the outcome and
               its rationale come back. */
        }
        then action appendAssessment {
            @XDesignCommon::DiagramPresentation { label = "Keep assessment"; }
            doc /* Append the assessment to the visit; its checkpoint is this
               class, and the class keeps the assessment as a member of its own
               evaluation history. */
        }

        first start then readCriteria;
    }

    action def TakeWishVerifyJudgment :> EvaluateWishVerifyCheckpoint {
        @XDesignCommon::DiagramPresentation { label = "Take judgment"; summary = "Review criteria with artifacts and evidence"; }
        @XDesignCommon::DocSection { document = "docs/design/flows/wish-verify/definition.md"; section = "D63"; }
        doc /* The semantic implementation of the wish-verify checkpoint class's
           evaluate operation: a permitted judge reads the criteria with the
           visit's artifacts and evidence, and the outcome and rationale are
           recorded. */

        in part redefines checkpoint : WishVerifySemanticCheckpoint;
    }

    part def WishVerifyFlowRun :> FlowExecution {
        @XDesignCommon::DiagramPresentation { label = "Wish-verify run"; summary = "Two stage visits, judgments and advancement"; }
        @XDesignCommon::DocSection { document = "docs/design/flows/wish-verify/implementation.md"; section = "D58"; }
        doc /* One run of the designed wish-verify flow: the class a wish's
           execution has. It refines the generic FlowExecution with the designed
           flow's plan — a visit to each of its two stages, more visits when the
           terminal stage is reworked — and carries operations to evaluate the
           current visit's gates, advance when authorized, and explicitly reopen
           a completed terminal visit. Its only automation port presents its
           gates to a judge; no check command exists to invoke. The factory
           creates it; the controller keeps it in the wish's trace. */

        ref part redefines assignedFlow : DesignedWishVerifyFlow;
        ref part checkpoints : WishVerifyCheckpoint [0..*] = assignedFlow.checkpoints;
        part redefines stageRecords : StageRecord [2..*];

        port judgment : ~JudgmentExchange {
            @XDesignCommon::DiagramPresentation { label = "review"; summary = "Present criteria and receive verdict"; }
        }

        event occurrence created;
        event occurrence completed;

        exhibit state lifecycle : WishVerifyRunLifecycle;
        bind lifecycle.run = self;

        perform action evaluateGate : EvaluateWishVerifyGate;
        perform action advance : AdvanceWishVerifyRun;
        perform action reopenStage : ReopenWishVerifyStage;

        assert constraint visitsRealizeThePlannedStages {
            stageRecords->forAll { in record : StageRecord;
                assignedFlow.stages->includes(record.stage)
            } and
            assignedFlow.stages->forAll { in stage : Stage;
                stageRecords->exists { in record : StageRecord;
                    record.stage == stage
                }
            }
        }
    }

    part def WishVerifyFlowRunFactory {
        @XDesignCommon::DiagramPresentation { label = "Run factory"; summary = "Creates a run and its initial visits"; }
        @XDesignCommon::DocSection { document = "docs/design/flows/wish-verify/implementation.md"; section = "D59"; }
        doc /* The factory of a wish-verify run: given a wish and the designed
           flow's plan, it creates the run the controller will hold. It realizes
           registration for this flow — the part of Controller::receiveWish that
           assigns the flow, opens the run and enters its initial stage — so an
           implementation of this flow does not have to re-invent it from the
           generic controller. */

        perform action createRun {
            @XDesignCommon::DocSection { document = "docs/design/flows/wish-verify/implementation.md"; section = "D59"; }
            doc /* Open a run of the designed wish-verify flow for a wish: create
               the run bound to the plan, add one pending visit per stage, enter
               the wish visit as current, and activate the run. The wish itself
               stays an item of the workflow; the trace that holds the run is
               the controller's, produced by receiveWish. */

            in part wish : Wish;
            out part run : WishVerifyFlowRun;

            action openRun {
                doc /* Create the run, bound to the designed flow's plan and
                   without visits yet. */
                out part run : WishVerifyFlowRun;
            }
            action addStageVisits {
                doc /* Add one pending visit per stage of the plan — wish and
                   verify — each bound to its stage. */
                in part redefines run = openRun.run;
            }
            action enterInitialStage {
                doc /* The wish visit becomes the current visit and is marked
                   active at artifact revision 0; the verify visit stays pending
                   at revision 0. Recording a content-change activity increments
                   a visit's revision before a gate assesses it. */
                in part redefines run = openRun.run;
            }
            action activateRun {
                doc /* The run becomes active, so the controller records
                   activities against its current visit and evaluates its gates. */
                in part redefines run = openRun.run;
            }

            first start then openRun;
            succession visitsAdded first openRun then addStageVisits;
            succession stageEntered first addStageVisits then enterInitialStage;
            succession runActivated first enterInitialStage then activateRun;
        }
    }

    action def EvaluateWishVerifyGate {
        @XDesignCommon::DiagramPresentation { label = "Evaluate stage gates"; summary = "Record one judgment per exit gate"; }
        @XDesignCommon::DocSection { document = "docs/design/flows/wish-verify/implementation.md"; section = "D60"; }
        doc /* Evaluate exactly the exit checkpoints of this visit's stage, once
           per checkpoint, in plan order. Each gate's own evaluate operation
           takes the judged path: its criteria and the visit's artifacts and
           evidence go through the judgment port to a permitted judge, whose
           rationale is recorded. Each evaluation appends one
           CheckpointAssessment to the visit; a gate with no current assessment
           authorizes nothing. */

        in part visit : StageRecord;
        ref part checkpoints : WishVerifyCheckpoint [0..*];
        out item assessments : CheckpointAssessment [0..*];

        action readGates {
            doc /* The checkpoint class instances of the visit's stage, read from
               the assigned flow's plan. Other stages' gates are excluded. */
            out part gates : WishVerifyCheckpoint [0..*] = checkpoints->select {
                in gate : WishVerifyCheckpoint;
                visit.stage.exitCheckpoints->includes(gate)
            };
        }
        then for gate in readGates.gates {
            action evaluateCurrentGate : EvaluateWishVerifyCheckpoint {
                doc /* The judged path is fixed by the gate's adaptation; this
                   action is performed once for each selected exit gate. */
                in part redefines checkpoint = gate;
                in part redefines visit = EvaluateWishVerifyGate::visit;
            }
        }

        first start then readGates;
    }

    action def AdvanceWishVerifyRun {
        @XDesignCommon::DiagramPresentation { label = "Advance flow run"; summary = "Authorization selects the next visit or completion"; }
        @XDesignCommon::DocSection { document = "docs/design/flows/wish-verify/implementation.md"; section = "D60"; }
        doc /* The successor rule of the designed wish-verify flow, for one run:
           when the current visit is authorized — its exit gate passed and cited
           at the visited revision — decide where the run goes. The wish stage
           hands the run to the verify stage's pending visit, which becomes
           current. The verify stage has none: the run completes and the flow
           status becomes completed, which is what lets the controller deliver
           the trace. An unauthorized visit remains current and blocked for
           correction at a later artifact revision. Reopening a completed
           terminal stage is a separate action. */

        in part run : WishVerifyFlowRun;
        out part nextVisit : StageRecord [0..1];

        action checkAuthorization {
            doc /* completionAuthorized at the visited revision: every exit gate
               of the current visit has a passed assessment cited at it, and any
               subflow has completed. */
        }
        then decide advancePath;
        action enterNextStage {
            doc /* The pending visit of the verify stage becomes the current
               visit; the run stays active. */
        }
        action retainBlockedVisit {
            doc /* Keep the unauthorized current visit selected and mark both the
               visit and the run blocked. Corrective work records a
               content-change activity in this visit and increments its artifact
               revision. */
            in part blockedVisit : StageRecord = run.currentRecord;
            assert constraint sameVisitIsBlocked {
                run.currentRecord == blockedVisit and
                blockedVisit.status == StageStatus::blocked and
                run.status == FlowStatus::blocked
            }
        }
        action completeRun {
            doc /* The current visit is the verify visit, whose stage completes
               the flow: the run completes and stays its current record. */
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

    action def ReopenWishVerifyStage {
        @XDesignCommon::DocSection { document = "docs/design/flows/wish-verify/implementation.md"; section = "D60"; }
        doc /* Explicit rework of a completed terminal visit. A completed run can
           reopen its terminal verify stage only on a recorded rework request.
           Keep the completed visit and its assessments immutable, append a new
           active visit for that stage at the next visit index with artifact
           revision 0, and return the run to active. Reopening an earlier
           completed stage requires downstream invalidation rules and is outside
           this action. */

        in part run : WishVerifyFlowRun;
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
            doc /* Preserve the prior visit and add a new active visit of the
               same stage with visitIndex one greater and artifactRevision 0. */
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

    part def WishVerifyAutomation {
        @XDesignCommon::DiagramPresentation { label = "Wish-verify automation"; summary = "Run and judge connected by the judgment port"; }
        @XDesignCommon::DocSection { document = "docs/design/flows/wish-verify/implementation.md"; section = "D67"; }
        doc /* The parts a run of the designed wish-verify flow is automated by,
           wired by their interfaces: the factory creates the run, and the run
           presents its gates to a judge through the judgment interface. No
           checker command is wired: this flow declares no syntactic gate. */

        part runner : WishVerifyFlowRun {
            @XDesignCommon::DiagramPresentation { label = "Flow run"; summary = "Automates the current stage visit"; }
        }
        part factory : WishVerifyFlowRunFactory;
        ref part checkpoints : WishVerifyCheckpoint [0..*] = runner.checkpoints;
        part judge : Judge {
            @XDesignCommon::DiagramPresentation { label = "Judge"; summary = "Agent or human reviews evidence"; }
        }

        interface judging : JudgmentInterface
            connect runner.judgment to judge.judgment;
    }

    occurrence def WishVerifyRunTimeline {
        @XDesignCommon::DiagramPresentation { label = "Run interactions"; summary = "Controller events enter and assess the two stages"; }
        @XDesignCommon::DocSection { document = "docs/design/flows/wish-verify/implementation.md"; section = "D68"; }
        doc /* One run of the designed wish-verify flow over time, whatever wish
           it answers: the run is created, the wish stage is entered and its
           gate assessed, the verify stage follows, and its completion finishes
           the run. The messages are the moments the flow defines; a particular
           wish's journey is the same shape with its own records. */

        ref part controller : Controller;
        ref part run : WishVerifyFlowRun;
        ref part 'flow' : DesignedWishVerifyFlow = run.assignedFlow;
        ref part wish : Stage = 'flow'.wish;
        ref part 'verify' : Stage = 'flow'.'verify';

        message createRun from controller.entering to run.created;
        message enterWish from controller.entering to wish.entered;
        message assessWish from controller.assessing to wish.assessed;
        message enterVerify from controller.entering to 'verify'.entered;
        message assessVerify from controller.assessing to 'verify'.assessed;
        message completeRun from controller.delivering to run.completed;
    }

    part def WishVerifyAuthorization {
        @XDesignCommon::DiagramPresentation { label = "Visit authorization"; summary = "Bind current visit values to the authorization constraint"; }
        @XDesignCommon::DocSection { document = "docs/design/flows/wish-verify/views.md"; section = "D70"; }
        doc /* Parametric context for a wish-verify stage visit, binding the same
           GateAuthorization constraint the simple flow defines. Counting gates
           rather than assessments prevents duplicate passes from compensating
           for a missing gate. A stale or failed assessment contributes
           nothing. */
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

    state def WishVerifyRunLifecycle {
        @XDesignCommon::DiagramPresentation { label = "Run lifecycle"; summary = "Activation, blocking, correction and completion"; }
        @XDesignCommon::DocSection { document = "docs/design/flows/wish-verify/implementation.md"; section = "D68"; }
        doc /* Lifecycle of one wish-verify run, distinct from the two
           engineering stages. The factory activates the run. A failed, missing
           or stale judgment blocks it; corrective work resumes the same current
           visit at a new artifact revision. Only authorization of the terminal
           verify visit permits completion. Advancing an already completed run
           revalidates its retained artifacts and passed assessment evidence;
           loss blocks it, and restoration permits completion again. An explicit
           rework action appends a new terminal visit without erasing history. */
        ref part run : WishVerifyFlowRun;
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
        transition revalidateCompleted first completed
            if run.status == FlowStatus::completed and
            run.currentRecord.completionAuthorized then completed;
        @XDesignCommon::DiagramPresentation about revalidateCompleted { label = "Revalidate"; summary = "retained evidence still available"; }
        transition completedEvidenceLost first completed
            if run.status == FlowStatus::blocked and
            run.currentRecord.status == StageStatus::blocked then blocked;
        @XDesignCommon::DiagramPresentation about completedEvidenceLost { label = "Evidence lost"; summary = "completed run is blocked"; }
        transition restoredCompletion first blocked
            if run.status == FlowStatus::completed and
            run.currentRecord.status == StageStatus::completed and
            run.currentRecord.stage.completesFlow then completed;
        @XDesignCommon::DiagramPresentation about restoredCompletion { label = "Restore"; summary = "retained evidence restored"; }
        transition reopenForRework first completed
            if run.status == FlowStatus::active and
            run.currentRecord.status == StageStatus::active then active;
        @XDesignCommon::DiagramPresentation about reopenForRework { label = "Rework"; summary = "run and new visit active"; }
    }

    use case def ProgressJudgedOutcome {
        @XDesignCommon::DiagramPresentation { label = "Progress judged outcome"; summary = "Reach an evidence-backed judged delivery"; }
        @XDesignCommon::DocSection { document = "docs/design/flows/wish-verify/views.md"; section = "D71"; }
        doc /* The requester's goal for a judged artifact: progress the wish to a
           delivery whose outcome a permitted judge accepted on recorded
           evidence. Gate evaluation is internal behavior, not another external
           user goal. */
        subject automation : WishVerifyAutomation;
        actor requester : WishRequester;
    }

    use case def JudgeRecordedOutcome {
        @XDesignCommon::DiagramPresentation { label = "Judge recorded outcome"; summary = "Return a reasoned verdict on the recorded outcome"; }
        @XDesignCommon::DocSection { document = "docs/design/flows/wish-verify/views.md"; section = "D71"; }
        doc /* A permitted agent or human judge reviews the gate criteria with
           the recorded artifacts and evidence, and returns a reasoned verdict
           through JudgmentInterface. */
        subject automation : WishVerifyAutomation;
        actor reviewer : Judge;
    }

    use case progressJudgedOutcome : ProgressJudgedOutcome {
        @XDesignCommon::DiagramPresentation { label = "Progress judged outcome"; summary = "Reach an evidence-backed judged delivery"; }
        @XDesignCommon::DocSection { document = "docs/design/flows/wish-verify/views.md"; section = "D71"; }
    }
    use case judgeRecordedOutcome : JudgeRecordedOutcome {
        @XDesignCommon::DiagramPresentation { label = "Judge recorded outcome"; summary = "Return a reasoned verdict on the recorded outcome"; }
        @XDesignCommon::DocSection { document = "docs/design/flows/wish-verify/views.md"; section = "D71"; }
    }

    item wishVerifyFlowFilePlan {
        @XDesignCommon::DiagramPresentation { label = "Wish-verify file plan"; summary = "Flow module and test files"; }
        @XDesignCommon::DocSection { document = "docs/records/plans/wish-verify.md"; section = "P27"; }
        doc /* Planned repository layout for the executable wish-verify slice.
           Only the files this slice owns are entries; the shared crate files it
           extends — src/lib.rs, src/main.rs and tests/plan_model_sync.rs among
           them — keep their SimpleFlow file-plan entries and responsibilities.
           Generated reports and tool caches belong under cache/ and are not
           source entries here. */

        item flowModule {
            item module : SimpleFlowFileEntry { attribute redefines repositoryPath = "src/wish_verify/mod.rs"; attribute redefines responsibility = "WishVerifyFlow public API within xdesign"; }
            item blueprint : SimpleFlowFileEntry { attribute redefines repositoryPath = "src/wish_verify/blueprint.rs"; attribute redefines responsibility = "The designed two-stage plan as data and its blueprint conformance"; }
            item factory : SimpleFlowFileEntry { attribute redefines repositoryPath = "src/wish_verify/factory.rs"; attribute redefines responsibility = "Create a conforming designed wish-verify run with the unused-check stub"; }
        }
        item testCases {
            item runCases : SimpleFlowFileEntry { attribute redefines repositoryPath = "tests/wish_verify.rs"; attribute redefines responsibility = "Plan conformance, judgment, authorization, recovery and reopen cases"; }
        }
    }

    view wishVerifyCliTimelineView : StandardViewDefinitions::GeneralView {
        @XDesignCommon::DiagramPresentation {
            label = "WishVerifyFlow CLI timeline";
            summary = "Simplest flow: wish -> verify -> completed; the same interaction at each stage";
            question = "How do next and check coordinate an agent iteration in the simplest two-stage WishVerifyFlow?";
        }
        @XDesignCommon::DocSection { document = "docs/design/flows/wish-verify/views.md"; section = "D84"; }
        @XDesignCommon::DocFigure { document = "docs/design/flows/wish-verify/views.md"; figure = "docs/design/flows/wish-verify/assets/cli-timeline.svg"; }
        doc /* The common stage CLI projected in the context of the simplest
           two-stage WishVerifyFlow. This example selects the gated specialization
           DesignedWishVerifyFlow: wishAccepted authorizes wish -> verify and
           outcomeAccepted authorizes terminal completion. Both gates require
           permitted agent or human judgment; neither invokes a fixed checker.
           The figure expands one AgentStageWork iteration through StageCli and
           Controller actions; the same pattern applies at each stage. It does
           not declare sequence Message usages or record an executed run.
           Repetition is documented intent, not a native action back-edge.
           Work metadata, artifact bindings and the next/check runtime remain
           separate integration obligations. */
        expose XDesignWorkflow::StageCli;
        expose XDesignWorkflow::StageWorkOrder;
        expose XDesignWorkflow::AgentStageWork;
        expose XDesignArchitecture::Controller;
        expose XDesignArchitecture::WishVerifyFlow;
        expose XDesignDetailedDesign::DesignedWishVerifyFlow;
        expose XDesignDetailedDesign::WishVerifySemanticCheckpoint;
    }

    view wishVerifyCliSwimlaneView : StandardViewDefinitions::GeneralView {
        @XDesignCommon::DiagramPresentation {
            label = "WishVerifyFlow CLI swimlane";
            summary = "Simplest flow: wish -> verify -> completed; agent work and xdesign responsibilities";
            question = "Who performs work, records evidence, judges gates and advances the simplest two-stage WishVerifyFlow?";
        }
        @XDesignCommon::DocSection { document = "docs/design/flows/wish-verify/views.md"; section = "D84"; }
        @XDesignCommon::DocFigure { document = "docs/design/flows/wish-verify/views.md"; figure = "docs/design/flows/wish-verify/assets/cli-swimlane.svg"; }
        doc /* Companion responsibility projection of the same two-stage CLI
           example as wishVerifyCliTimelineView. AgentStageWork produces workspace
           artifacts; StageCli collects them and delegates recordActivity and
           progressWish to Controller. The latter evaluates the current stage's
           semantic exit gate and advances or retains the visit. The completed
           run bypass preserves the read-only completion response. Lanes reflect
           action ownership and documented responsibility, not native allocations.
           Any repeat path is explicitly prose intent until an action loop is
           declared. Registration and final human acceptance remain outside this
           stage interaction; this view adds no runtime implementation. */
        expose XDesignWorkflow::StageCli;
        expose XDesignWorkflow::StageWorkOrder;
        expose XDesignWorkflow::AgentStageWork;
        expose XDesignArchitecture::Controller;
        expose XDesignArchitecture::WishVerifyFlow;
        expose XDesignDetailedDesign::DesignedWishVerifyFlow;
        expose XDesignDetailedDesign::WishVerifySemanticCheckpoint;
    }

    view wishVerifyFlowFilePlanView : StandardViewDefinitions::BrowserView {
        @XDesignCommon::DocSection { document = "docs/records/plans/wish-verify.md"; section = "P27"; }
        @XDesignCommon::DocFigure { document = "docs/records/plans/wish-verify.md"; figure = "docs/records/plans/assets/wish-verify-file-plan.svg"; }
        doc /* Which repository files realize the wish-verify flow module and its
           executable verification, with generated evidence kept in cache/. */
        expose XDesignDetailedDesign::wishVerifyFlowFilePlan;
    }

    view wishVerifyFlowTree : StandardViewDefinitions::BrowserView {
        @XDesignCommon::DocSection { document = "docs/design/flows/wish-verify/definition.md"; section = "D57"; }
        @XDesignCommon::DocFigure { document = "docs/design/flows/wish-verify/definition.md"; figure = "docs/design/flows/wish-verify/assets/designed-flow.svg"; }
        doc /* What the designed wish-verify flow is made of: its two stages and
           the two semantic exit checkpoints it declares for them. */
        expose XDesignDetailedDesign::DesignedWishVerifyFlow;
    }

    view wishVerifyDesignTree : StandardViewDefinitions::BrowserView {
        @XDesignCommon::DocSection { document = "docs/design/flows/wish-verify/implementation.md"; section = "D58"; }
        @XDesignCommon::DocFigure { document = "docs/design/flows/wish-verify/implementation.md"; figure = "docs/design/flows/wish-verify/assets/design-classes.svg"; }
        doc /* The design's classes, factory and functions with their members:
           the run's visits, judgment port and operations, the factory's
           creation steps, and the gate function's single judged path. */
        expose XDesignDetailedDesign::WishVerifyFlowRun;
        expose XDesignDetailedDesign::WishVerifyFlowRunFactory;
        expose XDesignDetailedDesign::EvaluateWishVerifyGate;
        expose XDesignDetailedDesign::AdvanceWishVerifyRun;
        expose XDesignDetailedDesign::WishVerifyCheckpoint;
        expose XDesignDetailedDesign::WishVerifySemanticCheckpoint;
        expose XDesignDetailedDesign::EvaluateWishVerifyCheckpoint;
    }

    view wishVerifyFactoryFlow : StandardViewDefinitions::ActionFlowView {
        @XDesignCommon::DocSection { document = "docs/design/flows/wish-verify/implementation.md"; section = "D59"; }
        @XDesignCommon::DocFigure { document = "docs/design/flows/wish-verify/implementation.md"; figure = "docs/design/flows/wish-verify/assets/factory.svg"; }
        doc /* The factory's creation steps in order: open the run, add the
           visits, enter the initial stage, activate. */
        expose XDesignDetailedDesign::WishVerifyFlowRunFactory::createRun;
        expose XDesignDetailedDesign::WishVerifyFlowRunFactory::createRun::*;
        render Views::asInterconnectionDiagram;
    }

    view wishVerifyGateEvaluationFlow : StandardViewDefinitions::ActionFlowView {
        @XDesignCommon::DocSection { document = "docs/design/flows/wish-verify/implementation.md"; section = "D60"; }
        @XDesignCommon::DocFigure { document = "docs/design/flows/wish-verify/implementation.md"; figure = "docs/design/flows/wish-verify/assets/gate-evaluation.svg"; }
        doc /* The gate function's shape: select only the visit stage's exit
           gates and delegate each to its judged evaluation path before
           recording one assessment per gate. */
        expose XDesignDetailedDesign::EvaluateWishVerifyGate;
        expose XDesignDetailedDesign::EvaluateWishVerifyGate::*;
        render Views::asInterconnectionDiagram;
    }

    view wishVerifyActivityDiagram : StandardViewDefinitions::ActionFlowView {
        @XDesignCommon::DiagramPresentation { label = "Checkpoint evaluation"; question = "How does one gate produce an assessment?"; }
        @XDesignCommon::DocSection { document = "docs/design/flows/wish-verify/definition.md"; section = "D63"; }
        @XDesignCommon::DocFigure { document = "docs/design/flows/wish-verify/definition.md"; figure = "docs/design/flows/wish-verify/assets/checkpoint-evaluation.svg"; }
        doc /* How is one checkpoint evaluated by a permitted judge? This flow
           declares no syntactic gate, so the activity has a single judged
           path. */
        expose XDesignDetailedDesign::EvaluateWishVerifyCheckpoint;
        expose XDesignDetailedDesign::EvaluateWishVerifyCheckpoint::*;
        render Views::asInterconnectionDiagram;
    }

    view wishVerifyPackageDiagram : StandardViewDefinitions::GeneralView {
        @XDesignCommon::DiagramPresentation { label = "Wish-verify packages"; question = "Which model packages carry this flow's design?"; }
        @XDesignCommon::DocSection { document = "docs/design/flows/wish-verify/implementation.md"; section = "D65"; }
        @XDesignCommon::DocFigure { document = "docs/design/flows/wish-verify/implementation.md"; figure = "docs/design/flows/wish-verify/assets/packages.svg"; }
        doc /* The packages that carry this flow's design: the architecture's
           WishVerifyFlow, this file's designed specialization, the composition's
           selectable usage, the Rust UML mapping and the planned verification. */
        expose XDesignArchitecture;
        expose XDesignDetailedDesign;
        expose XDesignComposition;
        expose XDesignRustUml;
        expose XDesignVerification;
    }

    view wishVerifyBlockDefinitionDiagram : StandardViewDefinitions::GeneralView {
        @XDesignCommon::DiagramPresentation { label = "Run structure"; question = "Which reusable types make up a wish-verify run?"; }
        @XDesignCommon::DocSection { document = "docs/design/flows/wish-verify/implementation.md"; section = "D66"; }
        @XDesignCommon::DocFigure { document = "docs/design/flows/wish-verify/implementation.md"; figure = "docs/design/flows/wish-verify/assets/block-definition.svg"; }
        doc /* Which reusable types form a wish-verify run, and how do they
           specialize and compose? */
        expose XDesignDetailedDesign::WishVerifyFlowRun;
        expose XDesignArchitecture::FlowExecution;
        expose XDesignDetailedDesign::WishVerifyFlowRunFactory;
        expose XDesignDetailedDesign::WishVerifyCheckpoint;
        expose XDesignDetailedDesign::WishVerifySemanticCheckpoint;
        expose XDesignArchitecture::StageRecord;
        expose XDesignArchitecture::CheckpointAssessment;
    }

    view wishVerifyInternalBlockDiagram : StandardViewDefinitions::InterconnectionView {
        @XDesignCommon::DiagramPresentation { label = "Automation connections"; question = "How do the run and the judge connect?"; }
        @XDesignCommon::DocSection { document = "docs/design/flows/wish-verify/implementation.md"; section = "D67"; }
        @XDesignCommon::DocFigure { document = "docs/design/flows/wish-verify/implementation.md"; figure = "docs/design/flows/wish-verify/assets/internal-block.svg"; }
        doc /* How do this automation's run and judge connect through ports? No
           checker command is wired: this flow declares no syntactic gate. */
        expose XDesignDetailedDesign::WishVerifyAutomation;
        render Views::asInterconnectionDiagram;
    }

    view wishVerifySequenceDiagram : StandardViewDefinitions::SequenceView {
        @XDesignCommon::DiagramPresentation { label = "Run interactions"; question = "In which order are the two stages entered and assessed?"; }
        @XDesignCommon::DocSection { document = "docs/design/flows/wish-verify/implementation.md"; section = "D68"; }
        @XDesignCommon::DocFigure { document = "docs/design/flows/wish-verify/implementation.md"; figure = "docs/design/flows/wish-verify/assets/sequence.svg"; }
        doc /* In which order do the controller, run and stages exchange events? */
        expose XDesignDetailedDesign::WishVerifyRunTimeline;
    }

    view wishVerifyStateMachineDiagram : StandardViewDefinitions::StateTransitionView {
        @XDesignCommon::DiagramPresentation { label = "Run lifecycle"; question = "When does a wish-verify run activate, block, resume or complete?"; }
        @XDesignCommon::DocSection { document = "docs/design/flows/wish-verify/implementation.md"; section = "D68"; }
        @XDesignCommon::DocFigure { document = "docs/design/flows/wish-verify/implementation.md"; figure = "docs/design/flows/wish-verify/assets/state-machine.svg"; }
        doc /* Which conditions activate, block, resume and complete a run? */
        expose XDesignDetailedDesign::WishVerifyRunLifecycle;
        expose XDesignDetailedDesign::WishVerifyRunLifecycle::*;
        render Views::asInterconnectionDiagram;
    }

    view wishVerifyRequirementsDiagram : StandardViewDefinitions::GeneralView {
        @XDesignCommon::DiagramPresentation { label = "Requirements coverage"; question = "Which requirements does this flow satisfy, and what verification is planned?"; }
        @XDesignCommon::DocSection { document = "docs/design/flows/wish-verify/views.md"; section = "D69"; }
        @XDesignCommon::DocFigure { document = "docs/design/flows/wish-verify/views.md"; figure = "docs/design/flows/wish-verify/assets/requirements.svg"; }
        doc /* Which requirements have design satisfiers in this flow and a
           planned review case? A verify relationship states coverage; it does
           not assert a passed test. */
        expose XDesignRequirements::designedFlowGatesEveryStage;
        expose XDesignRequirements::semanticCheckNamesItsJudges;
        expose XDesignRequirements::flowRunIsRealizedByDesign;
        expose XDesignRequirements::runRealizesItsDesign;
        expose XDesignDetailedDesign::DesignedWishVerifyFlow;
        expose XDesignDetailedDesign::WishVerifyFlowRun;
        expose XDesignVerification::WishVerifyFlowBehaviorVerification;
    }

    view wishVerifyParametricDiagram : StandardViewDefinitions::GeneralView {
        @XDesignCommon::DiagramPresentation { label = "Visit authorization"; question = "Which current visit values authorize advancement?"; }
        @XDesignCommon::DocSection { document = "docs/design/flows/wish-verify/views.md"; section = "D70"; }
        @XDesignCommon::DocFigure { document = "docs/design/flows/wish-verify/views.md"; figure = "docs/design/flows/wish-verify/assets/parametric.svg"; }
        doc /* Which bound record values determine authorization at this
           revision? */
        expose XDesignDetailedDesign::WishVerifyAuthorization;
        expose XDesignDetailedDesign::WishVerifyAuthorization::*;
        expose XDesignDetailedDesign::GateAuthorization;
    }

    view wishVerifyUseCaseDiagram : StandardViewDefinitions::GeneralView {
        @XDesignCommon::DiagramPresentation { label = "Actor goals"; question = "Which goals belong to each external actor?"; }
        @XDesignCommon::DocSection { document = "docs/design/flows/wish-verify/views.md"; section = "D71"; }
        @XDesignCommon::DocFigure { document = "docs/design/flows/wish-verify/views.md"; figure = "docs/design/flows/wish-verify/assets/use-cases.svg"; }
        doc /* Which external actors have which goals at this automation's
           boundary? */
        expose XDesignDetailedDesign::ProgressJudgedOutcome;
        expose XDesignDetailedDesign::JudgeRecordedOutcome;
        expose XDesignDetailedDesign::progressJudgedOutcome;
        expose XDesignDetailedDesign::judgeRecordedOutcome;
        expose XDesignDetailedDesign::WishVerifyAutomation;
        expose XDesignWorkflow::WishRequester;
        expose XDesignDetailedDesign::Judge;
    }

    viewpoint wishVerifyBlockDefinitionConcern : DocumentationViewpoint {
        @XDesignCommon::DocSection { document = "docs/design/flows/wish-verify/implementation.md"; section = "D66"; }
        subject wishVerifyBlockDefinitionDiagram;
        attribute redefines concernText = "Which reusable types make up a wish-verify run?";
        attribute redefines stakeholderRole = "flow designer";
    }

    viewpoint wishVerifyInternalBlockConcern : DocumentationViewpoint {
        @XDesignCommon::DocSection { document = "docs/design/flows/wish-verify/implementation.md"; section = "D67"; }
        subject wishVerifyInternalBlockDiagram;
        attribute redefines concernText = "How do the run and the judge connect?";
        attribute redefines stakeholderRole = "integrator";
    }

    viewpoint wishVerifyRequirementsConcern : DocumentationViewpoint {
        @XDesignCommon::DocSection { document = "docs/design/flows/wish-verify/views.md"; section = "D69"; }
        subject wishVerifyRequirementsDiagram;
        attribute redefines concernText = "Which requirements does this flow satisfy, and what verification is planned?";
        attribute redefines stakeholderRole = "requirements reviewer";
    }

    viewpoint wishVerifyParametricConcern : DocumentationViewpoint {
        @XDesignCommon::DocSection { document = "docs/design/flows/wish-verify/views.md"; section = "D70"; }
        subject wishVerifyParametricDiagram;
        attribute redefines concernText = "Which current visit values authorize advancement?";
        attribute redefines stakeholderRole = "verification engineer";
    }

    viewpoint wishVerifyUseCaseConcern : DocumentationViewpoint {
        @XDesignCommon::DocSection { document = "docs/design/flows/wish-verify/views.md"; section = "D71"; }
        subject wishVerifyUseCaseDiagram;
        attribute redefines concernText = "Which goals belong to each external actor?";
        attribute redefines stakeholderRole = "requester";
    }

    viewpoint wishVerifyActivityConcern : DocumentationViewpoint {
        @XDesignCommon::DocSection { document = "docs/design/flows/wish-verify/definition.md"; section = "D63"; }
        subject wishVerifyActivityDiagram;
        attribute redefines concernText = "How does one gate produce an assessment?";
        attribute redefines stakeholderRole = "implementer";
    }

    viewpoint wishVerifySequenceConcern : DocumentationViewpoint {
        @XDesignCommon::DocSection { document = "docs/design/flows/wish-verify/implementation.md"; section = "D68"; }
        subject wishVerifySequenceDiagram;
        attribute redefines concernText = "In which order are the two stages entered and assessed?";
        attribute redefines stakeholderRole = "integrator";
    }

    viewpoint wishVerifyStateMachineConcern : DocumentationViewpoint {
        @XDesignCommon::DocSection { document = "docs/design/flows/wish-verify/implementation.md"; section = "D68"; }
        subject wishVerifyStateMachineDiagram;
        attribute redefines concernText = "When does a wish-verify run activate, block, resume or complete?";
        attribute redefines stakeholderRole = "maintainer";
    }

    viewpoint wishVerifyPackageConcern : DocumentationViewpoint {
        @XDesignCommon::DocSection { document = "docs/design/flows/wish-verify/implementation.md"; section = "D65"; }
        subject wishVerifyPackageDiagram;
        attribute redefines concernText = "Which model packages carry this flow's design?";
        attribute redefines stakeholderRole = "model maintainer";
    }

    item def CheckerExecutorBundle {
        @XDesignCommon::DocSection { document = "docs/design/checker-execution.md"; section = "D111"; }
        attribute source : String;
        attribute entryUri : String;
        attribute python : String [0..1];
        attribute fileUris : String [1..*];
        attribute fileSha256 : String [1..*];
        doc /* A frozen plan binds each script entry to exact regular-file bytes.
           Bundle files are run-confined cache URIs backed by store materials.
           Python is an optional project-relative environment executable, not an
           archived environment. Interpreter/version/digest are recorded per attempt. */
    }
    part def ProjectCheckerExecution {
        @XDesignCommon::DocSection { document = "docs/design/checker-execution.md"; section = "D111"; }
        ref item bundle : CheckerExecutorBundle;
        attribute projectRoot : String;
        attribute attemptDirectory : String;
        doc /* Verify every retained executor digest before input discovery and
           evaluation. Execute with project cwd, cleared inherited Python import
           settings, user-site disabled and private temporary/cache paths. Augment
           reports with all executor-file hashes and interpreter identity, then
           apply the existing report and gate contracts. This is not an OS sandbox. */
    }

}
