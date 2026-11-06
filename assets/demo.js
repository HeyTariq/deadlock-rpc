/*
 * Interactive Discord Rich Presence sandbox.
 *
 * This is a port of the presence logic in the Rust source. Each branch below
 * cites the file and function it mirrors; if the Rust changes, change this to
 * match.
 *
 *   src/main.rs::build_activity          - assembles the Discord activity payload
 *   src/main.rs::run_rpc_loop            - decides details/state/party per phase
 *   src/game_state.rs::presence_status   - the per-phase status string
 *   src/game_state.rs::format_match_elapsed - the match clock
 *   src/hero_api.rs::fetch               - picks the portrait for each style
 *   src/config.rs::apply_vars            - the {token} substitution
 *
 * Hero data and art come live from the same places the app uses, so new heroes
 * and art show up here without a site change.
 */
(function () {
  'use strict';

  var DISCORD_APP_ID = '1566683259763429509';
  var HEROES_URL = 'https://api.deadlock-api.com/v1/assets/heroes';
  var ASSETS_URL = 'https://discord.com/api/v9/oauth2/applications/' + DISCORD_APP_ID + '/assets';
  var LOCAL_ICON = '../assets/icon.png';
  var DEFAULT_HERO = 'hero_inferno';

  // src/main.rs:29 - floor for how often the ticking clock alone pushes an update.
  var TIMER_REFRESH_SECS = 5;

  // GamePhase - src/game_state.rs:34
  // [ key, description(), shows_hero(), control label ]
  var PHASES = [
    ['NotRunning', 'Not Running',         false, 'Game closed'],
    ['MainMenu',   'Main Menu',           false, 'Main menu'],
    ['Hideout',    'Hideout',             true,  'Hideout'],
    ['InQueue',    'Searching for Match', true,  'In queue'],
    ['MatchIntro', 'Match Starting',      true,  'Loading in'],
    ['InMatch',    'In Match',            true,  'In match'],
    ['PostMatch',  'Post Match',          false, 'Post match'],
    ['Spectating', 'Spectating',          false, 'Spectating']
  ];

  // MatchMode - src/game_state.rs:5
  // [ key, display(), show_map_location() ]
  var MODES = [
    ['Unknown',       'In Match',       true],
    ['Standard',      'Standard Match', true],
    ['Ranked',        'Ranked Match',   true],
    ['StreetBrawl',   'Street Brawl',   true],
    ['BotMatch',      'Bot Match',      false],
    ['TrainingRange', 'Training Range', false],
    ['HeroLabs',      'Hero Labs',      false]
  ];

  // HeroPortraitStyle - src/config.rs:29
  var STYLES = [
    { value: 'normal',   label: 'Normal' },
    { value: 'gloat',    label: 'Gloat' },
    { value: 'critical', label: 'Critical' },
    { value: 'icon',     label: 'Icon' },
    { value: 'fullbody', label: 'Full body' }
  ];

  // Defaults mirror src/default_config.toml exactly.
  var DEFAULTS = {
    show_hero_image: true,
    show_match_timer: true,
    show_statlocker_button: false,
    hero_portrait_style: 'normal',
    details_with_hero: 'Playing as {hero}',
    details_without_hero: '{phase}',
    game_not_running: 'Not Running',
    in_main_menu: 'Browsing the Main Menu',
    in_hideout: 'In the Hideout',
    in_matchmaking: 'Searching for a Match...',
    loading_into_match: '{mode} - Loading into Match',
    in_match: 'In Match: {mode}',
    match_location_label: 'the Cursed Apple',
    post_match: 'Reviewing Match Results',
    spectating: 'Spectating a Match',
    fallback_large_image_tooltip: 'Deadlock RPC',
    corner_image_tooltip: 'Deadlock RPC'
  };

  // Which {tokens} each text field understands, for the chips in the UI.
  var TOKENS = {
    details_with_hero: ['hero'],
    details_without_hero: ['phase'],
    in_hideout: ['hero'],
    loading_into_match: ['mode'],
    in_match: ['mode', 'location', 'timer']
  };

  // Order and labels for the editable text fields.
  var TEXT_FIELDS = [
    ['details_with_hero',    'Details, hero known'],
    ['details_without_hero', 'Details, no hero'],
    ['game_not_running',     'Game not running'],
    ['in_main_menu',         'Main menu'],
    ['in_hideout',           'Hideout'],
    ['in_matchmaking',       'In queue'],
    ['loading_into_match',   'Loading in'],
    ['in_match',             'In match'],
    ['match_location_label', 'Map location'],
    ['post_match',           'Post match'],
    ['spectating',           'Spectating'],
    ['fallback_large_image_tooltip', 'Fallback art tooltip'],
    ['corner_image_tooltip', 'Corner icon tooltip']
  ];

  // ---- state -------------------------------------------------------------

  // Filled from the hero API: { key, name, hideout, images }.
  var heroes = [];
  // 'loading' | 'ok' | 'failed'
  var heroLoad = 'loading';
  // Discord app art assets, name -> CDN url.
  var assets = {};

  var cfg = {};
  var phaseKey = 'InMatch';
  var modeKey = 'Standard';
  var heroIdx = 0;
  var partySize = 1;
  var startedAt = Date.now();
  var matchStartedAt = Date.now();
  var lastTimerBucket = 0;
  var simTimer = null;
  var tickTimer = null;

  function resetCfg() {
    cfg = {};
    for (var k in DEFAULTS) { if (DEFAULTS.hasOwnProperty(k)) cfg[k] = DEFAULTS[k]; }
  }
  resetCfg();

  function phase() {
    for (var i = 0; i < PHASES.length; i++) {
      if (PHASES[i][0] === phaseKey) return PHASES[i];
    }
    return PHASES[0];
  }

  function mode() {
    for (var i = 0; i < MODES.length; i++) {
      if (MODES[i][0] === modeKey) return MODES[i];
    }
    return MODES[0];
  }

  function defaultHeroIdx() {
    for (var i = 0; i < heroes.length; i++) {
      if (heroes[i].key === DEFAULT_HERO) return i;
    }
    return 0;
  }

  // src/game_state.rs:215-231 - the clock starts on entering InMatch and is
  // cleared on leaving it.
  function setPhase(next) {
    if (next === 'InMatch' && phaseKey !== 'InMatch') {
      matchStartedAt = Date.now();
      lastTimerBucket = 0;
    }
    phaseKey = next;
  }

  // The deadlock_logo asset on the Discord app, or the local copy of it.
  function logoArt() {
    return assets.deadlock_logo || LOCAL_ICON;
  }

  // src/hero_api.rs:75-111 - the image sent for each portrait style. Returns
  // { key, url }: key is what the app sends to Discord, url is how it looks.
  function heroArt(hero) {
    var im = hero.images;
    var pick = function (preferred) { return preferred || im.icon_hero_card || ''; };
    switch (cfg.hero_portrait_style) {
      case 'gloat':    return urlArt(pick(im.hero_card_gloat));
      case 'critical': return urlArt(pick(im.hero_card_critical));
      case 'icon':     return urlArt(pick(im.icon_image_small));
      case 'fullbody':
        // Sent as a Discord asset key named after the hero, never as a URL.
        return { key: hero.key, url: assets[hero.key] || '' };
    }
    return urlArt(im.icon_hero_card || '');
  }

  function urlArt(url) { return { key: url, url: url }; }

  // ---- the port ----------------------------------------------------------

  // src/config.rs:226 - plain {key} replacement, no escaping.
  function applyVars(template, vars) {
    var out = template;
    for (var key in vars) {
      if (vars.hasOwnProperty(key)) {
        out = out.split('{' + key + '}').join(vars[key]);
      }
    }
    return out;
  }

  // src/game_state.rs:71 - "07:42", or "1:07:42" past an hour.
  function formatMatchElapsed(sec) {
    var h = Math.floor(sec / 3600);
    var m = Math.floor((sec % 3600) / 60);
    var s = sec % 60;
    var pad = function (n) { return n < 10 ? '0' + n : String(n); };
    if (h > 0) return h + ':' + pad(m) + ':' + pad(s);
    return pad(m) + ':' + pad(s);
  }

  // src/main.rs:161-166 - only during a match, and only with the timer on.
  // The clock reads as of the last push, which happens once per bucket.
  function matchTimer() {
    if (!cfg.show_match_timer || phaseKey !== 'InMatch') return null;
    return formatMatchElapsed(lastTimerBucket * TIMER_REFRESH_SECS);
  }

  // src/game_state.rs:252 - presence_status()
  function gameStatus(hero, timer) {
    var m = mode();
    switch (phaseKey) {
      case 'NotRunning': return cfg.game_not_running;
      case 'MainMenu':   return cfg.in_main_menu;
      case 'Hideout':
        // The hero's own API text wins when present.
        if (hero && hero.hideout) return hero.hideout;
        return applyVars(cfg.in_hideout, { hero: hero ? hero.name : '' });
      case 'InQueue':    return cfg.in_matchmaking;
      case 'MatchIntro': return applyVars(cfg.loading_into_match, { mode: m[1] });
      case 'InMatch': {
        // For modes without a map location the whole template is discarded and
        // the bare mode name is used. src/game_state.rs:274-288
        var usesTemplate = m[2];
        var status = usesTemplate
          ? applyVars(cfg.in_match, { mode: m[1], location: cfg.match_location_label, timer: timer || '' })
          : m[1];
        // Templates without {timer} still get it, appended. src/game_state.rs:289-293
        var placed = usesTemplate && cfg.in_match.indexOf('{timer}') !== -1;
        if (timer && !placed) status = status + ' - ' + timer;
        return status;
      }
      case 'PostMatch':  return cfg.post_match;
      case 'Spectating': return cfg.spectating;
    }
    return '';
  }

  // Produces the same values build_activity() sends to Discord.
  function buildActivity() {
    var ph = phase();

    // src/main.rs:209-215 - show_hero_image gates the hero entirely, so with it
    // off the name disappears along with the portrait. A failed hero fetch
    // has the same effect.
    var hero = (cfg.show_hero_image && ph[2] && heroes.length) ? heroes[heroIdx] : null;

    var status = gameStatus(hero, matchTimer());

    // src/main.rs:228-234
    var heroLabel = hero
      ? applyVars(cfg.details_with_hero, { hero: hero.name })
      : applyVars(cfg.details_without_hero, { phase: ph[1] });

    // src/main.rs:237 - party is only ever surfaced in the Hideout.
    var showParty = (phaseKey === 'Hideout' && partySize > 1);

    // src/main.rs:242-249
    var details, state;
    if (phaseKey === 'Hideout') {
      details = status;
      state = showParty ? 'In a Party' : null;
    } else if (phaseKey === 'NotRunning' || phaseKey === 'Spectating') {
      details = status;
      state = null;
    } else {
      details = heroLabel;
      state = status;
    }

    // src/main.rs:83-86 - hero art, else the deadlock_logo fallback key.
    var art = hero ? heroArt(hero) : null;
    var logoFallback = !art || !art.key;

    return {
      details: details,
      state: state,
      largeImage: logoFallback ? logoArt() : (art.url || logoArt()),
      largeText: hero ? hero.name : cfg.fallback_large_image_tooltip,
      // src/main.rs:96-101 - no corner icon when it would repeat the logo.
      showSmall: !logoFallback,
      smallText: cfg.corner_image_tooltip,
      // src/main.rs:116 - party max is hardcoded to 6.
      party: showParty ? partySize + ' of 6' : null,
      button: cfg.show_statlocker_button,
      missingFullBody: !!(art && art.key && !art.url)
    };
  }

  // ---- data --------------------------------------------------------------

  function getJson(url) {
    return fetch(url).then(function (r) {
      if (!r.ok) throw new Error('HTTP ' + r.status);
      return r.json();
    });
  }

  function loadData() {
    getJson(ASSETS_URL).then(function (list) {
      list.forEach(function (a) {
        assets[a.name] = 'https://cdn.discordapp.com/app-assets/' + DISCORD_APP_ID + '/' + a.id + '.png';
      });
      el.small.src = logoArt();
      render();
    }).catch(function () { /* the local icon stands in for the logo */ });

    getJson(HEROES_URL).then(function (list) {
      heroes = list
        .filter(function (h) { return h.player_selectable && !h.disabled && h.class_name && h.name; })
        .map(function (h) {
          return {
            key: h.class_name,
            name: h.name,
            hideout: h.hideout_rich_presence || '',
            images: h.images || {}
          };
        })
        .sort(function (a, b) { return a.name.localeCompare(b.name); });
      if (!heroes.length) throw new Error('no heroes');
      heroLoad = 'ok';
      heroIdx = defaultHeroIdx();
      fillHeroSelect();
      render();
    }).catch(function () {
      heroLoad = 'failed';
      fillHeroSelect();
      render();
    });
  }

  // ---- rendering ---------------------------------------------------------

  var el = {};
  function $(id) { return document.getElementById(id); }

  function renderCard() {
    var a = buildActivity();

    el.details.textContent = a.details;

    if (a.state === null) {
      el.state.hidden = true;
    } else {
      el.state.hidden = false;
      // Discord appends the party count to the state line.
      el.state.textContent = a.party ? a.state + ' (' + a.party + ')' : a.state;
    }

    if (el.large.getAttribute('data-src') !== a.largeImage) {
      el.large.setAttribute('data-src', a.largeImage);
      el.large.src = a.largeImage;
    }
    el.large.alt = a.largeText;
    el.large.title = a.largeText;
    el.small.hidden = !a.showSmall;
    el.small.alt = a.smallText;
    el.small.title = a.smallText;

    el.button.hidden = !a.button;
    return a;
  }

  // src/main.rs:149-153 captures this once at startup, so the card's elapsed
  // line measures how long the app has been running, not the current match.
  function fmtElapsed(sec) {
    var h = Math.floor(sec / 3600);
    var m = Math.floor((sec % 3600) / 60);
    var s = sec % 60;
    var pad = function (n) { return n < 10 ? '0' + n : String(n); };
    if (h > 0) return h + ':' + pad(m) + ':' + pad(s) + ' elapsed';
    return m + ':' + pad(s) + ' elapsed';
  }

  function tick() {
    el.time.textContent = fmtElapsed(Math.floor((Date.now() - startedAt) / 1000));
    if (phaseKey === 'InMatch') {
      var bucket = Math.floor((Date.now() - matchStartedAt) / 1000 / TIMER_REFRESH_SECS);
      if (bucket !== lastTimerBucket) {
        lastTimerBucket = bucket;
        renderCard();
      }
    }
  }

  // One context-sensitive line, so the sandbox can explain itself without
  // cluttering every control with helper text.
  function renderHint(a) {
    var ph = phase();
    var hero = heroes[heroIdx];
    var msg = '';

    if (heroLoad === 'failed') {
      msg = 'Couldn’t reach the hero API, so the card shows the logo instead of your hero.';
    } else if (heroLoad === 'loading' && ph[2] && cfg.show_hero_image) {
      msg = 'Loading heroes…';
    } else if (!cfg.show_hero_image && ph[2]) {
      msg = 'With the portrait off, your hero’s name is hidden too. The details line falls back to details_without_hero.';
    } else if (!ph[2] && phaseKey !== 'NotRunning') {
      msg = 'Deadlock RPC doesn’t show your hero during this phase.';
    } else if (a.missingFullBody) {
      msg = hero.name + ' has no full body art in the Discord app yet, so the logo stands in here.';
    } else if (partySize > 1 && phaseKey !== 'Hideout') {
      msg = 'Party size only appears while you’re in the Hideout.';
    } else if (phaseKey === 'Hideout' && hero && !hero.hideout) {
      msg = hero.name + ' has no custom Hideout line, so your in_hideout text is used instead.';
    } else if (phaseKey === 'InMatch' && !mode()[2]) {
      msg = 'For this mode the name is shown on its own, and the in_match template isn’t used.';
    } else if (phaseKey === 'InMatch' && cfg.show_match_timer) {
      msg = 'The match clock moves in ' + TIMER_REFRESH_SECS + ' second steps, as often as the app updates Discord.';
    } else if (cfg.show_statlocker_button) {
      msg = 'In the real app this button appears once your Steam ID is detected in the game log.';
    }

    el.hint.textContent = msg;
  }

  function tomlEscape(s) {
    return String(s).replace(/\\/g, '\\\\').replace(/"/g, '\\"');
  }

  function renderToml() {
    var q = function (v) { return '"' + tomlEscape(v) + '"'; };
    var lines = [
      '# Only the settings shown in this demo.',
      '# See the Setup Guide for every available option.',
      '',
      '[presence]',
      'show_hero_image = ' + cfg.show_hero_image,
      'show_match_timer = ' + cfg.show_match_timer,
      'show_statlocker_button = ' + cfg.show_statlocker_button,
      'hero_portrait_style = ' + q(cfg.hero_portrait_style),
      'details_with_hero = ' + q(cfg.details_with_hero),
      'details_without_hero = ' + q(cfg.details_without_hero),
      '',
      '[presence.status]',
      'game_not_running = ' + q(cfg.game_not_running),
      'in_main_menu = ' + q(cfg.in_main_menu),
      'in_hideout = ' + q(cfg.in_hideout),
      'in_matchmaking = ' + q(cfg.in_matchmaking),
      'loading_into_match = ' + q(cfg.loading_into_match),
      'in_match = ' + q(cfg.in_match),
      'match_location_label = ' + q(cfg.match_location_label),
      'post_match = ' + q(cfg.post_match),
      'spectating = ' + q(cfg.spectating),
      '',
      '[images]',
      'fallback_large_image = "deadlock_logo"',
      'fallback_large_image_tooltip = ' + q(cfg.fallback_large_image_tooltip),
      'corner_image = "deadlock_logo"',
      'corner_image_tooltip = ' + q(cfg.corner_image_tooltip)
    ];
    el.toml.textContent = lines.join('\n');
  }

  function render() {
    renderHint(renderCard());
    renderToml();
  }

  // ---- controls ----------------------------------------------------------

  function buildSegmented(container, items, getActive, onPick) {
    container.innerHTML = '';
    items.forEach(function (item) {
      var b = document.createElement('button');
      b.type = 'button';
      b.className = 'seg-btn';
      b.textContent = item.label;
      b.setAttribute('role', 'radio');
      b.dataset.value = item.value;
      b.addEventListener('click', function () {
        onPick(item.value);
        syncSegmented(container, getActive);
      });
      container.appendChild(b);
    });
    syncSegmented(container, getActive);
  }

  function syncSegmented(container, getActive) {
    var active = getActive();
    var btns = container.querySelectorAll('.seg-btn');
    for (var i = 0; i < btns.length; i++) {
      var on = btns[i].dataset.value === active;
      btns[i].classList.toggle('is-active', on);
      btns[i].setAttribute('aria-checked', on ? 'true' : 'false');
      // Roving tabindex so the group is one tab stop.
      btns[i].tabIndex = on ? 0 : -1;
    }
  }

  // Arrow-key navigation within a radiogroup.
  function wireSegmentedKeys(container) {
    container.addEventListener('keydown', function (e) {
      if (e.key !== 'ArrowRight' && e.key !== 'ArrowLeft' &&
          e.key !== 'ArrowDown' && e.key !== 'ArrowUp') return;
      var btns = Array.prototype.slice.call(container.querySelectorAll('.seg-btn'));
      var cur = btns.indexOf(document.activeElement);
      if (cur === -1) return;
      e.preventDefault();
      var fwd = (e.key === 'ArrowRight' || e.key === 'ArrowDown');
      var next = btns[(cur + (fwd ? 1 : -1) + btns.length) % btns.length];
      next.focus();
      next.click();
    });
  }

  function fillHeroSelect() {
    var sel = el.heroSel;
    sel.innerHTML = '';
    if (heroLoad !== 'ok') {
      var o = document.createElement('option');
      o.textContent = heroLoad === 'loading' ? 'Loading heroes…' : 'Heroes unavailable';
      sel.appendChild(o);
      sel.disabled = true;
      return;
    }
    heroes.forEach(function (h, i) {
      var opt = document.createElement('option');
      opt.value = String(i);
      opt.textContent = h.name;
      sel.appendChild(opt);
    });
    sel.disabled = false;
    sel.value = String(heroIdx);
  }

  function stopSim() {
    if (simTimer) {
      clearTimeout(simTimer);
      simTimer = null;
      el.sim.textContent = 'Simulate a match';
    }
  }

  // [ phase, ms to hold it ]. The match runs long enough for the clock to move.
  var SIM_SEQUENCE = [
    ['MainMenu', 2500], ['Hideout', 2500], ['InQueue', 2500],
    ['MatchIntro', 2500], ['InMatch', 11000], ['PostMatch', 2500]
  ];

  function syncPhaseSeg() {
    syncSegmented(el.phaseSeg, function () { return phaseKey; });
  }

  function runSim(i) {
    if (i >= SIM_SEQUENCE.length) { stopSim(); return; }
    setPhase(SIM_SEQUENCE[i][0]);
    syncPhaseSeg();
    render();
    simTimer = setTimeout(function () { runSim(i + 1); }, SIM_SEQUENCE[i][1]);
  }

  function insertToken(input, token) {
    var t = '{' + token + '}';
    var start = input.selectionStart;
    var end = input.selectionEnd;
    if (typeof start === 'number' && typeof end === 'number') {
      input.value = input.value.slice(0, start) + t + input.value.slice(end);
      input.selectionStart = input.selectionEnd = start + t.length;
    } else {
      input.value += t;
    }
    input.focus();
    input.dispatchEvent(new Event('input'));
  }

  function buildTextFields() {
    var wrap = el.textFields;
    wrap.innerHTML = '';
    TEXT_FIELDS.forEach(function (pair) {
      var key = pair[0], label = pair[1];

      var row = document.createElement('div');
      row.className = 'text-row';

      var lab = document.createElement('label');
      lab.setAttribute('for', 'txt-' + key);
      lab.innerHTML = label + ' <code>' + key + '</code>';

      var input = document.createElement('input');
      input.type = 'text';
      input.id = 'txt-' + key;
      // Discord's own limit for these fields.
      input.maxLength = 128;
      input.value = cfg[key];
      input.addEventListener('input', function () {
        cfg[key] = input.value;
        stopSim();
        render();
      });

      row.appendChild(lab);
      row.appendChild(input);

      var toks = TOKENS[key];
      if (toks) {
        var chips = document.createElement('div');
        chips.className = 'chips';
        toks.forEach(function (tok) {
          var c = document.createElement('button');
          c.type = 'button';
          c.className = 'chip';
          c.textContent = '{' + tok + '}';
          c.title = 'Insert {' + tok + '}';
          c.addEventListener('click', function () { insertToken(input, tok); });
          chips.appendChild(c);
        });
        row.appendChild(chips);
      }

      wrap.appendChild(row);
    });
  }

  function syncTextFields() {
    TEXT_FIELDS.forEach(function (pair) {
      var input = $('txt-' + pair[0]);
      if (input) input.value = cfg[pair[0]];
    });
  }

  function bindCheckbox(id, key) {
    var box = $(id);
    box.checked = cfg[key];
    box.addEventListener('change', function () {
      cfg[key] = box.checked;
      stopSim();
      render();
    });
    return box;
  }

  // ---- init --------------------------------------------------------------

  function init() {
    el.details = $('rpc-details');
    el.state = $('rpc-state');
    el.time = $('rpc-time');
    el.large = $('rpc-large');
    el.small = $('rpc-small');
    el.button = $('rpc-button');
    el.hint = $('d-hint');
    el.toml = $('d-toml');
    el.textFields = $('d-text-fields');
    el.phaseSeg = $('d-phase');
    el.heroSel = $('d-hero');
    el.sim = $('d-sim');

    if (!el.details) return;

    // A CDN hiccup degrades to the logo, the same fallback the real app uses
    // when it has no hero art.
    el.large.addEventListener('error', function () {
      var src = el.large.getAttribute('src');
      if (src !== logoArt() && src !== LOCAL_ICON) el.large.src = logoArt();
      else if (src !== LOCAL_ICON) el.large.src = LOCAL_ICON;
    });
    el.small.addEventListener('error', function () {
      if (el.small.getAttribute('src') !== LOCAL_ICON) el.small.src = LOCAL_ICON;
    });
    el.small.src = logoArt();

    // Game state
    buildSegmented(
      el.phaseSeg,
      PHASES.map(function (p) { return { value: p[0], label: p[3] }; }),
      function () { return phaseKey; },
      function (v) { setPhase(v); stopSim(); render(); }
    );
    wireSegmentedKeys(el.phaseSeg);

    // Portrait style
    var styleSeg = $('d-style');
    buildSegmented(
      styleSeg,
      STYLES,
      function () { return cfg.hero_portrait_style; },
      function (v) { cfg.hero_portrait_style = v; stopSim(); render(); }
    );
    wireSegmentedKeys(styleSeg);

    // Hero
    fillHeroSelect();
    el.heroSel.addEventListener('change', function () {
      heroIdx = parseInt(el.heroSel.value, 10);
      stopSim();
      render();
    });

    // Match mode
    var modeSel = $('d-mode');
    MODES.forEach(function (m) {
      var o = document.createElement('option');
      o.value = m[0];
      o.textContent = m[1];
      modeSel.appendChild(o);
    });
    modeSel.value = modeKey;
    modeSel.addEventListener('change', function () {
      modeKey = modeSel.value;
      stopSim();
      render();
    });

    // Party size
    var party = $('d-party');
    var partyVal = $('d-party-val');
    party.addEventListener('input', function () {
      partySize = parseInt(party.value, 10);
      partyVal.textContent = partySize + ' of 6';
      stopSim();
      render();
    });

    // Toggles
    var heroImg = bindCheckbox('d-heroimg', 'show_hero_image');
    var timerBox = bindCheckbox('d-timer', 'show_match_timer');
    var statlocker = bindCheckbox('d-statlocker', 'show_statlocker_button');

    // Simulate / reset
    var reduced = window.matchMedia &&
      window.matchMedia('(prefers-reduced-motion: reduce)').matches;

    el.sim.addEventListener('click', function () {
      if (simTimer) { stopSim(); return; }
      if (reduced) {
        // Step manually rather than auto-advancing.
        var keys = SIM_SEQUENCE.map(function (s) { return s[0]; });
        var i = keys.indexOf(phaseKey);
        setPhase(keys[(i + 1) % keys.length]);
        syncPhaseSeg();
        render();
        return;
      }
      el.sim.textContent = 'Stop';
      runSim(0);
    });
    if (reduced) el.sim.textContent = 'Step through a match';

    $('d-reset').addEventListener('click', function () {
      stopSim();
      resetCfg();
      phaseKey = 'MainMenu';
      setPhase('InMatch');
      modeKey = 'Standard';
      heroIdx = defaultHeroIdx();
      partySize = 1;
      if (heroLoad === 'ok') el.heroSel.value = String(heroIdx);
      modeSel.value = modeKey;
      party.value = '1';
      partyVal.textContent = '1 of 6';
      heroImg.checked = cfg.show_hero_image;
      timerBox.checked = cfg.show_match_timer;
      statlocker.checked = cfg.show_statlocker_button;
      syncPhaseSeg();
      syncSegmented(styleSeg, function () { return cfg.hero_portrait_style; });
      syncTextFields();
      render();
    });

    // The mock button is inert; say so rather than looking broken.
    el.button.addEventListener('click', function () {
      el.hint.textContent = 'This button is part of the mock. In Discord it opens your Statlocker match history.';
    });

    // Copy config
    var copyBtn = $('d-copy');
    copyBtn.addEventListener('click', function () {
      var text = el.toml.textContent;
      var done = function () {
        copyBtn.textContent = 'Copied';
        setTimeout(function () { copyBtn.textContent = 'Copy'; }, 2000);
      };
      if (navigator.clipboard && navigator.clipboard.writeText) {
        navigator.clipboard.writeText(text).then(done, function () { legacyCopy(text, done); });
      } else {
        legacyCopy(text, done);
      }
    });

    buildTextFields();

    tick();
    startTick();
    document.addEventListener('visibilitychange', function () {
      if (document.hidden) {
        clearInterval(tickTimer);
        tickTimer = null;
      } else {
        tick();
        startTick();
      }
    });

    render();
    loadData();
  }

  function startTick() {
    if (tickTimer) return;
    tickTimer = setInterval(tick, 1000);
  }

  function legacyCopy(text, done) {
    var ta = document.createElement('textarea');
    ta.value = text;
    ta.setAttribute('readonly', '');
    ta.style.position = 'absolute';
    ta.style.left = '-9999px';
    document.body.appendChild(ta);
    ta.select();
    try { document.execCommand('copy'); done(); } catch (e) { /* nothing to do */ }
    document.body.removeChild(ta);
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
  } else {
    init();
  }
}());
