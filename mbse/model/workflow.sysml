package XDesignWorkflow {
    @XDesignCommon::DiagramPresentation { label = "Workflow"; }
    @XDesignCommon::DocSection { document = "docs/architecture/system-composition.md"; section = "B02"; }
    private import ScalarValues::*;
    private import ControlFunctions::forAll;
    private import SequenceFunctions::includes;
    private import SequenceFunctions::size;
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

    item def StageWorkOrder {
        @XDesignCommon::DocSection { document = "docs/design/lifecycle.md"; section = "F04"; }
        doc /* Ephemeral read-only projection of one registered wish's current
           stage visit at query time, not a separately editable execution record.
           Work content and exit checks come from its assigned frozen flow plan;
           the agent does not invent a stage-specific command procedure.
           Artifact declarations identify workspace inputs and expected outputs,
           including the inputs read by fixed checkers. Concrete flow bindings
           must supply the goal, instructions and artifact declarations before
           the CLI can offer that stage as executable work. */

        ref part trace : Trace;
        ref part execution : FlowExecution;
        ref part visit : StageRecord;
        ref part stage : Stage = visit.stage;
        attribute wishId : String = trace.wish.wishId;
        attribute goal : String [0..1] = stage.goal;
        attribute workInstructions : String [0..1] = stage.workInstructions;
        attribute inputArtifacts : String [0..*] = stage.inputArtifacts;
        attribute expectedOutputs : String [0..*] = stage.expectedOutputs;
        attribute artifactRevision : Integer = visit.artifactRevision;
        ref part exitCheckpoints : Checkpoint [0..*] = stage.exitCheckpoints;
        ref item currentAssessments : CheckpointAssessment [0..*] = visit.currentAssessments;
        attribute correctionReasons : String [0..*];

    }

    part def StageCli {
        @XDesignCommon::DocSection { document = "docs/design/lifecycle.md"; section = "F04"; }
        doc /* Common command boundary for the agent's repeated stage loop.
           Its two public operations are xdesign next WISH_ID and
           xdesign check WISH_ID. Registration and final human acceptance are
           boundary interactions outside this loop. LocalStageCliRuntime realizes a bounded flat-flow implementation;
           historical flow results remain unchanged.
           Resolve the existing wish through controller.traces on every call;
           an unknown wish is an error, never an implicit registration.
           Only xdesign writes activities, assessments and execution state. */

        ref part controller : Controller;

        perform action 'next' {
            doc /* Read the existing wish's active execution path and return its
               current work order, or completed with no work when the root run
               is complete. Include the goal, instructions, input/output
               declarations, exit checkpoint criteria and permitted checker or
               judges, current assessments and correction reasons. Do not
               record work, assess, increment a revision, advance, initialize
               a subflow, or reopen a completed visit. Repeating next observes
               the same visit until a mutating operation changes it. Follow the
               deepest initialized incomplete child execution; a completed child
               resumes its parent. Reject an uninitialized required child or
               missing goal/instructions with a diagnostic and no mutation.
               Entry preparation belongs to registration/progression, and CLI
               integration for unprepared nested flows remains unavailable. */

            in attribute wishId : String;
            out attribute completed : Boolean;
            out item work : StageWorkOrder [0..1];

            assert constraint responseMatchesCurrentWork {
                ((completed and size(work) == 0) or
                    (not completed and size(work) == 1)) and
                work->forAll { in order : StageWorkOrder;
                    order.wishId == wishId and
                    controller.traces->includes(order.trace) and
                    order.visit == order.execution.currentRecord and
                    order.execution.assignedFlow.stages->includes(order.stage)
                }
            }
        }

        perform action check {
            doc /* Collect and record the current stage outputs, assess its exit
               checkpoints and advance when authorized, all in one public call.
               An optional --result FILE supplies work/evidence that the stage's
               artifact declarations cannot locate. It is not an unchecked pass.
               Fixed checkers produce syntactic verdicts; semantic checkpoints
               use only their permitted judges. A missing or refused judgment
               stays unevaluated; a valid negative evaluation is failed.
               Retain evidence and assessments before returning the result.
               Calling check on a completed run returns completion without
               recording, reassessing or reopening it. */

            in attribute wishId : String;
            in attribute resultUri : String [0..1];
            out attribute completed : Boolean = returnWork.completed;
            out item work : StageWorkOrder [0..1] = returnWork.work;
            out ref item assessments : CheckpointAssessment [0..*] = observe.work.visit.currentAssessments;

            action observe :> 'next' {
                in attribute redefines wishId = check.wishId;
            }
            then if not observe.completed {
                action collectOutputs {
                    doc /* Capture the declared outputs and any supplied result
                       as one activity for the observed visit. xdesign derives
                       content changes from the recorded artifacts. Unchanged
                       content does not raise the revision; changed content
                       invalidates earlier passes. Missing required outputs
                       remain diagnostic input to assessment, never a pass. */
                    in item workOrder : StageWorkOrder = observe.work;
                    in attribute suppliedResult : String [0..1] = resultUri;
                    out item activity : ActivityRecord;
                }
                then action record :> controller.recordActivity {
                    in part redefines trace = observe.work.trace;
                    in item redefines activity = collectOutputs.activity;
                }
                then action assessAndAdvance :> controller.progressWish {
                    doc /* Assess the recorded visit at its current revision.
                       Failed, stale or unevaluated exit checkpoints retain the
                       same visit and return correction reasons. A pass enters
                       the successor; an authorized terminal visit completes
                       the run. Return the assessed visit's assessments even
                       when the returned work order belongs to its successor. */
                    in part redefines trace = observe.work.trace;
                }
            }
            then action returnWork :> 'next' {
                in attribute redefines wishId = check.wishId;
            }
        }
    }

    action def AgentStageWork {
        @XDesignCommon::DocSection { document = "docs/design/lifecycle.md"; section = "F04"; }
        doc /* One uniform agent iteration for any stage. The agent calls next,
           performs the returned work directly in the project workspace, then
           calls check. Repeat with the returned current or successor work order
           until completion; next also recovers the work order after a restart.
           Work production belongs to the agent; record/check/advance belong
           to xdesign and are not separate public stage commands. */

        in part cli : StageCli;
        in attribute wishId : String;

        action observe :> cli.'next' {
            in attribute redefines wishId = AgentStageWork.wishId;
        }
        then if not observe.completed {
            action performWork {
                doc /* The agent edits workspace artifacts according to the
                   work order, without editing xdesign's execution records. */
                in item workOrder : StageWorkOrder = observe.work;
                out attribute resultUri : String [0..1];
            }
            then action submit :> cli.check {
                in attribute redefines wishId = AgentStageWork.wishId;
                in attribute redefines resultUri = performWork.resultUri;
            }
        }
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
               legacy independent SysML traces or store-backed operational revisions. */
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

    part def LocalStageCliRuntime :> StageCli {
        @XDesignCommon::DocSection { document = "docs/design/stage-cli.md"; section = "D85"; }
        doc /* Local flat-flow realization of StageCli. New registrations import a
           wish source into XDesignStorage and materialize its working copy under
           cache/wishes. Frozen plans, work definitions, run records and checker
           exchanges are working copies of stored revision materials. Readers use
           immutable stored heads; check holds a per-wish writer lock, rejects stale
           envelopes, records, assesses and advances through the existing core engine,
           then commits the complete stage revision and operation receipt together.
           Failed gates retain blocked facts. Legacy runs registered before this
           boundary remain on the cache backend and are not implicitly migrated.
           Unsupported nested flows, identity authentication and final human
           acceptance remain outside this local runtime. */
        attribute storageRoot : String = ".xdesign";
        attribute workingRoot : String = "cache";
        attribute schemaVersion : Integer = 1;
        attribute supportedFlows : String [2] = ("simple-flow", "wish-verify");
        perform action initialize {
            doc /* Import an existing SysML wish source, defaulting to the legacy
               mbse/wishes path or accepting --wish FILE. Create the store item and
               initial revision exclusively, freeze plan and work, materialize the
               wish working source, and publish registration last. */
        }
        perform action restore {
            doc /* Read the selected stored head and validate record identity, plan
               and work digests, replayed activity revisions, assessment authority
               and completed-stage authorization. Regenerate missing cache copies
               without overwriting uncommitted workspace edits. */
        }
        perform action saveRevision {
            doc /* Commit the registry, plan, work, run, wish source, checker input
               and report copies, current artifacts, evidence and response as one
               expected-head operation after the engine has produced the stage result. */
        }
    }
}
