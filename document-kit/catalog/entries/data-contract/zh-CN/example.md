<!-- document-kit: profile=xdesign-sysml-docs@1.3.0 class=interface template=data-contract language=zh-CN contract=0000000000000000000000000000000000000000000000000000000000000000 digest-version=3 -->
# 虚构 Aster Reading 数据契约

> 状态: 虚构示例 | 修订: 1 | 模型来源: `mbse/model/example.sysml`

<a id="I11400"></a>

## 1. 目的与参与者

**边界:** 契约覆盖 Aster 导出边界的序列化 Reading，不覆盖操作顺序或具体存储技术。

**参与者:** Aster 导出适配器负责序列化，外部 Controller 负责校验并消费数据。

<!-- document-kit:content:purpose:start -->
本数据契约定义交换 Reading 值及其校验和版本规则，不规定控制器如何编排请求。其边界是 Aster 导出端的序列化数据，而不是操作流程或存储技术。
<!-- document-kit:content:purpose:end -->

<a id="I11401"></a>

## 2. 数据契约

**数据结构:** 对象字段为 sensorId:string、utcTimestamp:UTC ISO8601 string、celsius:number，以及可选 quality:valid|suspect。

**必填字段:** sensorId、utcTimestamp 和 celsius 为必填字段，quality 可以缺省，任何必填字段均不接受 null。

<!-- document-kit:content:schema:start -->
Reading 对象必须包含 sensorId、utcTimestamp 和 celsius，quality 为可选字段且不得改变温度或时间戳的含义。缺省与 null 被明确区分，必填字段不得以 null 代替。
<!-- document-kit:content:schema:end -->

<a id="I11402"></a>

## 3. 校验与不变量

**校验规则:** 要求 sensorId 非空、时间戳为 UTC、celsius 有限、quality 合法，并拒绝非有限数值编码。

**不变量:** 必填字段含义保持不变，不得合成时间戳，模式版本不得改变测量来源。

<!-- document-kit:content:validation:start -->
序列化校验拒绝空或非字符串 sensorId、缺失或非 UTC 时间戳、非有限数及未声明的 quality 值。复制与别名所有权是数据模型中的语言侧防护要求，不属于可从编码值观察的序列化校验规则。
<!-- document-kit:content:validation:end -->

<a id="I11403"></a>

## 4. 版本管理与兼容性

**版本策略:** 删除必填字段或改变其类型属于主版本变化，增加可选 quality 属于次版本变化，未知可选字段可被消费者忽略。

**兼容性策略:** 版本 1 消费者通过忽略 quality 接受版本 2 对象，版本 2 消费者接受不含 quality 的版本 1 对象。

<!-- document-kit:content:compatibility:start -->
版本 1 要求三个核心字段，版本 2 增加可选 quality。旧消费者可以忽略该字段，新消费者必须接受其缺省，因此两个方向均不需要强制同步升级。
<!-- document-kit:content:compatibility:end -->

<a id="I11404"></a>

## 5. 模型与验证追踪

**模型元素:** 数据模型中的 Reading 是本序列化契约的虚构语义来源。

**验证链接:** 计划检查包括模式校验、必填字段、有限数值、可选 quality 以及版本 1 和版本 2 的互操作。

<!-- document-kit:content:trace:start -->
契约追踪到数据模型 Reading 以及计划中的有效值、缺失字段、非有限数、未知 quality 和版本兼容用例。计划链接不构成真实编码或互操作结果。
<!-- document-kit:content:trace:end -->
