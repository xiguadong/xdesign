package XDesignWorkflow {
    @XDesignCommon::DiagramPresentation { label = "Workflow"; }
    @XDesignCommon::DocSection { document = "docs/architecture/system-composition.md"; section = "B02"; }
    private import ScalarValues::*;
    private import XDesignArchitecture::*;
    private import XDesignCommon::*;

    doc /* Workflow structure, interfaces and a single-wish lifecycle scenario.
       Load this file together with common.sysml and architecture.sysml
       from the same directory. */

    port def WishExchange {
        @XDesignCommon::DocSection { document = "docs/architecture/system-composition.md"; section = "B02"; }
        doc /* A requested outcome enters the controller; registration returns
           the Wish and its independent execution Trace to the requester. */

        in attribute description : String;
        out item registeredWish : Wish;
        out ref part registeredTrace : Trace;
    }

    interface def WishInterface {
        @XDesignCommon::DocSection { document = "docs/architecture/system-composition.md"; section = "B02"; }
        end port requester : ~WishExchange;
        end port receiver : WishExchange;

        flow submission from requester.description to receiver.description;
        flow wishRegistration from receiver.registeredWish to requester.registeredWish;
        flow traceRegistration from receiver.registeredTrace to requester.registeredTrace;
    }

    port def FlowExchange {
        @XDesignCommon::DocSection { document = "docs/architecture/system-composition.md"; section = "B02"; }
        doc /* Publishes the reusable flow definition used for execution. */
        out ref part selectedFlow : Flow;
    }

    interface def FlowInterface {
        @XDesignCommon::DocSection { document = "docs/architecture/system-composition.md"; section = "B02"; }
        end port provider : FlowExchange;
        end port consumer : ~FlowExchange;

        flow flowDefinition from provider.selectedFlow to consumer.selectedFlow;
    }

    part def WishRequester {
        @XDesignCommon::DiagramPresentation { label = "Requester"; summary = "Expresses wishes and reviews outcomes"; }
        @XDesignCommon::DocSection { document = "docs/architecture/system-composition.md"; section = "B02"; }
        doc /* The user's interaction boundary. Wishes and traces remain owned
           by Workflow; the requester receives references through this port. */
        port wishPort : ~WishExchange;
    }

    part def WorkflowFlow :> SimpleFlow {
        @XDesignCommon::DocSection { document = "docs/architecture/system-composition.md"; section = "B02"; }
        port flowPort : FlowExchange;
        bind flowPort.selectedFlow = self;
    }

    part def WorkflowController :> Controller {
        @XDesignCommon::DocSection { document = "docs/architecture/system-composition.md"; section = "B02"; }
        port wishPort : WishExchange;
        port flowPort : ~FlowExchange;

        bind wishPort.description = receiveWish.description;
        bind wishPort.registeredWish = receiveWish.registeredWish;
        bind wishPort.registeredTrace = receiveWish.registeredTrace;
        bind flowPort.selectedFlow = defaultFlow;
    }

    action def StageWork {
        @XDesignCommon::DocSection { document = "docs/design/lifecycle.md"; section = "F04"; }
        doc /* Produce stage artifacts, append their activity record to the
           active visit, and ask the existing controller to assess and advance.
           The controller retains authority over checkpoints and trace status. */

        in part executor : Controller;
        in part wishTrace : Trace;
        in part stage : Stage;

        action produceArtifacts {
            ref part forStage : Stage = stage;
            out item activity : ActivityRecord;
        }
        then action record :> executor.recordActivity {
            in part redefines trace = wishTrace;
            in item redefines activity = produceArtifacts.activity;
        }
        then action advance :> executor.progressWish {
            in part redefines trace = wishTrace;
        }
    }

    part def Workflow :> XDesign {
        @XDesignCommon::DocSection { document = "docs/architecture/system-composition.md"; section = "B02"; }
        doc /* A four-stage workflow configuration and lifecycle scenario.
           Typed interfaces carry the request, registration and flow definition.
           The concrete selectable usage is owned by xdesign's flow portfolio. */

        part redefines 'flow' : WorkflowFlow subsets availableFlows {
            doc /* Requirements -> design -> implementation -> verification.
               The final checkpoint requires evidence that the implementation
               satisfies the requirements before the flow can complete. */
        }
        ref part redefines selectedFlow = 'flow';

        part redefines controller : WorkflowController {
            doc /* Receives wishes, records activities, evaluates checkpoints
               and advances or blocks each wish according to its evidence. */
        }

        item redefines wishes : Wish [0..*] {
            doc /* User-requested outcomes managed by this workflow system. */
        }

        part redefines traces : Trace [0..*] {
            doc /* Per-wish execution state, activity history and references to
               independent traceability models. */
        }

        part requester : WishRequester;

        interface wishExchange : WishInterface
            connect requester.wishPort to controller.wishPort;
        interface flowExchange : FlowInterface
            connect 'flow'.flowPort to controller.flowPort;

        perform action executeWish {
            doc /* One execution of the default flow. Activities and assessments
               accumulate in the same trace. Failed verification retries the
               current verification visit, preserving all earlier evidence. */

            out item registeredWish : Wish = intake.registeredWish;
            out part registeredTrace : Trace = intake.registeredTrace;

            action intake :> controller.receiveWish {
                in part redefines selectedFlow = 'flow';
            }
            action requirements : StageWork {
                in part redefines executor = controller;
                in part redefines wishTrace = intake.registeredTrace;
                in part redefines stage = 'flow'.requirements;
            }
            action design : StageWork {
                in part redefines executor = controller;
                in part redefines wishTrace = intake.registeredTrace;
                in part redefines stage = 'flow'.design;
            }
            action implementation : StageWork {
                in part redefines executor = controller;
                in part redefines wishTrace = intake.registeredTrace;
                in part redefines stage = 'flow'.implement;
            }
            merge verificationReady;
            action verifyRequirements : StageWork {
                in part redefines executor = controller;
                in part redefines wishTrace = intake.registeredTrace;
                in part redefines stage = 'flow'.'verify';
            }
            decide verificationOutcome;
            action blocked {
                doc /* Retain the blocked verification visit until corrective
                   work and new evidence are available. This is resumable. */
            }
            action correctiveWork {
                doc /* Correct artifacts in the current verification visit.
                   recordActivity increments the revision when artifacts change,
                   so the following verification must assess current evidence. */

                action correctArtifacts { out item activity : ActivityRecord; }
                then action record :> controller.recordActivity {
                    in part redefines trace = intake.registeredTrace;
                    in item redefines activity = correctArtifacts.activity;
                }
            }
            action delivered {
                doc /* Retain the traceability model reference, artifacts,
                   decisions and verification evidence as the delivery record. */
                assert constraint deliveryAuthorized {
                    intake.registeredTrace.status == WishStatus::delivered and
                    intake.registeredTrace.execution.status == FlowStatus::completed
                }
            }

            first start then intake;
            succession requirementsStarted first intake then requirements;
            succession designStarted first requirements then design;
            succession implementationStarted first design then implementation;
            succession verificationStarted first implementation then verificationReady;
            succession verificationAttempt first verificationReady then verifyRequirements;
            succession verificationAssessed first verifyRequirements then verificationOutcome;
            succession verificationPassed first verificationOutcome
                if intake.registeredTrace.status == WishStatus::delivered then delivered;
            succession verificationBlocked first verificationOutcome
                if intake.registeredTrace.status == WishStatus::blocked then blocked;
            succession correctionRequested first blocked then correctiveWork;
            succession verificationRetried first correctiveWork then verificationReady;
            first delivered then done;
        }
    }

    view wishLifecycle : StandardViewDefinitions::ActionFlowView {
        @XDesignCommon::DocSection { document = "docs/architecture/constraints.md"; section = "T03"; }
        doc /* Intake -> requirements -> design -> implementation -> verification
           -> delivered, with guarded verification failure and retry paths.
           Nested StageWork actions record artifacts and invoke the controller. */

        expose Workflow::executeWish;
        expose Workflow::executeWish::*;

        render Views::asInterconnectionDiagram;
    }

    view workflowInterfaces : StandardViewDefinitions::InterconnectionView {
        @XDesignCommon::DocSection { document = "docs/architecture/constraints.md"; section = "T03"; }
        doc /* Requester <-> controller through WishInterface, and reusable flow
           -> controller through FlowInterface. Port directions and interface
           flows specify the exchanged request, wish, trace and flow definition. */

        expose Workflow::requester;
        expose Workflow::controller;
        expose Workflow::'flow';
        expose Workflow::wishes;
        expose Workflow::traces;
        expose Workflow::wishExchange;
        expose Workflow::flowExchange;

        render Views::asInterconnectionDiagram;
    }
}
