<!-- document-kit: profile={{profile_id}}@{{version}} class={{document_class}} template={{template_id}} language={{language}} contract={{contract_sha256}} digest-version={{contract_digest_version}} -->
# {{title}}

> 状态: {{status}} | 修订: {{revision}} | 模型来源: {{model_sources}}

<a id="{{section_id:stages}}"></a>

## 1. 阶段与门控

**阶段:** {{field:stages}}

**转换:** {{field:transitions}}

**退出门控:** {{field:exit_gates}}

{{content:stages}}

<a id="{{section_id:behavior}}"></a>

## 2. 行为与约束

**行为:** {{field:behavior}}

**不变量:** {{field:invariants}}

{{content:behavior}}

<a id="{{section_id:trace}}"></a>

## 3. 追踪与验证

**模型元素:** {{field:model_elements}}

**需求链接:** {{field:requirements_links}}

**验证链接:** {{field:verification_links}}

{{content:trace}}
