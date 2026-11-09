<!-- document-kit: profile=xdesign-sysml-docs@1.3.0 class=design template=data-model language=zh-CN contract=0000000000000000000000000000000000000000000000000000000000000000 digest-version=3 -->
# 虚构 Aster 读数与缓存数据模型

> 状态: 虚构示例 | 修订: 1 | 模型来源: `mbse/model/example.sysml`

<a id="D11200"></a>

## 1. 目的与分配

**设计对象:** Reading 值及其字段、不变量、RingBuffer 所有权和保留语义。

**需求链接:** UTC 时间戳、有限摄氏值、最新有效值保留和历史不可变需求驱动数据模型。

<!-- document-kit:content:purpose:start -->
本数据模型定义 Reading 的含义、所有权和有界缓存如何保持值语义，不描述请求顺序。其核心问题是测量值在验证、保留和导出过程中是否保持同一来源和不可变含义。
<!-- document-kit:content:purpose:end -->

<a id="D11201"></a>

## 2. 实体与字段

**实体:** Reading 是值实体，Sensor 是具有稳定身份的数据来源，RingBuffer 是保留 Reading 值的所有者。

**字段:** sensorId 为非空文本，utcTimestamp 为 UTC ISO 8601 时间，celsius 为有限数，quality 为 valid、suspect 或缺省。

<!-- document-kit:content:entities:start -->
Reading 是由 sensorId、utcTimestamp、celsius 和 quality 组成的值对象，Sensor 身份跨读数保持稳定。RingBuffer 拥有保留值，但不获得修改测量来源的权利。
<!-- document-kit:content:entities:end -->

<a id="D11202"></a>

## 3. 关系与归属

**关系:** Sensor 与 Reading 为 1→多关系，RingBuffer 与 Reading 为 1→0..10 关系，Reading 与 ForecastRecord 无归属关系。

**归属:** AsterHub 在验证后拥有值，RingBuffer 拥有保留顺序，Exporter 仅获得不可变副本。

**图示决策:** [{"concern":"data-relationships","applies":true,"choice":"table","rationale":"两条基数和一项所有权规则用模式表即可准确说明，无需新增关系图。","asset":null,"model_view":null}]

<!-- document-kit:content:relationships:start -->
每项 Reading 恰有一个 Sensor 来源，且至多由一个集线器缓存保留。下表区分来源、保留基数和导出所有权；导出采用值复制，不共享可变别名。

| 关系 | 基数 | 所有权规则 |
| --- | --- | --- |
| Sensor → Reading | 1 → 0..多；每项 Reading 仅有一个来源 | 稳定传感器身份记录来源 |
| RingBuffer → Reading | 1 → 0..10；每个保留值至多属于一个缓存 | 集线器拥有值，缓存拥有 FIFO 保留顺序 |
| Exporter → Reading | 返回值的副本 | 不可变值副本，不共享可变引用 |
| Reading → ForecastRecord | 无 | 不存在归属或语义等同关系 |
<!-- document-kit:content:relationships:end -->

<a id="D11203"></a>

## 4. 不变量与生命周期

**不变量:** celsius 有限、时间戳为 UTC、sensorId 稳定、时间不倒退、导出值不可变以及 FIFO 淘汰均为强制规则。

**生命周期:** Reading 由验证创建，在缓存中保留直至淘汰，导出时复制，任何阶段都不得原地修改。

<!-- document-kit:content:invariants:start -->
同一传感器的时间戳不得倒退，celsius 必须为有限数，缓存淘汰只能移除最旧有效项。Reading 在验证后不被原地编辑，导出副本也不得改变原值。
<!-- document-kit:content:invariants:end -->

<a id="D11204"></a>

## 5. 映射与验证

**源码映射:** Reading、ReadingQuality 和 RingBuffer 是预期数据类型，不预设 ORM 或数据库实现。

**验证链接:** 计划检查覆盖字段有效性、时间戳顺序、基数、不可变性和 FIFO 行为。

<!-- document-kit:content:projection:start -->
实现映射采用不可变 Reading 记录和小型不变量测试集，覆盖有限值、时间顺序、基数和导出复制行为。该映射不暗示任何 ORM 或数据库模式。
<!-- document-kit:content:projection:end -->
