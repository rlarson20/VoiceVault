let isRecording = false;
let recordingStartTime = null;

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

function toggleRecord() {
  const button = document.getElementById("record-button");
  const title = document.getElementById("main-title");
  const metadataPanel = document.getElementById("metadata-panel");

  isRecording = !isRecording;

  if (isRecording) {
    // Start recording
    recordingStartTime = new Date();
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
  } else {
    // Stop recording
    const duration = recordingStartTime
      ? Math.round((new Date() - recordingStartTime) / 1000)
      : 0;

    button.textContent = "⏺ Record";
    button.classList.remove("recording");
    title.textContent = "Ready to Record";
    title.classList.remove("recording");

    updateMetadata({
      status: "idle",
      timestamp: recordingStartTime ? recordingStartTime.toISOString() : null,
      duration: duration,
      format: "webm",
      sampleRate: 48000,
      recording: false,
    });
  }
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
