"""Fetch exact official 26.3 server, verifying both size and Mojang SHA-1."""
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
    if len(jar) != download['size'] or hashlib.sha1(jar).hexdigest() != download['sha1']:
        raise ValueError('Official server size/SHA-1 mismatch')
    Path('server.jar').write_bytes(jar)
    Path('source.json').write_text(json.dumps({'version': '26.3', **download}, indent=2))


if __name__ == '__main__':
    main()
