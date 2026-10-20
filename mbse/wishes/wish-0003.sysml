package Wish0003 {
    @XDesignCommon::DocSection { document = "docs/review/01_wish_status.md"; section = "V02"; }
    private import ScalarValues::*;
    private import XDesignCommon::*;
    private import XDesignArchitecture::*;
    private import XDesignWorkflow::*;
    private import XDesignComposition::*;
    private import StandardViewDefinitions::*;
    private import Views::*;

    doc /* The independent traceability model for wish-0003: what a wish produces belongs
       to the project model, not to the wish's own record. This wish was raised in
       response to wish-0002, whose requirements had been left in its own file, and it
       is the first wish whose product is a boundary rule rather than a mechanism. */

    item wish0003 : Wish {
        attribute redefines wishId = "wish-0003";
        attribute redefines description = "Requirements a wish produces, and the model modifications it causes, are retained in the SysML project model instead of living only in the wish's own record.";
    }

    doc /* What this wish produced in the model. The one-way direction is the point: the
       model holds the requirements, and only this record names the wish. */
    dependency producedWishProductsAreRetained from wish0003 to XDesignRequirements::wishProductsAreRetained;
    dependency producedModelStandsWithoutWishes from wish0003 to XDesignRequirements::modelStandsWithoutWishes;

    part trace0003 : Trace {
        doc /* This wish ran to completion in one pass, with no failed assessment: the
           boundary it draws was already the direction the model used, so aligning the
           files with it did not change any mechanism. wish-0002's record needed a third
           revision because its own file changed, not because this wish's judgment did. */

        ref item redefines wish = wish0003;
        attribute redefines status = WishStatus::delivered;
        attribute redefines traceabilityModelUri = "mbse/wishes/wish-0003.sysml";

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
                    attribute redefines description = "Took the retention constraint as this wish's requirement and derived the invariant that makes it meaningful alongside it: the model stands without any wish.";
                    attribute redefines performedBy = "zcode-agent";
                    attribute redefines occurredAt = "2026-10-14T15:58:00Z";
                    attribute redefines artifactUris = ("mbse/model/requirements.sysml");
                }
            }

            part designRecord : StageRecord subsets stageRecords {
                ref part redefines stage = xdesign.workflowFlow.design;
                attribute redefines visitIndex = 0;
                attribute redefines status = StageStatus::completed;
                attribute redefines artifactRevision = 1;

                item designActivity : ActivityRecord subsets activities {
                    attribute redefines description = "Assigned each product a home: requirements in requirements.sysml, satisfaction links in traceability.sysml, this wish's derivation in this file, and nothing about wishes anywhere in the model. Requirements carry a rationale, so a reader judges a requirement on its reason rather than on which wish asked for it.";
                    attribute redefines performedBy = "zcode-agent";
                    attribute redefines occurredAt = "2026-10-14T16:00:00Z";
                    attribute redefines artifactUris = ("mbse/model/requirements.sysml", "mbse/model/traceability.sysml");
                }
            }

            part implementRecord : StageRecord subsets stageRecords {
                ref part redefines stage = xdesign.workflowFlow.implement;
                attribute redefines visitIndex = 0;
                attribute redefines status = StageStatus::completed;
                attribute redefines artifactRevision = 1;

                item implementActivity : ActivityRecord subsets activities {
                    attribute redefines description = "Created requirements.sysml and traceability.sysml, moved wish-0002's five requirements and the six satisfaction links into them, replaced those declarations in wish-0002 with derivation dependencies, and recorded the handover as its third revision so its completed stage stayed authorized.";
                    attribute redefines performedBy = "zcode-agent";
                    attribute redefines occurredAt = "2026-10-14T16:10:00Z";
                    attribute redefines artifactUris = ("mbse/model/requirements.sysml", "mbse/model/traceability.sysml", "mbse/wishes/wish-0002.sysml", "mbse/AGENTS.md");
                }
                item decisionActivity : ActivityRecord subsets activities {
                    attribute redefines description = "Judged that wish-0002's scope constraint, that it would write no checker code, is not a requirement on the tool but a decision about that wish, so it was recorded as an activity in wish-0002 rather than carried into the baseline.";
                    attribute redefines performedBy = "zcode-agent";
                    attribute redefines occurredAt = "2026-10-14T16:12:00Z";
                    attribute redefines artifactUris = ("mbse/wishes/wish-0002.sysml");
                }
            }

            part verifyRecord : StageRecord subsets stageRecords {
                ref part redefines stage = xdesign.workflowFlow.'verify';
                attribute redefines visitIndex = 0;
                attribute redefines status = StageStatus::completed;
                attribute redefines artifactRevision = 1;

                item verifyActivity : ActivityRecord subsets activities {
                    attribute redefines description = "Checked the boundary from both sides: the model resolves and checks on its own, and no model element names a wish. Also confirmed every requirement carries a rationale and every satisfaction link resolves.";
                    attribute redefines performedBy = "zcode-agent";
                    attribute redefines occurredAt = "2026-10-14T16:25:00Z";
                    attribute redefines artifactUris = ("cache/evidence/wish-0003/invariants.txt", "cache/sysml/check.json");
                }

                item redefines assessments : CheckpointAssessment [1];
                ref item redefines currentAssessments = verificationAssessment;

                item verificationAssessment : CheckpointAssessment subsets assessments {
                    ref part redefines checkpoint = xdesign.workflowFlow.verificationAccepted;
                    attribute redefines evaluatedRevision = 1;
                    attribute redefines outcome = CheckpointOutcome::passed;
                    attribute redefines judgedBy = JudgeAuthority::agent;
                    attribute redefines assessor = "zcode-agent";
                    attribute redefines rationale = "Both requirements hold as far as a reading can establish: the model's seven requirements each carry their reason, the six satisfaction links resolve against model features, the model checks without the wish directory, and the only trace of a wish anywhere in the model is the two-field Wish definition. Neither requirement is enforced by the tool yet, so both stand unmet in traceability.sysml rather than falsely satisfied.";
                    attribute redefines evidenceUris = ("cache/evidence/wish-0003/invariants.txt", "cache/sysml/check.json", "cache/sysml/parse.json");
                    ref item redefines activities = verifyActivity;
                }
            }
        }
    }

    occurrence def Wish0003Journey {
        doc /* This wish's journey as a sequence view: the controller enters each stage and
           evaluates the flow's only checkpoint once, with no rework, so each checkpoint
           is assessed exactly once and the wish is delivered. */

        ref part controller : Controller = xdesign.controller;
        ref part requirements : Stage = xdesign.workflowFlow.requirements;
        ref part design : Stage = xdesign.workflowFlow.design;
        ref part implement : Stage = xdesign.workflowFlow.implement;
        ref part 'verify' : Stage = xdesign.workflowFlow.'verify';
        ref part trace : Trace = trace0003;

        message enterRequirements from controller.entering to requirements.entered;
        message assessRequirements from controller.assessing to requirements.assessed;
        message enterDesign from controller.entering to design.entered;
        message assessDesign from controller.assessing to design.assessed;
        message enterImplementation from controller.entering to implement.entered;
        message assessImplementation from controller.assessing to implement.assessed;
        message enterVerification from controller.entering to 'verify'.entered;
        message assessVerification from controller.assessing to 'verify'.assessed;
        message deliverWish from controller.delivering to trace.delivered;
    }

    view wish0003Run : GeneralView {
        doc /* The run's records, and the two requirements this wish contributed to the
           project model. */

        expose Wish0003::wish0003;
        expose Wish0003::trace0003;
        expose Wish0003::trace0003::execution;
        render asInterconnectionDiagram;
    }
}
