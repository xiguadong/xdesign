# Template and language routing

Schema 1.3 keeps the ten document classes and their existing class/name/layer routes. Select an entry with `template_id`, or accept the class generic default. Select `language=en` or `language=zh-CN` explicitly for plan/create; omission means `en` and never examines locale or prose.

Each entry owns its English and Chinese template, guide, fictional example, and complete fixture under `catalog/entries/<template-id>/<language>/`. Read the guide for the selected variant before authoring. Examples and fixtures are readable evidence of the pack's mechanical contract, not selectable templates or engineering acceptance.

Update, attachment, and selected-document check use the persisted marker and registry language. A supplied language on those operations is an assertion, not a conversion or reselection. Whole-project and class-filter checks do not accept a language filter. A same-route second create in another language remains a collision.

The 20-document mixed fixture covers every class in both languages and two requirement entries. It uses figures for English system context and interaction flow, a reasoned table for Chinese system context, and reasoned ordered prose for Chinese interaction flow. All 42 variants are independently instantiated by the acceptance suite; the mixed fixture is not a substitute for that coverage.

Original 1.0.0, 1.1.0, and 1.2.0 packs remain selectable under `compatibility/`. They have no language selector and retain their original identities and bytes. The 1.3 selected pack can check and update supported mixed old/new projects without migrating old identities.
