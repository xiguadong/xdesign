<!-- document-kit: profile=xdesign-sysml-docs@1.3.0 class=design template=interaction-flow language=zh-CN contract=0000000000000000000000000000000000000000000000000000000000000000 digest-version=3 -->
# 虚构 Aster 当前读数交互流程

> 状态: 虚构示例 | 修订: 1 | 模型来源: `mbse/model/example.sysml`

<a id="D11000"></a>

## 1. 目的与分配

**设计对象:** Controller 与 Sensor 之间的同步请求、验证和响应流程，以及一个超时备选路径。

**需求链接:** 最新值查询、明确拒绝无效输入以及不得虚构读数的需求驱动该交互设计。

<!-- document-kit:content:purpose:start -->
本设计说明一次当前读数请求的顺序，包括标识验证、缓存读取和超时行为，而非静态数据结构。主问题是一次请求如何在不同条件下恰好产生一个明确的终止结果。
<!-- document-kit:content:purpose:end -->

<a id="D11001"></a>

## 2. 参与者与接口

**参与者:** Controller 是请求方，Sensor 是提供方，RingBuffer 仅作为内部只读参与者。

**接口:** ReadRequest 携带 sensorId；ReadingResponse 恰好携带 Reading、NOT_READY、UNKNOWN_SENSOR 或 TIMEOUT 之一，错误结果不携带 Reading 值。

<!-- document-kit:content:participants:start -->
Controller 发起 ReadRequest，Sensor 检查 sensorId 和缓存状态，操作边界最终返回 ReadingResponse 或明确错误。RingBuffer 是内部只读参与者，不应被误画为外部调用方。
<!-- document-kit:content:participants:end -->

<a id="D11002"></a>

## 3. 主交互流程

**交互:** Controller 发送 ReadRequest，Sensor 验证 sensorId 并读取最新有效 Reading，随后返回 ReadingResponse。

**顺序:** 验证先于缓存访问，缓存访问先于响应，超时先于任何迟到结果并终止本次操作。

**图示决策:** [{"concern":"interaction-ordering","applies":true,"choice":"prose","rationale":"一条主路径和三个终止分支可用编号顺序和禁止条件准确说明，无需额外序列图。","asset":null,"model_view":null}]

<!-- document-kit:content:flow:start -->
主流程严格按以下顺序执行：第一，Controller 发送 ReadRequest；第二，Sensor 验证 sensorId；第三，验证成功后读取最新有效 Reading；第四，操作边界返回 ReadingResponse。超时终止当前尝试，截止时间之后不得再产生读数响应。

本例只有一条主路径和三个明确终止分支，有序文字已经完整表达时序，因此不增加序列图。
<!-- document-kit:content:flow:end -->

<a id="D11003"></a>

## 4. 备选流程与错误

**备选流程:** UNKNOWN_SENSOR 跳过读取，NOT_READY 表示空历史，TIMEOUT 是截止时间到达时发出的唯一错误 ReadingResponse；后续提供方响应均被抑制。

**错误行为:** 所有错误都是明确终止结果，不修改历史，也不把旧值或新时间戳组合成替代读数。

<!-- document-kit:content:alternates:start -->
UNKNOWN_SENSOR 在缓存访问前返回，空历史返回 NOT_READY。到达截止时间时，操作边界发出一次不携带 Reading 的 ReadingResponse(TIMEOUT)，并丢弃后续提供方结果，不得重新标记旧值的时间戳。
<!-- document-kit:content:alternates:end -->

<a id="D11004"></a>

## 5. 行为与约束

**行为:** 一次请求至多产生一个终止结果，并且不改变缓存内容、读数顺序或原始时间戳。

**约束:** 禁止终止结果之后再响应、修改历史、隐式重试或返回替代测量值；截止时刻的 TIMEOUT 错误响应本身是允许的。

<!-- document-kit:content:constraints:start -->
该只读单请求设计不覆盖并发重试、认证和重传。每次请求至多产生一个终止 ReadingResponse，包括截止时刻的 TIMEOUT 错误；终止后不得转发迟到的提供方响应或回退值。
<!-- document-kit:content:constraints:end -->

<a id="D11005"></a>

## 6. 映射与验证

**源码映射:** RequestHandler、RequestValidator、ReadLatestExporter 和可控超时器是预期设计单元。

**验证链接:** 计划检查包括有序成功、未知传感器、空历史和超时的事件序列及终止结果唯一性。

<!-- document-kit:content:projection:start -->
计划测试以本页虚构的语言侧交互文字为依据，比较事件顺序及终止结果唯一性。Example::InteractionFlow 仅为空的具名视图声明，不是行为证据；源码单元名称仍为拟议分配。
<!-- document-kit:content:projection:end -->
