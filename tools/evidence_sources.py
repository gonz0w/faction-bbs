"""Resolve historical executed bytes without rewriting their recorded digests."""
import hashlib,json,re
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
MANIFEST_SHA256="cfbb0a840f3db6dc79954dfc49a80eb5f3a637560494e2eb4cfda7cb4ff717ac"

def manifest(root=ROOT):
    raw=(root/'dev-docs/evidence/manifest.json').read_bytes()
    assert hashlib.sha256(raw).hexdigest()==MANIFEST_SHA256,'frozen manifest changed'
    return json.loads(raw)

def key_manifest(root=ROOT):
    raw=(root/'dev-docs/evidence/key-lookup-manifest.json').read_bytes()
    assert hashlib.sha256(raw).hexdigest()=='db2914814764384ffd16d04dfce5de16a8a769c67fa494bb085b9a113bc81176','frozen key manifest changed'
    return json.loads(raw)

def global_manifest(root=ROOT):
    raw=(root/'dev-docs/evidence/global-admission-manifest.json').read_bytes()
    assert hashlib.sha256(raw).hexdigest()=='c578e074722b97c2fbbbeb543b0572199da52933190884f871ef4cbc22939e51','frozen global manifest changed'
    return json.loads(raw)

def file_baseline_manifest(root=ROOT):
    raw=(root/'dev-docs/evidence/file-areas-baseline-manifest.json').read_bytes()
    assert hashlib.sha256(raw).hexdigest()=='04e2da05bdb34f926bb7e85d181973bffde5bed1e0f8874f7492fabc7e57cfb5','frozen file baseline changed'
    return json.loads(raw)

def native_client_baseline_manifest(root=ROOT):
    raw=(root/'dev-docs/evidence/native-client-baseline-manifest.json').read_bytes()
    assert hashlib.sha256(raw).hexdigest()=='8139ae0981e9418ef9f10a6877aa051ecffad825fc9b9e3da56ee1caaebdc231','frozen native client baseline changed'
    return json.loads(raw)


def mail_client_baseline_manifest(root=ROOT):
    raw=(root/'dev-docs/evidence/mail-client-baseline-manifest.json').read_bytes()
    assert hashlib.sha256(raw).hexdigest()=='54b81f2454c64e4cb598bd55d8b8a535fd31f3bded266b15ed8c99346bd8c0b9','frozen mail client baseline changed'
    return json.loads(raw)


def room_reader_baseline_manifest(root=ROOT):
    raw=(root/'dev-docs/evidence/room-reader-baseline-manifest.json').read_bytes()
    assert hashlib.sha256(raw).hexdigest()=='ba5a596d9b5527ad171d6ea2f3d70f0ecda6d3490de9e04c902ef39489a80d65','frozen room reader baseline changed'
    return json.loads(raw)


def post_metadata_baseline_manifest(root=ROOT):
    raw=(root/'dev-docs/evidence/post-metadata-baseline-manifest.json').read_bytes()
    assert hashlib.sha256(raw).hexdigest()=='ed55e477c95b313004aebe7f7ab00aa16d9640bdc20f60a1490b6b011182cd57','frozen post metadata baseline changed'
    return json.loads(raw)


def authored_articles_baseline_manifest(root=ROOT):
    raw=(root/'dev-docs/evidence/authored-articles-baseline-manifest.json').read_bytes()
    assert hashlib.sha256(raw).hexdigest()=='4cc8ba0773e4675f875ae1f53c3043da8ee6a89ef11a140de99f646a2ad27c88','frozen authored article baseline changed'
    return json.loads(raw)


def broader_articles_baseline_manifest(root=ROOT):
    raw=(root/'dev-docs/evidence/broader-articles-baseline-manifest.json').read_bytes()
    assert hashlib.sha256(raw).hexdigest()=='7ad3539aa44efb95564aa342adeb7fce0e3dd2ed0f9ec573a14be740706fa6fa','frozen broader article baseline changed'
    return json.loads(raw)


def unicode_subjects_baseline_manifest(root=ROOT):
    raw=(root/'dev-docs/evidence/unicode-subjects-baseline-manifest.json').read_bytes()
    assert hashlib.sha256(raw).hexdigest()=='ef31461f4405196c3b1b24b2a7a4abe80353ab652ed14ef1426e043daa61798d','frozen Unicode subject baseline changed'
    return json.loads(raw)


def modern_terminals_baseline_manifest(root=ROOT):
    raw=(root/"dev-docs/evidence/modern-terminals-baseline-manifest.json").read_bytes()
    assert hashlib.sha256(raw).hexdigest()=="8eda37a79f543ac2c35f9bbe0159d14bcbbecec8f3377dabe04c81bc84ebb79b","frozen modern terminals baseline changed"
    return json.loads(raw)


def kitty_graphics_baseline_manifest(root=ROOT):
    raw=(root/"dev-docs/evidence/kitty-graphics-baseline-manifest.json").read_bytes()
    assert hashlib.sha256(raw).hexdigest()=="e9514c5de7f0fd478f2892daef97a950d1ebe9a9011fe69fe869ffea130e280c","frozen Kitty graphics baseline changed"
    return json.loads(raw)


def terminal_compatibility_baseline_manifest(root=ROOT):
    raw=(root/"dev-docs/evidence/terminal-compatibility-baseline-manifest.json").read_bytes()
    assert hashlib.sha256(raw).hexdigest()=="e82ff3c9aee567aa1070b0b915957cb64cdc8855669fa42ede54659db369dc72","frozen terminal compatibility baseline changed"
    return json.loads(raw)


def menu_themes_baseline_manifest(root=ROOT):
    raw=(root/"dev-docs/evidence/menu-themes-baseline-manifest.json").read_bytes()
    assert hashlib.sha256(raw).hexdigest()=="b794f6c283fe98e9896010f765d2539ee660b53706d272c0a026a09f017c1fc7","frozen menu themes baseline changed"
    return json.loads(raw)


def seasonal_themes_baseline_manifest(root=ROOT):
    raw=(root/"dev-docs/evidence/seasonal-themes-baseline-manifest.json").read_bytes()
    assert hashlib.sha256(raw).hexdigest()=="8521bc0b91fb022ba23d9bfd134314c71b2da2521c7bee5a6604d390809334aa","frozen seasonal themes baseline changed"
    return json.loads(raw)


def custom_menus_baseline_manifest(root=ROOT):
    raw=(root/"dev-docs/evidence/custom-menus-baseline-manifest.json").read_bytes()
    assert hashlib.sha256(raw).hexdigest()=="090ab8055e1695513b95926fbc244bdedefb53af5a50a7d591c5d37060ecde11","frozen custom menus baseline changed"
    return json.loads(raw)


def nightwire_baseline_manifest(root=ROOT):
    raw=(root/'dev-docs/evidence/nightwire-baseline-manifest.json').read_bytes()
    assert hashlib.sha256(raw).hexdigest()=='7f7e77b55f7b8b394911b364fde23dedaffc18c2f04740bff4d02080aec12c5a','frozen Nightwire baseline changed'
    return json.loads(raw)


def operator_baseline_manifest(root=ROOT):
    raw=(root/'dev-docs/evidence/operator-baseline-manifest.json').read_bytes()
    assert hashlib.sha256(raw).hexdigest()=='bf4bf77bb51e49e00c32599553a911cc484bf05ca048cc870fbbab640787debc','frozen operator baseline changed'
    return json.loads(raw)


def operator_host_baseline_manifest(root=ROOT):
    raw=(root/'dev-docs/evidence/operator-host-baseline-manifest.json').read_bytes()
    assert hashlib.sha256(raw).hexdigest()=='cbba9764c8225fdabd429a675f25ab3ff1600e46f78938e8d6f6e972faf27780','frozen operator host baseline changed'
    return json.loads(raw)


def operator_recovery_baseline_manifest(root=ROOT):
    raw=(root/'dev-docs/evidence/operator-recovery-baseline-manifest.json').read_bytes()
    assert hashlib.sha256(raw).hexdigest()=='cbb50e6ceeeb4dea59c56c5b79eb015e955627450de9637d21b5a4114ec05374','frozen operator host baseline changed'
    return json.loads(raw)


def directory_baseline_manifest(root=ROOT):
    raw=(root/'dev-docs/evidence/directory-baseline-manifest.json').read_bytes()
    assert hashlib.sha256(raw).hexdigest()=='0591b1fdb8b9da9ce7fbcd9e5c85762ba88fbe90d46e1e8d626af3e31a13a87a','frozen directory baseline changed'
    return json.loads(raw)


def board_launcher_baseline_manifest(root=ROOT):
    raw=(root/'dev-docs/evidence/board-launcher-baseline-manifest.json').read_bytes()
    assert hashlib.sha256(raw).hexdigest()=='3696efece3b953d6abfee7bb3a41ce88574297f289512fe68801949beb32579f','frozen board launcher baseline changed'
    return json.loads(raw)


def room_browser_baseline_manifest(root=ROOT):
    raw=(root/'dev-docs/evidence/room-browser-baseline-manifest.json').read_bytes()
    assert hashlib.sha256(raw).hexdigest()=='b677ac3f8d58d5df065be8fa71a20c15b8a8cb852cccf530a05b06a3ebee36e5','frozen room browser baseline changed'
    return json.loads(raw)


def writing_desk_baseline_manifest(root=ROOT):
    raw=(root/'dev-docs/evidence/writing-desk-baseline-manifest.json').read_bytes()
    assert hashlib.sha256(raw).hexdigest()=='2438cfe859e88c97eaeddec35d2ea87b3ee840a3c8733379ee3490a054c0ed5f','frozen writing desk baseline changed'
    return json.loads(raw)


def full_screen_baseline_manifest(root=ROOT):
    raw=(root/'dev-docs/evidence/full-screen-baseline-manifest.json').read_bytes()
    assert hashlib.sha256(raw).hexdigest()=='d6ac541742282d8ea6f1e1796ed3203e5839af9e48fc039159b18569a8942f75','frozen full-screen baseline changed'
    return json.loads(raw)


def community_wall_baseline_manifest(root=ROOT):
    raw=(root/'dev-docs/evidence/community-wall-baseline-manifest.json').read_bytes()
    assert hashlib.sha256(raw).hexdigest()=='4411bc3eed9b3dd3e4935baa50c7ba869b96c797fea8bb3a698fd04cc711a5b8','frozen community wall baseline changed'
    return json.loads(raw)



def member_pages_baseline_manifest(root=ROOT):
    raw=(root/'dev-docs/evidence/member-pages-baseline-manifest.json').read_bytes()
    assert hashlib.sha256(raw).hexdigest()=='29de7889416378014bc45cb29e83246b2747b6f324392224191182a9441875e0','frozen integration baseline changed'
    return json.loads(raw)


def local_mail_vault_baseline_manifest(root=ROOT):
    raw=(root/'dev-docs/evidence/local-mail-vault-baseline-manifest.json').read_bytes()
    assert hashlib.sha256(raw).hexdigest()=='a62f99db8edfaf6aad0fafe78655927a2a237901508b2802aa7ba28088c60985','frozen integration baseline changed'
    return json.loads(raw)



def client_vault_integration_baseline_manifest(root=ROOT):
    raw=(root/'dev-docs/evidence/client-vault-integration-baseline-manifest.json').read_bytes()
    assert hashlib.sha256(raw).hexdigest()=='c79f1dff0ac082f3c124e95fa5e353c615508c3df04512caa806aeefc0ca9ae0','frozen client-vault integration baseline changed'
    return json.loads(raw)


def gatherings_baseline_manifest(root=ROOT):
    raw=(root/'dev-docs/evidence/gatherings-baseline-manifest.json').read_bytes()
    assert hashlib.sha256(raw).hexdigest()=='f6ba4284904f01171bf02bef2de2edafd85bc9b12507da51d877028207ec7a90','frozen gatherings baseline changed'
    return json.loads(raw)


def polls_baseline_manifest(root=ROOT):
    raw=(root/'dev-docs/evidence/polls-baseline-manifest.json').read_bytes()
    assert hashlib.sha256(raw).hexdigest()=='5c6b649999f9f1eff8c569b886b5839ea36492789dbe72c6d56fcef80daacc62','frozen polls baseline changed'
    return json.loads(raw)


def catchup_baseline_manifest(root=ROOT):
    raw=(root/'dev-docs/evidence/catchup-baseline-manifest.json').read_bytes()
    assert hashlib.sha256(raw).hexdigest()=='af16a8c2b063ac1e3dab53a1ff480f3278bdd7e4b4213486b24bf7b394f51f0d','frozen catch-up baseline changed'
    return json.loads(raw)


def discussion_baseline_manifest(root=ROOT):
    raw=(root/'dev-docs/evidence/discussion-baseline-manifest.json').read_bytes()
    assert hashlib.sha256(raw).hexdigest()=='dcfd2d2df0bf4fbea5bce05c9c8625fb0da9751eae20d01971a9e49ffc2f8191','frozen discussion baseline changed'
    return json.loads(raw)


def gallery_baseline_manifest(root=ROOT):
    raw=(root/'dev-docs/evidence/gallery-baseline-manifest.json').read_bytes()
    assert hashlib.sha256(raw).hexdigest()=='f4ed962ba52372dc0938db73a015b5a208a2e5813a20fd698a105d5383794452','frozen gallery baseline changed'
    return json.loads(raw)


def travel_baseline_manifest(root=ROOT):
    raw=(root/'dev-docs/evidence/travel-baseline-manifest.json').read_bytes()
    assert hashlib.sha256(raw).hexdigest()=='d6ca296cf72f896cf7eaf883f6ef5de4eb4aef304684fd5dd14dd0f9dd7e6015','frozen travel baseline changed'
    return json.loads(raw)


def library_baseline_manifest(root=ROOT):
    raw=(root/'dev-docs/evidence/library-baseline-manifest.json').read_bytes()
    assert hashlib.sha256(raw).hexdigest()=='d65f868cc5e85b7eb028bf67fb25bc2c6c0daed6ae72b60042252956eb3f960e','frozen library baseline changed'
    return json.loads(raw)


def file_requests_baseline_manifest(root=ROOT):
    raw=(root/'dev-docs/evidence/file-requests-baseline-manifest.json').read_bytes()
    assert hashlib.sha256(raw).hexdigest()=='4bccd2e93778b981be9a1ac37944f80c16fba35db1851e901c855a366f2e396b','frozen file requests baseline changed'
    return json.loads(raw)


def magazine_baseline_manifest(root=ROOT):
    raw=(root/'dev-docs/evidence/magazine-baseline-manifest.json').read_bytes()
    assert hashlib.sha256(raw).hexdigest()=='a13d3f06599f09d8975a5921e73376027ffea0bdadda2fc513ff8e13c853432f','frozen magazine baseline changed'
    return json.loads(raw)


def source_bytes(name,expected,root=ROOT):
    assert re.fullmatch('[a-f0-9]{64}',expected)
    path=Path(name);assert not path.is_absolute() and '..' not in path.parts
    m=manifest(root)
    candidates=[m]+m.get('snapshots',[])+[key_manifest(root),global_manifest(root),file_baseline_manifest(root),native_client_baseline_manifest(root),mail_client_baseline_manifest(root),room_reader_baseline_manifest(root),post_metadata_baseline_manifest(root),authored_articles_baseline_manifest(root),broader_articles_baseline_manifest(root),unicode_subjects_baseline_manifest(root),modern_terminals_baseline_manifest(root),kitty_graphics_baseline_manifest(root),terminal_compatibility_baseline_manifest(root),menu_themes_baseline_manifest(root),seasonal_themes_baseline_manifest(root),custom_menus_baseline_manifest(root),nightwire_baseline_manifest(root),operator_baseline_manifest(root),operator_host_baseline_manifest(root),operator_recovery_baseline_manifest(root),directory_baseline_manifest(root),board_launcher_baseline_manifest(root),room_browser_baseline_manifest(root),writing_desk_baseline_manifest(root),full_screen_baseline_manifest(root),community_wall_baseline_manifest(root),member_pages_baseline_manifest(root),local_mail_vault_baseline_manifest(root),client_vault_integration_baseline_manifest(root),gatherings_baseline_manifest(root),polls_baseline_manifest(root),catchup_baseline_manifest(root),discussion_baseline_manifest(root),gallery_baseline_manifest(root),travel_baseline_manifest(root),library_baseline_manifest(root),file_requests_baseline_manifest(root),magazine_baseline_manifest(root)]
    matches=[snapshot for snapshot in candidates if snapshot['files'].get(name)==expected]
    if matches:
        target=root/'dev-docs/evidence'/matches[0]['source_root']/name
    else:target=root/name
    assert target.resolve().is_relative_to(root.resolve()),'source path escaped repository'
    raw=target.read_bytes()
    assert hashlib.sha256(raw).hexdigest()==expected,'executed source bytes changed: '+name
    return raw
