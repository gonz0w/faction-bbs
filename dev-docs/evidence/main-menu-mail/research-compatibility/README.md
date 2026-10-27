# Research menu compatibility postlude

Required [CI run40423857898](https://github.com/gonz0w/faction-bbs/actions/runs/40423857898)
correctly rejected the old shared-UI experiment's exhaustive menu match at commit
e270725b665420ccaf19d4bcdfe5afe2e7da5f29. The new HomeMail variant needed an
explicit entry in its existing unavailable-section/Pause fallback.

The [result](result.json) and [byte manifest](manifest.json) preserve an actual
local compiler RED before the one-line fix, then the same strict all-target release
Clippy command, formatting and the existing test GREEN. [Separate review](review.json)
confirms the fallback grants no handoff, unlock or home-network authority. Other
experimental menu consumers already have wildcard fallbacks.

This program is not installed by the maintained service. Its source is separate
from the unchanged maintained crates qualified in the [main record](../results.json).
This postlude adds no native-runtime measurement or refreshed historical hash.
