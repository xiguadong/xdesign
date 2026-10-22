package Wish0008 {
    @XDesignCommon::DocSection { document = "docs/records/wishes/wish-history.md"; section = "V07"; }
    private import ScalarValues::*;
    private import XDesignCommon::*;
    private import XDesignArchitecture::*;
    private import XDesignWorkflow::*;
    private import XDesignComposition::*;
    private import StandardViewDefinitions::*;
    private import Views::*;

    doc /* The independent design-stage record for reorganizing the project's
       documentation structure and its SysML v2 modelling/documentation contract.
       Revision 1 established the entry, locator contract, and local explanations
       for figures, but its acceptance overlooked the stage-specific subtopics in
       cache/templates/ and left nine primary views in a separate diagram gallery.
       The revision-1 assessment is retained as historical evidence, not as approval
       of the corrected documents. A second wish and verification visit at artifact
       revision 2 fills the SRS, SAD, module design, ICD, ADR, and review topics and
       places those views with their owning stage explanations. Revision 3 responds
       to the requester's directory-layout correction: every documentation subfolder
       has a 00_overview.md entry, later topic files and figure assets carry shifted
       prefixes, and docs/AGENTS.md states that convention. Revision 4 responds
       to the requester's asset-location correction: each figure directory sits
       inside its owning documentation directory, including the SimpleFlow folder.
       Each revision has its own script and agent assessments. The design-stage outcome does not claim
       implementation of the workflow application or its planned gate commands. */

    item wish0008 : Wish {
        attribute redefines wishId = "wish-0008";
        attribute redefines description = "Reorganize the project's documentation structure and its SysML v2 modelling and documentation specification: establish a documentation entry and document set, per-flow organization that supports multiple flows, embedded design figures and their explanatory blocks, rule documentation that implementation can follow, and the DocSection/DocFigure and view/viewpoint interaction between documents and the SysML v2 model.";
    }

    dependency producedDocumentationEntry from wish0008 to XDesignRequirements::documentationEntryMapsDesign;
    dependency producedPerFlowStructure from wish0008 to XDesignRequirements::documentationSupportsMultipleFlows;
    dependency producedEmbeddedFigureExplanations from wish0008 to XDesignRequirements::figuresStayWithDesignExplanation;
    dependency producedDocumentationLinks from wish0008 to XDesignRequirements::modelDocumentationLinksResolve;
    dependency producedViewpointContract from wish0008 to XDesignRequirements::viewAndViewpointRolesAreSpecified;
    dependency producedImplementableRuleDocumentation from wish0008 to XDesignRequirements::checkerRulesAreImplementationReady;

    part trace0008 : Trace {
        doc /* One designFlow run has four visits to each stage. Visit 0 records the
           revision-1 delivery and its original assessments. Visit 1 corrects the
           missed stage-template subtopics and relocates the nine primary figures
           at revision 2. Visit 2 corrects the subfolder overview and file-numbering
           convention at revision 3. Visit 3 moves figure assets beside their
           owning stage documents at revision 4. Each visit retains its own assessments. */

        ref item redefines wish = wish0008;
        attribute redefines status = WishStatus::delivered;
        attribute redefines traceabilityModelUri = "mbse/wishes/wish-0008.sysml";

        part redefines execution : FlowExecution {
            ref part redefines assignedFlow = xdesign.designFlow;
            attribute redefines status = FlowStatus::completed;
            part redefines stageRecords : StageRecord [8];
            ref part redefines currentRecord = verifyAssetLayoutRecord;

            part wishRecord : StageRecord subsets stageRecords {
                ref part redefines stage = xdesign.designFlow.wish;
                attribute redefines visitIndex = 0;
                attribute redefines status = StageStatus::completed;
                attribute redefines artifactRevision = 1;

                item deriveDocumentationContract : ActivityRecord subsets activities {
                    attribute redefines description = "Derived six project requirements with rationales for the documentation entry, per-flow extension, locally explained figures, checked model/document locators, view and viewpoint roles, and implementable checker-rule documentation. Added structural satisfaction links where model elements support those obligations and declared a documentation contract review case.";
                    attribute redefines performedBy = "codex-agent";
                    attribute redefines occurredAt = "2026-10-19T04:51:25Z";
                    attribute redefines artifactUris = ("mbse/model/requirements.sysml", "mbse/model/traceability.sysml", "mbse/model/verification.sysml", "mbse/wishes/wish-0008.sysml");
                }

                item produceDocumentationContract : ActivityRecord subsets activities {
                    attribute redefines description = "Established the documentation entry and reading paths, per-flow overview, reverse model map, and D27-D31 SysML documentation contract. Added nine formal viewpoint usages bound to the existing native views, placed 27 model-rendered figures beside explanatory blocks, specified the planned SimpleFlow checker rules, and implemented the repository documentation locator and figure-block audit. Preserved historical section identifiers and distinguished designed checks from running utilities.";
                    attribute redefines performedBy = "codex-agent";
                    attribute redefines occurredAt = "2026-10-19T05:02:15Z";
                    attribute redefines artifactUris = ("docs/00_index.md", "docs/details/04_documentation_contract.md", "docs/details/simple_flow/01_gates_and_rules.md", "docs/details/simple_flow/03_sysml_diagrams.md", "docs/requirements/00_baseline.md", "mbse/model/detailedDesign.sysml", "mbse/model/common.sysml", "tools/check_documentation.py", "mkdocs.yml");
                }
            }

            part verifyRecord : StageRecord subsets stageRecords {
                ref part redefines stage = xdesign.designFlow.'verify';
                attribute redefines visitIndex = 0;
                attribute redefines status = StageStatus::completed;
                attribute redefines artifactRevision = 1;

                item verificationActivity : ActivityRecord subsets activities {
                    attribute redefines description = "Parsed and checked the delivered SysML with the pinned compiler, including an independent project-model check; audited all documentation locators, figure targets, embeds, and adjacent blocks; probed checker failures; regenerated figures; verified the nine primary diagrams and built the site in strict mode. Compared nine viewpoint concerns with their native views and read the entry, figure explanations, implementation-ready rules, and status boundaries against the six requirements. The compiler and audit results are syntactic evidence; the separate design deliverable review records the semantic judgment.";
                    attribute redefines performedBy = "codex-agent";
                    attribute redefines occurredAt = "2026-10-19T05:02:15Z";
                    attribute redefines artifactUris = ("cache/evidence/wish-0008/parse.json", "cache/evidence/wish-0008/check.json", "cache/evidence/wish-0008/model-check.json", "cache/evidence/wish-0008/documentation-check.json", "cache/evidence/wish-0008/checker-probes.json", "cache/evidence/wish-0008/docs-build.log", "cache/evidence/wish-0008/render.log", "cache/evidence/wish-0008/viewpoint-agreement.json", "cache/evidence/wish-0008/figures-before.json", "cache/evidence/wish-0008/figures-after.json", "cache/evidence/wish-0008/diagram-verification/figure-manifest.json", "cache/evidence/wish-0008/diagram-verification.log", "cache/evidence/wish-0008/semantic-review.md");
                }

                item redefines assessments : CheckpointAssessment [2];
                ref item redefines currentAssessments = (sysmlAssessment, meaningAssessment);

                item sysmlAssessment : CheckpointAssessment subsets assessments {
                    ref part redefines checkpoint = xdesign.designFlow.sysmlDeliverableAccepted;
                    attribute redefines evaluatedRevision = 1;
                    attribute redefines outcome = CheckpointOutcome::passed;
                    attribute redefines judgedBy = JudgeAuthority::script;
                    attribute redefines assessor = "sysml-compiler";
                    attribute redefines rationale = "The repository-bundled compiler parsed the delivered project and wish SysML, checked their references and supported rules, and independently checked the project model without wish files. The retained reports contain no parse errors, unresolved names, collisions, or violations. Unevaluated rules and unexecuted constraint bodies remain visible in the reports; this assessment establishes only the compiler's syntactic checkpoint.";
                    attribute redefines evidenceUris = ("cache/evidence/wish-0008/parse.json", "cache/evidence/wish-0008/check.json", "cache/evidence/wish-0008/model-check.json");
                    ref item redefines activities = verificationActivity;
                }

                item meaningAssessment : CheckpointAssessment subsets assessments {
                    ref part redefines checkpoint = xdesign.designFlow.outcomeAccepted;
                    attribute redefines evaluatedRevision = 1;
                    attribute redefines outcome = CheckpointOutcome::passed;
                    attribute redefines judgedBy = JudgeAuthority::agent;
                    attribute redefines assessor = "codex-agent";
                    attribute redefines rationale = "Revision 1 meets the documentation design request. The entry provides reader paths, the actual document set, five flow-usage overviews, and a module-level reverse map; developed flow design has its own location without requiring undeveloped flows to have one. All current model-rendered figures sit in explanatory sections with source, companions, status, reading guidance, and limits. DocSection and DocFigure targets, embeds, and blocks passed the deterministic audit; nine formal viewpoints bind native views and agree with their stated questions. D06 and D31 provide rule inputs, decision conditions, diagnostics, evidence output, and honest implementation status. The strict site build, regenerated figures, viewpoint agreement, diagram verification, checker probes, and semantic review support this judgment. The documentation audit does not prove engineering meaning, the compiler does not execute constraint bodies, and the workflow application and planned gate commands remain unimplemented.";
                    attribute redefines evidenceUris = ("cache/evidence/wish-0008/semantic-review.md", "cache/evidence/wish-0008/documentation-check.json", "cache/evidence/wish-0008/checker-probes.json", "cache/evidence/wish-0008/viewpoint-agreement.json", "cache/evidence/wish-0008/docs-build.log", "cache/evidence/wish-0008/render.log", "cache/evidence/wish-0008/diagram-verification/figure-manifest.json", "cache/evidence/wish-0008/diagram-verification.log", "docs/00_index.md", "docs/details/04_documentation_contract.md", "docs/details/simple_flow/01_gates_and_rules.md", "docs/details/simple_flow/03_sysml_diagrams.md");
                    ref item redefines activities = verificationActivity;
                }
            }

            part wishCorrectionRecord : StageRecord subsets stageRecords {
                ref part redefines stage = xdesign.designFlow.wish;
                attribute redefines visitIndex = 1;
                attribute redefines status = StageStatus::completed;
                attribute redefines artifactRevision = 2;

                item correctionRequested : ActivityRecord subsets activities {
                    attribute redefines description = "Reopened wish-0008 after the requester found that revision 1 had treated cache/templates/ chiefly as an index and link pattern: the actual SRS, SAD, module design, ICD, ADR, and review documents had not carried every applicable stage subtopic, and nine primary SimpleFlow figures remained in a separate gallery instead of the sections that explain their design role. The revision-1 visit and its assessments remain historical; they do not authorize this correction.";
                    attribute redefines performedBy = "codex-agent";
                    attribute redefines occurredAt = "2026-10-19T05:32:07Z";
                    attribute redefines artifactUris = ("cache/templates/README.md", "cache/templates/docs/requirements/00_baseline.md", "cache/templates/docs/architecture/00_design.md", "cache/templates/docs/detailed-design/00_module.md", "cache/templates/docs/icd/00_interfaces.md", "cache/templates/docs/adr/0000_template.md", "cache/templates/docs/review/00_checklist.md", "docs/review/00_wish_status.md");
                }

                item correctionProduction : ActivityRecord subsets activities {
                    attribute redefines description = "Expanded the live requirements specification, system architecture design, SimpleFlow module design, interface control document, ADR index, and engineering review checklist using the template's stage subtopics. Filled each topic with present model facts and explicit open fields rather than treating an unmade decision or unheld SRR/PDR/CDR as completed. Moved the nine primary figure explanations out of the gallery into the owning SRS, SAD, and module design sections, retaining their view and viewpoint links, nearby reading guidance, companions, status, and limits. Updated entry and cross-stage paths to those sections.";
                    attribute redefines performedBy = "codex-agent";
                    attribute redefines occurredAt = "2026-10-19T05:32:07Z";
                    attribute redefines artifactUris = ("docs/00_index.md", "docs/requirements/00_baseline.md", "docs/arch/02_design.md", "docs/details/simple_flow/04_module_design.md", "docs/icd/00_interfaces.md", "docs/adr/00_index.md", "docs/review/01_checklist.md", "docs/details/simple_flow/03_sysml_diagrams.md", "docs/details/04_documentation_contract.md", "mbse/model/detailedDesign.sysml", "mkdocs.yml");
                }
            }

            part verifyCorrectionRecord : StageRecord subsets stageRecords {
                ref part redefines stage = xdesign.designFlow.'verify';
                attribute redefines visitIndex = 1;
                attribute redefines status = StageStatus::completed;
                attribute redefines artifactRevision = 2;

                item correctionVerification : ActivityRecord subsets activities {
                    attribute redefines description = "Parsed and checked revision 2 with the pinned SysML compiler, including an independent project-model check; audited documentation locators, figure targets, embeds, and adjacent explanations; regenerated figures, ran the renderer tests, verified the nine primary diagrams, and built the site in strict mode. Compared the live SRS, SAD, module design, ICD, ADR, and review sections with the supplied template subtopics and read the relocated figures in their owning design sections. Kept the compiler's syntactic results separate from the agent's stage-content judgment.";
                    attribute redefines performedBy = "codex-agent";
                    attribute redefines occurredAt = "2026-10-19T05:32:07Z";
                    attribute redefines artifactUris = ("cache/evidence/wish-0008/revision-2/parse.json", "cache/evidence/wish-0008/revision-2/check.json", "cache/evidence/wish-0008/revision-2/model-check.json", "cache/evidence/wish-0008/revision-2/documentation-check.json", "cache/evidence/wish-0008/revision-2/docs-build.log", "cache/evidence/wish-0008/revision-2/render.log", "cache/evidence/wish-0008/revision-2/renderer-tests.log", "cache/evidence/wish-0008/revision-2/diagram-verification.log", "cache/evidence/wish-0008/revision-2/template-coverage.json", "cache/evidence/wish-0008/revision-2/semantic-review.md");
                }

                item redefines assessments : CheckpointAssessment [2];
                ref item redefines currentAssessments = (correctionSysmlAssessment, correctionMeaningAssessment);

                item correctionSysmlAssessment : CheckpointAssessment subsets assessments {
                    ref part redefines checkpoint = xdesign.designFlow.sysmlDeliverableAccepted;
                    attribute redefines evaluatedRevision = 2;
                    attribute redefines outcome = CheckpointOutcome::passed;
                    attribute redefines judgedBy = JudgeAuthority::script;
                    attribute redefines assessor = "sysml-compiler";
                    attribute redefines rationale = "The repository-bundled compiler parsed the corrected SysML and checked the project plus wishes and the project model on its own. The retained reports show the supported reference and rule results without parse errors, unresolved names, collisions, or violations. Unevaluated rules remain disclosed; compiler success does not judge whether a stage document covers its template or whether a design claim is correct.";
                    attribute redefines evidenceUris = ("cache/evidence/wish-0008/revision-2/parse.json", "cache/evidence/wish-0008/revision-2/check.json", "cache/evidence/wish-0008/revision-2/model-check.json");
                    ref item redefines activities = correctionVerification;
                }

                item correctionMeaningAssessment : CheckpointAssessment subsets assessments {
                    ref part redefines checkpoint = xdesign.designFlow.outcomeAccepted;
                    attribute redefines evaluatedRevision = 2;
                    attribute redefines outcome = CheckpointOutcome::passed;
                    attribute redefines judgedBy = JudgeAuthority::agent;
                    attribute redefines assessor = "codex-agent";
                    attribute redefines rationale = "Revision 2 addresses the requester's scope finding. The live SRS, SAD, SimpleFlow module design, ICD, ADR index, and review checklist now follow the applicable subtopics of their cache/templates/ sources, with current decisions and evidence distinguished from open fields, planned interfaces, and unheld SRR/PDR/CDR gates. The nine primary SimpleFlow views are explained in the SRS, SAD, or module design section that uses each view, with local figure blocks and model/document links; the former gallery is a navigation page. The template coverage inventory and semantic review inspect substance and gaps, while documentation audit, rendering, diagram checks, and strict site build establish mechanical integrity. Those checks do not prove all engineering claims or execute planned workflow gates and application behavior.";
                    attribute redefines evidenceUris = ("cache/evidence/wish-0008/revision-2/template-coverage.json", "cache/evidence/wish-0008/revision-2/semantic-review.md", "cache/evidence/wish-0008/revision-2/documentation-check.json", "cache/evidence/wish-0008/revision-2/docs-build.log", "cache/evidence/wish-0008/revision-2/render.log", "cache/evidence/wish-0008/revision-2/renderer-tests.log", "cache/evidence/wish-0008/revision-2/diagram-verification.log", "docs/requirements/00_baseline.md", "docs/arch/02_design.md", "docs/details/simple_flow/04_module_design.md", "docs/icd/00_interfaces.md", "docs/adr/00_index.md", "docs/review/01_checklist.md");
                    ref item redefines activities = correctionVerification;
                }
            }

            part wishDirectoryLayoutRecord : StageRecord subsets stageRecords {
                ref part redefines stage = xdesign.designFlow.wish;
                attribute redefines visitIndex = 2;
                attribute redefines status = StageStatus::completed;
                attribute redefines artifactRevision = 3;

                item directoryLayoutRequested : ActivityRecord subsets activities {
                    attribute redefines description = "Reopened wish-0008 after the requester found that docs/AGENTS.md had not fully specified the desired directory convention and some documentation subfolders lacked a 00 overview. The requested convention uses 00_overview.md in every documentation subfolder, converts an existing 00_index.md where present, inserts an overview where absent, and shifts later topic-file numbers. Earlier wish and verification visits remain historical assessments of their own artifact revisions.";
                    attribute redefines performedBy = "codex-agent";
                    attribute redefines occurredAt = "2026-10-19T06:04:11Z";
                    attribute redefines artifactUris = ("docs/AGENTS.md", "docs/adr/00_overview.md", "docs/arch/00_overview.md", "docs/details/00_overview.md", "docs/details/simple_flow/00_overview.md", "docs/icd/00_overview.md", "docs/requirements/00_overview.md", "docs/review/00_overview.md");
                }

                item directoryLayoutProduction : ActivityRecord subsets activities {
                    attribute redefines description = "Revised docs/AGENTS.md to make the repository index, recursive subfolder overview, local numbering, stable section identifiers, and mirrored figure prefixes explicit. Established 00_overview.md in the seven documentation subfolders and shifted their later topic-file prefixes. Repaired document links, MkDocs navigation, SysML DocSection and DocFigure annotations, figure asset paths, renderer targets, and the documentation layout check. Preserved the stage-specific SRS, SAD, module design, ICD, ADR, and review content from revision 2.";
                    attribute redefines performedBy = "codex-agent";
                    attribute redefines occurredAt = "2026-10-19T06:04:11Z";
                    attribute redefines artifactUris = ("docs/AGENTS.md", "docs/adr/00_overview.md", "docs/arch/00_overview.md", "docs/details/00_overview.md", "docs/details/simple_flow/00_overview.md", "docs/icd/00_overview.md", "docs/requirements/00_overview.md", "docs/review/00_overview.md", "tools/check_documentation.py", "tools/render_docs.py", "mkdocs.yml");
                }
            }

            part verifyDirectoryLayoutRecord : StageRecord subsets stageRecords {
                ref part redefines stage = xdesign.designFlow.'verify';
                attribute redefines visitIndex = 2;
                attribute redefines status = StageStatus::completed;
                attribute redefines artifactRevision = 3;

                item directoryLayoutVerification : ActivityRecord subsets activities {
                    attribute redefines description = "Parsed and checked the revision-3 SysML with the pinned compiler, including an independent project-model check; audited the seven subfolder overviews, numbering, document and figure path migration, model locators, and adjacent figure explanations; regenerated figures, ran renderer tests, verified the nine primary diagrams, and built the documentation site in strict mode. Read the overview pages and retained stage topics for their intended roles. The compiler and mechanical checks are separate from the agent's documentation judgment.";
                    attribute redefines performedBy = "codex-agent";
                    attribute redefines occurredAt = "2026-10-19T06:04:11Z";
                    attribute redefines artifactUris = ("cache/evidence/wish-0008/revision-3/parse.json", "cache/evidence/wish-0008/revision-3/check.json", "cache/evidence/wish-0008/revision-3/model-check.json", "cache/evidence/wish-0008/revision-3/documentation-check.json", "cache/evidence/wish-0008/revision-3/docs-build.log", "cache/evidence/wish-0008/revision-3/render.log", "cache/evidence/wish-0008/revision-3/renderer-tests.log", "cache/evidence/wish-0008/revision-3/diagram-verification.log", "cache/evidence/wish-0008/revision-3/directory-layout.json", "cache/evidence/wish-0008/revision-3/path-migration.json", "cache/evidence/wish-0008/revision-3/semantic-review.md");
                }

                item redefines assessments : CheckpointAssessment [2];
                ref item redefines currentAssessments = (directoryLayoutSysmlAssessment, directoryLayoutMeaningAssessment);

                item directoryLayoutSysmlAssessment : CheckpointAssessment subsets assessments {
                    ref part redefines checkpoint = xdesign.designFlow.sysmlDeliverableAccepted;
                    attribute redefines evaluatedRevision = 3;
                    attribute redefines outcome = CheckpointOutcome::passed;
                    attribute redefines judgedBy = JudgeAuthority::script;
                    attribute redefines assessor = "sysml-compiler";
                    attribute redefines rationale = "The repository-bundled compiler parsed revision 3 and checked the project plus wishes and the project model independently. Its retained reports record the supported reference and rule results without parse errors, unresolved names, collisions, or violations. Unevaluated rules remain disclosed; compiler success alone cannot establish that documentation directories follow the requested convention.";
                    attribute redefines evidenceUris = ("cache/evidence/wish-0008/revision-3/parse.json", "cache/evidence/wish-0008/revision-3/check.json", "cache/evidence/wish-0008/revision-3/model-check.json");
                    ref item redefines activities = directoryLayoutVerification;
                }

                item directoryLayoutMeaningAssessment : CheckpointAssessment subsets assessments {
                    ref part redefines checkpoint = xdesign.designFlow.outcomeAccepted;
                    attribute redefines evaluatedRevision = 3;
                    attribute redefines outcome = CheckpointOutcome::passed;
                    attribute redefines judgedBy = JudgeAuthority::agent;
                    attribute redefines assessor = "codex-agent";
                    attribute redefines rationale = "Revision 3 addresses the requester's directory-layout correction. docs/AGENTS.md now states the recursive 00_overview.md and local topic-numbering rule. All seven documentation subfolders have their own overview, and the moved topic files retain their stage-specific explanations and stable section identifiers. The directory and path-migration inventories, documentation audit, renderer tests, regenerated figure and diagram checks, strict site build, and semantic review support the judgment that links, model annotations, navigation, and figure assets follow the new paths. These checks do not prove every engineering design claim or implement the planned workflow gates and application behavior.";
                    attribute redefines evidenceUris = ("cache/evidence/wish-0008/revision-3/directory-layout.json", "cache/evidence/wish-0008/revision-3/path-migration.json", "cache/evidence/wish-0008/revision-3/semantic-review.md", "cache/evidence/wish-0008/revision-3/documentation-check.json", "cache/evidence/wish-0008/revision-3/docs-build.log", "cache/evidence/wish-0008/revision-3/render.log", "cache/evidence/wish-0008/revision-3/renderer-tests.log", "cache/evidence/wish-0008/revision-3/diagram-verification.log", "docs/AGENTS.md", "docs/adr/00_overview.md", "docs/arch/00_overview.md", "docs/details/00_overview.md", "docs/details/simple_flow/00_overview.md", "docs/icd/00_overview.md", "docs/requirements/00_overview.md", "docs/review/00_overview.md");
                    ref item redefines activities = directoryLayoutVerification;
                }
            }

            part wishAssetLayoutRecord : StageRecord subsets stageRecords {
                ref part redefines stage = xdesign.designFlow.wish;
                attribute redefines visitIndex = 3;
                attribute redefines status = StageStatus::completed;
                attribute redefines artifactRevision = 4;

                item assetLayoutRequested : ActivityRecord subsets activities {
                    attribute redefines description = "Reopened wish-0008 after the requester clarified that figure assets belong in an assets/ child of the same documentation directory as their stage or flow topics. The prior central docs/assets/ tree separated figures from their owning arch, details, SimpleFlow, requirements, and review documents. The requester named docs/details/simpleflow/assets/ as an example; this repository's existing flow directory is docs/details/simple_flow/. Earlier assessments remain historical for their own artifact revisions.";
                    attribute redefines performedBy = "codex-agent";
                    attribute redefines occurredAt = "2026-10-19T06:19:46Z";
                    attribute redefines artifactUris = ("docs/AGENTS.md", "docs/arch/00_overview.md", "docs/details/00_overview.md", "docs/details/simple_flow/00_overview.md", "docs/requirements/00_overview.md", "docs/review/00_overview.md");
                }

                item assetLayoutProduction : ActivityRecord subsets activities {
                    attribute redefines description = "Moved stage and flow figures into assets/ directories beside the documents that own them; repaired Markdown image paths, SysML DocFigure locators, renderer and diagram-verification targets, documentation checks, and directory guidance. Preserved each figure's document prefix, section identifier, explanatory block, and stage-specific design text.";
                    attribute redefines performedBy = "codex-agent";
                    attribute redefines occurredAt = "2026-10-19T06:19:46Z";
                    attribute redefines artifactUris = ("docs/arch/assets", "docs/details/assets", "docs/details/simple_flow/assets", "docs/requirements/assets", "docs/review/assets", "docs/AGENTS.md", "tools/render_docs.py", "tools/verify_diagrams.py", "tools/check_documentation.py");
                }
            }

            part verifyAssetLayoutRecord : StageRecord subsets stageRecords {
                ref part redefines stage = xdesign.designFlow.'verify';
                attribute redefines visitIndex = 3;
                attribute redefines status = StageStatus::completed;
                attribute redefines artifactRevision = 4;

                item assetLayoutVerification : ActivityRecord subsets activities {
                    attribute redefines description = "Parsed and checked revision-4 SysML with the pinned compiler, including an independent project-model check; audited migrated figure paths, owning-directory assets placement, model locators, Markdown embeds, and adjacent explanations; regenerated figures, ran renderer tests, verified the nine primary diagrams, and built the site in strict mode. Read the revised asset guidance and retained stage documents. The compiler and mechanical checks are separate from the agent's documentation judgment.";
                    attribute redefines performedBy = "codex-agent";
                    attribute redefines occurredAt = "2026-10-19T06:19:46Z";
                    attribute redefines artifactUris = ("cache/evidence/wish-0008/revision-4/parse.json", "cache/evidence/wish-0008/revision-4/check.json", "cache/evidence/wish-0008/revision-4/model-check.json", "cache/evidence/wish-0008/revision-4/documentation-check.json", "cache/evidence/wish-0008/revision-4/docs-build.log", "cache/evidence/wish-0008/revision-4/render.log", "cache/evidence/wish-0008/revision-4/renderer-tests.log", "cache/evidence/wish-0008/revision-4/diagram-verification.log", "cache/evidence/wish-0008/revision-4/asset-migration.json", "cache/evidence/wish-0008/revision-4/asset-layout.json", "cache/evidence/wish-0008/revision-4/semantic-review.md");
                }

                item redefines assessments : CheckpointAssessment [2];
                ref item redefines currentAssessments = (assetLayoutSysmlAssessment, assetLayoutMeaningAssessment);

                item assetLayoutSysmlAssessment : CheckpointAssessment subsets assessments {
                    ref part redefines checkpoint = xdesign.designFlow.sysmlDeliverableAccepted;
                    attribute redefines evaluatedRevision = 4;
                    attribute redefines outcome = CheckpointOutcome::passed;
                    attribute redefines judgedBy = JudgeAuthority::script;
                    attribute redefines assessor = "sysml-compiler";
                    attribute redefines rationale = "The repository-bundled compiler parsed revision 4 and checked the project plus wishes and the project model independently. Its retained reports record supported reference and rule results without parse errors, unresolved names, collisions, or violations. Unevaluated rules remain disclosed; compiler success alone cannot establish that figures are stored beside their owning documents.";
                    attribute redefines evidenceUris = ("cache/evidence/wish-0008/revision-4/parse.json", "cache/evidence/wish-0008/revision-4/check.json", "cache/evidence/wish-0008/revision-4/model-check.json");
                    ref item redefines activities = assetLayoutVerification;
                }

                item assetLayoutMeaningAssessment : CheckpointAssessment subsets assessments {
                    ref part redefines checkpoint = xdesign.designFlow.outcomeAccepted;
                    attribute redefines evaluatedRevision = 4;
                    attribute redefines outcome = CheckpointOutcome::passed;
                    attribute redefines judgedBy = JudgeAuthority::agent;
                    attribute redefines assessor = "codex-agent";
                    attribute redefines rationale = "Revision 4 addresses the requester's asset-location correction. Stage and flow figures now reside in assets/ directories under the same folders as their owning documents, including docs/details/simple_flow/assets/. Figure identifiers, local explanatory blocks, and stage-specific document content are retained. The asset migration and layout inventories, documentation audit, rendering, diagram verification, strict site build, and semantic review support the path and placement judgment. These checks do not prove every engineering design claim or implement the planned workflow gates and application behavior.";
                    attribute redefines evidenceUris = ("cache/evidence/wish-0008/revision-4/asset-migration.json", "cache/evidence/wish-0008/revision-4/asset-layout.json", "cache/evidence/wish-0008/revision-4/semantic-review.md", "cache/evidence/wish-0008/revision-4/documentation-check.json", "cache/evidence/wish-0008/revision-4/docs-build.log", "cache/evidence/wish-0008/revision-4/render.log", "cache/evidence/wish-0008/revision-4/renderer-tests.log", "cache/evidence/wish-0008/revision-4/diagram-verification.log", "docs/AGENTS.md", "docs/arch/assets", "docs/details/assets", "docs/details/simple_flow/assets", "docs/requirements/assets", "docs/review/assets");
                    ref item redefines activities = assetLayoutVerification;
                }
            }
        }
    }

    occurrence wish0008Journey {
        doc /* Four visits to designFlow's wish and verification stages. The
           revision-1 delivery and revision-2 stage-topic correction remain
           historical. Revision 3 corrected the directory overview convention
           and received separate compiler and semantic assessments. Revision 4
           moves figure assets beside the stage and flow documents that own them. */
        ref part controller : Controller = xdesign.controller;
        ref part wish : Stage = xdesign.designFlow.wish;
        ref part 'verify' : Stage = xdesign.designFlow.'verify';
        ref part trace : Trace = trace0008;

        message enterWish from controller.entering to wish.entered;
        message enterVerification from controller.entering to 'verify'.entered;
        message assessDeliveredSysml from controller.assessing to 'verify'.assessed;
        message assessDocumentationMeaning from controller.assessing to 'verify'.assessed;
        message deliverDocumentationContract from controller.delivering to trace.delivered;
        message reopenWishForTemplateTopics from controller.entering to wish.entered;
        message enterCorrectionVerification from controller.entering to 'verify'.entered;
        message assessCorrectedSysml from controller.assessing to 'verify'.assessed;
        message assessStageDocumentationMeaning from controller.assessing to 'verify'.assessed;
        message deliverStageDocumentationCorrection from controller.delivering to trace.delivered;
        message reopenWishForDirectoryOverviews from controller.entering to wish.entered;
        message enterDirectoryLayoutVerification from controller.entering to 'verify'.entered;
        message assessDirectoryLayoutSysml from controller.assessing to 'verify'.assessed;
        message assessDirectoryLayoutMeaning from controller.assessing to 'verify'.assessed;
        message deliverDirectoryLayoutCorrection from controller.delivering to trace.delivered;
        message reopenWishForLocalAssets from controller.entering to wish.entered;
        message enterAssetLayoutVerification from controller.entering to 'verify'.entered;
        message assessAssetLayoutSysml from controller.assessing to 'verify'.assessed;
        message assessAssetLayoutMeaning from controller.assessing to 'verify'.assessed;
        message deliverAssetLayoutCorrection from controller.delivering to trace.delivered;
    }

    view wish0008Sequence : SequenceView {
        doc /* The event-anchored journey of the documentation design wish. */
        expose Wish0008::wish0008Journey;
    }

    view wish0008Run : GeneralView {
        doc /* The delivered design-stage run and independent revision-4 trace. */
        expose Wish0008::wish0008;
        expose Wish0008::trace0008;
        expose Wish0008::trace0008::execution;
        render asInterconnectionDiagram;
    }
}
