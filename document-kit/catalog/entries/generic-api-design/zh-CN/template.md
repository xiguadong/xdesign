<!-- document-kit: profile={{profile_id}}@{{version}} class={{document_class}} template={{template_id}} language={{language}} contract={{contract_sha256}} digest-version={{contract_digest_version}} -->
# {{title}}

> 状态: {{status}} | 修订: {{revision}} | 模型来源: {{model_sources}}

<a id="{{section_id:purpose}}"></a>

## 目的与边界

**API 标识:** {{field:api_identity}}

**API 类型:** {{field:api_kind}}

**层次:** {{field:layer}}

**目的:** {{field:purpose}}

{{content:purpose}}

<a id="{{section_id:contract}}"></a>

## 声明与契约

**声明:** {{field:declaration}}

**输入:** {{field:inputs}}

**输出:** {{field:outputs}}

**错误:** {{field:errors}}

**副作用:** {{field:side_effects}}

{{content:contract}}

<a id="{{section_id:behavior}}"></a>

## 行为与约束

**行为:** {{field:behavior}}

**不变量:** {{field:invariants}}

**约束:** {{field:constraints}}

{{content:behavior}}

<a id="{{section_id:trace}}"></a>

## 追踪与验证

**模型实现或理由:** {{field:model_realization_or_reason}}

**源码映射:** {{field:source_projection}}

**需求链接:** {{field:requirements_links}}

**设计链接:** {{field:design_links}}

**验证链接:** {{field:verification_links}}

{{content:trace}}

<a id="{{section_id:limits}}"></a>

## 状态与限制

**未决事项:** {{field:unresolved_items}}

{{content:limits}}
