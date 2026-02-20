#!/usr/bin/env python3
# /// script
# requires-python = ">=3.10"
# dependencies = [
#     "rich>=13.0.0",
#     "requests>=2.31.0",
# ]
# ///

"""
VoiceVault Test Suite

A comprehensive test script for validating the VoiceVoice pipeline:
- Configuration loading
- Model download from HuggingFace
- Audio transcription
- Vault writing
"""

from pathlib import Path
import subprocess
import tempfile
import shutil
from dataclasses import dataclass
from rich.console import Console
from rich.progress import Progress, SpinnerColumn, TextColumn
from rich.table import Table
from rich.panel import Panel
from rich.status import Status
import sys
import time
import os

console = Console()


@dataclass
class TestConfig:
    project_root: Path
    test_dir: Path
    vault_dir: Path
    config_file: Path


class VoiceVaultTestRunner:
    def __init__(self):
        self.project_root = Path(__file__).parent.parent
        self.test_dir = Path("/tmp/voicevault-test")
        self.vault_dir = self.test_dir / "vault"
        self.config_file = self.test_dir / "config.toml"
        self.results = []

    def setup(self):
        """Create test environment"""
        console.print("[bold blue]Setting up test environment...[/]")

        self.test_dir.mkdir(parents=True, exist_ok=True)
        self.vault_dir.mkdir(parents=True, exist_ok=True)
        (self.vault_dir / "voice-notes").mkdir(exist_ok=True)

        # Create test configuration
        config_content = f"""[vault]
path = "{self.vault_dir}"
folder = "voice-notes"

[stt]
model = "tiny.en"

[trigger]
push_to_talk = true
push_key = "Enter"

[audio]
sample_rate = 16000
channels = 1
"""
        self.config_file.write_text(config_content)
        console.print(f"  Test directory: {self.test_dir}")
        console.print(f"  Config file: {self.config_file}")
        console.print(f"  Vault directory: {self.vault_dir}")
        console.print("✓ Test environment ready\n")

    def build_project(self):
        """Build VoiceVault in release mode"""
        console.print("[bold blue]Building VoiceVault...[/]")

        with Status("[bold yellow]Building in release mode...", console=console) as status:
            result = subprocess.run(
                ["cargo", "build", "--workspace", "--release"],
                cwd=self.project_root,
                capture_output=True,
                text=True
            )

            if result.returncode != 0:
                console.print("[bold red]Build failed![/]")
                console.print(result.stderr)
                return False

        console.print("✓ Build successful\n")
        return True

    def test_config_loading(self):
        """Test configuration loading"""
        console.print("[bold blue]Testing configuration loading...[/]")

        result = subprocess.run(
            ["cargo", "run", "-p", "vv-daemon", "--release", "--",
             "--config", str(self.config_file), "--help"],
            cwd=self.project_root,
            capture_output=True,
            text=True
        )

        if result.returncode == 0:
            console.print("✓ Configuration loading works\n")
            return True
        else:
            console.print("[bold red]✗ Configuration loading failed[/]")
            console.print(result.stderr)
            console.print()
            return False

    def download_sample_audio(self):
        """Download test audio samples"""
        fixtures_dir = self.project_root / "scripts" / "fixtures"
        fixtures_dir.mkdir(parents=True, exist_ok=True)

        jfk_path = fixtures_dir / "jfk.wav"

        if not jfk_path.exists():
            console.print("[bold blue]Downloading JFK sample audio...[/]")

            try:
                import requests

                with Status("[bold yellow]Downloading from HuggingFace...", console=console) as status:
                    response = requests.get(
                        "https://huggingface.co/datasets/Narsil/candle_demo/resolve/main/samples_jfk.wav",
                        timeout=60
                    )

                if response.status_code == 200:
                    jfk_path.write_bytes(response.content)
                    console.print(f"✓ Downloaded to {jfk_path}\n")
                    return jfk_path
                else:
                    console.print(f"[bold yellow]⚠ Failed to download (status: {response.status_code})[/]\n")
                    return None

            except ImportError:
                console.print("[bold yellow]⚠ requests module not available, skipping download[/]\n")
                return None
            except Exception as e:
                console.print(f"[bold yellow]⚠ Download failed: {e}[/]\n")
                return None

        return jfk_path

    def test_transcription(self, audio_file: Path, expected_keywords: list[str]):
        """Test transcription with keyword matching"""
        console.print(f"[bold blue]Testing transcription: {audio_file.name}[/]")

        # Run transcription using vv-daemon --transcribe-only
        result = subprocess.run(
            ["cargo", "run", "-p", "vv-daemon", "--release", "--",
             "--config", str(self.config_file),
             "--transcribe-only", str(audio_file)],
            cwd=self.project_root,
            capture_output=True,
            text=True,
            timeout=120,
            env={**os.environ, "RUST_LOG": "info"}
        )

        if result.returncode != 0:
            console.print(f"[bold red]✗ Transcription failed[/]")
            if result.stderr:
                console.print(result.stderr[-500:] if len(result.stderr) > 500 else result.stderr)
            console.print()
            return False

        # Extract transcription from output (it's printed to stdout)
        transcription = result.stdout.strip()

        console.print(f"  Output: {transcription[:100]}...")

        # Check for keywords
        found_keywords = [kw for kw in expected_keywords
                        if kw.lower() in transcription.lower()]

        if found_keywords:
            console.print(f"✓ Found keywords: {', '.join(found_keywords)}\n")
            return True
        else:
            console.print(f"[bold yellow]⚠ Expected keywords not found[/]")
            console.print(f"  Expected: {', '.join(expected_keywords)}")
            console.print(f"  Got: {transcription[:200]}...")
            console.print()
            return False

    def synthesize_test_audio(self):
        """Generate test audio using system synthesis"""
        import platform

        test_audio = self.test_dir / "synthesized.wav"

        if platform.system() == "Darwin":  # macOS
            console.print("[bold blue]Synthesizing test audio (macOS)...[/]")

            # Check if ffmpeg is available
            if not shutil.which("ffmpeg"):
                console.print("[bold yellow]⚠ ffmpeg not found, skipping synthesis[/]\n")
                return None

            aiff_file = self.test_dir / "test.aiff"

            with Status("[bold yellow]Synthesizing speech...", console=console) as status:
                result = subprocess.run(
                    ["say", "Hello world, this is a test", "-o", str(aiff_file)],
                    capture_output=True
                )

                if result.returncode != 0:
                    console.print("[bold yellow]⚠ 'say' command failed[/]\n")
                    return None

                result = subprocess.run(
                    ["ffmpeg", "-y", "-i", str(aiff_file),
                     "-ar", "16000", "-ac", "1", "-c:a", "pcm_s16le", str(test_audio)],
                    capture_output=True
                )

                if result.returncode != 0:
                    console.print("[bold yellow]⚠ ffmpeg conversion failed[/]\n")
                    return None

            console.print(f"✓ Synthesized audio saved to {test_audio}\n")
            return test_audio

        elif platform.system() == "Linux":
            console.print("[bold blue]Synthesizing test audio (Linux)...[/]")

            # Try espeak
            if not shutil.which("espeak"):
                console.print("[bold yellow]⚠ espeak not found, skipping synthesis[/]\n")
                return None

            with Status("[bold yellow]Synthesizing speech...", console=console) as status:
                result = subprocess.run(
                    ["espeak", "Hello world, this is a test", "--stdout"],
                    capture_output=True
                )

                if result.returncode != 0:
                    console.print("[bold yellow]⚠ espeak failed[/]\n")
                    return None

                # Write raw audio
                raw_file = self.test_dir / "test.raw"
                raw_file.write_bytes(result.stdout)

                # Convert to WAV using sox or ffmpeg
                if shutil.which("ffmpeg"):
                    result = subprocess.run(
                        ["ffmpeg", "-y", "-f", "s16le", "-ar", "16000", "-ac", "1",
                         "-i", str(raw_file), str(test_audio)],
                        capture_output=True
                    )
                    if result.returncode == 0:
                        console.print(f"✓ Synthesized audio saved to {test_audio}\n")
                        return test_audio

            console.print("[bold yellow]⚠ Audio synthesis failed[/]\n")
            return None

        console.print("[bold yellow]⚠ No speech synthesis available on this platform[/]\n")
        return None

    def run_all_tests(self):
        """Run all automated tests"""
        console.print(Panel.fit("[bold cyan]VoiceVault Test Suite[/]", border_style="cyan"))

        self.setup()

        if not self.build_project():
            return False

        results = []

        # Test 1: Configuration
        results.append(("Configuration Loading", self.test_config_loading()))

        # Test 2: Model Download and Transcription with downloaded audio
        console.print("[bold blue]Note: Model will be downloaded on first transcription[/]")

        audio_file = self.download_sample_audio()
        if audio_file:
            results.append((
                "Transcription (JFK Sample)",
                self.test_transcription(audio_file, ["freedom", "people"])
            ))

        # Test 3: Synthesized audio
        synthesized_path = self.synthesize_test_audio()
        if synthesized_path:
            results.append((
                "Transcription (Synthesized)",
                self.test_transcription(synthesized_path, ["hello", "world", "test"])
            ))

        # Print summary
        self.print_summary(results)

        return all(passed for _, passed in results if passed is not None)

    def print_summary(self, results):
        """Print test summary table"""
        table = Table(title="Test Results", show_header=True, header_style="bold magenta")
        table.add_column("Test", style="cyan")
        table.add_column("Status", justify="center")

        for name, passed in results:
            if passed is None:
                status = "[dim]⊘ SKIP[/]"
            elif passed:
                status = "[bold green]✓ PASS[/]"
            else:
                status = "[bold red]✗ FAIL[/]"
            table.add_row(name, status)

        console.print("\n")
        console.print(table)

        passed_count = sum(1 for _, p in results if p)
        skip_count = sum(1 for _, p in results if p is None)
        total = len(results)

        console.print(f"\n[bold]Passed: {passed_count}/{total}[/]")

        if skip_count > 0:
            console.print(f"[dim]Skipped: {skip_count}/{total}[/]")

        console.print(f"\n[dim]Test artifacts: {self.test_dir}[/]")
        console.print("[dim]Run with --cleanup to remove[/]")

    def cleanup(self):
        """Remove test artifacts"""
        console.print(f"[bold yellow]Removing: {self.test_dir}[/]")
        shutil.rmtree(self.test_dir, ignore_errors=True)
        console.print("✓ Cleanup complete")


def main():
    import argparse

    parser = argparse.ArgumentParser(
        description="VoiceVault Test Suite",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  %(prog)s --auto          Run all automated tests
  %(prog)s --manual        Run manual test mode
  %(prog)s --cleanup       Remove test artifacts
        """
    )
    parser.add_argument("--auto", action="store_true", help="Run all automated tests")
    parser.add_argument("--manual", action="store_true", help="Run manual test mode")
    parser.add_argument("--cleanup", action="store_true", help="Remove test artifacts")

    args = parser.parse_args()

    runner = VoiceVaultTestRunner()

    if args.cleanup:
        runner.cleanup()
    elif args.manual:
        console.print(Panel.fit("[bold cyan]VoiceVault Manual Test Mode[/]", border_style="cyan"))
        console.print(f"\nConfig: {runner.config_file}")
        console.print(f"Vault: {runner.vault_dir}\n")
        console.print("[bold]Instructions:[/]")
        console.print("1. Press Enter to start recording")
        console.print("2. Speak clearly into your microphone")
        console.print("3. Press Enter again to stop")
        console.print("4. Check vault for transcription")
        console.print("\n[dim]Press Ctrl+C to exit[/]\n")

        try:
            subprocess.run(
                ["cargo", "run", "-p", "vv-daemon", "--release", "--",
                 "--config", str(runner.config_file), "-vvv"],
                cwd=runner.project_root
            )
        except KeyboardInterrupt:
            console.print("\n[yellow]Test interrupted by user[/]")
    elif args.auto:
        try:
            success = runner.run_all_tests()
            sys.exit(0 if success else 1)
        except KeyboardInterrupt:
            console.print("\n[yellow]Test interrupted by user[/]")
            sys.exit(130)
        except Exception as e:
            console.print(f"\n[bold red]Error: {e}[/]")
            sys.exit(1)
    else:
        parser.print_help()


if __name__ == "__main__":
    main()
