<!-- document-kit: profile=xdesign-sysml-docs@1.3.0 class=interface template=generic-interface language=zh-CN contract=0000000000000000000000000000000000000000000000000000000000000000 digest-version=3 -->
# 虚构 Aster 最新读数接口

> 状态: 虚构示例 | 修订: 1 | 模型来源: `mbse/model/example.sysml`

<a id="I10400"></a>

## 1. 目的与参与者

**边界:** 接口只提供缓存读数的只读访问，验证、存储和采样实现均位于接口之后。

**参与者:** 外部 Controller 是调用方，Aster 导出适配器是服务提供方。

<!-- document-kit:content:purpose:start -->
该接口允许外部控制器读取 Aster 最新有效样本，同时隐藏缓存修改和探头控制。其边界仅覆盖只读查询，因而不会把内部验证、存储或采样实现泄露为公共契约。
<!-- document-kit:content:purpose:end -->

<a id="I10401"></a>

## 2. 接口契约

**操作:** getLatestReading(sensorId) 是同步只读操作，终止结果为 Reading、NOT_READY 或 UNKNOWN_SENSOR。

**数据结构:** Reading 包含 sensorId、utcTimestamp、摄氏数值和 quality；响应不暴露缓存索引、内部对象引用或写入入口。

<!-- document-kit:content:contract:start -->
getLatestReading(sensorId) 对已知传感器返回 Reading，对没有有效样本的传感器返回 NOT_READY，对未知标识返回 UNKNOWN_SENSOR。返回值不包含可修改缓存的内部引用。
<!-- document-kit:content:contract:end -->

<a id="I10402"></a>

## 3. 错误与不变量

**错误行为:** 无法识别的标识返回 UNKNOWN_SENSOR，空缓存返回 NOT_READY；传输层故障不属于本接口契约。

**不变量:** 操作不得创建、修改、重排或删除读数，也不得虚构时间戳或测量值。

<!-- document-kit:content:errors:start -->
空历史以 NOT_READY 表示数据尚未就绪，未知传感器以 UNKNOWN_SENSOR 表示请求标识无效。两者语义不同，但都不改变缓存，也不得以旧值或新时间戳构造替代响应。
<!-- document-kit:content:errors:end -->

<a id="I10403"></a>

## 4. 版本管理与兼容性

**版本策略:** 仅在删除字段、改变既有含义或引入新的错误行为时允许增加主版本；可选字段的新增仍属于次版本变化。

**兼容性策略:** 版本 1 读取方可以忽略 quality，版本 2 读取方必须接受旧导出器未提供 quality 的响应。

<!-- document-kit:content:compatibility:start -->
版本 1 的消费者依赖三个基础读数字段，版本 2 仅增加可选 quality，操作名称和既有错误含义保持不变。该演进允许旧消费者忽略新增字段，也要求新消费者接受旧响应缺少该字段。
<!-- document-kit:content:compatibility:end -->

<a id="I10404"></a>

## 5. 模型与验证追踪

**模型元素:** Example::Controller 是虚构读取操作的外部参与者。

**验证链接:** 计划接口用例覆盖 NOT_READY、UNKNOWN_SENSOR、历史不可变性以及版本 1 和版本 2 的字段处理。

<!-- document-kit:content:trace:start -->
契约追踪到 Example::Controller 以及计划中的未知传感器、空历史、有效读数和可选字段兼容性检查。计划链接说明应验证的范围，不表示这些检查已有结果。
<!-- document-kit:content:trace:end -->
