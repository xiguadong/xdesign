<!-- document-kit: profile=xdesign-sysml-docs@1.1.0 class=architecture template=system-context contract=0000000000000000000000000000000000000000000000000000000000000000 digest-version=2 -->
# Aster system context and boundary

> Status: fictional-example | Revision: 1 | Model sources: `mbse/model/example.sysml`

<a id="B100"></a>

## 1. Context and boundary

**System boundary:** AsterHub owns sampling and buffering; Operator and WeatherService, probe power supply, and irrigation decisions are outside.

**External actors:** Operator requests current readings and sends calibration commands; WeatherService supplies forecast data used only for display context.

**Diagram decisions:** [{"concern":"system-boundary","applies":true,"choice":"figure","rationale":"A boundary view makes inside/outside ownership and exchange direction unambiguous.","asset":"assets/system-context.svg","model_view":"Example::SystemContext"}]

<!-- document-kit:content:boundary:start -->
AsterHub is the only element inside the product boundary. Operator and WeatherService are external actors; the figure shows exchange direction without implying internal decomposition.

![Aster boundary and exchanges](assets/system-context.svg)

> **Figure B100-1 — Aster boundary and exchanges.**  
> **Question:** Which parts are inside Aster and what crosses its boundary?  
> **Model view:** Example::SystemContext  
> **Companions:** The external interface table and component-composition example.  
> **Status:** renderable-now; designed-only  
> **Reading:** AsterHub is inside the dashed boundary; Operator and WeatherService are outside, with readings leaving and calibration commands entering.  
> **Limits:** The view shows context and exchange direction, not internal components, protocols, or deployment nodes.
<!-- document-kit:content:boundary:end -->

<a id="B101"></a>

## 2. External interfaces

**Information flows:** Outgoing Reading and HistoryResponse, incoming CalibrationCommand, and incoming ForecastRecord have distinct meanings and owners.

**Interface participants:** AsterHub is provider for readings and receiver for calibration; Operator consumes readings; WeatherService is a read-only forecast source.

<!-- document-kit:content:interfaces:start -->
Validated readings leave AsterHub for Operator, calibration commands enter from Operator, and forecast records arrive from WeatherService without being stored as sensor measurements.
<!-- document-kit:content:interfaces:end -->

<a id="B102"></a>

## 3. Interactions and lifecycle

**Interactions:** Operator queries and calibrates, WeatherService publishes forecasts, and AsterHub samples and responds independently of forecast availability.

**States:** Online, operator-disconnected, and forecast-unavailable states clarify which external losses do and do not affect collection.

<!-- document-kit:content:behavior:start -->
AsterHub continues sampling when Operator disconnects and labels forecast data as external context rather than using it to alter readings.
<!-- document-kit:content:behavior:end -->

<a id="B103"></a>

## 4. Constraints and decisions

**Constraints:** History is read-only externally, forecast data is informational, and calibration applies only to subsequent samples.

**Design rationale:** These boundaries prevent external systems from silently corrupting measurement provenance or becoming hidden sampling dependencies.

<!-- document-kit:content:constraints:start -->
No external actor may rewrite buffered history, and calibration commands never change the meaning of the Reading timestamp or measured value.
<!-- document-kit:content:constraints:end -->

<a id="B104"></a>

## 5. Trace and verification

**Model elements:** Example::SystemContext, Example::Sensor, Example::Controller, and Example::WeatherService define the context participants and view.

**Verification links:** A planned context review checks boundary placement, actor coverage, exchange direction, and the absence of internal-only details.

<!-- document-kit:content:trace:start -->
Example::SystemContext names the fictional view behind the figure; the component-composition and operation-contract examples own internal and protocol details.
<!-- document-kit:content:trace:end -->
