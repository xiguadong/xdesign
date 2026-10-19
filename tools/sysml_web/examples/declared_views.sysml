package DeclaredViewExample {
    private import Views::*;

    state def Lifecycle {
        first start then received;

        state received;
        state active;
        state delivered;

        transition beginWork first received then active;
        transition acceptDelivery first active then delivered;
    }

    view wishLifecycle : StandardViewDefinitions::GeneralView {
        expose Lifecycle::**;
    }

    view lifecycleTree {
        expose Lifecycle;
        render asTreeDiagram;
    }
}
