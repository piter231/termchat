import os
import json
import asyncio
from datetime import datetime
from fastapi import FastAPI, WebSocket, WebSocketDisconnect, Request
from fastapi.staticfiles import StaticFiles
from fastapi.responses import HTMLResponse, FileResponse
from fastapi.templating import Jinja2Templates

app = FastAPI()

# Directory setup
BASE_DIR = os.path.dirname(os.path.abspath(__file__))
STATIC_DIR = os.path.join(BASE_DIR, "static")
os.makedirs(STATIC_DIR, exist_ok=True)

# Mount static files directory
app.mount("/static", StaticFiles(directory=STATIC_DIR), name="static")
templates = Jinja2Templates(directory=BASE_DIR)

# Store connected clients and their nicknames
connected = set()
nicks = {}

class ConnectionManager:
    def __init__(self):
        self.active_connections = {}

    async def connect(self, websocket: WebSocket):
        await websocket.accept()
        client_ip = websocket.client.host
        self.active_connections[websocket] = client_ip
        connected.add(websocket)
        
        # Default nickname is the client IP
        nick = client_ip
        nicks[websocket] = nick
        
        timestamp = datetime.now().strftime("%H:%M:%S")
        join_msg = f"[{timestamp}] System: {nick} joined the chat"
        
        await self.broadcast(join_msg, exclude=[websocket])
        return nick

    def disconnect(self, websocket: WebSocket):
        if websocket in connected:
            connected.remove(websocket)
            
            client_ip = self.active_connections.get(websocket, "unknown")
            nick = nicks.get(websocket, client_ip)
            
            if websocket in nicks:
                del nicks[websocket]
            if websocket in self.active_connections:
                del self.active_connections[websocket]
                
            timestamp = datetime.now().strftime("%H:%M:%S")
            leave_msg = f"[{timestamp}] System: {nick} left the chat"
            
            asyncio.create_task(self.broadcast(leave_msg))
            print(f"Connection closed: {client_ip}")
        else:
            print("WebSocket not found in connected set")

    async def receive_message(self, websocket: WebSocket, data: str):
        client_ip = self.active_connections[websocket]
        nick = nicks.get(websocket, client_ip)
        
        timestamp = datetime.now().strftime("%H:%M:%S")
        print(f"[{timestamp}] Received from {client_ip}: {data}")

        try:
            json_data = json.loads(data)
            if "nick" in json_data and "message" in json_data:
                new_nick = json_data["nick"]
                old_nick = nick
                
                if new_nick != old_nick:
                    nicks[websocket] = new_nick
                    timestamp = datetime.now().strftime("%H:%M:%S")
                    nick_msg = f"[{timestamp}] System: {old_nick} is now known as {new_nick}"
                    await self.broadcast(nick_msg)
                    nick = new_nick
                
                msg_text = json_data["message"]
            else:
                msg_text = data
        except json.JSONDecodeError:
            msg_text = data

        formatted_lines = []
        lines = msg_text.split('\n')
        for i, line in enumerate(lines):
            if i == 0:
                formatted_lines.append(f"[{timestamp}] {nick}: {line}")
            else:
                indent = len(timestamp) + 3 + len(nick) + 2
                formatted_lines.append(f"{' ' * indent}{line}")
        
        formatted_msg = "\n".join(formatted_lines)
        await self.broadcast(formatted_msg)

    async def broadcast(self, message: str, exclude: list = None):
        if exclude is None:
            exclude = []
        
        # Create a copy to avoid modification during iteration
        connections = connected.copy()
        for connection in connections:
            if connection not in exclude:
                try:
                    await connection.send_text(message)
                except:
                    # Handle disconnected clients
                    if connection in connected:
                        connected.remove(connection)
                    if connection in self.active_connections:
                        del self.active_connections[connection]
                    if connection in nicks:
                        del nicks[connection]

manager = ConnectionManager()

@app.get("/", response_class=HTMLResponse)
async def read_root():
    return """
    <html>
        <head>
            <title>Chat Server</title>
            <style>
                body { 
                    font-family: Arial, sans-serif; 
                    margin: 40px; 
                    background: #f0f2f5;
                }
                .container {
                    max-width: 800px;
                    margin: 0 auto;
                    text-align: center;
                }
                h1 {
                    color: #2c3e50;
                }
                .card {
                    background: white;
                    border-radius: 10px;
                    padding: 30px;
                    box-shadow: 0 4px 6px rgba(0,0,0,0.1);
                    margin-top: 20px;
                }
                .btn {
                    display: inline-block;
                    background: #3498db;
                    color: white;
                    padding: 12px 24px;
                    border-radius: 5px;
                    text-decoration: none;
                    font-weight: bold;
                    margin: 10px;
                    transition: all 0.3s;
                }
                .btn:hover {
                    background: #2980b9;
                    transform: translateY(-2px);
                }
                .info {
                    margin-top: 30px;
                    text-align: left;
                    background: #e8f4fc;
                    padding: 15px;
                    border-radius: 5px;
                    border-left: 4px solid #3498db;
                }
            </style>
        </head>
        <body>
            <div class="container">
                <h1>💬 Rust Chat Server</h1>
                <div class="card">
                    <p>Welcome to the Rust-inspired chat server built with FastAPI</p>
                    <div>
                        <a href="/web" class="btn">Open Chat Client</a>
                        <a href="/docs" class="btn" style="background:#27ae60;">API Documentation</a>
                    </div>
                    <div class="info">
                        <h3>Server Information</h3>
                        <p><strong>WebSocket Endpoint:</strong> ws://{host}:9001/ws</p>
                        <p><strong>HTTP Port:</strong> 9001</p>
                        <p><strong>Web Client:</strong> Available at <a href="/web">/web</a></p>
                    </div>
                </div>
            </div>
            <script>
                // Replace host placeholder with current host
                const host = window.location.hostname;
                document.body.innerHTML = document.body.innerHTML.replace(/{host}/g, host);
            </script>
        </body>
    </html>
    """

@app.get("/web", response_class=HTMLResponse)
async def web_client(request: Request):
    # Render the chat client HTML
    return templates.TemplateResponse("chat_client.html", {"request": request})

@app.websocket("/ws")
async def websocket_endpoint(websocket: WebSocket):
    nick = await manager.connect(websocket)
    try:
        while True:
            data = await websocket.receive_text()
            await manager.receive_message(websocket, data)
    except WebSocketDisconnect:
        manager.disconnect(websocket)

# Save the chat client HTML to a file
CHAT_CLIENT_HTML = """
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>💬 Rust Chat Client</title>
    <link rel="stylesheet" href="/static/style.css">
</head>
<body>
    <div class="chat-container">
        <div class="title-bar">
            <h1>💬 Rust Chat Client</h1>
            <div class="connection-status">
                <span id="status-text">Connecting...</span>
                <div id="status-indicator" class="status-indicator connecting"></div>
            </div>
        </div>
        
        <div class="messages-container">
            <div id="messages" class="messages">
                <div class="message system">
                    <div class="text-content">Welcome to Rust Chat Client!</div>
                    <div class="timestamp">System</div>
                </div>
                <div class="message system">
                    <div class="text-content">Connecting to server...</div>
                    <div class="timestamp">System</div>
                </div>
            </div>
        </div>
        
        <div class="input-area">
            <div class="nick-form">
                <label for="nick-input">Your Nickname:</label>
                <input type="text" id="nick-input" placeholder="Enter your nickname">
                <button id="set-nick-btn">Set Nick</button>
            </div>
            
            <div class="input-title">Type your message (Shift+Enter for new line):</div>
            <div class="input-row">
                <textarea id="message-input" placeholder="Type your message here"></textarea>
                <button id="send-btn" class="send-button">Send</button>
            </div>
            <div class="controls">
                <div class="shortcut-hint">
                    <span class="shortcut-key">Shift+Enter</span>
                    <span>New line</span>
                </div>
                <div class="shortcut-hint">
                    <span class="shortcut-key">↑/↓</span>
                    <span>Message history</span>
                </div>
            </div>
        </div>
    </div>

    <script>
        class ChatClient {
            constructor() {
                // Automatically detect current host and port
                const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
                const host = window.location.hostname;
                const port = window.location.port || (window.location.protocol === 'https:' ? 443 : 80);
                
                // Connect to WebSocket endpoint
                this.wsUrl = `${protocol}//${host}:${port}/ws`;
                
                console.log(`Connecting to WebSocket at: ${this.wsUrl}`);
                
                // UI elements
                this.messagesContainer = document.querySelector('.messages-container');
                this.messagesElement = document.getElementById('messages');
                this.statusText = document.getElementById('status-text');
                this.statusIndicator = document.getElementById('status-indicator');
                this.messageInput = document.getElementById('message-input');
                this.sendButton = document.getElementById('send-btn');
                this.nickInput = document.getElementById('nick-input');
                this.setNickButton = document.getElementById('set-nick-btn');
                
                // State
                this.nick = localStorage.getItem('chatNick') || '';
                this.inputHistory = [];
                this.historyIndex = 0;
                this.connectionStatus = 'connecting';
                this.scrollToBottom = true;
                this.websocket = null;
                
                this.init();
            }
            
            init() {
                // Set up nickname
                if (this.nick) {
                    this.nickInput.value = this.nick;
                    this.addSystemMessage(`Nickname set to: ${this.nick}`);
                } else {
                    this.addSystemMessage('Please set your nickname');
                }
                
                this.setupEventListeners();
                this.connectWebSocket();
            }
            
            setupEventListeners() {
                this.messageInput.addEventListener('keydown', this.handleKeyDown.bind(this));
                this.sendButton.addEventListener('click', this.sendMessage.bind(this));
                this.setNickButton.addEventListener('click', this.setNick.bind(this));
                
                this.messagesContainer.addEventListener('scroll', () => {
                    // Check if user has scrolled away from the bottom
                    const isAtBottom = this.messagesContainer.scrollTop + this.messagesContainer.clientHeight >= 
                                      this.messagesContainer.scrollHeight - 10;
                    this.scrollToBottom = isAtBottom;
                });
                
                // Allow setting nick with Enter key
                this.nickInput.addEventListener('keydown', (e) => {
                    if (e.key === 'Enter') {
                        this.setNick();
                    }
                });
            }
            
            handleKeyDown(e) {
                if (e.key === 'Enter' && !e.shiftKey) {
                    e.preventDefault();
                    this.sendMessage();
                } else if (e.key === 'ArrowUp') {
                    e.preventDefault();
                    this.navigateHistory(-1);
                } else if (e.key === 'ArrowDown') {
                    e.preventDefault();
                    this.navigateHistory(1);
                }
            }
            
            navigateHistory(direction) {
                if (this.inputHistory.length === 0) return;
                
                if (this.historyIndex === 0 && direction === -1) {
                    this.historyIndex = this.inputHistory.length;
                }
                
                this.historyIndex = (this.historyIndex + direction) % this.inputHistory.length;
                if (this.historyIndex >= this.inputHistory.length) {
                    this.historyIndex = this.inputHistory.length - 1;
                }
                
                this.messageInput.value = this.inputHistory[this.historyIndex];
            }
            
            setNick() {
                const newNick = this.nickInput.value.trim();
                if (newNick) {
                    this.nick = newNick;
                    localStorage.setItem('chatNick', newNick);
                    this.addSystemMessage(`Nickname set to: ${newNick}`);
                } else {
                    this.addSystemMessage('Nickname cannot be empty');
                }
            }
            
            connectWebSocket() {
                this.websocket = new WebSocket(this.wsUrl);
                this.connectionStatus = 'connecting';
                this.updateStatus('Connecting to server...');
                this.statusIndicator.className = 'status-indicator connecting';
                
                this.websocket.onopen = () => {
                    this.connectionStatus = 'connected';
                    this.updateStatus('Connected');
                    this.statusIndicator.className = 'status-indicator connected';
                    this.addSystemMessage('Connected to server!');
                };
                
                this.websocket.onerror = (error) => {
                    this.connectionStatus = 'disconnected';
                    this.updateStatus('Connection error');
                    this.statusIndicator.className = 'status-indicator';
                    this.addSystemMessage('Connection error. Attempting to reconnect...');
                    
                    // Attempt to reconnect after 3 seconds
                    setTimeout(() => {
                        this.connectWebSocket();
                    }, 3000);
                };
                
                this.websocket.onclose = () => {
                    this.connectionStatus = 'disconnected';
                    this.updateStatus('Disconnected');
                    this.statusIndicator.className = 'status-indicator';
                    this.addSystemMessage('Connection closed. Attempting to reconnect...');
                    
                    // Attempt to reconnect after 3 seconds
                    setTimeout(() => {
                        this.connectWebSocket();
                    }, 3000);
                };
                
                this.websocket.onmessage = (event) => {
                    try {
                        // The backend sends pre-formatted messages
                        const message = event.data;
                        this.addRawMessage(message);
                    } catch (e) {
                        this.addSystemMessage(event.data);
                    }
                };
            }
            
            updateStatus(message) {
                this.statusText.textContent = message;
            }
            
            addSystemMessage(message) {
                const messageEl = document.createElement('div');
                messageEl.className = 'message system';
                
                const textDiv = document.createElement('div');
                textDiv.className = 'text-content';
                textDiv.textContent = message;
                
                const timestampDiv = document.createElement('div');
                timestampDiv.className = 'timestamp';
                timestampDiv.textContent = 'System';
                
                messageEl.appendChild(textDiv);
                messageEl.appendChild(timestampDiv);
                this.messagesElement.appendChild(messageEl);
                this.scrollToBottomIfNeeded();
            }
            
            addRawMessage(rawMessage) {
                const messageEl = document.createElement('div');
                messageEl.className = 'message other';
                
                const textDiv = document.createElement('div');
                textDiv.className = 'text-content';
                textDiv.textContent = rawMessage;
                
                const timestampDiv = document.createElement('div');
                timestampDiv.className = 'timestamp';
                timestampDiv.textContent = this.getCurrentTime();
                
                messageEl.appendChild(textDiv);
                messageEl.appendChild(timestampDiv);
                this.messagesElement.appendChild(messageEl);
                this.scrollToBottomIfNeeded();
            }
            
            getCurrentTime() {
                const now = new Date();
                const hours = String(now.getHours()).padStart(2, '0');
                const minutes = String(now.getMinutes()).padStart(2, '0');
                return `${hours}:${minutes}`;
            }
            
            scrollToBottomIfNeeded() {
                if (this.scrollToBottom) {
                    this.messagesContainer.scrollTop = this.messagesContainer.scrollHeight;
                }
            }
            
            sendMessage() {
                const message = this.messageInput.value;
                
                if (!this.nick) {
                    this.addSystemMessage('Please set your nickname first');
                    return;
                }
                
                // Check if message is empty or just whitespace
                if (!message.trim()) {
                    return;
                }
                
                if (this.websocket && this.websocket.readyState === WebSocket.OPEN) {
                    const jsonMessage = JSON.stringify({
                        nick: this.nick,
                        message: message
                    });
                    
                    this.websocket.send(jsonMessage);
                    
                    // Add to history
                    this.inputHistory.push(message);
                    this.historyIndex = this.inputHistory.length;
                    
                    // Clear input
                    this.messageInput.value = '';
                } else {
                    this.addSystemMessage('Failed to send message: Connection not established');
                }
            }
        }
        
        // Initialize the chat client when the page loads
        document.addEventListener('DOMContentLoaded', () => {
            const chatClient = new ChatClient();
        });
    </script>
</body>
</html>
"""

# Save the CSS to a file
CHAT_CSS = """
:root {
    --bg-color: #1e1e1e;
    --text-color: #f0f0f0;
    --border-color: #5f5f5f;
    --title-bg: #2f2f2f;
    --title-color: #5f9ea0;
    --status-color: #ffff00;
    --input-title-color: #add8e6;
    --input-bg: #0a0a0a;
    --button-bg: #333;
    --button-hover: #444;
    --self-message: #1a3a2a;
    --other-message: #1a1a2a;
    --system-message: #2a1a1a;
}

* {
    box-sizing: border-box;
    margin: 0;
    padding: 0;
    font-family: 'Courier New', monospace;
}

body {
    background-color: #121212;
    color: var(--text-color);
    display: flex;
    justify-content: center;
    align-items: center;
    min-height: 100vh;
    padding: 20px;
    background-image: 
        radial-gradient(circle at 10% 20%, rgba(40, 100, 100, 0.1) 0%, transparent 20%),
        radial-gradient(circle at 90% 80%, rgba(60, 140, 140, 0.1) 0%, transparent 20%);
}

.chat-container {
    width: 100%;
    max-width: 800px;
    height: 80vh;
    display: flex;
    flex-direction: column;
    border: 1px solid var(--border-color);
    background-color: #000;
    box-shadow: 0 0 30px rgba(0, 150, 200, 0.3);
    border-radius: 5px;
    overflow: hidden;
}

.title-bar {
    background-color: var(--title-bg);
    color: var(--title-color);
    padding: 12px 20px;
    text-align: center;
    font-weight: bold;
    font-size: 1.4rem;
    border-bottom: 1px solid var(--border-color);
    position: relative;
    overflow: hidden;
}

.title-bar::before {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 2px;
    background: linear-gradient(90deg, transparent, var(--title-color), transparent);
    animation: scan 3s linear infinite;
}

.title-bar h1 {
    font-size: 1.6rem;
    position: relative;
    z-index: 1;
}

.connection-status {
    display: flex;
    align-items: center;
    gap: 10px;
    position: relative;
    z-index: 1;
}

.status-indicator {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    background-color: #ff0000;
}

.status-indicator.connected {
    background-color: #00ff00;
    box-shadow: 0 0 10px #0f0;
}

.status-indicator.connecting {
    background-color: #ffff00;
    box-shadow: 0 0 10px #ff0;
    animation: pulse 1.5s infinite;
}

.messages-container {
    flex: 1;
    overflow-y: auto;
    padding: 15px;
    background-color: #000;
    display: flex;
    flex-direction: column;
    background-image: 
        radial-gradient(circle at 10% 10%, rgba(30, 60, 60, 0.05) 0%, transparent 10%),
        radial-gradient(circle at 90% 90%, rgba(30, 60, 60, 0.05) 0%, transparent 10%);
}

.messages {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 5px;
}

.message {
    padding: 10px 15px;
    border-radius: 4px;
    word-break: break-word;
    line-height: 1.5;
    animation: fadeIn 0.3s ease-out;
    border-left: 2px solid;
    position: relative;
    overflow: hidden;
    box-shadow: 0 2px 5px rgba(0,0,0,0.3);
}

.message::before {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 1px;
    background: linear-gradient(90deg, transparent, rgba(200, 200, 200, 0.2), transparent);
}

.message.system {
    background-color: var(--system-message);
    border-left-color: #ff9900;
}

.message.self {
    background-color: var(--self-message);
    border-left-color: #00cc66;
}

.message.other {
    background-color: var(--other-message);
    border-left-color: #5f9ea0;
}

.nick {
    font-weight: bold;
    margin-right: 6px;
    display: inline-block;
    margin-bottom: 5px;
}

.self .nick {
    color: #00cc66;
}

.other .nick {
    color: #5f9ea0;
}

.text-content {
    white-space: pre-wrap;
    line-height: 1.6;
}

.timestamp {
    font-size: 0.7rem;
    color: #777;
    margin-top: 5px;
    text-align: right;
}

.input-area {
    background-color: var(--input-bg);
    padding: 15px;
    border-top: 1px solid var(--border-color);
}

.input-title {
    color: var(--input-title-color);
    margin-bottom: 8px;
    font-size: 0.9rem;
}

.input-row {
    display: flex;
    gap: 10px;
}

textarea {
    flex: 1;
    background-color: #000;
    color: var(--text-color);
    border: 1px solid var(--border-color);
    padding: 12px;
    font-family: inherit;
    font-size: 1rem;
    border-radius: 4px;
    resize: none;
    min-height: 100px;
    outline: none;
}

textarea:focus {
    border-color: #5f9ea0;
    box-shadow: 0 0 5px rgba(95, 158, 160, 0.5);
}

.nick-form {
    display: flex;
    gap: 10px;
    margin-bottom: 15px;
    align-items: center;
}

.nick-form input {
    flex: 1;
    background-color: #000;
    color: var(--text-color);
    border: 1px solid var(--border-color);
    padding: 10px;
    border-radius: 4px;
    outline: none;
}

button {
    background-color: var(--button-bg);
    color: var(--text-color);
    border: none;
    padding: 10px 20px;
    border-radius: 4px;
    cursor: pointer;
    transition: all 0.2s;
    font-weight: bold;
}

button:hover {
    background-color: var(--button-hover);
    transform: translateY(-1px);
}

button:active {
    transform: translateY(1px);
}

.send-button {
    background-color: #2a5a5a;
    align-self: flex-end;
}

.send-button:hover {
    background-color: #3a7a7a;
}

.controls {
    display: flex;
    justify-content: space-between;
    margin-top: 10px;
}

.shortcut-hint {
    color: #888;
    font-size: 0.85rem;
    display: flex;
    align-items: center;
    gap: 5px;
}

.shortcut-key {
    background-color: #333;
    padding: 2px 6px;
    border-radius: 3px;
    font-size: 0.8rem;
}

/* Scrollbar styling */
.messages-container::-webkit-scrollbar {
    width: 8px;
}

.messages-container::-webkit-scrollbar-track {
    background: #0a0a0a;
}

.messages-container::-webkit-scrollbar-thumb {
    background: #2a5a5a;
    border-radius: 4px;
}

.messages-container::-webkit-scrollbar-thumb:hover {
    background: #3a7a7a;
}

/* Animations */
@keyframes fadeIn {
    from { opacity: 0; transform: translateY(5px); }
    to { opacity: 1; transform: translateY(0); }
}

@keyframes pulse {
    0% { opacity: 0.5; }
    50% { opacity: 1; }
    100% { opacity: 0.5; }
}

@keyframes scan {
    0% { transform: translateX(-100%); }
    100% { transform: translateX(100%); }
}

/* Responsive design */
@media (max-width: 768px) {
    .chat-container {
        height: 90vh;
    }
    
    .title-bar h1 {
        font-size: 1.3rem;
    }
    
    .input-row {
        flex-direction: column;
    }
    
    .nick-form {
        flex-direction: column;
        align-items: stretch;
    }
}
"""

# Create necessary files
def create_static_files():
    # Save chat client HTML
    with open(os.path.join(BASE_DIR, "chat_client.html"), "w") as f:
        f.write(CHAT_CLIENT_HTML)
    
    # Save CSS file
    with open(os.path.join(STATIC_DIR, "style.css"), "w") as f:
        f.write(CHAT_CSS)

# Create files when app starts
create_static_files()

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=9001)