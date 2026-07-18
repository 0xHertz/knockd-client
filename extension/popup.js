function $(id) { return document.getElementById(id); }

document.addEventListener('DOMContentLoaded', async () => {
  const s = await loadSettings();
  const cb = $('autoKnock');
  cb.checked = s.autoKnock !== false;
  cb.addEventListener('change', async () => {
    s.autoKnock = cb.checked;
    await saveSettings(s);
  });
  // Refresh from native host then render
  setStatus('Loading...');
  await refreshSites();
  renderSiteList();
});

async function renderSiteList() {
  const sites = await fetchSites();
  if (!sites.length) {
    $('siteList').innerHTML = '<div class="empty-state">No SPA sites. Add them in Knockd Client.</div>';
    return;
  }
  $('siteList').innerHTML = sites.map((s, i) =>
    '<div class="site-card"><div class="site-info"><div class="site-name">' + esc(s.name || s.site_id) +
    '</div><div class="site-url">' + esc(s.url) + '</div></div>' +
    '<button class="knock-btn" data-idx="' + i + '" title="Knock &amp; Open">▶</button></div>'
  ).join('');
  document.querySelectorAll('.knock-btn').forEach(b => b.addEventListener('click', handleKnock));
}

function esc(s) { const d = document.createElement('div'); d.textContent = s; return d.innerHTML; }

async function handleKnock(e) {
  const idx = parseInt(e.target.dataset.idx);
  const sites = await fetchSites();
  const site = sites[idx];
  const btn = e.target;
  btn.classList.add('sending'); btn.textContent = '...';
  setStatus('Knocking ' + (site.name || site.site_id) + '...');
  try {
    const resp = await chrome.runtime.sendMessage({ type: 'knock', siteId: site.site_id });
    btn.classList.remove('sending');
    if (resp && resp.success) {
      btn.classList.add('success'); btn.textContent = '\u2713';
      setStatus('Opening...');
      // Open site in new tab
      chrome.tabs.create({ url: site.url });
      setTimeout(() => window.close(), 500);
    } else {
      btn.classList.add('failed'); btn.textContent = '\u2717';
      setStatus('Failed: ' + (resp ? resp.message : 'unknown'));
      setTimeout(() => { btn.classList.remove('failed'); btn.textContent = '\u25B6'; }, 3000);
    }
  } catch (e) {
    btn.classList.remove('sending'); btn.classList.add('failed'); btn.textContent = '!';
    setStatus('Error: ' + e.message);
  }
}

function setStatus(msg) { $('status').textContent = msg; }
