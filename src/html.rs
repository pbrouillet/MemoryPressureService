/// Embedded HTML/CSS/JS template for the stats WebView window.
/// Styled with Fluent Design (WinUI 3 look-alike): dark/light mode,
/// card layout, Segoe UI Variable font, color-coded page bars.

pub const STATS_HTML: &str = r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="color-scheme" content="dark light">
<title>MPA — Memory Statistics</title>
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
    --bar-zeroed: #5b9bd5;
    --bar-free: #70ad47;
    --bar-modified: #ed7d31;
    --bar-standby: #ffc000;
    --bar-bad: #ff4444;
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
  table {
    width: 100%;
    border-collapse: collapse;
  }
  th, td {
    text-align: left;
    padding: 6px 12px 6px 0;
    font-size: 13px;
    border-bottom: 1px solid var(--border);
  }
  th {
    font-weight: 600;
    color: var(--text-secondary);
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }
  tr:last-child td { border-bottom: none; }
  td.num { text-align: right; font-variant-numeric: tabular-nums; }
  .bar-cell { width: 120px; }
  .bar-bg {
    background: var(--border);
    border-radius: 3px;
    height: 8px;
    overflow: hidden;
  }
  .bar-fill {
    height: 100%;
    border-radius: 3px;
    transition: width 0.4s ease;
  }
  .bar-zeroed  .bar-fill { background: var(--bar-zeroed); }
  .bar-free    .bar-fill { background: var(--bar-free); }
  .bar-modified .bar-fill { background: var(--bar-modified); }
  .bar-standby .bar-fill { background: var(--bar-standby); }
  .bar-bad     .bar-fill { background: var(--bar-bad); }
  .mem-gauge {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-bottom: 16px;
  }
  .gauge-bar {
    flex: 1;
    background: var(--border);
    border-radius: 6px;
    height: 24px;
    overflow: hidden;
    position: relative;
  }
  .gauge-fill {
    height: 100%;
    border-radius: 6px;
    background: linear-gradient(90deg, var(--bar-free), var(--accent));
    transition: width 0.6s ease;
  }
  .gauge-fill.high { background: linear-gradient(90deg, var(--bar-modified), var(--bar-bad)); }
  .gauge-label {
    font-size: 24px;
    font-weight: 700;
    min-width: 60px;
    text-align: right;
  }
  .gauge-text {
    font-size: 12px;
    color: var(--text-secondary);
    margin-top: 2px;
  }
  .toolbar {
    display: flex;
    justify-content: flex-end;
    margin-bottom: 16px;
    gap: 8px;
  }
  button {
    font-family: inherit;
    font-size: 13px;
    padding: 6px 16px;
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
  .standby-grid {
    display: grid;
    grid-template-columns: auto 1fr auto auto;
    gap: 4px 12px;
    align-items: center;
  }
  .standby-grid .label { font-size: 13px; }
  .standby-grid .bar-bg { height: 6px; min-width: 80px; }
  .standby-grid .pages, .standby-grid .size {
    font-size: 12px;
    color: var(--text-secondary);
    text-align: right;
    font-variant-numeric: tabular-nums;
  }
  .error-msg {
    background: #4a1818;
    border: 1px solid #ff4444;
    border-radius: var(--card-radius);
    padding: 12px 16px;
    color: #ff8888;
    margin-bottom: 16px;
    display: none;
  }
  @media (prefers-color-scheme: light) {
    .error-msg { background: #fff0f0; color: #cc0000; border-color: #ffaaaa; }
    button.primary { color: #fff; }
  }
</style>
</head>
<body>

<h1>Memory Pressure Agent</h1>
<p class="subtitle" id="timestamp">Loading...</p>

<div id="error" class="error-msg"></div>

<div class="toolbar">
  <button class="primary" onclick="requestRefresh()">⟳ Refresh</button>
</div>

<!-- Memory load gauge -->
<div class="card">
  <div class="card-title">Memory Load</div>
  <div class="mem-gauge">
    <div class="gauge-bar">
      <div class="gauge-fill" id="gauge-fill" style="width: 0%"></div>
    </div>
    <div>
      <div class="gauge-label" id="gauge-pct">—</div>
      <div class="gauge-text" id="gauge-text">— / — GB</div>
    </div>
  </div>
</div>

<!-- System overview -->
<div class="card">
  <div class="card-title">System Overview</div>
  <table>
    <thead>
      <tr><th>Metric</th><th class="num">Value</th></tr>
    </thead>
    <tbody id="overview-body"></tbody>
  </table>
</div>

<!-- Page lists -->
<div class="card">
  <div class="card-title">Page Lists</div>
  <table>
    <thead>
      <tr><th>List</th><th class="num">Pages</th><th class="num">Size</th><th class="bar-cell">Distribution</th></tr>
    </thead>
    <tbody id="pages-body"></tbody>
  </table>
</div>

<!-- Standby by priority -->
<div class="card">
  <div class="card-title">Standby by Priority</div>
  <div class="standby-grid" id="standby-grid"></div>
</div>

<script>
function requestRefresh() {
  window.ipc.postMessage('refresh');
}

function fmtNum(n) {
  return n.toLocaleString('en-US');
}

function fmtMB(mb) {
  if (mb >= 1024) return (mb / 1024).toFixed(1) + ' GB';
  return mb.toFixed(1) + ' MB';
}

function updateStats(data) {
  try {
    document.getElementById('error').style.display = 'none';

    // Timestamp
    document.getElementById('timestamp').textContent =
      'Updated ' + new Date().toLocaleTimeString();

    // Memory gauge
    const pct = data.memory_load_percent;
    const gaugeFill = document.getElementById('gauge-fill');
    gaugeFill.style.width = pct + '%';
    gaugeFill.className = 'gauge-fill' + (pct > 80 ? ' high' : '');
    document.getElementById('gauge-pct').textContent = pct + '%';
    document.getElementById('gauge-text').textContent =
      fmtMB(data.total_physical_mb - data.available_physical_mb) + ' used / ' +
      fmtMB(data.total_physical_mb) + ' total';

    // Overview table
    const overview = [
      ['Total Physical', fmtMB(data.total_physical_mb)],
      ['Available Physical', fmtMB(data.available_physical_mb)],
      ['Commit (Used / Limit)', fmtMB(data.commit_total_mb) + ' / ' + fmtMB(data.commit_limit_mb)],
      ['System Cache', fmtMB(data.system_cache_mb)],
      ['Kernel Paged / Nonpaged', fmtMB(data.kernel_paged_mb) + ' / ' + fmtMB(data.kernel_nonpaged_mb)],
      ['Processes / Threads / Handles', fmtNum(data.process_count) + ' / ' + fmtNum(data.thread_count) + ' / ' + fmtNum(data.handle_count)],
    ];
    const ob = document.getElementById('overview-body');
    ob.innerHTML = overview.map(r =>
      '<tr><td>' + r[0] + '</td><td class="num">' + r[1] + '</td></tr>'
    ).join('');

    // Page lists
    const pageSize = data.page_size_bytes;
    const toMB = (p) => (p * pageSize / 1048576);
    const totalPages = data.zeroed_pages + data.free_pages + data.modified_pages + data.total_standby_pages + (data.bad_pages || 0);
    const pctOf = (p) => totalPages > 0 ? (p / totalPages * 100) : 0;

    const pages = [
      ['Zeroed', data.zeroed_pages, toMB(data.zeroed_pages), 'zeroed'],
      ['Free', data.free_pages, toMB(data.free_pages), 'free'],
      ['Modified', data.modified_pages, toMB(data.modified_pages), 'modified'],
      ['Standby', data.total_standby_pages, toMB(data.total_standby_pages), 'standby'],
    ];
    if (data.bad_pages > 0) {
      pages.push(['Bad', data.bad_pages, toMB(data.bad_pages), 'bad']);
    }
    const pb = document.getElementById('pages-body');
    pb.innerHTML = pages.map(r => {
      const w = pctOf(r[1]).toFixed(1);
      return '<tr>' +
        '<td>' + r[0] + '</td>' +
        '<td class="num">' + fmtNum(r[1]) + '</td>' +
        '<td class="num">' + fmtMB(r[2]) + '</td>' +
        '<td class="bar-cell"><div class="bar-bg bar-' + r[3] + '"><div class="bar-fill" style="width:' + w + '%"></div></div></td>' +
        '</tr>';
    }).join('');

    // Standby by priority
    const sbp = data.standby_pages_by_priority;
    const maxSb = Math.max(...sbp, 1);
    const sg = document.getElementById('standby-grid');
    sg.innerHTML = sbp.map((count, i) => {
      const label = i === 0 ? '0 (Lowest)' : i === 7 ? '7 (Highest)' : String(i);
      const w = (count / maxSb * 100).toFixed(1);
      return '<span class="label">' + label + '</span>' +
        '<div class="bar-bg bar-standby"><div class="bar-fill" style="width:' + w + '%"></div></div>' +
        '<span class="pages">' + fmtNum(count) + '</span>' +
        '<span class="size">' + fmtMB(toMB(count)) + '</span>';
    }).join('');

  } catch (e) {
    showError('Render error: ' + e.message);
  }
}

function showError(msg) {
  const el = document.getElementById('error');
  el.textContent = msg;
  el.style.display = 'block';
}

// Auto-refresh on load
requestRefresh();
</script>
</body>
</html>
"##;
