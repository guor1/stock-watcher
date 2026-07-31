// Stock Watcher — front end logic

// Fixed index codes always shown
const INDEX_CODES = ["sh000001", "sz399001", "sz399006"];

let invoke, listen;
let config = { watchlist: [], hidden_from_bar: [], boss_key: "Ctrl+`" };
let pollTimer = null;

// ─── Init ─────────────────────────────────────────────────
async function init() {
  if (!window.__TAURI__ || !window.__TAURI__.core) {
    document.getElementById("indices").innerHTML =
      '<span class="error">⚠ Tauri API 未就绪</span>';
    return;
  }
  invoke = window.__TAURI__.core.invoke;

  try {
    listen = window.__TAURI__.event && window.__TAURI__.event.listen;
  } catch (e) {
    listen = null;
  }

  try {
    config = await invoke("get_config");
  } catch (e) {
    console.warn("load config failed, using defaults:", e);
  }

  render([]);
  startPolling();

  if (listen) {
    listen("config-updated", async () => {
      config = await invoke("get_config");
      poll();
    });
  }

  document.getElementById("btn-settings").addEventListener("click", () => {
    invoke("open_settings").catch(err => showError("打开设置失败: " + err));
  });
}

// ─── Polling ──────────────────────────────────────────────
function getInterval() {
  const now = new Date();
  const day = now.getDay(); // 0=Sun
  if (day === 0 || day === 6) return 60000;
  const hhmm = now.getHours() * 100 + now.getMinutes();
  const inSession =
    (hhmm >= 915 && hhmm <= 1130) || (hhmm >= 1300 && hhmm <= 1500);
  return inSession ? 3000 : 60000;
}

function startPolling() {
  poll();
  scheduleNext();
}

function scheduleNext() {
  if (pollTimer) clearTimeout(pollTimer);
  pollTimer = setTimeout(() => {
    poll();
    scheduleNext();
  }, getInterval());
}

async function poll() {
  const codes = [...INDEX_CODES, ...config.watchlist];
  try {
    const quotes = await invoke("fetch_quotes", { codes });
    render(quotes);
  } catch (e) {
    showError(e);
  }

  // Whole-market turnover — independent of the quote fetch, so a failure
  // here shouldn't blank out the watchlist.
  try {
    const market = await invoke("fetch_market_turnover");
    renderMarket(market);
  } catch (e) {
    console.warn("fetch market turnover failed:", e);
  }
}

// ─── Render ───────────────────────────────────────────────
// Convert a yuan amount to a 亿 string, e.g. 2559898300000 → "25598.98".
function toYi(yuan) {
  return (yuan / 1e8).toFixed(2);
}

function renderMarket(m) {
  const el = document.getElementById("market");
  if (!m) return;

  const change = m.turnover_change || 0;
  const cls = change > 0 ? "up" : change < 0 ? "down" : "flat";
  const sign = change > 0 ? "+" : "";

  el.innerHTML = `
    <div class="market-grid">
      <div class="mg-cell">
        <span class="mg-label">今日成交额</span>
        <span class="mg-val gold">${toYi(m.turnover)}<i class="mg-unit">亿</i></span>
      </div>
      <div class="mg-cell">
        <span class="mg-label">较昨日</span>
        <span class="mg-val ${cls}">${sign}${toYi(change)}<i class="mg-unit">亿</i></span>
      </div>
    </div>`;
  resizeToContent();
}

function render(quotes) {
  const listEl = document.getElementById("list");

  if (!quotes || quotes.length === 0) {
    listEl.innerHTML = '<span class="loading">加载中...</span>';
    return;
  }

  const hidden = config.hidden_from_bar || [];
  const visible = quotes.filter(q => !hidden.includes(q.code));

  const indices = visible.filter(q => INDEX_CODES.includes(q.code));
  const stocks = visible.filter(q => !INDEX_CODES.includes(q.code));

  let html = indices.map(q => rowHTML(q, true)).join("");
  if (indices.length && stocks.length) {
    html += '<div class="sep"></div>';
  }
  html += stocks.map(q => rowHTML(q, false)).join("");

  if (!html) {
    html = '<span class="loading">未选择显示项</span>';
  }

  listEl.innerHTML = html;
  resizeToContent();
}

function rowHTML(q, isIndex) {
  const pct = parseFloat(q.change_pct) || 0;
  const cls = pct > 0 ? "up" : pct < 0 ? "down" : "flat";
  const sign = pct > 0 ? "+" : "";
  return `
    <div class="row ${isIndex ? "row-index" : ""} ${cls}">
      <span class="name">${q.name}</span>
      <span class="price">${q.price}</span>
      <span class="change">${sign}${q.change}</span>
      <span class="pct">${sign}${q.change_pct}%</span>
    </div>`;
}

function resizeToContent() {
  try {
    const bar = document.getElementById("bar");
    const h = Math.ceil(bar.getBoundingClientRect().height);
    const win = window.__TAURI__.window;
    if (win && win.getCurrentWindow) {
      const cur = win.getCurrentWindow();
      // Keep the current window width; only adjust height to fit content.
      cur.innerSize().then(size => {
        const { PhysicalSize } = win;
        const factor = window.devicePixelRatio || 1;
        cur.setSize(new PhysicalSize(size.width, Math.ceil(h * factor))).catch(() => {});
      }).catch(() => {});
    }
  } catch (e) { /* best-effort */ }
}

function showError(msg) {
  const listEl = document.getElementById("list");
  listEl.innerHTML = `<span class="error">⚠ ${msg}</span>`;
}

// ─── Start ────────────────────────────────────────────────
document.addEventListener("DOMContentLoaded", init);
