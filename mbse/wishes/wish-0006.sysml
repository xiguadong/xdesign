package Wish0006 {
    @XDesignCommon::DocSection { document = "docs/review/01_wish_status.md"; section = "V05"; }
    private import ScalarValues::*;
    private import XDesignCommon::*;
    private import XDesignArchitecture::*;
    private import XDesignWorkflow::*;
    private import XDesignDetailedDesign::*;
    private import StandardViewDefinitions::*;
    private import Views::*;

    doc /* The independent traceability model for wish-0006: the detailed design of the
       simple flow — the class of a run, the factory that creates it, the functions that
       evaluate its gates and advance it, the unified checkpoint class with its syntactic
       and semantic adaptations and its evaluation loop, and the documentation details.

       This record is design-only. Visit 0 produced and accepted the original design,
       including three structure graphs. That delivery did not cover the nine SysML
       diagram types in the supplied engineering course PDFs. Visit 1 corrected that
       omission at artifact revision 2 with nine model views, their rendered figures and
       documentation, then passed its own syntactic and semantic assessments. The earlier
       passed assessments remain the evidence for revision 1; they do not authorize the
       correction. No implementation stage was entered and no implementation language
       has been selected.

       The checker commands that the designed simple flow's gates will use are not part of
       the project yet: the design stage keeps their rules as documentation (D06) and the
       checker declarations, commands and rule lists arrive at implementation. Earlier
       rounds of this wish that wrote Python checker prototypes are research conclusions,
       not project implementation; this record supersedes them. */

    item wish0006 : Wish {
        attribute redefines wishId = "wish-0006";
        attribute redefines description = "Design the simple flow down to what a run of it is made of: the class, the factory and the functions; and define its checkpoints exactly in semantics and syntax — the syntactic ones backed by checks that exist — with the details written up under docs/ and the sections carried from the model. Include all nine SysML diagram types identified in the supplied engineering course PDFs as meaningful model views and documented figures.";
    }

    doc /* The requirements this wish produced. They are retained in XDesignRequirements,
       where they stand on their own; this file records that this wish derived them. The
       dependency runs one way, from the wish to the model: the model never names a wish. */
    dependency producedFlowRunIsRealizedByDesign from wish0006 to XDesignRequirements::flowRunIsRealizedByDesign;
    dependency producedDesignedFlowGatesEveryStage from wish0006 to XDesignRequirements::designedFlowGatesEveryStage;
    dependency producedGateSyntaxIsRunnable from wish0006 to XDesignRequirements::gateSyntaxIsRunnable;
    dependency producedRunRecordIsScriptChecked from wish0006 to XDesignRequirements::runRecordIsScriptChecked;
    dependency producedDesignCarriesItsDocSections from wish0006 to XDesignRequirements::designCarriesItsDocSections;
    dependency producedCheckerRulesAreDeclared from wish0006 to XDesignRequirements::checkerRulesAreDeclared;
    dependency producedStructureGraphsAreDocumented from wish0006 to XDesignRequirements::structureGraphsAreDocumented;
    dependency producedRunRealizesItsDesign from wish0006 to XDesignRequirements::runRealizesItsDesign;

    part trace0006 : Trace {
        doc /* One design-stage run on designFlow with two visits per stage. Visit 0
           delivered revision 1 of the detailed design and its three structure graphs.
           Visit 1 corrects the missing nine-diagram coverage at revision 2, with passed
           syntactic and semantic assessments that judge that revision independently of
           the earlier delivery. No implementation was started. */
        ref item redefines wish = wish0006;
        attribute redefines status = WishStatus::delivered;
        attribute redefines traceabilityModelUri = "mbse/wishes/wish-0006.sysml";

        part redefines execution {
            ref part redefines assignedFlow = designFlow;
            attribute redefines status = FlowStatus::completed;
            part redefines stageRecords : StageRecord [4];
            ref part redefines currentRecord = verifyCorrectionRecord;

            part wishRecord : StageRecord subsets stageRecords {
                ref part redefines stage = designFlow.wish;
                attribute redefines visitIndex = 0;
                attribute redefines status = StageStatus::completed;
                attribute redefines artifactRevision = 1;

                item wishActivity : ActivityRecord subsets activities {
                    attribute redefines description = "Derived eight requirements for the simple flow's detailed design (a run realized by design elements, every stage gated, gate syntax backed by checks that exist, the rules declared and documented, a record check, documentation sections on every design element, the structure graphs, and the run typed by the design's class), retained them in the project model with their rationales and satisfiers. Produced the detailed design: the run class, the factory, the gate and advance functions, the unified checkpoint class with its syntactic and semantic adaptations and its evaluation loop, gate types and views. Wrote the documentation under docs/, including the rules documentation for the four planned checks; no implementation code or language is assumed.";
                    attribute redefines performedBy = "zcode-agent";
                    attribute redefines occurredAt = "2026-10-18T04:30:00Z";
                    attribute redefines artifactUris = ("mbse/model/requirements.sysml", "mbse/model/traceability.sysml", "mbse/model/detailedDesign.sysml", "mbse/model/architecture.sysml", "mbse/model/common.sysml", "docs/details/simple_flow/00_run_design.md", "docs/details/simple_flow/01_gates_and_rules.md", "docs/details/simple_flow/02_structure_graphs.md", "docs/requirements/00_baseline.md", "docs/arch/00_overview.md", "docs/arch/01_system_composition.md", "docs/details/00_flows_and_checkpoints.md", "docs/details/01_records_and_evidence.md", "docs/details/02_toolchain.md", "docs/review/00_wish_status.md", "mkdocs.yml");
                }
            }

            part verifyRecord : StageRecord subsets stageRecords {
                ref part redefines stage = designFlow.'verify';
                attribute redefines visitIndex = 0;
                attribute redefines status = StageStatus::completed;
                attribute redefines artifactRevision = 1;

                item verificationActivity : ActivityRecord subsets activities {
                    attribute redefines description = "Ran the pinned SysML compiler on the delivered model (parse and check) and read the design against the wish: the requirements and design classes, the documentation details, the unified checkpoint class and its loop, and the rules documented for implementation. Recorded the compiler report as the syntactic evidence and the reading as the semantic judgment.";
                    attribute redefines performedBy = "zcode-agent";
                    attribute redefines occurredAt = "2026-10-18T05:00:00Z";
                    attribute redefines artifactUris = ("cache/sysml/parse.json", "cache/sysml/check.json", "cache/evidence/wish-0006/design-deliverable-check.md");
                }

                item redefines assessments : CheckpointAssessment [2];
                ref item redefines currentAssessments = (sysmlAssessment, meaningAssessment);

                item sysmlAssessment : CheckpointAssessment subsets assessments {
                    ref part redefines checkpoint = designFlow.sysmlDeliverableAccepted;
                    attribute redefines evaluatedRevision = 1;
                    attribute redefines outcome = CheckpointOutcome::passed;
                    attribute redefines judgedBy = JudgeAuthority::script;
                    attribute redefines assessor = "sysml-compiler";
                    attribute redefines rationale = "The delivered SysML parses and checks: 12 files, no parse errors; 73,638 elements, 19,248/19,248 references resolved, 101 rules held, 23 unevaluated, no unresolved names, collisions or violations.";
                    attribute redefines evidenceUris = ("cache/sysml/parse.json", "cache/sysml/check.json");
                }

                item meaningAssessment : CheckpointAssessment subsets assessments {
                    ref part redefines checkpoint = designFlow.outcomeAccepted;
                    attribute redefines evaluatedRevision = 1;
                    attribute redefines outcome = CheckpointOutcome::passed;
                    attribute redefines judgedBy = JudgeAuthority::agent;
                    attribute redefines assessor = "zcode-agent";
                    attribute redefines rationale = "The design answers the wish. flowRunIsRealizedByDesign: the run class, the factory and both functions are declared. designedFlowGatesEveryStage: every stage of designedSimpleFlow redefines its exit checkpoints, and every gate is an instance of the unified checkpoint class. gateSyntaxIsRunnable: the design delivers SysML, and the design-stage run's own syntactic checkpoint is the pinned compiler whose command exists. checkerRulesAreDeclared: the rules that will decide the four designed checks are documented in D06 and the checker declarations, commands and rule lists are deferred to implementation. runRecordIsScriptChecked: the record gate is designed, its checker is not implemented; this is stated, not glossed. designCarriesItsDocSections: every top-level design element carries a section and every target resolves; verified by reading at the design stage. structureGraphsAreDocumented: the three graphs are declared, drawn, located by DocFigure and carried by D08-D10. runRealizesItsDesign: the designed flow's runs are typed by SimpleFlowRun in the design; no implementation run exists yet. What no script establishes, this reading did: the class/adaptation/operation loop, the criteria strings against the model, and the split between what is designed and what implementation must supply.";
                    attribute redefines evidenceUris = ("cache/evidence/wish-0006/design-deliverable-check.md", "docs/details/simple_flow/00_run_design.md", "docs/details/simple_flow/01_gates_and_rules.md", "docs/details/simple_flow/02_structure_graphs.md", "mbse/model/detailedDesign.sysml");
                }
            }

            part wishCorrectionRecord : StageRecord subsets stageRecords {
                ref part redefines stage = designFlow.wish;
                attribute redefines visitIndex = 1;
                attribute redefines status = StageStatus::completed;
                attribute redefines artifactRevision = 2;

                item correctionRequested : ActivityRecord subsets activities {
                    attribute redefines description = "Reopened the design-only wish because the original three structure graphs did not cover the nine SysML diagram types identified in the supplied engineering course PDFs. Revision 2 adds meaningful model views, rendered figures and explanations for the block definition, internal block, requirements, parametric, use case, activity, sequence, state machine and package diagrams identified on page 14 of 02_modeling_and_design.pdf; the original visit and its passed assessments remain unchanged.";
                    attribute redefines performedBy = "codex-agent";
                    attribute redefines occurredAt = "2026-10-19T02:56:27Z";
                    attribute redefines artifactUris = ("mbse/model/detailedDesign.sysml", "docs/details/simple_flow/03_sysml_diagrams.md", "docs/review/00_wish_status.md");
                }

                item correctionProduction : ActivityRecord subsets activities {
                    attribute redefines description = "Added nine coordinated SysML views over the simple flow. Added the constraint and bound authorization context for the parametric view, external actor goals for the use-case view, the run lifecycle with guarded blocking and rework, and a planned verification case with requirement references. Wrote D12-D21 with the PDF source pages, each view's selected elements and engineering purpose, its implementation limits and the stage handoffs. Retained the earlier three structure graphs.";
                    attribute redefines performedBy = "codex-agent";
                    attribute redefines occurredAt = "2026-10-19T03:01:47Z";
                    attribute redefines artifactUris = ("mbse/model/detailedDesign.sysml", "mbse/model/verification.sysml", "mbse/model/requirements.sysml", "mbse/model/common.sysml", "docs/details/simple_flow/03_sysml_diagrams.md", "tools/render_docs.py", "mkdocs.yml");
                }
            }

            part verifyCorrectionRecord : StageRecord subsets stageRecords {
                ref part redefines stage = designFlow.'verify';
                attribute redefines visitIndex = 1;
                attribute redefines status = StageStatus::completed;
                attribute redefines artifactRevision = 2;

                item correctionVerification : ActivityRecord subsets activities {
                    attribute redefines description = "Ran the pinned compiler on revision 2, including parse/check of the project and wish models and an independent check of the project model. Regenerated the documentation figures; passed the existing renderer suite and the focused diagram regressions; built the documentation in strict mode. Compared ten authorization scenarios with the reviewed completion predicate, inventoried the documentation targets and nine view mappings, and reviewed the model, figures and D12-D21 against the source PDFs. Retained separate syntactic evidence and semantic judgment for revision 2.";
                    attribute redefines performedBy = "codex-agent";
                    attribute redefines occurredAt = "2026-10-19T03:06:06Z";
                    attribute redefines artifactUris = ("cache/evidence/wish-0006/nine-diagrams/parse.json", "cache/evidence/wish-0006/nine-diagrams/check.json", "cache/evidence/wish-0006/nine-diagrams/model-check.json", "cache/evidence/wish-0006/nine-diagrams/render.log", "cache/evidence/wish-0006/nine-diagrams/renderer-tests.log", "cache/evidence/wish-0006/nine-diagrams/documentation-tests.log", "cache/evidence/wish-0006/nine-diagrams/docs-build.log", "cache/evidence/wish-0006/nine-diagrams/authorization-cases.json", "cache/evidence/wish-0006/nine-diagrams/inventory.json", "cache/evidence/wish-0006/nine-diagrams/review.md", "docs/details/simple_flow/03_sysml_diagrams.md");
                }

                item redefines assessments : CheckpointAssessment [2];
                ref item redefines currentAssessments = (correctionSysmlAssessment, correctionMeaningAssessment);

                item correctionSysmlAssessment : CheckpointAssessment subsets assessments {
                    ref part redefines checkpoint = designFlow.sysmlDeliverableAccepted;
                    attribute redefines evaluatedRevision = 2;
                    attribute redefines outcome = CheckpointOutcome::passed;
                    attribute redefines judgedBy = JudgeAuthority::script;
                    attribute redefines assessor = "sysml-compiler";
                    attribute redefines rationale = "The pinned compiler parsed revision 2 and checked the project and wish models: all references resolved, with no parse errors, unresolved names, collisions or violations. The project model also checks independently of wish records. The retained reports preserve held and unevaluated rule counts; they do not establish execution of constraint bodies or application behavior.";
                    attribute redefines evidenceUris = ("cache/evidence/wish-0006/nine-diagrams/parse.json", "cache/evidence/wish-0006/nine-diagrams/check.json", "cache/evidence/wish-0006/nine-diagrams/model-check.json");
                }

                item correctionMeaningAssessment : CheckpointAssessment subsets assessments {
                    ref part redefines checkpoint = designFlow.outcomeAccepted;
                    attribute redefines evaluatedRevision = 2;
                    attribute redefines outcome = CheckpointOutcome::passed;
                    attribute redefines judgedBy = JudgeAuthority::agent;
                    attribute redefines assessor = "codex-agent";
                    attribute redefines rationale = "Revision 2 completes the missing diagram scope from the source PDFs. All nine named views select meaningful SysML elements and relationships and have documented figures: types and ownership; connected internal parts; requirements with satisfaction and planned verification; parameter bindings and authorization; external user goals; guarded checkpoint activity; event-anchored messages; a lifecycle with blocking and rework; and project-package dependencies. D12-D21 trace these views to source pages and explain their engineering roles and stage handoffs. Documentation targets and nine figure mappings resolve, the figure renders and renderer regressions pass, and strict documentation builds. The finite authorization examples agree with the reviewed predicate without claiming to execute SysML expressions. Compiler export omissions and unevaluated rules remain disclosed, as do the planned checker commands and application behavior still awaiting implementation.";
                    attribute redefines evidenceUris = ("cache/evidence/wish-0006/nine-diagrams/review.md", "cache/evidence/wish-0006/nine-diagrams/inventory.json", "cache/evidence/wish-0006/nine-diagrams/render.log", "cache/evidence/wish-0006/nine-diagrams/renderer-tests.log", "cache/evidence/wish-0006/nine-diagrams/documentation-tests.log", "cache/evidence/wish-0006/nine-diagrams/docs-build.log", "cache/evidence/wish-0006/nine-diagrams/authorization-cases.json", "docs/details/simple_flow/03_sysml_diagrams.md", "mbse/model/detailedDesign.sysml", "mbse/model/verification.sysml");
                }
            }
        }
    }

    occurrence def Wish0006Journey {
        doc /* This wish's design-stage journey on designFlow, including the corrective
           visit for the nine SysML diagram types. The revision-1 delivery is historical;
           the revision-2 correction received its own syntactic and semantic judgments.
           There is no implementation lifeline. */
        ref part controller : Controller = workflow.controller;
        ref part wish : Stage = designFlow.wish;
        ref part 'verify' : Stage = designFlow.'verify';
        ref part trace : Trace = trace0006;

        message enterWish from controller.entering to wish.entered;
        message assessDeliveredSysml from controller.assessing to 'verify'.assessed;
        message assessDesignMeaning from controller.assessing to 'verify'.assessed;
        message deliverWish from controller.delivering to trace.delivered;
        message reopenWishForNineDiagrams from controller.entering to wish.entered;
        message assessNineDiagramSysml from controller.assessing to 'verify'.assessed;
        message assessNineDiagramMeaning from controller.assessing to 'verify'.assessed;
        message deliverNineDiagramCorrection from controller.delivering to trace.delivered;
    }

    view wish0006Run : GeneralView {
        doc /* The design-stage run's records, and the wish this design answers. */

        expose Wish0006::wish0006;
        expose Wish0006::trace0006;
        expose Wish0006::trace0006::execution;
        render asInterconnectionDiagram;
    }
}
