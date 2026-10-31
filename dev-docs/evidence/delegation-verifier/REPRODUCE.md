# Reproducing delegated retrieval checks

Use the current [documentation workflow](../../../.github/workflows/documentation.yml)
as the command and feature-matrix authority. Preserve the exact source and selected
inputs, tool versions, executable identities, arguments, logs and failures before
changing measured bytes. Use Rust/Cargo 1.96, an explicitly selected existing
OpenSSL 3 installation, two compiler jobs, locked offline builds after source
qualification, and a short private temporary/target root. Do not activate the owner
playground. See [tools](tools.json) for the recorded environments.

Before any selected compilation, warm the workflow's locked metadata for all
listed Linux, NetBSD and macOS targets and feature graphs, then run its provider
and standalone codec source gates (`tools/check_provider_dependencies.py` and
`tools/check_delegation_codec_dependencies.py`). Metadata warming is not build
approval. Follow the workflow's 16 provider feature selections and standalone codec
commands, including strict host Clippy and serial tests; do not replace them with
an unqualified broad feature combination. Compare the profile and counts with
[results](results.json), preserving each invocation's actual inputs and output.

For an authorized disposable console fixture, use a new private owned root and
exact marker, including its newline, as in the workflow:

```sh
mkdir -m 700 "$RUNNER_TEMP/faction280-console-OWNED"
printf '%s\n' 'Owned authenticated-provider console proof' > "$RUNNER_TEMP/faction280-console-OWNED/OWNED"
```

Use the workflow's explicit `probe_device_provider_console.py` arguments and
prebuilt client and fixture; keep its result and binary identities. Do not infer a
fresh client result from a provider test. Run only owned disposable fixtures,
verify their children exit, preserve evidence privately, and verify owned cleanup.
Do not treat reproduction as deployment or a physical capacity measurement.
