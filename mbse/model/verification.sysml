package XDesignVerification {
    @XDesignCommon::DiagramPresentation { label = "Verification"; }
    @XDesignCommon::DocSection { document = "docs/verification/requirements.md"; section = "D15"; }
    private import XDesignDetailedDesign::*;
    private import XDesignRustUml::*;

    doc /* Verification designs, separate from executed results. Declaring a case
       and its verify relationships specifies coverage; run evidence belongs to the
       independent trace record, and compiler checks do not execute this review. */

    verification def StageCliInteractionVerification {
        @XDesignCommon::DocSection { document = "docs/verification/design.md"; section = "D54"; }
        doc /* Planned verification of the two-command stage interaction, not
           an executed CLI test. On two different configured flows, call next
           repeatedly and confirm the current work order comes from the assigned
           stage without changing visits, revisions, activities or assessments.
           Perform workspace work and call check once: it must record outputs
           before evaluating that visit's exit checkpoints, retain assessments
           and evidence, and either return the same blocked visit with reasons,
           enter the successor and return its work, or report terminal completion.
           Correct blocked work in the same visit; only content changes raise
           its revision and invalidate old passes. Missing artifacts, unavailable
           checks, stale passes and unpermitted judges cannot authorize progress.
           A valid negative check is failed; an unavailable evaluation is
           unevaluated. Check can return the next work without another next call.
           A completed run returns completion from both commands without writing
           or reopening; an unknown wish produces an error without registration.
           Resume with next in a fresh process. Missing work instructions or an
           uninitialized required subflow must be rejected without mutation;
           select an initialized incomplete child and resume its parent after
           child completion. Registration, final human
           acceptance and any subflow-entry initialization stay outside the
           read-only query. No extra record/assess/advance command is required. */
        subject cli : XDesignWorkflow::StageCli;
        objective interaction {
            verify requirement scriptAssessment :> XDesignRequirements::syntacticAssessmentIsScripted;
            verify requirement judgeAuthority :> XDesignRequirements::semanticCheckNamesItsJudges;
            verify requirement honestStatus :> XDesignRequirements::softwareDesignClaimsNoImplementation;
        }
    }

    verification def SimpleFlowViewReview {
        @XDesignCommon::DiagramPresentation { label = "Design review"; summary = "Planned coverage review"; }
        @XDesignCommon::DocSection { document = "docs/verification/requirements.md"; section = "D15"; }
        doc /* Inspect the run/factory/operation design, every stage's gates, and all
           nine coordinated views. Re-render each figure, inspect native endpoints,
           check its documentation link and exercise authorization truth cases with
           missing, failed, stale and duplicate assessments and unfinished subflows. */
        subject automation : SimpleFlowAutomation;
        objective coverage {
            verify requirement runDesign :> XDesignRequirements::flowRunIsRealizedByDesign;
            verify requirement stageGates :> XDesignRequirements::designedFlowGatesEveryStage;
            verify requirement nineViews :> XDesignRequirements::structureGraphsAreDocumented;
        }
    }

    verification def DiagramPresentationReview {
        @XDesignCommon::DocSection { document = "docs/interfaces/documentation-contract.md"; section = "D25"; }
        @XDesignCommon::DiagramPresentation { label = "Diagram readability review"; summary = "Inspect the final nine figures and their source identity"; }
        doc /* This case specifies acceptance; it is not itself an executed test.
           Rebuild the nine primary SimpleFlow figures from their native views.
           Check relationship identity, multiplicities, guards, bound parameters,
           actor roles and interface endpoints against the export. Measure SVG
           text at a 900-pixel article width and inspect matching raster previews
           for clipping, node overlaps, label collisions and identifiable routes.
           Inspect boundaries, ports and notation for each diagram kind. Retain
           full source identity and details in tooltips or source references.
           Bind the review to the final SVG hashes and matching preview hashes;
           regenerate and re-review any changed figure before acceptance. */
        subject renderer : DiagramDocumentationRenderer;
        objective presentation {
            verify requirement readableViews :> XDesignRequirements::diagramViewsAreReadable;
            verify requirement sourceIdentity :> XDesignRequirements::diagramPresentationRetainsSourceIdentity;
        }
    }

    verification def DocumentationContractReview {
        @XDesignCommon::DocSection { document = "docs/records/wishes/wish-history.md"; section = "V07"; }
        doc /* This case specifies design-stage acceptance; its declaration is not an
           executed result. Inspect the documentation entry, current document set,
           reverse model map, and an independently located design section for each
           flow. Add a hypothetical second flow to the index convention to check
           that existing flow locations and section identifiers remain stable.
           For each documented figure, compare the native view and its concern to
           the DocFigure target, rendered asset, embed, companion references,
           explanation, status, and stated limits. For each DocSection, check the
           document and stable anchor. Read the viewpoint convention and nine
           reference concerns against declared views, recording applicable and
           inapplicable concerns with reasons. Read each checker rule's inputs,
           condition, diagnostic, evidence output, and implementation status.
           Run the documentation target audit and strict site build for mechanical
           consistency, then judge the reading path and meaning separately. */
        objective contract {
            verify requirement entryNavigation :> XDesignRequirements::documentationEntryMapsDesign;
            verify requirement flows :> XDesignRequirements::documentationSupportsMultipleFlows;
            verify requirement figures :> XDesignRequirements::figuresStayWithDesignExplanation;
            verify requirement links :> XDesignRequirements::modelDocumentationLinksResolve;
            verify requirement views :> XDesignRequirements::viewAndViewpointRolesAreSpecified;
            verify requirement rules :> XDesignRequirements::checkerRulesAreImplementationReady;
        }
    }

    verification def RustUmlDesignReview {
        @XDesignCommon::DocSection { document = "docs/design/flows/simple-flow/views.md"; section = "D52"; }
        doc /* This case specifies acceptance of the Rust UML design; its
           declaration is not an executed result. For every designed SimpleFlow
           element, find the Rust element that realizes it and read the recorded
           mapping; for every UML view of the guide, read the stated applicability
           and reason, open each adopted view's figure and read its block and
           status; then read the design document for any claim of code, tests or
           builds the repository does not contain. */
        subject crate : XDesignCrate;
        objective umlDesign {
            verify requirement mapping :> XDesignRequirements::rustUmlDesignMapsEveryDesignElement;
            verify requirement applicability :> XDesignRequirements::umlViewsAreSelectedWithReasons;
            verify requirement honesty :> XDesignRequirements::softwareDesignClaimsNoImplementation;
        }
    }

    verification def SimpleFlowGateBehaviorVerification {
        @XDesignCommon::DocSection { document = "docs/verification/design.md"; section = "D54"; }
        doc /* Planned executable cases for the designed SimpleFlow. Load a valid
           four-stage plan with one syntactic gate on each of the first three
           stages and both a syntactic and a semantic gate on verify. Reject a
           plan with an absent, duplicate or foreign exit gate. Evaluate each
           stage and assert that only its own exit gates receive assessments;
           verify must receive two assessments, one per kind. A failing rule,
           missing report, malformed rule list, or unpermitted judge must never
           produce a passed assessment. No result is claimed by this declaration. */
        subject run : SimpleFlowRun;
        objective gateBehavior {
            verify requirement stageGates :> XDesignRequirements::designedFlowGatesEveryStage;
            verify requirement scriptAssessment :> XDesignRequirements::syntacticAssessmentIsScripted;
            verify requirement judgeAuthority :> XDesignRequirements::semanticCheckNamesItsJudges;
        }
    }

    verification def SimpleFlowRecoveryVerification {
        @XDesignCommon::DocSection { document = "docs/verification/design.md"; section = "D54"; }
        doc /* Planned executable lifecycle cases. Creation starts all visits at
           artifact revision 0 and visit index 0. Recording a content change,
           including an in-place edit at the same URI, increments the current
           revision. A pass followed by a failure or unevaluated assessment of
           the same gate at one revision is unauthorized because the latest
           assessment governs. Failed advancement retains the blocked visit;
           correction and reevaluation occur there. An explicit reopen of a
           completed terminal run appends visit index 1 at revision 0 and keeps
           the prior visit and evidence; reopening an active or nonterminal
           stage is rejected. The case requires runtime evidence, not a compiler
           pass. */
        subject run : SimpleFlowRun;
        objective lifecycle {
            verify requirement designedRun :> XDesignRequirements::flowRunIsRealizedByDesign;
            verify requirement recordCheck :> XDesignRequirements::runRecordIsScriptChecked;
        }
    }

    verification def SimpleFlowFilePlanVerification {
        @XDesignCommon::DocSection { document = "docs/verification/design.md"; section = "D54"; }
        doc /* Planned file-level conformance case. Compare every path in
           simpleFlowFilePlan with the repository and every in-scope source and
           test file back to the plan, then compare the Rust module exports,
           four declared checker command URIs and their rule IDs. Require the
           pinned SysML parse/check, docs
           audit, figure regeneration and strict site build to pass. Retain all
           reports under cache/. A file's presence alone does not establish its
           behavior; run the gate and recovery cases separately. */
        objective fileCoverage {
            verify requirement mapping :> XDesignRequirements::rustUmlDesignMapsEveryDesignElement;
            verify requirement runnableGates :> XDesignRequirements::gateSyntaxIsRunnable;
            verify requirement declaredRules :> XDesignRequirements::checkerRulesAreDeclared;
        }
    }

    verification def WishVerifyFlowBehaviorVerification {
        @XDesignCommon::DocSection { document = "docs/records/plans/wish-verify.md"; section = "P28"; }
        doc /* Planned executable cases for the designed wish-verify flow. Load a
           valid two-stage plan whose wish and verify exits are both semantic
           gates. Reject a plan with an absent, duplicate or foreign exit gate
           or a syntactic gate naming a checker. Evaluate each stage and assert
           that only its own exit gate receives an assessment. An unpermitted
           judge, an empty rationale or missing evidence must never produce a
           passed assessment. No result is claimed by this declaration. */
        subject run : WishVerifyFlowRun;
        objective gateBehavior {
            verify requirement stageGates :> XDesignRequirements::designedFlowGatesEveryStage;
            verify requirement judgeAuthority :> XDesignRequirements::semanticCheckNamesItsJudges;
        }
    }

    verification def WishVerifyFlowFilePlanVerification {
        @XDesignCommon::DocSection { document = "docs/records/plans/wish-verify.md"; section = "P28"; }
        doc /* Planned file-level conformance case. Compare every path in
           wishVerifyFlowFilePlan with the repository and back, and compare the
           wishVerify Rust module's elements with the design elements they
           realize. Require the pinned SysML parse/check, the documentation
           audit, figure regeneration and the strict site build to pass, with
           reports retained under cache/. The documentation set gains a second
           developed flow without renumbering an existing section. A file's
           presence alone does not establish its behavior; run the behavior
           cases separately. */
        objective fileCoverage {
            verify requirement mapping :> XDesignRequirements::rustUmlDesignMapsEveryDesignElement;
            verify requirement flows :> XDesignRequirements::documentationSupportsMultipleFlows;
        }
    }

    verification def LocalStageCliRuntimeVerification {
        @XDesignCommon::DocSection { document = "docs/design/stage-cli.md"; section = "D85"; }
        doc /* Executable cases cover both built-in flows across independent
           processes; read-only repeated next and completed calls; fixed gate
           success and semantic refusal/recovery; stale envelopes and changed
           artifact judgment digests; corrupt records and plan/work drift;
           exclusive writer refusal and unsafe paths. See the implementation
           plan for revision-bound executed evidence; this declaration is not a pass. */
        subject cli : XDesignWorkflow::LocalStageCliRuntime;
        objective interaction {
            verify requirement uniform :> XDesignRequirements::stageCliUsesUniformInteraction;
            verify requirement integrity :> XDesignRequirements::stageCliPreservesRunIntegrity;
        }
    }

    verification def StorageVerification {
        @XDesignCommon::DocSection { document = "docs/verification/design.md"; section = "D87"; }
        doc /* Planned storage verification, not an executed result. Exercise format
           refusal, generic revision queries, content integrity, exact-operation retry,
           competing writers, injected crash boundaries, doctor findings, both flat
           flows, flow-derived status, cache-loss recovery and checkout/replay.
           STORE-01 through STORE-10 in the storage plan name the observable cases. */
        subject store : XDesignStorage::ProjectStore;
        objective storageBehavior {
            verify requirement authority :> XDesignRequirements::operationalRecordsHaveDurableAuthority;
            verify requirement revisions :> XDesignRequirements::wishHistoryIsRevisioned;
            verify requirement materials :> XDesignRequirements::materialsAreContentAddressed;
            verify requirement atomicSave :> XDesignRequirements::storageSaveIsAtomicAndIdempotent;
            verify requirement cacheRecovery :> XDesignRequirements::workingCacheIsRecoverable;
            verify requirement statusProjection :> XDesignRequirements::wishStatusProjectsFlowExecution;
            verify requirement integrity :> XDesignRequirements::storageInputAndIntegrityAreChecked;
        }
    }
}
