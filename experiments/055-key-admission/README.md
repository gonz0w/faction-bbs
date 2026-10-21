# Base-SSH key-admission accounting

The native metrics collector counts the base listener and all its descendants,
including transient key helpers and shell setup. The resource collector records
live limits and separately counts any SSH zombies; the selected path has zero
config-sender zombies at the measured steady stages. Whole-host counters retain
all OS and fixture costs. RSS sums are not unique/physical application budgets.

See [research 0036](../../dev-docs/research/0036-ssh-admission.md) and
[the operational contract](../../docs/experimental-admission.md). No private keys,
endpoints, generated passwords, transcripts or community data belong here.
