package Wish0002 {
    @XDesignCommon::DocSection { document = "docs/review/01_wish_status.md"; section = "V02"; }
    private import ScalarValues::*;
    private import XDesignCommon::*;
    private import XDesignArchitecture::*;
    private import XDesignWorkflow::*;
    private import XDesignComposition::*;
    private import StandardViewDefinitions::*;
    private import Views::*;

    doc /* The independent traceability model for wish-0002: split every checkpoint in
       the flow into the criteria a fixed code or script check can establish and the
       criteria that need judgment, and record why each checkpoint fell where it did.
       The requirements this wish produced are retained in XDesignRequirements, which
       is what a later wish moved them into; this file keeps the wish, the derivation
       and the account of the run. Definitions stay in mbse/model. */

    item wish0002 : Wish {
        attribute redefines wishId = "wish-0002";
        attribute redefines description = "Split each checkpoint into syntactic checks that code or scripts can fix, and semantic criteria that an agent or a human judges.";
    }

    doc /* Which requirements this wish produced. They are retained in
       XDesignRequirements, where they stand on their own; this file records that this
       wish derived them. The dependency runs one way, from the wish to the model:
       the model never names a wish. */
    dependency producedCheckpointKindIsDeclared from wish0002 to XDesignRequirements::checkpointKindIsDeclared;
    dependency producedSyntacticCheckIsFixed from wish0002 to XDesignRequirements::syntacticCheckIsFixed;
    dependency producedSyntacticAssessmentIsScripted from wish0002 to XDesignRequirements::syntacticAssessmentIsScripted;
    dependency producedSemanticCheckNamesItsJudges from wish0002 to XDesignRequirements::semanticCheckNamesItsJudges;
    dependency producedCheckerStatesItsConformance from wish0002 to XDesignRequirements::checkerStatesItsConformance;

    part trace0002 : Trace {
        doc /* Verification was assessed at each revision it judged. Revision 1
           classified the checkpoints by a multi-valued list of permitted judges; the
           export showed only the first of each pair, so a script reading it would deny
           the paired judge, and one flag per authority replaced it at revision 2. A
           later wish moved the requirements this wish produced into the model, which
           changed this file again and raised the revision to 3. */

        ref item redefines wish = wish0002;
        attribute redefines status = WishStatus::delivered;
        attribute redefines traceabilityModelUri = "mbse/wishes/wish-0002.sysml";

        part redefines execution : FlowExecution {
            ref part redefines assignedFlow = xdesign.workflowFlow;
            attribute redefines status = FlowStatus::completed;
            part redefines stageRecords : StageRecord [4];
            ref part redefines currentRecord = verifyRecord;

            part requirementsRecord : StageRecord subsets stageRecords {
                ref part redefines stage = xdesign.workflowFlow.requirements;
                attribute redefines visitIndex = 0;
                attribute redefines status = StageStatus::completed;
                attribute redefines artifactRevision = 1;

                item requirementsActivity : ActivityRecord subsets activities {
                    attribute redefines description = "Derived six design constraints from the wish and linked each to the model feature that satisfies it.";
                    attribute redefines performedBy = "zcode-agent";
                    attribute redefines occurredAt = "2026-10-14T11:00:00Z";
                    attribute redefines artifactUris = ("mbse/wishes/wish-0002.sysml");
                }
            }

            part designRecord : StageRecord subsets stageRecords {
                ref part redefines stage = xdesign.workflowFlow.design;
                attribute redefines visitIndex = 0;
                attribute redefines status = StageStatus::completed;
                attribute redefines artifactRevision = 1;

                item designActivity : ActivityRecord subsets activities {
                    attribute redefines description = "Designed the classification: a check kind, a judging authority, the conformance kinds a checker can establish, a Checker definition, and the kind, checker and judge features on Checkpoint.";
                    attribute redefines performedBy = "zcode-agent";
                    attribute redefines occurredAt = "2026-10-14T11:10:00Z";
                    attribute redefines artifactUris = ("mbse/model/common.sysml", "mbse/model/architecture.sysml");
                }
            }

            part implementRecord : StageRecord subsets stageRecords {
                ref part redefines stage = xdesign.workflowFlow.implement;
                attribute redefines visitIndex = 0;
                attribute redefines status = StageStatus::completed;
                attribute redefines artifactRevision = 1;

                item implementActivity : ActivityRecord subsets activities {
                    attribute redefines description = "Classified all twelve checkpoints, added three checkers to the V-model, and gave each assessment a judging authority.";
                    attribute redefines performedBy = "zcode-agent";
                    attribute redefines occurredAt = "2026-10-14T11:25:00Z";
                    attribute redefines artifactUris = ("mbse/model/common.sysml", "mbse/model/architecture.sysml", "mbse/wishes/wish-0001.sysml");
                }
            }

            part verifyRecord : StageRecord subsets stageRecords {
                ref part redefines stage = xdesign.workflowFlow.'verify';
                attribute redefines visitIndex = 0;
                attribute redefines status = StageStatus::completed;
                attribute redefines artifactRevision = 3;

                item firstAttemptActivity : ActivityRecord subsets activities {
                    attribute redefines description = "Checked the classification against the compiler's semantic export; the export carried only one of the two judges permitted on seven checkpoints.";
                    attribute redefines performedBy = "zcode-agent";
                    attribute redefines occurredAt = "2026-10-14T11:40:00Z";
                    attribute redefines artifactUris = ("cache/evidence/wish-0002/classification.txt");
                }
                item truncationActivity : ActivityRecord subsets activities {
                    attribute redefines description = "Isolated the cause with a three-element probe: the export keeps only the first element of a multi-valued value. Replaced the permitted-judge list with one flag per authority.";
                    attribute redefines performedBy = "zcode-agent";
                    attribute redefines occurredAt = "2026-10-14T11:50:00Z";
                    attribute redefines artifactUris = ("cache/review_probe/seqval/s.sysml", "mbse/model/architecture.sysml");
                }
                item secondAttemptActivity : ActivityRecord subsets activities {
                    attribute redefines description = "Re-ran the classification check against the re-exported model and the compiler check over the model and wish directories.";
                    attribute redefines performedBy = "zcode-agent";
                    attribute redefines occurredAt = "2026-10-14T12:00:00Z";
                    attribute redefines artifactUris = ("cache/evidence/wish-0002/classification.txt", "cache/sysml/check.json");
                }
                item handoverActivity : ActivityRecord subsets activities {
                    attribute redefines description = "Decided this wish produces no checker code, so the three syntactic checkers remain unimplemented and any wish on the V-model blocks at their checkpoints until they exist. That scope decision is recorded here rather than as a requirement, since it constrains only this wish.";
                    attribute redefines performedBy = "zcode-agent";
                    attribute redefines occurredAt = "2026-10-14T12:10:00Z";
                    attribute redefines artifactUris = ("mbse/model/architecture.sysml");
                }
                item migrationActivity : ActivityRecord subsets activities {
                    attribute redefines description = "A later wish moved the requirements this wish produced into the model and left only the derivation here, which changed this file and required the judgment below to be made again.";
                    attribute redefines performedBy = "zcode-agent";
                    attribute redefines occurredAt = "2026-10-14T16:26:00Z";
                    attribute redefines artifactUris = ("mbse/model/requirements.sysml", "mbse/wishes/wish-0002.sysml");
                }

                item redefines assessments : CheckpointAssessment [3];
                ref item redefines currentAssessments = migrationAssessment;

                item failedAssessment : CheckpointAssessment subsets assessments {
                    ref part redefines checkpoint = xdesign.workflowFlow.verificationAccepted;
                    attribute redefines evaluatedRevision = 1;
                    attribute redefines outcome = CheckpointOutcome::failed;
                    attribute redefines judgedBy = JudgeAuthority::agent;
                    attribute redefines assessor = "zcode-agent";
                    attribute redefines rationale = "Seven checkpoints permit both an agent and a human, but the exported model shows only the agent, so a script reading it would refuse a human judgment.";
                    attribute redefines evidenceUris = ("cache/evidence/wish-0002/classification.txt");
                    ref item redefines activities = firstAttemptActivity;
                }

                item passedAssessment : CheckpointAssessment subsets assessments {
                    ref part redefines checkpoint = xdesign.workflowFlow.verificationAccepted;
                    attribute redefines evaluatedRevision = 2;
                    attribute redefines outcome = CheckpointOutcome::passed;
                    attribute redefines judgedBy = JudgeAuthority::agent;
                    attribute redefines assessor = "zcode-agent";
                    attribute redefines rationale = "Three checkpoints reduce to facts a script can read from the export and are syntactic; the other nine ask whether parts address requirements, a design supports implementation, or evidence demonstrates an outcome, and need judgment. Each judgment permit now survives the export as its own flag. The checkers the syntactic checkpoints name do not exist yet, which this wish states and does not attempt.";
                    attribute redefines evidenceUris = ("cache/evidence/wish-0002/classification.txt", "cache/sysml/check.json", "cache/sysml/parse.json");
                    ref item redefines activities = (truncationActivity, secondAttemptActivity, handoverActivity);
                }

                item migrationAssessment : CheckpointAssessment subsets assessments {
                    ref part redefines checkpoint = xdesign.workflowFlow.verificationAccepted;
                    attribute redefines evaluatedRevision = 3;
                    attribute redefines outcome = CheckpointOutcome::passed;
                    attribute redefines judgedBy = JudgeAuthority::agent;
                    attribute redefines assessor = "zcode-agent";
                    attribute redefines rationale = "The five requirements this wish produced are now held by the model and stand without this record, and this file changed only by handing them over. The judgment of revision 2 still holds for the model as it now stands.";
                    attribute redefines evidenceUris = ("mbse/model/requirements.sysml", "cache/evidence/wish-0003/invariants.txt");
                    ref item redefines activities = migrationActivity;
                }
            }
        }
    }

    occurrence def Wish0002Journey {
        doc /* This wish's journey as a sequence view, on the same lifelines as any
           SimpleFlow wish: the controller enters each stage and then evaluates that
           stage's only checkpoint. Verification was assessed at each revision it
           judged, and handing the requirements over to the model changed this file
           again, which is why assessVerification repeats. */

        ref part controller : Controller = xdesign.controller;
        ref part requirements : Stage = xdesign.workflowFlow.requirements;
        ref part design : Stage = xdesign.workflowFlow.design;
        ref part implement : Stage = xdesign.workflowFlow.implement;
        ref part 'verify' : Stage = xdesign.workflowFlow.'verify';
        ref part trace : Trace = trace0002;

        message enterRequirements from controller.entering to requirements.entered;
        message assessRequirements from controller.assessing to requirements.assessed;
        message enterDesign from controller.entering to design.entered;
        message assessDesign from controller.assessing to design.assessed;
        message enterImplementation from controller.entering to implement.entered;
        message assessImplementation from controller.assessing to implement.assessed;
        message enterVerification from controller.entering to 'verify'.entered;
        message assessVerification from controller.assessing to 'verify'.assessed;
        message reassessVerification from controller.assessing to 'verify'.assessed;
        message rejudgeVerification from controller.assessing to 'verify'.assessed;
        message deliverWish from controller.delivering to trace.delivered;
    }

    view wish0002Run : GeneralView {
        doc /* The run's records, with the requests this wish was broken into and the
           model features that satisfy them. */

        expose Wish0002::wish0002;
        expose Wish0002::trace0002;
        expose Wish0002::trace0002::execution;
        render asInterconnectionDiagram;
    }
}
