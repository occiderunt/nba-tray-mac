(function(){const t=document.createElement("link").relList;if(t&&t.supports&&t.supports("modulepreload"))return;for(const o of document.querySelectorAll('link[rel="modulepreload"]'))r(o);new MutationObserver(o=>{for(const s of o)if(s.type==="childList")for(const i of s.addedNodes)i.tagName==="LINK"&&i.rel==="modulepreload"&&r(i)}).observe(document,{childList:!0,subtree:!0});function n(o){const s={};return o.integrity&&(s.integrity=o.integrity),o.referrerPolicy&&(s.referrerPolicy=o.referrerPolicy),o.crossOrigin==="use-credentials"?s.credentials="include":o.crossOrigin==="anonymous"?s.credentials="omit":s.credentials="same-origin",s}function r(o){if(o.ep)return;o.ep=!0;const s=n(o);fetch(o.href,s)}})();async function l(e,t={},n){return window.__TAURI_INTERNALS__.invoke(e,t,n)}let d=[],c=[],m=null;const v=`
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
`;function a(e,t,n){const r=document.createElement(e);return t&&(r.className=t),n!==void 0&&(r.textContent=n),r}function p(e,t){const n=a("img","team-logo");return n.alt=t,n.src=e,n.loading="lazy",n.addEventListener("error",()=>{n.style.visibility="hidden"}),n}function f(){const e=document.getElementById("followed-teams-list"),t=document.getElementById("team-count");if(!(!e||!t)){if(t.textContent=String(d.length),e.replaceChildren(),d.length===0){e.appendChild(a("div","empty-state","No teams yet. Search above and hit Add."));return}for(const n of d){const r=a("div","team-item");r.dataset.teamId=n.id;const o=a("div","team-info");o.append(a("span","team-name",n.display_name),a("span","team-abbr",n.abbreviation));const s=a("button","remove-btn","✕");s.title=`Stop following ${n.display_name}`,s.dataset.teamId=n.id,r.append(p(n.logo,n.abbreviation),o,s),e.appendChild(r)}}}function u(){const e=document.getElementById("search-results");if(e){if(e.replaceChildren(),c.length===0){e.hidden=!0;return}e.hidden=!1;for(const t of c){const n=d.some(i=>i.id===t.id),r=a("div",`search-result-item${n?" followed":""}`),o=a("div","team-info");o.append(a("span","team-name",t.display_name),a("span","team-abbr",t.abbreviation));const s=a("button",`add-btn${n?" added":""}`,n?"Added":"Add");s.disabled=n,s.dataset.teamId=t.id,r.append(p(t.logo,t.abbreviation),o,s),e.appendChild(r)}}}async function h(){try{d=await l("get_followed_teams")}catch(e){console.error("Failed to load teams",e),d=[]}f()}async function y(e){if(!e.trim()){c=[],u();return}try{c=await l("search_teams",{query:e})}catch(t){console.error("Search failed",t),c=[]}u()}async function g(e){const t=c.find(r=>r.id===e);if(!t||d.some(r=>r.id===e))return;const n={id:t.id,abbreviation:t.abbreviation,display_name:t.display_name,logo:t.logo};try{await l("add_team",{team:n}),d.push(n),f(),u()}catch(r){console.error("Failed to add team",r)}}async function b(e){try{await l("remove_team",{teamId:e}),d=d.filter(t=>t.id!==e),f(),u()}catch(t){console.error("Failed to remove team",t)}}function w(){const e=document.getElementById("team-search"),t=document.getElementById("search-results"),n=document.getElementById("followed-teams-list"),r=document.getElementById("close-settings");e?.addEventListener("input",()=>{m&&clearTimeout(m);const o=e.value;m=setTimeout(()=>void y(o),220)}),e?.addEventListener("keydown",o=>{o.key==="Escape"&&(e.value="",c=[],u(),e.blur())}),t?.addEventListener("click",o=>{const s=o.target.closest(".add-btn");if(!s||s.classList.contains("added"))return;const i=s.dataset.teamId;i&&g(i)}),n?.addEventListener("click",o=>{const s=o.target.closest(".remove-btn");if(!s)return;const i=s.dataset.teamId;i&&b(i)}),r?.addEventListener("click",()=>{l("close_settings").catch(console.error)}),window.addEventListener("keydown",o=>{o.key==="Escape"&&l("close_settings").catch(console.error)})}async function E(){const e=document.getElementById("app");if(!e)return;e.innerHTML=v,w(),await h(),document.getElementById("team-search")?.focus()}document.addEventListener("DOMContentLoaded",()=>{E()});
