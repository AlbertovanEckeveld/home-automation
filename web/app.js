const copyBtn = document.querySelector("#copy-url");
const cameraSelect = document.querySelector("#camera-select");
const playerA = document.querySelector("#player-a");
const playerB = document.querySelector("#player-b");
const players = [playerA, playerB];
const playlistUrlEl = document.querySelector("#playlist-url");
const playLatestBtn = document.querySelector("#play-latest");
const timelineRange = document.querySelector("#timeline-range");
const timelineStamp = document.querySelector("#timeline-stamp");
const timelineStart = document.querySelector("#timeline-start");
const timelineEnd = document.querySelector("#timeline-end");

copyBtn.addEventListener("click", async () => {
  try {
    await navigator.clipboard.writeText(window.location.origin);
    copyBtn.textContent = "Copied";
    setTimeout(() => {
      copyBtn.textContent = "Copy base URL";
    }, 2000);
  } catch (err) {
    copyBtn.textContent = "Copy failed";
    setTimeout(() => {
      copyBtn.textContent = "Copy base URL";
    }, 2000);
  }
});

const cameras = (document.body.dataset.cameras || "")
  .split(",")
  .map((value) => value.trim())
  .filter(Boolean);

let activeCamera = cameras[0] || "";
let allSegments = [];
let filteredSegments = [];
let currentIndex = -1;
let activePlayerIndex = 0;
let refreshTimer = null;
let lastSegmentCount = 0;

const setActivePill = (cam) => {
  const pills = cameraSelect.querySelectorAll("button");
  pills.forEach((pill) => {
    pill.classList.toggle("active", pill.dataset.camera === cam);
  });
};

const renderCameras = () => {
  cameraSelect.innerHTML = "";
  cameras.forEach((cam) => {
    const pill = document.createElement("button");
    pill.className = "camera-pill";
    pill.textContent = cam;
    pill.dataset.camera = cam;
    pill.addEventListener("click", () => loadCamera(cam));
    cameraSelect.appendChild(pill);
  });
  if (activeCamera) {
    setActivePill(activeCamera);
  }
};

const formatTimestamp = (unix) => {
  if (!unix) {
    return "unknown";
  }
  return new Date(unix * 1000).toLocaleString();
};

const setPlayerSource = (fileUrl, { autoplay = true } = {}) => {
  if (!fileUrl) {
    players.forEach((node) => {
      node.removeAttribute("src");
      node.dataset.currentSource = "";
    });
    playlistUrlEl.textContent = "-";
    timelineStamp.textContent = "-";
    return;
  }
  const activePlayer = players[activePlayerIndex];
  const inactiveIndex = activePlayerIndex === 0 ? 1 : 0;
  const inactivePlayer = players[inactiveIndex];

  if (activePlayer.dataset.currentSource === fileUrl) {
    return;
  }

  inactivePlayer.dataset.currentSource = fileUrl;
  inactivePlayer.preload = "auto";
  playlistUrlEl.textContent = fileUrl;
  inactivePlayer.src = fileUrl;
  inactivePlayer.muted = true;
  inactivePlayer.load();

  const onReady = () => {
    inactivePlayer.removeEventListener("canplay", onReady);
    inactivePlayer.classList.add("active");
    activePlayer.classList.remove("active");
    activePlayer.pause();
    activePlayer.removeAttribute("src");
    activePlayer.dataset.currentSource = "";
    activePlayerIndex = inactiveIndex;
    if (autoplay) {
      inactivePlayer.play().catch(() => {});
    }
  };

  inactivePlayer.addEventListener("canplay", onReady);
  if (!autoplay) {
    inactivePlayer.pause();
  }
};

const setActiveSegment = (index, { autoplay = true } = {}) => {
  const segment = filteredSegments[index];
  if (!segment) {
    return;
  }
  currentIndex = index;
  timelineRange.value = String(index);
  timelineStamp.textContent = segmentToStamp(segment);
  setPlayerSource(`/recordings/${activeCamera}/${segment.file_name}`, { autoplay });
};

const segmentToStamp = (segment) => {
  if (!segment) {
    return "-";
  }
  return formatTimestamp(segment.modified_unix);
};

const applySegments = ({ autoplay = false, preserveCurrent = false } = {}) => {
  const activePlayer = players[activePlayerIndex];
  const currentSource = activePlayer.dataset.currentSource || "";
  const wasEnded = activePlayer.ended;
  const previousCount = lastSegmentCount;
  filteredSegments = [...allSegments];
  lastSegmentCount = filteredSegments.length;

  if (filteredSegments.length === 0) {
    timelineRange.min = "0";
    timelineRange.max = "0";
    timelineRange.value = "0";
    timelineStart.textContent = "-";
    timelineEnd.textContent = "-";
    timelineStamp.textContent = "-";
    currentIndex = -1;
    return;
  }

  timelineRange.min = "0";
  timelineRange.max = String(filteredSegments.length - 1);
  timelineRange.value = String(filteredSegments.length - 1);
  timelineStart.textContent = segmentToStamp(filteredSegments[0]);
  timelineEnd.textContent = segmentToStamp(filteredSegments[filteredSegments.length - 1]);

  if (preserveCurrent && currentSource) {
    const currentFile = currentSource.split("/").pop();
    const index = filteredSegments.findIndex((seg) => seg.file_name === currentFile);
    if (index >= 0) {
      currentIndex = index;
      timelineRange.value = String(index);
      timelineStamp.textContent = segmentToStamp(filteredSegments[index]);
      if (wasEnded && previousCount < filteredSegments.length && index + 1 < filteredSegments.length) {
        setActiveSegment(index + 1, { autoplay: true });
      }
      return;
    }
  }

  timelineStamp.textContent = segmentToStamp(filteredSegments[filteredSegments.length - 1]);
  currentIndex = filteredSegments.length - 1;

  if (autoplay && filteredSegments.length > 0) {
    setActiveSegment(filteredSegments.length - 1, { autoplay: true });
  }
};

const enforceContinuousPlayback = (node) => {
  node.addEventListener("pause", () => {
    if (node.ended) {
      return;
    }
    node.play().catch(() => {});
  });
  node.addEventListener("stalled", () => {
    node.play().catch(() => {});
  });
  node.addEventListener("waiting", () => {
    node.play().catch(() => {});
  });
  node.addEventListener("suspend", () => {
    node.play().catch(() => {});
  });
  node.addEventListener("error", () => {
    node.play().catch(() => {});
  });
};

const loadSegments = async (cam, { preserveCurrent = false } = {}) => {
  try {
    const response = await fetch(`/recordings/${cam}/metadata.json`, { cache: "no-store" });
    if (!response.ok) {
      throw new Error("metadata not found");
    }
    const data = await response.json();
    allSegments = data.segments || [];
    applySegments({ autoplay: !preserveCurrent, preserveCurrent });
  } catch (err) {
    allSegments = [];
    filteredSegments = [];
    applySegments({ autoplay: false });
  }
};

const startAutoRefresh = () => {
  if (refreshTimer) {
    window.clearInterval(refreshTimer);
  }
  refreshTimer = window.setInterval(() => {
    if (!activeCamera) {
      return;
    }
    loadSegments(activeCamera, { preserveCurrent: true });
  }, 5000);
};

const loadCamera = (cam) => {
  activeCamera = cam;
  setActivePill(cam);
  setPlayerSource("");

  loadSegments(cam);
  startAutoRefresh();
};

playLatestBtn.addEventListener("click", () => {
  if (filteredSegments.length === 0) {
    return;
  }
  setActiveSegment(filteredSegments.length - 1, { autoplay: true });
});

timelineRange.addEventListener("input", () => {
  const index = Number(timelineRange.value);
  const segment = filteredSegments[index];
  if (!segment) {
    return;
  }
  timelineStamp.textContent = segmentToStamp(segment);
});

timelineRange.addEventListener("change", () => {
  const index = Number(timelineRange.value);
  setActiveSegment(index, { autoplay: true });
});

const handleEnded = (event) => {
  const endedPlayer = event.currentTarget;
  if (players[activePlayerIndex] !== endedPlayer) {
    return;
  }
  if (currentIndex < 0) {
    return;
  }
  const nextIndex = currentIndex + 1;
  if (nextIndex >= filteredSegments.length) {
    loadSegments(activeCamera, { preserveCurrent: true });
    return;
  }
  setActiveSegment(nextIndex, { autoplay: true });
};

players.forEach((node) => node.addEventListener("ended", handleEnded));
players.forEach((node) => enforceContinuousPlayback(node));

window.setInterval(() => {
  const activePlayer = players[activePlayerIndex];
  if (!activePlayer) {
    return;
  }
  if (!activePlayer.paused && !activePlayer.ended) {
    return;
  }
  activePlayer.play().catch(() => {});
}, 1500);

document.addEventListener("visibilitychange", () => {
  if (document.visibilityState !== "visible") {
    return;
  }
  const activePlayer = players[activePlayerIndex];
  if (!activePlayer) {
    return;
  }
  if (activePlayer.paused && !activePlayer.ended) {
    activePlayer.play().catch(() => {});
  }
});

renderCameras();
if (activeCamera) {
  loadCamera(activeCamera);
}
