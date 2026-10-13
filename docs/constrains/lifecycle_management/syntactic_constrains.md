# 运行观察与生命周期演进的语法及阶段合规约束

规则集版本：0.1。日期：2026-10-12。状态：目标规范；不声明运行观察、生命周期判断或新增量创建已有自动执行实现。

## 适用范围、依据与输入

适用于[生命周期源模型](../../../wish_to_product.sysml)的 `WishToProductLifecycle::operate`、`lifecycleDecision`、`evolve`，对应 `OperateAndObserve`、`DecideLifecycle`、`PrepareNextIncrement`。这是十阶段 V-model 之外的运行与演进范围；源模型只有 `DecideLifecycle::authorization` 的 `Lifecycle disposition` 关卡，不为普通观察或准备动作新增关卡。

依据还包括[整体架构设计 0.2](../../../design/workflow_mbse_architecture.md)的生命周期分类、开发与发布边界及[V-model 的 ReviewAgent、ChangeControl](../../../v_model_agents.sysml)。输入为已放行 `OperationsBundle` 及其成功封存的发布身份、运行配置、观察计划、真实事件与反馈、当前生命周期规则和授权配置；演进另输入当前观察包与有效 `EvolutionDecision`。项目固定初始化工具 T1，约束显式选版本；源模型包与不可变发布身份的映射保存在选定记录中，不由类型声明推定。通用结构要求见[目录总说明](../README.md)、[模型与追溯语法约束](../model_traceability/syntactic_constrains.md)及[运行语法约束](../workflow_execution/syntactic_constrains.md)。

## 允许工作与产出

允许运行成功封存的已发布版本、采集遥测/事件/用户反馈/维护证据、形成版本化观察包，评价生命周期选择，准备新 wish 版本和新增量上下文。运行观察产出 `OperationsBundle`；生命周期选择产出 `EvolutionDecision` 与授权决定；演进产出 `nextWish`、`nextContext` 和变更影响材料，均写入 `development`。从 R1 只读读取和继承后继续开发，完成后由[跨阶段发布操作](../project_release/syntactic_constrains.md)创建新 R2，T1 保持不变。既有发布内容、证据、决定和依赖不被覆盖。

支持材料采用 `operations_observation.md`、`incident_record_<incident-id>.md`、`lifecycle_decision.md`、`increment_change.md`；拆分观察按稳定范围追加标识，由观察包索引路由。原始遥测和事件可使用其实际格式，但需版本化资源引用及清单。其他项目命名在选定契约中给出映射。

## 规则

| ID | 约束判定 | 诊断位置或检查证据 |
| --- | --- | --- |
| LC-SYN-001 | 外层 `productReleased` 只有在所选开发实例 `operationsReady` 时进入 `operate`，且 `releasedProduct` 是同一 `development.product`。观察记录固定实际运行产品的身份、基线、配置、时间区间和观察范围。 | 外层运行状态、产品输入清单与观察记录头部。 |
| LC-SYN-002 | 观察包按 `OperationsSpec` 的选定版本组织实际模型、材料、证据和追溯关系；模型以选定语言/库解析，运行行为、生命周期接口和原生 view 可定位。声明 `OperateAndObserve` 不计为已采集观察。 | `OperationsBundle` 清单、SysML 限定名、源位置和语言诊断。 |
| LC-SYN-003 | 每项遥测、事件、反馈和维护证据保存实际结果、方法、执行主体、时间和可读取版本引用；`observations.sourceBaselineId` 对应被观察 `product.baselineId`，包内检查基线对应观察包基线。未执行、失败和结果不确定如实保存。 | 原始观察资源、`Evidence`、`BundleChecks` 和基线字段。 |
| LC-SYN-004 | 生命周期材料列出产品和观察选择、候选 choice、理由、已知问题和请求工作；`EvolutionDecision` 的 choice、`requestedNextWork`、rationale、`reviewedBaselineId` 及授权事实均可定位到实际判断记录。 | `lifecycle_decision.md`、候选说明与 `EvolutionDecision`。 |
| LC-SYN-005 | choice 与下一项工作严格按源模型配对：`continueOperations → operate`、`evolve → evolve`、`retire → retire`、`hold → holdForAuthority`；`authorization.nextWorkId` 绑定该请求，不能用展示标签替代实际工作标识。 | `requestedWorkMatchesChoice`、决定字段和 succession 选择。 |
| LC-SYN-006 | `Lifecycle disposition` 的授权记录包含配置 AGENT 的身份、kind、范围、时间、理由、观察 snapshot 和允许工作；`decision.authorized` 源于有效 `currentApproval`，`reviewedBaselineId` 源于所审 snapshot。授权与观察包契约合规分别记录。 | `authorization`、`ReviewDecision`、AGENT 配置和 `decisionCurrent`。 |
| LC-SYN-007 | `nextContext.incrementId` 非空且不同于先前增量，迭代号大于先前产品；`priorReleasedBaselineId` 等于旧发布基线，`inputBaselineId` 等于观察基线，`nextWish.wishId` 保持原 wish 身份。新版本、变更理由和实际输入选择须记录。 | `PrepareNextIncrement::prepared` 字段及 `increment_change.md`。 |
| LC-SYN-008 | 影响分析列出变更内容、相关模型/关系/证据、最早受影响阶段与受影响决定；旧记录保留，受影响后续决定对新选择标明失效或待重评。旧证据复用需显式记录范围与版本，不能无声继承。 | 变更清单、追溯影响、决定历史和新尝试选择。 |
| LC-SYN-009 | `evolve.prepared` 后按源模型返回 `clarify`，继而经 `feasibility` 与新的 `development`；准备不充分或判断无效进入 `holdForAuthority`，有效停止条件走 `stopLifecycle`。暂停恢复记录新上下文和当前决定，不凭超时自动批准或重发未知效果。 | 外层 succession、暂停/恢复点、尝试身份和效果核对记录。 |
| LC-SYN-010 | 用户修改生命周期结果形成关联旧决定的新记录，写明原因、例外范围和请求工作；工具失败、未执行检查及旧判断保留。调度分别确认产品/观察可读取、选择可解释和执行适配器可用，不以授权修改伪造实际前提。 | 用户修改、例外、有效决定与调度诊断。 |
| LC-SYN-011 | 全生命周期使用初始化 T1 及固定依赖，不因机器升级、运行发现问题或新增量而替换 V-model、元模型、编译/渲染/追溯机制。用户约束更改生成新版本并显式选用，观察、检查和决定保留各自采用版本；工具迁移另行设计。 | 工具绑定、约束选择、新增量清单和历史检查范围。 |
| LC-SYN-012 | R1 的继承选择固定其身份和内容，新增量修改模型、view、实现与新决定写开发区，发布只创建新 R2。用户改结果、撤回/替代说明、运行观察和再渲染缓存存于 R1 外；读取核对摘要，损坏报告不能更新为“原 R1 新内容”。 | 继承清单、保存目标、R1/R2 清单及外部说明/缓存。 |

## 出口与返工

通过结构检查后提交[语义评审](semantic_constrains.md)。源模型要求选择与请求工作匹配、观察来源基线匹配、观察包契约合规、审查基线当前，并有有效授权和无有效停止条件，才进入 `operate`、`evolve` 或 `retire`。`hold`、授权缺失或决定不当前进入 `holdForAuthority`；该动作没有自动恢复批准路径。有效停止条件进入 `stopLifecycle`，保留既有事实。

演进准备充分后进入 `clarify`；不充分则保持待处理。运行问题经过影响分析进入受影响阶段的新尝试，或形成明确的新版本增量。开发修改后的采用决定和 R2 发布由各自操作保留，不覆盖 R1；撤回或替代 R1 的说明也在其外部记录。各阶段出口使用所选流程与完成契约；生命周期持续运行不是 wish 永远未完成的依据，运行停止也不等于已完成退役。

## 示例

合格：观察包引用成功封存的 R1，事件日志写开发区；授权决定选择 `evolve` 且请求 `evolve`，从 R1 只读继承，新上下文以观察为输入并递增迭代号。使用固定 T1 和显式新约束重评，完成后产生独立 R2，R1 保持原内容。

不合格：以历史遥测审查新产品，决定 choice 为 `retire` 却请求 `operate`；把 R1 打开为可修改开发包并覆盖其中判断，升级机器工具后自动用于当前项目，还将旧检查布尔值复制到新包并在暂停超时后自动继续。
