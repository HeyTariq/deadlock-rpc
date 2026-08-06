/*
 * Interactive Discord Rich Presence sandbox.
 *
 * This is a faithful port of the presence logic in the Rust source. Each branch
 * below cites the file and function it mirrors; if the Rust changes, change this
 * to match.
 *
 *   src/main.rs::build_activity      - assembles the Discord activity payload
 *   src/main.rs::run_rpc_loop        - decides details/state/party per phase
 *   src/game_state.rs::presence_status - the per-phase status string
 *   src/config.rs::apply_vars        - the {token} substitution
 */
(function () {
  'use strict';

  var CDN = 'https://assets-bucket.deadlock-api.com/assets-api-res/images/heroes/';
  var FALLBACK_ART = '../assets/icon.png';

  /* Hero roster. Values come from the same API the app uses:
   *   api.deadlock-api.com/v1/assets/heroes  -> class_name, display name
   *   assets.deadlock-api.com/v2/heroes/by-name/<name> -> images, hideout text
   *
   * The image base is NOT derivable from the class name (hero_ghost -> spectre,
   * hero_atlas -> bull), so it is recorded per hero.
   *
   * [ class_name, display name, image base, hideout_rich_presence ]
   */
  var HEROES = [
    ['hero_hornet',   'Vindicta',   'hornet',   'Brooding in the Hideout'],
    ['hero_inferno',  'Infernus',   'inferno',  'Mixing Drinks in the Hideout'],
    ['hero_atlas',    'Abrams',     'bull',     'Investigating the Hideout'],
    ['hero_haze',     'Haze',       'haze',     'Sleep Walking in the Hideout'],
    ['hero_gigawatt', 'Seven',      'gigawatt', 'Plotting in the Hideout'],
    ['hero_ghost',    'Lady Geist', 'spectre',  'Being Fabulous in the Hideout'],
    ['hero_bebop',    'Bebop',      'bebop',    'Ignoring Lash in the Hideout'],
    ['hero_lash',     'Lash',       'lash',     'Thinking About Lash in the Hideout'],
    ['hero_chrono',   'Paradox',    'chrono',   'Scheming in the Hideout'],
    ['hero_tengu',    'Ivy',        'tengu',    'Wishing the Arroyos were in the Hideout'],
    ['hero_kelvin',   'Kelvin',     'kelvin',   'Chilling in the Hideout'],
    // Mo & Krill has no hideout_rich_presence, so the in_hideout template is used.
    ['hero_krill',    'Mo & Krill', 'digger',   '']
  ];

  // GamePhase - src/game_state.rs:33
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

  // MatchMode - src/game_state.rs:4
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

  // Defaults mirror src/default_config.toml exactly.
  var DEFAULTS = {
    show_hero_image: true,
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
    fallback_large_image_tooltip: 'Deadlock',
    corner_image_tooltip: 'Deadlock RPC'
  };

  // Which {tokens} each text field understands, for the chips in the UI.
  var TOKENS = {
    details_with_hero: ['hero'],
    details_without_hero: ['phase'],
    in_hideout: ['hero'],
    loading_into_match: ['mode'],
    in_match: ['mode', 'location']
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

  var cfg = {};
  var phaseKey = 'InMatch';
  var modeKey = 'Standard';
  var heroIdx = 0;
  var partySize = 1;
  var startedAt = Date.now();
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

  function heroArt() {
    var base = HEROES[heroIdx][2];
    var suffix = cfg.hero_portrait_style === 'gloat' ? '_card_gloat'
               : cfg.hero_portrait_style === 'critical' ? '_card_critical'
               : '_card';
    return CDN + base + suffix + '.webp';
  }

  // ---- the port ----------------------------------------------------------

  // src/config.rs:210 - plain {key} replacement, no escaping.
  function applyVars(template, vars) {
    var out = template;
    for (var key in vars) {
      if (vars.hasOwnProperty(key)) {
        out = out.split('{' + key + '}').join(vars[key]);
      }
    }
    return out;
  }

  // src/game_state.rs:188 - presence_status()
  function gameStatus(hero) {
    var m = mode();
    switch (phaseKey) {
      case 'NotRunning': return cfg.game_not_running;
      case 'MainMenu':   return cfg.in_main_menu;
      case 'Hideout':
        // The hero's own API text wins when present.
        if (hero && hero[3]) return hero[3];
        return applyVars(cfg.in_hideout, { hero: hero ? hero[1] : '' });
      case 'InQueue':    return cfg.in_matchmaking;
      case 'MatchIntro': return applyVars(cfg.loading_into_match, { mode: m[1] });
      case 'InMatch':
        // For modes without a map location the whole template is discarded and
        // the bare mode name is used. src/game_state.rs:208-217
        if (!m[2]) return m[1];
        return applyVars(cfg.in_match, { mode: m[1], location: cfg.match_location_label });
      case 'PostMatch':  return cfg.post_match;
      case 'Spectating': return cfg.spectating;
    }
    return '';
  }

  // Produces the same values build_activity() sends to Discord.
  function buildActivity() {
    var ph = phase();

    // src/main.rs:184-189 - show_hero_image gates the hero entirely, so with it
    // off the name disappears along with the portrait.
    var hero = (cfg.show_hero_image && ph[2]) ? HEROES[heroIdx] : null;

    var status = gameStatus(hero);

    // src/main.rs:202-208
    var heroLabel = hero
      ? applyVars(cfg.details_with_hero, { hero: hero[1] })
      : applyVars(cfg.details_without_hero, { phase: ph[1] });

    // src/main.rs:211 - party is only ever surfaced in the Hideout.
    var showParty = (phaseKey === 'Hideout' && partySize > 1);

    // src/main.rs:216-223
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

    return {
      details: details,
      state: state,
      // src/main.rs:69-76 - hero art else the configured fallback key.
      largeImage: hero ? heroArt() : FALLBACK_ART,
      largeText: hero ? hero[1] : cfg.fallback_large_image_tooltip,
      // src/main.rs:81-82 - the corner icon is always set.
      smallImage: FALLBACK_ART,
      smallText: cfg.corner_image_tooltip,
      // src/main.rs:97 - party max is hardcoded to 6.
      party: showParty ? partySize + ' of 6' : null,
      button: cfg.show_statlocker_button
    };
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
    el.small.alt = a.smallText;
    el.small.title = a.smallText;

    el.button.hidden = !a.button;
  }

  function fmtElapsed(sec) {
    var h = Math.floor(sec / 3600);
    var m = Math.floor((sec % 3600) / 60);
    var s = sec % 60;
    var pad = function (n) { return n < 10 ? '0' + n : String(n); };
    if (h > 0) return h + ':' + pad(m) + ':' + pad(s) + ' elapsed';
    return m + ':' + pad(s) + ' elapsed';
  }

  function renderElapsed() {
    el.time.textContent = fmtElapsed(Math.floor((Date.now() - startedAt) / 1000));
  }

  // One context-sensitive line, so the sandbox can explain itself without
  // cluttering every control with helper text.
  function renderHint() {
    var ph = phase();
    var msg = '';

    if (!cfg.show_hero_image && ph[2]) {
      msg = 'With the portrait off, your hero’s name is hidden too — the details line falls back to details_without_hero.';
    } else if (!ph[2] && phaseKey !== 'NotRunning') {
      msg = 'Deadlock RPC doesn’t show your hero during this phase.';
    } else if (partySize > 1 && phaseKey !== 'Hideout') {
      msg = 'Party size only appears while you’re in the Hideout.';
    } else if (phaseKey === 'Hideout' && !HEROES[heroIdx][3]) {
      msg = HEROES[heroIdx][1] + ' has no custom Hideout line, so your in_hideout text is used instead.';
    } else if (phaseKey === 'InMatch' && !mode()[2]) {
      msg = 'For this mode the name is shown on its own — the in_match template isn’t used.';
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
    renderCard();
    renderHint();
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

  function stopSim() {
    if (simTimer) {
      clearTimeout(simTimer);
      simTimer = null;
      el.sim.textContent = 'Simulate a match';
    }
  }

  var SIM_SEQUENCE = ['MainMenu', 'Hideout', 'InQueue', 'MatchIntro', 'InMatch', 'PostMatch'];

  function runSim(i) {
    if (i >= SIM_SEQUENCE.length) { stopSim(); return; }
    phaseKey = SIM_SEQUENCE[i];
    syncSegmented(el.phaseSeg, function () { return phaseKey; });
    render();
    simTimer = setTimeout(function () { runSim(i + 1); }, 2500);
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
    el.sim = $('d-sim');

    if (!el.details) return;

    // A CDN hiccup degrades to the app icon, which is the same fallback the
    // real app uses when it has no hero art.
    el.large.addEventListener('error', function () {
      if (el.large.src.indexOf('icon.png') === -1) el.large.src = FALLBACK_ART;
    });
    el.small.src = FALLBACK_ART;

    // Game state
    buildSegmented(
      el.phaseSeg,
      PHASES.map(function (p) { return { value: p[0], label: p[3] }; }),
      function () { return phaseKey; },
      function (v) { phaseKey = v; stopSim(); render(); }
    );
    wireSegmentedKeys(el.phaseSeg);

    // Portrait style
    var styleSeg = $('d-style');
    buildSegmented(
      styleSeg,
      [{ value: 'normal', label: 'Normal' },
       { value: 'gloat', label: 'Gloat' },
       { value: 'critical', label: 'Critical' }],
      function () { return cfg.hero_portrait_style; },
      function (v) { cfg.hero_portrait_style = v; stopSim(); render(); }
    );
    wireSegmentedKeys(styleSeg);

    // Hero
    var heroSel = $('d-hero');
    HEROES.forEach(function (h, i) {
      var o = document.createElement('option');
      o.value = String(i);
      o.textContent = h[1];
      heroSel.appendChild(o);
    });
    heroSel.value = String(heroIdx);
    heroSel.addEventListener('change', function () {
      heroIdx = parseInt(heroSel.value, 10);
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
    var heroImg = $('d-heroimg');
    heroImg.checked = cfg.show_hero_image;
    heroImg.addEventListener('change', function () {
      cfg.show_hero_image = heroImg.checked;
      stopSim();
      render();
    });

    var statlocker = $('d-statlocker');
    statlocker.checked = cfg.show_statlocker_button;
    statlocker.addEventListener('change', function () {
      cfg.show_statlocker_button = statlocker.checked;
      stopSim();
      render();
    });

    // Simulate / reset
    var reduced = window.matchMedia &&
      window.matchMedia('(prefers-reduced-motion: reduce)').matches;

    el.sim.addEventListener('click', function () {
      if (simTimer) { stopSim(); return; }
      if (reduced) {
        // Step manually rather than auto-advancing.
        var i = SIM_SEQUENCE.indexOf(phaseKey);
        phaseKey = SIM_SEQUENCE[(i + 1) % SIM_SEQUENCE.length];
        syncSegmented(el.phaseSeg, function () { return phaseKey; });
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
      phaseKey = 'InMatch';
      modeKey = 'Standard';
      heroIdx = 0;
      partySize = 1;
      heroSel.value = '0';
      modeSel.value = modeKey;
      party.value = '1';
      partyVal.textContent = '1 of 6';
      heroImg.checked = cfg.show_hero_image;
      statlocker.checked = cfg.show_statlocker_button;
      syncSegmented(el.phaseSeg, function () { return phaseKey; });
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

    // Elapsed timer. src/main.rs:130-134 captures this once at startup, so it
    // measures how long the app has been running, not the current match.
    renderElapsed();
    startTick();
    document.addEventListener('visibilitychange', function () {
      if (document.hidden) {
        clearInterval(tickTimer);
        tickTimer = null;
      } else {
        renderElapsed();
        startTick();
      }
    });

    render();
  }

  function startTick() {
    if (tickTimer) return;
    tickTimer = setInterval(renderElapsed, 1000);
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
