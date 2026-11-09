<!-- document-kit: profile=xdesign-sysml-docs@1.3.0 class=architecture template=generic-architecture language=zh-CN contract=0000000000000000000000000000000000000000000000000000000000000000 digest-version=3 -->
# 虚构 Aster 传感器集线器架构

> 状态: 虚构示例 | 修订: 1 | 模型来源: `mbse/model/example.sysml`

<a id="B10200"></a>

## 1. 背景与边界

**系统边界:** 集线器拥有采样、验证、短期缓冲和导出职责；探头硬件、控制器决策与灌溉执行均不属于内部组成。

**外部参与者:** 本地探头提供原始读数，外部控制器发起查询并接收经过验证的读数。

<!-- document-kit:content:context:start -->
虚构 Aster 集线器位于本地探头与外部控制器之间，负责采样有效性检查和短期可见性，不负责决定灌溉动作。该边界将测量责任与控制责任分离，使外部故障不会被误解为内部采样故障。
<!-- document-kit:content:context:end -->

<a id="B10201"></a>

## 2. 组成与职责

**组成部分:** 本示例由 Collector、UTC 时钟、十项 RingBuffer 和导出适配器构成，不增加与当前职责无关的层次。

**关系:** Collector 只写入有效读数，Exporter 只读取历史副本且不能修改缓存，UTC 时钟为读数提供统一时间基准。

<!-- document-kit:content:composition:start -->
Collector 对每个样本执行验证，RingBuffer 保存最近十个有效读数，Exporter 响应控制器查询。三者按职责划分，使采集可靠性不依赖控制器持续在线。
<!-- document-kit:content:composition:end -->

<a id="B10202"></a>

## 3. 行为与生命周期

**交互:** 探头样本进入 Collector，有效值随后写入 RingBuffer，控制器查询触发 Exporter 返回最新有效值。

**状态:** 空缓冲、正常采集和导出退化三个状态分别描述启动、稳定采样及控制器暂时不可用时的可观察行为。

<!-- document-kit:content:behavior:start -->
启动时缓冲区为空，周期运行阶段仅追加有效样本；导出中断进入退化状态，但采集和缓存继续工作。该状态区分说明控制器不可用不应反向阻断本地采集。
<!-- document-kit:content:behavior:end -->

<a id="B10203"></a>

## 4. 约束与决策

**约束:** 缓存容量为十项：从空缓存开始，保留最新的 min(已收到的有效样本数, 10) 项读数；各边界使用 UTC，拒绝非有限值，导出不得阻塞采集。

**设计依据:** 小型只读历史能够满足故障期间的可追溯查询，同时将采集可靠性与控制器行为解耦，是有意的容量和职责取舍。

<!-- document-kit:content:constraints:start -->
缓存被明确限制为十项，导出接口被限制为只读，以防止慢速控制器无限占用内存或改变测量历史。上述取舍适用于短期监测示例，但不满足长期趋势分析需求。
<!-- document-kit:content:constraints:end -->

<a id="B10204"></a>

## 5. 追踪与验证

**模型元素:** Example::Sensor 和 Example::Controller 分别标识内部采样主体和外部读取方。

**需求链接:** 60 秒采样、UTC 时间戳以及中断期间保留最新有效读数的需求规定可观察目标。

**验证链接:** 计划验证包括缓存容量、先进先出淘汰和导出中断三类场景，当前仅保留验证设计链接。

<!-- document-kit:content:trace:start -->
架构组成可追踪到 Example::Sensor、Example::Controller 以及采样和最新有效值需求。计划中的缓存容量与导出中断用例用于检验关键约束，而非证明这些用例已经执行。
<!-- document-kit:content:trace:end -->
