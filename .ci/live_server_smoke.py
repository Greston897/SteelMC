#!/usr/bin/env python3
"""Boot the shipped server, exercise TCP status/RCON, stop and reopen its disk world.

No mocks, real players or third-party Python dependencies. This is a bounded
runtime smoke test, NOT a claim of complete vanilla conformance.
"""
import argparse
import json
from pathlib import Path
import secrets
import shutil
import socket
import struct
import subprocess
import time


def exact(stream, size):
    result = bytearray()
    while len(result) < size:
        chunk = stream.recv(size - len(result))
        if not chunk:
            raise EOFError("connection closed inside a frame")
        result.extend(chunk)
    return bytes(result)


def varint(value):
    value &= 0xffffffff
    result = bytearray()
    while True:
        byte = value & 127
        value >>= 7
        result.append(byte | (128 if value else 0))
        if not value:
            return bytes(result)


def read_varint(stream):
    result = 0
    for offset in range(5):
        byte = exact(stream, 1)[0]
        result |= (byte & 127) << (7 * offset)
        if byte < 128:
            return result
    raise ValueError("oversized VarInt")


def status(port):
    with socket.create_connection(("127.0.0.1", port), timeout=10) as stream:
        host = b"localhost"
        handshake = b"\0" + varint(-1) + varint(len(host)) + host + struct.pack(">H", port) + b"\1"
        stream.sendall(varint(len(handshake)) + handshake + b"\1\0")
        size = read_varint(stream)
        if not 1 <= size <= 2**20:
            raise ValueError("invalid status frame length")
        # Decode the frame separately so malformed sizes cannot eat the next packet.
        import io
        class Frame(io.BytesIO):
            recv = io.BytesIO.read
        frame = Frame(exact(stream, size))
        assert read_varint(frame) == 0, "wrong status packet id"
        text = exact(frame, read_varint(frame)).decode()
        assert not frame.read(), "trailing status payload"
        data = json.loads(text)
        assert data["players"]["online"] == 0, data
        assert isinstance(data["version"]["protocol"], int), data
        payload = struct.pack(">q", 731)
        stream.sendall(b"\x09\x01" + payload)
        assert read_varint(stream) == 9 and exact(stream, 9) == b"\1" + payload
        return data


class Rcon:
    def __init__(self, port, password):
        self.stream = socket.create_connection(("127.0.0.1", port), timeout=15)
        self.ident = 0
        self.transcript = []
        ident, kind, _ = self.exchange(3, password)
        if ident == -1 or kind != 2:
            self.close()
            raise PermissionError("RCON authentication rejected")

    def exchange(self, kind, body):
        self.ident += 1
        data = struct.pack("<ii", self.ident, kind) + body.encode() + b"\0\0"
        self.stream.sendall(struct.pack("<i", len(data)) + data)
        length = struct.unpack("<i", exact(self.stream, 4))[0]
        if not 10 <= length <= 2**20:
            raise ValueError("invalid RCON frame length")
        data = exact(self.stream, length)
        ident, kind = struct.unpack("<ii", data[:8])
        assert ident in (-1, self.ident), "RCON reply does not match request"
        assert data.endswith(b"\0\0"), "missing RCON terminators"
        return ident, kind, data[8:-2].decode()

    def command(self, command):
        ident, kind, body = self.exchange(2, command)
        assert ident == self.ident and kind == 0
        self.transcript.append({"command": command, "reply": body})
        return body

    def close(self):
        self.stream.close()


def probe_reply(text):
    # Scoreboard get_score emits this literal server-side response. Merely
    # echoing a command or an error mentioning 731 is not proof of execution.
    return any(line.strip() == "probe has 731 [ci]" for line in text.splitlines())


def run(binary, root, output, timeout):
    root.mkdir(parents=True, exist_ok=False)
    output.mkdir(parents=True, exist_ok=True)
    repo = Path(__file__).resolve().parent.parent
    config = root / "config"
    config.mkdir()
    password = secrets.token_hex(24)
    text = (repo / "package-content/config.toml").read_text()
    replacements = {"server_port = 25565": "server_port = 25585", "port = 25575": "port = 25595",
                    'password = ""': f'password = "{password}"', "view_distance = 10": "view_distance = 2",
                    "simulation_distance = 10": "simulation_distance = 2", "online_mode = true": "online_mode = false",
                    "use_favicon = true": "use_favicon = false"}
    for old, new in replacements.items():
        assert text.count(old) == 1, old
        text = text.replace(old, new)
    (config / "config.toml").write_text(text)
    shutil.copy(repo / "package-content/worlds.toml", config / "worlds.toml")
    results = []
    transcript = []
    process = None
    client = None

    def record(name, callback):
        start = time.monotonic()
        try:
            value = callback()
        except Exception as error:
            results.append({"name": name, "passed": False, "error": str(error)})
            raise
        results.append({"name": name, "passed": True, "seconds": round(time.monotonic() - start, 3)})
        return value

    def boot(phase):
        nonlocal process, client
        log = (output / f"server-{phase}.log").open("wb")
        process = subprocess.Popen([str(binary)], cwd=root, stdin=subprocess.DEVNULL,
                                   stdout=log, stderr=subprocess.STDOUT)
        log.close()
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            if process.poll() is not None:
                raise RuntimeError(f"server exited during startup: {process.returncode}")
            try:
                client = Rcon(25595, password)
                client.transcript = transcript
                response = client.command("list")
                if response:
                    return
                client.close()
            except (OSError, EOFError):
                pass
            time.sleep(0.5)
        raise TimeoutError("server did not become RCON-ready")

    def check(condition, wait=30):
        # A successful conditional must execute a distinct command returning the
        # pre-existing marker. Require its exact response, not an echoed input
        # or an error message containing the number.
        deadline = time.monotonic() + wait
        last = ""
        while time.monotonic() < deadline:
            last = client.command(f"execute if {condition} run scoreboard players get probe ci")
            if probe_reply(last):
                return
            time.sleep(0.2)
        raise AssertionError(f"condition not satisfied: {condition}; response={last!r}")

    def stop():
        nonlocal client
        try:
            client.command("stop")
        except (EOFError, ConnectionResetError):
            pass  # Shutdown may close the listener before the stop reply.
        client.close()
        client = None
        assert process.wait(timeout=90) == 0, "unclean server shutdown"

    try:
        record("boot_real_server", lambda: boot("first"))
        record("tcp_status_and_ping_without_players", lambda: status(25585))
        def reject_password():
            try:
                bad = Rcon(25595, password + "wrong")
            except PermissionError:
                return
            bad.close()
            raise AssertionError("RCON accepted wrong password")
        record("rcon_rejects_wrong_password", reject_password)
        client.command("scoreboard objectives add ci dummy")
        client.command("scoreboard players set probe ci 731")
        client.command("forceload add 0 0 15 15")
        record("forceload_loads_chunk_without_players", lambda: check("loaded 8 100 8", wait=120))
        client.command("fill 5 100 5 14 105 14 air")
        client.command("setblock 8 100 8 diamond_block")
        record("setblock_and_execute_read_world", lambda: check("block 8 100 8 diamond_block"))
        client.command("clone 8 100 8 8 100 8 8 101 8")
        record("clone_copies_live_block", lambda: check("block 8 101 8 diamond_block"))
        client.command("setblock 8 102 8 barrel")
        client.command('item replace block 8 102 8 container.0 with minecraft:diamond 3')
        record("item_replace_and_container_predicate", lambda: check("items block 8 102 8 container.0 minecraft:diamond"))
        client.command("setblock 11 100 8 redstone_lamp")
        client.command("setblock 12 100 8 redstone_block")
        record("redstone_neighbor_update_lights_lamp", lambda: check("block 11 100 8 redstone_lamp[lit=true]"))
        client.command("setblock 12 100 8 air")
        record("scheduled_tick_unlights_lamp", lambda: check("block 11 100 8 redstone_lamp[lit=false]"))
        client.command("setblock 10 100 11 barrel")
        client.command("setblock 10 101 11 hopper[facing=down]")
        client.command("item replace block 10 101 11 container.0 with minecraft:emerald 2")
        record("hopper_moves_items_on_server_ticks_without_players", lambda: check("items block 10 100 11 container.0 minecraft:emerald"))
        record("graceful_stop_saves_world", stop)
        record("restart_same_disk_world", lambda: boot("restart"))
        record("scoreboard_survives_restart", lambda: check("score probe ci matches 731"))
        record("chunk_blocks_survive_restart", lambda: check("block 8 101 8 diamond_block", wait=120))
        record("container_items_survive_restart", lambda: check("items block 8 102 8 container.0 minecraft:diamond"))
        record("transferred_items_survive_restart", lambda: check("items block 10 100 11 container.0 minecraft:emerald"))
        record("second_clean_shutdown", stop)
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
        report = {"scope": "real steel binary; TCP status, RCON, disk-world restart; no real players",
                  "complete_vanilla_parity": False, "results": results}
        (output / "commands.json").write_text(json.dumps(transcript, indent=2) + "\n")
        (output / "results.json").write_text(json.dumps(report, indent=2) + "\n")
        print(json.dumps(report, indent=2), flush=True)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--workdir", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--startup-timeout", type=float, default=180)
    args = parser.parse_args()
    run(args.binary.resolve(), args.workdir.resolve(), args.output.resolve(), args.startup_timeout)
