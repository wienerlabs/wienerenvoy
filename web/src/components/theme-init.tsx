// Server component that emits an inline script before hydration so the theme
// attribute is on <html> before first paint. Avoids a flash of the wrong theme.

const SCRIPT = `
(function() {
  try {
    var stored = localStorage.getItem('wienerenvoy:theme');
    var sys = window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
    var resolved = (stored === 'light' || stored === 'dark') ? stored : sys;
    document.documentElement.setAttribute('data-theme', resolved);
    if (stored === 'system' || stored === null) {
      document.documentElement.setAttribute('data-theme-pref', 'system');
    } else {
      document.documentElement.setAttribute('data-theme-pref', stored);
    }
  } catch (e) {
    document.documentElement.setAttribute('data-theme', 'dark');
  }
})();
`.trim();

export function ThemeInit() {
  return <script dangerouslySetInnerHTML={{ __html: SCRIPT }} />;
}
