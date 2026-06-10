# Man pages

Generate Unix man pages for `dsc` and every subcommand.

```
dsc man --dir <path>
```

Writes one ROFF-formatted file per (sub)command:

- `dsc.1` - root command
- `dsc-tag.1`, `dsc-setting.1`, ... - top-level subcommands
- `dsc-tag-pull.1`, `dsc-setting-diff.1`, ... - nested subcommands

Naming follows the `git`/`cargo` convention: nested commands are joined with hyphens. All pages live in section 1 (user commands).

## For users

```bash
mkdir -p ~/.local/share/man/man1
dsc man --dir ~/.local/share/man/man1
mandb 2>/dev/null || true   # rebuild the index on Linux
man dsc                     # try it
man dsc-tag-pull            # nested commands work too
```

## For distro packagers

`dsc man` is the recommended way to materialise man pages at package-build time. The pages are not committed to the repository and not bundled in release tarballs - generate them yourself with the version of `dsc` you are packaging:

```bash
dsc man --dir "${pkgdir}/usr/share/man/man1"
gzip -9 "${pkgdir}/usr/share/man/man1/"*.1   # if your packaging convention expects compressed pages
```

The output is deterministic for a given `dsc` version, so the page set is suitable for inclusion in distribution package manifests.

Regenerate after upgrading `dsc` - the page set tracks the CLI surface, which grows between releases.
