#!/usr/bin/env python3
"""Compare the original and geometrically unobstructed Flint fixture on vanilla.

Diagnostic only: the original input is never replaced in the Steel Flint suite.
The server and bot are real processes; outputs and exit statuses are preserved.
"""
import argparse
import hashlib
import importlib.util
import json
import subprocess
import time
import uuid
from pathlib import Path

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('live_server_smoke', HERE / 'live_server_smoke.py')
smoke = importlib.util.module_from_spec(spec)
spec.loader.exec_module(smoke)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--server', type=Path, required=True)
    parser.add_argument('--client', type=Path, required=True)
    parser.add_argument('--fixture', type=Path, required=True)
    parser.add_argument('--workdir', type=Path, required=True)
    args = parser.parse_args()
    root = args.workdir.resolve()
    root.mkdir(parents=True, exist_ok=False)
    server, client, fixture = (p.resolve() for p in (args.server, args.client, args.fixture))
    (root / 'eula.txt').write_text('eula=true\n')
    (root / 'server.properties').write_text(
        'server-port=25585\nenable-rcon=true\nrcon.port=25595\nrcon.password=probe-only\n'
        'online-mode=false\nview-distance=3\nsimulation-distance=3\nspawn-protection=0\n'
        'pause-when-empty-seconds=-1\nlevel-type=minecraft:flat\n')
    identity = uuid.UUID(bytes=hashlib.md5(b'OfflinePlayer:flintmc_testbot').digest(), version=3)
    (root / 'ops.json').write_text(json.dumps([{'uuid': str(identity), 'name': 'flintmc_testbot',
                                                'level': 4, 'bypassesPlayerLimit': False}]))
    original = json.loads(fixture.read_text())
    adjusted = json.loads(fixture.read_text())
    teleports = [entry for entry in adjusted['timeline'] if entry.get('do') == 'tp' and entry.get('at') == 6]
    assert len(teleports) == 1 and teleports[0]['pos'] == [8.5, 0, 4.5]
    teleports[0]['pos'] = [6.5, 0, 4.5]
    for name, data in [('original', original), ('unobstructed', adjusted)]:
        (root / f'{name}.json').write_text(json.dumps(data))
    rcon = None
    with (root / 'server.log').open('w') as log:
        process = subprocess.Popen(['java', '-Xms512M', '-Xmx2G', '-jar', str(server), 'nogui'],
                                   cwd=root, stdin=subprocess.DEVNULL, stdout=log, stderr=subprocess.STDOUT)
        try:
            deadline = time.monotonic() + 240
            while time.monotonic() < deadline:
                if process.poll() is not None:
                    raise RuntimeError(f'vanilla server exited: {process.returncode}')
                try:
                    rcon = smoke.Rcon(25595, 'probe-only')
                    if rcon.command('list'):
                        break
                    rcon.close()
                except (OSError, EOFError):
                    time.sleep(0.5)
            else:
                raise TimeoutError('vanilla server did not start')
            for name in ('original', 'unobstructed'):
                print(f'=== Vanilla FlintCLI: {name} ===', flush=True)
                try:
                    result = subprocess.run([str(client), str(root / f'{name}.json'), '-s', '127.0.0.1:25585', '-v'],
                                            cwd=root, stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                                            text=True, timeout=240, check=False)
                    print(result.stdout, flush=True)
                    print(f'{name}: FlintCLI exit {result.returncode}', flush=True)
                except subprocess.TimeoutExpired as error:
                    print(f'{name}: TIMEOUT; output={error.stdout!r}', flush=True)
                # Avoid a frozen test world leaking into the next independent case.
                rcon.command('tick unfreeze')
        finally:
            if rcon is not None:
                try:
                    rcon.command('stop')
                    rcon.close()
                except (OSError, EOFError):
                    pass
            try:
                process.wait(timeout=30)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait()


if __name__ == '__main__':
    main()
