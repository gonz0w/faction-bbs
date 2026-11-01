"""Fixed CI scheduling and exact moved-step contracts; no product execution."""
from pathlib import Path
import re
import unittest
import check_provider_dependencies as provider

ROOT = Path(__file__).resolve().parents[1]
TARGETS = ['x86_64-unknown-linux-gnu', 'aarch64-unknown-netbsd', 'aarch64-apple-darwin']
MANIFESTS = ['Cargo.toml', 'experiments/055-shared-ui/Cargo.toml',
             'experiments/058-headroom/Cargo.toml', 'experiments/058-transport-memory/Cargo.toml',
             'experiments/055-shared-board/Cargo.toml']
MOVED_STEPS = (
    '      - name: Validate optional Lua game evaluator and examples\n'
    '        run: |\n'
    '          sudo apt-get update -qq\n'
    '          sudo apt-get install -y --no-install-recommends liblua5.4-dev\n'
    '          python3 tools/build_lua_doors.py --include /usr/include/lua5.4 --library lua5.4 --output "$RUNNER_TEMP/faction-doors"\n'
    '          cc -std=c11 -D_GNU_SOURCE -Wall -Wextra -Werror -fanalyzer -I/usr/include/lua5.4 -fsyntax-only native/faction-doors/main.c\n'
    '          sudo install -d -o root -g root -m 755 "$RUNNER_TEMP/faction-door-fixtures"\n'
    '          sudo install -o root -g root -m 644 doors/examples/*.lua tools/fixtures/lua-doors/*.lua "$RUNNER_TEMP/faction-door-fixtures/"\n'
    '          python3 tools/smoke_lua_worker.py --binary "$RUNNER_TEMP/faction-doors" --scripts "$RUNNER_TEMP/faction-door-fixtures"\n'
    '      - name: Validate the local Rust core and real terminal flow\n'
    '        env:\n'
    '          CARGO_TARGET_DIR: ${{ runner.temp }}/faction-target\n'
    '          RUSTUP_TOOLCHAIN: 1.96.0\n'
    '        run: |\n'
    '          rustup toolchain install 1.96.0 --profile minimal --component rustfmt --component clippy\n'
    '          cargo fmt --all -- --check\n'
    '          cargo fmt --manifest-path experiments/055-shared-board/Cargo.toml -- --check\n'
    '          cargo fmt --manifest-path experiments/058-transport-memory/Cargo.toml -- --check\n'
    '          cargo fmt --manifest-path experiments/058-headroom/Cargo.toml -- --check\n'
    '          cargo fmt --manifest-path experiments/055-shared-ui/Cargo.toml -- --check\n'
    '          python3 tools/check_ui_dependencies.py\n'
    '          CARGO_TARGET_DIR="$CARGO_TARGET_DIR/ui" cargo clippy --manifest-path experiments/055-shared-ui/Cargo.toml --all-targets --release --locked -- -D warnings\n'
    '          CARGO_TARGET_DIR="$CARGO_TARGET_DIR/ui" cargo test --manifest-path experiments/055-shared-ui/Cargo.toml --release --locked\n'
    '          CARGO_TARGET_DIR="$CARGO_TARGET_DIR/ui" cargo build --manifest-path experiments/055-shared-ui/Cargo.toml --release --locked\n'
    '          python3 tools/check_headroom_dependencies.py\n'
    '          CARGO_TARGET_DIR="$CARGO_TARGET_DIR/headroom" cargo clippy --manifest-path experiments/058-headroom/Cargo.toml --all-targets --release --locked -- -D warnings\n'
    '          CARGO_TARGET_DIR="$CARGO_TARGET_DIR/headroom" cargo test --manifest-path experiments/058-headroom/Cargo.toml --release --locked\n'
    '          CARGO_TARGET_DIR="$CARGO_TARGET_DIR/headroom" cargo build --manifest-path experiments/058-headroom/Cargo.toml --release --locked\n'
    '          python3 tools/check_transport_dependencies.py\n'
    '          CARGO_TARGET_DIR="$CARGO_TARGET_DIR/transport" cargo clippy --manifest-path experiments/058-transport-memory/Cargo.toml --all-targets --release --locked -- -D warnings\n'
    '          CARGO_TARGET_DIR="$CARGO_TARGET_DIR/transport" cargo build --manifest-path experiments/058-transport-memory/Cargo.toml --release --locked\n'
    '          python3 tools/check_shared_dependencies.py\n'
    '          CARGO_TARGET_DIR="$CARGO_TARGET_DIR/shared" cargo clippy --manifest-path experiments/055-shared-board/Cargo.toml --all-targets --release --locked -- -D warnings\n'
    '          CARGO_TARGET_DIR="$CARGO_TARGET_DIR/shared" cargo build --manifest-path experiments/055-shared-board/Cargo.toml --release --locked\n'
    '          python3 tools/check_rust_dependencies.py\n'
    '          cargo clippy --workspace --all-targets --release --locked -- -D warnings\n'
    '          cargo test --workspace --locked --release\n'
    '          cargo build --workspace --locked --release\n'
    '          python3 tools/probe_picker_home_mail.py --client "$CARGO_TARGET_DIR/release/faction-client" --unsupported --output "$RUNNER_TEMP/picker-default-off.json"\n'
    '          python3 tools/probe_native_mail.py --client "$CARGO_TARGET_DIR/release/faction-client"\n'
    '          python3 tools/smoke_local_vault_launcher.py "$CARGO_TARGET_DIR/release/faction-client"\n'
    '          cargo build -p faction-core --example playground_seed --release --locked\n'
    '          python3 tools/smoke_playground_seed.py "$CARGO_TARGET_DIR/release/examples/playground_seed"\n'
    '          cargo build -p faction-terminal --examples --release --locked\n'
    '          python3 tools/render_theme_preview.py "$CARGO_TARGET_DIR/release/examples/preview_theme" --theme themes/base --output "$RUNNER_TEMP/menu-base.svg"\n'
    '          cmp docs/images/menu-base.svg "$RUNNER_TEMP/menu-base.svg"\n'
    '          python3 tools/render_theme_preview.py "$CARGO_TARGET_DIR/release/examples/preview_theme" --theme themes/base --season themes/seasons/autumn --output "$RUNNER_TEMP/menu-autumn.svg"\n'
    '          cmp docs/images/menu-autumn.svg "$RUNNER_TEMP/menu-autumn.svg"\n'
    '          python3 tools/render_theme_preview.py "$CARGO_TARGET_DIR/release/examples/preview_theme" --theme themes/base --columns 40 --output "$RUNNER_TEMP/menu-compact.svg"\n'
    '          cmp docs/images/menu-compact.svg "$RUNNER_TEMP/menu-compact.svg"\n'
    '          python3 tools/smoke_operator.py "$CARGO_TARGET_DIR/release/faction-sysop"\n'
    '          python3 tools/smoke_operator_health.py "$CARGO_TARGET_DIR/release/faction-sysop"\n'
    '          python3 tools/smoke_passport.py "$CARGO_TARGET_DIR/release/faction-client"\n'
    '          python3 tools/smoke_connection_desk.py "$CARGO_TARGET_DIR/release/faction-client"\n'
    '          python3 tools/smoke_travel.py "$CARGO_TARGET_DIR/release/faction-client"\n'
    '          python3 tools/smoke_board_launcher.py "$CARGO_TARGET_DIR/release/faction-client"\n'
    '          python3 tools/smoke_directory.py "$CARGO_TARGET_DIR/release/faction" "$CARGO_TARGET_DIR/release/faction-member"\n'
    '          python3 tools/smoke_directory_submissions.py "$CARGO_TARGET_DIR/release/faction" "$CARGO_TARGET_DIR/release/faction-member"\n'
    '          python3 tools/smoke_backup_recovery.py "$CARGO_TARGET_DIR/release/faction" "$CARGO_TARGET_DIR/release/faction-sysop"\n'
    '          python3 tools/build_nightwire.py --check\n'
    '          python3 tools/check_nightwire.py "$CARGO_TARGET_DIR/release/examples/preview_theme"\n'
    '          python3 tools/smoke_nightwire.py "$CARGO_TARGET_DIR/release/faction" "$CARGO_TARGET_DIR/release/faction-member"\n'
    '          python3 tools/smoke_login_paging.py "$CARGO_TARGET_DIR/release/faction" "$CARGO_TARGET_DIR/release/faction-member"\n'
    '          python3 tools/render_theme_preview.py "$CARGO_TARGET_DIR/release/examples/preview_theme" --theme themes/longform --login --page 1 --output "$RUNNER_TEMP/login-longform-1.svg"\n'
    '          cmp docs/images/login-longform-1.svg "$RUNNER_TEMP/login-longform-1.svg"\n'
    '          python3 tools/render_theme_preview.py "$CARGO_TARGET_DIR/release/examples/preview_theme" --theme themes/longform --login --page 2 --output "$RUNNER_TEMP/login-longform-2.svg"\n'
    '          cmp docs/images/login-longform-2.svg "$RUNNER_TEMP/login-longform-2.svg"\n'
    '          python3 tools/render_theme_preview.py "$CARGO_TARGET_DIR/release/examples/preview_theme" --theme themes/longform --login --page 3 --output "$RUNNER_TEMP/login-longform-3.svg"\n'
    '          cmp docs/images/login-longform-3.svg "$RUNNER_TEMP/login-longform-3.svg"\n'
    '          python3 tools/smoke_custom_menus.py "$CARGO_TARGET_DIR/release/faction" "$CARGO_TARGET_DIR/release/faction-member"\n'
    '          python3 tools/render_theme_preview.py "$CARGO_TARGET_DIR/release/examples/preview_theme" --theme themes/grid --output "$RUNNER_TEMP/menu-grid.svg"\n'
    '          cmp docs/images/menu-grid.svg "$RUNNER_TEMP/menu-grid.svg"\n'
    '          python3 tools/render_theme_preview.py "$CARGO_TARGET_DIR/release/examples/preview_theme" --theme themes/grid --login --output "$RUNNER_TEMP/login-grid.svg"\n'
    '          cmp docs/images/login-grid.svg "$RUNNER_TEMP/login-grid.svg"\n'
    '          python3 tools/smoke_theme_schedule.py "$CARGO_TARGET_DIR/release/faction" "$CARGO_TARGET_DIR/release/faction-member" "$CARGO_TARGET_DIR/release/examples/preview_theme"\n'
    '          python3 tools/render_theme_preview.py "$CARGO_TARGET_DIR/release/examples/preview_theme" --theme themes/base --season themes/seasons/autumn --login --output "$RUNNER_TEMP/login-autumn.svg"\n'
    '          cmp docs/images/login-autumn.svg "$RUNNER_TEMP/login-autumn.svg"\n'
    '          python3 tools/render_theme_preview.py "$CARGO_TARGET_DIR/release/examples/preview_theme" --theme themes/base --season themes/seasons/winter --login --output "$RUNNER_TEMP/login-winter.svg"\n'
    '          cmp docs/images/login-winter.svg "$RUNNER_TEMP/login-winter.svg"\n'
    '          python3 tools/render_theme_preview.py "$CARGO_TARGET_DIR/release/examples/preview_theme" --theme themes/base --season themes/seasons/winter --login --columns 40 --output "$RUNNER_TEMP/login-compact.svg"\n'
    '          cmp docs/images/login-compact.svg "$RUNNER_TEMP/login-compact.svg"\n'
    '          python3 tools/smoke_local_console.py "$CARGO_TARGET_DIR/release/faction"\n'
    '          python3 tools/smoke_room_browser.py "$CARGO_TARGET_DIR/release/faction" "$CARGO_TARGET_DIR/release/faction-member"\n'
    '          python3 tools/smoke_writing_desk.py "$CARGO_TARGET_DIR/release/faction" "$CARGO_TARGET_DIR/release/faction-member"\n'
    '          python3 tools/smoke_social.py "$CARGO_TARGET_DIR/release/faction" "$CARGO_TARGET_DIR/release/faction-member"\n'
    '          python3 tools/smoke_standalone_social.py "$CARGO_TARGET_DIR/release/faction" "$CARGO_TARGET_DIR/release/faction-member"\n'
    '          python3 tools/smoke_idle_notifications.py "$CARGO_TARGET_DIR/release/faction" "$CARGO_TARGET_DIR/release/faction-member"\n'
    '          python3 tools/smoke_board_guide.py "$CARGO_TARGET_DIR/release/faction" "$CARGO_TARGET_DIR/release/faction-member"\n'
    '          python3 tools/smoke_who.py "$CARGO_TARGET_DIR/release/faction" "$CARGO_TARGET_DIR/release/faction-member"\n'
    '          python3 tools/smoke_editorial.py "$CARGO_TARGET_DIR/release/faction" "$CARGO_TARGET_DIR/release/faction-member"\n'
    '          python3 tools/smoke_magazine.py "$CARGO_TARGET_DIR/release/faction" "$CARGO_TARGET_DIR/release/faction-member"\n'
    '          python3 tools/smoke_file_requests.py "$CARGO_TARGET_DIR/release/faction" "$CARGO_TARGET_DIR/release/faction-member"\n'
    '          python3 tools/smoke_library.py "$CARGO_TARGET_DIR/release/faction" "$CARGO_TARGET_DIR/release/faction-member"\n'
    '          python3 tools/smoke_gallery.py "$CARGO_TARGET_DIR/release/faction" "$CARGO_TARGET_DIR/release/faction-member"\n'
    '          python3 tools/smoke_discussion.py "$CARGO_TARGET_DIR/release/faction" "$CARGO_TARGET_DIR/release/faction-member"\n'
    '          python3 tools/smoke_catchup.py "$CARGO_TARGET_DIR/release/faction" "$CARGO_TARGET_DIR/release/faction-member"\n'
    '          python3 tools/smoke_polls.py "$CARGO_TARGET_DIR/release/faction" "$CARGO_TARGET_DIR/release/faction-member"\n'
    '          python3 tools/smoke_gatherings.py "$CARGO_TARGET_DIR/release/faction" "$CARGO_TARGET_DIR/release/faction-member"\n'
    '          python3 tools/smoke_member_pages.py "$CARGO_TARGET_DIR/release/faction" "$CARGO_TARGET_DIR/release/faction-member"\n'
    '          python3 tools/smoke_wall.py "$CARGO_TARGET_DIR/release/faction" "$CARGO_TARGET_DIR/release/faction-member"\n'
    '          python3 tools/smoke_full_screen.py "$CARGO_TARGET_DIR/release/faction" "$CARGO_TARGET_DIR/release/faction-member"\n'
    '          python3 tools/smoke_room_reader.py "$CARGO_TARGET_DIR/release/faction"\n'
    '          python3 tools/smoke_unicode_subjects.py "$CARGO_TARGET_DIR/release/faction" "$CARGO_TARGET_DIR/release/faction-member"\n'
    '          python3 tools/smoke_modern_terminals.py "$CARGO_TARGET_DIR/release/faction" "$CARGO_TARGET_DIR/release/faction-member"\n'
    '          python3 tools/smoke_kitty_graphics.py "$CARGO_TARGET_DIR/release/faction" "$CARGO_TARGET_DIR/release/faction-member"\n'
    '          python3 tools/smoke_native_graphics.py "$CARGO_TARGET_DIR/release/faction-client"\n'
    '          cargo build -p faction-articles --examples --release --locked\n'
    '          python3 tools/check_article_interop.py "$CARGO_TARGET_DIR/release/examples/emit_fixture"\n'
    '          python3 tools/smoke_local_chat.py "$CARGO_TARGET_DIR/release/faction"\n'
    '          python3 tools/smoke_export_restore.py "$CARGO_TARGET_DIR/release/faction"\n'
    '          python3 tools/smoke_member_worker.py "$CARGO_TARGET_DIR/release/faction" "$CARGO_TARGET_DIR/release/faction-member"\n'
    '          python3 tools/smoke_broker_lifecycle.py "$CARGO_TARGET_DIR/release/faction-broker"\n'
    '          python3 tools/smoke_credential_budget.py "$CARGO_TARGET_DIR/release/faction"\n'
    '\n'
    '      - name: Validate articles with original offline INN\n'
    '        env:\n'
    '          CARGO_TARGET_DIR: ${{ runner.temp }}/faction-target\n'
    '          RUSTUP_TOOLCHAIN: 1.96.0\n'
    '        run: |\n'
    '          umask 077\n'
    '          python3 - "$RUNNER_TEMP/inn-2.7.4.tar.gz" <<\'PY\'\n'
    '          import sys, urllib.request\n'
    '          from pathlib import Path\n'
    '          from tools.build_inn_reference import URL, SHA256\n'
    '          import hashlib\n'
    '          raw = urllib.request.urlopen(URL, timeout=30).read(10 * 1024 * 1024 + 1)\n'
    '          assert len(raw) <= 10 * 1024 * 1024 and hashlib.sha256(raw).hexdigest() == SHA256\n'
    '          Path(sys.argv[1]).write_bytes(raw)\n'
    '          PY\n'
    '          python3 tools/build_inn_reference.py --archive "$RUNNER_TEMP/inn-2.7.4.tar.gz" --work-dir "$RUNNER_TEMP/inn-reference"\n'
    '          python3 tools/check_inn_articles.py --reference-dir "$RUNNER_TEMP/inn-reference" --examples "$CARGO_TARGET_DIR/release/examples"\n'
)
NATIVE_MAIL_FIXTURE = (
    '  native_mail:\n'
    '    name: Optional native mail\n'
    '    runs-on: ubuntu-24.04\n'
    '    timeout-minutes: 15\n'
    '    steps:\n'
    '      - name: Fetch the public repository revision\n'
    '        env:\n'
    '          REPOSITORY_URL: ${{ github.server_url }}/${{ github.repository }}.git\n'
    '          CHECKOUT_SHA: ${{ github.sha }}\n'
    '        run: |\n'
    '          git init .\n'
    '          git remote add origin "$REPOSITORY_URL"\n'
    '          git fetch --no-tags --depth=1 origin "$CHECKOUT_SHA"\n'
    '          git checkout --detach FETCH_HEAD\n'
    '      - name: Validate the optional member-device checkpoint path\n'
    '        env:\n'
    '          CARGO_TARGET_DIR: ${{ runner.temp }}/faction-native-mail\n'
    '          RUSTUP_TOOLCHAIN: 1.96.0\n'
    '        run: |\n'
    '          rustup toolchain install 1.96.0 --profile minimal --component rustfmt --component clippy\n'
    '          cargo fmt --all -- --check\n'
    '          python3 tools/check_omemo_dependencies.py\n'
    '          cargo clippy -p faction-client -p faction-omemo --features faction-client/native-mail --all-targets --release --locked -- -D warnings\n'
    '          cargo test -p faction-client -p faction-omemo --features faction-client/native-mail --all-targets --release --locked -- --test-threads=1\n'
    '          cargo build -p faction-client --features native-mail --bin faction-client --example checkpoint_ui_fixture --release --locked\n'
    '          python3 tools/probe_checkpoint_ui.py --client "$CARGO_TARGET_DIR/release/faction-client" --fixture "$CARGO_TARGET_DIR/release/examples/checkpoint_ui_fixture"\n'
    '          python3 tools/probe_unread_mail.py --client "$CARGO_TARGET_DIR/release/faction-client" --fixture "$CARGO_TARGET_DIR/release/examples/checkpoint_ui_fixture" --output "$RUNNER_TEMP/local-unread-mail.json"\n'
    '          mkdir -m 700 "$RUNNER_TEMP/inbox-owned"\n'
    '          printf \'%s\\n\' \'Owned issue219 inbox controls\' > "$RUNNER_TEMP/inbox-owned/OWNERSHIP"\n'
    '          python3 tools/probe_mail_inbox.py --client "$CARGO_TARGET_DIR/release/faction-client" --fixture "$CARGO_TARGET_DIR/release/examples/checkpoint_ui_fixture" --owned-root "$RUNNER_TEMP/inbox-owned" --output "$RUNNER_TEMP/inbox-native.json"\n'
    '          python3 tools/probe_picker_home_mail.py --client "$CARGO_TARGET_DIR/release/faction-client" --unsupported --output "$RUNNER_TEMP/picker-native-only.json"\n'
    '          cargo metadata --locked --format-version 1 --filter-platform x86_64-unknown-linux-gnu --features faction-client/network-mail > /dev/null\n'
    '          cargo metadata --locked --format-version 1 --filter-platform aarch64-unknown-netbsd --features faction-client/network-mail > /dev/null\n'
    '          cargo metadata --locked --format-version 1 --filter-platform aarch64-apple-darwin --features faction-client/network-mail > /dev/null\n'
    '          python3 tools/check_network_mail_dependencies.py\n'
    '          python3 tools/check_xmpp_transport_dependencies.py\n'
    '          cargo clippy -p faction-client -p faction-omemo --features faction-client/network-mail --all-targets --release --locked -- -D warnings\n'
    '          cargo test -p faction-client -p faction-omemo --features faction-client/network-mail --all-targets --release --locked -- --test-threads=1\n'
    '          cargo build -p faction-client --features network-mail --bin faction-client --example checkpoint_ui_fixture --example home_mail_fixture --example mail_reply_fixture --example mail_correspondents_fixture --release --locked\n'
    '          mkdir -m 700 "$RUNNER_TEMP/reply-owned"\n'
    '          printf \'CI owned reply fixtures\\n\' > "$RUNNER_TEMP/reply-owned/OWNED"\n'
    '          python3 tools/probe_mail_reply.py --client "$CARGO_TARGET_DIR/release/faction-client" --fixture "$CARGO_TARGET_DIR/release/examples/mail_reply_fixture" --owned-root "$RUNNER_TEMP/reply-owned" --output "$RUNNER_TEMP/reply-classic.json"\n'
    '          python3 tools/probe_mail_reply.py --client "$CARGO_TARGET_DIR/release/faction-client" --fixture "$CARGO_TARGET_DIR/release/examples/mail_reply_fixture" --owned-root "$RUNNER_TEMP/reply-owned" --unicode --output "$RUNNER_TEMP/reply-unicode.json"\n'
    '          mkdir -m 700 "$RUNNER_TEMP/correspondents-owned"\n'
    '          printf \'CI owned correspondent fixtures\\n\' > "$RUNNER_TEMP/correspondents-owned/OWNED"\n'
    '          python3 tools/probe_mail_correspondents.py --client "$CARGO_TARGET_DIR/release/faction-client" --fixture "$CARGO_TARGET_DIR/release/examples/mail_correspondents_fixture" --owned-root "$RUNNER_TEMP/correspondents-owned" --output "$RUNNER_TEMP/correspondents-classic.json"\n'
    '          python3 tools/probe_mail_correspondents.py --client "$CARGO_TARGET_DIR/release/faction-client" --fixture "$CARGO_TARGET_DIR/release/examples/mail_correspondents_fixture" --owned-root "$RUNNER_TEMP/correspondents-owned" --unicode --output "$RUNNER_TEMP/correspondents-unicode.json"\n'
    '          python3 tools/probe_mail_inbox.py --client "$CARGO_TARGET_DIR/release/faction-client" --fixture "$CARGO_TARGET_DIR/release/examples/checkpoint_ui_fixture" --owned-root "$RUNNER_TEMP/inbox-owned" --online --output "$RUNNER_TEMP/inbox-network.json"\n'
    '          python3 tools/probe_home_ui.py --client "$CARGO_TARGET_DIR/release/faction-client" --checkpoint-fixture "$CARGO_TARGET_DIR/release/examples/checkpoint_ui_fixture" --mail-fixture "$CARGO_TARGET_DIR/release/examples/home_mail_fixture"\n'
    '          python3 tools/probe_home_archive_ui.py --client "$CARGO_TARGET_DIR/release/faction-client" --fixture "$CARGO_TARGET_DIR/release/examples/checkpoint_ui_fixture" --output "$RUNNER_TEMP/archive-ui.json"\n'
    '          python3 tools/probe_picker_home_mail.py --client "$CARGO_TARGET_DIR/release/faction-client" --checkpoint-fixture "$CARGO_TARGET_DIR/release/examples/checkpoint_ui_fixture" --mail-fixture "$CARGO_TARGET_DIR/release/examples/home_mail_fixture" --output "$RUNNER_TEMP/picker-home-mail.json"\n'
    '          cargo fmt --manifest-path crates/faction-xmpp/Cargo.toml -- --check\n'
    '          cargo clippy --manifest-path crates/faction-xmpp/Cargo.toml --all-targets --release --locked -- -D warnings\n'
    '          cargo test --manifest-path crates/faction-xmpp/Cargo.toml --all-targets --release --locked\n'
    '          cargo build --manifest-path crates/faction-xmpp/Cargo.toml --example transport_fixture --release --locked\n'
    '          python3 tools/probe_home_transport.py --binary "$CARGO_TARGET_DIR/release/examples/transport_fixture"\n'
    '          cargo fmt --manifest-path experiments/208-xmpp-transport/Cargo.toml -- --check\n'
    '          cargo clippy --manifest-path experiments/208-xmpp-transport/Cargo.toml --all-targets --release --locked -- -D warnings\n'
    '          cargo test --manifest-path experiments/208-xmpp-transport/Cargo.toml --all-targets --release --locked\n'
    '          cargo build --manifest-path experiments/208-xmpp-transport/Cargo.toml --release --locked\n'
    '          python3 tools/probe_xmpp_transport.py --binary "$CARGO_TARGET_DIR/release/faction-xmpp-probe"\n'
    '          python3 tools/check_provider_auth_dependencies.py\n'
    '          cargo fmt --manifest-path experiments/270-provider-auth/Cargo.toml -- --check\n'
    '          cargo clippy --manifest-path experiments/270-provider-auth/Cargo.toml --all-targets --release --locked -- -D warnings\n'
    '          cargo test --manifest-path experiments/270-provider-auth/Cargo.toml --release --locked -- --test-threads=1\n'
)
NAMES = ['Validate optional Lua game evaluator and examples',
         'Validate the local Rust core and real terminal flow',
         'Validate articles with original offline INN']

def warm_commands():
    return ['cargo metadata --locked --manifest-path ' + manifest +
            ' --format-version 1 --filter-platform ' + target + ' > /dev/null'
            for manifest in MANIFESTS for target in TARGETS]

def bbs_fixture(source):
    """Add the accepted literal successor to existing provider CI fixtures."""
    if '  native_mail:\n' not in source:
        source = source.replace('jobs:\n', 'jobs:\n' + NATIVE_MAIL_FIXTURE, 1)
    if '      - name: Validate the isolated delegation wire codec\n' not in source:
        from check_delegation_codec_dependencies import ci_step
        source += ci_step()
    fetch = ('      - name: Fetch the public repository revision\n' +
             provider._step(provider._job(source, 'own_home_client'), 'Fetch the public repository revision'))
    warm = ('      - name: Warm the selected BBS regression source graphs\n'
            '        env:\n'
            '          CARGO_TARGET_DIR: ${{ runner.temp }}/faction-target\n'
            '          RUSTUP_TOOLCHAIN: 1.96.0\n'
            '        run: |\n'
            '          ' + provider.TOOLCHAIN + '\n' +
            ''.join('          ' + line + '\n' for line in warm_commands()) +
            '          python3 tools/check_bbs_regressions_ci.py\n')
    job = ('  bbs_regressions:\n'
           '    name: BBS regressions\n'
           '    needs: [own_home_client]\n'
           '    runs-on: ubuntu-24.04\n'
           '    timeout-minutes: 15\n'
           '    steps:\n' + fetch + warm + MOVED_STEPS)
    for name in NAMES:
        block = re.search(r'(?ms)^      - name: ' + re.escape(name) + r'\n.*?(?=^      - |\Z)', source)
        if block:
            source = source.replace(block.group(), '', 1)
    source = source.replace('  documentation:\n', job + '  documentation:\n', 1)
    source = source.replace('needs: [native_mail, provider, provider_operator, provider_client, provider_composed, own_home_client]',
                            'needs: [native_mail, provider, provider_operator, provider_client, provider_composed, own_home_client, bbs_regressions]')
    source = source.replace('          OWN_HOME_CLIENT_RESULT: ${{ needs.own_home_client.result }}\n',
                            '          OWN_HOME_CLIENT_RESULT: ${{ needs.own_home_client.result }}\n'
                            '          BBS_REGRESSIONS_RESULT: ${{ needs.bbs_regressions.result }}\n')
    return source.replace('          test "$OWN_HOME_CLIENT_RESULT" = success\n',
                          '          test "$OWN_HOME_CLIENT_RESULT" = success\n'
                          '          test "$BBS_REGRESSIONS_RESULT" = success\n')

class BbsRegressionCI(unittest.TestCase):
    def source(self):
        return (ROOT / '.github/workflows/documentation.yml').read_text()

    def test_actual_bbs_lane_replaces_completed_own_home_slot_with_minimal_fresh_warming(self):
        source = self.source()
        self.assertIn('  bbs_regressions:\n', source)
        job = provider._job(source, 'bbs_regressions')
        self.assertIn('    needs: [own_home_client]', job.splitlines())
        self.assertIn('    runs-on: ubuntu-24.04', job.splitlines())
        self.assertIn('    timeout-minutes: 15', job.splitlines())
        self.assertNotIn('CARGO_BUILD_JOBS', job)
        fetch = provider._step(job, 'Fetch the public repository revision')
        self.assertEqual(fetch, provider._step(provider._job(source, 'own_home_client'), 'Fetch the public repository revision'))
        warm = provider._step(job, 'Warm the selected BBS regression source graphs')
        actual = [line.strip() for line in warm.splitlines() if line.strip().startswith('cargo metadata ')]
        self.assertEqual(actual, warm_commands())
        self.assertEqual(len(actual), 15)
        self.assertNotIn('--features', warm)
        self.assertNotIn('check_provider_dependencies.py', job)
        self.assertLess(job.index(warm), job.index('      - name: ' + NAMES[0]))
        self.assertIn('          python3 tools/check_bbs_regressions_ci.py', warm.splitlines())
        dependencies = {'native_mail': set(), 'provider': set(), 'provider_operator': set(), 'provider_client': set(),
                        'provider_composed': {'provider'}, 'own_home_client': {'provider_client'},
                        'bbs_regressions': {'own_home_client'}}
        from itertools import combinations
        peak = 0
        for size in range(8):
            for completed in combinations(dependencies, size):
                done = set(completed)
                if all(dependencies[name] <= done for name in done):
                    peak = max(peak, sum(name not in done and needs <= done for name, needs in dependencies.items()))
        self.assertEqual(peak, 4)

    def test_lua_core_and_inn_move_complete_once_with_shared_target_and_unchanged_environments(self):
        source = self.source()
        self.assertIn('  bbs_regressions:\n', source)
        job = provider._job(source, 'bbs_regressions')
        self.assertIn(MOVED_STEPS, job)
        self.assertEqual(source.count(MOVED_STEPS), 1)
        self.assertEqual(re.findall(r'^      - name: (.*)$', job, re.M),
                         ['Fetch the public repository revision', 'Warm the selected BBS regression source graphs', *NAMES])
        for name in NAMES:
            self.assertNotIn('      - name: ' + name, provider._job(source, 'documentation'))
        for name in NAMES[1:]:
            body = provider._step(job, name)
            self.assertIn('          CARGO_TARGET_DIR: ${{ runner.temp }}/faction-target', body.splitlines())
            self.assertNotIn('CARGO_BUILD_JOBS', body)
        self.assertEqual(source.count('timeout-minutes: 15'), 8)

    def test_aggregate_requires_bbs_success_after_any_failure_skip_or_cancellation(self):
        source = self.source()
        doc = provider._job(source, 'documentation')
        self.assertIn('    needs: [native_mail, provider, provider_operator, provider_client, provider_composed, own_home_client, bbs_regressions]', doc.splitlines())
        self.assertIn('    if: ${{ always() }}', doc.splitlines())
        gate = provider._step(doc, 'Require optional native tests')
        self.assertIn('          BBS_REGRESSIONS_RESULT: ${{ needs.bbs_regressions.result }}', gate.splitlines())
        self.assertIn('          test "$BBS_REGRESSIONS_RESULT" = success', gate.splitlines())
        for step in ['Validate the isolated delegation wire codec', 'Validate documentation, policies, and archive']:
            self.assertIn('      - name: ' + step, doc)

    def test_existing_checker_cannot_accept_the_old_unsplit_layout(self):
        source = re.sub(r'(?ms)^  bbs_regressions:\n.*?(?=^  [a-z_]+:|\Z)', '', self.source())
        with self.assertRaises(ValueError):
            provider.validate_ci(source)

    def test_fixed_guard_rejects_wrong_predecessor_caps_skips_and_missing_warm_graphs(self):
        source = self.source()
        self.assertIn('  bbs_regressions:\n', source)
        import check_bbs_regressions_ci as guard
        guard.validate_ci(source)
        job = provider._job(source, 'bbs_regressions')
        for old, new in [('needs: [own_home_client]', 'needs: []'),
                         ('needs: [own_home_client]', 'needs: [provider_composed]'),
                         ('timeout-minutes: 15', 'timeout-minutes: 30'),
                         ('    steps:', '    if: ${{ always() }}\n    steps:'),
                         ('    steps:', '    continue-on-error: true\n    steps:'),
                         ('    steps:', '    env:\n      CARGO_BUILD_JOBS: 2\n    steps:'),
                         ('CHECKOUT_SHA: ${{ github.sha }}', 'CHECKOUT_SHA: main')]:
            with self.subTest(change=new), self.assertRaises(ValueError):
                guard.validate_ci(source.replace(job, job.replace(old, new)))
        for command in warm_commands():
            line = '          ' + command + '\n'
            for replacement in ['', line.replace(' --locked', ''), line.replace(' --locked', ' --locked --offline'), line + line]:
                with self.subTest(warm=command, replacement=replacement), self.assertRaises(ValueError):
                    guard.validate_ci(source.replace(job, job.replace(line, replacement, 1)))
        for old, new in [('${{ needs.bbs_regressions.result }}', 'success'),
                         ('test "$BBS_REGRESSIONS_RESULT" = success', 'test "$BBS_REGRESSIONS_RESULT" = success || true')]:
            with self.subTest(result=new), self.assertRaises(ValueError):
                provider.validate_ci(source.replace(old, new))

    def test_exact_steps_reject_omissions_reordering_duplicates_and_changed_inn_heredoc(self):
        source = self.source()
        self.assertIn('  bbs_regressions:\n', source)
        import check_bbs_regressions_ci as guard
        job = provider._job(source, 'bbs_regressions')
        for name in NAMES:
            block = '      - name: ' + name + '\n' + provider._step(job, name)
            with self.subTest(missing=name), self.assertRaises(ValueError):
                guard.validate_ci(source.replace(block, '', 1))
            with self.subTest(duplicate=name), self.assertRaises(ValueError):
                guard.validate_ci(source.replace(block, block + block, 1))
            run = block.split('        run: |\n', 1)[1]
            for line in run.splitlines():
                if line.startswith('          ') and line.strip():
                    with self.subTest(altered_line=line), self.assertRaises(ValueError):
                        guard.validate_ci(source.replace(block, block.replace(line + '\n', '', 1), 1))
        lua = '      - name: ' + NAMES[0] + '\n' + provider._step(job, NAMES[0])
        core = '      - name: ' + NAMES[1] + '\n' + provider._step(job, NAMES[1])
        with self.assertRaises(ValueError):
            guard.validate_ci(source.replace(lua + core, core + lua, 1))
        inn = '      - name: ' + NAMES[2] + '\n' + provider._step(job, NAMES[2])
        for old, new in [('timeout=30', 'timeout=60'), ('10 * 1024 * 1024 + 1', '100 * 1024 * 1024 + 1'),
                         ('hashlib.sha256(raw).hexdigest() == SHA256', 'True')]:
            self.assertIn(old, inn)
            with self.subTest(heredoc=new), self.assertRaises(ValueError):
                guard.validate_ci(source.replace(inn, inn.replace(old, new, 1), 1))

if __name__ == '__main__':
    unittest.main()
