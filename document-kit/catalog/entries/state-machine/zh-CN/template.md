<!-- document-kit: profile={{profile_id}}@{{version}} class={{document_class}} template={{template_id}} language={{language}} contract={{contract_sha256}} digest-version={{contract_digest_version}} -->
# {{title}}

> 状态: {{status}} | 修订: {{revision}} | 模型来源: {{model_sources}}

<a id="{{section_id:purpose}}"></a>

## 1. 目的与分配

**设计对象:** {{field:design_subject}}

**需求链接:** {{field:requirements_links}}

{{content:purpose}}

<a id="{{section_id:states}}"></a>

## 2. 状态与不变量

**状态:** {{field:states}}

**不变量:** {{field:invariants}}

**图示决策:** {{field:diagram_decisions}}

{{content:states}}

<a id="{{section_id:events}}"></a>

## 3. 事件与监护条件

**事件:** {{field:events}}

**监护条件:** {{field:guards}}

{{content:events}}

<a id="{{section_id:transitions}}"></a>

## 4. 转换与生命周期

**转换:** {{field:transitions}}

**终止状态:** {{field:terminal_states}}

{{content:transitions}}

<a id="{{section_id:recovery}}"></a>

## 5. 恢复与约束

**恢复行为:** {{field:recovery_behavior}}

**约束:** {{field:constraints}}

{{content:recovery}}

<a id="{{section_id:projection}}"></a>

## 6. 映射与验证

**源码映射:** {{field:source_projection}}

**验证链接:** {{field:verification_links}}

{{content:projection}}
