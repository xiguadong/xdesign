package XDesignCommon {
    @XDesignCommon::DiagramPresentation { label = "Common"; }
    @XDesignCommon::DocSection { document = "docs/details/01_flows_and_checkpoints.md"; section = "F02"; }
    private import ScalarValues::*;

    doc /* Shared lifecycle, assessment types and documentation location metadata for
       the project model. */

    enum def WishStatus {
        @XDesignCommon::DocSection { document = "docs/arch/00_overview.md"; section = "A03"; }
        enum received;
        enum active;
        enum blocked;
        enum delivered;
    }

    enum def StageStatus {
        @XDesignCommon::DocSection { document = "docs/details/02_records_and_evidence.md"; section = "E02"; }
        enum pending;
        enum active;
        enum blocked;
        enum completed;
    }

    enum def FlowStatus {
        @XDesignCommon::DocSection { document = "docs/details/02_records_and_evidence.md"; section = "E01"; }
        enum pending;
        enum active;
        enum blocked;
        enum completed;
    }

    enum def CheckpointOutcome {
        @XDesignCommon::DocSection { document = "docs/details/02_records_and_evidence.md"; section = "E04"; }
        enum unevaluated;
        enum failed;
        enum passed;
    }

    enum def CheckKind {
        @XDesignCommon::DocSection { document = "docs/details/01_flows_and_checkpoints.md"; section = "F02"; }
        doc /* How a checkpoint's criteria can be established.
           Syntactic criteria are fixed as deterministic checks in code or scripts,
           so the flow can run them unchanged for every assessment. Semantic
           criteria are abstract descriptions that need judgment. */

        enum syntactic;
        enum semantic;
    }

    enum def JudgeAuthority {
        @XDesignCommon::DocSection { document = "docs/details/01_flows_and_checkpoints.md"; section = "F02"; }
        doc /* Who may judge a checkpoint: a fixed script, or an agent or human
           reading its criteria. */

        enum script;
        enum agent;
        enum human;
    }

    enum def ConformanceKind {
        @XDesignCommon::DocSection { document = "docs/details/01_flows_and_checkpoints.md"; section = "F02"; }
        doc /* What a syntactic check establishes over a stage's artifacts. */

        enum artifactPresence;
        enum identifierConformance;
        enum definitionCoverage;
        enum traceConformance;
    }

    metadata def DocSection {
        @XDesignCommon::DocSection { document = "docs/details/05_documentation_contract.md"; section = "D27"; }
        doc /* Locates the exact project documentation section that explains the
           annotated element. The repository-relative document path and stable
           section identifier let a reader move from a model element to its
           explanation; the documentation index provides the reverse route. */

        attribute document : String;
        attribute section : String;
    }

    metadata def DocFigure {
        @XDesignCommon::DiagramPresentation { label = "Figure location"; summary = "Links the model view to its documentation figure"; }
        @XDesignCommon::DocSection { document = "docs/details/05_documentation_contract.md"; section = "D28"; }
        doc /* Locates a model-derived figure in the section that explains it.
           The figure is a repository-relative file under the carrying document's
           sibling assets/ directory and is
           embedded in the named document with its question, source view,
           reading note and limits. The annotation locates the presentation;
           a view's expose declarations select its model content. */

        attribute document : String;
        attribute figure : String;
    }

    metadata def DiagramPresentation {
        @XDesignCommon::DocSection { document = "docs/details/04_diagram_presentation.md"; section = "D23"; }
        doc /* Presentation text for a native model element or view. A short label
           and optional summary keep a focused figure legible; a view's question
           states the concern it answers. They change neither the element's name,
           identity, typing nor relationships. The renderer retains the qualified
           source name and full documentation in its source references and tooltips. */
        attribute label : String;
        attribute summary : String [0..1];
        attribute question : String [0..1];
    }
}
