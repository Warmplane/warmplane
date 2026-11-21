"""Run a loopback inventory API and generate a matching Warmplane pilot configuration."""

import argparse
import hmac
import json
import os
from http import HTTPStatus
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import parse_qs, unquote, urlsplit

TOKEN_ENV = "WARMPLANE_INVENTORY_PILOT_TOKEN"
ITEMS = {
    "A42": {"id": "A42", "name": "Barcode scanner", "tag": "stock", "private": "fixture-only-42"},
    "B17": {"id": "B17", "name": "Packing scale", "tag": "shipping", "private": "fixture-only-17"},
}


class InventoryHandler(BaseHTTPRequestHandler):
    """Serve synthetic, read-only inventory data using an explicit bearer credential."""

    def do_GET(self):
        """Serve a single asset or a filtered list."""
        supplied = self.headers.get("Authorization", "").encode()
        expected = f"Bearer {self.server.inventory_token}".encode()
        if not hmac.compare_digest(supplied, expected):
            self.send_json(HTTPStatus.UNAUTHORIZED, {"error": "Unauthorized"})
            return

        request = urlsplit(self.path)
        if request.path == "/inventory/assets":
            tag = parse_qs(request.query).get("tag", [None])[0]
            assets = [item for item in ITEMS.values() if tag is None or item["tag"] == tag]
            self.send_json(HTTPStatus.OK, {"assets": assets})
            return

        prefix = "/inventory/assets/"
        asset = ITEMS.get(unquote(request.path[len(prefix):])) if request.path.startswith(prefix) else None
        if asset is None:
            self.send_json(HTTPStatus.NOT_FOUND, {"error": "Asset not found"})
            return
        self.send_json(HTTPStatus.OK, asset)

    def send_json(self, status, data):
        """Send a JSON response with a known body length."""
        payload = json.dumps(data).encode()
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(payload)))
        self.end_headers()
        self.wfile.write(payload)


def write_pilot(directory, address):
    """Write a bearer-enabled spec and an isolated Warmplane configuration."""
    directory.mkdir(parents=True, exist_ok=True)
    spec_path = directory / "inventory.json"
    config_path = directory / "mcp_servers.json"
    if spec_path.exists() or config_path.exists():
        raise SystemExit("Use a fresh pilot directory.")

    spec = json.loads(Path(__file__).with_name("inventory.json").read_text())
    spec["security"] = [{"inventoryBearer": []}]
    spec["components"]["securitySchemes"] = {
        "inventoryBearer": {"type": "http", "scheme": "bearer"}
    }
    spec["paths"]["/assets"]["get"]["responses"]["200"] = {
        "description": "Filtered inventory",
        "content": {"application/json": {"schema": {
            "type": "object",
            "properties": {"assets": {"type": "array", "items": {"type": "object"}}},
        }}},
    }
    config = {
        "authToken": "inventory-pilot-admin",
        "state": {"enabled": True, "dir": str(directory / "state")},
        "mcpServers": {
            "inventory": {
                "requireToolPins": True,
                "openApi": {
                    "specPath": spec_path.name,
                    "baseUrl": f"http://{address[0]}:{address[1]}/inventory",
                    "operations": ["getAsset", "listAssets"],
                    "auth": {"type": "bearer", "tokenEnv": TOKEN_ENV},
                },
            },
        },
        "policy": {"resultSanitization": {"fieldNames": ["private"]}},
    }
    for path, value in [(spec_path, spec), (config_path, config)]:
        with path.open("x") as output:
            json.dump(value, output, indent=2)
            output.write("\n")
    return config_path


def main():
    """Start the fixture and print the generated config path."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path, help="fresh directory for generated pilot config")
    parser.add_argument("--port", type=int, default=0)
    options = parser.parse_args()
    token = os.environ.get(TOKEN_ENV)
    if not token:
        parser.error(f"Set {TOKEN_ENV} to a synthetic test bearer token.")
    with ThreadingHTTPServer(("127.0.0.1", options.port), InventoryHandler) as server:
        server.inventory_token = token
        config = write_pilot(options.directory.resolve(), server.server_address)
        print(config, flush=True)
        try:
            server.serve_forever()
        except KeyboardInterrupt:
            pass


if __name__ == "__main__":
    main()
