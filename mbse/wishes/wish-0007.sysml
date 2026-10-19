package Wish0007 {
    @XDesignCommon::DocSection { document = "docs/review/01_wish_status.md"; section = "V06"; }
    private import ScalarValues::*;
    private import XDesignCommon::*;
    private import XDesignArchitecture::*;
    private import XDesignWorkflow::*;
    private import StandardViewDefinitions::*;
    private import Views::*;

    doc /* The independent record for wish-0007: make the nine primary SimpleFlow
       diagrams readable. Earlier coverage and compiler checks established that
       figures existed, but did not establish usable composition or typography.
       This is a new implementation wish on the default four-stage SimpleFlow.
       Earlier wishes and their historical assessments remain unchanged.

       Requirements and renderer design stay in the project model. This record
       describes their derivation, implementation work and revision-specific
       evidence. The default flow has no exit checkpoint on its first three
       stages; the final semantic gate requires evidence that the implementation
       satisfies the requirements. Passing a compiler alone cannot authorize it. */

    item wish0007 : Wish {
        attribute redefines wishId = "wish-0007";
        attribute redefines description = "Make the nine primary SimpleFlow diagrams more visible and understandable, using the supplied engineering course PDFs as composition references. Replace the unreadable rendering layout, retain SysML source identity, document the presentation pipeline, and judge the final figures at their normal documentation display size.";
    }

    dependency producedReadableViews from wish0007 to XDesignRequirements::diagramViewsAreReadable;
    dependency producedSourceIdentity from wish0007 to XDesignRequirements::diagramPresentationRetainsSourceIdentity;

    part trace0007 : Trace {
        ref item redefines wish = wish0007;
        attribute redefines status = WishStatus::delivered;
        attribute redefines traceabilityModelUri = "mbse/wishes/wish-0007.sysml";

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

                item defineReadability : ActivityRecord subsets activities {
                    attribute redefines description = "Derived two project requirements: readable composition of the nine primary SimpleFlow diagrams at a 900-pixel article width, and native source identity with reproducible final-figure evidence. Readability requires at least 11-pixel displayed text, meaningful diagram notation, identifiable endpoints and visual review of the final SVGs and corresponding previews. Full-model membership trees remain expandable reference overviews.";
                    attribute redefines performedBy = "codex-agent";
                    attribute redefines occurredAt = "2026-10-19T03:44:10Z";
                    attribute redefines artifactUris = ("mbse/model/requirements.sysml", "docs/requirements/00_baseline.md");
                }
            }

            part designRecord : StageRecord subsets stageRecords {
                ref part redefines stage = workflow.'flow'.design;
                attribute redefines visitIndex = 0;
                attribute redefines status = StageStatus::completed;
                attribute redefines artifactRevision = 1;

                item designPresentation : ActivityRecord subsets activities {
                    attribute redefines description = "Specified DiagramPresentation metadata and DiagramDocumentationRenderer: export native model IDs and views, project selected elements using diagram-specific notation, lay out focused graphs with Graphviz, render source-linked SVGs, then validate the final artifact and matching previews. Added planned DiagramPresentationReview and satisfy/verify relationships. Friendly labels change presentation without renaming model elements or fabricating relationships.";
                    attribute redefines performedBy = "codex-agent";
                    attribute redefines occurredAt = "2026-10-19T03:44:10Z";
                    attribute redefines artifactUris = ("mbse/model/common.sysml", "mbse/model/detailedDesign.sysml", "mbse/model/verification.sysml", "mbse/model/traceability.sysml", "docs/details/03_diagram_presentation.md");
                }
            }

            part implementationRecord : StageRecord subsets stageRecords {
                ref part redefines stage = workflow.'flow'.implement;
                attribute redefines visitIndex = 0;
                attribute redefines status = StageStatus::completed;
                attribute redefines artifactRevision = 1;

                item implementPresentation : ActivityRecord subsets activities {
                    attribute redefines description = "Implemented focused diagram composition and Graphviz-based layout, model-owned presentation metadata, native source references and hash-bound final-figure verification. Regenerated the nine primary figures with diagram-specific boundaries, ports, parameter bindings, actor goals, control guards and ordered sequence messages. Removed one unintended duplicate checkpoint-control succession. Added regression tests and updated the documented rendering and article-display process. The documentation utility is implemented; the designed workflow application remains a separate scope.";
                    attribute redefines performedBy = "codex-agent";
                    attribute redefines occurredAt = "2026-10-19T03:44:10Z";
                    attribute redefines artifactUris = ("tools/render_docs.py", "tools/sysml_web/renderer.py", "tools/sysml_web/documentation_views.py", "tools/sysml_web/diagram_layout.py", "tools/verify_diagrams.py", "tools/sysml_web/tests/test_documentation_views.py", "tools/sysml_web/tests/test_diagram_layout.py", "tools/sysml_web/tests/test_verify_diagrams.py", "tools/docs_site/theme/javascripts/diagrams.js", "tools/docs_site/theme/stylesheets/xdesign.css", "docs/details/simple_flow/03_sysml_diagrams.md", "docs/details/03_diagram_presentation.md");
                }
            }

            part verifyRecord : StageRecord subsets stageRecords {
                ref part redefines stage = workflow.'flow'.'verify';
                attribute redefines visitIndex = 0;
                attribute redefines status = StageStatus::completed;
                attribute redefines artifactRevision = 1;

                item verificationActivity : ActivityRecord subsets activities {
                    attribute redefines description = "Verified revision 1 against the two presentation requirements and DiagramPresentationReview. The pinned compiler parsed and checked the complete project and wish models, and the project model checked independently. All 52 renderer and final-asset tests passed, all 27 documentation figures regenerated, and strict documentation build passed. Inspected all nine final 900-pixel previews and the actual built article; checked native relationships, source identity, readable text, boundaries, ports, bindings, guards and message order. Bound the accepted SVGs, corresponding PNG previews, source bundle and built-article screenshots through the final manifest and browser review. The final evidence refresh after this record's closure must retain the reviewed figure contents and reconcile final source hashes.";
                    attribute redefines performedBy = "codex-agent";
                    attribute redefines occurredAt = "2026-10-19T03:56:42Z";
                    attribute redefines artifactUris = ("cache/evidence/wish-0007/parse.json", "cache/evidence/wish-0007/check.json", "cache/evidence/wish-0007/model-check.json", "cache/evidence/wish-0007/renderer-tests.log", "cache/evidence/wish-0007/render.log", "cache/evidence/wish-0007/docs-build.log", "cache/evidence/wish-0007/visual-review.md", "cache/evidence/wish-0007/figure-manifest.json", "cache/evidence/wish-0007/browser-review.json");
                }

                item redefines assessments : CheckpointAssessment [1];
                ref item redefines currentAssessments = presentationAssessment;

                item presentationAssessment : CheckpointAssessment subsets assessments {
                    ref part redefines checkpoint = workflow.'flow'.verificationAccepted;
                    attribute redefines evaluatedRevision = 1;
                    attribute redefines outcome = CheckpointOutcome::passed;
                    attribute redefines judgedBy = JudgeAuthority::agent;
                    attribute redefines assessor = "codex-agent";
                    attribute redefines rationale = "Revision 1 satisfies the readable-diagram and retained-source-identity requirements for all nine primary SimpleFlow figures. Focused model-owned labels and diagram-specific notation replace the former fixed-grid detail dump. Final previews and the built article show legible text, clear boundaries and endpoints, attached ports and parameter terminals, understandable guarded paths, all six lifelines and ten ordered messages, and all seven packages with eleven native imports. The smallest primary-figure text at a 900-pixel article width is approximately 12.11 pixels, exceeding the 11-pixel requirement; browser geometry reports no text collisions or clipping, and served SVG hashes match the reviewed source figures. The final manifest binds SVG and preview bytes to their source bundle and browser evidence. All 52 tests, 27 figure renders, strict documentation build and supported compiler checks pass; the compiler retains 127 held and 26 unevaluated rules. This is the agent's semantic judgment of the final presentation, supported by the evidence, rather than an inference from asset existence or compiler success. Whole-model reference overviews, smaller-screen typography and execution of workflow application behavior or SysML constraint bodies remain outside this presentation acceptance. Historical assessments of earlier wishes remain unchanged.";
                    attribute redefines evidenceUris = ("cache/evidence/wish-0007/visual-review.md", "cache/evidence/wish-0007/figure-manifest.json", "cache/evidence/wish-0007/browser-review.json", "cache/evidence/wish-0007/renderer-tests.log", "cache/evidence/wish-0007/render.log", "cache/evidence/wish-0007/parse.json", "cache/evidence/wish-0007/check.json", "cache/evidence/wish-0007/model-check.json", "cache/evidence/wish-0007/docs-build.log", "docs/details/03_diagram_presentation.md", "docs/details/simple_flow/03_sysml_diagrams.md");
                    ref item redefines activities = verificationActivity;
                }
            }
        }
    }

    occurrence wish0007Journey {
        doc /* This run's journey over the real default-flow participants.
           Requirements, design and implementation have no exit gate in this
           flow. Revision 1 completed all four stages; its final semantic gate
           judged the rendered figures and documentation against retained
           evidence. This is an occurrence usage, not a new type. */
        ref part controller : Controller = workflow.controller;
        ref part requirements : Stage = workflow.'flow'.requirements;
        ref part design : Stage = workflow.'flow'.design;
        ref part implement : Stage = workflow.'flow'.implement;
        ref part 'verify' : Stage = workflow.'flow'.'verify';
        ref part trace : Trace = trace0007;

        message enterRequirements from controller.entering to requirements.entered;
        message enterDesign from controller.entering to design.entered;
        message enterImplementation from controller.entering to implement.entered;
        message enterVerification from controller.entering to 'verify'.entered;
        message assessPresentation from controller.assessing to 'verify'.assessed;
        message deliverReadableDiagrams from controller.delivering to trace.delivered;
    }

    view wish0007Sequence : SequenceView {
        doc /* Event-anchored progression on the real participants. The trace's
           revision-specific assessment and evidence establish delivery. */
        expose Wish0007::wish0007Journey;
    }

    view wish0007Run : GeneralView {
        doc /* The delivered readability correction, its default-flow run and
           four completed stage records with revision-specific review evidence. */
        expose Wish0007::wish0007;
        expose Wish0007::trace0007;
        expose Wish0007::trace0007::execution;
        render asInterconnectionDiagram;
    }
}
