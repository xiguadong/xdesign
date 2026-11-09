<!-- document-kit: profile=xdesign-sysml-docs@1.3.0 class=architecture template=system-context language=zh-CN contract=0000000000000000000000000000000000000000000000000000000000000000 digest-version=3 -->
# 虚构 Aster 系统背景与边界

> 状态: 虚构示例 | 修订: 1 | 模型来源: `mbse/model/example.sysml`

<a id="B10800"></a>

## 1. 背景与边界

**系统边界:** AsterHub 负责采样和缓冲；Operator、WeatherService、探头供电及灌溉决策均位于边界之外。

**外部参与者:** Operator 查询当前读数并发送校准命令，WeatherService 仅提供用于显示背景的天气预报。

**图示决策:** [{"concern":"system-boundary","applies":true,"choice":"table","rationale":"三个参与者和两类交换方向用表格即可无歧义地表达，图形不会增加新的边界信息。","asset":null,"model_view":null}]

<!-- document-kit:content:boundary:start -->
AsterHub 是产品边界内唯一的系统主体，Operator 和 WeatherService 均为外部参与者。本例仅有三个参与者和两类主要交换方向，表格足以准确呈现归属与方向，因此不再增加背景图。

| 边界要素 | 归属与交换关系 |
| --- | --- |
| AsterHub | 边界内负责采样、缓冲和响应 |
| Operator | 边界外读取 Reading 并发送 CalibrationCommand |
| WeatherService | 边界外提供 ForecastRecord，不参与测量写入 |
<!-- document-kit:content:boundary:end -->

<a id="B10801"></a>

## 2. 外部接口

**信息流:** 出向 Reading 与 HistoryResponse、入向 CalibrationCommand 和入向 ForecastRecord 分别具有独立的生产者、消费者和语义。

**接口参与者:** AsterHub 是读数提供者和校准接收者，Operator 消费读数，WeatherService 是只读预报来源。

<!-- document-kit:content:interfaces:start -->
经过验证的读数由 AsterHub 流向 Operator，校准命令由 Operator 流入，天气预报则由 WeatherService 独立到达。ForecastRecord 不会被保存为传感器测量，因而三类信息的所有权和语义不会混淆。
<!-- document-kit:content:interfaces:end -->

<a id="B10802"></a>

## 3. 交互与生命周期

**交互:** Operator 执行查询和校准，WeatherService 发布预报，AsterHub 独立采样并响应，预报可用性不参与测量控制。

**状态:** 在线、操作者断连和预报不可用三个状态用于区分外部缺失是否影响采集行为。

<!-- document-kit:content:behavior:start -->
Operator 断开时 AsterHub 继续采样，天气预报不可用时仅标记外部背景缺失，不改变任何测量值。该行为说明两类外部依赖的消失具有不同影响，不能统一解释为采集中断。
<!-- document-kit:content:behavior:end -->

<a id="B10803"></a>

## 4. 约束与决策

**约束:** 历史对外只读，预报仅作背景信息，校准只影响命令接受后的样本。

**设计依据:** 明确限制外部写入和校准作用时点，可以保护测量溯源并避免天气预报成为采样正确性的隐式前提。

<!-- document-kit:content:constraints:start -->
任何外部参与者都不得改写已缓冲历史，校准命令不得改变 Reading 时间戳和测量值的含义。通过这些约束，外部输入不能悄悄破坏测量来源或成为隐藏的采样依赖。
<!-- document-kit:content:constraints:end -->

<a id="B10804"></a>

## 5. 追踪与验证

**模型元素:** Example::SystemContext、Example::Sensor、Example::Controller 和 Example::WeatherService 定义背景参与者及视图。

**验证链接:** 计划背景评审检查边界位置、参与者覆盖、交换方向以及内部细节是否被误写入本文。

<!-- document-kit:content:trace:start -->
Example::SystemContext 标识本虚构背景的模型视图，component-composition 和 operation-contract 示例分别拥有内部组成与协议细节。计划评审只检查边界和交换，不据此声称内部实现已经验证。
<!-- document-kit:content:trace:end -->
