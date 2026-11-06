package XDesignTraceability {
    @XDesignCommon::DiagramPresentation { label = "Traceability"; }
    @XDesignCommon::DocSection { document = "docs/requirements/requirements.md"; section = "R02"; }
    private import XDesignArchitecture::*;
    private import XDesignCommon::*;
    private import XDesignDetailedDesign::*;
    private import XDesignComposition::*;
    private import XDesignRequirements::*;
    private import XDesignRustUml::*;

    doc /* The traceability relationships of the project model: which model element
       answers which requirement. They hold for the project rather than for one wish,
       so they live here; what a single wish derived, and why, stays with that wish.

       wishProductsAreRetained, modelStandsWithoutWishes and runRecordIsScriptChecked
       have no project-wide satisfier yet. Nothing in the model holds a wish's
       products to the model or keeps wish identity out of it; the first two are
       maintained by hand. The designed SimpleFlow declares a record checker, but
       that narrower checker does not establish record conformance for the default
       V-model or every historical wish trace.

       Each linked requirement below names one satisfier, except semanticCheckNamesItsJudges,
       which names two because a semantic checkpoint needs both judge flags. The export
       cannot attribute two satisfy usages of one requirement to their requirement — an
       attribution question that is still open. The designed wish-verify flow's coverage
       of designedFlowGatesEveryStage, flowRunIsRealizedByDesign and runRealizesItsDesign
       is therefore recorded by its planned verification case and its design
       documentation, not by second satisfy usages here: adding them removed the
       existing edges from the requirements figure's export.

       The documentation entry and per-flow organization requirements have no model
       satisfier: their product is the documentation set itself. The new links below
       name the model elements that support the other documentation requirements;
       the actual anchors, embeds, explanatory blocks, rules, and reading paths
       require a separate documentation review. */

    satisfy checkpointSourceIsProjectOwned by CheckpointLibrarySource;
    satisfy configuredChecksRetainAuthority by XDesignRustUml::RsCustomChecker;
    satisfy operatorDelegatesRunState by OperatorWebProgram;
    /* sharedChecksHavePinnedOrigins and crossProjectHistoryKeepsItsOwner are
       proposed requirements without implemented satisfiers. See R15 and D108. */
    satisfy checkpointKindIsDeclared by Checkpoint.kind;
    satisfy syntacticCheckIsFixed by Checkpoint.checkers;
    satisfy syntacticAssessmentIsScripted by CheckpointAssessment.judgedBy;
    satisfy semanticCheckNamesItsJudges by Checkpoint.agentMayJudge;
    satisfy semanticCheckNamesItsJudges by Checkpoint.humanMayJudge;
    satisfy checkerStatesItsConformance by Checker.establishes;

    satisfy flowRunIsRealizedByDesign by SimpleFlowRun;
    satisfy designedFlowGatesEveryStage by xdesign.designedSimpleFlow;
    satisfy gateSyntaxIsRunnable by xdesign.designFlow;
    satisfy designCarriesItsDocSections by DocSection;
    satisfy checkerRulesAreDeclared by CheckerRule;
    satisfy structureGraphsAreDocumented by DocFigure;
    satisfy runRealizesItsDesign by SimpleFlowRun;
    satisfy diagramViewsAreReadable by DiagramDocumentationRenderer;
    satisfy diagramPresentationRetainsSourceIdentity by DiagramDocumentationRenderer;
    satisfy figuresStayWithDesignExplanation by DocFigure;
    satisfy modelDocumentationLinksResolve by DocSection;
    satisfy viewAndViewpointRolesAreSpecified by DocumentationViewpoint;
    satisfy checkerRulesAreImplementationReady by CheckerRule;
    satisfy rustUmlDesignMapsEveryDesignElement by XDesignCrate;

    /* umlViewsAreSelectedWithReasons and softwareDesignClaimsNoImplementation have
       no model satisfier: their product is the documentation set — the stated
       applicability of each UML view and the honest designed-only status the
       documents carry — which the documentation review judges. */
    satisfy stageCliUsesUniformInteraction by XDesignWorkflow::StageCli;
    satisfy stageCliPreservesRunIntegrity by XDesignWorkflow::LocalStageCliRuntime;

    satisfy operationalRecordsHaveDurableAuthority by XDesignStorage::ProjectStore;
    satisfy wishHistoryIsRevisioned by XDesignStorage::ItemRevision;
    satisfy materialsAreContentAddressed by XDesignStorage::Material;
    satisfy storageSaveIsAtomicAndIdempotent by XDesignStorage::SaveOperation;
    satisfy workingCacheIsRecoverable by XDesignStorage::CheckoutOperation;
    satisfy wishStatusProjectsFlowExecution by XDesignStorage::WishProjection;
    satisfy storageInputAndIntegrityAreChecked by XDesignStorage::DoctorOperation;

    /* R21 is allocated to the explicitly planned worktree selection, migration,
       access and integration definitions. These links declare design responsibility;
       they do not claim that ProjectStore, the lifecycle CLI or Web currently
       implements it, nor that any migration case or acceptance has passed. */
    satisfy linkedWorktreeStoreIsIsolated by XDesignStorage::PlannedWorktreeStoreSelection;
    satisfy storeRootSelectionIsConsistent by XDesignStorage::PlannedWorktreeStoreAccess;
    satisfy globalStoreNamespaceProtectsOtherContent by XDesignStorage::PlannedWorktreeStoreSelection;
    satisfy worktreeStoreMigrationPreservesHistory by XDesignStorage::PlannedWorktreeStoreMigration;
    satisfy migratedRunsKeepTheirLifecycle by XDesignStorage::PlannedWorktreeStoreIntegration;
    satisfy worktreeStoreConflictsFailClosed by XDesignStorage::PlannedWorktreeStoreAccess;
    satisfy worktreeMigrationHasRecoverableCutover by XDesignStorage::PlannedWorktreeStoreMigration;
    satisfy legacyCacheRegistrationsStayLegacy by XDesignStorage::PlannedWorktreeStoreAccess;

    satisfy releasedCheckersAreSelfContained by CheckerResourceDistribution;
    satisfy checkerExecutorsAreRetained by CheckerExecutorBundle;
    satisfy checkerExecutionUsesProjectContext by ProjectCheckerExecution;

    satisfy documentContractIsPortable by PortableDocumentKit;
    satisfy documentAuthoringFollowsContract by DocumentContractEngine;
    satisfy documentChecksReportContractGaps by DocumentContractEngine;
    satisfy documentToolsShareOneContract by DocumentContractEngine;
    /* documentTemplatesCoverScenarios, documentTemplateSourcesAreInspectable,
       documentTemplateSelectionIsExplicit, documentTemplateChecksUseSelectedContract,
       documentTemplateCatalogPreservesCompatibility, documentTemplateEvidenceIsBound,
       and documentTemplateCatalogSurfaceIsShared are proposed requirements without
       design satisfiers. Existing source and engine elements are candidates for later
       design allocation, not established satisfaction. */

    /* structuredDocumentationHasNamedRoutes, apiDesignDocumentsCoverContract,
       flowDesignDocumentsCoverLayers, structuredDocumentationUsesSafeOperations,
       structuredDocumentationKeepsVersionIdentity, structuredDocumentationHasSharedSurfaces,
       documentKitPublicApiIsDocumented, and structuredDocumentationEvidenceIsReviewable
       are proposed requirements. No satisfier is asserted before the structured
       authoring design is allocated and reviewed; planned verification links live
       in XDesignVerification::StructuredDocumentKitVerification. */

}
