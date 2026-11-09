<!-- document-kit: profile={{profile_id}}@{{version}} class={{document_class}} template={{template_id}} language={{language}} contract={{contract_sha256}} digest-version={{contract_digest_version}} -->
# {{title}}

> 状态: {{status}} | 修订: {{revision}} | 模型来源: {{model_sources}}

<a id="{{section_id:boundary}}"></a>

## 1. 背景与边界

**系统边界:** {{field:system_boundary}}

**外部参与者:** {{field:external_actors}}

**图示决策:** {{field:diagram_decisions}}

{{content:boundary}}

<a id="{{section_id:interfaces}}"></a>

## 2. 外部接口

**信息流:** {{field:information_flows}}

**接口参与者:** {{field:interface_participants}}

{{content:interfaces}}

<a id="{{section_id:behavior}}"></a>

## 3. 交互与生命周期

**交互:** {{field:interactions}}

**状态:** {{field:states}}

{{content:behavior}}

<a id="{{section_id:constraints}}"></a>

## 4. 约束与决策

**约束:** {{field:constraints}}

**设计依据:** {{field:design_rationale}}

{{content:constraints}}

<a id="{{section_id:trace}}"></a>

## 5. 追踪与验证

**模型元素:** {{field:model_elements}}

**验证链接:** {{field:verification_links}}

{{content:trace}}
