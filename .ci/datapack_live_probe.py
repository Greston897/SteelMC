#!/usr/bin/env python3
"""Compare real server datapack function/reload/enable/disable behavior via RCON.

One process per engine, no game clients. Failures and prerequisite blocks are
retained individually. Test directory packs only; this does not prove recipe,
loot, tag, overlay, ZIP, or online client synchronization parity.
"""
import argparse
import importlib.util
import json
from pathlib import Path
import secrets
import shutil
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parent.parent
spec = importlib.util.spec_from_file_location('live_smoke', ROOT / '.ci/live_server_smoke.py')
smoke = importlib.util.module_from_spec(spec)
spec.loader.exec_module(smoke)

CASE_IDS = ('startup_directory_function', 'reload_modified_function',
            'disable_stops_function', 'enable_restores_function',
            'disabled_selection_survives_restart')


def write_pack(root):
    """26.3 server pack format 121.0 from SharedConstants, not old pack_format."""
    root.mkdir(parents=True, exist_ok=False)
    (root / 'pack.mcmeta').write_text(json.dumps({'pack': {
        'description': 'Steel vanilla datapack comparison',
        'min_format': [121, 0], 'max_format': [121, 0],
    }}) + '\n', encoding='utf-8')
    function = root / 'data/arena/function/probe.mcfunction'
    function.parent.mkdir(parents=True)
    function.write_text('scoreboard players set probe ci 101\n', encoding='utf-8')
    return function


def has_score(reply, value):
    return any(line.strip() == f'probe has {value} [ci]' for line in reply.splitlines())


def wait_for_function(client, value, timeout=15):
    deadline = time.monotonic() + timeout
    last = ''
    while time.monotonic() < deadline:
        client.command('function arena:probe')
        last = client.command('scoreboard players get probe ci')
        if has_score(last, value):
            return
        time.sleep(0.2)
    raise AssertionError(f'function did not set score to {value}; last score={last!r}')


def wait_for_disabled(client, timeout=15, stable_seconds=2):
    """Prove the pre-existing probe no longer runs for a sustained interval."""
    deadline = time.monotonic() + timeout
    stable_since = None
    last = ''
    while time.monotonic() < deadline:
        client.command('scoreboard players set probe ci 0')
        client.command('function arena:probe')
        last = client.command('scoreboard players get probe ci')
        now = time.monotonic()
        if has_score(last, 0):
            stable_since = now if stable_since is None else stable_since
            if now - stable_since >= stable_seconds:
                return
        else:
            stable_since = None
        time.sleep(0.2)
    raise AssertionError(f'disabled pack function continued running; last score={last!r}')


def run(binary, vanilla_jar, root, output, timeout):
    root.mkdir(parents=True, exist_ok=False)
    output.mkdir(parents=True, exist_ok=True)
    config = root / 'config'
    config.mkdir()
    password = secrets.token_hex(24)
    text = (ROOT / 'package-content/config.toml').read_text(encoding='utf-8')
    replacements = {'server_port = 25565': 'server_port = 25585', 'port = 25575': 'port = 25595',
                    'password = ""': f'password = "{password}"', 'view_distance = 10': 'view_distance = 2',
                    'simulation_distance = 10': 'simulation_distance = 2', 'online_mode = true': 'online_mode = false',
                    'use_favicon = true': 'use_favicon = false'}
    for old, new in replacements.items():
        if text.count(old) != 1:
            raise ValueError(f'unexpected config template: {old}')
        text = text.replace(old, new)
    (config / 'config.toml').write_text(text, encoding='utf-8')
    shutil.copy(ROOT / 'package-content/worlds.toml', config / 'worlds.toml')
    if vanilla_jar:
        (root / 'eula.txt').write_text('eula=true\n', encoding='utf-8')
        (root / 'server.properties').write_text(
            f'server-port=25585\nenable-rcon=true\nrcon.port=25595\nrcon.password={password}\n'
            'online-mode=false\nview-distance=2\nsimulation-distance=2\n'
            'spawn-protection=0\npause-when-empty-seconds=-1\n', encoding='utf-8')
    pack = root / ('world/datapacks/test' if vanilla_jar else 'saves/datapacks/test')
    function = write_pack(pack)
    results = []
    transcript = []
    process = None
    client = None

    def boot(phase):
        nonlocal process, client
        with (output / f'server-{phase}.log').open('wb') as log:
            command = (['java', '-Xms512M', '-Xmx2G', '-jar', str(vanilla_jar), 'nogui']
                       if vanilla_jar else [str(binary)])
            process = subprocess.Popen(command, cwd=root, stdin=subprocess.DEVNULL,
                                       stdout=log, stderr=subprocess.STDOUT)
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            if process.poll() is not None:
                raise RuntimeError(f'server exited during startup: {process.returncode}')
            try:
                client = smoke.Rcon(25595, password)
                client.transcript = transcript
                if client.command('list'):
                    return
                client.close()
                client = None
            except (OSError, EOFError):
                pass
            time.sleep(0.5)
        raise TimeoutError('server did not become RCON-ready')

    def stop():
        nonlocal client
        try:
            client.command('stop')
        except (EOFError, ConnectionResetError):
            pass
        client.close()
        client = None
        if process.wait(timeout=90) != 0:
            raise RuntimeError('unclean server shutdown')

    def case(name, fn, required=()):
        outcomes = {row['name']: row['status'] for row in results}
        if any(outcomes.get(previous) != 'passed' for previous in required):
            results.append({'name': name, 'status': 'blocked', 'reason': 'prerequisite not passed'})
            return
        try:
            fn()
        except (AssertionError, OSError, EOFError, RuntimeError, TimeoutError) as error:
            results.append({'name': name, 'status': 'failed', 'reason': str(error)})
        else:
            results.append({'name': name, 'status': 'passed'})

    def initial():
        client.command('scoreboard objectives add ci dummy')
        client.command('scoreboard players set probe ci 0')
        wait_for_function(client, 101)

    def reload_changed():
        function.write_text('scoreboard players set probe ci 202\n', encoding='utf-8')
        client.command('scoreboard players set probe ci 0')
        client.command('reload')
        wait_for_function(client, 202)

    def disable():
        client.command('datapack disable "file/test"')
        wait_for_disabled(client)

    def enable():
        client.command('scoreboard players set probe ci 0')
        client.command('datapack enable "file/test"')
        wait_for_function(client, 202)

    def restart_disabled():
        client.command('datapack disable "file/test"')
        wait_for_disabled(client)
        stop()
        boot('restart')
        wait_for_disabled(client)

    try:
        boot('first')
        case(CASE_IDS[0], initial)
        case(CASE_IDS[1], reload_changed, (CASE_IDS[0],))
        case(CASE_IDS[2], disable, (CASE_IDS[1],))
        case(CASE_IDS[3], enable, (CASE_IDS[2],))
        case(CASE_IDS[4], restart_disabled, (CASE_IDS[3],))
        if client is not None:
            stop()
    finally:
        if client:
            client.close()
        if process and process.poll() is None:
            process.terminate()
            try:
                process.wait(timeout=20)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait(timeout=10)
        seen = {row['name'] for row in results}
        for name in CASE_IDS:
            if name not in seen:
                results.append({'name': name, 'status': 'blocked', 'reason': 'probe did not reach case'})
        report = {'schema': 1, 'engine': 'vanilla-26.3' if vanilla_jar else 'steel',
                  'scope': 'directory function packs, RCON, no connected game clients', 'results': results}
        (output / 'results.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
        (output / 'commands.json').write_text(json.dumps(transcript, indent=2) + '\n', encoding='utf-8')
        print(json.dumps(report, indent=2), flush=True)
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    engine = parser.add_mutually_exclusive_group(required=True)
    engine.add_argument('--binary', type=Path)
    engine.add_argument('--vanilla-jar', type=Path)
    parser.add_argument('--workdir', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--startup-timeout', type=float, default=180)
    args = parser.parse_args()
    report = run(args.binary.resolve() if args.binary else None,
                 args.vanilla_jar.resolve() if args.vanilla_jar else None,
                 args.workdir.resolve(), args.output.resolve(), args.startup_timeout)
    if any(row['status'] != 'passed' for row in report['results']):
        sys.exit(1)


if __name__ == '__main__':
    main()
