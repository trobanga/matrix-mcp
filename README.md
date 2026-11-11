# matrix-mcp

Send-only MCP server for Matrix, with end-to-end encryption through
`matrix-sdk` (vodozemac). It has one tool, `send_message(room_id, text)`, and
sends to whitelisted room IDs only. A second tool, `list_rooms`, shows the
whitelist.

## Whitelist

`MATRIX_ROOM_WHITELIST` is a comma-separated list of `label=!roomid` or bare
`!roomid` entries. `send_message` accepts a label (case ignored) or an exact
room ID. You choose the labels, so a room rename on the server cannot change
where a message goes. The server reads the list at start; restart Claude Code
after a change.

The server is its own device on your account. Its session (access token) and
its key store live in `~/.config/matrix-mcp/`. Both are secrets.

## Install

```sh
cargo install --path ~/code/matrix-mcp --locked
```

## Set up the device (in a separate terminal, not through Claude)

```sh
matrix-mcp login --user @you:matrix.org   # asks for your password
matrix-mcp verify                         # then start verification in Element
```

In Element: *Settings → Sessions*, select the `matrix-mcp` session, choose
*Verify*, and compare the emojis. Do not use the Element access token: the
server needs its own device.

## Register with Claude Code

`--env` takes a list, so another option must come between it and the name.

```sh
claude mcp add --env 'MATRIX_ROOM_WHITELIST=self=!roomid:matrix.org,team=!other:matrix.org' \
  --scope user --transport stdio matrix -- ~/.cargo/bin/matrix-mcp serve
```

## Keep the secrets from Claude

In `~/.claude/settings.json`:

```json
{
  "permissions": {
    "deny": [
      "Read(~/.config/matrix-mcp/**)",
      "Edit(~/.config/matrix-mcp/**)",
      "Edit(~/.cargo/bin/matrix-mcp)",
      "Edit(~/.claude.json)"
    ]
  },
  "sandbox": {
    "allowUnsandboxedCommands": false,
    "filesystem": { "denyRead": ["~/.config/matrix-mcp"] }
  }
}
```

The server runs outside the sandbox, so any code it runs can read the
secrets. The `Edit` rules stop Claude from changing the installed binary or
the server command in `~/.claude.json`.

## Test

```sh
cargo test
```
