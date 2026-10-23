package XDesignComposition {
    @XDesignCommon::DiagramPresentation { label = "XDesign composition"; }
    @XDesignCommon::DocSection { document = "docs/architecture/system-composition.md"; section = "B01"; }
    private import XDesignCommon::*;
    private import XDesignArchitecture::*;
    private import XDesignWorkflow::*;
    private import XDesignDetailedDesign::*;

    doc /* Integrates the reusable flow definitions and detailed design into one
       XDesign-owned portfolio. The abstract xdesign.'flow' is the common origin;
       every concrete selectable usage specializes it and is owned here. The
       V-model is the default selection, while each wish records the specific
       portfolio member on which it was historically run. */

    part xdesign : XDesign {
        @XDesignCommon::DocSection { document = "docs/architecture/system-composition.md"; section = "B01"; }

        part cli : StageCli;
        bind cli.controller = controller;

        part vModel : VModel :> 'flow' subsets availableFlows {
            @XDesignCommon::DocSection { document = "docs/design/flow.md"; section = "F03"; }
            doc /* The default full V-model path. */
        }
        part workflowFlow : WorkflowFlow :> 'flow' subsets availableFlows {
            @XDesignCommon::DocSection { document = "docs/design/flow.md"; section = "F03"; }
            doc /* Historical four-stage workflow path used by wishes 0001–0004
               and 0007. Its assessment records remain on this exact flow. */
        }
        part wishVerifyFlow : WishVerifyFlow :> 'flow' subsets availableFlows {
            @XDesignCommon::DocSection { document = "docs/design/flow.md"; section = "F03"; }
            doc /* Two-stage path for judged artifacts. */
        }
        part designFlow : DesignFlow :> 'flow' subsets availableFlows {
            @XDesignCommon::DocSection { document = "docs/design/flow.md"; section = "F03"; }
            doc /* Two-stage path for SysML design deliverables. */
        }
        part designedSimpleFlow : DesignedSimpleFlow :> 'flow' subsets availableFlows {
            @XDesignCommon::DocSection { document = "docs/design/flows/simple-flow/definition.md"; section = "D01"; }
            doc /* Executable gated four-stage path with a frozen plan identity. */
        }
        part designedWishVerifyFlow : DesignedWishVerifyFlow :> 'flow' subsets availableFlows {
            @XDesignCommon::DocSection { document = "docs/design/flows/wish-verify/definition.md"; section = "D57"; }
            doc /* Executable gated two-stage path for judged outcomes; both
               exits are judged by a permitted agent or human. */
        }
        part nestedFlowExample : NestedFlowExample :> 'flow' subsets availableFlows {
            @XDesignCommon::DocSection { document = "docs/design/lifecycle.md"; section = "F05"; }
            doc /* Example of a flow with a design-stage subflow. */
        }

        ref part redefines selectedFlow = vModel;
    }

    view stageCliPackageView : StandardViewDefinitions::GeneralView {
        @XDesignCommon::DiagramPresentation {
            label = "Where the two-command CLI is embedded";
            summary = "Focused package projection of the current SysML design";
            question = "Where does the stage CLI connect to the existing xdesign model?";
        }
        @XDesignCommon::DocSection { document = "docs/architecture/system-composition.md"; section = "B01"; }
        @XDesignCommon::DocFigure { document = "docs/architecture/system-composition.md"; figure = "docs/architecture/assets/stage-cli-packages.svg"; }
        doc /* Focused package projection of the stage CLI, its work order and
           agent iteration, the concrete cli.controller binding, the existing
           controller and records, and the planned interaction verification.
           Containment expresses ownership; imports and subject typing express
           model dependencies. This view does not add runtime messages. */
        expose XDesignComposition;
        expose XDesignWorkflow;
        expose XDesignArchitecture;
        expose XDesignVerification;
    }
}
