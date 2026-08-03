(function () {
  var el = document.getElementById('version-badge');
  if (!el) return;

  var cacheKey = 'deadlock-rpc-latest-version';
  var cacheTtl = 60 * 60 * 1000;

  function render(tag) {
    var suffix = el.getAttribute('data-suffix');
    el.textContent = suffix ? (tag + ', ' + suffix) : ('Latest release: ' + tag);
  }

  var cached = null;
  try { cached = JSON.parse(localStorage.getItem(cacheKey) || 'null'); } catch (e) {}
  if (cached && cached.tag && Date.now() - cached.time < cacheTtl) {
    render(cached.tag);
  }

  fetch('https://api.github.com/repos/HeyTariq/deadlock-rpc/releases/latest')
    .then(function (res) { return res.ok ? res.json() : null; })
    .then(function (data) {
      if (!data || !data.tag_name) return;
      render(data.tag_name);
      try {
        localStorage.setItem(cacheKey, JSON.stringify({ tag: data.tag_name, time: Date.now() }));
      } catch (e) {}
    })
    .catch(function () {});
}());
