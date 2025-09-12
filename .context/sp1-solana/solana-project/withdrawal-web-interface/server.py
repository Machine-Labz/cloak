#!/usr/bin/env python3
"""
Simple HTTP server for the Privacy-Preserving Pool System web interface
"""

import http.server
import socketserver
import json
import os
import urllib.parse
from pathlib import Path

class WebHandler(http.server.SimpleHTTPRequestHandler):
    def do_GET(self):
        if self.path == '/':
            self.path = '/index.html'
        return super().do_GET()
    
    def do_POST(self):
        if self.path == '/api/generate-proof':
            self.handle_generate_proof()
        elif self.path == '/api/verify-proof':
            self.handle_verify_proof()
        elif self.path == '/api/health':
            self.handle_health()
        else:
            self.send_error(404, "Not Found")
    
    def handle_generate_proof(self):
        """Handle proof generation request"""
        try:
            content_length = int(self.headers['Content-Length'])
            post_data = self.rfile.read(content_length)
            data = json.loads(post_data.decode('utf-8'))
            
            # Validate input
            required_fields = ['user_address', 'pool_id', 'user_balance', 'withdrawal_amount', 'pool_liquidity']
            for field in required_fields:
                if field not in data:
                    self.send_error(400, f"Missing required field: {field}")
                    return
            
            # Create mock proof data (in real implementation, this would call the Rust proof generator)
            proof_data = {
                "proof": "0x" + "a" * 2000,  # Mock proof
                "public_inputs": "0x" + "b" * 100,  # Mock public inputs
                "vkey_hash": "0x" + "c" * 64,  # Mock vkey hash
                "mode": "compressed",
                "user_address": data['user_address'],
                "pool_id": int(data['pool_id']),
                "user_balance": int(data['user_balance']),
                "withdrawal_amount": int(data['withdrawal_amount']),
                "pool_liquidity": int(data['pool_liquidity']),
                "timestamp": int(__import__('time').time() * 1000),
                "is_valid": True
            }
            
            self.send_response(200)
            self.send_header('Content-type', 'application/json')
            self.send_header('Access-Control-Allow-Origin', '*')
            self.end_headers()
            self.wfile.write(json.dumps({
                "success": True,
                "data": proof_data
            }).encode('utf-8'))
            
        except Exception as e:
            self.send_error(500, f"Internal server error: {str(e)}")
    
    def handle_verify_proof(self):
        """Handle proof verification request"""
        try:
            content_length = int(self.headers['Content-Length'])
            post_data = self.rfile.read(content_length)
            data = json.loads(post_data.decode('utf-8'))
            
            if 'proof_data' not in data:
                self.send_error(400, "Missing proof_data")
                return
            
            proof_data = data['proof_data']
            
            # Mock verification result (in real implementation, this would use WASM verifier)
            import random
            is_valid = random.random() > 0.1  # 90% success rate for demo
            
            self.send_response(200)
            self.send_header('Content-type', 'application/json')
            self.send_header('Access-Control-Allow-Origin', '*')
            self.end_headers()
            self.wfile.write(json.dumps({
                "success": True,
                "valid": is_valid,
                "data": proof_data
            }).encode('utf-8'))
            
        except Exception as e:
            self.send_error(500, f"Internal server error: {str(e)}")
    
    def handle_health(self):
        """Handle health check request"""
        self.send_response(200)
        self.send_header('Content-type', 'application/json')
        self.send_header('Access-Control-Allow-Origin', '*')
        self.end_headers()
        self.wfile.write(json.dumps({
            "status": "healthy",
            "timestamp": __import__('time').time(),
            "version": "1.0.0"
        }).encode('utf-8'))

def main():
    PORT = 8000
    
    # Change to the directory containing the HTML file
    os.chdir(os.path.dirname(os.path.abspath(__file__)))
    
    with socketserver.TCPServer(("", PORT), WebHandler) as httpd:
        print(f"🚀 Privacy-Preserving Pool System running on http://localhost:{PORT}")
        print(f"📊 Web Interface: http://localhost:{PORT}")
        print(f"🔍 API Health: http://localhost:{PORT}/api/health")
        print("Press Ctrl+C to stop the server")
        httpd.serve_forever()

if __name__ == "__main__":
    main()
