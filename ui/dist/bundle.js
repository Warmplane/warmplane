class C{state={configPath:"mcp_servers.json",config:{mcpServers:{}},serverStatuses:{},capabilities:[],selectedCapabilityId:null,activeTab:"overview",eventLogs:[],metrics:{totalCatalogRequests:0,totalEtagHits:0,totalToolCalls:0,totalToolDurationUs:0}};listeners=[];getState(){return this.state}setState(t){this.state={...this.state,...t},this.listeners.forEach((e)=>e(this.state))}subscribe(t){return this.listeners.push(t),()=>{this.listeners=this.listeners.filter((e)=>e!==t)}}addEventLog(t,e,a,s){let i=[{time:new Date().toLocaleTimeString(),method:t,target:e,status:a,latency:s},...this.state.eventLogs].slice(0,50);this.setState({eventLogs:i})}}var n=new C;class k{baseUrl;constructor(t=""){this.baseUrl=t}async getConfig(){return(await fetch(`${this.baseUrl}/v1/config`)).json()}async listCapabilities(){return(await fetch(`${this.baseUrl}/v1/capabilities`)).json()}async callCapability(t){let e=performance.now(),a=await fetch(`${this.baseUrl}/v1/tools/call`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify(t)}),s=performance.now()-e,o=await a.json();return{status:a.status,durationMs:s,data:o}}async upsertServer(t,e){return(await fetch(`${this.baseUrl}/v1/config/servers`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({name:t,server:e})})).json()}async deleteServer(t){return(await fetch(`${this.baseUrl}/v1/config/servers/${encodeURIComponent(t)}`,{method:"DELETE"})).json()}async getEcosystemSources(){return(await fetch(`${this.baseUrl}/v1/config/ecosystem`)).json()}async importConfig(t,e=!1){return(await fetch(`${this.baseUrl}/v1/config/import`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({source_path:t,overwrite:e})})).json()}async savePolicy(t){return(await fetch(`${this.baseUrl}/v1/config/policy`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify(t)})).json()}async updateAlias(t,e,a){return(await fetch(`${this.baseUrl}/v1/config/alias`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({kind:t,alias:e,target:a})})).json()}}var c=new k;function j(){let t=n.getState(),e=t.config.mcpServers||{},a=Object.keys(e),s=a.length,o="";if(a.length===0)o=`
      <div style="grid-column: 1 / -1; padding: 32px; text-align: center; color: var(--text-dim); background: var(--surface-card); border-radius: var(--radius-md); border: 1px dashed var(--border);">
        <div style="font-size: 14px; color: var(--text-main); font-weight: 600; margin-bottom: 6px;">No Upstream MCP Servers Connected</div>
        <div style="font-size: 12px; margin-bottom: 16px;">Initialize connections by adding a server or syncing existing IDE configurations.</div>
        <div style="display: flex; gap: 8px; justify-content: center;">
          <button class="btn btn-primary" onclick="window.app.openAddServerModal()">+ Add Server</button>
          <button class="btn btn-ghost" onclick="window.app.openImportModal()">Sync from IDEs</button>
        </div>
      </div>
    `;else o=a.map((g)=>{let y=e[g],E=y.command?"stdio":"http / sse",S=y.command?`${y.command} ${(y.args||[]).join(" ")}`:y.url,$=t.serverStatuses[g]||{status:"connected",protocol_version:"2026-07-28"};return`
        <div class="bento-card">
          <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
            <span style="font-weight: 700; color: var(--text-main); display: flex; align-items: center; gap: 6px;">
              <span style="width: 8px; height: 8px; border-radius: 50%; background: var(--green-400); display: inline-block;"></span>
              ${m(g)}
            </span>
            <span class="brand-badge">${E}</span>
          </div>
          <div style="font-family: var(--ff-mono); font-size: 11.5px; color: var(--text-dim); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; margin-bottom: 12px;" title="${m(S||"")}">
            ${m(S||"")}
          </div>
          <div style="display: flex; justify-content: space-between; font-size: 11px; color: var(--text-muted); border-top: 1px solid var(--border); padding-top: 8px;">
            <span>Status: <strong style="color: var(--green-400);">${$.status}</strong></span>
            <span>Protocol: ${$.protocol_version}</span>
          </div>
        </div>
      `}).join("");let i=t.eventLogs.length===0?`
    <div class="feed-row" style="grid-template-columns: 80px 100px 1fr 100px 80px;">
      <span style="color: var(--text-dim);">ready</span>
      <span style="color: var(--cyan-400); font-weight: 600;">SSE</span>
      <span style="color: var(--text-main);">/v1/resources/updates stream active</span>
      <span style="color: var(--green-400);">CONNECTED</span>
      <span style="color: var(--amber-300); text-align: right;">0.0ms</span>
    </div>
  `:t.eventLogs.map((g)=>`
    <div class="feed-row" style="grid-template-columns: 80px 100px 1fr 100px 80px;">
      <span style="color: var(--text-dim);">${m(g.time)}</span>
      <span style="color: var(--cyan-400); font-weight: 600;">${m(g.method)}</span>
      <span style="color: var(--text-main); font-family: var(--ff-mono);">${m(g.target)}</span>
      <span style="color: var(--green-400);">${m(g.status)}</span>
      <span style="color: var(--amber-300); text-align: right;">${m(g.latency)}</span>
    </div>
  `).join(""),r=t.metrics,l=r.totalCatalogRequests,d=r.totalEtagHits,p=l>0?`${(d/l*100).toFixed(1)}%`:"0.0%",h=l>0?`${d} of ${l} requests served via HTTP 304`:"Waiting for client requests",b=r.totalToolCalls,O=b>0?`${(r.totalToolDurationUs/b/1000).toFixed(1)}ms`:"0.0ms",N=b>0?`${b} tool executions processed`:"Local worker task queues warm",x=Object.keys(t.config.capabilityAliases||{}).length+Object.keys(t.config.resourceAliases||{}).length+Object.keys(t.config.promptAliases||{}).length,L=x>0?`${x*18}B / call`:"0B",I=x>0?`${x} active facade aliases pruning prompt size`:"Configure aliases in Studio to reduce prompt size";return`
    <div class="bento-grid">
      <div class="bento-card col-3">
        <div class="stat-label">Token Savings Rate</div>
        <div class="stat-value" style="color: var(--amber-300);">${L}</div>
        <div class="stat-sub">${I}</div>
      </div>
      <div class="bento-card col-3">
        <div class="stat-label">ETag Cache Hit Rate</div>
        <div class="stat-value" style="color: var(--cyan-400);">${p}</div>
        <div class="stat-sub">${h}</div>
      </div>
      <div class="bento-card col-3">
        <div class="stat-label">Connected Upstreams</div>
        <div class="stat-value" style="color: var(--green-400);">${s} Active</div>
        <div class="stat-sub">${s>0?"Persistent worker task channels":"No active upstream servers"}</div>
      </div>
      <div class="bento-card col-3">
        <div class="stat-label">Avg Execution Latency</div>
        <div class="stat-value">${O}</div>
        <div class="stat-sub">${N}</div>
      </div>
    </div>

    <div style="display: flex; justify-content: space-between; align-items: center; margin: 24px 0 12px;">
      <div style="font-size: 15px; font-weight: 700; color: var(--text-main);">Connected Upstream Servers</div>
      <button class="btn btn-ghost" onclick="window.app.switchTab('servers')">Manage All (${s}) →</button>
    </div>

    <div style="display: grid; grid-template-columns: repeat(auto-fill, minmax(280px, 1fr)); gap: 14px; margin-bottom: 24px;">
      ${o}
    </div>

    <div style="font-size: 15px; font-weight: 700; color: var(--text-main); margin-bottom: 12px;">
      Live Control Plane Event Stream
    </div>
    <div style="background: var(--surface-card); border: 1px solid var(--border); border-radius: var(--radius-md); overflow: hidden; font-family: var(--ff-mono); font-size: 11.5px;">
      <div style="display: grid; grid-template-columns: 80px 100px 1fr 100px 80px; padding: 8px 14px; background: var(--surface-hover); border-bottom: 1px solid var(--border); color: var(--text-muted); font-weight: 600;">
        <span>TIME</span>
        <span>METHOD</span>
        <span>EVENT / TARGET</span>
        <span>STATUS</span>
        <span style="text-align: right;">LATENCY</span>
      </div>
      <div id="overview-event-rows">
        ${i}
      </div>
    </div>
  `}function m(t){return String(t||"").replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;")}function A(){let t=n.getState(),e=t.config.mcpServers||{},a=Object.keys(e),s="";if(a.length===0)s=`
      <div style="padding: 40px; text-align: center; color: var(--text-dim); background: var(--surface-card); border-radius: var(--radius-md); border: 1px dashed var(--border);">
        <div style="font-size: 15px; color: var(--text-main); font-weight: 600; margin-bottom: 8px;">No Servers Configured in ${u(t.configPath)}</div>
        <p style="font-size: 12px; margin-bottom: 20px; max-width: 480px; margin-left: auto; margin-right: auto;">
          Warmplane bridges local tools and remote MCP servers into one unified facade. Add your first server or import existing configs from Claude Desktop or Cursor.
        </p>
        <div style="display: flex; gap: 10px; justify-content: center;">
          <button class="btn btn-primary" onclick="window.app.openAddServerModal()">+ Add New Server</button>
          <button class="btn btn-ghost" onclick="window.app.openImportModal()">Sync from IDEs</button>
        </div>
      </div>
    `;else s=a.map((o)=>{let i=e[o],r=i.command?"stdio":"http / sse",l=i.command?`${i.command} ${(i.args||[]).join(" ")}`:i.url,d=t.serverStatuses[o]||{status:"connected",protocol_version:"2026-07-28"},p=i.env?Object.keys(i.env).map((h)=>`${h}=***`).join(", "):"None";return`
        <div class="bento-card" style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px;">
          <div>
            <div style="display: flex; align-items: center; gap: 8px; margin-bottom: 4px;">
              <span style="width: 8px; height: 8px; border-radius: 50%; background: var(--green-400); display: inline-block;"></span>
              <span style="font-size: 15px; font-weight: 700; color: var(--text-main);">${u(o)}</span>
              <span class="brand-badge">${r}</span>
              <span class="brand-badge" style="color: var(--cyan-400); border-color: rgba(34, 211, 238, 0.25);">Protocol: ${d.protocol_version}</span>
            </div>
            <div style="font-family: var(--ff-mono); font-size: 12px; color: var(--text-muted); margin-top: 4px;">
              ${i.command?"Command: ":"URL: "}<code>${u(l||"")}</code>
            </div>
            ${i.env?`<div style="font-family: var(--ff-mono); font-size: 11px; color: var(--text-dim); margin-top: 2px;">Env: ${u(p)}</div>`:""}
          </div>
          <div style="display: flex; gap: 8px;">
            <button class="btn btn-danger" onclick="window.app.deleteServer('${u(o)}')">Remove</button>
          </div>
        </div>
      `}).join("");return`
    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 18px;">
      <div>
        <div style="font-size: 16px; font-weight: 700; color: var(--text-main);">Configured MCP Upstream Servers</div>
        <div style="font-size: 11px; color: var(--text-dim);">Active configuration file: <code>${u(t.configPath)}</code></div>
      </div>
      <div style="display: flex; gap: 8px;">
        <button class="btn btn-ghost" onclick="window.app.openImportModal()">Sync from IDEs</button>
        <button class="btn btn-primary" onclick="window.app.openAddServerModal()">+ Add New Server</button>
      </div>
    </div>

    ${s}
  `}function u(t){return String(t||"").replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;")}function P(){let t=n.getState(),e=t.capabilities,a=t.selectedCapabilityId||(e.length>0?e[0].id:null),s=e.find((r)=>r.id===a),o="";if(e.length===0)o=`
      <div style="padding: 24px 16px; text-align: center; color: var(--text-dim); font-size: 11.5px;">
        No tools or capabilities discovered from connected servers.
      </div>
    `;else o=e.map((r)=>{return`
        <div class="cap-item ${r.id===a?"active":""}" onclick="window.app.selectCapability('${f(r.id)}')">
          <div style="display: flex; justify-content: space-between; align-items: center;">
            <span style="font-weight: 600; color: var(--text-main); font-family: var(--ff-mono); font-size: 12px;">${f(r.id)}</span>
            <span style="font-size: 10px; color: var(--green-400);">${f(r.mode||"read")}</span>
          </div>
          <div style="font-size: 11px; color: var(--text-dim); margin-top: 2px;">server: ${f(r.server||"local")}</div>
        </div>
      `}).join("");let i=s&&s.input_schema?JSON.stringify(s.input_schema.properties||{},null,2):"{}";return`
    <div style="display: grid; grid-template-columns: 320px 1fr; gap: 16px; height: calc(100vh - 120px);">
      <!-- Left Sidebar: Capabilities Catalog -->
      <div style="background: var(--surface-card); border: 1px solid var(--border); border-radius: var(--radius-md); display: flex; flex-direction: column; overflow: hidden;">
        <div style="padding: 12px; border-bottom: 1px solid var(--border);">
          <input type="text" class="form-input" placeholder="Search ${e.length} capabilities..." oninput="window.app.filterCapabilities(this.value)">
        </div>
        <div style="flex: 1; overflow-y: auto; padding: 8px;">
          ${o}
        </div>
      </div>

      <!-- Right Panel: Capability Execution & Envelope Visualizer -->
      <div style="background: var(--surface-card); border: 1px solid var(--border); border-radius: var(--radius-md); display: flex; flex-direction: column; overflow: hidden;">
        <div style="padding: 14px 18px; border-bottom: 1px solid var(--border); display: flex; justify-content: space-between; align-items: center;">
          <div>
            <div style="font-size: 15px; font-weight: 700; color: var(--text-main); font-family: var(--ff-mono);" id="pg-selected-title">
              ${f(s?s.id:"No Capability Selected")}
            </div>
            <div style="font-size: 11.5px; color: var(--text-dim);" id="pg-selected-desc">
              ${f(s?s.summary||s.description:"Connect servers to inspect and execute tools")}
            </div>
          </div>
          <button class="btn btn-primary" onclick="window.app.executePlaygroundTool()" ${s?"":"disabled"}>
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polygon points="5 3 19 12 5 21 5 3"></polygon></svg>
            Execute Capability
          </button>
        </div>

        <div style="flex: 1; display: grid; grid-template-columns: 1fr 1fr; overflow: hidden;">
          <!-- Request Builder -->
          <div style="padding: 16px; border-right: 1px solid var(--border); overflow-y: auto;">
            <div class="form-group">
              <label class="form-label">Arguments JSON (Object)</label>
              <textarea class="form-textarea" rows="8" id="pg-args-input">${f(i)}</textarea>
            </div>
            <div class="form-group">
              <label class="form-label">Request Context / Operation ID (Optional)</label>
              <input type="text" class="form-input" id="pg-context-input" placeholder="e.g. op-dev-test-1">
            </div>
            ${s&&s.input_schema?`
              <div style="margin-top: 14px;">
                <label class="form-label">Input JSON Schema</label>
                <pre style="background: var(--surface); padding: 10px; border-radius: var(--radius-sm); border: 1px solid var(--border); font-size: 11px; color: var(--text-muted); max-height: 180px; overflow-y: auto;">${f(JSON.stringify(s.input_schema,null,2))}</pre>
              </div>
            `:""}
          </div>

          <!-- Response Inspector -->
          <div style="padding: 16px; background: var(--bg-app); display: flex; flex-direction: column; overflow: hidden;">
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
              <span style="font-size: 11px; font-weight: 600; color: var(--text-dim);">NORMALIZED EXECUTION ENVELOPE</span>
              <span id="pg-status-badge" style="font-size: 11px; font-weight: 600; color: var(--text-dim); font-family: var(--ff-mono);">READY</span>
            </div>
            <pre id="pg-response-json" style="flex: 1; background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 14px; color: var(--amber-300); font-size: 11.5px; overflow-y: auto; margin: 0;">// Response envelope output will be formatted here</pre>
          </div>
        </div>
      </div>
    </div>
  `}function f(t){return String(t||"").replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;")}function T(){let e=n.getState().config.policy||{},a=e.allow||[],s=e.deny||[],o=e.redact_keys||[],i=a.length===0?`
    <div style="color: var(--text-dim); font-size: 12px;">No allow list (all non-denied operations permitted)</div>
  `:a.map((d,p)=>`
    <div style="display: flex; justify-content: space-between; align-items: center; background: var(--surface); padding: 8px 12px; border-radius: var(--radius-sm); border: 1px solid var(--border);">
      <span style="font-family: var(--ff-mono); font-size: 12px; color: var(--green-400);">✔ ${w(d)}</span>
      <button class="btn btn-ghost" style="padding: 2px 6px; font-size: 11px; color: var(--red-400);" onclick="window.app.removePolicyRule('allow', ${p})">✕</button>
    </div>
  `).join(""),r=s.length===0?`
    <div style="color: var(--text-dim); font-size: 12px;">No deny rules configured</div>
  `:s.map((d,p)=>`
    <div style="display: flex; justify-content: space-between; align-items: center; background: var(--surface); padding: 8px 12px; border-radius: var(--radius-sm); border: 1px solid var(--border);">
      <span style="font-family: var(--ff-mono); font-size: 12px; color: var(--red-400);">✖ ${w(d)}</span>
      <button class="btn btn-ghost" style="padding: 2px 6px; font-size: 11px; color: var(--red-400);" onclick="window.app.removePolicyRule('deny', ${p})">✕</button>
    </div>
  `).join(""),l=o.length===0?`
    <div style="color: var(--text-dim); font-size: 12px;">No key redaction patterns configured</div>
  `:o.map((d,p)=>`
    <span class="brand-badge" style="color: var(--amber-300); padding: 5px 10px; font-size: 11px; display: inline-flex; align-items: center; gap: 6px;">
      ${w(d)}
      <span style="cursor: pointer; color: var(--red-400); font-weight: bold;" onclick="window.app.removePolicyRule('redact', ${p})">✕</span>
    </span>
  `).join("");return`
    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 18px;">
      <div>
        <div style="font-size: 16px; font-weight: 700; color: var(--text-main);">Security Governance &amp; Data Redaction</div>
        <div style="font-size: 11px; color: var(--text-dim);">Wildcard capability access control and sensitive key masking</div>
      </div>
    </div>

    <div class="bento-grid">
      <!-- Allow Rules -->
      <div class="bento-card col-6">
        <div class="stat-header">
          <span class="stat-label" style="color: var(--green-400);">Allow List Patterns</span>
        </div>
        <div style="display: flex; flex-direction: column; gap: 8px; margin: 12px 0;">
          ${i}
        </div>
        <div style="display: flex; gap: 8px; margin-top: 14px;">
          <input type="text" class="form-input" id="policy-new-allow" placeholder="e.g. github.*, db.read_*">
          <button class="btn btn-ghost" onclick="window.app.addPolicyRule('allow', document.getElementById('policy-new-allow').value)">Add Allow</button>
        </div>
      </div>

      <!-- Deny Rules -->
      <div class="bento-card col-6">
        <div class="stat-header">
          <span class="stat-label" style="color: var(--red-400);">Deny List Patterns (Strict Precedence)</span>
        </div>
        <div style="display: flex; flex-direction: column; gap: 8px; margin: 12px 0;">
          ${r}
        </div>
        <div style="display: flex; gap: 8px; margin-top: 14px;">
          <input type="text" class="form-input" id="policy-new-deny" placeholder="e.g. *.drop_*, filesystem.write_*">
          <button class="btn btn-ghost" onclick="window.app.addPolicyRule('deny', document.getElementById('policy-new-deny').value)">Add Deny</button>
        </div>
      </div>

      <!-- Key Redaction -->
      <div class="bento-card col-12">
        <div class="stat-header">
          <span class="stat-label" style="color: var(--amber-300);">Sensitive Key Redaction Patterns</span>
          <span style="font-size: 11px; color: var(--text-dim); margin-left: 8px;">Keys automatically masked as &lt;redacted&gt; in logs and envelopes</span>
        </div>
        <div style="display: flex; gap: 8px; flex-wrap: wrap; margin: 12px 0;">
          ${l}
        </div>
        <div style="display: flex; gap: 8px; margin-top: 14px; max-width: 420px;">
          <input type="text" class="form-input" id="policy-new-redact" placeholder="e.g. token, api_key, password, secret">
          <button class="btn btn-ghost" onclick="window.app.addPolicyRule('redact', document.getElementById('policy-new-redact').value)">Add Key</button>
        </div>
      </div>

      <!-- Evaluation Sandbox Tester -->
      <div class="bento-card col-12">
        <div class="stat-header">
          <span class="stat-label">Policy Evaluation Sandbox</span>
          <span style="font-size: 11px; color: var(--text-dim); margin-left: 8px;">Live verification of capability access against active rules</span>
        </div>
        <div style="display: flex; gap: 12px; align-items: center; margin-top: 12px;">
          <input type="text" class="form-input" placeholder="Type capability identifier, e.g. github.create_issue or sqlite.drop_table" style="flex: 1;" oninput="window.app.testPolicySandbox(this.value)">
          <div id="policy-test-verdict" style="font-weight: 700; font-family: var(--ff-mono); font-size: 12.5px; padding: 7px 16px; border-radius: var(--radius-sm); background: var(--surface); border: 1px solid var(--border); color: var(--green-400);">
            ALLOWED
          </div>
        </div>
      </div>
    </div>
  `}function w(t){return String(t||"").replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;")}function z(){let t=n.getState(),e=t.config,a=Object.entries(e.capabilityAliases||{}),s=Object.entries(e.resourceAliases||{}),o=Object.entries(e.promptAliases||{}),i="";if(a.length===0&&s.length===0&&o.length===0)i=`
      <div style="padding: 24px; text-align: center; color: var(--text-dim);">
        No facade aliases configured in ${v(t.configPath)}. Add short names to prune token payload sizes.
      </div>
    `;else{for(let[r,l]of a)i+=`
        <div class="feed-row" style="grid-template-columns: 90px 180px 1fr 80px;">
          <span style="color: var(--cyan-400);">Tool</span>
          <span style="font-weight: 700; color: var(--text-main); font-family: var(--ff-mono);">${v(r)}</span>
          <span style="color: var(--text-dim); font-family: var(--ff-mono);">${v(l)}</span>
          <div style="text-align: right;">
            <button class="btn btn-ghost" style="padding: 2px 6px; color: var(--red-400);" onclick="window.app.deleteAlias('tool', '${v(r)}')">✕</button>
          </div>
        </div>
      `;for(let[r,l]of s)i+=`
        <div class="feed-row" style="grid-template-columns: 90px 180px 1fr 80px;">
          <span style="color: var(--green-400);">Resource</span>
          <span style="font-weight: 700; color: var(--text-main); font-family: var(--ff-mono);">${v(r)}</span>
          <span style="color: var(--text-dim); font-family: var(--ff-mono);">${v(l)}</span>
          <div style="text-align: right;">
            <button class="btn btn-ghost" style="padding: 2px 6px; color: var(--red-400);" onclick="window.app.deleteAlias('resource', '${v(r)}')">✕</button>
          </div>
        </div>
      `;for(let[r,l]of o)i+=`
        <div class="feed-row" style="grid-template-columns: 90px 180px 1fr 80px;">
          <span style="color: var(--amber-300);">Prompt</span>
          <span style="font-weight: 700; color: var(--text-main); font-family: var(--ff-mono);">${v(r)}</span>
          <span style="color: var(--text-dim); font-family: var(--ff-mono);">${v(l)}</span>
          <div style="text-align: right;">
            <button class="btn btn-ghost" style="padding: 2px 6px; color: var(--red-400);" onclick="window.app.deleteAlias('prompt', '${v(r)}')">✕</button>
          </div>
        </div>
      `}return`
    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 18px;">
      <div>
        <div style="font-size: 16px; font-weight: 700; color: var(--text-main);">Facade &amp; Alias Studio</div>
        <div style="font-size: 11px; color: var(--text-dim);">Shorten capability IDs to prune prompt tokens and create stable public interfaces</div>
      </div>
    </div>

    <!-- Quick Add Form -->
    <div class="bento-card" style="margin-bottom: 20px;">
      <div class="stat-header" style="margin-bottom: 12px;">
        <span class="stat-label">Create New Alias</span>
      </div>
      <div style="display: grid; grid-template-columns: 140px 1fr 1fr 100px; gap: 10px; align-items: center;">
        <select class="form-input" id="alias-kind">
          <option value="tool">Tool / Capability</option>
          <option value="resource">Resource</option>
          <option value="prompt">Prompt</option>
        </select>
        <input type="text" class="form-input" id="alias-name" placeholder="Public alias (e.g. db.query)">
        <input type="text" class="form-input" id="alias-target" placeholder="Target ID (e.g. sqlite.read_query)">
        <button class="btn btn-primary" onclick="window.app.createAlias()">+ Save</button>
      </div>
    </div>

    <!-- Aliases Table -->
    <div style="background: var(--surface-card); border: 1px solid var(--border); border-radius: var(--radius-md); overflow: hidden; font-family: var(--ff-mono); font-size: 12px;">
      <div style="display: grid; grid-template-columns: 90px 180px 1fr 80px; padding: 10px 14px; background: var(--surface-hover); border-bottom: 1px solid var(--border); color: var(--text-muted); font-weight: 600;">
        <span>TYPE</span>
        <span>PUBLIC ALIAS</span>
        <span>CANONICAL TARGET</span>
        <span style="text-align: right;">ACTION</span>
      </div>
      ${i}
    </div>
  `}function v(t){return String(t||"").replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;")}class _{async init(){let t=window.location.port?`:${window.location.port}`:"",e=document.getElementById("daemon-port-label");if(e)e.textContent=`Daemon ${t}`;await this.refreshData(),this.initSSE(),this.render(),n.subscribe(()=>{this.render()})}async refreshData(){try{let[t,e]=await Promise.all([c.getConfig(),c.listCapabilities()]);if(t.ok)n.setState({configPath:t.config_path,config:t.config,serverStatuses:t.server_statuses||{},metrics:{totalCatalogRequests:t.metrics?.total_catalog_requests||0,totalEtagHits:t.metrics?.total_etag_hits||0,totalToolCalls:t.metrics?.total_tool_calls||0,totalToolDurationUs:t.metrics?.total_tool_duration_us||0}});if(e.ok)n.setState({capabilities:e.capabilities||[]})}catch(t){console.error("Failed to fetch daemon state:",t)}}initSSE(){try{let t=new EventSource("/v1/resources/updates");t.onmessage=(e)=>{n.addEventLog("SSE","/v1/resources/updates","UPDATED","0.1ms"),this.refreshData()}}catch(t){console.warn("SSE connection unavailable")}}switchTab(t){n.setState({activeTab:t})}render(){let t=n.getState(),e=document.getElementById("app-main");if(!e)return;document.querySelectorAll(".nav-item").forEach((o)=>{if(o.getAttribute("data-tab")===t.activeTab)o.classList.add("active");else o.classList.remove("active")});let a=document.getElementById("top-title"),s={overview:"Overview Cockpit",servers:"Server Hub & Connections",playground:"MCP Capability Playground",policy:"Security Governance & Redaction",aliases:"Facade & Alias Studio"};if(a)a.textContent=s[t.activeTab];switch(t.activeTab){case"overview":e.innerHTML=j();break;case"servers":e.innerHTML=A();break;case"playground":e.innerHTML=P();break;case"policy":e.innerHTML=T();break;case"aliases":e.innerHTML=z();break}}selectCapability(t){n.setState({selectedCapabilityId:t})}filterCapabilities(t){let e=t.toLowerCase().trim(),s=n.getState().capabilities.filter((i)=>i.id.toLowerCase().includes(e)||i.summary&&i.summary.toLowerCase().includes(e)||i.server&&i.server.toLowerCase().includes(e)),o=document.querySelector(".playground-sidebar div:last-child");if(o)o.innerHTML=s.map((i)=>`
        <div class="cap-item ${i.id===n.getState().selectedCapabilityId?"active":""}" onclick="window.app.selectCapability('${i.id}')">
          <div style="display: flex; justify-content: space-between; align-items: center;">
            <span style="font-weight: 600; color: var(--text-main); font-family: var(--ff-mono); font-size: 12px;">${i.id}</span>
            <span style="font-size: 10px; color: var(--green-400);">${i.mode||"read"}</span>
          </div>
          <div style="font-size: 11px; color: var(--text-dim); margin-top: 2px;">server: ${i.server||"local"}</div>
        </div>
      `).join("")}async executePlaygroundTool(){let t=n.getState(),e=t.selectedCapabilityId||(t.capabilities[0]?t.capabilities[0].id:null);if(!e)return;let a=document.getElementById("pg-args-input")?.value||"{}",s=document.getElementById("pg-context-input")?.value||void 0,o=document.getElementById("pg-status-badge"),i=document.getElementById("pg-response-json"),r={};try{r=JSON.parse(a)}catch{alert("Invalid arguments JSON object");return}if(o)o.textContent="EXECUTING...",o.style.color="var(--amber-400)";try{let l=await c.callCapability({capability_id:e,args:r,context:s?{operation_id:s}:void 0,request_id:`ui-req-${Date.now()}`});if(o)o.textContent=`HTTP ${l.status} · ${l.durationMs.toFixed(1)}ms`,o.style.color=l.status===200?"var(--green-400)":"var(--red-400)";if(i)i.textContent=JSON.stringify(l.data,null,2);n.addEventLog("POST",`/v1/tools/call → ${e}`,l.status===200?"200 OK":`HTTP ${l.status}`,`${l.durationMs.toFixed(1)}ms`)}catch(l){if(o)o.textContent="ERROR",o.style.color="var(--red-400)";if(i)i.textContent=l.toString()}}async addPolicyRule(t,e){if(!e||!e.trim())return;let s=n.getState().config.policy||{},o=[...s.allow||[]],i=[...s.deny||[]],r=[...s.redact_keys||[]];if(t==="allow"&&!o.includes(e.trim()))o.push(e.trim());if(t==="deny"&&!i.includes(e.trim()))i.push(e.trim());if(t==="redact"&&!r.includes(e.trim()))r.push(e.trim());await c.savePolicy({allow:o,deny:i,redact_keys:r}),await this.refreshData()}async removePolicyRule(t,e){let s=n.getState().config.policy||{},o=[...s.allow||[]],i=[...s.deny||[]],r=[...s.redact_keys||[]];if(t==="allow")o.splice(e,1);if(t==="deny")i.splice(e,1);if(t==="redact")r.splice(e,1);await c.savePolicy({allow:o,deny:i,redact_keys:r}),await this.refreshData()}testPolicySandbox(t){let e=document.getElementById("policy-test-verdict");if(!e)return;let a=t.trim();if(!a){e.textContent="ENTER ID",e.style.color="var(--text-dim)";return}let o=n.getState().config.policy||{},i=o.deny||[],r=o.allow||[],l=(d,p)=>{if(d==="*")return!0;if(d.endsWith("*"))return p.startsWith(d.slice(0,-1));return d===p};if(i.some((d)=>l(d,a))){e.textContent="DENIED (Strict Block)",e.style.color="var(--red-400)";return}if(r.length>0&&!r.some((d)=>l(d,a))){e.textContent="DENIED (Not in Allow List)",e.style.color="var(--red-400)";return}e.textContent="ALLOWED",e.style.color="var(--green-400)"}async deleteServer(t){if(!confirm(`Are you sure you want to remove server '${t}' from config?`))return;await c.deleteServer(t),await this.refreshData()}openAddServerModal(){let t=document.getElementById("modal-add-server");if(t)t.classList.add("active")}async submitAddServer(){let t=document.getElementById("modal-srv-name")?.value.trim(),e=document.getElementById("modal-srv-transport")?.value;if(!t){alert("Server name is required");return}let a={};if(e==="stdio"){let i=(document.getElementById("modal-srv-command")?.value.trim()).split(/\s+/).filter(Boolean);if(i.length===0){alert("Command is required");return}a.command=i[0],a.args=i.slice(1)}else{let o=document.getElementById("modal-srv-url")?.value.trim();if(!o){alert("URL is required");return}a.url=o}let s=await c.upsertServer(t,a);if(s.ok)this.closeModals(),await this.refreshData();else alert(`Failed to save server: ${s.error}`)}async openImportModal(){let t=document.getElementById("modal-import");if(t)t.classList.add("active");let e=document.getElementById("modal-eco-list");if(!e)return;e.innerHTML='<div style="color: var(--text-dim); padding: 12px; text-align: center;">Scanning IDE configs...</div>';try{let a=await c.getEcosystemSources();if(a.sources&&a.sources.length>0)e.innerHTML=a.sources.map((s)=>`
          <label style="display: flex; align-items: center; gap: 10px; background: var(--surface); padding: 10px 14px; border-radius: var(--radius-sm); border: 1px solid var(--border); cursor: pointer;">
            <input type="checkbox" class="eco-checkbox" value="${s.path}" checked>
            <div>
              <div style="font-weight: 600; color: var(--text-main);">${s.name}</div>
              <div style="font-size: 11px; color: var(--text-dim);">${s.server_count} servers (${s.servers.join(", ")})</div>
            </div>
          </label>
        `).join("");else e.innerHTML='<div style="color: var(--text-dim); padding: 12px; text-align: center;">No external MCP configuration files found on this system.</div>'}catch{e.innerHTML='<div style="color: var(--red-400); padding: 12px; text-align: center;">Failed to scan ecosystem sources.</div>'}}async submitImport(){let t=document.querySelectorAll(".eco-checkbox:checked");if(t.length===0){alert("No sources selected");return}for(let e of Array.from(t))await c.importConfig(e.value,!1);this.closeModals(),await this.refreshData()}async createAlias(){let t=document.getElementById("alias-kind")?.value,e=document.getElementById("alias-name")?.value.trim(),a=document.getElementById("alias-target")?.value.trim();if(!e||!a){alert("Please provide both alias name and canonical target");return}await c.updateAlias(t,e,a),await this.refreshData()}async deleteAlias(t,e){await c.updateAlias(t,e,void 0),await this.refreshData()}closeModals(){document.querySelectorAll(".modal-backdrop").forEach((t)=>t.classList.remove("active"))}}var R=new _;window.app=R;window.addEventListener("DOMContentLoaded",()=>R.init());
