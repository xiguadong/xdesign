# 阶段与目标校验约束路由

规则集版本：0.1。状态：目标约束草案。日期：2026-10-12。

本目录把[当前整体架构 0.2](../../design/workflow_mbse_architecture.md)中的阶段契约与生命周期边界展开为可定位的语法校验规则与语义校验规则。阶段、关卡与行为名称依据 [V-model 定义](../../v_model_agents.sysml)和 [Wish 生命周期及产物契约](../../wish_to_product.sysml)。这些文档规定待落实的校验行为；文档存在、模型声明和示例中的布尔值均不能证明校验已执行。

规则集 0.1 与现有 `DeliverableSpec` 子类型的契约版本 1.0 分别选择和保存。源模型中的 `BundleChecks`、`Evidence` 等字段提供当前声明基础；逐规则状态、恢复和例外等目标要求可以通过类型化附属记录与绑定展开，不宣称源模型已经具备这些字段或运行能力。既有模板的人类权限策略由具体 workflow 选择，新的通用约束以整体架构的人或 AI AGENT 配置和用户修改决定协议为准。

项目初始化绑定一个工具发布，例如 T1；其 V-model、公共元模型、编译解析、渲染、追溯机制及固定依赖在项目全生命周期保持不变。每次工作核对该绑定，再显式选择用户约束版本及工作输入。用户可以修改约束 SysML 和文档规范，但规则变更不能移动固定关卡或替换工具机制。本文提供约束的人可读规范，约束 SysML 的实现需保留对应规则身份和版本。

阶段产物、可变模型/view、约束和新的工作记录写入开发区。发布操作从确定候选创建一个新的不可变发布对象；修改开发内容、约束或用户决定不回写旧发布。从 R1 继续工作时只读继承，后续成果封存为 R2；工具升级属于单独迁移设计。

## 目录路由

每个子目录对应一个校验目标，均包含 `syntactic_constrains.md` 和 `semantic_constrains.md`。架构设计与详细设计另有 `index.md`，规定产物命名、组织和设计表达。目录名称及 `constrains` 拼写按本次约定保留。

| 范围 | 目标目录 | 语法规则 | 语义规则 | 组织规范 |
| --- | --- | --- | --- | --- |
| 全阶段 | 原生模型、Wish 追溯与 view | [model_traceability](model_traceability/syntactic_constrains.md) | [语义](model_traceability/semantic_constrains.md) | 本 README |
| 全阶段 | 工作流、关卡与运行记录 | [workflow_execution](workflow_execution/syntactic_constrains.md) | [语义](workflow_execution/semantic_constrains.md) | 本 README |
| 发布操作 | 候选快照、发布决定与不可变封存 | [project_release](project_release/syntactic_constrains.md) | [语义](project_release/semantic_constrains.md) | 本 README |
| 开发前 | Wish 登记、增量初始化与澄清 | [wish_intake](wish_intake/syntactic_constrains.md) | [语义](wish_intake/semantic_constrains.md) | 本 README |
| 开发前 | 可行性评估 | [feasibility_assessment](feasibility_assessment/syntactic_constrains.md) | [语义](feasibility_assessment/semantic_constrains.md) | 本 README |
| V-model 01 | 利益相关方需求 | [stakeholder_needs](stakeholder_needs/syntactic_constrains.md) | [语义](stakeholder_needs/semantic_constrains.md) | 本 README |
| V-model 02 | 系统需求 | [system_requirements](system_requirements/syntactic_constrains.md) | [语义](system_requirements/semantic_constrains.md) | 本 README |
| V-model 03 | 系统架构与方案 | [system_architecture](system_architecture/syntactic_constrains.md) | [语义](system_architecture/semantic_constrains.md) | [index](system_architecture/index.md) |
| V-model 04 | 详细设计 | [detailed_design](detailed_design/syntactic_constrains.md) | [语义](detailed_design/semantic_constrains.md) | [index](detailed_design/index.md) |
| V-model 05 | 实现与构建 | [implementation_build](implementation_build/syntactic_constrains.md) | [语义](implementation_build/semantic_constrains.md) | 本 README |
| V-model 06 | 组件验证 | [component_verification](component_verification/syntactic_constrains.md) | [语义](component_verification/semantic_constrains.md) | 本 README |
| V-model 07 | 集成与接口验证 | [integration_verification](integration_verification/syntactic_constrains.md) | [语义](integration_verification/semantic_constrains.md) | 本 README |
| V-model 08 | 系统验证 | [system_verification](system_verification/syntactic_constrains.md) | [语义](system_verification/semantic_constrains.md) | 本 README |
| V-model 09 | 使用确认与验收 | [acceptance_validation](acceptance_validation/syntactic_constrains.md) | [语义](acceptance_validation/semantic_constrains.md) | 本 README |
| V-model 10 | 交付与运行移交 | [operations_handover](operations_handover/syntactic_constrains.md) | [语义](operations_handover/semantic_constrains.md) | 本 README |
| 交付后 | 运行观察、生命周期判断与演进 | [lifecycle_management](lifecycle_management/syntactic_constrains.md) | [语义](lifecycle_management/semantic_constrains.md) | 本 README |
| 交付后 | 退役与归档 | [retirement_archive](retirement_archive/syntactic_constrains.md) | [语义](retirement_archive/semantic_constrains.md) | 本 README |

登记、初始化和澄清按 `WishToProductLifecycle` 的现有次序组织；可行性、运行演进和退役属于外层生命周期。发布是贯穿成果选择与交付的操作目标，不新增 V-model 阶段或移动已有关卡。项目初始化确定具体流程及工具发布；运行只能使用项目绑定的版本，其他模板在项目初始化或独立迁移设计中选择。

## 阶段、关卡与 V-model 对应

| 阶段 | 原生 action usage | 关卡 | 主要产物 / 对应的左侧基线 | 放行的下一项工作 |
| --- | --- | --- | --- | --- |
| 利益相关方需求 | `stakeholderNeeds` | `Needs review` | 使用场景、预期用途、验收意图 | `systemRequirements` |
| 系统需求 | `systemRequirements` | `SRR` | 需求基线、系统验证计划 | `systemArchitecture` |
| 系统架构与方案 | `systemArchitecture` | `PDR` | 方案、组成、接口、集成计划 | `detailedDesign` |
| 详细设计 | `detailedDesign` | `CDR` | 组件结构、行为、约束、组件验证用例 | `implementation` |
| 实现与构建 | `implementation` | `Build review` | 受控源码、配置、构建清单、构建产物 | `componentVerification` |
| 组件验证 | `componentVerification` | `Component review` | 实际组件证据 / 详细设计 | `integrationVerification` |
| 集成与接口验证 | `integrationVerification` | `Integration review` | 实际集成证据 / 架构与接口 | `systemVerification` |
| 系统验证 | `systemVerification` | `System verification review` | 实际系统证据 / 系统需求 | `stakeholderValidation` |
| 使用确认与验收 | `stakeholderValidation` | `Acceptance review` | 实际用途证据 / 利益相关方需求与 Wish | `operationsHandover` |
| 交付与运行 | `operationsHandover` | `Operations handover` | 发布配置、操作维护材料、支持计划 | `lifecycleManagement`（外层接入运行观察） |

外层生命周期中的 `Feasibility review` 允许 `development`；`Lifecycle disposition` 的选择与下一工作绑定为 `continueOperations → operate`、`evolve → evolve`、`retire → retire`、`hold → holdForAuthority`；`Retirement authorization` 允许 `decommissionAndArchive`。登记和澄清不额外虚构一个关卡。

## 两类校验的边界

**语法校验**包含 SysML 解析、名称与类型解析、原生关系合法性，以及文件位置、文档结构、必要产物、追溯关系、记录字段和阶段操作范围等明确可检查的合规规则。Pilot 可能把类型、引用等检查称为 semantic validation；在本目录中它们仍归入结构合规层。

**语义校验**评价 Wish 理解、需求质量、方案适合性、设计可实现性、代码符合性、证据充分性和剩余问题是否可接受。由选定 workflow 所配置的人或 AI AGENT 作出有理由的判断，工具结果为其提供事实。工作执行者与关卡判断者分别配置，可为同一主体。

每次审查同时选择两份目标规则及两个全阶段目标的规则，涉及候选和封存时再应用发布目标。阶段规则限定本阶段所需内容；后续设计、实现和执行证据尚未产生时，记录为尚未产生，不要求在上游阶段提前声明全生命周期完成。语言 profile 与机制来自固定工具，检查绑定、用户规则及产物位置在其支持范围内由选定契约确定。

## 通用产物与文档规范

1. 产物至少能够定位到稳定身份、版本或内容摘要、Wish 范围、增量、阶段及本次尝试、上游基线、所用契约与规则版本、来源或生产者。允许通过审查清单关联这些信息，不强制每个文件复制一份运行记录。
2. 新增工程文档默认使用小写英文与下划线组成的有意义文件名，以 `.md` 结尾；普通资料采用 `<scope>_<kind>.md`，例如 `coordinator_system_verification_plan.md`。模型使用 `.sysml`，Wish 模型可保持 `W001.sysml` 等稳定身份命名；代码、构建工具和第三方文件沿用各自规范。架构与详细设计采用各自 `index.md` 的细化规范。
3. 新文档在标题之后注明文档身份或所属模型、版本、状态、范围和上游依据；正文覆盖目标与边界、内容或方法、关联关系、未解决问题及证据。状态区分草案、候选和已采纳；“已批准”须能引用对应范围的实际决定。
4. 目录是产物位置的契约，不能代替元素身份。架构参考位置为 `.xflow/tool.lock`、`constraints/`、`development/models/`、`development/docs/`、`development/code/`、`development/records/`、`development/cache/` 及 `releases/`。Git/ProjectStore 可承担对应职责，现有 `design/` 等位置可通过契约映射为开发资产；本次新增规则不自动搬迁文件。工具绑定由初始化维护，日常保存只写用户约束或开发资产，发布写入新的发布对象，旧发布只读。
5. 外部文档引用包含文档身份、选定版本、章节号或稳定章节标识、关联模型元素及引用理由。无法恢复旧内容或定位章节时，报告缺失或待重审。所有模型文件维护自己的原生 view usage，默认与模型同文件。
6. 空文件、占位文字、`TODO` 和只有目录标题的材料不能满足必需内容。允许记录尚待解决的问题，但要声明其影响、责任、处理方式及阻塞范围；文本中出现 `TODO` 本身不构成失败，必需内容缺失才构成失败。
7. 发布候选有确定内容标识，不包含尚未产生的最终发布决定；最终决定引用该候选，封存清单关联候选与决定。旧发布、原生 view 及包内渲染文件不可原位保存覆盖；撤回/替代说明追加在发布对象之外，重新渲染只写缓存。发现内容摘要不符报告损坏。
8. 各阶段证据保留实际检查时的输入、规则和环境，后续评审说明适用性，不能重标为针对最终发布执行。发布保存所需依据的完整依赖集合，无需复制全部开发历史；共享不可变内容必须在已有发布需要时持续可读。

各子文档中的名称和记录例子用于解释规则，不是已执行的事实，也不冻结尚待详细设计的存储字段或 API。

## 规则身份与结果协议

规则 ID 使用 `<目标前缀>-SYN-<三位序号>` 或 `<目标前缀>-SEM-<三位序号>`。前缀为 `MT`、`WF`、`REL`、`WI`、`FEAS`、`NEED`、`REQ`、`ARCH`、`DD`、`BUILD`、`CV`、`IV`、`SV`、`AC`、`OPS`、`LC`、`RET`。ID 在含义不变时保持稳定；正式选择后的规则内容变化形成新规则集版本并保存旧版本。本次 0.1 草案统一依据架构 0.2 整理。

规则表默认规定必须满足的要求；明确写为建议的组织方式不作阻断规则。每条规则的实际检查或判断需要保留规则 ID 和版本、适用对象及内容选择、执行主体与工具版本、时间、结果、诊断位置、依据和尝试身份。

| 结果 | 含义 | 对推进的作用 |
| --- | --- | --- |
| `pass` | 此规则在本次选定材料上实际检查满足，或 AGENT 判断满足 | 仅支持此规则，不自动批准关卡 |
| `fail` | 发现明确违规或工程内容不足 | 记录缺口与受影响范围，提交返工或明确例外判断 |
| `not_applicable` | 规则不适用于当前范围，附理由与契约依据 | 不计作已执行通过，必需规则不能无理由省略 |
| `not_run` | 尚未执行检查或评审 | 不得当作成功 |
| `indeterminate` | 工具失败、证据缺失或判断依据不足，不能确定结果 | 保留原因并补材料或核查 |

语义评审逐项注明支持、缺口和不确定性，再形成关卡建议。最终关卡结果使用选定模型的 `pending`、`approved`、`rework`、`rejected`；`stale` 表示旧决定不再适用于当前材料选择。决定附带主体、授权范围、审查包、理由、证据及允许的下一工作。

## 出口、返工与用户修改

正常推进要求契约必需产物齐备、实际检查有结果、语义判断充分，并且输入可读取、流程可解释、执行适配器可用。设有关卡的步骤还须当前范围有有效关卡决定；没有关卡的登记、初始化、澄清等动作保存准备事实和条件判断，按固定流程推进。语法通过不能自动转为语义充分、需求满足、关卡批准或 Wish 完成。

用户可以修改开发区的关卡结果。系统新增替代决定，记录修改对象、原因、材料选择、例外规则与范围、剩余问题以及允许的后续工作，保留原决定与原检查事实。例外批准可以改变当前有效关卡结论，但不能把原检查或观察事实的失败改写为通过，不能补出未生成材料或未执行证据，不能使无法解释的流程变为可执行；已发布对象内的决定保持不变。

返工按流程中已声明的路径回到最早受影响阶段，建立新尝试；合法返工回路不能因为通用任务图要求无环而被拒绝。相关依赖变更才重新评估对应决定；无关范围保持原适用性。新规则只有在显式选择后约束新的运行或审查，历史运行继续保留其原选择。

工具项目自举选择既有工具发布或有限且明确的引导机制，并保持本次执行机制固定。候选 V-model、元模型或工具代码作为开发资产接受验证，发布后形成新工具 T2，不在开发中替换当前 T1。用户约束独立换版；不能以尚未采纳的规则评价自身或回写旧项目发布。

## 使用顺序

1. 核对初始化绑定的固定工具/流程与依赖，选择 Wish、增量、阶段契约与本目录规则集版本，并固定上游依赖。
2. 查阅两个全阶段目标，再进入当前阶段目标；架构和详细设计先读 `index.md`。
3. 在允许范围内生产实际产物、关系和原生 view，提交有身份及版本的审查清单。
4. 执行适用语法规则，保存真实诊断；AGENT 对同一选择执行语义评审。
5. 将事实与新决定写入开发区；按有效关卡决定或无关卡步骤的条件调度，或按影响范围返工。发布按目标规则检查候选并创建新不可变对象。验证工具能力与验证一个业务 Wish 的成果分别选择范围和证据。
