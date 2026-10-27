"""Resolve the exact reviewed inventory for the unchanged standalone experiments."""
import hashlib
import json
from evidence_sources import ROOT, network_dependency_baseline_manifest, source_bytes

INVENTORY_NAME = 'dev-docs/dependencies/local-core.json'


def reviewed_research_inventory(root=ROOT):
    # These experiment locks predate product network-mail adoption. Resolve the
    # independently pinned pre-209 bytes, never a union with the rolling product
    # inventory: a product dependency update grants no new research allowance.
    baseline = network_dependency_baseline_manifest(root)
    digest = baseline['files'][INVENTORY_NAME]
    raw = source_bytes(INVENTORY_NAME, digest, root=root)
    assert hashlib.sha256(raw).hexdigest() == digest, 'reviewed research inventory changed'
    return json.loads(raw)
