<!-- document-kit: profile=xdesign-sysml-docs@1.3.0 class=interface template=operation-contract language=zh-CN contract=0000000000000000000000000000000000000000000000000000000000000000 digest-version=3 -->
# 虚构 Aster getCurrentReading 操作契约

> 状态: 虚构示例 | 修订: 1 | 模型来源: `mbse/model/example.sysml`

<a id="I11300"></a>

## 1. 目的与参与者

**边界:** 操作是公共只读查询边界，验证和缓冲属于内部职责，控制器编排属于外部职责。

**参与者:** Controller 调用操作，Aster 导出适配器以只读方式执行。

<!-- document-kit:content:purpose:start -->
getCurrentReading 返回一个已知传感器的最新有效值，不暴露历史修改，也不承担通用数据模式文档的职责。该操作只定义查询边界及其可观察行为。
<!-- document-kit:content:purpose:end -->

<a id="I11301"></a>

## 2. 操作契约

**操作:** getCurrentReading(sensorId) 返回 Reading、NOT_READY、UNKNOWN_SENSOR 或 TIMEOUT，且无重试或修改副作用。

**数据结构:** 请求为 {sensorId: string}；Reading 为 {sensorId, utcTimestamp, celsius, quality?}，其中 celsius 必须为有限数。

<!-- document-kit:content:operation:start -->
请求仅包含 sensorId，成功时返回当前 Reading，不接受时间范围、写入参数或任意缓存索引。操作不包含自动重试，也不产生历史修改副作用。
<!-- document-kit:content:operation:end -->

<a id="I11302"></a>

## 3. 错误与不变量

**错误行为:** UNKNOWN_SENSOR、NOT_READY 和 TIMEOUT 均为终止结果，不得创建、更新或回填 Reading。

**不变量:** 每次请求至多一个终止结果，历史不得改变，成功响应必须是读取时刻最新的有效值。

<!-- document-kit:content:errors:start -->
未知标识在访问缓存前返回 UNKNOWN_SENSOR，空历史返回 NOT_READY，超时返回 TIMEOUT 且不附带数值或回退时间戳。三类结果均终止请求且不创建或回填 Reading。
<!-- document-kit:content:errors:end -->

<a id="I11303"></a>

## 4. 版本管理与兼容性

**版本策略:** 改变必填字段或结果语义属于主版本变化，增加可选响应字段和补充文档属于兼容的次版本变化。

**兼容性策略:** 旧调用方可以忽略 quality，新调用方必须接受 quality 缺省，UNKNOWN_SENSOR 和 NOT_READY 的含义不得改变。

<!-- document-kit:content:compatibility:start -->
版本 1 返回三个必填字段，版本 2 可增加可选 quality，同时保持请求结构、操作名称和全部错误含义。旧调用方可忽略新增字段，新调用方必须接受其缺省。
<!-- document-kit:content:compatibility:end -->

<a id="I11304"></a>

## 5. 模型与验证追踪

**模型元素:** Example::InteractionFlow 仅声明一个空视图名称。顺序行为由虚构的语言侧操作及交互文字规定，所提供模型并未编码该顺序。

**验证链接:** 计划验证包括成功、未知、空历史、超时、无修改以及版本 1 和版本 2 的响应检查。

<!-- document-kit:content:trace:start -->
操作追踪到 Example::InteractionFlow 以及计划中的成功、未知、空历史、超时和兼容性检查。该追踪规定应验证的行为范围，不表示检查已经执行。
<!-- document-kit:content:trace:end -->
