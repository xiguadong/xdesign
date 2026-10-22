package Wish0009 {
    @XDesignCommon::DocSection { document = "docs/records/wishes/wish-history.md"; section = "V16"; }
    private import ScalarValues::*;
    private import XDesignCommon::*;
    private import XDesignArchitecture::*;
    private import XDesignWorkflow::*;
    private import XDesignComposition::*;
    private import StandardViewDefinitions::*;
    private import Views::*;

    doc /* The independent traceability model for wish-0009: a Rust UML design of the
       designed simple flow, produced as a judged artifact on xdesign.wishVerifyFlow —
       requirements where they apply, the SimpleFlowRustUml design model, the UML
       guide and the module's UML design document, without code implementation.

       The wish was registered on xdesign.wishVerifyFlow: the outcome is a software
       design document set whose acceptance is a reading, and no fixed check can
       establish that the mapping covers the design or that the status claims are
       honest. The requirements, design model, verification case and documentation
       this wish produced are retained in mbse/model and docs/, not here. */

    item wish0009 : Wish {
        attribute redefines wishId = "wish-0009";
        attribute redefines description = "Produce a Rust UML design for the designed simple flow (the future simpflow crate), using WishVerifyFlow: derive requirements where they apply, express the design in SysML at mbse/model/uml.sysml, and document it — a UML guide following cache/templates/docs/uml/, the module's UML design document with embedded rendered views and a stated view applicability, and the related docs/AGENTS.md conventions. No code implementation.";
    }

    dependency producedUmlMappingRequirement from wish0009 to XDesignRequirements::rustUmlDesignMapsEveryDesignElement;
    dependency producedUmlApplicabilityRequirement from wish0009 to XDesignRequirements::umlViewsAreSelectedWithReasons;
    dependency producedUmlHonestyRequirement from wish0009 to XDesignRequirements::softwareDesignClaimsNoImplementation;

    part trace0009 : Trace {
        doc /* Three visits to each wishVerifyFlow stage. Visit 0 records the
           revision-1 module-level delivery. Visit 1 corrects the granularity at
           revision 2 with the API-reference structure: complete inventory plus one
           page per API. Visit 2 closes the wish at revision 3 by writing the
           API-reference structure into the UML guide and repository conventions,
           so later module designs follow it by default. Earlier assessments are
           retained as historical evidence of their own artifact revisions. */

        ref item redefines wish = wish0009;
        attribute redefines status = WishStatus::delivered;
        attribute redefines traceabilityModelUri = "mbse/wishes/wish-0009.sysml";

        part redefines execution : FlowExecution {
            ref part redefines assignedFlow = xdesign.wishVerifyFlow;
            attribute redefines status = FlowStatus::completed;
            part redefines stageRecords : StageRecord [6];
            ref part redefines currentRecord = verifyGuideRecord;

            part wishRecord : StageRecord subsets stageRecords {
                ref part redefines stage = xdesign.wishVerifyFlow.wish;
                attribute redefines visitIndex = 0;
                attribute redefines status = StageStatus::completed;
                attribute redefines artifactRevision = 1;

                item deriveRequirements : ActivityRecord subsets activities {
                    attribute redefines description = "Derived three requirements with rationales: the Rust UML design must name one Rust element per designed-flow design element and record the realization mapping; every UML view of the guide must carry stated applicability with reasons; and a software design must claim no implementation the repository lacks. Added the realization satisfy link, noted the two documentation-product requirements without model satisfiers, and declared the planned RustUmlDesignReview case.";
                    attribute redefines performedBy = "kimi-agent";
                    attribute redefines occurredAt = "2026-10-19T09:50:00Z";
                    attribute redefines artifactUris = ("mbse/model/requirements.sysml", "mbse/model/traceability.sysml", "mbse/model/verification.sysml", "docs/requirements/01_baseline.md");
                }
                item produceUmlModel : ActivityRecord subsets activities {
                    attribute redefines description = "Wrote mbse/model/uml.sysml (package SimpleFlowRustUml): the simpflow crate's seven-module layout with allowed dependency directions; the status enums mirroring the model vocabulary plus RsAdvanceOutcome; the record and plan structs, the CheckCommand and Judge traits, the run struct with its trait objects; create_run, evaluate_gates, evaluate_checkpoint and advance functions; the gate-evaluation interaction and the run lifecycle state handling; one realization dependency per element to its SysML design element; four declared views with viewpoint usages and DocSection/DocFigure locators. Probed the compiler after writing and simplified one state guard to stay within the supported expression syntax.";
                    attribute redefines performedBy = "kimi-agent";
                    attribute redefines occurredAt = "2026-10-19T09:55:00Z";
                    attribute redefines artifactUris = ("mbse/model/uml.sysml", "cache/sysml/uml-parse-smoke.json", "cache/sysml/uml-check-smoke.json");
                }
                item produceDocumentation : ActivityRecord subsets activities {
                    attribute redefines description = "Wrote the UML documentation: docs/uml/ overview and C4/UML guide adapted from cache/templates/docs/uml/ onto the repository's directory contract; the module's Rust UML design document following the template's purpose, structure, interaction, state, interface, error, test and applicability topics; three new rows in the requirements inventory; and convention updates to docs/AGENTS.md (UML section and U prefix), mbse/AGENTS.md (uml.sysml in the model structure), the documentation index, navigation, the SimpleFlow overview and the wish status board.";
                    attribute redefines performedBy = "kimi-agent";
                    attribute redefines occurredAt = "2026-10-19T10:05:00Z";
                    attribute redefines artifactUris = ("docs/uml/00_overview.md", "docs/uml/01_c4_and_uml_guide.md", "docs/details/simple_flow/06_rust_uml_design.md", "docs/details/simple_flow/00_overview.md", "docs/requirements/01_baseline.md", "docs/00_index.md", "docs/AGENTS.md", "mbse/AGENTS.md", "mkdocs.yml", "docs/review/01_wish_status.md");
                }
                item renderFigures : ActivityRecord subsets activities {
                    attribute redefines description = "Registered the four UML views and the wish journey in tools/render_docs.py and rendered every figure from the model with the pinned toolchain: crate module layout, Rust type structure, gate-evaluation sequence and run state machine, beside their carrying documents. No figure is drawn by hand; each is a declared SysML projection regenerated by one command.";
                    attribute redefines performedBy = "kimi-agent";
                    attribute redefines occurredAt = "2026-10-19T10:10:00Z";
                    attribute redefines artifactUris = ("tools/render_docs.py", "docs/details/simple_flow/assets/06_crate_modules.svg", "docs/details/simple_flow/assets/06_class_structure.svg", "docs/details/simple_flow/assets/06_gate_sequence.svg", "docs/details/simple_flow/assets/06_run_states.svg", "docs/review/assets/01_journey_wish0009.svg", "cache/docs-render/");
                }
            }

            part verifyRecord : StageRecord subsets stageRecords {
                ref part redefines stage = xdesign.wishVerifyFlow.'verify';
                attribute redefines visitIndex = 0;
                attribute redefines status = StageStatus::completed;
                attribute redefines artifactRevision = 1;

                item verificationActivity : ActivityRecord subsets activities {
                    attribute redefines description = "Parsed and checked the extended model with the pinned compiler, including an independent project-model check without wish files; ran the documentation audit over the new locators, anchors, figures, embeds and figure blocks; regenerated all documentation figures; built the documentation site in strict mode; and read the design model, guide and design document against the wish — the realization mapping against the designed flow's elements, the applicability table against the guide's fourteen views, and every status claim against the repository's actual contents.";
                    attribute redefines performedBy = "kimi-agent";
                    attribute redefines occurredAt = "2026-10-19T10:20:00Z";
                    attribute redefines artifactUris = ("cache/evidence/wish-0009/parse.json", "cache/evidence/wish-0009/check.json", "cache/evidence/wish-0009/model-check.json", "cache/evidence/wish-0009/documentation-check.json", "cache/evidence/wish-0009/render.log", "cache/evidence/wish-0009/docs-build.log", "cache/evidence/wish-0009/semantic-review.md");
                }

                item redefines assessments : CheckpointAssessment [1];
                ref item redefines currentAssessments = outcomeAssessment;

                item outcomeAssessment : CheckpointAssessment subsets assessments {
                    ref part redefines checkpoint = xdesign.wishVerifyFlow.outcomeAccepted;
                    attribute redefines evaluatedRevision = 1;
                    attribute redefines outcome = CheckpointOutcome::passed;
                    attribute redefines judgedBy = JudgeAuthority::agent;
                    attribute redefines assessor = "kimi-agent";
                    attribute redefines rationale = "The delivered artifacts let the requester see the Rust UML design from them without reading the model or re-asking. The design document opens with purpose, scope and an explicit designed-only status, then walks crate layout, types, the gate interaction and run states with a rendered figure and reading note in each section; the guide states which UML view answers which question, and the fourteen-view applicability table makes every omission a reasoned one. The realization mapping covers the designed flow's run, factory, checkpoint class and adaptations, gate and advance functions, records, statuses and automation ports, each recorded as a dependency in SimpleFlowRustUml, satisfying the mapping requirement; the crate honors the design's existing exchanges instead of inventing interfaces. Diagrams carry the explanation where a diagram is clearer than prose — module layout, type structure, the interaction and the state machine — and tables carry what tables carry better, such as applicability and test points. Nothing is added that the wish did not ask for: no code, no crate manifest, no build or test claim; the verification case is declared as planned coverage, not executed. The compiler reports, the documentation audit, the regenerated figures and the strict site build bound this judgment mechanically; unevaluated compiler rules and the unexecuted constraint bodies remain disclosed limits, and the workflow application and planned gate commands remain unimplemented.";
                    attribute redefines evidenceUris = ("cache/evidence/wish-0009/semantic-review.md", "cache/evidence/wish-0009/parse.json", "cache/evidence/wish-0009/check.json", "cache/evidence/wish-0009/model-check.json", "cache/evidence/wish-0009/documentation-check.json", "cache/evidence/wish-0009/render.log", "cache/evidence/wish-0009/docs-build.log", "mbse/model/uml.sysml", "docs/uml/00_overview.md", "docs/uml/01_c4_and_uml_guide.md", "docs/details/simple_flow/06_rust_uml_design.md");
                    ref item redefines activities = verificationActivity;
                }
            }

            part wishApiRecord : StageRecord subsets stageRecords {
                ref part redefines stage = xdesign.wishVerifyFlow.wish;
                attribute redefines visitIndex = 1;
                attribute redefines status = StageStatus::completed;
                attribute redefines artifactRevision = 2;

                item apiGranularityRequested : ActivityRecord subsets activities {
                    attribute redefines description = "Reopened wish-0009 after the requester found revision 1 module-level: it named types and responsibilities but gave no class-level attribute and operation tables, no function signatures, and no per-function parameter, return, error or logic contracts, citing cache/kimi_template's class-and-function-level template. The requester then named the PyTorch API reference as the target organization: a complete grouped inventory page plus one page per API, each documenting a single API. The revision-1 visit and its assessments remain historical; they do not authorize this correction.";
                    attribute redefines performedBy = "kimi-agent";
                    attribute redefines occurredAt = "2026-10-19T10:30:00Z";
                    attribute redefines artifactUris = ("cache/kimi_template/docs/detailed-design/DD-类与函数级设计模板.md", "cache/kimi_template/docs/detailed-design/DD-API参考级设计模板.md", "docs/details/simple_flow/06_rust_uml_design.md");
                }

                item apiReferenceProduction : ActivityRecord subsets activities {
                    attribute redefines description = "Built the API-reference structure under docs/uml/simpflow/: a grouped inventory of 31 dedicated API entries and 31 pages — one per entry — with full Rust signatures, parameter, return and error contracts, stepwise logic, and examples where useful, in the per-item style of the PyTorch API reference. Trivial getters are documented on their owning type pages. Wrote the API-level template beside the requester's template. Added RsPlanError and RsGateError to the design model and the class view, pointed their locators at their API pages, linked the module-level documents to the reference, and listed every page in navigation.";
                    attribute redefines performedBy = "kimi-agent";
                    attribute redefines occurredAt = "2026-10-19T10:45:00Z";
                    attribute redefines artifactUris = ("docs/uml/simpflow/00_overview.md", "docs/uml/simpflow/01_create_run.md", "docs/uml/simpflow/05_evaluate_checkpoint.md", "docs/uml/simpflow/12_stage_visit.md", "docs/uml/simpflow/19_checkpoint_spec.md", "docs/uml/simpflow/31_gate_error.md", "cache/kimi_template/docs/detailed-design/DD-API参考级设计模板.md", "mbse/model/uml.sysml", "docs/uml/00_overview.md", "docs/details/simple_flow/06_rust_uml_design.md", "mkdocs.yml", "docs/details/simple_flow/assets/06_class_structure.svg");
                }
            }

            part verifyApiRecord : StageRecord subsets stageRecords {
                ref part redefines stage = xdesign.wishVerifyFlow.'verify';
                attribute redefines visitIndex = 1;
                attribute redefines status = StageStatus::completed;
                attribute redefines artifactRevision = 2;

                item apiVerification : ActivityRecord subsets activities {
                    attribute redefines description = "Parsed and checked revision 2 with the pinned compiler, including an independent project-model check; audited the ninth documentation directory's layout, anchors, locators, figures and figure blocks; regenerated all figures including the class view with the two error enums; built the site in strict mode with all 32 new pages in navigation, fixing a relative-link depth fault found by the build. Read the inventory against the design model for one-to-one coverage, the five behavioral pages for contract depth against the SysML operations they realize, and every page's status against the repository. Kept the compiler's syntactic results separate from the agent's reading.";
                    attribute redefines performedBy = "kimi-agent";
                    attribute redefines occurredAt = "2026-10-19T10:55:00Z";
                    attribute redefines artifactUris = ("cache/evidence/wish-0009/revision-2/parse.json", "cache/evidence/wish-0009/revision-2/check.json", "cache/evidence/wish-0009/revision-2/model-check.json", "cache/evidence/wish-0009/revision-2/documentation-check.json", "cache/evidence/wish-0009/revision-2/render.log", "cache/evidence/wish-0009/revision-2/docs-build.log", "cache/evidence/wish-0009/revision-2/semantic-review.md");
                }

                item redefines assessments : CheckpointAssessment [1];
                ref item redefines currentAssessments = apiOutcomeAssessment;

                item apiOutcomeAssessment : CheckpointAssessment subsets assessments {
                    ref part redefines checkpoint = xdesign.wishVerifyFlow.outcomeAccepted;
                    attribute redefines evaluatedRevision = 2;
                    attribute redefines outcome = CheckpointOutcome::passed;
                    attribute redefines judgedBy = JudgeAuthority::agent;
                    attribute redefines assessor = "kimi-agent";
                    attribute redefines rationale = "Revision 2 answers the requester's granularity finding. The inventory lists every public API of the designed crate exactly once, and each of the 31 pages documents exactly one API with its full signature, contracts and stepwise logic — the reader sees which APIs exist from the inventory alone, and what each one promises from its page, without reading prose essays or the model. The five behavioral pages were read in full: their logic matches the SysML operations they realize, including factory steps, the kind branch, the three advance paths with blocked-visit retention, and the distinct-gate authorization count with the subflow clause disclosed. The GateError split keeps 'the gate says no' as data and 'the gate cannot say' as error, consistently across pages. Names match across signatures, inventory, figures and model; every page is designed-only; the template beside the requester's makes the granularity repeatable. The compiler reports, documentation audit, regenerated figures and strict site build bound the judgment mechanically; inventory-versus-model coverage remains an agent reading, signatures are not compiled, and no code, build or test is claimed. Unevaluated compiler rules remain disclosed, and the workflow application and planned gate commands remain unimplemented.";
                    attribute redefines evidenceUris = ("cache/evidence/wish-0009/revision-2/semantic-review.md", "cache/evidence/wish-0009/revision-2/parse.json", "cache/evidence/wish-0009/revision-2/check.json", "cache/evidence/wish-0009/revision-2/model-check.json", "cache/evidence/wish-0009/revision-2/documentation-check.json", "cache/evidence/wish-0009/revision-2/render.log", "cache/evidence/wish-0009/revision-2/docs-build.log", "docs/uml/simpflow/00_overview.md", "cache/kimi_template/docs/detailed-design/DD-API参考级设计模板.md", "docs/details/simple_flow/06_rust_uml_design.md");
                    ref item redefines activities = apiVerification;
                }
            }
            part wishGuideRecord : StageRecord subsets stageRecords {
                ref part redefines stage = xdesign.wishVerifyFlow.wish;
                attribute redefines visitIndex = 2;
                attribute redefines status = StageStatus::completed;
                attribute redefines artifactRevision = 3;

                item guideAdoptionRequested : ActivityRecord subsets activities {
                    attribute redefines description = "The requester accepted closing the wish with the API-reference structure written into the project's UML conventions, so later module designs follow it by default rather than by rediscovery. Earlier visits and their assessments remain historical for their own revisions.";
                    attribute redefines performedBy = "kimi-agent";
                    attribute redefines occurredAt = "2026-10-19T14:35:00Z";
                    attribute redefines artifactUris = ("docs/uml/01_c4_and_uml_guide.md", "docs/AGENTS.md");
                }

                item guideAdoptionProduction : ActivityRecord subsets activities {
                    attribute redefines description = "Wrote the API-reference structure into the conventions: new guide section U47 defines the two required granularities, the inventory-page and one-page-per-API organization, the entry contents and the trivial-getter, naming, traceability and status conventions, with the simpflow reference as the first instance; the U12 review checklist now requires it; the docs/uml overview's guide row and docs/AGENTS.md's UML section state the docs/uml/<module>/ layout rule.";
                    attribute redefines performedBy = "kimi-agent";
                    attribute redefines occurredAt = "2026-10-19T14:40:00Z";
                    attribute redefines artifactUris = ("docs/uml/01_c4_and_uml_guide.md", "docs/uml/00_overview.md", "docs/AGENTS.md");
                }
            }

            part verifyGuideRecord : StageRecord subsets stageRecords {
                ref part redefines stage = xdesign.wishVerifyFlow.'verify';
                attribute redefines visitIndex = 2;
                attribute redefines status = StageStatus::completed;
                attribute redefines artifactRevision = 3;

                item guideVerification : ActivityRecord subsets activities {
                    attribute redefines description = "Parsed and checked revision 3 with the pinned compiler, including an independent project-model check; audited the documentation layout, anchors and locators; regenerated all figures including the journey with the closing visit; built the site in strict mode. Read U47 against the simpflow reference it normatively describes and against the requester's template, and checked that no earlier section was renumbered. Kept the compiler's syntactic results separate from the agent's reading.";
                    attribute redefines performedBy = "kimi-agent";
                    attribute redefines occurredAt = "2026-10-19T14:45:00Z";
                    attribute redefines artifactUris = ("cache/evidence/wish-0009/revision-3/parse.json", "cache/evidence/wish-0009/revision-3/check.json", "cache/evidence/wish-0009/revision-3/model-check.json", "cache/evidence/wish-0009/revision-3/documentation-check.json", "cache/evidence/wish-0009/revision-3/render.log", "cache/evidence/wish-0009/revision-3/docs-build.log", "cache/evidence/wish-0009/revision-3/semantic-review.md");
                }

                item redefines assessments : CheckpointAssessment [1];
                ref item redefines currentAssessments = guideOutcomeAssessment;

                item guideOutcomeAssessment : CheckpointAssessment subsets assessments {
                    ref part redefines checkpoint = xdesign.wishVerifyFlow.outcomeAccepted;
                    attribute redefines evaluatedRevision = 3;
                    attribute redefines outcome = CheckpointOutcome::passed;
                    attribute redefines judgedBy = JudgeAuthority::agent;
                    attribute redefines assessor = "kimi-agent";
                    attribute redefines rationale = "Revision 3 closes the wish. U47 states the convention the requester asked to make default: two granularities for a software module, a complete grouped inventory, one page per API with signature, contracts and stepwise logic, and the trivial-getter, naming, traceability and status rules — normatively describing exactly what the simpflow reference already instantiates, so the convention and its first instance agree by construction. The U12 checklist now audits it, the overview and docs/AGENTS.md point at it, and no existing section identifier was renumbered. The compiler reports, documentation audit, regenerated figures and strict site build bound the judgment mechanically. The convention's adoption by future modules cannot be established by this wish and is not claimed; signatures remain uncompiled design text; the workflow application and planned gate commands remain unimplemented.";
                    attribute redefines evidenceUris = ("cache/evidence/wish-0009/revision-3/semantic-review.md", "cache/evidence/wish-0009/revision-3/parse.json", "cache/evidence/wish-0009/revision-3/check.json", "cache/evidence/wish-0009/revision-3/model-check.json", "cache/evidence/wish-0009/revision-3/documentation-check.json", "cache/evidence/wish-0009/revision-3/render.log", "cache/evidence/wish-0009/revision-3/docs-build.log", "docs/uml/01_c4_and_uml_guide.md", "docs/uml/simpflow/00_overview.md");
                    ref item redefines activities = guideVerification;
                }
            }
        }
    }

    occurrence def Wish0009Journey {
        doc /* This wish's journey as a sequence view on its own flow's lifelines,
           over three visits per stage. Visit 0 produced the module-level design,
           delivered at revision 1. Visit 1 corrected the granularity at revision 2
           with the API-reference structure. Visit 2 closed the wish at revision 3
           by adopting the structure into the UML guide and repository conventions. */

        ref part controller : Controller = xdesign.controller;
        ref part wish : Stage = xdesign.wishVerifyFlow.wish;
        ref part 'verify' : Stage = xdesign.wishVerifyFlow.'verify';
        ref part trace : Trace = trace0009;

        message enterWish from controller.entering to wish.entered;
        message enterVerification from controller.entering to 'verify'.entered;
        message assessOutcome from controller.assessing to 'verify'.assessed;
        message deliverWish from controller.delivering to trace.delivered;
        message reopenWishForApiGranularity from controller.entering to wish.entered;
        message enterApiVerification from controller.entering to 'verify'.entered;
        message assessApiOutcome from controller.assessing to 'verify'.assessed;
        message deliverApiCorrection from controller.delivering to trace.delivered;
        message reopenWishForGuideAdoption from controller.entering to wish.entered;
        message enterGuideVerification from controller.entering to 'verify'.entered;
        message assessGuideOutcome from controller.assessing to 'verify'.assessed;
        message deliverGuideAdoption from controller.delivering to trace.delivered;
    }

    view wish0009Sequence : SequenceView {
        @XDesignCommon::DocFigure { document = "docs/records/wishes/wish-history.md"; figure = "docs/records/wishes/assets/journey-wish0009.svg"; }
        doc /* The event-anchored journey of the Rust UML design wish, including the
           revision-2 API-granularity correction and the revision-3 guide adoption. */
        expose Wish0009::Wish0009Journey;
    }

    view wish0009Run : GeneralView {
        doc /* The delivered run's records across both revisions, and the wish this
           design answers. */

        expose Wish0009::wish0009;
        expose Wish0009::trace0009;
        expose Wish0009::trace0009::execution;
        render asInterconnectionDiagram;
    }
}
