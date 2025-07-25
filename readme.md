# Rust TUI Chat Application

## Overview

This project is a terminal-based chat application built with Rust, featuring a rich Text-based User Interface (TUI) and real-time messaging capabilities through WebSockets. The application allows multiple users to communicate in a shared chat room with features like nickname identification, multi-line messaging, message history navigation, and real-time synchronization.

## Features

- 🚀 **Real-time messaging** with WebSocket backend
- 👤 **Nickname identification** (specified at launch)
- 🌐 **Custom backend support** (connect to any WebSocket server)
- ✍️ **Multi-line input** support (Tab+Enter for new lines)
- ⏱️ **Message history** navigation with Up/Down arrows
- 📜 **Scrollable message history** with auto-scroll to new messages
- ⏲️ **Timestamps** on all messages
- 🔄 **Auto-reconnect** with connection status indicator
- 💻 **Intuitive TUI interface** with clear section separation
- 🔒 **Nickname change notifications** when users update their identity

## Prerequisites

- Rust (latest stable version)
- Python 3.7+ (for WebSocket server)
- Websockets library for Python (`pip install websockets`)

## Installation

### 1. Clone the repository

```bash
git clone https://github.com/piter231/termchat.git
cd termchat
```

### 2. Set up the backend server

```bash
# Create and activate virtual environment (recommended)
python -m venv .venv
source .venv/bin/activate  # Linux/Mac
.\.venv\Scripts\activate   # Windows

# Install dependencies
pip install websockets

# Start the server
python server.py
```

### 3. Build and run the Rust client

```bash
# Run with nickname and default backend
cargo run -- --nick "YourNickname"

# Run with nickname and custom backend
cargo run -- --nick "YourNickname" --backend 192.168.1.100:9001
```

## Usage

### Client Options

| Option      | Description                         | Example                       |
| ----------- | ----------------------------------- | ----------------------------- |
| `--nick`    | Specify your nickname (required)    | `--nick "Alice"`              |
| `--backend` | Custom WebSocket address (optional) | `--backend myserver.com:8080` |

### Client Controls

| Key Combination   | Action                                       |
| ----------------- | -------------------------------------------- |
| Enter             | Send message                                 |
| Tab+Enter         | Create new line in message                   |
| Up/Down Arrow     | Navigate message vertically / access history |
| Left/Right Arrows | Move cursor horizontally                     |
| Home/End          | Jump to start/end of line                    |
| Backspace/Delete  | Delete characters                            |
| Esc               | Exit application                             |

### UI Layout

The interface is divided into five sections:

1. **Title Bar**: Application name and branding
2. **Message Display Area**: Chat history with nicknames and timestamps
3. **Status Bar**: Connection status information
4. **Input Title**: Instructions for message input
5. **Input Area**: Where you type messages with cursor indicator (│)

## Technical Details

### Backend Architecture

The Python WebSocket server:

- Manages client connections and disconnections
- Handles nickname registration and changes
- Broadcasts messages to all connected clients
- Adds timestamps to messages
- Handles user join/leave notifications
- Formats messages with nicknames
- Implements timeout handling for stable connections

### Frontend Architecture

The Rust TUI client:

- Uses Ratatui for terminal UI rendering
- Implements a custom text editor with cursor navigation
- Manages WebSocket communication with threads:
  - Dedicated thread for sending messages
  - Dedicated thread for receiving messages
- Maintains message history with efficient scrolling
- Features input history navigation
- Supports custom backend addresses
- Formats messages as JSON with nickname metadata

## Known Issues

- Terminal resizing during operation may cause UI glitches
- Complex Unicode characters may affect cursor positioning
- High message volume may impact performance
- Nicknames with special characters might cause formatting issues

## Future Improvements

- [ ] Private messaging between users
- [ ] Multiple chat rooms/channels
- [ ] Message persistence
- [ ] User authentication
- [ ] File sharing capabilities
- [ ] Emoji support
- [ ] Typing indicators
- [ ] Message formatting (bold, italics)
- [ ] Connection recovery on network failure

## Troubleshooting

### Common Issues

1. **Connection failures**:

   - Ensure the Python server is running before starting clients
   - Verify firewall settings allow connections on the specified port
   - Check the backend address format: `IP:PORT` or `hostname:PORT`

2. **Nickname conflicts**:

   - Server uses first-come-first-served for nicknames
   - Try a different nickname if you can't send messages

3. **UI rendering issues**:

   - Try resizing your terminal window
   - Ensure your terminal supports UTF-8 characters

4. **WebSocket errors**:
   - Check Python version (requires 3.7+)
   - Verify websockets library is installed correctly
   - Ensure no other service is using port 9001

## License

This project is licensed under the MIT License.
