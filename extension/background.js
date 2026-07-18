importScripts('storage.js');

var bypassTabs = new Set();

// Refresh cache on startup
refreshSites();

chrome.runtime.onMessage.addListener((msg, sender, sendResponse) => {
  if (msg.type === 'list') {
    refreshSites().then(sites => sendResponse({ sites })).catch(() => sendResponse({ sites: [] }));
    return true;
  }

  if (msg.type === 'knock') {
    chrome.runtime.sendNativeMessage(NATIVE_HOST, { action: 'auth', site_id: msg.siteId })
      .then(sendResponse).catch(err => sendResponse({ success: false, message: err.message }));
    return true;
  }

  if (msg.type === 'navigate') {
    chrome.tabs.query({ active: true, currentWindow: true }, tabs => {
      if (tabs[0]) { bypassTabs.add(tabs[0].id); chrome.tabs.update(tabs[0].id, { url: msg.url }); }
    });
  }
});

chrome.webNavigation.onBeforeNavigate.addListener(async details => {
  if (details.frameId !== 0 || bypassTabs.has(details.tabId)) {
    bypassTabs.delete(details.tabId); return;
  }
  const sites = await fetchSites();
  if (!sites.length) return;
  const url = new URL(details.url);
  const match = sites.find(s => {
    try {
      const su = new URL(s.url);
      return su.hostname === url.hostname &&
        (su.port || (su.protocol === 'https:' ? '443' : '80')) ===
        (url.port || (url.protocol === 'https:' ? '443' : '80'));
    } catch { return false; }
  });
  if (match) {
    chrome.tabs.update(details.tabId, {
      url: chrome.runtime.getURL('blocked.html') +
        '?site=' + encodeURIComponent(match.name) +
        '&url=' + encodeURIComponent(details.url) +
        '&siteId=' + encodeURIComponent(match.site_id),
    });
  }
});
