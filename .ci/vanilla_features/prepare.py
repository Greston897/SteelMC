"""Download the official jar with SHA-1/size verification; select tracked clusters."""
import hashlib
import json
from pathlib import Path
import urllib.request


def get_json(url):
    with urllib.request.urlopen(url, timeout=120) as response:
        return json.load(response)


def main():
    manifest = get_json('https://piston-meta.mojang.com/mc/game/version_manifest_v2.json')
    version = next(v for v in manifest['versions'] if v['id'] == '26.3')
    download = get_json(version['url'])['downloads']['server']
    with urllib.request.urlopen(download['url'], timeout=180) as response:
        jar = response.read()
    assert len(jar) == download['size']
    assert hashlib.sha1(jar).hexdigest() == download['sha1']
    Path('server.jar').write_bytes(jar)
    Path('source.json').write_text(json.dumps(download, indent=2))
    targets = [(-408528, -480569), (5360, -23628), (160238, 40382),
               (160239, 40379), (188867, -113097), (300247, 59310), (442900, 111599)]
    fixture = json.loads(Path('fixture.json').read_text())
    tracked = sorted((c['x'], c['z']) for c in fixture['dimensions']['minecraft:overworld']['chunks']
                     if any(abs(c['x'] - x) <= 12 and abs(c['z'] - z) <= 12 for x, z in targets))
    Path('tracked.tsv').write_text(''.join(f'{x}\t{z}\n' for x, z in tracked))
    Path('dump.tsv').write_text(''.join(f'{x}\t{z}\n' for x, z in targets))
    print(f'Official server verified; {len(tracked)} tracked chunks, {len(targets)} captures')


if __name__ == '__main__':
    main()
