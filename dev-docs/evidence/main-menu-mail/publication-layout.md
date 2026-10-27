# Frozen Markdown publication layout

The [first complete record](source-objects/a2c16502e0a61c1bc2e09a70272b8c7f0c61ded9966625cadffe6d20969a90cd)
remains unchanged. Its reviewed source subsets included ten Markdown snapshots
whose links referred to their original repository tree. Keeping those snapshots
as current Markdown files caused the normal relative-link check to fail.

The [current record](results.json) maps only those ten logical source paths to
hash-named raw objects. Payload bytes, hashes, native source-stage names, test
results, resource observations and cleanup facts remain identical. The validator
compares the current record with the original after only these declared path
changes; publication metadata also preserves the executed repair source, actual
failing link/unit checks, passing checks and intermediate layout candidates.
No general link-check exemption or refreshed measurement was introduced.
