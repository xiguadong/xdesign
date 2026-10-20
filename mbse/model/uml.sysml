package SimpleFlowRustUml {
    @XDesignCommon::DiagramPresentation { label = "Rust UML design"; }
    @XDesignCommon::DocSection { document = "docs/details/simple_flow/06_rust_uml_design.md"; section = "D44"; }
    private import ScalarValues::*;
    private import XDesignCommon::*;
    private import XDesignArchitecture::*;
    private import XDesignDetailedDesign::*;
    private import StandardViewDefinitions::*;
    private import Views::*;

    doc /* The Rust software design of the designed simple flow, expressed as UML
       views over this SysML model: a crate and module layout, the structs, enums
       and traits implemented in the simpflow crate, the gate-evaluation interaction,
       and the run's state handling. The crate exists in src/, and model realization
       dependencies connect its designed elements to the detailed design. */

    package crate {
        @XDesignCommon::DocSection { document = "docs/details/simple_flow/06_rust_uml_design.md"; section = "D45"; }
        doc /* Module layout of the implemented simpflow crate. One module owns one
           concern; dependencies point toward the data definitions, so the run
           behavior never reaches back into its callers. */

        package status {
            doc /* The run's vocabulary: flow, stage, checkpoint, outcome and
               judge enums. Depends on nothing. */
        }
        package record {
            doc /* Visit, activity and assessment entries a run is made of.
               Depends on status and plan. */
        }
        package plan {
            doc /* Stage, checkpoint, checker and rule specifications read from
               the flow definition. Depends on status. */
        }
        package ports {
            doc /* The check-command and judge trait boundaries the run is
               automated through. Depends on status, plan and record. */
        }
        package gate {
            doc /* Gate evaluation and visit authorization. Depends on record,
               plan, ports and status. */
        }
        package run {
            doc /* The run type, its state handling and advancement.
               Depends on record, plan, gate, ports and status. */
        }
        package factory {
            doc /* Run construction: open the run, add the planned visits,
               enter the initial stage. Depends on run, plan, ports, record and status. */
        }

        dependency recordUsesStatus from crate::record to crate::status;
        dependency recordUsesPlan from crate::record to crate::plan;
        dependency planUsesStatus from crate::plan to crate::status;
        dependency portsUseStatus from crate::ports to crate::status;
        dependency portsUsePlan from crate::ports to crate::plan;
        dependency portsUseRecord from crate::ports to crate::record;
        dependency gateUsesPorts from crate::gate to crate::ports;
        dependency gateUsesRecord from crate::gate to crate::record;
        dependency gateUsesPlan from crate::gate to crate::plan;
        dependency gateUsesStatus from crate::gate to crate::status;
        dependency runUsesGate from crate::run to crate::gate;
        dependency runUsesRecord from crate::run to crate::record;
        dependency runUsesPlan from crate::run to crate::plan;
        dependency runUsesPorts from crate::run to crate::ports;
        dependency runUsesStatus from crate::run to crate::status;
        dependency factoryUsesRun from crate::factory to crate::run;
        dependency factoryUsesPlan from crate::factory to crate::plan;
        dependency factoryUsesPorts from crate::factory to crate::ports;
        dependency factoryUsesRecord from crate::factory to crate::record;
        dependency factoryUsesStatus from crate::factory to crate::status;
    }

    enum def RsFlowStatus {
        @XDesignCommon::DocSection { document = "docs/uml/simpflow/24_flow_status.md"; section = "U39"; }
        doc /* Rust enum mirroring the flow lifecycle vocabulary. */
        enum pending;
        enum active;
        enum blocked;
        enum completed;
    }
    dependency rsFlowStatusMirrors from RsFlowStatus to XDesignCommon::FlowStatus;

    enum def RsStageStatus {
        @XDesignCommon::DocSection { document = "docs/uml/simpflow/25_stage_status.md"; section = "U40"; }
        doc /* Rust enum mirroring the stage visit lifecycle vocabulary. */
        enum pending;
        enum active;
        enum blocked;
        enum completed;
    }
    dependency rsStageStatusMirrors from RsStageStatus to XDesignCommon::StageStatus;

    enum def RsCheckKind {
        @XDesignCommon::DocSection { document = "docs/uml/simpflow/26_check_kind.md"; section = "U41"; }
        doc /* Rust enum mirroring how a checkpoint's criteria are established. */
        enum syntactic;
        enum semantic;
    }
    dependency rsCheckKindMirrors from RsCheckKind to XDesignCommon::CheckKind;

    enum def RsCheckpointOutcome {
        @XDesignCommon::DocSection { document = "docs/uml/simpflow/27_checkpoint_outcome.md"; section = "U42"; }
        doc /* Rust enum mirroring an assessment's outcome vocabulary. */
        enum unevaluated;
        enum failed;
        enum passed;
    }
    dependency rsCheckpointOutcomeMirrors from RsCheckpointOutcome to XDesignCommon::CheckpointOutcome;

    enum def RsJudgeAuthority {
        @XDesignCommon::DocSection { document = "docs/uml/simpflow/28_judge_authority.md"; section = "U43"; }
        doc /* Rust enum mirroring who may judge a checkpoint. */
        enum script;
        enum agent;
        enum human;
    }
    dependency rsJudgeAuthorityMirrors from RsJudgeAuthority to XDesignCommon::JudgeAuthority;

    enum def RsAdvanceOutcome {
        @XDesignCommon::DocSection { document = "docs/uml/simpflow/29_advance_outcome.md"; section = "U44"; }
        doc /* What advance() decided for one authorized or blocked visit:
           enter the next stage, complete the run, or remain blocked.
           A Rust enum with no SysML counterpart; the decision is implicit in
           AdvanceSimpleFlowRun's three guarded paths. */
        enum enterNextStage;
        enum completeRun;
        enum remainBlocked;
    }
    dependency rsAdvanceOutcomeMirrors from RsAdvanceOutcome to XDesignDetailedDesign::AdvanceSimpleFlowRun;

    enum def RsPlanError {
        @XDesignCommon::DocSection { document = "docs/uml/simpflow/30_plan_error.md"; section = "U45"; }
        doc /* How create_run rejects an inconsistent plan before a run exists.
           A Rust enum with no SysML counterpart: the model assumes a
           well-formed flow definition; the Rust design makes plan validation
           typed. */
        enum missingInitialStage;
        enum duplicateStageId;
        enum unknownStageReference;
        enum syntacticSpecWithoutChecker;
        enum semanticSpecWithChecker;
        enum missingWishModelUri;
        enum missingRunRecordUri;
        enum missingCriteria;
        enum noPermittedJudge;
        enum invalidDesignedFlow;
        enum invalidChecker;
        enum invalidPlanFile;
    }

    enum def RsGateError {
        @XDesignCommon::DocSection { document = "docs/uml/simpflow/31_gate_error.md"; section = "U46"; }
        doc /* How gate evaluation fails without fabricating an outcome: the
           checker a syntactic spec needs is absent, the command itself failed
           to produce a report, or the judge declined to judge. A failed rule
           is not here — it is data in RsRuleReport. A Rust enum with no SysML
           counterpart; the model leaves failure channels to the exchanges. */
        enum checkerMissing;
        enum commandFailed;
        enum judgmentRefused;
        enum judgeNotPermitted;
        enum missingEvidence;
    }

    part def RsActivityEntry {
        @XDesignCommon::DocSection { document = "docs/uml/simpflow/16_activity_entry.md"; section = "U31"; }
        doc /* Rust struct realizing ActivityRecord: one recorded piece of work
           in a visit. Owned by its visit as a Vec entry. */
        attribute description : String;
        attribute performedBy : String;
        attribute occurredAt : String;
        attribute artifactUris : String [0..*];
        attribute artifactsChanged : Boolean;
        part artifactDigests : RsArtifactDigest [0..*];
        attribute removedArtifactUris : String [0..*];
    }
    dependency rsActivityEntryRealizes from RsActivityEntry to XDesignArchitecture::ActivityRecord;

    part def RsAssessmentEntry {
        @XDesignCommon::DocSection { document = "docs/uml/simpflow/17_assessment_entry.md"; section = "U32"; }
        doc /* Rust struct realizing CheckpointAssessment. The checkpoint is
           named by identifier, so an entry stays serializable; the run resolves
           it against the plan when authorization is counted. The assessment
           cites both the visit revision and exact artifact snapshot digest. */
        attribute checkpointId : String;
        attribute evaluatedRevision : Integer;
        attribute evaluatedSnapshotDigest : String;
        attribute outcome : RsCheckpointOutcome;
        attribute judgedBy : RsJudgeAuthority;
        attribute assessor : String;
        attribute rationale : String;
        attribute evidenceUris : String [0..*];
    }
    dependency rsAssessmentEntryRealizes from RsAssessmentEntry to XDesignArchitecture::CheckpointAssessment;

    part def RsStageVisit {
        @XDesignCommon::DocSection { document = "docs/uml/simpflow/12_stage_visit.md"; section = "U27"; }
        doc /* Rust struct realizing StageRecord: one visit of one stage, owning
           its activities and assessments. Earlier visits are never rewritten;
           a blocked visit is corrected in place; reopening a completed
           terminal stage appends a new visit at the next index. */
        attribute stageId : String;
        attribute visitIndex : Integer;
        attribute status : RsStageStatus;
        attribute artifactRevision : Integer;
        attribute artifactSnapshotDigest : String;
        part artifactDigests : RsArtifactDigest [0..*];
        part activities : RsActivityEntry [0..*];
        part assessments : RsAssessmentEntry [0..*];

        perform action recordActivity : RsRecordActivity;
        perform action appendAssessment : RsAppendAssessment;
        perform action completionAuthorized : RsCompletionAuthorized;
    }
    dependency rsStageVisitRealizes from RsStageVisit to XDesignArchitecture::StageRecord;

    part def RsArtifactDigest {
        @XDesignCommon::DocSection { document = "docs/uml/simpflow/35_artifact_digest.md"; section = "U51"; }
        doc /* Content-addressed identity of one artifact: its repository-relative
           URI and SHA-256 of the exact file bytes. */
        attribute uri : String;
        attribute sha256 : String;
    }

    part def RsCheckerRuleSpec {
        @XDesignCommon::DocSection { document = "docs/uml/simpflow/21_checker_rule_spec.md"; section = "U36"; }
        doc /* Rust struct realizing CheckerRule: one fixed rule's identity and
           its documented condition, read from the plan and reported verdict by
           verdict. */
        attribute ruleId : String;
        attribute condition : String;
        attribute reads : String [1..*];
    }
    dependency rsCheckerRuleSpecRealizes from RsCheckerRuleSpec to XDesignArchitecture::CheckerRule;

    enum def RsConformanceKind {
        @XDesignCommon::DocSection { document = "docs/uml/simpflow/34_conformance_kind.md"; section = "U50"; }
        doc /* Rust enum mirroring the kinds of conformance a fixed checker establishes. */
        enum artifactPresence;
        enum identifierConformance;
        enum definitionCoverage;
        enum traceConformance;
    }
    dependency rsConformanceKindMirrors from RsConformanceKind to XDesignCommon::ConformanceKind;

    part def RsCheckerSpec {
        @XDesignCommon::DocSection { document = "docs/uml/simpflow/20_checker_spec.md"; section = "U35"; }
        doc /* Rust struct realizing Checker: the command a syntactic gate runs
           and the rules its report must answer. */
        attribute checkerId : String;
        attribute commandUri : String;
        attribute establishes : RsConformanceKind [1..*];
        part rules : RsCheckerRuleSpec [1..*];
    }
    dependency rsCheckerSpecRealizes from RsCheckerSpec to XDesignArchitecture::Checker;

    part def RsCheckpointSpec {
        @XDesignCommon::DocSection { document = "docs/uml/simpflow/19_checkpoint_spec.md"; section = "U34"; }
        doc /* Rust struct realizing SimpleFlowCheckpoint and its two
           adaptations: kind selects the scripted or the judged path, checker
           is present only for the syntactic kind, and the judge flags name
           who may judge a semantic gate. */
        attribute checkpointId : String;
        attribute stageId : String;
        attribute kind : RsCheckKind;
        attribute criteria : String [1..*];
        part checker : RsCheckerSpec [0..1];
        attribute agentMayJudge : Boolean;
        attribute humanMayJudge : Boolean;

        event occurrence evaluationEntered;
        event occurrence rulesRead;
        event occurrence assessed;
    }
    dependency rsCheckpointSpecRealizes from RsCheckpointSpec to XDesignDetailedDesign::SimpleFlowCheckpoint;
    dependency rsCheckpointSpecSyntactic from RsCheckpointSpec to XDesignDetailedDesign::SimpleFlowSyntacticCheckpoint;
    dependency rsCheckpointSpecSemantic from RsCheckpointSpec to XDesignDetailedDesign::SimpleFlowSemanticCheckpoint;

    part def RsStageSpec {
        @XDesignCommon::DocSection { document = "docs/uml/simpflow/18_stage_spec.md"; section = "U33"; }
        doc /* Rust struct realizing one stage of the designed flow: its
           successors, its exit checkpoint identifiers and whether its
           completion finishes the flow. */
        attribute stageId : String;
        attribute nextStageIds : String [0..*];
        attribute exitCheckpointIds : String [0..*];
        attribute completesFlow : Boolean;
    }
    dependency rsStageSpecRealizes from RsStageSpec to XDesignArchitecture::Stage;

    part def RsFlowPlan {
        @XDesignCommon::DocSection { document = "docs/uml/simpflow/10_flow_plan.md"; section = "U25"; }
        doc /* Rust struct realizing designedSimpleFlow as data: stage and checkpoint
           specifications the run consults. Its validation enforces this design's
           exact four-stage, five-checkpoint blueprint. */
        attribute planId : String;
        attribute planUri : String;
        attribute planSha256 : String;
        attribute initialStageId : String;
        part stages : RsStageSpec [4];
        part checkpoints : RsCheckpointSpec [5];

        perform action validate : RsValidatePlan;
    }
    dependency rsFlowPlanRealizes from RsFlowPlan to XDesignDetailedDesign::designedSimpleFlow;

    part def RsRuleReport {
        @XDesignCommon::DocSection { document = "docs/uml/simpflow/23_rule_report.md"; section = "U38"; }
        doc /* One rule's verdict in a checker report: whether it held and the
           diagnostic when it did not. */
        attribute ruleId : String;
        attribute holds : Boolean;
        attribute diagnostic : String [0..1];
    }
    dependency rsRuleReportRealizes from RsRuleReport to XDesignDetailedDesign::CheckCommandExchange;

    part def RsGateReport {
        @XDesignCommon::DocSection { document = "docs/uml/simpflow/22_gate_report.md"; section = "U37"; }
        doc /* The check command's answer, realizing the output side of
           CheckCommandExchange: the overall verdict and every rule's report,
           retained as the assessment's evidence. */
        attribute holds : Boolean;
        attribute artifactRevision : Integer;
        attribute artifactSnapshotDigest : String;
        attribute reportUri : String;
        part rules : RsRuleReport [0..*];
    }
    dependency rsGateReportRealizes from RsGateReport to XDesignDetailedDesign::CheckCommandExchange;

    part def RsCheckCommand {
        @XDesignCommon::DocSection { document = "docs/uml/simpflow/06_check_command.md"; section = "U21"; }
        doc /* Rust trait realizing CheckerCommand: the boundary every fixed
           check is invoked through. One implementation dispatches the exact
           checker spec; the run knows only the trait. */
        perform action runCheck {
            @XDesignCommon::DocSection { document = "docs/uml/simpflow/07_run_check.md"; section = "U22"; }
            doc /* Invoke the selected checker with the independent wish trace,
               runtime JSON record, stage and artifact revision; return a rule
               report bound to that revision. */
            in attribute wishModelUri : String;
            in attribute runRecord : String;
            in attribute planId : String;
            in attribute planUri : String;
            in attribute planSha256 : String;
            in attribute stage : String;
            in part checker : RsCheckerSpec;
            in attribute artifactRevision : Integer;
            in attribute snapshotDigest : String;
            out part report : RsGateReport;
        }

        event occurrence invoked;
        event occurrence reported;
    }
    dependency rsCheckCommandRealizes from RsCheckCommand to XDesignDetailedDesign::CheckerCommand;

    part def RsJudge {
        @XDesignCommon::DocSection { document = "docs/uml/simpflow/08_judge.md"; section = "U23"; }
        doc /* Rust trait realizing Judge: the boundary a semantic gate is
           judged through. An agent or a human stands behind it; the run knows
           only the trait. */
        perform action review {
            @XDesignCommon::DocSection { document = "docs/uml/simpflow/09_review.md"; section = "U24"; }
            doc /* Present the criteria and the visit's artifact and evidence
               URIs; answer with the outcome and its rationale. */
            in attribute criteria : String [1..*];
            in attribute evidenceUris : String [0..*];
            out attribute outcome : RsCheckpointOutcome;
            out attribute rationale : String;
        }
        perform action authority {
            @XDesignCommon::DocSection { document = "docs/uml/simpflow/08_judge.md"; section = "U23"; }
            out attribute judgeAuthority : RsJudgeAuthority;
        }
        perform action identity {
            @XDesignCommon::DocSection { document = "docs/uml/simpflow/08_judge.md"; section = "U23"; }
            out attribute assessor : String;
        }

        event occurrence reviewRequested;
        event occurrence verdictReturned;
    }
    dependency rsJudgeRealizes from RsJudge to XDesignDetailedDesign::Judge;

    part def RsSimpleFlowRun {
        @XDesignCommon::DocSection { document = "docs/uml/simpflow/02_simple_flow_run.md"; section = "U17"; }
        doc /* Rust struct realizing SimpleFlowRun: the plan as data, visits in
           order, the current visit by index, and the trait objects it is
           automated through. The persistence fields latch a failed record
           write so a later advancement cannot treat it as a saved result.
           Its state handling is RsRunLifecycle. */
        attribute runId : String;
        attribute planId : String;
        attribute planUri : String;
        attribute planSha256 : String;
        attribute wishModelUri : String;
        attribute runRecordUri : String;
        part plan : RsFlowPlan;
        attribute status : RsFlowStatus;
        part visits : RsStageVisit [4..*];
        attribute currentVisit : Integer;
        ref part check : RsCheckCommand;
        ref part judgment : RsJudge;
        attribute persistenceError : Boolean;
        attribute lastError : String [0..1];

        perform action evaluateGates : RsEvaluateGates;
        perform action advance : RsAdvance;
        perform action recordActivity : RsRunRecordActivity;
        perform action reopenStage : RsReopenStage;

        exhibit state lifecycle : RsRunLifecycle;
        bind lifecycle.run = self;

        event occurrence evaluationRequested;
        event occurrence advanceRequested;
    }
    dependency rsSimpleFlowRunRealizes from RsSimpleFlowRun to XDesignDetailedDesign::SimpleFlowRun;

    action def RsValidatePlan {
        @XDesignCommon::DocSection { document = "docs/uml/simpflow/11_flow_plan_validate.md"; section = "U26"; }
        doc /* Validate stage order, references, checkpoint kind, checker rules,
           criteria and permitted judges before creating a run. */
        in part plan : RsFlowPlan;
    }

    action def RsRecordActivity {
        @XDesignCommon::DocSection { document = "docs/uml/simpflow/13_record_activity.md"; section = "U28"; }
        doc /* Append activity and increment the visited artifact revision when
           the activity declares an artifact content change. */
        in part visit : RsStageVisit;
        in part activity : RsActivityEntry;
        out attribute artifactsChanged : Boolean;
    }
    dependency rsRecordActivityRealizes from RsRecordActivity to XDesignWorkflow::StageWork;

    action def RsAppendAssessment {
        @XDesignCommon::DocSection { document = "docs/uml/simpflow/14_append_assessment.md"; section = "U29"; }
        doc /* Append one revision-bound assessment while retaining history. */
        in part visit : RsStageVisit;
        in part assessment : RsAssessmentEntry;
    }
    dependency rsAppendAssessmentRealizes from RsAppendAssessment to XDesignDetailedDesign::EvaluateSimpleFlowGate;

    action def RsCompletionAuthorized {
        @XDesignCommon::DocSection { document = "docs/uml/simpflow/15_completion_authorized.md"; section = "U30"; }
        doc /* Select the latest assessment per required checkpoint and authorize
           only when each latest outcome passed at this visit's artifact revision. */
        in part visit : RsStageVisit;
        in part plan : RsFlowPlan;
        out attribute authorized : Boolean;
    }
    dependency rsCompletionAuthorizedRealizes from RsCompletionAuthorized to XDesignDetailedDesign::SimpleFlowAuthorization;

    action def RsRunRecordActivity {
        @XDesignCommon::DocSection { document = "docs/uml/simpflow/32_run_record_activity.md"; section = "U48"; }
        doc /* Record corrective activity on the current blocked or active visit;
           a blocked run resumes on the same visit before its gates are re-evaluated. */
        in part activity : RsActivityEntry;
    }
    dependency rsRunRecordActivityRealizes from RsRunRecordActivity to XDesignArchitecture::Controller::recordActivity;

    action def RsReopenStage {
        @XDesignCommon::DocSection { document = "docs/uml/simpflow/33_reopen_stage.md"; section = "U49"; }
        doc /* Reopen a previously completed terminal stage by appending a new
           revision-zero visit while retaining the completed visit. */
        in attribute stageId : String;
        out attribute reopened : Boolean;
    }
    dependency rsReopenStageRealizes from RsReopenStage to XDesignDetailedDesign::ReopenSimpleFlowStage;

    action def RsCreateRun {
        @XDesignCommon::DocSection { document = "docs/uml/simpflow/01_create_run.md"; section = "U16"; }
        doc /* Rust fn realizing SimpleFlowRunFactory::createRun: build the run
           from the plan, push one pending visit per stage, mark the initial
           stage's visit active and the run active. */
        in part plan : RsFlowPlan;
        in attribute wishModelUri : String;
        in attribute runRecordUri : String;
        in ref part check : RsCheckCommand;
        in ref part judgment : RsJudge;
        out part run : RsSimpleFlowRun;
    }
    dependency rsCreateRunRealizes from RsCreateRun to XDesignDetailedDesign::SimpleFlowRunFactory;

    action def RsEvaluateGates {
        @XDesignCommon::DocSection { document = "docs/uml/simpflow/03_evaluate_gates.md"; section = "U18"; }
        doc /* Rust method realizing EvaluateSimpleFlowGate: for each exit
           checkpoint of the current visit's stage, evaluate it by kind and
           append the assessment, leaving earlier assessments in place. */
        out part assessments : RsAssessmentEntry [0..*];
    }
    dependency rsEvaluateGatesRealizes from RsEvaluateGates to XDesignDetailedDesign::EvaluateSimpleFlowGate;

    action def RsEvaluateCheckpoint {
        @XDesignCommon::DocSection { document = "docs/uml/simpflow/05_evaluate_checkpoint.md"; section = "U20"; }
        doc /* Rust fn realizing EvaluateSimpleFlowCheckpoint: a syntactic spec
           runs its checker through the check trait and holds only when every
           rule holds; a semantic spec presents its criteria through the judge
           trait and records the verdict. */
        in part checkpoint : RsCheckpointSpec;
        in part visit : RsStageVisit;
        in attribute planId : String;
        in attribute planUri : String;
        in attribute planSha256 : String;
        in attribute wishModelUri : String;
        in attribute runRecordUri : String;
        in ref part check : RsCheckCommand;
        in ref part judge : RsJudge;
        out part assessment : RsAssessmentEntry [0..1];
    }
    dependency rsEvaluateCheckpointRealizes from RsEvaluateCheckpoint to XDesignDetailedDesign::EvaluateSimpleFlowCheckpoint;

    action def RsAdvance {
        @XDesignCommon::DocSection { document = "docs/uml/simpflow/04_advance.md"; section = "U19"; }
        doc /* Rust method realizing AdvanceSimpleFlowRun: count distinct
           required gates with a passed assessment at the visited revision,
           then enter the next stage's pending visit, complete the run at the
           terminal stage, or retain a blocked current visit when not authorized. */
        out attribute outcome : RsAdvanceOutcome;
    }
    dependency rsAdvanceRealizes from RsAdvance to XDesignDetailedDesign::AdvanceSimpleFlowRun;

    part def SimpflowCrate {
        @XDesignCommon::DocSection { document = "docs/details/simple_flow/06_rust_uml_design.md"; section = "D45"; }
        doc /* The implemented simpflow crate as one component: it owns the modules
           under crate and exposes the two trait boundaries the controller
           automates a run through. The ports are the design's existing
           exchanges, so the crate honors the modeled interfaces rather than
           inventing new ones. */
        port check : ~XDesignDetailedDesign::CheckCommandExchange;
        port judgment : ~XDesignDetailedDesign::JudgmentExchange;
    }

    part simpflowCrate : SimpflowCrate {
        @XDesignCommon::DocSection { document = "docs/details/simple_flow/06_rust_uml_design.md"; section = "D45"; }
        doc /* The designed crate instance the verification case reviews;
           Cargo.toml and src/ implement its first executable slice. */
    }

    occurrence def RsGateEvaluationInteraction {
        @XDesignCommon::DocSection { document = "docs/details/simple_flow/06_rust_uml_design.md"; section = "D47"; }
        @XDesignCommon::DocFigure { document = "docs/details/simple_flow/06_rust_uml_design.md"; figure = "docs/details/simple_flow/assets/06_gate_sequence.svg"; }
        doc /* One gate evaluation in the Rust design: the run asks the
           checkpoint spec to evaluate; a syntactic spec invokes its checker
           through the check trait and a semantic spec requests a judgment
           through the judge trait; either way the run appends the assessment
           to the current visit. The kind branch is exclusive; both messages
           are drawn for one specification of the path. */

        ref part run : RsSimpleFlowRun;
        ref part checkpoint : RsCheckpointSpec;
        ref part command : RsCheckCommand;
        ref part judge : RsJudge;

        message requestEvaluation from run.evaluationRequested to checkpoint.evaluationEntered;
        message readRules from checkpoint.evaluationEntered to checkpoint.rulesRead;
        message invokeCheck from checkpoint.rulesRead to command.invoked;
        message returnReport from command.reported to checkpoint.assessed;
        message requestJudgment from checkpoint.rulesRead to judge.reviewRequested;
        message returnVerdict from judge.verdictReturned to checkpoint.assessed;
    }

    state def RsRunLifecycle {
        @XDesignCommon::DocSection { document = "docs/details/simple_flow/06_rust_uml_design.md"; section = "D48"; }
        @XDesignCommon::DocFigure { document = "docs/details/simple_flow/06_rust_uml_design.md"; figure = "docs/details/simple_flow/assets/06_run_states.svg"; }
        doc /* The run's state handling in the Rust design, mirroring
           SimpleFlowRunLifecycle: create_run activates, a gate that is not
           authorized blocks, corrective work resumes the active visit, an
           authorized nonterminal visit advances, and an authorized terminal visit
           completes. Rechecking a completed run can block it when retained evidence
           disappears and complete it again after restoration; explicit rework
           appends a new visit. */
        ref part run : RsSimpleFlowRun;
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
            if run.status == RsFlowStatus::active then active;
        transition gateBlocked first active
            if run.status == RsFlowStatus::blocked then blocked;
        transition resumeAfterCorrection first blocked
            if run.status == RsFlowStatus::active then active;
        transition advanceStage first active
            if run.status == RsFlowStatus::active then active;
        transition complete first active
            if run.status == RsFlowStatus::completed then completed;
        transition revalidateCompleted first completed
            if run.status == RsFlowStatus::completed then completed;
        transition completedEvidenceLost first completed
            if run.status == RsFlowStatus::blocked then blocked;
        transition restoredCompletion first blocked
            if run.status == RsFlowStatus::completed then completed;
        transition reopenForRework first completed
            if run.status == RsFlowStatus::active then active;
    }
    dependency rsRunLifecycleMirrors from RsRunLifecycle to XDesignDetailedDesign::SimpleFlowRunLifecycle;

    view rustModuleLayout : StandardViewDefinitions::GeneralView {
        @XDesignCommon::DiagramPresentation { label = "Crate modules"; question = "How is the simpflow crate organized, and which module dependencies are allowed?"; }
        @XDesignCommon::DocSection { document = "docs/details/simple_flow/06_rust_uml_design.md"; section = "D45"; }
        @XDesignCommon::DocFigure { document = "docs/details/simple_flow/06_rust_uml_design.md"; figure = "docs/details/simple_flow/assets/06_crate_modules.svg"; }
        doc /* The crate's modules and their allowed dependency directions. */
        expose SimpleFlowRustUml::crate;
        expose SimpleFlowRustUml::crate::*;
    }

    view rustClassStructure : StandardViewDefinitions::GeneralView {
        @XDesignCommon::DiagramPresentation { label = "Rust types"; question = "Which structs, enums and traits would the crate declare, and what does each realize?"; }
        @XDesignCommon::DocSection { document = "docs/details/simple_flow/06_rust_uml_design.md"; section = "D46"; }
        @XDesignCommon::DocFigure { document = "docs/details/simple_flow/06_rust_uml_design.md"; figure = "docs/details/simple_flow/assets/06_class_structure.svg"; }
        doc /* The crate's types and their realization mapping to the SysML
           detailed design. */
        expose SimpleFlowRustUml::RsSimpleFlowRun;
        expose SimpleFlowRustUml::RsStageVisit;
        expose SimpleFlowRustUml::RsActivityEntry;
        expose SimpleFlowRustUml::RsAssessmentEntry;
        expose SimpleFlowRustUml::RsArtifactDigest;
        expose SimpleFlowRustUml::RsFlowPlan;
        expose SimpleFlowRustUml::RsStageSpec;
        expose SimpleFlowRustUml::RsCheckpointSpec;
        expose SimpleFlowRustUml::RsCheckerSpec;
        expose SimpleFlowRustUml::RsCheckerRuleSpec;
        expose SimpleFlowRustUml::RsGateReport;
        expose SimpleFlowRustUml::RsRuleReport;
        expose SimpleFlowRustUml::RsCheckCommand;
        expose SimpleFlowRustUml::RsJudge;
        expose SimpleFlowRustUml::RsFlowStatus;
        expose SimpleFlowRustUml::RsStageStatus;
        expose SimpleFlowRustUml::RsCheckKind;
        expose SimpleFlowRustUml::RsConformanceKind;
        expose SimpleFlowRustUml::RsCheckpointOutcome;
        expose SimpleFlowRustUml::RsJudgeAuthority;
        expose SimpleFlowRustUml::RsAdvanceOutcome;
        expose SimpleFlowRustUml::RsPlanError;
        expose SimpleFlowRustUml::RsGateError;
    }

    view rustGateSequence : StandardViewDefinitions::SequenceView {
        @XDesignCommon::DiagramPresentation { label = "Gate evaluation"; question = "Which Rust objects exchange which calls when one gate is evaluated?"; }
        @XDesignCommon::DocSection { document = "docs/details/simple_flow/06_rust_uml_design.md"; section = "D47"; }
        @XDesignCommon::DocFigure { document = "docs/details/simple_flow/06_rust_uml_design.md"; figure = "docs/details/simple_flow/assets/06_gate_sequence.svg"; }
        doc /* The run, checkpoint spec, check command and judge during one
           gate evaluation. */
        expose SimpleFlowRustUml::RsGateEvaluationInteraction;
    }

    view rustRunStates : StandardViewDefinitions::StateTransitionView {
        @XDesignCommon::DiagramPresentation { label = "Run states"; question = "When does the Rust run activate, block, resume, complete or reopen?"; }
        @XDesignCommon::DocSection { document = "docs/details/simple_flow/06_rust_uml_design.md"; section = "D48"; }
        @XDesignCommon::DocFigure { document = "docs/details/simple_flow/06_rust_uml_design.md"; figure = "docs/details/simple_flow/assets/06_run_states.svg"; }
        doc /* The run's states and the conditions its methods decide them by. */
        expose SimpleFlowRustUml::RsRunLifecycle;
        expose SimpleFlowRustUml::RsRunLifecycle::*;
        render Views::asInterconnectionDiagram;
    }

    view projectPackageIntegration : StandardViewDefinitions::GeneralView {
        @XDesignCommon::DiagramPresentation { label = "Project packages including Rust UML"; question = "How does the Rust software design depend on the other project model concerns?"; }
        @XDesignCommon::DocSection { document = "docs/arch/02_design.md"; section = "D21"; }
        @XDesignCommon::DocFigure { document = "docs/arch/02_design.md"; figure = "docs/arch/assets/02_packages.svg"; }
        doc /* The whole project model's package organization, including the
           SimpleFlow Rust design and its dependencies on architecture and
           detailed design. Wish trace packages remain outside this boundary. */
        expose XDesignCommon;
        expose XDesignRequirements;
        expose XDesignArchitecture;
        expose XDesignWorkflow;
        expose XDesignDetailedDesign;
        expose SimpleFlowRustUml;
        expose XDesignVerification;
        expose XDesignTraceability;
    }

    viewpoint rustModuleLayoutConcern : DocumentationViewpoint {
        @XDesignCommon::DocSection { document = "docs/details/simple_flow/06_rust_uml_design.md"; section = "D45"; }
        subject rustModuleLayout;
        attribute redefines concernText = "How is the simpflow crate organized, and which module dependencies are allowed?";
        attribute redefines stakeholderRole = "crate maintainer";
    }

    viewpoint rustClassStructureConcern : DocumentationViewpoint {
        @XDesignCommon::DocSection { document = "docs/details/simple_flow/06_rust_uml_design.md"; section = "D46"; }
        subject rustClassStructure;
        attribute redefines concernText = "Which structs, enums and traits would the crate declare, and what does each realize?";
        attribute redefines stakeholderRole = "implementer";
    }

    viewpoint rustGateSequenceConcern : DocumentationViewpoint {
        @XDesignCommon::DocSection { document = "docs/details/simple_flow/06_rust_uml_design.md"; section = "D47"; }
        subject rustGateSequence;
        attribute redefines concernText = "Which Rust objects exchange which calls when one gate is evaluated?";
        attribute redefines stakeholderRole = "integrator";
    }

    viewpoint rustRunStatesConcern : DocumentationViewpoint {
        @XDesignCommon::DocSection { document = "docs/details/simple_flow/06_rust_uml_design.md"; section = "D48"; }
        subject rustRunStates;
        attribute redefines concernText = "When does the Rust run activate, block, resume, complete or reopen?";
        attribute redefines stakeholderRole = "maintainer";
    }
}
