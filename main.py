import asyncio
import websockets
from datetime import datetime

connected = set()

async def handle_connection(websocket):
    client_ip = websocket.remote_address[0]
    print(f"New connection from {client_ip}")
    timestamp = datetime.now().strftime("%H:%M:%S")
    join_msg = f"[{timestamp}] System: {client_ip} joined the chat"
    
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
            
            formatted_lines = []
            for i, line in enumerate(message.split('\n')):
                if i == 0:
                    formatted_lines.append(f"[{timestamp}] {client_ip}: {line}")
                else:
                    formatted_lines.append(f"{' ' * (len(timestamp) + 3)}  {line}")
            
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
            timestamp = datetime.now().strftime("%H:%M:%S")
            leave_msg = f"[{timestamp}] System: {client_ip} left the chat"
            for conn in connected:
                try:
                    await conn.send(leave_msg)
                except websockets.exceptions.ConnectionClosed:
                    print(f"Failed to send to disconnected client: {conn.remote_address}")
            print(f"Connection closed: {client_ip}")


async def main():
    server = await websockets.serve(
        handle_connection, 
        "localhost", 
        9001,
        ping_interval=1,
        ping_timeout=60,
        close_timeout=10
    )
    print("WebSocket server started on ws://localhost:9001")
    await asyncio.Future() 

if __name__ == "__main__":
    asyncio.run(main())