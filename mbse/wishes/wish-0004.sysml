package Wish0004 {
    @XDesignCommon::DocSection { document = "docs/review/01_wish_status.md"; section = "V04"; }
    private import ScalarValues::*;
    private import XDesignCommon::*;
    private import XDesignArchitecture::*;
    private import XDesignWorkflow::*;

    doc /* The independent traceability model for wish-0004, registered at intake and
       deliberately left there. No requirements have been derived from it, no design
       or implementation work follows from it, and no checkpoint has been assessed,
       so this file holds only the wish, its assigned flow and the pending records
       that registration creates.

       It was raised while reviewing wish-0002's trace. Two satisfy usages of one
       requirement reference each other in the compiler's export rather than the
       requirement they were written for, so anything rendering or reading a trace
       cannot attribute them without an inference the model does not support.
       Answering whether one requirement may have two satisfiers at all is part of
       this wish, and advancing it before that answer would build on the inference. */

    item wish0004 : Wish {
        attribute redefines wishId = "wish-0004";
        attribute redefines description = "Make a wish's traceability chain a capability of the tool instead of a one-off projection: render wish, the requirements it produced and the model elements that satisfy them. Settle first whether one requirement with two satisfiers is a valid model, since the export cannot attribute them.";
    }

    part trace0004 : Trace {
        doc /* Registered, not blocked. No stage has been entered beyond the initial one,
           nothing has been attempted and assessed, and nothing about this wish exists
           in the model yet, so resuming it costs nothing. */

        ref item redefines wish = wish0004;
        attribute redefines status = WishStatus::active;
        attribute redefines traceabilityModelUri = "mbse/wishes/wish-0004.sysml";

        part redefines execution : FlowExecution {
            ref part redefines assignedFlow = workflow.'flow';
            attribute redefines status = FlowStatus::active;
            part redefines stageRecords : StageRecord [4];
            ref part redefines currentRecord = requirementsRecord;

            part requirementsRecord : StageRecord subsets stageRecords {
                ref part redefines stage = workflow.'flow'.requirements;
                attribute redefines visitIndex = 0;
                attribute redefines status = StageStatus::active;
                attribute redefines artifactRevision = 0;
            }

            part designRecord : StageRecord subsets stageRecords {
                ref part redefines stage = workflow.'flow'.design;
                attribute redefines visitIndex = 0;
                attribute redefines status = StageStatus::pending;
                attribute redefines artifactRevision = 0;
            }

            part implementRecord : StageRecord subsets stageRecords {
                ref part redefines stage = workflow.'flow'.implement;
                attribute redefines visitIndex = 0;
                attribute redefines status = StageStatus::pending;
                attribute redefines artifactRevision = 0;
            }

            part verifyRecord : StageRecord subsets stageRecords {
                ref part redefines stage = workflow.'flow'.'verify';
                attribute redefines visitIndex = 0;
                attribute redefines status = StageStatus::pending;
                attribute redefines artifactRevision = 0;
            }
        }
    }
}
