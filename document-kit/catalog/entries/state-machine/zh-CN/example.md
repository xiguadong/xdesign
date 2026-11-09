<!-- document-kit: profile=xdesign-sysml-docs@1.3.0 class=design template=state-machine language=zh-CN contract=0000000000000000000000000000000000000000000000000000000000000000 digest-version=3 -->
# 虚构 Aster 采样生命周期状态机

> 状态: 虚构示例 | 修订: 1 | 模型来源: `mbse/model/example.sysml`

<a id="D11100"></a>

## 1. 目的与分配

**设计对象:** AsterHub 的生命周期状态、事件、监护条件、转换、终止行为和故障恢复。

**需求链接:** 连续采样、明确处理无效数据和保留最新有效值的需求驱动生命周期规则。

<!-- document-kit:content:purpose:start -->
生命周期模型区分 Stopped、Ready、Sampling 和 Fault，使事件处理和故障恢复不隐含在线性流程中。状态的意义由可接受事件和历史行为决定，而不是由名称数量决定。
<!-- document-kit:content:purpose:end -->

<a id="D11101"></a>

## 2. 状态与不变量

**状态:** Stopped、Ready、Sampling 和 Fault 是有行为区分的全部状态，历史在每个状态下都保持可读。

**不变量:** 只有 Sampling 能产生新样本，Fault 不得清空历史，Stopped 不得产生新的时间戳。

**图示决策:** [{"concern":"lifecycle-states","applies":true,"choice":"table","rationale":"四个状态及其监护条件用转换表更便于核对非法事件和历史规则。","asset":null,"model_view":null}]

<!-- document-kit:content:states:start -->
Stopped 不采集，Ready 等待启动，Sampling 周期采集，Fault 暂停采集但保留历史。四个状态及其监护条件用转换表即可准确比较，因此本例不增加状态图。
<!-- document-kit:content:states:end -->

<a id="D11102"></a>

## 3. 事件与监护条件

**事件:** initialize 获取生命周期资源但不清空历史，start 启用采集，stop 暂停，tick 采样，probeFault 进入故障，reset 请求恢复，shutdown 释放资源。

**监护条件:** initialize 要求 Stopped 且资源可用，start 要求 Ready，tick 要求 Sampling 且探头正常，probeFault 要求 Sampling 且检测到故障，reset 要求 Fault 且自检成功，shutdown 要求 Stopped。

<!-- document-kit:content:events:start -->
事件为 initialize、start、stop、tick、probeFault、reset 和 shutdown。转换表规定可接受源状态和监护条件；未列出的组合及监护失败均被拒绝，状态、资源和历史保持不变；终止后拒绝全部事件。
<!-- document-kit:content:events:end -->

<a id="D11103"></a>

## 4. 转换与生命周期

**转换:** initialize 使 Stopped 转为 Ready，start 使 Ready 转为 Sampling，probeFault 使 Sampling 转为 Fault，reset 使 Fault 转为 Sampling，stop 使非终止状态转为 Stopped。

**终止状态:** Stopped 状态下的 shutdown 进入终止状态并释放资源，不再产生读数，也不清空保留历史。

<!-- document-kit:content:transitions:start -->
计划转换表给出完整事件判据。初始状态为 Stopped，历史为空且尚未获取生命周期资源；重新初始化时保留已有历史。

| 源状态 | 事件及监护条件 | 目标状态和动作 | 历史影响 |
| --- | --- | --- | --- |
| Stopped | initialize；资源可用 | Ready；获取资源 | 保持 |
| Ready | start；无附加条件 | Sampling；启用 tick | 保持 |
| Sampling | tick；探头正常 | Sampling；校验并追加 | 追加有效值，满时 FIFO 淘汰；无效值不改变历史 |
| Sampling | probeFault；检测到故障 | Fault；暂停 tick | 保持 |
| Fault | reset；自检成功 | Sampling；恢复下一次新 tick | 保持 |
| 任意非终止状态 | stop；无附加条件 | Stopped；禁用 tick | 保持 |
| Stopped | shutdown；无附加条件 | 终止；释放资源 | 保持 |
| 任意状态 | 未列出的事件或监护失败 | 拒绝；不改变状态或资源 | 保持 |
| 终止状态 | 任意事件 | 拒绝；保持终止 | 保持 |
<!-- document-kit:content:transitions:end -->

<a id="D11104"></a>

## 5. 恢复与约束

**恢复行为:** reset 前先执行自检，失败则保持 Fault，成功后以下一个新的真实样本恢复周期。

**约束:** 不得自动删除历史、虚构恢复样本或在 Fault 状态处理 tick。

<!-- document-kit:content:recovery:start -->
自检失败时保持 Fault 并报告明确的维护状态，自检成功后才允许 reset 恢复采样。恢复后的第一个读数必须使用新的时间戳，不得用回填值模拟恢复成功。
<!-- document-kit:content:recovery:end -->

<a id="D11105"></a>

## 6. 映射与验证

**源码映射:** LifecycleController 和 ProbeSelfTest 是预期实现单元，转换行为通过显式事件日志进行核对。

**验证链接:** 计划检查包括非法 tick、reset 失败、reset 成功、stop、shutdown 和历史保留。

<!-- document-kit:content:projection:start -->
实现映射以显式事件日志和转换表为基础，计划测试覆盖非法事件、监护失败、恢复、终止和历史保持。LifecycleController 与 ProbeSelfTest 仅为预期单元，不代表已有代码。
<!-- document-kit:content:projection:end -->
