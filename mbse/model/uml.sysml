package XDesignRustUml {
    @XDesignCommon::DiagramPresentation { label = "Rust UML design"; }
    @XDesignCommon::DocSection { document = "docs/details/simple_flow/06_rust_uml_design.md"; section = "D44"; }
    private import ScalarValues::*;
    private import XDesignCommon::*;
    private import XDesignArchitecture::*;
    private import XDesignDetailedDesign::*;
    private import StandardViewDefinitions::*;
    private import Views::*;

    doc /* The Rust software design of the whole xdesign crate, expressed as UML
       views over this SysML model: a generic core of flow-agnostic plan, record,
       ports and run-engine infrastructure, plus one designed flow module per
       realized flow — simpleFlow for the designed four-stage flow and
       wishVerify for the designed two-stage wish-verify flow. The views present
       the crate's layer and module layout, the structs, enums and traits the
       crate implements, the gate-evaluation interaction, and the run's state
       handling. The xdesign crate exists in src/, and model realization
       dependencies connect its designed elements to the architecture and
       detailed design. */

    package crate {
        @XDesignCommon::DocSection { document = "docs/details/simple_flow/06_rust_uml_design.md"; section = "D45"; }
        doc /* Module layout of the implemented xdesign crate, in two layers:
           core owns the flow-agnostic infrastructure; one module per realized
           designed flow — simpleFlow and wishVerify — builds on it. One module
           owns one concern; dependencies point toward the data definitions, so
           the run behavior never reaches back into its callers, and a flow
           module may depend on core, never the reverse. */

        package core {
            doc /* Flow-agnostic infrastructure of the xdesign crate: status and
               error vocabulary, plan data and structural validation, run records,
               the check-command and judge trait boundaries, gate evaluation, the
               generic run engine, and generic run construction. Nothing here
               knows the designed four-stage flow. */

            package status {
                doc /* The run's vocabulary: flow, stage, checkpoint, outcome,
                   judge, advance and conformance enums and the error vocabulary.
                   Depends on nothing. */
            }
            package plan {
                doc /* Flow plan data — stage, checkpoint, checker and rule
                   specifications read from a flow definition — and its structural
                   validation. Depends on status. */
            }
            package record {
                doc /* Visit, activity and assessment entries a run is made of, and
                   artifact digests. Depends on status and plan. */
            }
            package ports {
                doc /* The check-command and judge trait boundaries the run is
                   automated through, their reports, and the supplied judge.
                   Depends on status, plan and record. */
            }
            package gate {
                doc /* Single-checkpoint evaluation. Depends on status, plan,
                   record and ports. */
            }
            package run {
                doc /* The generic run engine RsFlowRun: run state, gate
                   evaluation, advancement and reopen. Depends on status, plan,
                   record, ports and gate. */
            }
            package factory {
                doc /* Generic run construction create_run: open the run, add the
                   planned visits, enter the initial stage. Depends on status,
                   plan, record, ports and run. */
            }

            dependency planUsesStatus from crate::core::plan to crate::core::status;
            dependency recordUsesStatus from crate::core::record to crate::core::status;
            dependency recordUsesPlan from crate::core::record to crate::core::plan;
            dependency portsUseStatus from crate::core::ports to crate::core::status;
            dependency portsUsePlan from crate::core::ports to crate::core::plan;
            dependency portsUseRecord from crate::core::ports to crate::core::record;
            dependency gateUsesStatus from crate::core::gate to crate::core::status;
            dependency gateUsesPlan from crate::core::gate to crate::core::plan;
            dependency gateUsesRecord from crate::core::gate to crate::core::record;
            dependency gateUsesPorts from crate::core::gate to crate::core::ports;
            dependency runUsesStatus from crate::core::run to crate::core::status;
            dependency runUsesPlan from crate::core::run to crate::core::plan;
            dependency runUsesRecord from crate::core::run to crate::core::record;
            dependency runUsesPorts from crate::core::run to crate::core::ports;
            dependency runUsesGate from crate::core::run to crate::core::gate;
            dependency factoryUsesStatus from crate::core::factory to crate::core::status;
            dependency factoryUsesPlan from crate::core::factory to crate::core::plan;
            dependency factoryUsesRecord from crate::core::factory to crate::core::record;
            dependency factoryUsesPorts from crate::core::factory to crate::core::ports;
            dependency factoryUsesRun from crate::core::factory to crate::core::run;
        }

        package simpleFlow {
            doc /* The designed four-stage flow module within xdesign: the
               designedSimpleFlow plan as data with its blueprint conformance, the
               designed-run factory, and the fixed four-checker command adapter.
               It depends on core; core never depends on it. */

            package blueprint {
                doc /* The designed four-stage plan as data and its blueprint
                   conformance: construct the modeled designedSimpleFlow and
                   reject any plan that differs from it. Depends on core::plan
                   and core::status. */
            }
            package factory {
                doc /* Designed-run construction create_designed_run: validate
                   blueprint conformance, then construct the run through the
                   generic create_run. Depends on blueprint, core::run, core::plan,
                   core::ports and core::status. */
            }
            package check {
                doc /* RsProcessCheck, the adapter bound to the four fixed checker
                   commands; its read-coverage policy encodes this flow's contract.
                   Depends on core::ports, core::plan, core::record and
                   core::status. */
            }

            dependency blueprintUsesCorePlan from crate::simpleFlow::blueprint to crate::core::plan;
            dependency blueprintUsesCoreStatus from crate::simpleFlow::blueprint to crate::core::status;
            dependency factoryUsesBlueprint from crate::simpleFlow::factory to crate::simpleFlow::blueprint;
            dependency factoryUsesCoreRun from crate::simpleFlow::factory to crate::core::run;
            dependency factoryUsesCorePlan from crate::simpleFlow::factory to crate::core::plan;
            dependency factoryUsesCorePorts from crate::simpleFlow::factory to crate::core::ports;
            dependency factoryUsesCoreStatus from crate::simpleFlow::factory to crate::core::status;
            dependency checkUsesCorePorts from crate::simpleFlow::check to crate::core::ports;
            dependency checkUsesCorePlan from crate::simpleFlow::check to crate::core::plan;
            dependency checkUsesCoreRecord from crate::simpleFlow::check to crate::core::record;
            dependency checkUsesCoreStatus from crate::simpleFlow::check to crate::core::status;
        }

        dependency simpleFlowUsesCore from crate::simpleFlow to crate::core;

        package wishVerify {
            doc /* The designed two-stage wish-verify flow module within
               xdesign: the designedWishVerifyFlow plan as data with its
               blueprint conformance, and the designed-run factory. Both gates
               are semantic, so the module declares no checker adapter; its
               never-invoked check stub lives in the factory. It depends on
               core; core never depends on it. */

            package blueprint {
                doc /* The designed two-stage plan as data and its blueprint
                   conformance: construct the modeled designedWishVerifyFlow and
                   reject any plan that differs from it. Depends on core::plan
                   and core::status. */
            }
            package factory {
                doc /* Designed-run construction create_wish_verify_run:
                   validate blueprint conformance, then construct the run through
                   the generic create_run with the unused-check stub and the
                   caller's judge. Depends on blueprint, core::run, core::plan,
                   core::ports and core::status. */
            }

            dependency wishVerifyBlueprintUsesCorePlan from crate::wishVerify::blueprint to crate::core::plan;
            dependency wishVerifyBlueprintUsesCoreStatus from crate::wishVerify::blueprint to crate::core::status;
            dependency wishVerifyFactoryUsesBlueprint from crate::wishVerify::factory to crate::wishVerify::blueprint;
            dependency wishVerifyFactoryUsesCoreRun from crate::wishVerify::factory to crate::core::run;
            dependency wishVerifyFactoryUsesCorePlan from crate::wishVerify::factory to crate::core::plan;
            dependency wishVerifyFactoryUsesCorePorts from crate::wishVerify::factory to crate::core::ports;
            dependency wishVerifyFactoryUsesCoreStatus from crate::wishVerify::factory to crate::core::status;
        }

        dependency wishVerifyUsesCore from crate::wishVerify to crate::core;
    }

    enum def RsFlowStatus {
        @XDesignCommon::DocSection { document = "docs/api/core/01_flow_status.md"; section = "U39"; }
        doc /* Rust enum mirroring the flow lifecycle vocabulary. */
        enum pending;
        enum active;
        enum blocked;
        enum completed;
    }
    dependency rsFlowStatusMirrors from RsFlowStatus to XDesignCommon::FlowStatus;

    enum def RsStageStatus {
        @XDesignCommon::DocSection { document = "docs/api/core/02_stage_status.md"; section = "U40"; }
        doc /* Rust enum mirroring the stage visit lifecycle vocabulary. */
        enum pending;
        enum active;
        enum blocked;
        enum completed;
    }
    dependency rsStageStatusMirrors from RsStageStatus to XDesignCommon::StageStatus;

    enum def RsCheckKind {
        @XDesignCommon::DocSection { document = "docs/api/core/03_check_kind.md"; section = "U41"; }
        doc /* Rust enum mirroring how a checkpoint's criteria are established. */
        enum syntactic;
        enum semantic;
    }
    dependency rsCheckKindMirrors from RsCheckKind to XDesignCommon::CheckKind;

    enum def RsCheckpointOutcome {
        @XDesignCommon::DocSection { document = "docs/api/core/04_checkpoint_outcome.md"; section = "U42"; }
        doc /* Rust enum mirroring an assessment's outcome vocabulary. */
        enum unevaluated;
        enum failed;
        enum passed;
    }
    dependency rsCheckpointOutcomeMirrors from RsCheckpointOutcome to XDesignCommon::CheckpointOutcome;

    enum def RsJudgeAuthority {
        @XDesignCommon::DocSection { document = "docs/api/core/05_judge_authority.md"; section = "U43"; }
        doc /* Rust enum mirroring who may judge a checkpoint. */
        enum script;
        enum agent;
        enum human;
    }
    dependency rsJudgeAuthorityMirrors from RsJudgeAuthority to XDesignCommon::JudgeAuthority;

    enum def RsAdvanceOutcome {
        @XDesignCommon::DocSection { document = "docs/api/core/06_advance_outcome.md"; section = "U44"; }
        doc /* What advance() decided for one authorized or blocked visit:
           enter the next stage, complete the run, or remain blocked.
           A Rust enum with no SysML counterpart; the decision is implicit in
           AdvanceSimpleFlowRun's three guarded paths. */
        enum enterNextStage;
        enum completeRun;
        enum remainBlocked;
    }
    dependency rsAdvanceOutcomeMirrors from RsAdvanceOutcome to XDesignDetailedDesign::AdvanceSimpleFlowRun;

    enum def RsConformanceKind {
        @XDesignCommon::DocSection { document = "docs/api/core/07_conformance_kind.md"; section = "U50"; }
        doc /* Rust enum mirroring the kinds of conformance a fixed checker establishes. */
        enum artifactPresence;
        enum identifierConformance;
        enum definitionCoverage;
        enum traceConformance;
    }
    dependency rsConformanceKindMirrors from RsConformanceKind to XDesignCommon::ConformanceKind;

    enum def RsPlanError {
        @XDesignCommon::DocSection { document = "docs/api/core/08_plan_error.md"; section = "U45"; }
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
        @XDesignCommon::DocSection { document = "docs/api/core/09_gate_error.md"; section = "U46"; }
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

    part def RsFlowPlan {
        @XDesignCommon::DocSection { document = "docs/api/core/10_flow_plan.md"; section = "U25"; }
        doc /* Rust struct realizing a Flow as data: stage and checkpoint
           specifications the run consults. Its validation is structural only —
           conformance to a designed flow's exact blueprint is the flow module's
           concern (for the simple flow, RsValidateDesignedPlan). */
        attribute planId : String;
        attribute planUri : String;
        attribute planSha256 : String;
        attribute initialStageId : String;
        part stages : RsStageSpec [0..*];
        part checkpoints : RsCheckpointSpec [0..*];

        perform action validate : RsValidatePlan;
    }
    dependency rsFlowPlanRealizes from RsFlowPlan to XDesignArchitecture::Flow;

    action def RsValidatePlan {
        @XDesignCommon::DocSection { document = "docs/api/core/11_flow_plan_validate.md"; section = "U26"; }
        doc /* Structural validation of any plan before a run is created: stage
           order, references, checkpoint kind, checker rules, criteria, permitted
           judges, and the frozen plan's byte binding. Blueprint conformance —
           equality with the designed four-stage blueprint — lives in
           RsValidateDesignedPlan. */
        in part plan : RsFlowPlan;
    }

    part def RsStageSpec {
        @XDesignCommon::DocSection { document = "docs/api/core/13_stage_spec.md"; section = "U33"; }
        doc /* Rust struct realizing one stage of the designed flow: its
           successors, its exit checkpoint identifiers and whether its
           completion finishes the flow. */
        attribute stageId : String;
        attribute nextStageIds : String [0..*];
        attribute exitCheckpointIds : String [0..*];
        attribute completesFlow : Boolean;
    }
    dependency rsStageSpecRealizes from RsStageSpec to XDesignArchitecture::Stage;

    part def RsCheckpointSpec {
        @XDesignCommon::DocSection { document = "docs/api/core/14_checkpoint_spec.md"; section = "U34"; }
        doc /* Rust struct realizing the designed flows' checkpoint classes and
           their adaptations — SimpleFlowCheckpoint with its two adaptations,
           and WishVerifyCheckpoint with its semantic adaptation: kind selects
           the scripted or the judged path, checker is present only for the
           syntactic kind, and the judge flags name who may judge a semantic
           gate. */
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

    part def RsCheckerSpec {
        @XDesignCommon::DocSection { document = "docs/api/core/15_checker_spec.md"; section = "U35"; }
        doc /* Rust struct realizing Checker: the command a syntactic gate runs
           and the rules its report must answer. */
        attribute checkerId : String;
        attribute commandUri : String;
        attribute establishes : RsConformanceKind [1..*];
        part rules : RsCheckerRuleSpec [1..*];
    }
    dependency rsCheckerSpecRealizes from RsCheckerSpec to XDesignArchitecture::Checker;

    part def RsCheckerRuleSpec {
        @XDesignCommon::DocSection { document = "docs/api/core/16_checker_rule_spec.md"; section = "U36"; }
        doc /* Rust struct realizing CheckerRule: one fixed rule's identity and
           its documented condition, read from the plan and reported verdict by
           verdict. */
        attribute ruleId : String;
        attribute condition : String;
        attribute reads : String [1..*];
    }
    dependency rsCheckerRuleSpecRealizes from RsCheckerRuleSpec to XDesignArchitecture::CheckerRule;

    part def RsStageVisit {
        @XDesignCommon::DocSection { document = "docs/api/core/17_stage_visit.md"; section = "U27"; }
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

    action def RsRecordActivity {
        @XDesignCommon::DocSection { document = "docs/api/core/18_record_activity.md"; section = "U28"; }
        doc /* Append activity and increment the visited artifact revision when
           the activity declares an artifact content change. */
        in part visit : RsStageVisit;
        in part activity : RsActivityEntry;
        out attribute artifactsChanged : Boolean;
    }
    dependency rsRecordActivityRealizes from RsRecordActivity to XDesignWorkflow::StageWork;

    action def RsAppendAssessment {
        @XDesignCommon::DocSection { document = "docs/api/core/19_append_assessment.md"; section = "U29"; }
        doc /* Append one revision-bound assessment while retaining history. */
        in part visit : RsStageVisit;
        in part assessment : RsAssessmentEntry;
    }
    dependency rsAppendAssessmentRealizes from RsAppendAssessment to XDesignDetailedDesign::EvaluateSimpleFlowGate;

    action def RsCompletionAuthorized {
        @XDesignCommon::DocSection { document = "docs/api/core/20_completion_authorized.md"; section = "U30"; }
        doc /* Select the latest assessment per required checkpoint and authorize
           only when each latest outcome passed at this visit's artifact revision. */
        in part visit : RsStageVisit;
        in part plan : RsFlowPlan;
        out attribute authorized : Boolean;
    }
    dependency rsCompletionAuthorizedRealizes from RsCompletionAuthorized to XDesignDetailedDesign::SimpleFlowAuthorization;

    part def RsActivityEntry {
        @XDesignCommon::DocSection { document = "docs/api/core/21_activity_entry.md"; section = "U31"; }
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
        @XDesignCommon::DocSection { document = "docs/api/core/22_assessment_entry.md"; section = "U32"; }
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

    part def RsArtifactDigest {
        @XDesignCommon::DocSection { document = "docs/api/core/23_artifact_digest.md"; section = "U51"; }
        doc /* Content-addressed identity of one artifact: its repository-relative
           URI and SHA-256 of the exact file bytes. */
        attribute uri : String;
        attribute sha256 : String;
    }

    part def RsCheckCommand {
        @XDesignCommon::DocSection { document = "docs/api/core/29_check_command.md"; section = "U21"; }
        doc /* Rust trait realizing CheckerCommand: the boundary every fixed
           check is invoked through. One implementation dispatches the exact
           checker spec; the run knows only the trait. */
        perform action runCheck {
            @XDesignCommon::DocSection { document = "docs/api/core/30_run_check.md"; section = "U22"; }
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
        @XDesignCommon::DocSection { document = "docs/api/core/31_judge.md"; section = "U23"; }
        doc /* Rust trait realizing Judge: the boundary a semantic gate is
           judged through. An agent or a human stands behind it; the run knows
           only the trait. */
        perform action review {
            @XDesignCommon::DocSection { document = "docs/api/core/32_review.md"; section = "U24"; }
            doc /* Present the criteria and the visit's artifact and evidence
               URIs; answer with the outcome and its rationale. */
            in attribute criteria : String [1..*];
            in attribute evidenceUris : String [0..*];
            out attribute outcome : RsCheckpointOutcome;
            out attribute rationale : String;
        }
        perform action authority {
            @XDesignCommon::DocSection { document = "docs/api/core/31_judge.md"; section = "U23"; }
            out attribute judgeAuthority : RsJudgeAuthority;
        }
        perform action identity {
            @XDesignCommon::DocSection { document = "docs/api/core/31_judge.md"; section = "U23"; }
            out attribute assessor : String;
        }

        event occurrence reviewRequested;
        event occurrence verdictReturned;
    }
    dependency rsJudgeRealizes from RsJudge to XDesignDetailedDesign::Judge;

    part def RsGateReport {
        @XDesignCommon::DocSection { document = "docs/api/core/33_gate_report.md"; section = "U37"; }
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

    part def RsRuleReport {
        @XDesignCommon::DocSection { document = "docs/api/core/34_rule_report.md"; section = "U38"; }
        doc /* One rule's verdict in a checker report: whether it held and the
           diagnostic when it did not. */
        attribute ruleId : String;
        attribute holds : Boolean;
        attribute diagnostic : String [0..1];
    }
    dependency rsRuleReportRealizes from RsRuleReport to XDesignDetailedDesign::CheckCommandExchange;

    action def RsEvaluateCheckpoint {
        @XDesignCommon::DocSection { document = "docs/api/core/39_evaluate_checkpoint.md"; section = "U20"; }
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

    part def RsFlowRun {
        @XDesignCommon::DocSection { document = "docs/api/core/40_flow_run.md"; section = "U17"; }
        doc /* Rust struct realizing FlowExecution: the generic run engine — the
           plan as data, visits in order, the current visit by index, and the
           trait objects it is automated through. The persistence fields latch a
           failed record write so a later advancement cannot treat it as a saved
           result. Its state handling is RsRunLifecycle. A run whose plan is the
           frozen designedSimpleFlow blueprint plays the SimpleFlowRun role; a
           run whose plan is the frozen designedWishVerifyFlow blueprint plays
           the WishVerifyFlowRun role. */
        attribute runId : String;
        attribute planId : String;
        attribute planUri : String;
        attribute planSha256 : String;
        attribute wishModelUri : String;
        attribute runRecordUri : String;
        part plan : RsFlowPlan;
        attribute status : RsFlowStatus;
        part visits : RsStageVisit [1..*];
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
    dependency rsFlowRunRealizes from RsFlowRun to XDesignArchitecture::FlowExecution;

    action def RsEvaluateGates {
        @XDesignCommon::DocSection { document = "docs/api/core/41_evaluate_gates.md"; section = "U18"; }
        doc /* Rust method realizing EvaluateSimpleFlowGate: for each exit
           checkpoint of the current visit's stage, evaluate it by kind and
           append the assessment, leaving earlier assessments in place. */
        out part assessments : RsAssessmentEntry [0..*];
    }
    dependency rsEvaluateGatesRealizes from RsEvaluateGates to XDesignDetailedDesign::EvaluateSimpleFlowGate;

    action def RsAdvance {
        @XDesignCommon::DocSection { document = "docs/api/core/42_advance.md"; section = "U19"; }
        doc /* Rust method realizing AdvanceSimpleFlowRun: count distinct
           required gates with a passed assessment at the visited revision,
           then enter the next stage's pending visit, complete the run at the
           terminal stage, or retain a blocked current visit when not authorized. */
        out attribute outcome : RsAdvanceOutcome;
    }
    dependency rsAdvanceRealizes from RsAdvance to XDesignDetailedDesign::AdvanceSimpleFlowRun;

    action def RsRunRecordActivity {
        @XDesignCommon::DocSection { document = "docs/api/core/43_run_record_activity.md"; section = "U48"; }
        doc /* Record corrective activity on the current blocked or active visit;
           a blocked run resumes on the same visit before its gates are re-evaluated. */
        in part activity : RsActivityEntry;
    }
    dependency rsRunRecordActivityRealizes from RsRunRecordActivity to XDesignArchitecture::Controller::recordActivity;

    action def RsReopenStage {
        @XDesignCommon::DocSection { document = "docs/api/core/44_reopen_stage.md"; section = "U49"; }
        doc /* Reopen a completed visit of a stage whose completesFlow flag is
           set — no stage name is hardcoded — by appending a new revision-zero
           visit while retaining the completed visit. */
        in attribute stageId : String;
        out attribute reopened : Boolean;
    }
    dependency rsReopenStageRealizes from RsReopenStage to XDesignDetailedDesign::ReopenSimpleFlowStage;

    action def RsCreateRun {
        @XDesignCommon::DocSection { document = "docs/api/core/46_create_run.md"; section = "U16"; }
        doc /* Rust fn create_run: the generic construction of a run from any
           structurally valid plan — build the run from the plan, push one
           pending visit per stage, mark the initial stage's visit active and
           the run active. It is the language-side construction of a
           FlowExecution; a flow's own factory decides which plan it hands in. */
        in part plan : RsFlowPlan;
        in attribute wishModelUri : String;
        in attribute runRecordUri : String;
        in ref part check : RsCheckCommand;
        in ref part judgment : RsJudge;
        out part run : RsFlowRun;
    }
    dependency rsCreateRunRealizes from RsCreateRun to XDesignArchitecture::FlowExecution;

    action def RsDesignedSimpleFlowPlan {
        @XDesignCommon::DocSection { document = "docs/api/simple_flow/01_designed_simple_flow_plan.md"; section = "U52"; }
        doc /* Rust fn designed_simple_flow_plan: construct the modeled
           designedSimpleFlow as plan data — the four stages in order with
           their five exit checkpoints, fixed checker identities, rule sets and
           judge flags — before freezing binds the plan's bytes. */
        out part plan : RsFlowPlan;
    }
    dependency rsDesignedSimpleFlowPlanProjects from RsDesignedSimpleFlowPlan to XDesignDetailedDesign::DesignedSimpleFlow;

    action def RsValidateDesignedPlan {
        @XDesignCommon::DocSection { document = "docs/api/simple_flow/02_validate_designed_plan.md"; section = "U69"; }
        doc /* Rust fn validate_designed_plan: blueprint conformance — reject any
           plan whose stages, checkpoints, checker identities, rule sets or plan
           identity differ from the designed four-stage blueprint. Structural
           validity is RsValidatePlan's separate concern. */
        in part plan : RsFlowPlan;
    }
    dependency rsValidateDesignedPlanConformance from RsValidateDesignedPlan to XDesignDetailedDesign::DesignedSimpleFlow;

    action def RsCreateDesignedRun {
        @XDesignCommon::DocSection { document = "docs/api/simple_flow/03_create_designed_run.md"; section = "U70"; }
        doc /* Rust fn create_designed_run realizing SimpleFlowRunFactory:
           validate the plan's blueprint conformance, then construct the run
           through the generic create_run. */
        in part plan : RsFlowPlan;
        in attribute wishModelUri : String;
        in attribute runRecordUri : String;
        in ref part check : RsCheckCommand;
        in ref part judgment : RsJudge;
        out part run : RsFlowRun;
    }
    dependency rsCreateDesignedRunRealizes from RsCreateDesignedRun to XDesignDetailedDesign::SimpleFlowRunFactory;

    part def XDesignCrate {
        @XDesignCommon::DocSection { document = "docs/details/simple_flow/06_rust_uml_design.md"; section = "D45"; }
        doc /* The implemented xdesign crate as one component: it owns the modules
           under crate and exposes the two trait boundaries the controller
           automates a run through. The ports are the design's existing
           exchanges, so the crate honors the modeled interfaces rather than
           inventing new ones. */
        port check : ~XDesignDetailedDesign::CheckCommandExchange;
        port judgment : ~XDesignDetailedDesign::JudgmentExchange;
    }

    part xdesignCrate : XDesignCrate {
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

        ref part run : RsFlowRun;
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
        ref part run : RsFlowRun;
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
        @XDesignCommon::DiagramPresentation { label = "Crate modules"; question = "How is the xdesign crate organized, and which module dependencies are allowed?"; }
        @XDesignCommon::DocSection { document = "docs/details/simple_flow/06_rust_uml_design.md"; section = "D45"; }
        @XDesignCommon::DocFigure { document = "docs/details/simple_flow/06_rust_uml_design.md"; figure = "docs/details/simple_flow/assets/06_crate_modules.svg"; }
        doc /* The crate's modules and their allowed dependency directions. */
        expose XDesignRustUml::crate;
        expose XDesignRustUml::crate::*;
        expose XDesignRustUml::crate::core::*;
        expose XDesignRustUml::crate::simpleFlow::*;
        expose XDesignRustUml::crate::wishVerify::*;
    }

    view rustCrateLayers : StandardViewDefinitions::GeneralView {
        @XDesignCommon::DiagramPresentation { label = "Crate layers"; question = "How is the xdesign crate divided into layers, and which direction may dependencies point?"; }
        @XDesignCommon::DocSection { document = "docs/api/01_struct.md"; section = "U68"; }
        @XDesignCommon::DocFigure { document = "docs/api/01_struct.md"; figure = "docs/api/assets/01_crate_layers.svg"; }
        doc /* The crate's layers — core infrastructure and one designed flow
           module per realized flow, simpleFlow and wishVerify — and the one-way
           dependencies between them. */
        expose XDesignRustUml::crate;
        expose XDesignRustUml::crate::core;
        expose XDesignRustUml::crate::simpleFlow;
        expose XDesignRustUml::crate::wishVerify;
        expose XDesignRustUml::crate::simpleFlowUsesCore;
        expose XDesignRustUml::crate::wishVerifyUsesCore;
    }

    view rustClassStructure : StandardViewDefinitions::GeneralView {
        @XDesignCommon::DiagramPresentation { label = "Rust types"; question = "Which structs, enums and traits would the crate declare, and what does each realize?"; }
        @XDesignCommon::DocSection { document = "docs/details/simple_flow/06_rust_uml_design.md"; section = "D46"; }
        @XDesignCommon::DocFigure { document = "docs/details/simple_flow/06_rust_uml_design.md"; figure = "docs/details/simple_flow/assets/06_class_structure.svg"; }
        doc /* The crate's types and their realization mapping to the SysML
           detailed design. */
        expose XDesignRustUml::RsFlowRun;
        expose XDesignRustUml::RsStageVisit;
        expose XDesignRustUml::RsActivityEntry;
        expose XDesignRustUml::RsAssessmentEntry;
        expose XDesignRustUml::RsArtifactDigest;
        expose XDesignRustUml::RsFlowPlan;
        expose XDesignRustUml::RsStageSpec;
        expose XDesignRustUml::RsCheckpointSpec;
        expose XDesignRustUml::RsCheckerSpec;
        expose XDesignRustUml::RsCheckerRuleSpec;
        expose XDesignRustUml::RsGateReport;
        expose XDesignRustUml::RsRuleReport;
        expose XDesignRustUml::RsCheckCommand;
        expose XDesignRustUml::RsJudge;
        expose XDesignRustUml::RsFlowStatus;
        expose XDesignRustUml::RsStageStatus;
        expose XDesignRustUml::RsCheckKind;
        expose XDesignRustUml::RsConformanceKind;
        expose XDesignRustUml::RsCheckpointOutcome;
        expose XDesignRustUml::RsJudgeAuthority;
        expose XDesignRustUml::RsAdvanceOutcome;
        expose XDesignRustUml::RsPlanError;
        expose XDesignRustUml::RsGateError;
    }

    view rustGateSequence : StandardViewDefinitions::SequenceView {
        @XDesignCommon::DiagramPresentation { label = "Gate evaluation"; question = "Which Rust objects exchange which calls when one gate is evaluated?"; }
        @XDesignCommon::DocSection { document = "docs/details/simple_flow/06_rust_uml_design.md"; section = "D47"; }
        @XDesignCommon::DocFigure { document = "docs/details/simple_flow/06_rust_uml_design.md"; figure = "docs/details/simple_flow/assets/06_gate_sequence.svg"; }
        doc /* The run, checkpoint spec, check command and judge during one
           gate evaluation. */
        expose XDesignRustUml::RsGateEvaluationInteraction;
    }

    view rustRunStates : StandardViewDefinitions::StateTransitionView {
        @XDesignCommon::DiagramPresentation { label = "Run states"; question = "When does the Rust run activate, block, resume, complete or reopen?"; }
        @XDesignCommon::DocSection { document = "docs/details/simple_flow/06_rust_uml_design.md"; section = "D48"; }
        @XDesignCommon::DocFigure { document = "docs/details/simple_flow/06_rust_uml_design.md"; figure = "docs/details/simple_flow/assets/06_run_states.svg"; }
        doc /* The run's states and the conditions its methods decide them by. */
        expose XDesignRustUml::RsRunLifecycle;
        expose XDesignRustUml::RsRunLifecycle::*;
        render Views::asInterconnectionDiagram;
    }

    view projectPackageIntegration : StandardViewDefinitions::GeneralView {
        @XDesignCommon::DiagramPresentation { label = "Project packages including Rust UML"; question = "How does the Rust software design depend on the other project model concerns?"; }
        @XDesignCommon::DocSection { document = "docs/arch/02_design.md"; section = "D21"; }
        @XDesignCommon::DocFigure { document = "docs/arch/02_design.md"; figure = "docs/arch/assets/02_packages.svg"; }
        doc /* The whole project model's package organization, including the
           xdesign flow composition, the xdesign crate's Rust design, and their
           dependencies on architecture and detailed design. Wish trace packages
           remain outside this boundary. */
        expose XDesignCommon;
        expose XDesignRequirements;
        expose XDesignArchitecture;
        expose XDesignWorkflow;
        expose XDesignDetailedDesign;
        expose XDesignComposition;
        expose XDesignRustUml;
        expose XDesignVerification;
        expose XDesignTraceability;
    }

    viewpoint rustModuleLayoutConcern : DocumentationViewpoint {
        @XDesignCommon::DocSection { document = "docs/details/simple_flow/06_rust_uml_design.md"; section = "D45"; }
        subject rustModuleLayout;
        attribute redefines concernText = "How is the xdesign crate organized, and which module dependencies are allowed?";
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

    action def RsWishVerifyFlowPlan {
        @XDesignCommon::DocSection { document = "docs/api/wish_verify_flow/01_wish_verify_flow_plan.md"; section = "U74"; }
        doc /* Rust fn wish_verify_flow_plan: construct the modeled
           designedWishVerifyFlow as plan data — the two stages in order with
           their two semantic exit checkpoints and their judge flags — before
           freezing binds the plan's bytes. */
        out part plan : RsFlowPlan;
    }
    dependency rsWishVerifyFlowPlanProjects from RsWishVerifyFlowPlan to XDesignDetailedDesign::DesignedWishVerifyFlow;

    action def RsValidateWishVerifyPlan {
        @XDesignCommon::DocSection { document = "docs/api/wish_verify_flow/02_validate_wish_verify_plan.md"; section = "U75"; }
        doc /* Rust fn validate_wish_verify_plan: blueprint conformance — reject
           any plan whose stages, checkpoints, kinds or judge flags differ from
           the designed two-stage blueprint, including a syntactic gate the
           design does not declare. Structural validity is RsValidatePlan's
           separate concern. */
        in part plan : RsFlowPlan;
    }
    dependency rsValidateWishVerifyPlanConformance from RsValidateWishVerifyPlan to XDesignDetailedDesign::DesignedWishVerifyFlow;

    action def RsCreateWishVerifyRun {
        @XDesignCommon::DocSection { document = "docs/api/wish_verify_flow/03_create_wish_verify_run.md"; section = "U76"; }
        doc /* Rust fn create_wish_verify_run realizing WishVerifyFlowRunFactory:
           validate the plan's blueprint conformance, then construct the run
           through the generic create_run. The designed flow declares no
           syntactic gate, so the check-command boundary the generic
           construction requires is satisfied by a stub that fails any call —
           an all-semantic plan never invokes it. The stub is a Rust-side
           addition with no matching UML class, like RsProcessCheck. */
        in part plan : RsFlowPlan;
        in attribute wishModelUri : String;
        in attribute runRecordUri : String;
        in ref part judgment : RsJudge;
        out part run : RsFlowRun;
    }
    dependency rsCreateWishVerifyRunRealizes from RsCreateWishVerifyRun to XDesignDetailedDesign::WishVerifyFlowRunFactory;

    /* Realization of the wish-verify design by the generic core elements,
       alongside the simple-flow realizations recorded above: one Rust element
       realizes both designed flows' elements where the design is shared. */
    dependency rsFlowRunRealizesWishVerifyRun from RsFlowRun to XDesignDetailedDesign::WishVerifyFlowRun;
    dependency rsCheckpointSpecRealizesWishVerify from RsCheckpointSpec to XDesignDetailedDesign::WishVerifyCheckpoint;
    dependency rsCheckpointSpecRealizesWishVerifySemantic from RsCheckpointSpec to XDesignDetailedDesign::WishVerifySemanticCheckpoint;
    dependency rsEvaluateCheckpointRealizesWishVerify from RsEvaluateCheckpoint to XDesignDetailedDesign::EvaluateWishVerifyCheckpoint;
    dependency rsEvaluateCheckpointTakesWishVerifyJudgment from RsEvaluateCheckpoint to XDesignDetailedDesign::TakeWishVerifyJudgment;
    dependency rsEvaluateGatesRealizesWishVerify from RsEvaluateGates to XDesignDetailedDesign::EvaluateWishVerifyGate;
    dependency rsAdvanceRealizesWishVerify from RsAdvance to XDesignDetailedDesign::AdvanceWishVerifyRun;
    dependency rsReopenStageRealizesWishVerify from RsReopenStage to XDesignDetailedDesign::ReopenWishVerifyStage;
    dependency rsCompletionAuthorizedRealizesWishVerify from RsCompletionAuthorized to XDesignDetailedDesign::WishVerifyAuthorization;
    dependency rsRunLifecycleMirrorsWishVerify from RsRunLifecycle to XDesignDetailedDesign::WishVerifyRunLifecycle;

    view wishVerifyRustModules : StandardViewDefinitions::GeneralView {
        @XDesignCommon::DiagramPresentation { label = "Wish-verify module"; question = "Where does the wish-verify module sit in the crate, and what may it depend on?"; }
        @XDesignCommon::DocSection { document = "docs/details/wish_verify_flow/05_rust_uml_design.md"; section = "D76"; }
        @XDesignCommon::DocFigure { document = "docs/details/wish_verify_flow/05_rust_uml_design.md"; figure = "docs/details/wish_verify_flow/assets/05_modules.svg"; }
        doc /* The wishVerify package beside simpleFlow over the shared core, and
           its allowed dependency directions. */
        expose XDesignRustUml::crate::wishVerify;
        expose XDesignRustUml::crate::wishVerify::*;
        expose XDesignRustUml::crate::core;
    }

    view wishVerifyRustTypes : StandardViewDefinitions::GeneralView {
        @XDesignCommon::DiagramPresentation { label = "Wish-verify Rust additions"; question = "Which Rust elements does the wish-verify module add, and what does each realize?"; }
        @XDesignCommon::DocSection { document = "docs/details/wish_verify_flow/05_rust_uml_design.md"; section = "D77"; }
        @XDesignCommon::DocFigure { document = "docs/details/wish_verify_flow/05_rust_uml_design.md"; figure = "docs/details/wish_verify_flow/assets/05_class_structure.svg"; }
        doc /* The module's three functions and the core types they build on,
           with the realization dependencies back to the wish-verify design. */
        expose XDesignRustUml::RsWishVerifyFlowPlan;
        expose XDesignRustUml::RsValidateWishVerifyPlan;
        expose XDesignRustUml::RsCreateWishVerifyRun;
        expose XDesignRustUml::RsFlowPlan;
        expose XDesignRustUml::RsFlowRun;
        expose XDesignRustUml::RsJudge;
    }

    viewpoint wishVerifyRustModulesConcern : DocumentationViewpoint {
        @XDesignCommon::DocSection { document = "docs/details/wish_verify_flow/05_rust_uml_design.md"; section = "D76"; }
        subject wishVerifyRustModules;
        attribute redefines concernText = "Where does the wish-verify module sit in the crate, and what may it depend on?";
        attribute redefines stakeholderRole = "crate maintainer";
    }

    viewpoint wishVerifyRustTypesConcern : DocumentationViewpoint {
        @XDesignCommon::DocSection { document = "docs/details/wish_verify_flow/05_rust_uml_design.md"; section = "D77"; }
        subject wishVerifyRustTypes;
        attribute redefines concernText = "Which Rust elements does the wish-verify module add, and what does each realize?";
        attribute redefines stakeholderRole = "implementer";
    }
}
