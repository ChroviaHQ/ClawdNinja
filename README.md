# ClawdNinja

ClawdNinja is a desktop app for macOS and Windows that keeps Claude Desktop, Claude Science, and Claude Code on the network egress you choose. ChatGPT is also supported.

```
Application ──► Network egress (127.0.0.1:1080) ──► Destination
```

## Features

- Launches Claude Desktop, Claude Science, and ChatGPT through the egress.
- Opens Claude Code CLI in a terminal through the egress, or writes the egress into Claude Code's `settings.json`.

## Egress Variables

| Variable                                 | Value                     |
| ---------------------------------------- | ------------------------- |
| `HTTP_PROXY`, `HTTPS_PROXY`, `ALL_PROXY` | `http://127.0.0.1:<port>` |
| `http_proxy`, `https_proxy`, `all_proxy` | `http://127.0.0.1:<port>` |
| `NO_PROXY`, `no_proxy`                   | `localhost,127.0.0.1,::1` |
| `NODE_USE_ENV_PROXY`                     | `1`                       |

Claude Desktop and ChatGPT also receive `--proxy-server=http://127.0.0.1:<port>` and `--force-webrtc-ip-handling-policy=disable_non_proxied_udp`.

`settings.json` is located in `CLAUDE_CONFIG_DIR` when set, otherwise in `~/.claude`.

## Getting Started

1. Set the port of the network egress, a local HTTP or mixed port.
2. Select **Launch** next to an application.

## Building from Source

Building requires Rust, Node.js 22 or later, and the [Tauri prerequisites](https://tauri.app/start/prerequisites/) for your platform.

```sh
npm install
npm run tauri build -- --no-bundle
```

Use `npm run tauri dev` to run the app in development mode.

## Releasing

### Setup

1. Generate the updater signing key, then put the contents of `~/.tauri/clawdninja.key.pub` into `plugins.updater.pubkey` in [`src-tauri/tauri.conf.json`](src-tauri/tauri.conf.json):

   ```sh
   npx tauri signer generate -w ~/.tauri/clawdninja.key
   ```

2. Add these secrets under **Settings** → **Secrets and variables** → **Actions**:

   | Secret                               | Value                                                |
   | ------------------------------------ | ---------------------------------------------------- |
   | `TAURI_SIGNING_PRIVATE_KEY`          | Contents of `~/.tauri/clawdninja.key`                |
   | `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | Key password, if any                                 |
   | `APPLE_CERTIFICATE`                  | Developer ID Application `.p12`, Base64-encoded      |
   | `APPLE_CERTIFICATE_PASSWORD`         | `.p12` export password                               |
   | `APPLE_SIGNING_IDENTITY`             | Output of `security find-identity -v -p codesigning` |
   | `APPLE_ID`                           | Apple ID email                                       |
   | `APPLE_PASSWORD`                     | App-specific password from appleid.apple.com         |
   | `APPLE_TEAM_ID`                      | Apple Developer Team ID                              |

### Publishing

```sh
npm run release -- 0.2.0
```

This bumps the version, tags `v0.2.0`, and pushes. GitHub Actions then builds and signs both platforms into a draft release. Publish the draft, and installed copies will offer the update.

## License

ClawdNinja is released under the [MIT License](LICENSE).
