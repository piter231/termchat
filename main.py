import asyncio
import websockets
import json
from datetime import datetime

connected = set()
nicks = {}

async def handle_connection(websocket):
    client_ip = websocket.remote_address[0]
    print(f"New connection from {client_ip}")
    
    # Default nickname is the client IP
    nick = client_ip
    nicks[websocket] = nick
    
    timestamp = datetime.now().strftime("%H:%M:%S")
    join_msg = f"[{timestamp}] System: {nick} joined the chat"
    
    connected.add(websocket)
    
    for conn in connected:
        try:
            await conn.send(join_msg)
        except websockets.exceptions.ConnectionClosed:
            print(f"Failed to send to disconnected client: {conn.remote_address}")

    try:
        async for message in websocket:
            timestamp = datetime.now().strftime("%H:%M:%S")
            print(f"[{timestamp}] Received from {client_ip}: {message}")

            try:
                data = json.loads(message)
                if "nick" in data and "message" in data:
                    new_nick = data["nick"]
                    old_nick = nicks.get(websocket, client_ip)
                    
                    if new_nick != old_nick:
                        nicks[websocket] = new_nick
                        timestamp = datetime.now().strftime("%H:%M:%S")
                        nick_msg = f"[{timestamp}] System: {old_nick} is now known as {new_nick}"
                        for conn in connected:
                            try:
                                await conn.send(nick_msg)
                            except:
                                pass
                        nick = new_nick
                    
                    msg_text = data["message"]
                else:
                    msg_text = message
            except json.JSONDecodeError:
                msg_text = message

            formatted_lines = []
            lines = msg_text.split('\n')
            for i, line in enumerate(lines):
                if i == 0:
                    formatted_lines.append(f"[{timestamp}] {nick}: {line}")
                else:
                    indent = len(timestamp) + 3 + len(nick) + 2
                    formatted_lines.append(f"{' ' * indent}{line}")
            
            formatted_msg = "\n".join(formatted_lines)
            
            for conn in connected:
                try:
                    await conn.send(formatted_msg)
                except websockets.exceptions.ConnectionClosed:
                    print(f"Failed to send to disconnected client: {conn.remote_address}")
    
    except websockets.exceptions.ConnectionClosed as e:
        print(f"Connection closed unexpectedly: {e}")
    finally:
        if websocket in connected:
            connected.discard(websocket)
            if websocket in nicks:
                nick = nicks[websocket]
                del nicks[websocket]
            else:
                nick = client_ip
                
            timestamp = datetime.now().strftime("%H:%M:%S")
            leave_msg = f"[{timestamp}] System: {nick} left the chat"
            for conn in connected:
                try:
                    await conn.send(leave_msg)
                except websockets.exceptions.ConnectionClosed:
                    print(f"Failed to send to disconnected client: {conn.remote_address}")
            print(f"Connection closed: {client_ip}")


async def main():
    server = await websockets.serve(
        handle_connection, 
        "0.0.0.0", 
        9001,
        ping_interval=1,
        ping_timeout=60,
        close_timeout=10
    )
    print("WebSocket server started on ws://0.0.0.0:9001")
    await asyncio.Future() 

if __name__ == "__main__":
    asyncio.run(main())