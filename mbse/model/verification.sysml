package XDesignVerification {
    @XDesignCommon::DiagramPresentation { label = "Verification"; }
    @XDesignCommon::DocSection { document = "docs/requirements/01_baseline.md"; section = "D15"; }
    private import XDesignDetailedDesign::*;
    private import XDesignRustUml::*;

    doc /* Verification designs, separate from executed results. Declaring a case
       and its verify relationships specifies coverage; run evidence belongs to the
       independent trace record, and compiler checks do not execute this review. */

    verification def SimpleFlowViewReview {
        @XDesignCommon::DiagramPresentation { label = "Design review"; summary = "Planned coverage review"; }
        @XDesignCommon::DocSection { document = "docs/requirements/01_baseline.md"; section = "D15"; }
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
        @XDesignCommon::DocSection { document = "docs/details/04_diagram_presentation.md"; section = "D25"; }
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
        @XDesignCommon::DocSection { document = "docs/review/01_wish_status.md"; section = "V07"; }
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
        @XDesignCommon::DocSection { document = "docs/details/simple_flow/06_rust_uml_design.md"; section = "D52"; }
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
        @XDesignCommon::DocSection { document = "docs/prj_manage/06_file_plan_and_verification.md"; section = "D54"; }
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
        @XDesignCommon::DocSection { document = "docs/prj_manage/06_file_plan_and_verification.md"; section = "D54"; }
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
        @XDesignCommon::DocSection { document = "docs/prj_manage/06_file_plan_and_verification.md"; section = "D54"; }
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
}
