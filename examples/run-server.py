#!/usr/bin/python3
from http.server import SimpleHTTPRequestHandler
import socketserver


class JsonHTTPRequestHandler(SimpleHTTPRequestHandler):
    def translate_path(self, path):
        result = super().translate_path(path) + ".json"
        # Windows doesn't allow `:` in file names
        return result.replace(":", "%3A")


PORT = 8000
with socketserver.TCPServer(("", PORT), JsonHTTPRequestHandler) as httpd:
    print("Service at port", PORT)
    httpd.serve_forever()
