#!/usr/bin/env python3
"""Check the selected product network-mail graph offline; not a security audit."""
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
INVENTORY = ROOT / 'dev-docs/dependencies/network-mail.json'
TARGETS = ['x86_64-unknown-linux-gnu', 'aarch64-unknown-netbsd', 'aarch64-apple-darwin']
REGISTRY = 'registry+https://github.com/rust-lang/crates.io-index'

def scan_package(package, features, checksum):
    raise ValueError('network dependency source review not implemented')

def verify_archive_source(root, archive, checksum):
    raise ValueError('network archive/source verification not implemented')

def selected_packages(metadata):
    raise ValueError('network selected graph review not implemented')

def validate_profile(workspace, client, transport):
    raise ValueError('network feature profile review not implemented')

def check_inventory(path, current):
    raise ValueError('network inventory comparison not implemented')
