# dsc version

```
dsc version [<discourse>]
```

With no argument, prints `dsc`'s own version.

With a forum name, prints that forum's **live Discourse version and git commit**, read from its `/about.json` using the configured API key - so it works even on login-required forums where an anonymous request is rejected.

```bash
dsc version                # → 0.10.20  (dsc itself)
dsc version accm           # → accm: Discourse 2026.6.0-latest (70aacf7…)
```

Useful for checking which build each forum in your fleet is on - e.g. whether a forum is new enough to include a recently-bundled core plugin. For a quick fleet-wide sweep, loop over `dsc list`:

```bash
dsc list -f json | jq -r '.[].name' | while read f; do dsc version "$f"; done
```
