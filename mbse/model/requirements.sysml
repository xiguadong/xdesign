package XDesignRequirements {
    @XDesignCommon::DiagramPresentation { label = "Requirements"; }
    @XDesignCommon::DocSection { document = "docs/requirements/requirements.md"; section = "R01"; }
    private import ScalarValues::*;
    private import Requirements::*;

    doc /* The requirements baseline for this tool, written outwards from requirements.
       A requirement states what the tool must do and records why it must; a wish that
       asked for it is named in that wish's own record, never here. Nothing in this
       package references a wish. */

    requirement def XDesignRequirement :> DesignConstraintCheck {
        @XDesignCommon::DocSection { document = "docs/requirements/requirements.md"; section = "R01"; }
        doc /* A requirement on this tool, carrying the reason it exists, so a reader can
           judge the requirement instead of taking it on authority. */
        attribute rationale : String;
    }

    requirement checkpointSourceIsProjectOwned : XDesignRequirement {
        @XDesignCommon::DocSection { document = "docs/requirements/requirements.md"; section = "R16"; }
        doc /* A reusable checkpoint definition and its scripts belong to the
           selected project's engineering source. A run retains its resolved
           rules, criteria and executor selection separately from mutable source. */
        attribute redefines rationale = "Installing or updating xdesign must not move a user's project checks into the tool repository or rewrite a previously registered definition.";
    }

    requirement configuredChecksRetainAuthority : XDesignRequirement {
        @XDesignCommon::DocSection { document = "docs/requirements/requirements.md"; section = "R16"; }
        doc /* Configured evaluations retain the actual script or agent authority,
           executor identity, assessed revision and evidence. A CLI agent's answer
           must not be represented as deterministic script evidence. */
        attribute redefines rationale = "Executor selection changes the strength and repeatability of an assessment; readers need the actual authority to interpret its result.";
    }

    requirement operatorDelegatesRunState : XDesignRequirement {
        @XDesignCommon::DocSection { document = "docs/requirements/requirements.md"; section = "R16"; }
        doc /* The embedded local operator projects stored run facts and delegates
           lifecycle changes to init/next/check. Checkpoint authoring changes source;
           it does not create a passed assessment or start stage work. */
        attribute redefines rationale = "A second mutable UI status or an implicit stage-work launch would make the displayed history disagree with the flow's recorded decisions.";
    }

    requirement sharedChecksHavePinnedOrigins : XDesignRequirement {
        @XDesignCommon::DocSection { document = "docs/requirements/requirements.md"; section = "R16"; }
        doc /* Proposed cross-project obligation, not implemented: an imported
           definition identifies its namespace and upstream revision and pins the
           exact definition, executor bundle and project input mapping. A later
           upstream update cannot change an existing binding. */
        attribute redefines rationale = "The same local slug or script path can name unrelated bytes in two projects; sharing requires immutable content identity and explicit adaptation.";
    }

    requirement crossProjectHistoryKeepsItsOwner : XDesignRequirement {
        @XDesignCommon::DocSection { document = "docs/requirements/requirements.md"; section = "R16"; }
        doc /* Proposed federation obligation, not implemented: a history reference
           identifies project, store, item, immutable revision and assessment.
           Aggregation remains a projection of each project's authority and shows
           unavailable sources explicitly. Preservation also requires a verified
           backup/restore contract before the store is a sole archive. */
        attribute redefines rationale = "A project path or wish ID is not globally unique, and a central status cache cannot preserve the evidence or authority of its source projects.";
    }

    requirement checkpointKindIsDeclared : XDesignRequirement {
        @XDesignCommon::DocSection { document = "docs/requirements/requirements.md"; section = "R01"; }
        doc /* Every checkpoint declares whether its criteria are established
           syntactically, by a fixed code or script check, or semantically, by
           judgment. */
        attribute redefines rationale = "A flow can route a check to the right executor, a script or a judging reader, only if the checkpoint says which kind it is.";
    }

    requirement syntacticCheckIsFixed : XDesignRequirement {
        @XDesignCommon::DocSection { document = "docs/requirements/requirements.md"; section = "R01"; }
        doc /* A syntactic checkpoint names at least one checker whose command fixes the
           check, so the flow runs it unchanged rather than judging afresh. */
        attribute redefines rationale = "Repeating the same criteria by hand invites drift; a fixed command makes one check reproducible and its output comparable across revisions.";
    }

    requirement syntacticAssessmentIsScripted : XDesignRequirement {
        @XDesignCommon::DocSection { document = "docs/requirements/requirements.md"; section = "R01"; }
        doc /* An assessment of a syntactic checkpoint is judged by a script, and the
           script's output for the assessed revision is its evidence. */
        attribute redefines rationale = "A script cannot rationalize a result, and its output can be re-examined later, which is what makes the evidence worth keeping.";
    }

    requirement semanticCheckNamesItsJudges : XDesignRequirement {
        @XDesignCommon::DocSection { document = "docs/requirements/requirements.md"; section = "R01"; }
        doc /* A semantic checkpoint names no checker and states whether an agent or a
           human may judge it. */
        attribute redefines rationale = "Criteria such as whether parts address requirements cannot be reduced to a command, so the checkpoint must say who is competent to judge them.";
    }

    requirement checkerStatesItsConformance : XDesignRequirement {
        @XDesignCommon::DocSection { document = "docs/requirements/requirements.md"; section = "R01"; }
        doc /* A checker states which kinds of conformance it establishes over the
           stage's artifacts: artifact presence, identifier conformance, definition
           coverage or trace conformance. */
        attribute redefines rationale = "A reader who knows what a check covers also knows what it leaves uncovered, and that uncovered part is where judgment is still needed.";
    }

    requirement wishProductsAreRetained : XDesignRequirement {
        @XDesignCommon::DocSection { document = "docs/requirements/requirements.md"; section = "R01"; }
        doc /* The requirements a wish produces, and the model modifications it causes,
           are retained in this model. A wish's own record may describe them, but it
           never holds the only copy. */
        attribute redefines rationale = "A wish record is a working record: it is rewritten as a wish advances and may move out of SysML entirely. Design kept only there would be lost to the project, so the model keeps the substance while the wish record keeps the account of how it came about.";
    }

    requirement modelStandsWithoutWishes : XDesignRequirement {
        @XDesignCommon::DocSection { document = "docs/requirements/requirements.md"; section = "R01"; }
        doc /* The project model resolves and checks on its own. No model element names a
           wish or a wish record, and no model element carries a wish's identity; a wish
           may reference model elements, never the reverse. */
        attribute redefines rationale = "A model that needed wish records in order to check would make those records load-bearing, and then rewriting one as a wish advances, or moving wish records out of SysML, would break the model.";
    }

    requirement flowRunIsRealizedByDesign : XDesignRequirement {
        @XDesignCommon::DiagramPresentation { label = "Run has a concrete design"; summary = "Run class, factory and operations"; }
        @XDesignCommon::DocSection { document = "docs/requirements/requirements.md"; section = "R01"; }
        doc /* A flow's run is realized by design elements: a class for the run, a
           factory that creates it, and the functions that evaluate its gates and
           advance it. */
        attribute redefines rationale = "The architecture says what a flow is made of; only a design says what it is made into. Without one, every implementation of a flow re-invents its run, its visits and the way its gates are judged.";
    }

    requirement designedFlowGatesEveryStage : XDesignRequirement {
        @XDesignCommon::DiagramPresentation { label = "Every stage has an exit gate"; summary = "Advancement requires explicit authorization"; }
        @XDesignCommon::DocSection { document = "docs/requirements/requirements.md"; section = "R01"; }
        doc /* Every stage of a designed flow declares the exit checkpoint that must
           pass before the flow leaves it. */
        attribute redefines rationale = "A stage left without a gate completes on the mere passage of work; gating every stage makes a designed flow's advancement checkable everywhere, not only at its end.";
    }

    requirement gateSyntaxIsRunnable : XDesignRequirement {
        @XDesignCommon::DocSection { document = "docs/requirements/requirements.md"; section = "R01"; }
        doc /* Every syntactic gate's checker names a command that exists in the
           repository; the flow runs it unchanged, and its output for the assessed
           revision is the evidence. */
        attribute redefines rationale = "A gate whose command does not exist can never pass by its own rule, and the work would either stall at the gate or fall back to unrecorded judgment.";
    }

    requirement runRecordIsScriptChecked : XDesignRequirement {
        @XDesignCommon::DocSection { document = "docs/requirements/requirements.md"; section = "R01"; }
        doc /* The conformance of a run's record — at least one visit per stage,
           assessments cited at the visited revision, evidence that exists — is
           established by a checker, not by reading. */
        attribute redefines rationale = "These invariants were kept by reading; reading does not re-run per revision and does not fail a record rather than trusting a reader to notice.";
    }

    requirement designCarriesItsDocSections : XDesignRequirement {
        @XDesignCommon::DocSection { document = "docs/requirements/requirements.md"; section = "R01"; }
        doc /* Every element of the detailed design carries the documentation section
           that covers it, and the design's own checker establishes that the sections
           are present and well formed. */
        attribute redefines rationale = "A design read from declarations alone forces the reader to reconstruct the reasoning; the section carries it, and a checker keeps the map from rotting as the design changes.";
    }

    requirement checkerRulesAreDeclared : XDesignRequirement {
        @XDesignCommon::DocSection { document = "docs/requirements/requirements.md"; section = "R01"; }
        doc /* Every checker of a designed flow declares the rules by which its
           outcome is determined — each rule's exact condition and what it reads —
           and its command reports each rule's verdict; the design's check
           establishes that the declared rules and the command's rule list agree. */
        attribute redefines rationale = "A checker that only names a command hides its logic in code, and the model cannot say why a gate passed or what would make it fail. Declared rules make the outcome follow from stated conditions, and the agreement check keeps the model's copy and the command's copy from drifting apart.";
    }

    requirement structureGraphsAreDocumented : XDesignRequirement {
        @XDesignCommon::DiagramPresentation { label = "Nine views document the design"; summary = "Each view selects native elements and locates its figure"; }
        @XDesignCommon::DocSection { document = "docs/requirements/requirements.md"; section = "R01"; }
        doc /* A designed flow declares nine complementary views of the same model:
           block definition, internal block, requirements, parametric, use case,
           activity, sequence, state machine and package. Each view selects native
           model elements, locates a rendered figure and its documentation section,
           and states what question it answers. A check establishes that every
           declared figure exists and is referenced. */
        attribute redefines rationale = "A timeline, a logical graph and an interface graph alone leave actor goals, requirement satisfaction and verification, authorization constraints, lifecycle guards and package dependencies unexplained. Nine coordinated views expose these concerns without maintaining a second model.";
    }

    requirement runRealizesItsDesign : XDesignRequirement {
        @XDesignCommon::DocSection { document = "docs/requirements/requirements.md"; section = "R01"; }
        doc /* A run of a designed flow is recorded with the design's class, so the
           design is realized by the run rather than declared beside it. */
        attribute redefines rationale = "A class nothing is typed by is a picture of a run, not a run's design; the typing is what makes the design load-bearing, and a check can establish it.";
    }

    requirement diagramViewsAreReadable : XDesignRequirement {
        @XDesignCommon::DocSection { document = "docs/requirements/requirements.md"; section = "R01"; }
        @XDesignCommon::DiagramPresentation { label = "Nine views are readable"; summary = "Review final figures at article size"; }
        doc /* The nine primary SimpleFlow documentation views are legible at a
           900-pixel article width, with displayed diagram text at least 11 pixels,
           no clipped labels or node overlaps, and relationships whose endpoints
           remain identifiable. Each view focuses on one question and uses its
           own diagram notation: visible internal boundaries and ports, parameter
           bindings, actor goals inside the subject boundary, and readable guards.
           Full-model membership trees remain expandable reference overviews.
           Final SVGs and their matching previews receive a visual review in
           addition to compiler and semantic checks. */
        attribute redefines rationale = "A complete inventory of model elements does not explain their relationships when labels shrink, edges obscure one another or diagram notation is flattened. A measured display size and review of the actual delivered figures make readability an explicit delivery obligation.";
    }

    requirement diagramPresentationRetainsSourceIdentity : XDesignRequirement {
        @XDesignCommon::DocSection { document = "docs/requirements/requirements.md"; section = "R01"; }
        @XDesignCommon::DiagramPresentation { label = "Figures retain model identity"; summary = "Short labels preserve native source references"; }
        doc /* Documentation diagrams derive their selected elements and
           relationships from native SysML views and compiler exports. Optional
           presentation metadata supplies short labels, summaries and the question
           a view answers while native qualified names, full descriptions and
           endpoint identities remain available through tooltips or source
           references. A final figure manifest records source and SVG hashes,
           generation parameters and the preview correspondence used in review,
           so stale previews cannot authorize changed figures. */
        attribute redefines rationale = "Readable composition needs less text on the page, but readers must still be able to identify the actual modeled element, inspect omitted detail and reproduce the figure that was accepted. Presentation metadata and final-artifact provenance prevent a second, disconnected diagram model.";
    }

    requirement documentationEntryMapsDesign : XDesignRequirement {
        @XDesignCommon::DocSection { document = "docs/requirements/requirements.md"; section = "R01"; }
        doc /* The documentation set has one entry document that gives each reader a
           route through the current documents and maps the model's modules and
           current design concerns to their detailed sections. */
        attribute redefines rationale = "Without an entry and a model-to-document map, readers must infer the document set and reconstruct the relationship between the system design and its explanations.";
    }

    requirement documentationSupportsMultipleFlows : XDesignRequirement {
        @XDesignCommon::DocSection { document = "docs/requirements/requirements.md"; section = "R01"; }
        doc /* The documentation structure has an independently addressable overview
           for each defined flow and a separate detailed location for each developed
           flow design. Another flow can be added without renumbering or recasting
           existing flow sections. */
        attribute redefines rationale = "The project can run more than one flow; a document hierarchy tied to the current SimpleFlow would hide another flow or force an unrelated design into its pages.";
    }

    requirement figuresStayWithDesignExplanation : XDesignRequirement {
        @XDesignCommon::DocSection { document = "docs/requirements/requirements.md"; section = "R01"; }
        doc /* Each documented design figure is embedded in the subsection whose
           design question it answers and accompanied there by its native view,
           companion views, status, reading guidance, and limits. */
        attribute redefines rationale = "A disconnected image catalog makes readers join the design claim, source projection, and interpretation themselves; a local explanation lets them judge what the figure establishes.";
    }

    requirement modelDocumentationLinksResolve : XDesignRequirement {
        @XDesignCommon::DocSection { document = "docs/requirements/requirements.md"; section = "R01"; }
        doc /* A documentation check resolves every DocSection document and stable
           section anchor, every DocFigure asset and carrying document, and each
           figure's embed in that document; the entry's module-level reverse index
           agrees with the annotated modules and sections it lists. */
        attribute redefines rationale = "A locator that names a missing section or unembedded figure gives false confidence that the model is explained; a repeatable audit detects drift as either side changes.";
    }

    requirement viewAndViewpointRolesAreSpecified : XDesignRequirement {
        @XDesignCommon::DocSection { document = "docs/requirements/requirements.md"; section = "R01"; }
        doc /* The documentation specification distinguishes a viewpoint's question
           and selection convention from a native SysML view's selected elements,
           and identifies how the view, DocFigure, rendered asset, and carrying
           DocSection answer that question together. It states applicability of
           the nine reference diagram concerns for each designed flow. */
        attribute redefines rationale = "An image alone cannot tell a reader which concern selected its elements or whether omitted diagrams were inapplicable; explicit roles keep the figure tied to the model and expose coverage decisions.";
    }

    requirement checkerRulesAreImplementationReady : XDesignRequirement {
        @XDesignCommon::DocSection { document = "docs/requirements/requirements.md"; section = "R01"; }
        doc /* Each documented design-stage checker rule states its input artifacts,
           exact decision condition, failure diagnostic, evidence output, and
           current implementation status so a later implementation can reproduce
           the specified result. */
        attribute redefines rationale = "Rule names and broad criteria leave later implementers to invent the actual check; operational inputs, conditions and outputs make the intended gate reproducible while status prevents a planned check being mistaken for a running one.";
    }

    requirement rustUmlDesignMapsEveryDesignElement : XDesignRequirement {
        @XDesignCommon::DocSection { document = "docs/requirements/requirements.md"; section = "R01"; }
        doc /* A Rust software design of a designed flow names one Rust element for
           each of the flow's design elements — the run, the factory, the checkpoint
           class and its adaptations, the gate and advance functions, the records,
           the statuses and the automation ports — and records which SysML design
           element each Rust element realizes. */
        attribute redefines rationale = "An implementation that starts from an unmapped design re-decides the structure the design already fixed, and a reviewer cannot tell whether the crate covers the design or quietly drops part of it; the mapping makes coverage inspectable before any code exists.";
    }

    requirement umlViewsAreSelectedWithReasons : XDesignRequirement {
        @XDesignCommon::DocSection { document = "docs/requirements/requirements.md"; section = "R01"; }
        doc /* For a module's UML design, every UML view of the project's UML guide
           is marked required, recommended or not applicable with a reason, and each
           adopted view is embedded in the section whose question it answers, with
           its figure block and an honest render status. */
        attribute redefines rationale = "A design that shows only the views someone happened to draw hides whether the others were considered; stated applicability lets a reviewer distinguish a reasoned omission from an overlooked one, and the figure block keeps each adopted view tied to its question and source.";
    }

    requirement softwareDesignClaimsNoImplementation : XDesignRequirement {
        @XDesignCommon::DocSection { document = "docs/requirements/requirements.md"; section = "R01"; }
        doc /* A software design document distinguishes designed elements from
           implemented code and claims no implementation, test result or build that
           does not exist in the repository. */
        attribute redefines rationale = "A design read as a finished product sends integrators looking for code that is not there and hides the work that remains; an explicit status keeps the design usable without pretending to be its realization.";
    }

    requirement stageCliUsesUniformInteraction : XDesignRequirement {
        @XDesignCommon::DocSection { document = "docs/requirements/requirements.md"; section = "R01"; }
        doc /* Every supported stage exposes the same next/check interaction.
           next returns current work without writes. check records collected outputs,
           assesses every exit gate, advances only when authorized, and returns the
           assessed visit plus correction work, successor work, or completion. */
        attribute redefines rationale = "Agents should follow stage content rather than coordinate separate recording, assessment and progression commands.";
    }
    requirement stageCliPreservesRunIntegrity : XDesignRequirement {
        @XDesignCommon::DocSection { document = "docs/requirements/requirements.md"; section = "R01"; }
        doc /* Independent command processes restore the same registered run and
           frozen work definitions. Reject corrupt records, changed plans, stale
           submissions and concurrent writes. Semantic judgments bind to the exact
           collected artifact snapshot and never substitute for fixed checker reports.
           Completed next/check calls are read-only; registration is explicit. */
        attribute redefines rationale = "A short command interface is useful only if process restarts and corrections preserve evidence, authority and stage identity.";
    }

    requirement operationalRecordsHaveDurableAuthority : XDesignRequirement {
        @XDesignCommon::DocSection { document = "docs/requirements/requirements.md"; section = "R01"; }
        doc /* New wish registrations, flow run facts, revision manifests, material
           and evidence bytes, and operation receipts persist in a local operational
           store outside regenerable cache. Git retains engineering definitions and
           products; existing Git wish traces remain legacy records. */
        attribute redefines rationale = "A run whose only authoritative record lives under cache can lose the account of what executed and which bytes were judged; separating operational facts from engineering products gives each an explicit durable authority.";
    }

    requirement wishHistoryIsRevisioned : XDesignRequirement {
        @XDesignCommon::DocSection { document = "docs/requirements/requirements.md"; section = "R01"; }
        doc /* A generic store item has a stable identity and kind. Each wish save
           appends an immutable, numbered revision containing typed metadata and a
           complete URI-to-material manifest. List, get, history, read and checkout
           expose the selected revision without deleting earlier revisions. */
        attribute redefines rationale = "A mutable current JSON snapshot cannot explain earlier decisions or reproduce an earlier checked state; complete retained revisions make wish history inspectable and recoverable without hard-coding the store to one future item kind.";
    }

    requirement materialsAreContentAddressed : XDesignRequirement {
        @XDesignCommon::DocSection { document = "docs/requirements/requirements.md"; section = "R01"; }
        doc /* Every stored material is identified by the SHA-256 of its exact bytes
           and bound to a logical URI in each revision manifest. Publication is atomic,
           identical bytes deduplicate, and read and checkout verify byte length and
           digest through end of file before returning content. */
        attribute redefines rationale = "A path alone does not identify the evidence that was judged; content identity prevents later edits from silently replacing historical bytes and makes loss or corruption detectable.";
    }

    requirement storageSaveIsAtomicAndIdempotent : XDesignRequirement {
        @XDesignCommon::DocSection { document = "docs/requirements/requirements.md"; section = "R01"; }
        doc /* A save names an operation ID, exact request bytes and expected head.
           The store durably reserves the operation before blob publication, commits
           the complete revision and receipt in one SQLite transaction, and exposes
           outcome reconciliation. Repeating the same operation and request has no
           second effect; a changed request or stale head cannot overwrite another
           writer. One stage check is saved as one logical operation. */
        attribute redefines rationale = "Process crashes and retries must not duplicate work or leave run rows without their evidence; explicit guards and receipts turn uncertain outcomes into inspectable recovery instead of silent overwrite.";
    }

    requirement workingCacheIsRecoverable : XDesignRequirement {
        @XDesignCommon::DocSection { document = "docs/requirements/requirements.md"; section = "R01"; }
        doc /* Runtime plans, work definitions, wish working sources, run records and
           checker exchanges can be regenerated from the store after their cache copies
           are deleted. Checkout restores every byte in a selected revision manifest,
           and the restored SimpleFlow record remains acceptable to its record checker. */
        attribute redefines rationale = "Treating cache as disposable is safe only when every required operational byte has a durable copy and the restored record remains usable by the existing flow contract.";
    }

    requirement wishStatusProjectsFlowExecution : XDesignRequirement {
        @XDesignCommon::DocSection { document = "docs/requirements/requirements.md"; section = "R01"; }
        doc /* Wish status and current stage shown by store queries are derived from
           the selected flow identity and stored run record. No independently editable
           wish-status flag may contradict the execution position. Generic queries can
           filter by item kind and projected wish status without hard-coding one flow. */
        attribute redefines rationale = "The flow already owns stage and completion facts; duplicating them as a separate status creates two authorities that can drift, while a projection works for future flow definitions.";
    }

    requirement storageInputAndIntegrityAreChecked : XDesignRequirement {
        @XDesignCommon::DocSection { document = "docs/requirements/requirements.md"; section = "R01"; }
        doc /* The store refuses unknown formats and schema versions, malformed or
           duplicate JSON fields, excessive inputs, unsafe logical paths and symlink
           escapes, digest mismatches, and replaced store roots or databases. Doctor
           reports missing, altered, orphan and pending storage state without converting
           integrity defects into successful reads or writes. */
        attribute redefines rationale = "Durability is not trust in whatever bytes happen to occupy a database or path; explicit refusal and diagnosis keep incompatible, unsafe or corrupted state out of engineering decisions.";
    }
}
