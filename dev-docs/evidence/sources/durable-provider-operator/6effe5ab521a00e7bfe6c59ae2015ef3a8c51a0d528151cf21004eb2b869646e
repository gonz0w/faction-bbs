"""Policy controls for the isolated provider authority experiment."""
import copy
from pathlib import Path
import unittest
import check_provider_auth_dependencies as policy

ROOT=Path(__file__).resolve().parents[1]

def manifest():
    return {'package':{'name':'faction-provider-auth-research','version':'0.0.0','edition':'2024','rust-version':'1.96','license':'BSD-3-Clause','publish':False}, 'workspace':{}, 'lib':{'path':'src/lib.rs'}, 'dependencies':{'faction-files':{'path':'../../crates/faction-files'},'openssl':'=0.10.81'}}

def ci_script(script,step_extra=''):
    return 'jobs:\n  native_mail:\n    steps:\n      - name: Validate the optional member-device checkpoint path\n'+step_extra+'        run: |\n'+''.join('          '+line+'\n' for line in script.splitlines())+'  documentation:\n    needs: native_mail\n'

class ProfileTests(unittest.TestCase):
    def test_exact_isolated_profile(self):
        policy.validate_profile(manifest(), {'workspace':{'members':['crates/faction-files']}})
    def test_no_automatic_product_membership(self):
        for member in ['experiments/270-provider-auth','experiments/*','experiments/270-*']:
            with self.subTest(member=member),self.assertRaises(ValueError):
                policy.validate_profile(manifest(),{'workspace':{'members':[member]}})
    def test_dependency_drift_vendoring_and_replacement_refused(self):
        changes=[('dependencies',{'faction-files':{'path':'../../crates/faction-files'},'openssl':'0.10'}),('dependencies',{'faction-files':{'path':'../../crates/faction-files'},'openssl':{'version':'=0.10.81','features':['vendored']}}),('patch',{}),('replace',{}),('features',{'default':['online']}),('dev-dependencies',{'unreviewed':'*'}),('build-dependencies',{'unreviewed':'*'})]
        for key,value in changes:
            m=manifest();m[key]=value
            with self.subTest(key=key),self.assertRaises(ValueError):policy.validate_profile(m,{'workspace':{'members':[]}})
    def test_package_and_entrypoint_drift_refused(self):
        for key,value in [('publish',True),('license','unknown'),('rust-version','1.95')]:
            m=manifest();m['package'][key]=value
            with self.subTest(key=key),self.assertRaises(ValueError):policy.validate_profile(m,{'workspace':{'members':[]}})
        m=manifest();m['workspace']={'members':['extra']}
        with self.assertRaises(ValueError):policy.validate_profile(m,{'workspace':{'members':[]}})
    def test_only_exact_existing_registry_versions_and_checksums(self):
        p={'name':'openssl','version':'0.10.81','source':policy.REGISTRY,'checksum':'a'*64}
        root={'package':[p]};experiment={'package':[dict(p),{'name':'local','version':'0.0.0'}]}
        self.assertEqual(policy.existing_checks(root,experiment),{('openssl','0.10.81',policy.REGISTRY):'a'*64})
        for key,value in [('version','0.10.82'),('checksum','b'*64),('source','git+https://example.invalid')]:
            changed=copy.deepcopy(experiment);changed['package'][0][key]=value
            with self.subTest(key=key),self.assertRaises(ValueError):policy.existing_checks(root,changed)
    def test_registry_duplicate_identity_refused(self):
        p={'name':'openssl','version':'0.10.81','source':policy.REGISTRY,'checksum':'a'*64}
        with self.assertRaises(ValueError):policy.existing_checks({'package':[p]},{'package':[p,p]})
    def test_required_ci_checks_sources_before_experimental_tests(self):
        policy.validate_ci((ROOT/'.github/workflows/documentation.yml').read_text())
    def test_ci_comment_echo_or_failure_waiver_is_not_execution(self):
        commands=['python3 tools/check_provider_auth_dependencies.py',
          'cargo fmt --manifest-path experiments/270-provider-auth/Cargo.toml -- --check',
          'cargo clippy --manifest-path experiments/270-provider-auth/Cargo.toml --all-targets --release --locked -- -D warnings',
          'cargo test --manifest-path experiments/270-provider-auth/Cargo.toml --release --locked -- --test-threads=1']
        for prefix,suffix in [('# ',''),('echo ',''),('',' || true')]:
            with self.subTest(prefix=prefix,suffix=suffix),self.assertRaises(ValueError):
                policy.validate_ci(ci_script('\n'.join(prefix+c+suffix for c in commands)))

    def test_inert_shell_or_conditional_step_is_not_ci_execution(self):
        script='\n'.join(['python3 tools/check_provider_auth_dependencies.py',
          'cargo fmt --manifest-path experiments/270-provider-auth/Cargo.toml -- --check',
          'cargo clippy --manifest-path experiments/270-provider-auth/Cargo.toml --all-targets --release --locked -- -D warnings',
          'cargo test --manifest-path experiments/270-provider-auth/Cargo.toml --release --locked -- --test-threads=1'])
        for source in [ci_script('if false; then\n'+script+'\nfi'),ci_script("cat <<'INERT'\n"+script+'\nINERT'),ci_script(script,'        if: false\n'),ci_script(script).replace('    steps:', '    if: false\n    steps:')]:
            with self.assertRaises(ValueError):policy.validate_ci(source)

    def test_ci_missing_or_unlocked_proof_refused(self):
        source='python3 tools/check_provider_auth_dependencies.py'
        fmt='cargo fmt --manifest-path experiments/270-provider-auth/Cargo.toml -- --check'
        lint='cargo clippy --manifest-path experiments/270-provider-auth/Cargo.toml --all-targets --release --locked -- -D warnings'
        test='cargo test --manifest-path experiments/270-provider-auth/Cargo.toml --release --locked -- --test-threads=1'
        good='\n'.join([source,fmt,lint,test]);policy.validate_ci(ci_script(good))
        for bad in [good.replace(source,''),good.replace('--locked',''),good.replace('--test-threads=1',''),good.replace(fmt,''),good.replace(lint,''),'\n'.join([test,source,fmt,lint])]:
            with self.assertRaises(ValueError):policy.validate_ci(ci_script(bad))

if __name__=='__main__':unittest.main()
