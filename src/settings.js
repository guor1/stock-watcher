// Settings panel logic
let invoke;
let config = { watchlist: [], hidden_from_bar: [], boss_key: "Ctrl+`" };
let quoteMap = {}; // code -> quote
let refreshTimer = null;

const INDEX_CODES = ["sh000001", "sz399001", "sz399006"];
const INDEX_NAMES = { "sh000001": "上证指数", "sz399001": "深证成指", "sz399006": "创业板指" };

async function init() {
  try {
    invoke = window.__TAURI__.core.invoke;
  } catch (e) {
    document.getElementById("status").textContent = "⚠ Tauri API 未就绪";
    return;
  }

  try {
    config = await invoke("get_config");
  } catch (e) {
    console.warn("load config failed, using defaults:", e);
  }
  if (!config.hidden_from_bar) config.hidden_from_bar = [];

  renderList();
  refreshQuotes();
  startRefresh();

  document.getElementById("btn-add-stock").addEventListener("click", addStock);
  document.getElementById("stock-code").addEventListener("keydown", e => {
    if (e.key === "Enter") addStock();
  });
  document.getElementById("btn-close").addEventListener("click", () => {
    invoke("close_settings").catch(() => window.close());
  });
}

// ─── Quotes ───────────────────────────────────────────────
function startRefresh() {
  if (refreshTimer) clearInterval(refreshTimer);
  refreshTimer = setInterval(refreshQuotes, 3000);
}

async function refreshQuotes() {
  const codes = [...INDEX_CODES, ...config.watchlist];
  try {
    const quotes = await invoke("fetch_quotes", { codes });
    quoteMap = {};
    quotes.forEach(q => (quoteMap[q.code] = q));
    renderList();
  } catch (e) {
    console.warn("refresh quotes failed:", e);
  }
}

// ─── List ─────────────────────────────────────────────────
function renderList() {
  const el = document.getElementById("watchlist");
  let html = "";

  // Indices section (toggle only, no delete)
  html += '<div class="group-label">大盘指数</div>';
  html += INDEX_CODES.map(code => rowHTML(code, -1, true)).join("");

  // Stocks section
  html += '<div class="group-label">自选股</div>';
  if (config.watchlist.length === 0) {
    html += '<div class="list-empty">暂无自选股，在上方添加</div>';
  } else {
    html += config.watchlist.map((code, i) => rowHTML(code, i, false)).join("");
  }

  el.innerHTML = html;

  el.querySelectorAll(".btn-del").forEach(btn => {
    btn.addEventListener("click", () => removeStock(+btn.dataset.idx));
  });
  el.querySelectorAll(".toggle").forEach(cb => {
    cb.addEventListener("change", () => toggleVisible(cb.dataset.code, cb.checked));
  });
}

function rowHTML(code, idx, isIndex) {
  const q = quoteMap[code];
  const visible = !config.hidden_from_bar.includes(code);
  const name = q ? q.name : (isIndex ? INDEX_NAMES[code] || code : "加载中...");

  let price = "", change = "", pctText = "", cls = "";
  if (q) {
    const pct = parseFloat(q.change_pct) || 0;
    cls = pct > 0 ? "up" : pct < 0 ? "down" : "flat";
    const sign = pct > 0 ? "+" : "";
    price = q.price;
    change = sign + q.change;
    pctText = sign + q.change_pct + "%";
  }

  const delBtn = isIndex
    ? '<span></span>'
    : `<button class="btn-del" data-idx="${idx}">删除</button>`;

  return `
    <div class="list-item">
      <input type="checkbox" class="toggle" data-code="${code}" ${visible ? "checked" : ""} title="在悬浮窗显示" />
      <span class="stock-name">${name}</span>
      <span class="stock-code">${code}</span>
      <span class="stock-price ${cls}">${price}</span>
      <span class="stock-change ${cls}">${change}</span>
      <span class="stock-pct ${cls}">${pctText}</span>
      ${delBtn}
    </div>`;
}

// ─── Actions ──────────────────────────────────────────────
async function toggleVisible(code, checked) {
  const idx = config.hidden_from_bar.indexOf(code);
  if (checked && idx >= 0) {
    config.hidden_from_bar.splice(idx, 1);
  } else if (!checked && idx < 0) {
    config.hidden_from_bar.push(code);
  }
  await invoke("save_config", { config });
}

async function addStock() {
  const market = document.getElementById("market").value;
  const codeInput = document.getElementById("stock-code");
  const code = codeInput.value.trim();
  if (!code || code.length < 4) {
    showStatus("请输入正确的股票代码");
    return;
  }

  const full = market + code;
  if (config.watchlist.includes(full)) {
    showStatus("已存在: " + full);
    return;
  }
  config.watchlist.push(full);
  await saveAndRefresh();
  codeInput.value = "";
  showStatus("已添加: " + full);
  refreshQuotes();
}

async function removeStock(idx) {
  const code = config.watchlist[idx];
  config.watchlist.splice(idx, 1);
  // Also drop any hidden entry for the removed code
  const hi = config.hidden_from_bar.indexOf(code);
  if (hi >= 0) config.hidden_from_bar.splice(hi, 1);
  await saveAndRefresh();
  showStatus("已删除");
}

// ─── Utils ────────────────────────────────────────────────
async function saveAndRefresh() {
  await invoke("save_config", { config });
  renderList();
}

function showStatus(msg) {
  const el = document.getElementById("status");
  el.textContent = msg;
  setTimeout(() => (el.textContent = ""), 3000);
}

document.addEventListener("DOMContentLoaded", init);
