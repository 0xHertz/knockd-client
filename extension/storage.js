var NATIVE_HOST = 'com.knockd.client';
var STORAGE_KEY = 'knockd_sites';
var SETTINGS_KEY = 'knockd_settings';

async function fetchSites() {
  try {
    const result = await chrome.storage.local.get(STORAGE_KEY);
    return result[STORAGE_KEY] || [];
  } catch { return []; }
}

async function saveSites(sites) {
  return chrome.storage.local.set({ [STORAGE_KEY]: sites });
}

function loadSettings() {
  return new Promise((resolve) => {
    chrome.storage.local.get(SETTINGS_KEY, (result) => {
      resolve(result[SETTINGS_KEY] || { autoKnock: true });
    });
  });
}

function saveSettings(settings) {
  return new Promise((resolve) => {
    chrome.storage.local.set({ [SETTINGS_KEY]: settings }, resolve);
  });
}

async function refreshSites() {
  try {
    const resp = await chrome.runtime.sendNativeMessage(NATIVE_HOST, { action: 'list' });
    if (resp && resp.success) {
      const sites = JSON.parse(resp.message);
      await saveSites(sites);
      return sites;
    }
  } catch(e) { console.log('refresh error:', e.message); }
  return [];
}
