package Wish0001 {
    @XDesignCommon::DocSection { document = "docs/review/01_wish_status.md"; section = "V02"; }
    private import ScalarValues::*;
    private import XDesignCommon::*;
    private import XDesignArchitecture::*;
    private import XDesignWorkflow::*;
    private import StandardViewDefinitions::*;
    private import Views::*;

    doc /* Historical illustrative trace for wish-0001. The recorded delivered
       status and passed assessment describe a proposed SimpleFlow walk-through;
       their named source, build, and check artifacts are not retained, so this
       record cannot establish actual delivery or engineering acceptance. No
       project requirements derived from this wish are retained. Definitions
       stay in mbse/model; load both directories to resolve references. */

    item wish0001 : Wish {
        attribute redefines wishId = "wish-0001";
        attribute redefines description = "Export the project model as a reviewable report.";
    }

    part trace0001 : Trace {
        doc /* Verification failed once and was retried in the same visit, so this
           trace keeps both assessments. Artifact revision 3 carries the failed
           attempt; corrective work raised it to 4, and only an assessment of
           revision 4 authorizes completion. */

        ref item redefines wish = wish0001;
        attribute redefines status = WishStatus::delivered;
        attribute redefines traceabilityModelUri = "mbse/wishes/wish-0001.sysml";

        part redefines execution : FlowExecution {
            ref part redefines assignedFlow = workflow.'flow';
            attribute redefines status = FlowStatus::completed;
            part redefines stageRecords : StageRecord [4];
            ref part redefines currentRecord = verifyRecord;

            part requirementsRecord : StageRecord subsets stageRecords {
                ref part redefines stage = workflow.'flow'.requirements;
                attribute redefines visitIndex = 0;
                attribute redefines status = StageStatus::completed;
                attribute redefines artifactRevision = 1;

                item requirementsActivity : ActivityRecord subsets activities {
                    attribute redefines description = "Outlined four candidate report checks; no derived project requirements are retained.";
                    attribute redefines performedBy = "analyst";
                    attribute redefines occurredAt = "2026-10-14T08:10:00Z";
                    attribute redefines artifactUris = ("mbse/model/requirements.sysml");
                }
            }

            part designRecord : StageRecord subsets stageRecords {
                ref part redefines stage = workflow.'flow'.design;
                attribute redefines visitIndex = 0;
                attribute redefines status = StageStatus::completed;
                attribute redefines artifactRevision = 1;

                item designActivity : ActivityRecord subsets activities {
                    attribute redefines description = "Described the parts and interfaces that meet the requirements.";
                    attribute redefines performedBy = "architect";
                    attribute redefines occurredAt = "2026-10-14T08:40:00Z";
                    attribute redefines artifactUris = ("mbse/model/architecture.sysml");
                }
            }

            part implementRecord : StageRecord subsets stageRecords {
                ref part redefines stage = workflow.'flow'.implement;
                attribute redefines visitIndex = 0;
                attribute redefines status = StageStatus::completed;
                attribute redefines artifactRevision = 3;

                item implementActivity : ActivityRecord subsets activities {
                    attribute redefines description = "Built the report generator and its build evidence.";
                    attribute redefines performedBy = "implementer";
                    attribute redefines occurredAt = "2026-10-14T09:20:00Z";
                    attribute redefines artifactUris = ("src/report.rs", "cache/build/report.xml");
                }
            }

            part verifyRecord : StageRecord subsets stageRecords {
                ref part redefines stage = workflow.'flow'.'verify';
                attribute redefines visitIndex = 0;
                attribute redefines status = StageStatus::completed;
                attribute redefines artifactRevision = 4;

                item firstAttemptActivity : ActivityRecord subsets activities {
                    attribute redefines description = "Ran the four requirement checks against revision 3.";
                    attribute redefines performedBy = "verifier";
                    attribute redefines occurredAt = "2026-10-14T09:30:00Z";
                    attribute redefines artifactUris = ("cache/evidence/req-check-rev3.log");
                }
                item correctiveActivity : ActivityRecord subsets activities {
                    attribute redefines description = "Fixed the report layout that failed the usability requirement.";
                    attribute redefines performedBy = "implementer";
                    attribute redefines occurredAt = "2026-10-14T10:05:00Z";
                    attribute redefines artifactUris = ("src/report.rs");
                }
                item secondAttemptActivity : ActivityRecord subsets activities {
                    attribute redefines description = "Re-ran the four requirement checks against revision 4.";
                    attribute redefines performedBy = "verifier";
                    attribute redefines occurredAt = "2026-10-14T10:20:00Z";
                    attribute redefines artifactUris = ("cache/evidence/req-check-rev4.log");
                }

                item redefines assessments : CheckpointAssessment [2];
                ref item redefines currentAssessments = passedAssessment;

                item failedAssessment : CheckpointAssessment subsets assessments {
                    ref part redefines checkpoint = workflow.'flow'.verificationAccepted;
                    attribute redefines evaluatedRevision = 3;
                    attribute redefines outcome = CheckpointOutcome::failed;
                    attribute redefines judgedBy = JudgeAuthority::human;
                    attribute redefines assessor = "review-board";
                    attribute redefines rationale = "The usability requirement is not met at revision 3.";
                    attribute redefines evidenceUris = ("cache/evidence/req-check-rev3.log");
                    ref item redefines activities = firstAttemptActivity;
                }

                item passedAssessment : CheckpointAssessment subsets assessments {
                    ref part redefines checkpoint = workflow.'flow'.verificationAccepted;
                    attribute redefines evaluatedRevision = 4;
                    attribute redefines outcome = CheckpointOutcome::passed;
                    attribute redefines judgedBy = JudgeAuthority::human;
                    attribute redefines assessor = "review-board";
                    attribute redefines rationale = "All four requirements pass at revision 4.";
                    attribute redefines evidenceUris = ("cache/evidence/req-check-rev4.log", "cache/build/report.xml");
                    ref item redefines activities = (correctiveActivity, secondAttemptActivity);
                }
            }
        }
    }

    occurrence def Wish0001Journey {
        doc /* This wish's journey as a sequence view. The lifelines are bound to the
           workflow's own controller, stages and this trace, so the messages are the
           steps this run actually took. Verification is assessed twice: the failed
           assessment of revision 3, then the passing assessment of revision 4 after
           corrective work, which is why assessVerification repeats. */

        ref part controller : Controller = workflow.controller;
        ref part requirements : Stage = workflow.'flow'.requirements;
        ref part design : Stage = workflow.'flow'.design;
        ref part implement : Stage = workflow.'flow'.implement;
        ref part 'verify' : Stage = workflow.'flow'.'verify';
        ref part trace : Trace = trace0001;

        message enterRequirements from controller.entering to requirements.entered;
        message assessRequirements from controller.assessing to requirements.assessed;
        message enterDesign from controller.entering to design.entered;
        message assessDesign from controller.assessing to design.assessed;
        message enterImplementation from controller.entering to implement.entered;
        message assessImplementation from controller.assessing to implement.assessed;
        message enterVerification from controller.entering to 'verify'.entered;
        message assessVerification from controller.assessing to 'verify'.assessed;
        message reassessVerification from controller.assessing to 'verify'.assessed;
        message deliverWish from controller.delivering to trace.delivered;
    }

    view wish0001Run : GeneralView {
        doc /* The run's records: the trace, its execution and the stage records it
           visited, each carrying the activities and assessments of this wish. */

        expose Wish0001::trace0001;
        expose Wish0001::trace0001::execution;
        render asInterconnectionDiagram;
    }
}
