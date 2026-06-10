---
name: Spec request
about: A substantial new command surface that warrants a written spec
title: 'spec: '
labels: spec
---

<!-- The fastest path is to draft the spec yourself under spec/ and link it
     here. AGENTS.md has a copy-pasteable template plus "what makes a spec
     land fast." -->

## Real-world driver

<!-- Name the actual install / use case. "I'm doing X for forum.example.com;
     I need Y; today I work around it by Z." Field-driven specs land faster
     than speculative ones. -->

## Why this is a separate spec, not a feature request

<!-- Roughly: would it add a new top-level command, change config schema,
     introduce a new file format, or wrap a Discourse API surface dsc
     doesn't touch today? Any of those = spec. -->

## Reference: API calls observed in the field

<!-- If you've been working around this with curl / the Admin UI's network
     tab, paste the requests and redacted responses here. Note the Discourse
     version you tested against. This is the highest-value content in any
     spec request — it removes the discovery phase entirely. -->

```
GET /admin/...
Api-Key: <redacted>
→ 200
{ ... }
```

## Draft spec

<!-- Either drop a markdown body in here or link a draft PR adding it under
     spec/. The spec template in AGENTS.md covers what to include. -->
