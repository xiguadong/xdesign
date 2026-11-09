<!-- document-kit: profile=xdesign-sysml-docs@1.3.0 class=requirements template=change-requirements language=zh-CN contract=0000000000000000000000000000000000000000000000000000000000000000 digest-version=3 -->
# 虚构 Aster 采样周期变更需求

> 状态: 虚构示例 | 修订: 1 | 模型来源: `mbse/model/example.sysml`

<a id="R10700"></a>

## 1. 变更摘要

**当前行为:** 正常运行以 60 秒为周期采样，并向所有控制器提供相同结构的 Reading。

**拟议行为:** 高温告警生效期间以 30 秒为周期采样，告警结束恢复 60 秒，Reading 字段和查询语义不变。

<!-- document-kit:content:change:start -->
既有 Aster 集线器每 60 秒采样一次。拟议变更仅在高温告警生效时改为每 30 秒采样，告警结束后恢复原周期；正常默认行为和既有读取契约均保持不变。
<!-- document-kit:content:change:end -->

<a id="R10701"></a>

## 2. 受影响的需求

**受影响项:** R-SAMPLE-60、调度负载、高温告警状态和周期验证用例需要修订，接口与数据模式需求不需要修订。

**依据:** 更密集的高温样本能够缩短响应延迟，将差异限制在周期和告警状态则可降低既有消费者受到非预期影响的风险。

<!-- document-kit:content:impact:start -->
变更影响周期需求 R-SAMPLE-60、调度容量和告警状态及其验证资产，不影响数据验证、缓冲、读数结构或最新值错误语义。影响范围的限定避免无关契约被一并改写。
<!-- document-kit:content:impact:end -->

<a id="R10702"></a>

## 3. 兼容性与迁移

**兼容性约束:** Reading 字段、查询操作、UTC 格式和最新有效值错误不得改变，高温告警速率必须可选且默认禁用。

**迁移说明:** 部署时保持告警模式关闭，经评审后按集线器启用；未配置集线器继续使用 60 秒周期，并保留关闭告警即可回退的路径。

<!-- document-kit:content:compatibility:start -->
既有控制器只会观察到更频繁的读数，无需修改解析逻辑。告警速率默认关闭，只有经过评审的配置才能对指定集线器启用，未配置设备继续保持原行为。
<!-- document-kit:content:compatibility:end -->

<a id="R10703"></a>

## 4. 验收条件

**可测量条件:** 告警期间采样间隔为 30±0.1 秒，退出告警后恢复 60±0.1 秒，所有响应保留原有必填字段和错误含义。

**验证方法:** 使用虚拟时间演示告警进入和退出，并运行未经修改的版本 1 控制器契约检查。

<!-- document-kit:content:acceptance:start -->
验收在同一受控会话中比较变更前后的采样周期，并使用未经修改的旧版控制器解析全部响应。新行为与保留行为必须分别满足明确的数值和契约条件。
<!-- document-kit:content:acceptance:end -->

<a id="R10704"></a>

## 5. 追踪与限制

**模型元素:** Example::Sensor 是已提供的参与者；周期、热警报、Reading 和查询概念是拟议语言侧设计，并非 example.sysml 中的声明。

**未决事项:** 电池消耗、告警授权以及可持续告警时长仍需进一步评审。

<!-- document-kit:content:trace:start -->
所提供模型仅将 Example::Sensor 声明为参与者。周期、热警报状态、Reading 和查询变更属于拟议的语言侧设计，当前没有 SysML 实现；该最小模型不能证明这些行为。
<!-- document-kit:content:trace:end -->
