import http.server
import socketserver

PORT = 32400

# extend http.server.SimpleHTTPRequestHandler to add json extensions
class ExtenstionRemoverHandler(http.server.SimpleHTTPRequestHandler):
    def translate_path(self, path):
        path = http.server.SimpleHTTPRequestHandler.translate_path(self, path)
        # add extensions
        path += ".json"
        print(path)
        return path

with socketserver.TCPServer(("", PORT), ExtenstionRemoverHandler) as httpd:
    print("serving at port", PORT)
    httpd.serve_forever()