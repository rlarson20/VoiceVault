╭───────────────────────╮
│ VoiceVault Test Suite │
╰───────────────────────╯
Setting up test environment...
  Test directory: /tmp/voicevault-test
  Config file: /tmp/voicevault-test/config.toml
  Vault directory: /tmp/voicevault-test/vault
✓ Test environment ready

Building VoiceVault...
✓ Build successful

Testing configuration loading...
✓ Configuration loading works

Note: Model will be downloaded on first transcription
Testing transcription: jfk.wav
  Output: [2m2026-02-20T04:53:42.616758Z[0m [32m INFO[0m [2mvv_daemon[0m[2m:[0m VoiceVault daemon star...
⚠ Expected keywords not found
  Expected: freedom, people
  Got: [2m2026-02-20T04:53:42.616758Z[0m [32m INFO[0m [2mvv_daemon[0m[2m:[0m VoiceVault daemon starting
[2m2026-02-20T04:53:42.616977Z[0m [32m INFO[0m [2mvv_daemon::app[0m[2m:[0m Starting Voi...

Synthesizing test audio (macOS)...
✓ Synthesized audio saved to /tmp/voicevault-test/synthesized.wav

Testing transcription: synthesized.wav
  Output: [2m2026-02-20T04:53:47.293519Z[0m [32m INFO[0m [2mvv_daemon[0m[2m:[0m VoiceVault daemon star...
✓ Found keywords: test



              Test Results              
┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┳━━━━━━━━┓
┃ Test                        ┃ Status ┃
┡━━━━━━━━━━━━━━━━━━━━━━━━━━━━━╇━━━━━━━━┩
│ Configuration Loading       │ ✓ PASS │
│ Transcription (JFK Sample)  │ ✗ FAIL │
│ Transcription (Synthesized) │ ✓ PASS │
└─────────────────────────────┴────────┘

Passed: 2/3

Test artifacts: /tmp/voicevault-test
Run with --cleanup to remove
