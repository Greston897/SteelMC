"""Wire codec regression tests; the separate Actions job runs the REAL server."""
import importlib.util
import io
from pathlib import Path
import socket
import struct
import threading
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('live_smoke', Path(__file__).with_name('live_server_smoke.py'))
smoke = importlib.util.module_from_spec(spec)
spec.loader.exec_module(smoke)


class Bytes(io.BytesIO):
    recv = io.BytesIO.read


class WireTests(unittest.TestCase):
    def test_varint_boundaries(self):
        for value in [0, 1, 127, 128, 255, 2**21, 2**31 - 1, -1]:
            self.assertEqual(smoke.read_varint(Bytes(smoke.varint(value))), value & 0xffffffff)

    def test_malformed_and_truncated_frames_rejected(self):
        with self.assertRaises(ValueError):
            smoke.read_varint(Bytes(b'\x80' * 5))
        with self.assertRaises(EOFError):
            smoke.read_varint(Bytes(b'\x80'))
        with self.assertRaises(EOFError):
            smoke.exact(Bytes(b'abc'), 4)

    def test_rcon_fragmented_replies_and_authentication(self):
        client, server = socket.socketpair()
        client.settimeout(3)
        server.settimeout(3)
        errors = []
        def peer():
            try:
                for expected_kind, reply_kind, reply in [(3, 2, b''), (2, 0, b'marker=731')]:
                    size = struct.unpack('<i', smoke.exact(server, 4))[0]
                    data = smoke.exact(server, size)
                    ident, kind = struct.unpack('<ii', data[:8])
                    self.assertEqual(kind, expected_kind)
                    body = struct.pack('<ii', ident, reply_kind) + reply + b'\0\0'
                    for byte in struct.pack('<i', len(body)) + body:
                        server.sendall(bytes([byte]))
            except Exception as error:
                errors.append(error)
            finally:
                server.close()
        thread = threading.Thread(target=peer)
        thread.start()
        try:
            with patch.object(smoke.socket, 'create_connection', return_value=client):
                connection = smoke.Rcon(1234, 'test-only-password')
                self.assertEqual(connection.command('list'), 'marker=731')
                connection.close()
        finally:
            client.close()
            thread.join(timeout=4)
        self.assertFalse(thread.is_alive())
        if errors:
            raise errors[0]

    def test_rcon_rejects_invalid_auth(self):
        client, server = socket.socketpair()
        client.settimeout(3)
        # Rcon sends a tiny auth request, so the preloaded peer response does not
        # require a reader thread and cannot wait forever if the assertion fails.
        body = struct.pack('<ii', -1, 2) + b'\0\0'
        server.sendall(struct.pack('<i', len(body)) + body)
        try:
            with patch.object(smoke.socket, 'create_connection', return_value=client):
                with self.assertRaises(PermissionError):
                    smoke.Rcon(1234, 'wrong')
        finally:
            client.close()
            server.close()


if __name__ == '__main__':
    unittest.main()
