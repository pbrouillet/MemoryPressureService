/// Embedded HTML/CSS/JS template for the Settings WebView window.
/// Matches the Fluent Design style of the stats window.

pub const SETTINGS_HTML: &str = r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="color-scheme" content="dark light">
<title>MPA — Settings</title>
<style>
  :root {
    --bg: #202020;
    --surface: #2d2d2d;
    --surface-hover: #383838;
    --text: #e4e4e4;
    --text-secondary: #999;
    --accent: #60cdff;
    --accent-dim: #60cdff44;
    --border: #3a3a3a;
    --input-bg: #1a1a1a;
    --card-radius: 8px;
  }
  @media (prefers-color-scheme: light) {
    :root {
      --bg: #f3f3f3;
      --surface: #ffffff;
      --surface-hover: #f0f0f0;
      --text: #1a1a1a;
      --text-secondary: #666;
      --accent: #005fb8;
      --accent-dim: #005fb822;
      --border: #e0e0e0;
      --input-bg: #ffffff;
    }
  }
  * { margin: 0; padding: 0; box-sizing: border-box; }
  body {
    font-family: 'Segoe UI Variable', 'Segoe UI', system-ui, sans-serif;
    background: var(--bg);
    color: var(--text);
    padding: 20px 24px;
    font-size: 14px;
    line-height: 1.5;
    user-select: none;
    overflow-y: auto;
  }
  h1 {
    font-size: 20px;
    font-weight: 600;
    margin-bottom: 4px;
    letter-spacing: -0.3px;
  }
  .subtitle {
    font-size: 12px;
    color: var(--text-secondary);
    margin-bottom: 20px;
  }
  .card {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--card-radius);
    padding: 16px 20px;
    margin-bottom: 16px;
  }
  .card-title {
    font-size: 13px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--accent);
    margin-bottom: 12px;
  }
  .setting-row {
    display: grid;
    grid-template-columns: 1fr 140px 150px;
    gap: 12px;
    align-items: center;
    padding: 10px 0;
    border-bottom: 1px solid var(--border);
  }
  .setting-row:last-child { border-bottom: none; }
  .setting-label {
    font-size: 14px;
    font-weight: 500;
  }
  .setting-hint {
    font-size: 11px;
    color: var(--text-secondary);
    margin-top: 2px;
  }
  input[type="number"], select {
    font-family: inherit;
    font-size: 13px;
    padding: 7px 10px;
    border-radius: 4px;
    border: 1px solid var(--border);
    background: var(--input-bg);
    color: var(--text);
    width: 100%;
    outline: none;
    transition: border-color 0.15s;
  }
  input[type="number"]:focus, select:focus {
    border-color: var(--accent);
  }
  select {
    cursor: pointer;
    appearance: none;
    background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 12 12'%3E%3Cpath d='M3 5l3 3 3-3' fill='none' stroke='%23999' stroke-width='1.5'/%3E%3C/svg%3E");
    background-repeat: no-repeat;
    background-position: right 10px center;
    padding-right: 28px;
  }
  .btn-row {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
    margin-top: 20px;
  }
  button {
    font-family: inherit;
    font-size: 13px;
    padding: 8px 24px;
    border-radius: 4px;
    border: 1px solid var(--border);
    background: var(--surface);
    color: var(--text);
    cursor: pointer;
    transition: background 0.15s;
  }
  button:hover { background: var(--surface-hover); }
  button.primary {
    background: var(--accent);
    color: #000;
    border-color: transparent;
    font-weight: 600;
  }
  button.primary:hover { opacity: 0.9; }
  @media (prefers-color-scheme: light) {
    button.primary { color: #fff; }
  }
  .toast-msg {
    position: fixed;
    bottom: 16px;
    left: 50%;
    transform: translateX(-50%);
    background: var(--accent);
    color: #000;
    padding: 8px 20px;
    border-radius: 6px;
    font-size: 13px;
    font-weight: 600;
    opacity: 0;
    transition: opacity 0.3s;
    pointer-events: none;
  }
  .toast-msg.show { opacity: 1; }
  @media (prefers-color-scheme: light) {
    .toast-msg { color: #fff; }
  }
</style>
</head>
<body>

<h1>Settings</h1>
<p class="subtitle">Configure warning thresholds and automated actions per memory area.</p>

<div class="card">
  <div class="card-title">Warning Thresholds</div>

  <div class="setting-row">
    <div>
      <div class="setting-label">Memory Load</div>
      <div class="setting-hint">Overall physical memory usage (%)</div>
    </div>
    <input type="number" id="th-memory-load" min="1" max="100" step="1" value="85">
    <select id="act-memory-load">
      <option value="none">None</option>
      <option value="notify">Notify</option>
      <option value="purge">Purge</option>
    </select>
  </div>

  <div class="setting-row">
    <div>
      <div class="setting-label">Available Memory</div>
      <div class="setting-hint">Alert when available drops below (MB)</div>
    </div>
    <input type="number" id="th-available-memory" min="0" step="256" value="4096">
    <select id="act-available-memory">
      <option value="none">None</option>
      <option value="notify">Notify</option>
      <option value="purge">Purge</option>
    </select>
  </div>

  <div class="setting-row">
    <div>
      <div class="setting-label">Modified List</div>
      <div class="setting-hint">Dirty pages awaiting disk flush (MB)</div>
    </div>
    <input type="number" id="th-modified-list" min="0" step="128" value="1024">
    <select id="act-modified-list">
      <option value="none">None</option>
      <option value="notify">Notify</option>
      <option value="purge">Purge</option>
    </select>
  </div>

  <div class="setting-row">
    <div>
      <div class="setting-label">Standby List</div>
      <div class="setting-hint">Cached pages that can be repurposed (MB)</div>
    </div>
    <input type="number" id="th-standby-list" min="0" step="256" value="2048">
    <select id="act-standby-list">
      <option value="none">None</option>
      <option value="notify">Notify</option>
      <option value="purge">Purge</option>
    </select>
  </div>
</div>

<div class="btn-row">
  <button onclick="doCancel()">Cancel</button>
  <button class="primary" onclick="doSave()">Save</button>
</div>

<div class="toast-msg" id="toast-msg"></div>

<script>
function loadSettings(s) {
  document.getElementById('th-memory-load').value = s.memory_load.warning;
  document.getElementById('act-memory-load').value = s.memory_load.action;
  document.getElementById('th-available-memory').value = s.available_memory.warning;
  document.getElementById('act-available-memory').value = s.available_memory.action;
  document.getElementById('th-modified-list').value = s.modified_list.warning;
  document.getElementById('act-modified-list').value = s.modified_list.action;
  document.getElementById('th-standby-list').value = s.standby_list.warning;
  document.getElementById('act-standby-list').value = s.standby_list.action;
}

function collectSettings() {
  return {
    memory_load: {
      warning: parseFloat(document.getElementById('th-memory-load').value) || 85,
      action: document.getElementById('act-memory-load').value
    },
    available_memory: {
      warning: parseFloat(document.getElementById('th-available-memory').value) || 4096,
      action: document.getElementById('act-available-memory').value
    },
    modified_list: {
      warning: parseFloat(document.getElementById('th-modified-list').value) || 1024,
      action: document.getElementById('act-modified-list').value
    },
    standby_list: {
      warning: parseFloat(document.getElementById('th-standby-list').value) || 2048,
      action: document.getElementById('act-standby-list').value
    }
  };
}

function doSave() {
  const settings = collectSettings();
  window.ipc.postMessage(JSON.stringify({ cmd: 'save', settings: settings }));
}

function doCancel() {
  window.ipc.postMessage(JSON.stringify({ cmd: 'cancel' }));
}

function showToast(msg) {
  const el = document.getElementById('toast-msg');
  el.textContent = msg;
  el.classList.add('show');
  setTimeout(() => el.classList.remove('show'), 2000);
}
</script>
</body>
</html>
"##;
