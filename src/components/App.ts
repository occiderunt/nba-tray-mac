import { invoke } from '@tauri-apps/api/core';

interface FollowedTeam {
  id: string;
  abbreviation: string;
  display_name: string;
  logo: string;
}

interface TeamSearchResult {
  id: string;
  abbreviation: string;
  display_name: string;
  short_name: string;
  logo: string;
  color: string;
  alternate_color: string;
}

let followedTeams: FollowedTeam[] = [];
let searchResults: TeamSearchResult[] = [];
let searchDebounce: ReturnType<typeof setTimeout> | null = null;

const SHELL = `
  <div class="app-container">
    <div class="header">
      <div class="header-text">
        <h1>NBA Tracker</h1>
        <span class="version">Menubar scores</span>
      </div>
      <button id="close-settings" class="close-btn" title="Close (Esc)">✕</button>
    </div>

    <div class="search-section">
      <div class="search-wrapper">
        <span class="search-icon">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round">
            <circle cx="11" cy="11" r="7"></circle>
            <line x1="16.5" y1="16.5" x2="21" y2="21"></line>
          </svg>
        </span>
        <input
          id="team-search"
          class="search-input"
          type="text"
          placeholder="Add a team..."
          autocomplete="off"
          autocorrect="off"
          spellcheck="false"
        />
      </div>
      <div id="search-results" class="search-results" hidden></div>
    </div>

    <div class="followed-teams-section">
      <div class="section-header">
        <h2>Following</h2>
        <span class="team-count" id="team-count">0</span>
      </div>
      <div id="followed-teams-list"></div>
    </div>

    <div class="footer-note">Live games refresh every 30 seconds.</div>
  </div>
`;

function el<K extends keyof HTMLElementTagNameMap>(
  tag: K,
  className?: string,
  text?: string
): HTMLElementTagNameMap[K] {
  const node = document.createElement(tag);
  if (className) node.className = className;
  if (text !== undefined) node.textContent = text;
  return node;
}

function logo(src: string, alt: string): HTMLImageElement {
  const img = el('img', 'team-logo');
  img.alt = alt;
  img.src = src;
  img.loading = 'lazy';
  img.addEventListener('error', () => {
    img.style.visibility = 'hidden';
  });
  return img;
}

function renderFollowed() {
  const list = document.getElementById('followed-teams-list');
  const count = document.getElementById('team-count');
  if (!list || !count) return;

  count.textContent = String(followedTeams.length);
  list.replaceChildren();

  if (followedTeams.length === 0) {
    list.appendChild(
      el('div', 'empty-state', 'No teams yet. Search above and hit Add.')
    );
    return;
  }

  for (const team of followedTeams) {
    const row = el('div', 'team-item');
    row.dataset.teamId = team.id;

    const info = el('div', 'team-info');
    info.append(
      el('span', 'team-name', team.display_name),
      el('span', 'team-abbr', team.abbreviation)
    );

    const remove = el('button', 'remove-btn', '✕');
    remove.title = `Stop following ${team.display_name}`;
    remove.dataset.teamId = team.id;

    row.append(logo(team.logo, team.abbreviation), info, remove);
    list.appendChild(row);
  }
}

function renderSearch() {
  const results = document.getElementById('search-results');
  if (!results) return;

  results.replaceChildren();

  if (searchResults.length === 0) {
    results.hidden = true;
    return;
  }

  results.hidden = false;

  for (const team of searchResults) {
    const isFollowed = followedTeams.some((t) => t.id === team.id);
    const row = el('div', `search-result-item${isFollowed ? ' followed' : ''}`);

    const info = el('div', 'team-info');
    info.append(
      el('span', 'team-name', team.display_name),
      el('span', 'team-abbr', team.abbreviation)
    );

    const button = el(
      'button',
      `add-btn${isFollowed ? ' added' : ''}`,
      isFollowed ? 'Added' : 'Add'
    );
    (button as HTMLButtonElement).disabled = isFollowed;
    button.dataset.teamId = team.id;

    row.append(logo(team.logo, team.abbreviation), info, button);
    results.appendChild(row);
  }
}

async function loadFollowed() {
  try {
    followedTeams = await invoke<FollowedTeam[]>('get_followed_teams');
  } catch (error) {
    console.error('Failed to load teams', error);
    followedTeams = [];
  }
  renderFollowed();
}

async function search(query: string) {
  if (!query.trim()) {
    searchResults = [];
    renderSearch();
    return;
  }

  try {
    searchResults = await invoke<TeamSearchResult[]>('search_teams', { query });
  } catch (error) {
    console.error('Search failed', error);
    searchResults = [];
  }
  renderSearch();
}

async function addTeam(id: string) {
  const team = searchResults.find((t) => t.id === id);
  if (!team || followedTeams.some((t) => t.id === id)) return;

  const followed: FollowedTeam = {
    id: team.id,
    abbreviation: team.abbreviation,
    display_name: team.display_name,
    logo: team.logo,
  };

  try {
    await invoke('add_team', { team: followed });
    followedTeams.push(followed);
    renderFollowed();
    renderSearch();
  } catch (error) {
    console.error('Failed to add team', error);
  }
}

async function removeTeam(id: string) {
  try {
    await invoke('remove_team', { teamId: id });
    followedTeams = followedTeams.filter((t) => t.id !== id);
    renderFollowed();
    renderSearch();
  } catch (error) {
    console.error('Failed to remove team', error);
  }
}

function wireEvents() {
  const input = document.getElementById('team-search') as HTMLInputElement | null;
  const results = document.getElementById('search-results');
  const list = document.getElementById('followed-teams-list');
  const close = document.getElementById('close-settings');

  input?.addEventListener('input', () => {
    if (searchDebounce) clearTimeout(searchDebounce);
    const value = input.value;
    searchDebounce = setTimeout(() => void search(value), 220);
  });

  input?.addEventListener('keydown', (event) => {
    if (event.key === 'Escape') {
      input.value = '';
      searchResults = [];
      renderSearch();
      input.blur();
    }
  });

  results?.addEventListener('click', (event) => {
    const target = (event.target as HTMLElement).closest('.add-btn');
    if (!target || target.classList.contains('added')) return;
    const id = (target as HTMLElement).dataset.teamId;
    if (id) void addTeam(id);
  });

  list?.addEventListener('click', (event) => {
    const target = (event.target as HTMLElement).closest('.remove-btn');
    if (!target) return;
    const id = (target as HTMLElement).dataset.teamId;
    if (id) void removeTeam(id);
  });

  close?.addEventListener('click', () => {
    invoke('close_settings').catch(console.error);
  });

  window.addEventListener('keydown', (event) => {
    if (event.key === 'Escape') {
      invoke('close_settings').catch(console.error);
    }
  });
}

export async function initApp() {
  const root = document.getElementById('app');
  if (!root) return;

  root.innerHTML = SHELL;
  wireEvents();
  await loadFollowed();

  const input = document.getElementById('team-search') as HTMLInputElement | null;
  input?.focus();
}
