"""Exact accepted BBS CI partition; source checks only, no runtime qualification."""
from pathlib import Path
import hashlib
import re

ROOT = Path(__file__).resolve().parents[1]
TARGETS = ['x86_64-unknown-linux-gnu', 'aarch64-unknown-netbsd', 'aarch64-apple-darwin']
MANIFESTS = ['Cargo.toml', 'experiments/055-shared-ui/Cargo.toml',
             'experiments/058-headroom/Cargo.toml', 'experiments/058-transport-memory/Cargo.toml',
             'experiments/055-shared-board/Cargo.toml']
TOOLCHAIN = 'rustup toolchain install 1.96.0 --profile minimal --component rustfmt --component clippy'
# Complete pre-move step bytes from24f1372. Includes every environment, command
# and the INN heredoc/archive bound; no provider straight-line parser relaxation.
STEP_SHA256 = {'Validate optional Lua game evaluator and examples': '1315d89247197b709a006eb5bd03124bfeafa64ac04f3e4dc1cc4bfcaf9fac0d', 'Validate the local Rust core and real terminal flow': '5b75f4dfbb312f6ddb9a4b2d540492698886bd1396f26a420058a0f06cf0efdd', 'Validate articles with original offline INN': '19d5f9e9bf0e8741e34edc6dd7153c3e72502badd62b2c78fc493a47740460aa'}
REPEATED_COMMAND_COUNTS = {
    TOOLCHAIN: 11,  # Retained lanes plus the fresh sequential durable verifier.
    'cargo fmt --all -- --check': 2,
    'umask 077': 5,
}

def warm_commands():
    return ['cargo metadata --locked --manifest-path ' + manifest +
            ' --format-version 1 --filter-platform ' + target + ' > /dev/null'
            for manifest in MANIFESTS for target in TARGETS]

def validate_ci(source):
    from check_provider_dependencies import _job, _step, _lines, require
    job = _job(source, 'bbs_regressions')
    require(job.split('    steps:\n', 1)[0] ==
            '    name: BBS regressions\n    needs: [own_home_client]\n'
            '    runs-on: ubuntu-24.04\n    timeout-minutes: 15\n',
            'BBS CI must replace the successful own-home lane without overrides')
    require(set(re.findall(r'^  ([a-z_]+):$', source.split('jobs:\n', 1)[1], re.M)) ==
            {'native_mail', 'provider', 'provider_client', 'provider_operator', 'provider_verifier_durable',
             'provider_composed', 'own_home_client', 'bbs_regressions', 'documentation'},
            'BBS partition must retain exactly the accepted nine workflow jobs')
    names = ['Fetch the public repository revision', 'Warm the selected BBS regression source graphs', *STEP_SHA256]
    require(re.findall(r'^      - name: (.*)$', job, re.M) == names,
            'BBS CI checkout, warming and complete moved-step order changed')
    fetch = _step(job, names[0])
    require(fetch == _step(_job(source, 'own_home_client'), names[0]),
            'BBS CI must freshly fetch the exact same workflow revision')
    require('          CHECKOUT_SHA: ${{ github.sha }}' in fetch.splitlines() and
            _lines(fetch, {'git'}) == ['git init .', 'git remote add origin "$REPOSITORY_URL"',
                                     'git fetch --no-tags --depth=1 origin "$CHECKOUT_SHA"',
                                     'git checkout --detach FETCH_HEAD'],
            'BBS checkout cannot substitute another revision')
    warm = _step(job, names[1])
    expected_warm = ('        env:\n'
                     '          CARGO_TARGET_DIR: ${{ runner.temp }}/faction-target\n'
                     '          RUSTUP_TOOLCHAIN: 1.96.0\n'
                     '        run: |\n' +
                     ''.join('          ' + c + '\n' for c in [TOOLCHAIN, *warm_commands(),
                                                             'python3 tools/check_bbs_regressions_ci.py']))
    require(warm == expected_warm, 'BBS CI must warm only the15 locked default graphs before compilation')
    for name, digest in STEP_SHA256.items():
        body = _step(job, name)
        block = '      - name: ' + name + '\n' + body
        require(hashlib.sha256(block.encode()).hexdigest() == digest and source.count(block) == 1,
                'complete moved BBS step bytes or execution count changed: ' + name)
        require('      - name: ' + name not in _job(source, 'documentation'),
                'BBS runtime checks must leave the documentation lane')
        run = body.split('        run: |\n', 1)[1].splitlines()
        # INN is an exact bounded heredoc contract. Count only its three outer
        # commands; never send its Python body through the shell-lines parser.
        commands = ([run[0][10:], run[-2][10:], run[-1][10:]] if name.endswith('offline INN')
                    else [line[10:] for line in run if line.startswith('          ') and line.strip()])
        for command in commands:
            require(source.splitlines().count('          ' + command) == REPEATED_COMMAND_COUNTS.get(command, 1),
                    'moved BBS command omitted or duplicated: ' + command)
    documentation = _job(source, 'documentation')
    require(documentation.splitlines().count('    needs: [native_mail, provider, provider_operator, provider_verifier_durable, provider_client, provider_composed, own_home_client, bbs_regressions]') == 1
            and '    if: ${{ always() }}' in documentation.splitlines(),
            'documentation must require all eight prerequisite results')
    gate = _step(documentation, 'Require optional native tests')
    results = [('NATIVE_MAIL_RESULT','native_mail'), ('PROVIDER_RESULT','provider'),
               ('PROVIDER_OPERATOR_RESULT','provider_operator'), ('PROVIDER_VERIFIER_DURABLE_RESULT','provider_verifier_durable'), ('PROVIDER_CLIENT_RESULT','provider_client'),
               ('PROVIDER_COMPOSED_RESULT','provider_composed'), ('OWN_HOME_CLIENT_RESULT','own_home_client'),
               ('BBS_REGRESSIONS_RESULT','bbs_regressions')]
    for variable, name in results:
        require(gate.splitlines().count('          ' + variable + ': ${{ needs.' + name + '.result }}') == 1,
                'required BBS aggregate result missing or overridden')
    require(_lines(gate, {'test'}) == ['test "$' + variable + '" = success' for variable, _ in results],
            'BBS aggregate must reject failure, cancellation and skipped work')

if __name__ == '__main__':
    validate_ci((ROOT / '.github/workflows/documentation.yml').read_text())
    print('Exact BBS CI predecessor,15-graph warming and moved step contracts match; no runtime claim.')
