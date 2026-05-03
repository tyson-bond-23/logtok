/* logtok dashboard -- Alpine.js app state */

function app() {
  return {
    // D-24: Last active tab persists via localStorage
    tab: localStorage.getItem('logtok-tab') || 'tokenize',
    // D-26: Theme follows OS preference -- NOT persisted
    theme: window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light',
    // D-25: Recent file paths persist via localStorage
    recentFiles: JSON.parse(localStorage.getItem('logtok-recent') || '[]'),
    // Translations object -- populated from window.__i18n
    t: {},
    // Config editor mode
    configMode: 'form', // 'form' or 'toml'
    // Onboarding: show on first visit OR new version
    showOnboarding: (function() {
      var ver = document.querySelector('[data-version]');
      var currentVersion = ver ? ver.getAttribute('data-version') : '';
      var lastVersion = localStorage.getItem('logtok-version');
      return !lastVersion || lastVersion !== currentVersion;
    })(),
    isNewVersion: !!(localStorage.getItem('logtok-version') && localStorage.getItem('logtok-version') !== (document.querySelector('[data-version]') ? document.querySelector('[data-version]').getAttribute('data-version') : '')),
    showChangelog: false,

    setTab(name) {
      this.tab = name;
      localStorage.setItem('logtok-tab', name);
      // Auto-refresh store data when switching to store tab
      if (name === 'store') {
        document.body.dispatchEvent(new CustomEvent('refreshStore'));
      }
    },

    toggleTheme() {
      this.theme = this.theme === 'dark' ? 'light' : 'dark';
      // D-26: Do NOT persist to localStorage
    },

    addRecentFile(path) {
      if (!path) return;
      this.recentFiles = [path, ...this.recentFiles.filter(function(f) { return f !== path; })].slice(0, 10);
      localStorage.setItem('logtok-recent', JSON.stringify(this.recentFiles));
    },

    removeRecentFile(path) {
      this.recentFiles = this.recentFiles.filter(function(f) { return f !== path; });
      localStorage.setItem('logtok-recent', JSON.stringify(this.recentFiles));
    },

    // Toast notification
    toast: '',
    toastType: 'success',
    toastVisible: false,

    dismissOnboarding() {
      this.showOnboarding = false;
      this.showChangelog = false;
      var ver = document.querySelector('[data-version]');
      if (ver) localStorage.setItem('logtok-version', ver.getAttribute('data-version'));
    },

    showToast(message, type) {
      this.toast = message;
      this.toastType = type || 'success';
      this.toastVisible = true;
      setTimeout(function() { this.toastVisible = false; }.bind(this), 4000);
    },

    init() {
      // Load initial translations from inlined data
      if (window.__i18n) {
        this.t = window.__i18n;
      }

      // D-12: Watch OS theme changes in real-time
      window.matchMedia('(prefers-color-scheme: dark)')
        .addEventListener('change', function(e) { this.theme = e.matches ? 'dark' : 'light'; }.bind(this));

      // Listen for HTMX responses to show toast banners
      var self = this;
      document.body.addEventListener('htmx:afterSwap', function(evt) {
        var target = evt.detail.target;
        var html = target.innerHTML;
        var id = target.id || '';
        var isError = html.indexOf('text-red-400') !== -1 || html.indexOf('border-red-500') !== -1;
        var isSuccess = html.indexOf('text-emerald-400') !== -1 || html.indexOf('border-emerald-500') !== -1;

        if (isError) {
          var errDiv = target.querySelector('div');
          var msg = errDiv ? errDiv.textContent.trim() : '';
          self.showToast(msg || self.t['status.error'] || 'An error occurred', 'error');
        } else if (isSuccess) {
          // Detect which action completed
          if (id === 'tokenize-result') {
            self.showToast(self.t['status.tokenized'] || 'Tokenized successfully', 'success');
          } else if (id === 'detokenize-result') {
            self.showToast(self.t['status.detokenized'] || 'Detokenized successfully', 'success');
          } else if (id === 'config-result') {
            self.showToast(self.t['status.config_saved'] || 'Configuration saved', 'success');
          } else {
            self.showToast('Completed successfully', 'success');
          }
        }
      });

      // D-22: WebSocket heartbeat for auto-stop
      this.startHeartbeat();
    },

    startHeartbeat() {
      var protocol = location.protocol === 'https:' ? 'wss:' : 'ws:';
      var ws = new WebSocket(protocol + '//' + location.host + '/ws/heartbeat');
      var interval = setInterval(function() {
        if (ws.readyState === WebSocket.OPEN) {
          ws.send('ping');
        }
      }, 5000);
      ws.onclose = function() {
        clearInterval(interval);
      };
      ws.onerror = function() {
        clearInterval(interval);
      };
    }
  };
}
