var params = new URLSearchParams(window.location.search);
var targetUrl = params.get('url') || '';
var siteId = params.get('siteId') || '';

var title = document.getElementById('title');
var statusEl = document.getElementById('status');
var retryBtn = document.getElementById('retryBtn');
var spinner = document.querySelector('.spinner');

function showError(msg) {
  spinner.classList.add('hidden');
  title.textContent = 'Connection Failed';
  statusEl.textContent = msg || 'Connection failed';
  statusEl.className = 'status error';
  retryBtn.classList.remove('hidden');
}

function attemptOnce() {
  chrome.runtime.sendMessage({ type: 'knock', siteId: siteId }, function (resp) {
    if (chrome.runtime.lastError) {
      showError(chrome.runtime.lastError.message || 'Unable to connect');
      return;
    }
    if (resp && resp.success) {
      chrome.runtime.sendMessage({ type: 'navigate', url: targetUrl });
      return;
    }
    showError((resp && resp.message) || 'Unable to connect');
  });
}

retryBtn.addEventListener('click', function () {
  retryBtn.classList.add('hidden');
  spinner.classList.remove('hidden');
  title.textContent = 'Connecting...';
  statusEl.textContent = '';
  statusEl.className = 'status';
  setTimeout(attemptOnce, 300);
});

loadSettings().then(function (s) {
  if (s.autoKnock !== false) {
    title.textContent = 'Connecting...';
    attemptOnce();
  } else {
    showError('Connection failed');
  }
});
