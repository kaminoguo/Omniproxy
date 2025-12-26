# Omniproxy

Unified API gateway for AI model subscriptions. Route requests across Claude, Codex, and Gemini with multi-account load balancing.

## Features

- **Multi-Provider** - Claude, Codex, Gemini through one endpoint
- **Multi-Account** - Load balance across multiple subscriptions
- **OpenAI-Compatible** - Drop-in `/v1/chat/completions`
- **Auto Token Refresh** - No manual re-login
- **Single Binary** - No dependencies

## Quick Start

```bash
# Build
git clone https://github.com/kaminoguo/Omniproxy.git
cd Omniproxy && cargo build --release

# Add accounts (opens browser for OAuth)
omniproxy account add codex
omniproxy account add claude
omniproxy account add gemini

# Start server
omniproxy serve --port 8000
```

## Usage

Works like OpenRouter - specify model in each request, Omniproxy routes automatically:

```bash
# Use Codex
curl http://localhost:8000/v1/chat/completions -d '{"model": "gpt-5.1-codex-max", ...}'

# Use Claude
curl http://localhost:8000/v1/chat/completions -d '{"model": "claude-opus-4", ...}'

# Use Gemini
curl http://localhost:8000/v1/chat/completions -d '{"model": "gemini-3-pro", ...}'
```

Model is parsed from request → routed to correct provider → account auto-selected from pool.

## Example: 3 Codex + 2 Claude + 1 Gemini

```bash
# Add 3 ChatGPT Pro accounts
omniproxy account add codex --name "gpt-1"
omniproxy account add codex --name "gpt-2"
omniproxy account add codex --name "gpt-3"

# Add 2 Claude Max accounts
omniproxy account add claude --name "claude-1"
omniproxy account add claude --name "claude-2"

# Add 1 Gemini account
omniproxy account add gemini --name "gemini-1"

# Check accounts
omniproxy account list
# codex:   gpt-1, gpt-2, gpt-3
# claude:  claude-1, claude-2
# gemini:  gemini-1

# Start
omniproxy serve --port 8000
```

Now when requests come in:
- `model: "gpt-5.1-codex-max"` → round-robin across gpt-1, gpt-2, gpt-3
- `model: "claude-opus-4"` → round-robin across claude-1, claude-2
- `model: "gemini-3-pro"` → uses gemini-1

## CLI

```bash
omniproxy account add <provider>   # Add account
omniproxy account list             # List accounts
omniproxy account remove <id>      # Remove account
omniproxy models                   # List available models
omniproxy serve                    # Start server
```

## Deployment

```bash
# Local (needs browser for OAuth)
omniproxy account add codex

# Copy to server
scp -r ~/.omniproxy/ user@server:~/

# Server
omniproxy serve --host 0.0.0.0 --port 8000
```

## License

MIT
