#!/usr/bin/env python3
"""
NitroGen Sidecar - Tauri bundled Python script for AI game playing
Communicates with NitroGen server via ZeroMQ and game via JSON IPC
"""
import sys
import json
import os
import base64
from pathlib import Path

# Check for required packages
try:
    import zmq
    from PIL import Image
    import io
except ImportError as e:
    print(json.dumps({"type": "error", "message": f"Missing dependency: {e}. Run: pip install pyzmq pillow"}))
    sys.exit(1)

class NitrogenSidecar:
    def __init__(self, server_addr: str = "tcp://127.0.0.1:5555"):
        self.server_addr = server_addr
        self.context = None
        self.socket = None
        self.connected = False
    
    def connect(self) -> bool:
        """Connect to NitroGen server"""
        try:
            self.context = zmq.Context()
            self.socket = self.context.socket(zmq.REQ)
            self.socket.setsockopt(zmq.RCVTIMEO, 5000)  # 5s timeout
            self.socket.setsockopt(zmq.SNDTIMEO, 5000)
            self.socket.connect(self.server_addr)
            
            # Send reset to initialize session
            import pickle
            self.socket.send(pickle.dumps({"type": "reset"}))
            response = pickle.loads(self.socket.recv())
            
            if response.get("status") == "ok":
                self.connected = True
                return True
            return False
        except Exception as e:
            self.log_error(f"Connection failed: {e}")
            return False
    
    def predict(self, image_b64: str) -> dict:
        """Send image to NitroGen, get gamepad prediction"""
        if not self.connected:
            return {"error": "Not connected"}
        
        try:
            import pickle
            
            # Decode and resize image
            img_data = base64.b64decode(image_b64)
            img = Image.open(io.BytesIO(img_data)).convert("RGB").resize((256, 256))
            
            # Send to server
            self.socket.send(pickle.dumps({"type": "predict", "image": img}))
            response = pickle.loads(self.socket.recv())
            
            if response.get("status") == "ok":
                pred = response["pred"]
                # Convert numpy arrays to lists
                return {
                    "j_left": self._to_list(pred.get("j_left", [[0, 0]])[0]),
                    "j_right": self._to_list(pred.get("j_right", [[0, 0]])[0]),
                    "buttons": self._to_list(pred.get("buttons", [[0]*21])[0])
                }
            return {"error": response.get("error", "Unknown error")}
        except zmq.error.Again:
            return {"error": "Server timeout"}
        except Exception as e:
            return {"error": str(e)}
    
    def _to_list(self, arr):
        """Convert numpy array or list to plain list"""
        if hasattr(arr, "tolist"):
            return arr.tolist()
        return list(arr) if arr else []
    
    def disconnect(self):
        """Clean up ZMQ resources"""
        if self.socket:
            self.socket.close()
        if self.context:
            self.context.term()
        self.connected = False
    
    def log_error(self, msg: str):
        print(json.dumps({"type": "error", "message": msg}), flush=True)
    
    def run(self):
        """Main loop - read JSON commands from stdin, write responses to stdout"""
        print(json.dumps({"type": "ready"}), flush=True)
        
        for line in sys.stdin:
            try:
                cmd = json.loads(line.strip())
                cmd_type = cmd.get("type", "")
                
                if cmd_type == "connect":
                    addr = cmd.get("addr", self.server_addr)
                    self.server_addr = addr
                    success = self.connect()
                    print(json.dumps({
                        "type": "connected" if success else "error",
                        "message": "Connected to NitroGen" if success else "Failed to connect"
                    }), flush=True)
                
                elif cmd_type == "predict":
                    result = self.predict(cmd.get("image", ""))
                    print(json.dumps({"type": "prediction", **result}), flush=True)
                
                elif cmd_type == "disconnect":
                    self.disconnect()
                    print(json.dumps({"type": "disconnected"}), flush=True)
                
                elif cmd_type == "quit":
                    self.disconnect()
                    break
                
                elif cmd_type == "ping":
                    print(json.dumps({"type": "pong", "connected": self.connected}), flush=True)
                
                else:
                    print(json.dumps({"type": "error", "message": f"Unknown command: {cmd_type}"}), flush=True)
                    
            except json.JSONDecodeError as e:
                print(json.dumps({"type": "error", "message": f"Invalid JSON: {e}"}), flush=True)
            except Exception as e:
                print(json.dumps({"type": "error", "message": str(e)}), flush=True)

def main():
    sidecar = NitrogenSidecar()
    sidecar.run()

if __name__ == "__main__":
    main()
