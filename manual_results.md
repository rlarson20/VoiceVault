uv run scripts/test.py --manual
╭─────────────────────────────╮
│ VoiceVault Manual Test Mode │
╰─────────────────────────────╯

Config: /tmp/voicevault-test/config.toml
Vault: /tmp/voicevault-test/vault

Instructions:

1. Press Enter to start recording
2. Speak clearly into your microphone
3. Press Enter again to stop
4. Check vault for transcription

Press Ctrl+C to exit

    Finished `release` profile [optimized] target(s) in 0.40s
     Running `target/release/vv-daemon --config /tmp/voicevault-test/config.toml -vvv`

2026-02-20T04:47:50.970149Z INFO vv_daemon: VoiceVault daemon starting
2026-02-20T04:47:50.970579Z INFO vv_daemon::app: Starting VoiceVault
2026-02-20T04:47:50.970584Z INFO vv_daemon::app: Starting daemon mode
2026-02-20T04:47:50.970784Z INFO vv_core::config: Loading configuration from: /tmp/voicevault-test/config.toml
2026-02-20T04:47:50.972554Z INFO vv_daemon::app: Configuration validated
2026-02-20T04:47:50.972558Z INFO vv_daemon::app: Vault: "/tmp/voicevault-test/vault"
2026-02-20T04:47:50.972561Z INFO vv_daemon::app: Model: tiny.en
2026-02-20T04:47:50.972563Z INFO vv_daemon::app: Using device: Cpu
2026-02-20T04:47:50.972564Z INFO vv_daemon::app: Loading Whisper model...
2026-02-20T04:47:50.972838Z INFO vv_stt::engine: Loading Whisper model: tiny.en
2026-02-20T04:47:50.974400Z INFO vv_stt::engine: Downloading model files from HuggingFace Hub...
2026-02-20T04:47:51.366424Z INFO vv_stt::engine: Model vocab_size: 51864
2026-02-20T04:47:51.368742Z INFO vv_stt::engine: Tokenizer vocab_size: 51864
2026-02-20T04:47:51.368751Z INFO vv_stt::engine: Model max_target_positions: 448
2026-02-20T04:47:51.368755Z DEBUG vv_stt::engine: Token '<|startoftranscript|>' -> id 50257
2026-02-20T04:47:51.368758Z DEBUG vv_stt::engine: Token '<|endoftext|>' -> id 50256
2026-02-20T04:47:51.368760Z DEBUG vv_stt::engine: Token '<|transcribe|>' -> id 50358
2026-02-20T04:47:51.368762Z DEBUG vv_stt::engine: Token '<|notimestamps|>' -> id 50362
2026-02-20T04:47:51.368764Z INFO vv_stt::engine: Whisper model loaded successfully
2026-02-20T04:47:51.438436Z INFO vv_capture::capture: Using audio device: MacBook Pro Microphone
2026-02-20T04:47:51.476086Z DEBUG vv_capture::capture: Default input config: SupportedStreamConfig { channels: 1, sample_rate: SampleRate(48000), buffer_size: Range { min: 15, max: 4096 }, sample_format: F32 }
2026-02-20T04:47:51.476140Z INFO vv_daemon::app: Trigger: PushToTalk (press Enter to start/stop recording)
2026-02-20T04:47:51.476143Z INFO vv_daemon::app: VoiceVault ready
Press Enter to start recording...
2026-02-20T04:47:52.432204Z INFO vv_daemon::app: Recording started
2026-02-20T04:47:52.432796Z INFO vv_capture::capture: Using audio device: MacBook Pro Microphone
2026-02-20T04:47:52.483725Z DEBUG vv_capture::capture: Default input config: SupportedStreamConfig { channels: 1, sample_rate: SampleRate(48000), buffer_size: Range { min: 15, max: 4096 }, sample_format: F32 }
Recording... Press Enter to stop2026-02-20T04:47:52.564792Z INFO vv_capture::capture: Recording started
(USER NOTE: TRUNCATED TOKEN DEBUGS WHERE SEVERAL TOKENS IN A ROW MATCHED)
2026-02-20T04:47:56.703061Z INFO vv_daemon::app: Recording stopped
2026-02-20T04:47:56.704010Z INFO vv_daemon::app: Collected 4.1s of audio (66348 samples)
2026-02-20T04:47:56.707206Z INFO vv_daemon::app: Transcribing...
2026-02-20T04:47:56.707308Z INFO vv_stt::engine: Transcribing: "/tmp/voicevault_recording.wav"
2026-02-20T04:47:57.007214Z INFO vv_stt::engine: Starting greedy decode (max 448 iterations)
2026-02-20T04:47:57.007234Z DEBUG vv_stt::engine: Added language token <|en|>
2026-02-20T04:47:57.033248Z DEBUG vv_stt::engine: Token 4: 360
2026-02-20T04:47:57.046251Z DEBUG vv_stt::engine: Token 5: 146
2026-02-20T04:47:57.058752Z DEBUG vv_stt::engine: Tokens 6-53: 131
2026-02-20T04:47:58.128190Z INFO vv_stt::engine: Decode iteration 50/448 (54 tokens so far)
2026-02-20T04:47:58.155755Z DEBUG vv_stt::engine: Tokens 54-80: 131
2026-02-20T04:47:58.951772Z DEBUG vv_stt::engine: Token 81: 201
2026-02-20T04:47:58.983202Z DEBUG vv_stt::engine: Token 82: 146
2026-02-20T04:47:59.015381Z DEBUG vv_stt::engine: Token 83-96: 131
2026-02-20T04:47:59.502281Z DEBUG vv_stt::engine: Token 97: 201
2026-02-20T04:47:59.540977Z DEBUG vv_stt::engine: Token 98: 6
2026-02-20T04:47:59.579062Z DEBUG vv_stt::engine: Token 99: 132
2026-02-20T04:47:59.615819Z DEBUG vv_stt::engine: Token 100: 131
2026-02-20T04:47:59.615833Z WARN vv_stt::engine: Generated 101 tokens without EOT, forcing exit
2026-02-20T04:47:59.615836Z INFO vv_stt::engine: Decode complete: 101 tokens generated
2026-02-20T04:47:59.615837Z DEBUG vv_stt::engine: Decoding 101 tokens to text
2026-02-20T04:47:59.615855Z DEBUG vv_stt::engine: Token 0: id='50257' text='<|startoftranscript|>'
2026-02-20T04:47:59.615866Z DEBUG vv_stt::engine: Token 1: id='50358' text='<|transcribe|>'
2026-02-20T04:47:59.615868Z DEBUG vv_stt::engine: Token 2: id='50362' text='<|notimestamps|>'
2026-02-20T04:47:59.615869Z DEBUG vv_stt::engine: Token 3: id='50258' text='<|en|>'
2026-02-20T04:47:59.615876Z DEBUG vv_stt::engine: Token 4: id='360' text='ĠD'
2026-02-20T04:47:59.615881Z DEBUG vv_stt::engine: Token 5: id='146' text='Ö'
2026-02-20T04:47:59.615886Z DEBUG vv_stt::engine: Tokens 6-19: id='131' text='Ç'
'�����������������������������������_stt::engine: Decoded text (98 chars): D���������������������������������������
'�����������������������������������_daemon::app: Transcription: D���������������������������������������
2026-02-20T04:47:59.617302Z DEBUG vv_vault::writer: Target directory ensured: /tmp/voicevault-test/vault/voice-notes
2026-02-20T04:47:59.617310Z DEBUG vv_vault::writer: Writing note to temporary file: /tmp/voicevault-test/vault/voice-notes/2026-02-19T23-47-59.md.tmp
2026-02-20T04:47:59.617594Z DEBUG vv_vault::writer: Atomically renaming to final path: /tmp/voicevault-test/vault/voice-notes/2026-02-19T23-47-59.md
2026-02-20T04:47:59.617673Z INFO vv_vault::writer: Note written successfully to: /tmp/voicevault-test/vault/voice-notes/2026-02-19T23-47-59.md
2026-02-20T04:47:59.617676Z INFO vv_daemon::app: Saved note to: /tmp/voicevault-test/vault/voice-notes/2026-02-19T23-47-59.md
Press Enter to start recording...
2026-02-20T04:48:02.983581Z INFO vv_daemon::app: Recording started
2026-02-20T04:48:02.984309Z INFO vv_capture::capture: Using audio device: MacBook Pro Microphone
2026-02-20T04:48:03.049773Z DEBUG vv_capture::capture: Default input config: SupportedStreamConfig { channels: 1, sample_rate: SampleRate(48000), buffer_size: Range { min: 15, max: 4096 }, sample_format: F32 }
Recording... Press Enter to stop2026-02-20T04:48:03.085894Z INFO vv_capture::capture: Recording started

2026-02-20T04:48:08.178763Z INFO vv_daemon::app: Recording stopped
2026-02-20T04:48:08.179375Z INFO vv_daemon::app: Collected 5.1s of audio (81567 samples)
2026-02-20T04:48:08.181618Z INFO vv_daemon::app: Transcribing...
2026-02-20T04:48:08.181712Z INFO vv_stt::engine: Transcribing: "/tmp/voicevault_recording.wav"
2026-02-20T04:48:08.460746Z INFO vv_stt::engine: Starting greedy decode (max 448 iterations)
2026-02-20T04:48:08.460763Z DEBUG vv_stt::engine: Added language token <|en|>
2026-02-20T04:48:08.484915Z DEBUG vv_stt::engine: Token 4: 132
2026-02-20T04:48:08.498401Z DEBUG vv_stt::engine: Tokens 5-53: 131
2026-02-20T04:48:09.562198Z INFO vv_stt::engine: Decode iteration 50/448 (54 tokens so far)
2026-02-20T04:48:09.589534Z DEBUG vv_stt::engine: Tokens 54-90: 131
2026-02-20T04:48:10.723930Z DEBUG vv_stt::engine: Token 91: 201
2026-02-20T04:48:10.757492Z DEBUG vv_stt::engine: Token 92: 6
2026-02-20T04:48:10.792666Z DEBUG vv_stt::engine: Token 93: 152
2026-02-20T04:48:10.827417Z DEBUG vv_stt::engine: Token 94: 146
2026-02-20T04:48:10.862240Z DEBUG vv_stt::engine: Tokens 95-100: 131
2026-02-20T04:48:11.048265Z WARN vv_stt::engine: Generated 101 tokens without EOT, forcing exit
2026-02-20T04:48:11.048268Z INFO vv_stt::engine: Decode complete: 101 tokens generated
2026-02-20T04:48:11.048270Z DEBUG vv_stt::engine: Decoding 101 tokens to text
2026-02-20T04:48:11.048273Z DEBUG vv_stt::engine: Token 0: id='50257' text='<|startoftranscript|>'
2026-02-20T04:48:11.048275Z DEBUG vv_stt::engine: Token 1: id='50358' text='<|transcribe|>'
2026-02-20T04:48:11.048277Z DEBUG vv_stt::engine: Token 2: id='50362' text='<|notimestamps|>'
2026-02-20T04:48:11.048279Z DEBUG vv_stt::engine: Token 3: id='50258' text='<|en|>'
2026-02-20T04:48:11.048281Z DEBUG vv_stt::engine: Token 4: id='132' text='È'
2026-02-20T04:48:11.052532Z DEBUG vv_stt::engine: Tokens 5-19: id='131' text='Ç'
2026-02-20T04:48:11.060906Z DEBUG vv_stt::engine: Decoded text (97 chars): ������������������������������������������'�����������������������������������������
'�����������������������������������������n::app: Transcription: ��������������������������������������������
2026-02-20T04:48:11.065032Z DEBUG vv_vault::writer: Target directory ensured: /tmp/voicevault-test/vault/voice-notes
2026-02-20T04:48:11.065040Z DEBUG vv_vault::writer: Writing note to temporary file: /tmp/voicevault-test/vault/voice-notes/2026-02-19T23-48-11.md.tmp
2026-02-20T04:48:11.065247Z DEBUG vv_vault::writer: Atomically renaming to final path: /tmp/voicevault-test/vault/voice-notes/2026-02-19T23-48-11.md
2026-02-20T04:48:11.065324Z INFO vv_vault::writer: Note written successfully to: /tmp/voicevault-test/vault/voice-notes/2026-02-19T23-48-11.md
2026-02-20T04:48:11.068958Z INFO vv_daemon::app: Saved note to: /tmp/voicevault-test/vault/voice-notes/2026-02-19T23-48-11.md
Press Enter to start recording... 2026-02-20T04:48:11.069026Z INFO vv_daemon::app: Recording started
2026-02-20T04:48:11.069149Z INFO vv_capture::capture: Using audio device: MacBook Pro Microphone
2026-02-20T04:48:11.105315Z DEBUG vv_capture::capture: Default input config: SupportedStreamConfig { channels: 1, sample_rate: SampleRate(48000), buffer_size: Range { min: 15, max: 4096 }, sample_format: F32 }
Recording... Press Enter to stop2026-02-20T04:48:11.141644Z INFO vv_capture::capture: Recording started
^C
Test interrupted by user
