// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// tokless dashboard client application - pure vanilla ES6+ with zero external npm dependencies

class ToklessApp {
  constructor() {
    this.state = {
      overview: {
        proxy_up: true,
        daemon_up: true,
        tokens_saved_lifetime: 14892104,
        tokens_saved_today: 482300,
        tokens_saved_7d: 3120500,
        cost_saved_usd: 184.26,
        total_calls: 1284,
        compression_pct: 68.4,
        cache_warmth_pct: 91.2,
      },
      waterfall: [],
      agents: [],
      memory: {
        items: [],
        dag_nodes: [],
        dag_links: [],
        heatmap: [],
      },
      codegraph: {
        symbols: [],
        selected_symbol: null,
        blast_depth: 3,
      },
      byok_routes: [],
      sse_connected: false,
    };

    this.presets = {
      cargo: `   Compiling tokless v0.4.0 (D:\\AI\\neo\\crates\\tokless)
   Compiling syn v2.0.98
   Compiling quote v1.0.38
warning: unused variable: \`x\`
  --> crates\\tokless\\src\\temp.rs:14:9
   |
14 |     let x = 42;
   |         ^ help: if this is intentional, prefix it with an underscore: \`_x\`
   |
   = note: \`#[warn(unused_variables)]\` on by default

test commands::doctor::tests::test_doctor_probe ... ok
test commands::init::tests::test_init_all ... ok
test headroom::cache_align::tests::test_1024_alignment ... ok
test projectmem::fts::tests::test_fts5_search ... ok
test rtk::filter::tests::test_ansi_strip ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.42s`,

      git: `On branch main
Your branch is up to date with 'origin/main'.

Changes not staged for commit:
  (use "git add <file>..." to update what will be committed)
  (use "git restore <file>..." to discard changes in working directory)
	modified:   crates/tokless/CONSOLIDATION_PLAN.md
	modified:   crates/tokless/src/main.rs

Untracked files:
  (use "git add <file>..." to include in what will be committed)
	crates/tokless/src/dashboard/

no changes added to commit (use "git add" to track)`,

      npm: `npm warn deprecated rimraf@2.6.3: Rimraf versions prior to v4 are no longer supported
npm warn deprecated inflight@1.0.6: This module is not supported, and leaks memory.
npm warn deprecated glob@7.2.3: Glob versions prior to v9 are no longer supported
added 842 packages from 612 contributors and audited 845 packages in 14.821s
112 packages are looking for funding
  run \`npm fund\` for details
found 0 vulnerabilities`,

      json: `{\n  "status": "success",\n  "data": [\n    {"id": 1, "name": "Item A", "details": "Unused debug metadata in payload", "timestamp": "2026-10-30T12:00:00Z"},\n    {"id": 2, "name": "Item B", "details": "Redundant nested dictionary fields", "timestamp": "2026-10-30T12:01:00Z"}\n  ],\n  "paging": {"page": 1, "total": 1000, "has_more": true}\n}`
    };
  }

  init() {
    this.setupTabs();
    this.connectSSE();
    this.loadOverview();
    this.loadWaterfall();
    this.loadAgents();
    this.loadMemory();
    this.loadCodeGraph();
    this.loadByokRoutes();
    this.applySandboxPreset('cargo');

    // Polling fallback if SSE drops
    setInterval(() => {
      if (!this.state.sse_connected) {
        this.loadOverview();
      }
    }, 5000);
  }

  // --- Tabs Navigation ---
  setupTabs() {
    document.querySelectorAll('.tab-btn').forEach(btn => {
      btn.addEventListener('click', () => {
        const tabId = btn.getAttribute('data-tab');
        document.querySelectorAll('.tab-btn').forEach(b => b.classList.remove('active'));
        document.querySelectorAll('.tab-pane').forEach(p => p.classList.remove('active'));
        btn.classList.add('active');
        const target = document.getElementById(tabId);
        if (target) target.classList.add('active');
      });
    });
  }

  // --- SSE Real-time Stream ---
  connectSSE() {
    const ssePill = document.getElementById('sse-pill');
    const sseText = document.getElementById('sse-text');

    try {
      const evtSource = new EventSource('/api/events');
      evtSource.onopen = () => {
        this.state.sse_connected = true;
        if (ssePill) {
          ssePill.className = 'status-pill online';
          sseText.textContent = 'LIVE SSE (9099)';
        }
      };

      evtSource.onmessage = (e) => {
        try {
          const data = JSON.parse(e.data);
          if (data.type === 'tick') {
            this.handleLiveTick(data.payload);
          } else if (data.type === 'waterfall_item') {
            this.prependWaterfallItem(data.payload);
          }
        } catch (err) {
          console.error('SSE parse error:', err);
        }
      };

      evtSource.onerror = () => {
        this.state.sse_connected = false;
        if (ssePill) {
          ssePill.className = 'status-pill offline';
          sseText.textContent = 'RECONNECTING...';
        }
      };
    } catch (e) {
      console.warn('EventSource unsupported or blocked:', e);
    }
  }

  handleLiveTick(payload) {
    if (payload.tokens_saved_lifetime) {
      this.state.overview = { ...this.state.overview, ...payload };
      this.renderOverview();
    }
  }

  // --- API Calls & Data Loaders ---
  async loadOverview() {
    try {
      const res = await fetch('/api/overview');
      if (res.ok) {
        const data = await res.json();
        this.state.overview = data;
        this.renderOverview();
      }
    } catch (e) {
      console.error('loadOverview failed:', e);
    }
  }

  renderOverview() {
    const o = this.state.overview;
    const fmt = n => Number(n).toLocaleString();

    document.getElementById('stat-tokens-lifetime').textContent = fmt(o.tokens_saved_lifetime || 0);
    document.getElementById('stat-tokens-today').textContent = `+${fmt(o.tokens_saved_today || 0)} today (7d: ${fmt(o.tokens_saved_7d || 0)})`;
    document.getElementById('stat-cost-saved').textContent = `$${Number(o.cost_saved_usd || 0).toFixed(2)}`;
    document.getElementById('hero-usd-val').textContent = `$${Number(o.cost_saved_usd || 0).toFixed(2)}`;
    document.getElementById('stat-compression-pct').textContent = `${Number(o.compression_pct || 0).toFixed(1)}%`;
    document.getElementById('stat-total-calls').textContent = `${fmt(o.total_calls || 0)} calls processed`;
    document.getElementById('stat-cache-warmth').textContent = `${Number(o.cache_warmth_pct || 0).toFixed(1)}%`;

    const proxyPill = document.getElementById('proxy-pill');
    const proxyText = document.getElementById('proxy-text');
    const sysBadge = document.getElementById('sys-proxy-badge');
    if (o.proxy_up) {
      if (proxyPill) proxyPill.className = 'status-pill online';
      if (proxyText) proxyText.textContent = 'PROXY :8787';
      if (sysBadge) { sysBadge.className = 'badge badge-green'; sysBadge.textContent = 'RUNNING'; }
    } else {
      if (proxyPill) proxyPill.className = 'status-pill offline';
      if (proxyText) proxyText.textContent = 'PROXY DOWN';
      if (sysBadge) { sysBadge.className = 'badge badge-red'; sysBadge.textContent = 'STOPPED'; }
    }
  }

  async loadWaterfall() {
    try {
      const res = await fetch('/api/waterfall');
      if (res.ok) {
        const data = await res.json();
        this.state.waterfall = data.items || [];
        this.renderWaterfall();
        if (data.layers) {
          this.renderLayers(data.layers);
        }
      }
    } catch (e) {
      console.error('loadWaterfall error:', e);
    }
  }

  renderLayers(layers) {
    if (layers.rtk) {
      document.getElementById('layer-rtk-share').textContent = `${layers.rtk.share}% Share`;
      document.getElementById('layer-rtk-saved').textContent = `Saved: ${Number(layers.rtk.saved).toLocaleString()} tokens`;
    }
    if (layers.headroom) {
      document.getElementById('layer-headroom-share').textContent = `${layers.headroom.share}% Share`;
      document.getElementById('layer-headroom-saved').textContent = `Saved: ${Number(layers.headroom.saved).toLocaleString()} tokens`;
    }
    if (layers.caveman) {
      document.getElementById('layer-caveman-share').textContent = `${layers.caveman.share}% Share`;
      document.getElementById('layer-caveman-saved').textContent = `Saved: ${Number(layers.caveman.saved).toLocaleString()} tokens`;
    }
  }

  renderWaterfall() {
    const tbody = document.getElementById('waterfall-tbody');
    if (!tbody) return;
    tbody.innerHTML = '';

    this.state.waterfall.forEach(item => {
      const tr = document.createElement('tr');
      tr.innerHTML = `
        <td style="font-family: var(--font-mono); color: var(--text-muted);">${item.time}</td>
        <td><strong>${item.agent}</strong></td>
        <td style="font-family: var(--font-mono); font-size: 0.8rem; color: var(--text-secondary);">${item.endpoint}</td>
        <td style="font-family: var(--font-mono);">${Number(item.raw_tokens).toLocaleString()}</td>
        <td style="font-family: var(--font-mono); color: var(--accent-cyan);">${Number(item.compressed_tokens).toLocaleString()}</td>
        <td><span class="badge badge-green">${item.savings_pct.toFixed(1)}%</span></td>
        <td style="font-size: 0.75rem; color: var(--text-secondary);">${item.pipeline}</td>
        <td style="font-family: var(--font-mono); font-weight: 600; color: #34d399;">+$${item.saved_usd.toFixed(3)}</td>
      `;
      tbody.appendChild(tr);
    });
  }

  prependWaterfallItem(item) {
    this.state.waterfall.unshift(item);
    if (this.state.waterfall.length > 50) this.state.waterfall.pop();
    this.renderWaterfall();
  }

  refreshWaterfall() {
    this.loadWaterfall();
    this.showToast('Waterfall stream refreshed', 'success');
  }

  // --- Agents Fleet ---
  async loadAgents() {
    try {
      const res = await fetch('/api/agents');
      if (res.ok) {
        this.state.agents = await res.json();
        this.renderAgents();
      }
    } catch (e) {
      console.error('loadAgents error:', e);
    }
  }

  renderAgents() {
    const grid = document.getElementById('agent-fleet-grid');
    if (!grid) return;
    grid.innerHTML = '';

    this.state.agents.forEach(ag => {
      const card = document.createElement('div');
      card.className = `agent-card ${ag.wired ? 'wired' : 'unwired'}`;
      card.innerHTML = `
        <div class="agent-head">
          <div>
            <div class="agent-name">${ag.label}</div>
            <div class="agent-id">ID: ${ag.id}</div>
          </div>
          <span class="badge ${ag.installed ? 'badge-cyan' : 'badge-gray'}">
            ${ag.installed ? (ag.source || 'INSTALLED') : 'NOT DETECTED'}
          </span>
        </div>
        <div class="agent-details">
          <div>Port: <strong style="color: ${ag.wired ? 'var(--accent-green)' : 'var(--text-muted)'};">${ag.port}</strong></div>
          <div>Tools: ${ag.tools_summary || 'RTK, CodeGraph, ProjectMem'}</div>
          <div style="font-family: var(--font-mono); font-size: 0.7rem; color: var(--text-muted); overflow: hidden; text-overflow: ellipsis; white-space: nowrap;" title="${ag.config_path}">
            Cfg: ${ag.config_path}
          </div>
        </div>
        <div class="agent-action">
          <span class="badge ${ag.wired ? 'badge-green' : 'badge-amber'}">${ag.wired ? 'WIRED' : 'UNWIRED'}</span>
          <button class="btn ${ag.wired ? 'btn-danger' : 'btn-success'}" onclick="app.toggleAgent('${ag.id}')">
            ${ag.wired ? 'Unwire' : 'Wire Agent'}
          </button>
        </div>
      `;
      grid.appendChild(card);
    });
  }

  async toggleAgent(agentId) {
    try {
      const res = await fetch('/api/agents/toggle', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ agent_id: agentId })
      });
      if (res.ok) {
        const result = await res.json();
        this.showToast(`Agent ${agentId} ${result.wired ? 'wired' : 'unwired'} successfully`, 'success');
        this.loadAgents();
      } else {
        this.showToast(`Failed to toggle agent ${agentId}`, 'error');
      }
    } catch (e) {
      this.showToast(`Error: ${e.message}`, 'error');
    }
  }

  async wireAllAgents() {
    for (const ag of this.state.agents) {
      if (ag.installed && !ag.wired) {
        await this.toggleAgent(ag.id);
      }
    }
    this.showToast('All installed agents wired', 'success');
  }

  async unwireAllAgents() {
    for (const ag of this.state.agents) {
      if (ag.wired) {
        await this.toggleAgent(ag.id);
      }
    }
    this.showToast('All agents unwired', 'success');
  }

  // --- Memory Decision DAG ---
  async loadMemory(query = '') {
    try {
      const url = query ? `/api/memory?q=${encodeURIComponent(query)}` : '/api/memory';
      const res = await fetch(url);
      if (res.ok) {
        this.state.memory = await res.json();
        this.renderMemoryDAG();
        this.renderHeatmap();
      }
    } catch (e) {
      console.error('loadMemory error:', e);
    }
  }

  searchMemory(q) {
    clearTimeout(this._searchTimer);
    this._searchTimer = setTimeout(() => {
      this.loadMemory(q);
    }, 250);
  }

  renderMemoryDAG() {
    const svg = document.getElementById('dag-svg');
    if (!svg) return;
    svg.innerHTML = '';

    const nodes = this.state.memory.dag_nodes || [];
    const links = this.state.memory.dag_links || [];

    // SVG definitions for arrow markers
    const defs = document.createElementNS('http://www.w3.org/2000/svg', 'defs');
    defs.innerHTML = `
      <marker id="arrow" viewBox="0 0 10 10" refX="22" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse">
        <path d="M 0 0 L 10 5 L 0 10 z" fill="#06b6d4" />
      </marker>
    `;
    svg.appendChild(defs);

    // Draw links
    links.forEach(link => {
      const source = nodes.find(n => n.id === link.source);
      const target = nodes.find(n => n.id === link.target);
      if (source && target) {
        const line = document.createElementNS('http://www.w3.org/2000/svg', 'line');
        line.setAttribute('x1', source.x);
        line.setAttribute('y1', source.y);
        line.setAttribute('x2', target.x);
        line.setAttribute('y2', target.y);
        line.setAttribute('class', 'dag-link');
        line.setAttribute('marker-end', 'url(#arrow)');
        svg.appendChild(line);
      }
    });

    // Draw nodes
    nodes.forEach(node => {
      const g = document.createElementNS('http://www.w3.org/2000/svg', 'g');
      g.setAttribute('class', 'dag-node');
      g.setAttribute('transform', `translate(${node.x}, ${node.y})`);

      const circle = document.createElementNS('http://www.w3.org/2000/svg', 'circle');
      circle.setAttribute('r', '18');
      circle.setAttribute('fill', node.type === 'decision' ? '#06b6d4' : (node.type === 'fix' ? '#10b981' : '#f59e0b'));
      circle.setAttribute('stroke', '#0d1322');
      circle.setAttribute('stroke-width', '3');
      g.appendChild(circle);

      const idText = document.createElementNS('http://www.w3.org/2000/svg', 'text');
      idText.setAttribute('text-anchor', 'middle');
      idText.setAttribute('dy', '4');
      idText.setAttribute('fill', '#090d16');
      idText.setAttribute('font-size', '10');
      idText.setAttribute('font-weight', 'bold');
      idText.textContent = node.id.slice(0, 4);
      g.appendChild(idText);

      const label = document.createElementNS('http://www.w3.org/2000/svg', 'text');
      label.setAttribute('x', '26');
      label.setAttribute('y', '4');
      label.setAttribute('fill', '#f8fafc');
      label.setAttribute('font-size', '12');
      label.setAttribute('font-family', 'var(--font-sans)');
      label.textContent = node.summary.length > 40 ? node.summary.slice(0, 40) + '...' : node.summary;
      g.appendChild(label);

      g.addEventListener('click', () => {
        this.showToast(`[${node.type.toUpperCase()}] ${node.summary} (${node.location})`, 'info');
      });

      svg.appendChild(g);
    });
  }

  renderHeatmap() {
    const grid = document.getElementById('heatmap-grid');
    if (!grid) return;
    grid.innerHTML = '';

    const items = this.state.memory.heatmap || [];
    items.forEach(h => {
      const cell = document.createElement('div');
      cell.className = `heatmap-cell ${h.level}`;
      cell.innerHTML = `
        <div style="font-weight: 700; font-size: 0.85rem;">${h.component}</div>
        <div style="font-size: 0.75rem; color: var(--text-muted); margin-top: 0.25rem;">
          ${h.issues} bugs / ${h.fixes} fixes
        </div>
      `;
      grid.appendChild(cell);
    });
  }

  // --- CodeGraph Explorer ---
  async loadCodeGraph() {
    try {
      const res = await fetch('/api/codegraph');
      if (res.ok) {
        this.state.codegraph = await res.json();
        this.renderCodeGraphSymbols();
      }
    } catch (e) {
      console.error('loadCodeGraph error:', e);
    }
  }

  renderCodeGraphSymbols() {
    const tbody = document.getElementById('codegraph-symbols-tbody');
    if (!tbody) return;
    tbody.innerHTML = '';

    const syms = this.state.codegraph.symbols || [];
    syms.forEach(sym => {
      const tr = document.createElement('tr');
      tr.innerHTML = `
        <td style="font-family: var(--font-mono); font-weight: 600; color: var(--accent-cyan);">${sym.name}</td>
        <td><span class="badge badge-purple">${sym.kind}</span></td>
        <td style="font-size: 0.75rem; color: var(--text-muted);">${sym.file}:${sym.line}</td>
        <td>
          <button class="btn btn-primary" style="padding: 2px 8px; font-size: 0.7rem;" onclick="app.simulateBlastRadius('${sym.id}')">
            Simulate
          </button>
        </td>
      `;
      tbody.appendChild(tr);
    });
  }

  updateBlastDepth(val) {
    this.state.codegraph.blast_depth = parseInt(val, 10);
    document.getElementById('blast-depth-val').textContent = val;
    if (this.state.codegraph.selected_symbol) {
      this.simulateBlastRadius(this.state.codegraph.selected_symbol);
    }
  }

  async simulateBlastRadius(symId) {
    this.state.codegraph.selected_symbol = symId;
    const depth = this.state.codegraph.blast_depth;
    try {
      const res = await fetch(`/api/codegraph/blast?symbol=${encodeURIComponent(symId)}&depth=${depth}`);
      if (res.ok) {
        const report = await res.json();
        this.renderBlastReport(report);
      }
    } catch (e) {
      console.error('simulateBlastRadius error:', e);
    }
  }

  renderBlastReport(report) {
    const container = document.getElementById('blast-radius-details');
    if (!container) return;

    container.innerHTML = `
      <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 0.75rem;">
        <div>
          <span style="font-size: 0.75rem; text-transform: uppercase; color: var(--text-muted);">Focal Symbol:</span>
          <strong style="color: var(--accent-cyan); font-family: var(--font-mono); font-size: 1.1rem; margin-left: 0.5rem;">${report.focal_name}</strong>
        </div>
        <span class="badge badge-amber">Impacted Files: ${report.affected_files ? report.affected_files.length : 0}</span>
      </div>

      <div style="font-size: 0.8rem; color: var(--text-secondary); margin-bottom: 0.75rem;">
        Defined in: <code>${report.focal_file}</code> (Depth ${report.depth})
      </div>

      <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 1rem; margin-bottom: 1rem;">
        <div style="background: #090d16; padding: 0.75rem; border-radius: 4px; border: 1px solid var(--border-color);">
          <div style="font-size: 0.75rem; font-weight: 600; color: var(--accent-green); margin-bottom: 0.35rem;">
            Upstream Callers (${report.upstream_callers ? report.upstream_callers.length : 0})
          </div>
          <ul style="list-style: none; font-size: 0.75rem; font-family: var(--font-mono);">
            ${(report.upstream_callers || []).map(c => `<li>• ${c.name} (${c.file})</li>`).join('') || '<li style="color: var(--text-muted);">None</li>'}
          </ul>
        </div>

        <div style="background: #090d16; padding: 0.75rem; border-radius: 4px; border: 1px solid var(--border-color);">
          <div style="font-size: 0.75rem; font-weight: 600; color: var(--accent-purple); margin-bottom: 0.35rem;">
            Downstream Callees (${report.downstream_callees ? report.downstream_callees.length : 0})
          </div>
          <ul style="list-style: none; font-size: 0.75rem; font-family: var(--font-mono);">
            ${(report.downstream_callees || []).map(c => `<li>• ${c.name} (${c.file})</li>`).join('') || '<li style="color: var(--text-muted);">None</li>'}
          </ul>
        </div>
      </div>

      <div style="font-size: 0.8rem; font-weight: 600; margin-bottom: 0.25rem;">Affected Test Suites:</div>
      <div style="display: flex; flex-wrap: wrap; gap: 0.4rem;">
        ${(report.affected_tests || ['crates/tokless/src/tests.rs']).map(t => `<span class="badge badge-red">${t}</span>`).join('')}
      </div>
    `;
  }

  // --- Context-Mode Sandbox Inspector ---
  applySandboxPreset(presetName) {
    const rawInput = document.getElementById('raw-output-input');
    if (rawInput && this.presets[presetName]) {
      rawInput.value = this.presets[presetName];
      this.runSandboxFilter();
    }
  }

  async runSandboxFilter() {
    const rawText = document.getElementById('raw-output-input').value;
    try {
      const res = await fetch('/api/contextmode/filter', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          raw_output: rawText,
          cmd: 'test-filter',
          max_lines: 50,
          max_bytes: 32768,
        })
      });

      if (res.ok) {
        const result = await res.json();
        document.getElementById('filtered-output-display').textContent = result.filtered;
        document.getElementById('raw-meta').textContent = `${result.raw_lines} lines (${result.raw_bytes} B)`;
        document.getElementById('filtered-meta').textContent = `${result.filtered_lines} lines (${result.filtered_bytes} B)`;
        document.getElementById('filter-savings-val').textContent = `${result.savings_pct.toFixed(1)}%`;
      }
    } catch (e) {
      console.error('runSandboxFilter error:', e);
    }
  }

  // --- System Controls & BYOK ---
  async systemProxyRestart() {
    try {
      const res = await fetch('/api/system/proxy-restart', { method: 'POST' });
      if (res.ok) {
        this.showToast('Headroom proxy restarted successfully', 'success');
        this.loadOverview();
      }
    } catch (e) {
      this.showToast('Proxy restart failed', 'error');
    }
  }

  async systemProxyStop() {
    try {
      const res = await fetch('/api/system/proxy-stop', { method: 'POST' });
      if (res.ok) {
        this.showToast('Headroom proxy stopped', 'success');
        this.loadOverview();
      }
    } catch (e) {
      this.showToast('Proxy stop failed', 'error');
    }
  }

  async systemCacheVacuum() {
    try {
      const res = await fetch('/api/system/cache-vacuum', { method: 'POST' });
      if (res.ok) {
        const result = await res.json();
        document.getElementById('vacuum-result').textContent = `Vacuum complete: ${result.message} (freed ${result.freed_kb} KB)`;
        this.showToast('Cache vacuum complete', 'success');
      }
    } catch (e) {
      this.showToast('Vacuum failed', 'error');
    }
  }

  async loadByokRoutes() {
    try {
      const res = await fetch('/api/system/byok');
      if (res.ok) {
        this.state.byok_routes = await res.json();
        this.renderByokRoutes();
      }
    } catch (e) {
      console.error('loadByokRoutes error:', e);
    }
  }

  renderByokRoutes() {
    const tbody = document.getElementById('byok-routes-tbody');
    if (!tbody) return;
    tbody.innerHTML = '';

    (this.state.byok_routes || []).forEach(r => {
      const tr = document.createElement('tr');
      tr.innerHTML = `
        <td style="font-family: var(--font-mono); font-weight: 600; color: var(--accent-cyan);">${r.id}</td>
        <td><span class="badge badge-purple">${r.protocol}</span></td>
        <td style="font-family: var(--font-mono); font-size: 0.8rem;">${r.upstream}</td>
        <td style="font-family: var(--font-mono); color: var(--text-muted);">${r.token.slice(0, 8)}...</td>
        <td>
          <button class="btn btn-danger" style="padding: 2px 8px; font-size: 0.7rem;" onclick="app.deleteByokRoute('${r.id}')">
            Remove
          </button>
        </td>
      `;
      tbody.appendChild(tr);
    });
  }

  async addByokRoute() {
    const id = document.getElementById('byok-new-id').value.trim();
    const protocol = document.getElementById('byok-new-proto').value;
    const upstream = document.getElementById('byok-new-upstream').value.trim();

    if (!id || !upstream) {
      this.showToast('Please enter Route ID and Upstream URL', 'error');
      return;
    }

    try {
      const res = await fetch('/api/system/byok-switch', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ id, protocol, upstream })
      });
      if (res.ok) {
        this.showToast(`BYOK route ${id} activated`, 'success');
        document.getElementById('byok-new-id').value = '';
        document.getElementById('byok-new-upstream').value = '';
        this.loadByokRoutes();
      } else {
        this.showToast('Failed to add BYOK route', 'error');
      }
    } catch (e) {
      this.showToast(`Error: ${e.message}`, 'error');
    }
  }

  async deleteByokRoute(routeId) {
    try {
      const res = await fetch(`/api/system/byok?id=${encodeURIComponent(routeId)}`, { method: 'DELETE' });
      if (res.ok) {
        this.showToast(`Route ${routeId} removed`, 'success');
        this.loadByokRoutes();
      }
    } catch (e) {
      this.showToast('Failed to delete route', 'error');
    }
  }

  // --- Toast Notifications ---
  showToast(message, type = 'info') {
    const container = document.getElementById('toast-container');
    if (!container) return;

    const toast = document.createElement('div');
    toast.className = `toast ${type}`;
    toast.textContent = message;
    container.appendChild(toast);

    setTimeout(() => {
      toast.style.opacity = '0';
      toast.style.transition = 'opacity 0.3s ease';
      setTimeout(() => toast.remove(), 300);
    }, 3000);
  }
}

// Instantiate and initialize
const app = new ToklessApp();
document.addEventListener('DOMContentLoaded', () => app.init());

