package XDesignArchitecture {
    @XDesignCommon::DiagramPresentation { label = "Architecture"; }
    @XDesignCommon::DocSection { document = "docs/architecture/index.md"; section = "A05"; }
    private import ScalarValues::*;
    private import ControlFunctions::forAll;
    private import ControlFunctions::exists;
    private import ControlFunctions::select;
    private import SequenceFunctions::includes;
    private import SequenceFunctions::last;
    private import SequenceFunctions::size;
    private import XDesignCommon::*;
    private import XDesignStorage::*;

    doc /* Flow describes the workflow, Wish expresses the requested outcome,
       Trace retains its execution history and independent model reference, and
       Controller decides and applies the next step. */

    part def Stage {
        @XDesignCommon::DocSection { document = "docs/design/flow.md"; section = "F01"; }
        doc /* Reusable workflow stage definition. Successors allow branching
           and rework. Exit checkpoints are optional; every configured checkpoint
           must pass before leaving a stage.
           An optional subflow decomposes the work within this stage and must
           complete before the parent stage can finish.
           Verification stages may reference the development stage they check.
           The entered and assessed event occurrences anchor journey sequences
           that present a flow as a sequence view. */

        attribute goal : String [0..1];
        attribute workInstructions : String [0..1];
        attribute inputArtifacts : String [0..*];
        attribute expectedOutputs : String [0..*];

        part subFlow : Flow [0..1];
        ref part nextStages : Stage [0..*];
        ref part exitCheckpoints : Checkpoint [0..*];
        ref part verifiesStage : Stage [0..1];
        attribute completesFlow : Boolean default false;

        event occurrence entered;
        event occurrence assessed;
    }

    part def ProjectCheckContext {
        @XDesignCommon::DocSection { document = "docs/design/checkpoint-library.md"; section = "D107"; }
        doc /* As-built local project boundary. The operator selects one project
           root and one operational store root. Paths in library definitions are
           relative to that project, not to the xdesign executable repository.
           A filesystem path is not a stable cross-project identity. */
        attribute projectRoot : String;
        part checkpointLibrary : CheckpointLibrarySource;
        ref part store : XDesignStorage::ProjectStore;
    }

    part def CheckpointLibrarySource {
        @XDesignCommon::DocSection { document = "docs/design/checkpoint-library.md"; section = "D107"; }
        doc /* Git-owned reusable definitions and scripts. This is authoring
           source, not a record of executions. Missing user source means built-ins
           only; malformed source is an error. */
        attribute libraryUri : String default "checkpoints/library.json";
        attribute scriptsDirectory : String default "checkpoints/scripts";
        item definitions : ReusableCheckpointDefinition [0..*];
    }

    item def ReusableCheckpointDefinition {
        @XDesignCommon::DocSection { document = "docs/design/checkpoint-library.md"; section = "D98"; }
        doc /* Existing library entry, before per-run resolution. Syntactic
           entries own rules, declared input paths and a script or CLI executor;
           semantic entries own criteria and permitted judges. These are source
           definitions, not passed assessments. Entry versions are local authoring
           counters and are not frozen into today's run provenance. */
        attribute checkpointId : String;
        attribute title : String;
        attribute description : String;
        attribute kind : CheckKind;
        attribute entryVersion : Integer;
        item rules : CheckerRule [0..*];
        attribute inputs : String [0..*];
        attribute executorMode : String [0..1];
        attribute commandUri : String [0..1];
        attribute checkerId : String [0..1];
        attribute cli : String [0..1];
        attribute model : String [0..1];
        attribute criteria : String [0..*];
        attribute agentMayJudge : Boolean default false;
        attribute humanMayJudge : Boolean default false;
    }

    part def OperatorWebProgram {
        @XDesignCommon::DocSection { document = "docs/design/web-operator.md"; section = "D91"; }
        doc /* Implemented local operator boundary: embedded assets, store-derived
           views, checkpoint authoring and stage CLI subprocesses. It delegates
           progression to the existing flat-flow runtime and does not implement
           the generic Controller. Saving a wish does not start stage work. */
        ref part project : ProjectCheckContext;
        ref part cli : XDesignWorkflow::StageCli;
    }

    part def Checkpoint {
        @XDesignCommon::DocSection { document = "docs/design/adapters.md"; section = "F02"; }
        doc /* Reusable gate definition, independent of a particular wish.
           Criteria require evidence-backed checks or engineering judgments;
           assessments of these criteria belong to the wish's stage record.
           kind separates criteria a fixed check can establish (syntactic) from
           criteria that need judgment (semantic). A syntactic checkpoint names
           the checkers that fix its check, and the script judges it, so it sets
           no judge flag. A semantic checkpoint names no checker, because no code
           can establish it, and states whether an agent or a human may judge it.
           Judgment permission is one flag per authority rather than a list of
           authorities: the compiler's export keeps only the first element of a
           multi-valued value, and a script reading that export must still see
           the whole permission. */

        ref part sourceStage : Stage;
        attribute kind : CheckKind;
        attribute criteria : String [1..*];
        ref item checkers : Checker [0..*];
        attribute agentMayJudge : Boolean default false;
        attribute humanMayJudge : Boolean default false;

        assert constraint syntacticCriteriaAreFixedByCheckers {
            kind != CheckKind::syntactic or
            (size(checkers) > 0 and not agentMayJudge and not humanMayJudge)
        }

        assert constraint semanticCriteriaNeedJudgment {
            kind != CheckKind::semantic or
            (size(checkers) == 0 and (agentMayJudge or humanMayJudge))
        }
    }

    item def Checker {
        @XDesignCommon::DocSection { document = "docs/design/adapters.md"; section = "F02"; }
        doc /* A deterministic check fixed as code or a script, so the flow runs it
           unchanged instead of judging afresh at each assessment. A checker
           declares which kinds of conformance it establishes over the artifacts
           of the stage it guards; the check itself lives at commandUri. A designed
           flow's checkers also declare rules: the exact conditions whose
           conjunction determines the check's outcome, which the command applies
           and reports verdict by verdict. */

        attribute checkerId : String;
        attribute commandUri : String;
        attribute establishes : ConformanceKind [1..*];
        ref item rules : CheckerRule [0..*];
    }

    item def CheckerRule {
        @XDesignCommon::DocSection { document = "docs/design/flows/simple-flow/definition.md"; section = "D06"; }
        doc /* One logical rule a checker applies: the exact condition it decides and
           what it reads to decide it. The checker's outcome is the conjunction of
           its rules' verdicts — it holds only when every rule holds — and its
           report names each rule with its verdict, so a reader sees which
           condition made the difference. The rule ids the model declares and the
           rule ids the command reports are one list, kept together by the
           design's own check. */

        attribute ruleId : String;
        attribute condition : String;
        attribute reads : String [1..*];
    }

    part def Flow {
        @XDesignCommon::DocSection { document = "docs/design/flow.md"; section = "F01"; }
        doc /* Owns the stages and checkpoints of an overall workflow.
           Specialize this definition to supply different workflows. */

        part stages : Stage [1..*];
        part checkpoints : Checkpoint [1..*];
        item checkers : Checker [0..*];
        ref part initialStage : Stage subsets stages;

        assert constraint stagesUseOwnedElements {
            stages->forAll { in stage : Stage;
                stages->includes(stage.nextStages) and
                stages->includes(stage.verifiesStage) and
                checkpoints->includes(stage.exitCheckpoints) and
                (not stage.completesFlow or
                    (size(stage.nextStages) == 0 and size(stage.exitCheckpoints) > 0)) and
                stage.exitCheckpoints->forAll { in checkpoint : Checkpoint;
                    checkpoint.sourceStage == stage and
                    checkers->includes(checkpoint.checkers)
                }
            }
        }
    }

    part def VModel :> Flow {
        @XDesignCommon::DocSection { document = "docs/design/flow.md"; section = "F03"; }
        doc /* Default V-model: decompose needs into designs, implement, then
           verify components, integration and the system, validate against
           stakeholder needs, and deliver with accepted evidence. */

        ref part redefines initialStage = wishIntake;

        part wishIntake : Stage subsets stages {
            ref part redefines nextStages = stakeholderNeeds;
            ref part redefines exitCheckpoints = intakeAccepted;
        }
        part stakeholderNeeds : Stage subsets stages {
            ref part redefines nextStages = systemRequirements;
            ref part redefines exitCheckpoints = needsAccepted;
        }
        part systemRequirements : Stage subsets stages {
            ref part redefines nextStages = systemArchitecture;
            ref part redefines exitCheckpoints = requirementsAccepted;
        }
        part systemArchitecture : Stage subsets stages {
            ref part redefines nextStages = detailedDesign;
            ref part redefines exitCheckpoints = architectureAccepted;
        }
        part detailedDesign : Stage subsets stages {
            ref part redefines nextStages = implementation;
            ref part redefines exitCheckpoints = designAccepted;
        }
        part implementation : Stage subsets stages {
            ref part redefines nextStages = componentVerification;
            ref part redefines exitCheckpoints = implementationReady;
        }
        part componentVerification : Stage subsets stages {
            ref part redefines nextStages = integrationVerification;
            ref part redefines exitCheckpoints = componentsVerified;
            ref part redefines verifiesStage = detailedDesign;
        }
        part integrationVerification : Stage subsets stages {
            ref part redefines nextStages = systemVerification;
            ref part redefines exitCheckpoints = integrationVerified;
            ref part redefines verifiesStage = systemArchitecture;
        }
        part systemVerification : Stage subsets stages {
            ref part redefines nextStages = acceptanceValidation;
            ref part redefines exitCheckpoints = systemVerified;
            ref part redefines verifiesStage = systemRequirements;
        }
        part acceptanceValidation : Stage subsets stages {
            ref part redefines nextStages = delivery;
            ref part redefines exitCheckpoints = wishValidated;
            ref part redefines verifiesStage = stakeholderNeeds;
        }
        part delivery : Stage subsets stages {
            ref part redefines nextStages = ();
            ref part redefines exitCheckpoints = deliveryAccepted;
            attribute redefines completesFlow = true;
        }

        item requirementTraceCheck : Checker subsets checkers {
            doc /* Confirms that every requirement and its verification criterion
               links back to the wish, and that declared names match the model. */
            attribute redefines checkerId = "requirement-trace";
            attribute redefines commandUri = "tools/checks/requirement_trace.py";
            attribute redefines establishes = (
                ConformanceKind::traceConformance,
                ConformanceKind::identifierConformance);
        }
        item implementationArtifactCheck : Checker subsets checkers {
            doc /* Confirms that the stage produced code and build evidence, and that
               every modeled action and state has an implementation. */
            attribute redefines checkerId = "implementation-artifacts";
            attribute redefines commandUri = "tools/checks/implementation_artifacts.py";
            attribute redefines establishes = (
                ConformanceKind::artifactPresence,
                ConformanceKind::definitionCoverage);
        }
        item deliveryRecordCheck : Checker subsets checkers {
            doc /* Confirms that the stage recorded its artifacts, the traceability
               model reference and the acceptance evidence. */
            attribute redefines checkerId = "delivery-record";
            attribute redefines commandUri = "tools/checks/delivery_record.py";
            attribute redefines establishes = (
                ConformanceKind::artifactPresence,
                ConformanceKind::traceConformance);
        }

        part intakeAccepted : Checkpoint subsets checkpoints {
            ref part redefines sourceStage = wishIntake;
            attribute redefines kind = CheckKind::semantic;
            attribute redefines criteria = "The wish, scope and intended outcome are recorded.";
            attribute redefines agentMayJudge = true;
            attribute redefines humanMayJudge = true;
        }
        part needsAccepted : Checkpoint subsets checkpoints {
            ref part redefines sourceStage = stakeholderNeeds;
            attribute redefines kind = CheckKind::semantic;
            attribute redefines criteria = "Stakeholder needs and acceptance criteria are agreed.";
            attribute redefines humanMayJudge = true;
        }
        part requirementsAccepted : Checkpoint subsets checkpoints {
            ref part redefines sourceStage = systemRequirements;
            attribute redefines kind = CheckKind::syntactic;
            attribute redefines criteria = "Requirements and their verification criteria trace to the wish.";
            ref item redefines checkers = requirementTraceCheck;
        }
        part architectureAccepted : Checkpoint subsets checkpoints {
            ref part redefines sourceStage = systemArchitecture;
            attribute redefines kind = CheckKind::semantic;
            attribute redefines criteria = "System parts, relationships and interfaces address the requirements.";
            attribute redefines agentMayJudge = true;
            attribute redefines humanMayJudge = true;
        }
        part designAccepted : Checkpoint subsets checkpoints {
            ref part redefines sourceStage = detailedDesign;
            attribute redefines kind = CheckKind::semantic;
            attribute redefines criteria = "The detailed design supports implementation and component verification.";
            attribute redefines agentMayJudge = true;
            attribute redefines humanMayJudge = true;
        }
        part implementationReady : Checkpoint subsets checkpoints {
            ref part redefines sourceStage = implementation;
            attribute redefines kind = CheckKind::syntactic;
            attribute redefines criteria = "Implementation artifacts and build evidence are available.";
            ref item redefines checkers = implementationArtifactCheck;
        }
        part componentsVerified : Checkpoint subsets checkpoints {
            ref part redefines sourceStage = componentVerification;
            attribute redefines kind = CheckKind::semantic;
            attribute redefines criteria = "Component verification provides evidence against the detailed design.";
            attribute redefines agentMayJudge = true;
            attribute redefines humanMayJudge = true;
        }
        part integrationVerified : Checkpoint subsets checkpoints {
            ref part redefines sourceStage = integrationVerification;
            attribute redefines kind = CheckKind::semantic;
            attribute redefines criteria = "Integration verification provides evidence for architectural interfaces.";
            attribute redefines agentMayJudge = true;
            attribute redefines humanMayJudge = true;
        }
        part systemVerified : Checkpoint subsets checkpoints {
            ref part redefines sourceStage = systemVerification;
            attribute redefines kind = CheckKind::semantic;
            attribute redefines criteria = "System verification provides evidence against the system requirements.";
            attribute redefines agentMayJudge = true;
            attribute redefines humanMayJudge = true;
        }
        part wishValidated : Checkpoint subsets checkpoints {
            ref part redefines sourceStage = acceptanceValidation;
            attribute redefines kind = CheckKind::semantic;
            attribute redefines criteria = "Validation demonstrates the intended stakeholder outcome.";
            attribute redefines humanMayJudge = true;
        }
        part deliveryAccepted : Checkpoint subsets checkpoints {
            ref part redefines sourceStage = delivery;
            attribute redefines kind = CheckKind::syntactic;
            attribute redefines criteria = "Delivery artifacts, traceability and acceptance evidence are recorded.";
            ref item redefines checkers = deliveryRecordCheck;
        }
    }

    part def SimpleFlow :> Flow {
        @XDesignCommon::DocSection { document = "docs/design/flow.md"; section = "F03"; }
        doc /* Requirements, design, implementation and verification in sequence.
           Only verification has an exit checkpoint; passing it completes this flow. */

        part redefines stages : Stage [4];
        part redefines checkpoints : Checkpoint [1];
        ref part redefines initialStage = requirements;

        part requirements : Stage subsets stages {
            ref part redefines nextStages = design;
            ref part redefines exitCheckpoints = ();
        }
        part design : Stage subsets stages {
            ref part redefines nextStages = implement;
            ref part redefines exitCheckpoints = ();
        }
        part implement : Stage subsets stages {
            ref part redefines nextStages = 'verify';
            ref part redefines exitCheckpoints = ();
        }
        part 'verify' : Stage subsets stages {
            ref part redefines nextStages = ();
            ref part redefines exitCheckpoints = verificationAccepted;
            ref part redefines verifiesStage = requirements;
            attribute redefines completesFlow = true;
        }

        part verificationAccepted : Checkpoint subsets checkpoints {
            ref part redefines sourceStage = 'verify';
            attribute redefines kind = CheckKind::semantic;
            attribute redefines criteria = "Verification evidence demonstrates that the implementation satisfies the requirements.";
            attribute redefines agentMayJudge = true;
            attribute redefines humanMayJudge = true;
        }
    }

    part def WishVerifyFlow :> Flow {
        @XDesignCommon::DocSection { document = "docs/design/flow.md"; section = "F03"; }
        doc /* A two-stage flow for work whose outcome is judged rather than checked:
           the wish stage records the request and produces the work that answers it,
           and verification judges that outcome semantically. Nothing is derived that
           a fixed check could establish, so the flow carries no requirement, design
           or implementation stage. */

        part redefines stages : Stage [2];
        part redefines checkpoints : Checkpoint [1];
        ref part redefines initialStage = wish;

        part wish : Stage subsets stages {
            ref part redefines nextStages = 'verify';
            ref part redefines exitCheckpoints = ();
        }
        part 'verify' : Stage subsets stages {
            ref part redefines nextStages = ();
            ref part redefines exitCheckpoints = outcomeAccepted;
            ref part redefines verifiesStage = wish;
            attribute redefines completesFlow = true;
        }

        part outcomeAccepted : Checkpoint subsets checkpoints {
            ref part redefines sourceStage = 'verify';
            attribute redefines kind = CheckKind::semantic;
            attribute redefines criteria = (
                "The delivered artifacts let the requester see the outcome from them, without reading the model or re-asking.",
                "Diagrams carry the explanation where a diagram is clearer than prose, no statement goes beyond what the evidence supports, and nothing is added that the requester did not ask for.");
            attribute redefines agentMayJudge = true;
            attribute redefines humanMayJudge = true;
        }
    }

    part def DesignFlow :> WishVerifyFlow {
        @XDesignCommon::DocSection { document = "docs/design/flow.md"; section = "F03"; }
        doc /* A two-stage flow for design work whose outcome is a SysML deliverable plus
           its documentation. The wish stage records the request and produces the
           deliverable; verification is two checks: a syntactic check that the delivered
           SysML parses and checks with the pinned compiler, and the inherited semantic
           judgment of what the design means. There is no implementation stage:
           implementation starts only after a language is selected. The syntactic
           checkpoint names the compiler as its checker; it declares no rules of its own
           because the compiler supplies them. */

        part redefines checkpoints : Checkpoint [2];

        part redefines 'verify' {
            ref part redefines exitCheckpoints = (sysmlDeliverableAccepted, outcomeAccepted);
        }

        item sysmlCompilerCheck : Checker subsets checkers {
            @XDesignCommon::DocSection { document = "docs/design/flow.md"; section = "F03"; }
            doc /* The pinned SysML compiler as the fixed syntactic check: it parses the
               delivered model and applies the specification's rules. The compiler's own
               rules are its report; no separate rule list is declared here. */
            attribute redefines checkerId = "sysml-compiler";
            attribute redefines commandUri = "tools/sysml_web/bin/sysml";
            attribute redefines establishes = (ConformanceKind::identifierConformance);
        }

        part sysmlDeliverableAccepted : Checkpoint subsets checkpoints {
            @XDesignCommon::DocSection { document = "docs/design/flow.md"; section = "F03"; }
            doc /* The syntactic checkpoint of a design delivery: the delivered SysML
               parses and passes the pinned compiler's check. */
            ref part redefines sourceStage = 'verify';
            attribute redefines kind = CheckKind::syntactic;
            attribute redefines criteria = "The delivered SysML parses and passes the pinned compiler's check.";
            ref item redefines checkers = sysmlCompilerCheck;
        }
    }

    item def ActivityRecord {
        @XDesignCommon::DocSection { document = "docs/design/evidence-and-revisions.md"; section = "E02"; }
        doc /* Records work, artifact changes, checks, judgments and controller
           decisions in execution order. Time is recorded in ISO 8601 format.
           Artifact references link to the independent model referenced by Trace. */

        attribute description : String;
        attribute performedBy : String;
        attribute occurredAt : String;
        attribute artifactUris : String [0..*];
    }

    item def CheckpointAssessment {
        @XDesignCommon::DiagramPresentation { label = "Assessment"; summary = "Verdict, evidence and judged revision"; }
        @XDesignCommon::DocSection { document = "docs/design/evidence-and-revisions.md"; section = "E04"; }
        doc /* Preserved evaluation of a checkpoint for one stage visit and
           artifact revision. A passed result requires supporting evidence;
           unevaluated criteria provide no authorization to advance. */

        ref part checkpoint : Checkpoint;
        attribute evaluatedRevision : Integer;
        attribute outcome : CheckpointOutcome default CheckpointOutcome::unevaluated;
        attribute judgedBy : JudgeAuthority;
        attribute assessor : String;
        attribute rationale : String;
        attribute evidenceUris : String [0..*];
        ref item activities : ActivityRecord [0..*];

        assert constraint passedAssessmentHasEvidence {
            outcome != CheckpointOutcome::passed or size(evidenceUris) > 0
        }

        assert constraint judgmentMatchesCheckpointKind {
            (checkpoint.kind != CheckKind::syntactic or judgedBy == JudgeAuthority::script) and
            (checkpoint.kind != CheckKind::semantic or judgedBy != JudgeAuthority::script)
        }
    }

    part def StageRecord {
        @XDesignCommon::DiagramPresentation { label = "Stage visit"; summary = "Artifacts and assessments at one revision"; }
        @XDesignCommon::DocSection { document = "docs/design/evidence-and-revisions.md"; section = "E03"; }
        doc /* One planned or actual visit to a stage. Initial records cover
           every stage in the assigned flow, including pending stages. Rework
           appends a new visit; earlier visits and their evidence are retained.
           currentAssessments selects the latest assessment for each checkpoint.
           Artifact changes increment artifactRevision and invalidate old passes.
           A subflow execution is created when entering a stage with a subflow. */

        ref part stage : Stage;
        attribute visitIndex : Integer default 0;
        attribute status : StageStatus default := StageStatus::pending;
        attribute artifactRevision : Integer default := 0;
        part subFlowExecution : FlowExecution [0..1];
        item activities : ActivityRecord [0..*] ordered;
        item assessments : CheckpointAssessment [0..*] ordered;
        ref item currentAssessments : CheckpointAssessment [0..*] subsets assessments;

        assert constraint selectsLatestAssessments {
            currentAssessments->forAll { in candidate : CheckpointAssessment;
                stage.exitCheckpoints->includes(candidate.checkpoint) and
                candidate == (assessments->select { in historical : CheckpointAssessment;
                    historical.checkpoint == candidate.checkpoint
                })->last()
            }
        }

        attribute checkpointsSatisfied : Boolean =
            stage.exitCheckpoints->forAll { in checkpoint : Checkpoint;
                currentAssessments->exists { in assessment : CheckpointAssessment;
                    assessment.checkpoint == checkpoint and
                    assessment.outcome == CheckpointOutcome::passed and
                    assessment.evaluatedRevision == artifactRevision
                }
            };

        assert constraint subFlowExecutionMatchesStage {
            subFlowExecution->forAll { in execution : FlowExecution;
                execution.assignedFlow == stage.subFlow
            }
        }

        attribute subFlowSatisfied : Boolean =
            size(stage.subFlow) == 0 or
            subFlowExecution->exists { in execution : FlowExecution;
                execution.assignedFlow == stage.subFlow and
                execution.status == FlowStatus::completed
            };

        attribute completionAuthorized : Boolean =
            subFlowSatisfied and checkpointsSatisfied;

        assert constraint completedStageIsAuthorized {
            status != StageStatus::completed or completionAuthorized
        }
    }

    part def FlowExecution {
        @XDesignCommon::DiagramPresentation { label = "Run record"; summary = "Owns the history of stage visits"; }
        @XDesignCommon::DocSection { document = "docs/design/evidence-and-revisions.md"; section = "E02"; }
        doc /* Execution history of one flow, either the root workflow for a wish
           or a subflow within a stage visit. Definitions are shared; each execution
           owns its records. Completing a subflow resumes its parent stage. */

        ref part assignedFlow : Flow;
        attribute status : FlowStatus default := FlowStatus::pending;
        part stageRecords : StageRecord [1..*] ordered;
        ref part currentRecord : StageRecord [0..1] subsets stageRecords;

        assert constraint recordsEveryFlowStage {
            assignedFlow.stages->forAll { in requiredStage : Stage;
                stageRecords->exists { in record : StageRecord;
                    record.stage == requiredStage
                }
            }
        }

        assert constraint recordsUseAssignedFlow {
            stageRecords->forAll { in record : StageRecord;
                assignedFlow.stages->includes(record.stage)
            }
        }

        assert constraint completedFlowHasAuthorizedTerminalStage {
            status != FlowStatus::completed or
            currentRecord->exists { in record : StageRecord;
                record.stage.completesFlow and
                record.status == StageStatus::completed and
                record.completionAuthorized
            }
        }
    }

    item def Wish {
        @XDesignCommon::DocSection { document = "docs/interfaces/record-format.md"; section = "E01"; }
        doc /* A user's requested outcome. Workflow state and evidence belong
           to the separate Trace that references this wish. */

        attribute wishId : String;
        attribute description : String;
    }

    part def Trace {
        @XDesignCommon::DocSection { document = "docs/interfaces/record-format.md"; section = "E01"; }
        doc /* Tracks one wish through its root workflow and nested executions.
           References an independent SysML traceability model linking the wish to
           project requirements, designs, implementations and verification results.
           Only completion of the root execution delivers the wish. */

        ref item wish : Wish;
        attribute status : WishStatus default := WishStatus::received;
        attribute traceabilityModelUri : String;
        part execution : FlowExecution;

        event occurrence delivered;

        assert constraint deliveredWishHasCompletedRootFlow {
            status != WishStatus::delivered or
            execution.status == FlowStatus::completed
        }
    }

    part def Controller {
        @XDesignCommon::DiagramPresentation { label = "Controller"; summary = "Records work and authorizes progression"; }
        @XDesignCommon::DocSection { document = "docs/design/lifecycle.md"; section = "F04"; }
        doc /* Registers wishes and traces, follows root and nested workflows,
           evaluates checkpoints and decides the next step. Incomplete subflows
           or failed, missing or stale assessments prevent stage completion. */

        ref part availableFlows : Flow [1..*];
        ref part defaultFlow : Flow subsets availableFlows;
        ref item wishes : Wish [0..*];
        ref part traces : Trace [0..*];
        ref part store : ProjectStore;

        event occurrence entering;
        event occurrence assessing;
        event occurrence delivering;

        perform action receiveWish {
            doc /* Register the wish and a separate trace with an independent
               model reference. Assign selectedFlow to the trace's root execution,
               create pending records for its stages, enter its initialStage,
               activate the trace and execution, and record intake and selection.
               The directed features are inputs and outputs of this behavior. */

            in attribute description : String;
            in part selectedFlow : Flow default defaultFlow;
            out item registeredWish : Wish;
            out part registeredTrace : Trace;

            assert constraint selectedFlowIsAvailable {
                availableFlows->includes(selectedFlow)
            }
        }

        perform action recordActivity {
            doc /* Follow the trace's active execution path to the deepest current
               visit and append the activity there. If its artifacts change,
               increment that visit's revision before further assessment. */

            in part trace : Trace;
            in item activity : ActivityRecord;
        }

        perform action progressWish {
            in part trace : Trace;

            action selectCurrentExecution {
                doc /* Start at the root execution. When a current stage has a
                   subflow, initialize its execution and pending records if needed,
                   enter its initialStage, and descend while it is incomplete.
                   Once a child execution completes, resume the parent visit.
                   Return the execution and current visit to advance. */

                in part rootExecution : FlowExecution = trace.execution;
                out part execution : FlowExecution;
                out part currentRecord : StageRecord;

                assert constraint usesCurrentVisit {
                    currentRecord == execution.currentRecord and
                    execution.assignedFlow.stages->includes(currentRecord.stage)
                }
            }

            then action evaluateCheckpoints {
                doc /* Evaluate each exit checkpoint against the current
                   artifacts and evidence, append assessments to currentRecord,
                   and select only the latest assessment for each checkpoint.
                   Preserve diagnostics and unevaluated criteria in the records. */

                in part record : StageRecord = selectCurrentExecution.currentRecord;
            }

            then if selectCurrentExecution.currentRecord.completionAuthorized {
                action decideNextStep {
                    doc /* Select an allowed successor from stage.nextStages
                       in the assigned flow. A terminal stage selects no next
                       stage. Record the selected step and decision rationale. */

                    in part record : StageRecord = selectCurrentExecution.currentRecord;
                    out part nextStage : Stage [0..1];
                }

                then action updateExecutionState {
                    doc /* Complete the current visit. A terminal stage completes
                       this execution and remains its current record. Otherwise
                       enter the selected successor, keep this execution active,
                       and activate its pending
                       record or append a new visit for rework. Record the old
                       and new states with the checkpoint evidence. A completed
                       child execution allows its parent stage to resume. */

                    in part updatedExecution : FlowExecution = selectCurrentExecution.execution;
                    in part completedRecord : StageRecord = selectCurrentExecution.currentRecord;
                    in part nextStage : Stage [0..1] = decideNextStep.nextStage;

                    assert constraint advancementAuthorized {
                        completedRecord.completionAuthorized and
                        ((completedRecord.stage.completesFlow and nextStage == null) or
                            (not completedRecord.stage.completesFlow and
                             completedRecord.stage.nextStages->includes(nextStage) and
                             updatedExecution.assignedFlow.stages->includes(nextStage) and
                             nextStage != null))
                    }
                }
            } else {
                action retainCurrentStage {
                    doc /* Keep the current visit selected, mark the execution
                       and visit blocked, and record incomplete subflows or unmet
                       checkpoints and the work needed before reevaluation.
                       Preserve all prior activities. */

                    in part blockedExecution : FlowExecution = selectCurrentExecution.execution;
                    in part blockedRecord : StageRecord = selectCurrentExecution.currentRecord;
                }
            }

            then action updateTraceStatus {
                doc /* Set the trace delivered only when its root execution has
                   completed. Otherwise mark it blocked if an active visit is
                   blocked, or active while workflow work remains. */

                in part updatedTrace : Trace = trace;
            }
        }
    }

    part def XDesign {
        @XDesignCommon::DocSection { document = "docs/architecture/system-composition.md"; section = "B01"; }
        doc /* Owns the shared flow origin and a portfolio of concrete flow usages.
           The selected flow is the default; the controller may register a wish on
           any owned portfolio member. Each wish has one independent trace;
           nested executions stay in it. */

        abstract part 'flow' : Flow;
        part availableFlows : Flow [1..*];
        ref part selectedFlow : Flow subsets availableFlows;
        part controller : Controller;
        part store : ProjectStore;
        item wishes : Wish [0..*];
        part traces : Trace [0..*];

        bind controller.availableFlows = availableFlows;
        bind controller.defaultFlow = selectedFlow;
        bind controller.wishes = wishes;
        bind controller.traces = traces;
        bind controller.store = store;

        assert constraint everyWishHasOneTrace {
            wishes->forAll { in wish : Wish;
                size(traces->select { in trace : Trace;
                    trace.wish == wish
                }) == 1
            } and
            traces->forAll { in trace : Trace;
                wishes->includes(trace.wish)
            }
        }
    }

    part def NestedFlowExample :> SimpleFlow {
        @XDesignCommon::DocSection { document = "docs/design/lifecycle.md"; section = "F05"; }
        doc /* The design stage contains its own four-stage SimpleFlow. */

        part redefines design {
            part redefines subFlow : SimpleFlow [1];
        }
    }

    view recordStructure : StandardViewDefinitions::BrowserView {
        @XDesignCommon::DocSection { document = "docs/architecture/constraints.md"; section = "T03"; }
        doc /* The record vocabulary: each definition a wish's execution is recorded
           with, its fields and the rules that constrain how it is used. A trace
           owns one flow execution, an execution owns one stage record per stage of
           its assigned flow, and a stage visit owns its activities and checkpoint
           assessments. */

        expose Wish;
        expose Trace;
        expose FlowExecution;
        expose StageRecord;
        expose ActivityRecord;
        expose CheckpointAssessment;
    }

    part def CheckerResourceDistribution {
        @XDesignCommon::DocSection { document = "docs/design/checker-execution.md"; section = "D111"; }
        doc /* The executable embeds built-in Python sources, their helper and
           the selected host SysML parser. Project Git owns editable checkpoints
           and explicitly declared helper/data files. The project store retains
           immutable run bundles; cache only materializes them. Logical built-in
           source paths identify embedded resources and never search another project. */
    }

}
