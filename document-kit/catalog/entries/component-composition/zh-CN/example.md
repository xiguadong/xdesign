<!-- document-kit: profile=xdesign-sysml-docs@1.3.0 class=architecture template=component-composition language=zh-CN contract=0000000000000000000000000000000000000000000000000000000000000000 digest-version=3 -->
# 虚构 Aster 采集缓存与导出组成

> 状态: 虚构示例 | 修订: 1 | 模型来源: `mbse/model/example.sysml`

<a id="B10910"></a>

## 1. 背景与边界

**系统边界:** Collector、UtcClock、RingBuffer 和 Exporter 是四个内部部件。

**外部参与者:** Probe 和 Operator 仅作为接口端点出现，以保持内部职责和外部所有权的区分。

<!-- document-kit:content:context:start -->
虚构 AsterHub 边界内的四个内部部件分别承担采集、UTC 授时、保留和查询职责。Probe 与 Operator 保持为外部接口端点。
<!-- document-kit:content:context:end -->

<a id="B10911"></a>

## 2. 组成部分与职责

**组成部分:** Collector、UtcClock、RingBuffer 和 Exporter 是具有明确职责的全部部件，无需进一步分解。

**职责:** Collector 拒绝无效值，UtcClock 提供 UTC，RingBuffer 按最旧优先淘汰，Exporter 返回最新值或 NOT_READY。

**图示决策:** [{"concern":"component-structure","applies":true,"choice":"table","rationale":"所提供表格已完整分配四项职责和三条依赖，可直接核对所有权及接口方向。","asset":null,"model_view":null}]

<!-- document-kit:content:parts:start -->
四项职责和三条内部依赖在下表中明确分配。该有界分配使接口方向可直接核对，无需额外组成图。

| 部件 | 职责 | 内部依赖 |
| --- | --- | --- |
| Collector | 校验并追加读数 | 向 UtcClock 获取 UTC，通过 RingBuffer.append 写入 |
| UtcClock | 提供 UTC 时刻 | 无 |
| RingBuffer | 按 FIFO 顺序保留至多十项有效值 | 无 |
| Exporter | 读取最新不可变副本或 NOT_READY | 仅依赖 RingBuffer.readLatest |
<!-- document-kit:content:parts:end -->

<a id="B10912"></a>

## 3. 关系与接口

**关系:** 内部依赖仅为 Collector→UtcClock 获取时间、Collector→RingBuffer 追加以及 Exporter→RingBuffer 调用 readLatest。

**接口:** RingBuffer 向 Collector 提供 append(Reading)，向 Exporter 提供 readLatest()；Exporter 不具备任何修改接口。

<!-- document-kit:content:relationships:start -->
Collector 依赖 UtcClock 并向 RingBuffer 写入，Exporter 只依赖 RingBuffer 的读取接口，不能调用 append 或淘汰操作。该依赖方向防止查询路径反向改变采集历史。
<!-- document-kit:content:relationships:end -->

<a id="B10913"></a>

## 4. 行为与生命周期

**交互:** 验证先于追加，readLatest 返回不可变副本，导出失败不会传播到采集过程。

**状态:** 空、部分填充和已满三种缓存状态分别对应 NOT_READY、正常追加和 FIFO 淘汰行为。

<!-- document-kit:content:behavior:start -->
Exporter 不可用时 Collector 继续追加，第十一个有效样本只淘汰最旧保留项。读取返回副本，导出故障不会反向传播到采集过程，因此各职责可以独立验证。
<!-- document-kit:content:behavior:end -->

<a id="B10914"></a>

## 5. 约束与决策

**约束:** 每个缓存只有一个写入者，导出只读，容量固定为十项，控制器不得回调采集过程。

**设计依据:** 单写入者 FIFO 和窄接口使顺序、所有权及故障隔离可以直接检查，并降低无关实现技术对职责分配的干扰。

<!-- document-kit:content:constraints:start -->
本组成刻意不引入共享数据库或控制器双向回调，避免所有权模糊和故障反向传播。单写入者、只读导出和固定容量是保证可检查性的关键约束。
<!-- document-kit:content:constraints:end -->

<a id="B10915"></a>

## 6. 追踪与验证

**模型元素:** Example::Sensor 是已提供的拥有参与者；Collector、UtcClock、RingBuffer 和 Exporter 属于语言侧职责，不声明已有模型实现。

**验证链接:** 计划验证覆盖容量、只读导出、FIFO 淘汰和控制器中断，以检验组成行为。

<!-- document-kit:content:trace:start -->
组成追踪到 Example::Sensor 和交互设计，缓存淘汰与导出隔离用例覆盖主要关系。Collector、RingBuffer 和 Exporter 作为语言侧职责被明确标识，不虚构其模型实现。
<!-- document-kit:content:trace:end -->
