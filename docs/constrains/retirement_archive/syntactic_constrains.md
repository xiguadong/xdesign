# 退役与归档的语法及阶段合规约束

规则集版本：0.1。日期：2026-10-12。状态：目标规范；退役计划、授权及产出声明不表示实际停用、处置或归档已经发生。

## 适用范围、依据与输入

适用于[生命周期源模型](../../../wish_to_product.sysml)的 `RetireProduct` 和 `DecommissionAndArchive`。外层 `Lifecycle disposition` 选择 `retire` 后进入本范围；`RetireProduct::obtainClosureAuthority` 的关卡为 `Retirement authorization`，允许下一项工作为 `decommissionAndArchive`。该授权发生在实际退役之前；实际退役产出后的检查用于判定 `recordValid`，本规范不新增源模型未定义的退役后关卡。

依据还包括[整体架构设计 0.2](../../../design/workflow_mbse_architecture.md)的发布保留、不可变性与开发记录边界，以及[V-model 的 ReviewAgent](../../../v_model_agents.sysml)。输入为选定已发布 `OperationsBundle` 及成功封存的 release 身份、当前有效的退役生命周期决定、停用/处置计划、归档与保留策略、`RetirementSpec` 选择和 AGENT 配置。工具及依赖保持初始化 T1，用户约束显式选版本。通用要求见[目录总说明](../README.md)、[模型与追溯语法约束](../model_traceability/syntactic_constrains.md)及[运行语法约束](../workflow_execution/syntactic_constrains.md)。

## 允许工作与产出

允许规划停止运行、解除运行依赖、资源或数据处置、知识转移和归档；有效授权及源模型前提满足后，才执行本次范围的退役操作并记录实际效果。产出为 `RetirementBundle`，包括退役状态与行为模型、处置证据、归档基线索引、知识转移、追溯关系及实际检查，均写开发区或原 release 外部记录。被已发布对象引用的工具、约束、模型、证据及其依赖不能改写或删除；发布退役成果只由[跨阶段发布操作](../project_release/syntactic_constrains.md)创建新对象。退役前计划使用未来行动描述，退役后记录使用实际观察描述，两者不能混用。

支持材料采用 `retirement_plan.md`、`decommission_record.md`、`archive_index.md`、`retention_record.md`、`knowledge_transfer.md`；实际资源清单可按格式拆分并由索引路由。其他命名须在选定契约中映射。归档索引记录稳定身份、版本、摘要及可读取位置，目录名不能单独证明已归档。

## 规则

| ID | 约束判定 | 诊断位置或检查证据 |
| --- | --- | --- |
| RET-SYN-001 | 退役输入固定产品基线、wish、增量、迭代和观察/处置决定选择；进入 `retire` 的外层决定 choice 为 `retire`、请求为 `retire`，且按源模型当前、授权有效并无有效停止条件。 | 产品清单、`EvolutionDecision` 和外层 `beginRetirement` 选择。 |
| RET-SYN-002 | 实际 SysML 模型能在选定语言/库下解析，声明退役状态、停用行为及归档追溯，关联具体被退役部件和生命周期接口，并具有所属原生 view。状态声明不能当作实际系统已停用。 | 模型限定名、行列、关系定义和语言检查输出。 |
| RET-SYN-003 | 退役计划列出范围、受影响依赖、行动顺序、执行条件、停止或恢复条件、责任主体、保留与处置选择、待解决项；计划与知识转移材料具有身份、版本及所涉产品基线。 | `retirement_plan.md`、依赖清单和交接章节。 |
| RET-SYN-004 | `Retirement authorization` 决定包含配置 AGENT 的身份、kind、授权范围、时间、理由、所审产品 snapshot 和 `permittedNextWork = decommissionAndArchive`，并引用同次退役计划与材料选择。人工接管按本 workflow 配置执行。 | `obtainClosureAuthority`、`ReviewDecision`、审查包与 AGENT 配置。 |
| RET-SYN-005 | 实际退役只在 `closureAuthorized`、输入生命周期 `decision.authorized` 及无 `humanVetoActive` 时调度，且实际产品、授权和所选 `RetirementSpec` 一同传入执行动作；计划批准不填写为已经停用。 | `approvedRetirement`、执行输入清单、授权与调度记录。 |
| RET-SYN-006 | `decommission_record.md` 逐项记录实际操作、资源/数据对象、执行主体、时间、方法、结果和原始证据链接；失败、部分执行、未执行及效果不确定单独记录，重试前核对既有外部效果。 | 实际停用/处置输出、效果核对、事件与尝试历史。 |
| RET-SYN-007 | 外部归档索引定位原发布及其相关 wish、模型/材料/证据和决定的选定版本及摘要；保留记录在原发布之外列明对象、策略、条件、访问范围与实际处置状态。运行资源处置不得移除发布所需不可变依赖，知识转移注明材料和接收职责。 | 外部归档/保留索引、依赖保留核对与知识转移记录。 |
| RET-SYN-008 | `RetirementBundle` 具备完整 `ProducedBundle` 字段和实际条目；源模型当前采用 `RetirementSpec` 1.0。`sourceBaselineId` 等于被退役产品基线，wish、增量和迭代与产品一致，包检查基线等于退役包基线。 | `retirementRecord`、`expected`、`BundleChecks` 及 `recordValid` 字段。 |
| RET-SYN-009 | 退役后检查的事实与退役前授权分别存储；只有 `closureAuthorized` 和 `recordValid` 均满足，外层 `lifecycleClosed` 才能结束。局部动作结束、命令成功或 archive 状态标签不能替代外层关闭判定。 | 前置授权、后置证据、动作状态和外层关闭记录。 |
| RET-SYN-010 | 用户修改退役结果记录新决定、旧决定关联、原因、例外边界与允许工作，保留原失败、未执行检查和已发生效果。调度仍确认输入可读取、流程可解释及适配器可用，不伪造 `recordValid` 或实际处置前提。 | 用户决定历史、例外清单、真实效果和调度诊断。 |
| RET-SYN-011 | 退役操作继续使用初始化 T1 的固定 V-model、元模型、编译/渲染/追溯机制及工具依赖。用户约束或保留策略变更产生显式新版本，用于新尝试和判断；旧发布、旧检查采用的规则与输入选择不重标。 | 工具锁定、规则/策略选择、尝试输入与历史范围。 |
| RET-SYN-012 | 退役状态、用户新决定、撤回/替代和真实保留策略保存于原 release 之外，不回写包内状态或清单；只读核对发现损坏应报告，重渲染只写缓存。归档必须保持各已发布对象必要依赖可读取；需要发布退役成果时，封存为独立新发布。 | 保存目标、外部说明、依赖读取核对和新发布选择。 |

## 出口与返工

退役计划提交[语义评审](semantic_constrains.md)，有效 `Retirement authorization` 和输入生命周期决定满足源模型条件后进入 `decommissionAndArchive`。实际产出检查用于 `recordValid`；本地动作返回不等于生命周期关闭。缺失、失败或不确定的效果必须保留并核对，关闭条件不满足时不能宣布完成退役。源模型未提供自动补做或跳过失败的路径，补充执行与恢复需由选定流程/profile 和新的明确授权范围表达。

发现产品设计、移交或保留策略需要调整时，影响分析确定受影响阶段及新尝试，旧退役事实、旧授权和归档选择不覆盖。改变保留策略、撤回或替代的说明保存在原发布之外，发布所需依赖继续保留；退役成果若发布，生成独立新对象。取消或停止需说明哪些动作已停止及哪些效果已发生；运行停止不等于完整退役，归档也不能重写历史 wish 完成结论。

## 示例

合格：固定 T1 下对成功 R1 的退役计划取得授权，实际停用记录和保留策略存于 R1 外；处置运行资源后，R1 的工具、约束、成果与证据依赖仍可读取且摘要一致，退役包真实检查满足后形成外层关闭记录。

不合格：把 R1 包内状态改为 `archived` 就宣称退役，按新保留策略删除其证据依赖；停用结果不确定却盲目重试，并以局部动作结束宣布关闭，没有真实处置证据或可读取的完整发布。
