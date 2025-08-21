let isRecording = false;
let recordingStartTime = null;
let mediaRecorder = null;
let audioChunks = [];

// API endpoints
const API_BASE_URL = '';
const TRANSCRIBE_ENDPOINT = '/transcribe';
const HEALTH_ENDPOINT = '/health';

function togglePanel(panelType) {
  if (panelType === "metadata") {
    const panel = document.getElementById("metadata-panel");
    const btn = panel.querySelector(".collapse-btn");

    if (panel.classList.contains("collapsed")) {
      panel.classList.remove("collapsed");
      btn.textContent = "−";
    } else {
      panel.classList.add("collapsed");
      btn.textContent = "+";
    }
  } else if (panelType === "markdown") {
    const panel = document.querySelector(".markdown-panel");
    const content = panel.querySelector(".markdown-content");
    const btn = panel.querySelector(".collapse-btn");

    if (content.style.display === "none") {
      content.style.display = "block";
      btn.textContent = "−";
      panel.style.flex = "1";
    } else {
      content.style.display = "none";
      btn.textContent = "+";
      panel.style.flex = "0 0 auto";
    }
  }
}

async function toggleRecord() {
  const button = document.getElementById("record-button");
  const title = document.getElementById("main-title");
  const metadataPanel = document.getElementById("metadata-panel");

  if (!isRecording) {
    // Start recording
    try {
      const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
      mediaRecorder = new MediaRecorder(stream);
      audioChunks = [];

      mediaRecorder.ondataavailable = (event) => {
        if (event.data.size > 0) {
          audioChunks.push(event.data);
        }
      };

      mediaRecorder.onstop = async () => {
        const audioBlob = new Blob(audioChunks, { type: 'audio/webm' });
        await uploadAudio(audioBlob);
      };

      mediaRecorder.start();
      
      recordingStartTime = new Date();
      isRecording = true;
      
      button.textContent = "⏹ Recording";
      button.classList.add("recording");
      title.textContent = "Recording...";
      title.classList.add("recording");

      // Open metadata panel when recording starts
      if (metadataPanel.classList.contains("collapsed")) {
        togglePanel("metadata");
      }

      updateMetadata({
        status: "recording",
        timestamp: recordingStartTime.toISOString(),
        duration: 0,
        format: "webm",
        sampleRate: 48000,
        recording: true,
      });

      // Update duration timer
      startDurationTimer();
    } catch (error) {
      console.error('Error accessing microphone:', error);
      showError('Failed to access microphone. Please check permissions.');
    }
  } else {
    // Stop recording
    if (mediaRecorder && mediaRecorder.state !== 'inactive') {
      mediaRecorder.stop();
      mediaRecorder.stream.getTracks().forEach(track => track.stop());
    }
    
    isRecording = false;
    
    const duration = recordingStartTime
      ? Math.round((new Date() - recordingStartTime) / 1000)
      : 0;

    button.textContent = "⏺ Record";
    button.classList.remove("recording");
    title.textContent = "Processing...";
    title.classList.remove("recording");

    updateMetadata({
      status: "processing",
      timestamp: recordingStartTime ? recordingStartTime.toISOString() : null,
      duration: duration,
      format: "webm",
      sampleRate: 48000,
      recording: false,
    });
  }
}

function startDurationTimer() {
  const updateDuration = () => {
    if (isRecording && recordingStartTime) {
      const duration = Math.round((new Date() - recordingStartTime) / 1000);
      const metadataDisplay = document.getElementById("metadata-display");
      const currentMetadata = JSON.parse(metadataDisplay.textContent || '{}');
      currentMetadata.duration = duration;
      metadataDisplay.textContent = JSON.stringify(currentMetadata, null, 2);
    }
  };
  
  const interval = setInterval(() => {
    if (!isRecording) {
      clearInterval(interval);
      return;
    }
    updateDuration();
  }, 1000);
}

async function uploadAudio(audioBlob) {
  const formData = new FormData();
  formData.append('voice_file', audioBlob, 'recording.webm');

  try {
    const response = await fetch(TRANSCRIBE_ENDPOINT, {
      method: 'POST',
      body: formData,
    });

    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }

    const data = await response.json();
    
    // Display the transcription
    displayTranscription(data.text);
    
    // Update metadata
    updateMetadata({
      status: "completed",
      timestamp: data.received_at,
      duration: Math.round((new Date() - recordingStartTime) / 1000),
      format: "webm",
      sampleRate: 48000,
      recording: false,
      transcription: data.text,
      filename: data.filename
    });

    // Update title
    document.getElementById("main-title").textContent = "Ready to Record";
    
  } catch (error) {
    console.error('Error uploading audio:', error);
    showError('Failed to transcribe audio. Please try again.');
    
    // Reset UI
    document.getElementById("main-title").textContent = "Ready to Record";
    updateMetadata({
      status: "error",
      timestamp: new Date().toISOString(),
      duration: Math.round((new Date() - recordingStartTime) / 1000),
      format: "webm",
      sampleRate: 48000,
      recording: false,
      error: error.message
    });
  }
}

function displayTranscription(text) {
  const timestamp = new Date().toISOString();
  const markdownContent = `# Voice Note - ${new Date(timestamp).toLocaleString()}

${text}

---
*Transcribed at ${timestamp}*`;
  
  renderMarkdown(markdownContent);
}

function updateMetadata(data) {
  document.getElementById("metadata-display").textContent = JSON.stringify(
    data,
    null,
    2,
  );
}

// Sample markdown rendering function
function renderMarkdown(markdownText) {
  const htmlContent = marked.parse(markdownText);
  document.getElementById("markdown-display").innerHTML = htmlContent;
}

// Error handling
function showError(message) {
  const errorDiv = document.createElement('div');
  errorDiv.className = 'error-message';
  errorDiv.style.cssText = `
    position: fixed;
    top: 20px;
    right: 20px;
    background: #ff4444;
    color: white;
    padding: 15px;
    border-radius: 5px;
    box-shadow: 0 2px 10px rgba(0,0,0,0.2);
    z-index: 1000;
    max-width: 300px;
  `;
  errorDiv.textContent = message;
  
  document.body.appendChild(errorDiv);
  
  setTimeout(() => {
    errorDiv.remove();
  }, 5000);
}

// Health check on page load
async function checkHealth() {
  try {
    const response = await fetch(HEALTH_ENDPOINT);
    if (response.ok) {
      const data = await response.json();
      console.log('Backend is healthy:', data);
    } else {
      console.warn('Backend health check failed');
      showError('Backend service unavailable. Some features may not work.');
    }
  } catch (error) {
    console.error('Health check failed:', error);
    showError('Cannot connect to backend. Please check if the server is running.');
  }
}

// Initialize on page load
document.addEventListener('DOMContentLoaded', () => {
  checkHealth();
  
  // Add keyboard shortcut for recording (spacebar)
  document.addEventListener('keydown', (e) => {
    if (e.code === 'Space' && e.target.tagName !== 'INPUT' && e.target.tagName !== 'TEXTAREA') {
      e.preventDefault();
      toggleRecord();
    }
  });
});
