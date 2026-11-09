<!-- document-kit: profile={{profile_id}}@{{version}} class={{document_class}} template={{template_id}} language={{language}} contract={{contract_sha256}} digest-version={{contract_digest_version}} -->
# {{title}}

> 状态: {{status}} | 修订: {{revision}} | 模型来源: {{model_sources}}

<a id="{{section_id:purpose}}"></a>

## 1. 目的与参与者

**边界:** {{field:boundary}}

**参与者:** {{field:participants}}

{{content:purpose}}

<a id="{{section_id:operation}}"></a>

## 2. 操作契约

**操作:** {{field:operations}}

**数据结构:** {{field:data_shapes}}

{{content:operation}}

<a id="{{section_id:errors}}"></a>

## 3. 错误与不变量

**错误行为:** {{field:error_behavior}}

**不变量:** {{field:invariants}}

{{content:errors}}

<a id="{{section_id:compatibility}}"></a>

## 4. 版本管理与兼容性

**版本策略:** {{field:version_policy}}

**兼容性策略:** {{field:compatibility_policy}}

{{content:compatibility}}

<a id="{{section_id:trace}}"></a>

## 5. 模型与验证追踪

**模型元素:** {{field:model_elements}}

**验证链接:** {{field:verification_links}}

{{content:trace}}
