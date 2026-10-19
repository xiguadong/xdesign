package XDesignTraceability {
    @XDesignCommon::DiagramPresentation { label = "Traceability"; }
    @XDesignCommon::DocSection { document = "docs/requirements/01_baseline.md"; section = "R02"; }
    private import XDesignArchitecture::*;
    private import XDesignCommon::*;
    private import XDesignDetailedDesign::*;
    private import XDesignRequirements::*;
    private import SimpleFlowRustUml::*;

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
       attribution question that is still open.

       The documentation entry and per-flow organization requirements have no model
       satisfier: their product is the documentation set itself. The new links below
       name the model elements that support the other documentation requirements;
       the actual anchors, embeds, explanatory blocks, rules, and reading paths
       require a separate documentation review. */

    satisfy checkpointKindIsDeclared by Checkpoint.kind;
    satisfy syntacticCheckIsFixed by Checkpoint.checkers;
    satisfy syntacticAssessmentIsScripted by CheckpointAssessment.judgedBy;
    satisfy semanticCheckNamesItsJudges by Checkpoint.agentMayJudge;
    satisfy semanticCheckNamesItsJudges by Checkpoint.humanMayJudge;
    satisfy checkerStatesItsConformance by Checker.establishes;

    satisfy flowRunIsRealizedByDesign by SimpleFlowRun;
    satisfy designedFlowGatesEveryStage by designedSimpleFlow;
    satisfy gateSyntaxIsRunnable by designFlow;
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
    satisfy rustUmlDesignMapsEveryDesignElement by SimpflowCrate;

    /* umlViewsAreSelectedWithReasons and softwareDesignClaimsNoImplementation have
       no model satisfier: their product is the documentation set — the stated
       applicability of each UML view and the honest designed-only status the
       documents carry — which the documentation review judges. */
}
