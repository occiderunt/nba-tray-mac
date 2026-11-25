import { chromium } from '/Users/family/node_modules/playwright/index.mjs';
import { spawn } from 'node:child_process';
import { mkdirSync } from 'node:fs';
import { setTimeout as sleep } from 'node:timers/promises';

const OUT = new URL('../qa/', import.meta.url).pathname;
mkdirSync(OUT, { recursive: true });

const TEAMS = [
  { id: '2', abbreviation: 'BOS', display_name: 'Boston Celtics', logo: 'https://a.espncdn.com/i/teamlogos/nba/500/bos.png' },
];

const SEARCH = [
  { id: '13', abbreviation: 'LAL', display_name: 'Los Angeles Lakers', short_name: 'Lakers', logo: 'https://a.espncdn.com/i/teamlogos/nba/500/lal.png', color: '552583', alternate_color: 'fdb927' },
  { id: '17', abbreviation: 'BKN', display_name: 'Brooklyn Nets', short_name: 'Nets', logo: 'https://a.espncdn.com/i/teamlogos/nba/500/bkn.png', color: '000000', alternate_color: 'FFFFFF' },
  { id: '2', abbreviation: 'BOS', display_name: 'Boston Celtics', short_name: 'Celtics', logo: 'https://a.espncdn.com/i/teamlogos/nba/500/bos.png', color: '007a33', alternate_color: 'ffffff' },
];

const server = spawn('npx', ['vite', 'preview', '--port', '4173', '--strictPort'], {
  cwd: new URL('..', import.meta.url).pathname,
  stdio: 'ignore',
});
await sleep(1500);

const browser = await chromium.launch({ channel: 'chrome' });
const page = await browser.newPage({ viewport: { width: 380, height: 560 }, deviceScaleFactor: 2 });

let followed = [...TEAMS];
const calls = [];

await page.addInitScript(() => {
  window.__MOCK__ = true;
  window.__TAURI_INTERNALS__ = {
    invoke: async (cmd, args) => {
      const state = window.__STATE__;
      state.calls.push({ cmd, args });
      switch (cmd) {
        case 'get_followed_teams':
          return JSON.parse(JSON.stringify(state.followed));
        case 'search_teams': {
          const q = (args?.query ?? '').toLowerCase();
          return state.search
            .filter(
              (t) => t.display_name.toLowerCase().includes(q) || t.abbreviation.toLowerCase().includes(q)
            )
            .map((t) => JSON.parse(JSON.stringify(t)));
        }
        case 'add_team':
          state.followed.push(JSON.parse(JSON.stringify(args.team)));
          return null;
        case 'remove_team':
          state.followed = state.followed.filter((t) => t.id !== args.teamId);
          return null;
        default:
          return null;
      }
    },
  };
});

await page.addInitScript(
  ([followed, search]) => {
    window.__STATE__ = { followed, search, calls: [] };
  },
  [followed, SEARCH]
);

await page.goto('http://localhost:4173/', { waitUntil: 'networkidle' });
await page.waitForSelector('#followed-teams-list');

const checks = [];
const check = (name, ok, extra = '') => checks.push({ name, ok, extra });

check('shell renders', (await page.locator('.app-container').count()) === 1);
check('title present', (await page.locator('h1').innerText()) === 'NBA Tracker');
check('one followed team rendered', (await page.locator('#followed-teams-list .team-item').count()) === 1);
check('team count badge', (await page.locator('#team-count').innerText()) === '1');

await page.screenshot({ path: `${OUT}settings-initial.png` });

// search
await page.fill('#team-search', 'b');
await page.waitForSelector('#search-results .search-result-item');
const rows = await page.locator('#search-results .search-result-item').count();
check('search returns results', rows === 2, `rows=${rows}`);
check('already-followed marked Added', (await page.locator('#search-results .add-btn.added').count()) === 1);

await page.screenshot({ path: `${OUT}settings-search.png` });

// add Lakers
await page.fill('#team-search', 'lal');
await page.waitForSelector('#search-results .search-result-item');
await page.locator('#search-results .search-result-item', { hasText: 'Los Angeles Lakers' }).locator('.add-btn').click();
await sleep(200);
check('add button adds team', (await page.locator('#followed-teams-list .team-item').count()) === 2);
check('badge updates', (await page.locator('#team-count').innerText()) === '2');
check('add_team called', (await page.evaluate(() => window.__STATE__.calls.filter((c) => c.cmd === 'add_team').length)) === 1);

await page.screenshot({ path: `${OUT}settings-added.png` });

// remove Celtics
await page.locator('#followed-teams-list .team-item', { hasText: 'Boston' }).locator('.remove-btn').click();
await sleep(200);
check('remove works', (await page.locator('#followed-teams-list .team-item').count()) === 1);
check('remove_team called', (await page.evaluate(() => window.__STATE__.calls.filter((c) => c.cmd === 'remove_team').length)) === 1);

// empty query hides panel
await page.fill('#team-search', '');
await sleep(300);
check('empty query hides results', await page.locator('#search-results').isHidden());

// close button calls backend
await page.click('#close-settings');
await sleep(100);
check('close invokes backend', (await page.evaluate(() => window.__STATE__.calls.some((c) => c.cmd === 'close_settings'))));

let failed = 0;
for (const c of checks) {
  if (!c.ok) failed++;
  console.log(`${c.ok ? 'ok  ' : 'FAIL'} ${c.name}${c.extra ? ` (${c.extra})` : ''}`);
}
console.log(`\n${checks.length - failed}/${checks.length} checks passed`);

await browser.close();
server.kill();
process.exit(failed ? 1 : 0);
