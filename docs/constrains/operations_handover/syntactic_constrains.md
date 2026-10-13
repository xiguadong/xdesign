# 交付与运行移交的语法及阶段合规约束

规则集版本：0.1。日期：2026-10-12。状态：目标规范；产出契约、交付操作和检查要求不表示已有自动执行能力或实际移交结果。

## 适用范围、依据与输入

适用于 `VModelAgents::VModelDevelopment::operationsHandover` 和 `WishToProduct::ContractedVModelDevelopment::operationsHandover`。源模型关卡为 `Operations handover`，`nextWorkId` 为 `lifecycleManagement`。V-model 内部在 `operationsReady` 时结束开发；[外层生命周期](../../../wish_to_product.sysml)通过 `productReleased` 将同一产品交给 `operate`，不能把 `lifecycleManagement` 解释成源模型未定义的额外关卡。

依据[整体架构设计 0.2](../../../design/workflow_mbse_architecture.md)、[V-model 的 OperationsAgentBehavior](../../../v_model_agents.sysml)和生命周期模型的 `OperationsSpec`、`OperationsBundle`、`OperateAndObserve`。输入为选定 wish、完成契约、当前验收基线及决定、发布配置、受控构建、验证和确认材料、运行环境及接收范围。通用要求见[目录总说明](../README.md)、[模型与追溯语法约束](../model_traceability/syntactic_constrains.md)和[运行语法约束](../workflow_execution/syntactic_constrains.md)。

## 允许工作与产出

允许在 `development` 组装拟交付包、执行本次授权范围内的安装与投运准备、生产操作和维护材料、明确支持责任与生命周期接口，并记录实际交接和准备验证。产出为 `OperationsBundle`：拟交付产品配置模型、运行行为与生命周期接口、发布材料、运行/维护指南、支持计划、实际证据和追溯关系。声明该类型或通过移交关卡不能证明已经封存发布；需区分开发产物、确定内容的候选和成功封存的不可变 release。跨阶段[项目发布操作](../project_release/syntactic_constrains.md)创建新 release，不移动现有 `Operations handover` 关卡。新检查、关卡决定及用户修改写开发区；修改实现或验收范围应进入影响分析。

材料采用 `release_manifest.md`、`handover_record.md`、`operations_guide.md`、`maintenance_guide.md`、`support_plan.md`；有恢复、演进及退役专题时可拆分为 `recovery_plan.md`、`lifecycle_plan.md`，由清单路由。其他项目命名应在选定阶段契约中一一映射。命名和目录只说明归属，交付版本由固定清单与摘要确定。

## 规则

| ID | 约束判定 | 诊断位置或检查证据 |
| --- | --- | --- |
| OPS-SYN-001 | 固定 wish、增量、迭代、尝试、验收决定、产品基线和完成契约选择；`sourceBaselineId` 对应本次验收基线。工具及依赖与初始化 T1 一致，V-model、元模型、编译/渲染/追溯机制不日常变更；用户约束显式选版本。实际交付、证据与审查选择指向同一配置。 | 工具锁定、约束选择、输入/发布清单和审查快照。 |
| OPS-SYN-002 | 产品配置、运行行为与生命周期接口的实际 SysML 模型能在选定库及语言版本下解析；部件、接口、行为、外部资源引用与所属原生 view 能定位，正式关系和自定义关系角色分别声明。 | 模型文件、限定名、行列与语言检查诊断。 |
| OPS-SYN-003 | `OperationsBundle` 具备完整包身份、版本、当前及来源基线、wish、增量、迭代与契约选择，包含实际模型、支持材料、证据及追溯条目。源模型当前采用 `OperationsSpec` 1.0，检查基线与提交包基线一致。 | `ProducedBundle`、`BundleChecks` 和所选 `expected`。 |
| OPS-SYN-004 | 最终发布清单固定工具及依赖、约束 SysML 与外部规范确定内容、原生模型/view、实现/文档、原生关系、实际检查、AGENT 判断、用户修改及最终发布决定，保留所采纳证据各自的输入/规则依赖；仅收集相关完整依赖。交付说明列出环境和限制，不以可变网页、分支或开发文件替代冻结内容。 | `release_manifest.md`、最终发布清单、证据依赖及不可变资源引用。 |
| OPS-SYN-005 | 操作、维护、恢复与支持材料包含前提、步骤、责任主体、异常处置和升级/回退入口；生命周期材料列出观察、变更、演进和退役的触发条件及接收入口。没有适用项时须记录不适用理由。 | 指南、支持计划、恢复计划和生命周期章节。 |
| OPS-SYN-006 | 移交记录列明交付方、接收方或接收角色、范围、环境、时间、材料选择、未决项和条件；检查与演练记录保存真实方法、主体、时间、实际输出及证据链接，未执行或失败保持原状态。 | `handover_record.md`、投运检查、恢复演练和原始记录。 |
| OPS-SYN-007 | 本阶段仅执行已授权的发布组装、准备验证与交接行为；安装、配置或外部运行效果逐项记录。实现、需求或验收基线变化须提交影响分析和新尝试，不能把变更后的产品继续套用旧验收证据。 | 行为授权范围、外部效果记录、差异与返工选择。 |
| OPS-SYN-008 | `Operations handover` 决定包含 AGENT 身份与 kind、授权、所审 snapshot、时间、理由、结论和 `permittedNextWork = lifecycleManagement`；AGENT 与人工接管配置来自本 workflow。与外层 `operate` 的交接记录指明映射及实际产品选择。 | 关卡记录、workflow 配置与外层生命周期输入。 |
| OPS-SYN-009 | 保留本次结构检查、语义评审、有效决定和逐 wish 完成记录；完成记录按契约写明交付/投运/后续运行的纳入范围。`operationsReady`、开发运行结束、真实接收与 wish 完成作为不同记录保存。 | 检查清单、移交决定、运行状态与完成记录。 |
| OPS-SYN-010 | 用户改关卡结果记录新决定、替代对象、原因、例外范围和允许的后续工作；原失败、未执行检查及旧决定不删除。调度前另查交付物可读取、执行器支持所选流程和适配器可用，不能以修改结果代替实际前提。 | 用户修改历史、例外清单、有效决定与调度诊断。 |
| OPS-SYN-011 | 候选有独立内容标识、固定清单与摘要，其标识不包含尚未产生的最终发布决定；决定引用候选标识，最终 release 清单关联候选与决定。核对选择或封存失败分别记录，不产生成功 release；实际交付发布成果必须引用成功封存的对象。 | 候选标识、最终决定、封存结果及交付引用。 |
| OPS-SYN-012 | release 成功后只读，开发编辑或用户改 gate 不覆盖包内文件、关系与决定；读取核对内容，被外部修改则报告损坏。原生 view 与交付渲染固定，重新渲染只写缓存；撤回或替代说明存于原发布对象之外。 | 发布读写边界、摘要核对、渲染缓存和外部说明。 |

## 出口与返工

结构合规后提交[语义评审](semantic_constrains.md)。源模型要求当前有效关卡决定、产出契约与来源基线有效，并满足 `acceptanceReady`，才得到 `operationsReady`。V-model 内部由此结束开发；选用 `WishToProductLifecycle` 时，外层按 `productReleased` 进入 `operate`。这些流程条件不单独证明 release 封存成功；实际发布交付还须具有候选核对及成功封存的真实记录。发布操作按独立规则处理，不增设或移动现有阶段关卡。待判断则等待，拒绝或有效停止条件则停止，失效及返工进入 `impactAndRework` 并回到最早受影响阶段。

运行中发现产品或移交材料问题时，保存观察并确定影响范围，回到受影响阶段的新尝试或建立有版本的新增量；不能改写已交付基线、旧证据与旧决定。wish 是否完成仍由完成契约及逐 wish 决定确定。

## 示例

合格：候选与受验构建一致，最终发布决定引用候选标识，核对后封存为新 R1；清单关联固定 T1、约束、成果、关系和相关证据依赖。移交记录引用成功 R1，重新渲染写缓存，外层运行收到同一产品。

不合格：验收针对构建 A，交付时替换为构建 B，却沿用 A 的验证证据；封存失败后仍宣称 R1 成功，用户改关卡又覆盖 R1 包内决定。修改结果不能补出封存成功，也不能改变旧发布。
