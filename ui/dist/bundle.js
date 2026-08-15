class T{state={configPath:"mcp_servers.json",config:{mcpServers:{}},serverStatuses:{},capabilities:[],approvals:[],selectedCapabilityId:null,activeTab:"overview",eventLogs:[],executionResult:null,metrics:{totalCatalogRequests:0,totalEtagHits:0,totalToolCalls:0,totalToolDurationUs:0}};listeners=[];getState(){return this.state}setState(e){this.state={...this.state,...e},this.listeners.forEach((t)=>t(this.state))}subscribe(e){return this.listeners.push(e),()=>{this.listeners=this.listeners.filter((t)=>t!==e)}}addEventLog(e,t,r,i){let o=[{time:new Date().toLocaleTimeString(),method:e,target:t,status:r,latency:i},...this.state.eventLogs].slice(0,50);this.setState({eventLogs:o})}}var l=new T;class P{baseUrl;constructor(e=""){this.baseUrl=e}async getConfig(){return(await fetch(`${this.baseUrl}/v1/config`)).json()}async listCapabilities(){return(await fetch(`${this.baseUrl}/v1/capabilities`)).json()}async callCapability(e){let t=performance.now(),r=await fetch(`${this.baseUrl}/v1/tools/call`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify(e)}),i=performance.now()-t,a=await r.json();return{status:r.status,durationMs:i,data:a}}async upsertServer(e,t){return(await fetch(`${this.baseUrl}/v1/config/servers`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({name:e,server:t})})).json()}async deleteServer(e){return(await fetch(`${this.baseUrl}/v1/config/servers/${encodeURIComponent(e)}`,{method:"DELETE"})).json()}async getEcosystemSources(){return(await fetch(`${this.baseUrl}/v1/config/ecosystem`)).json()}async importConfig(e,t=!1){return(await fetch(`${this.baseUrl}/v1/config/import`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({source_path:e,overwrite:t})})).json()}async savePolicy(e){let t={allow:e.allow||[],deny:e.deny||[],redactKeys:e.redact_keys||e.redactKeys||[],requireApproval:e.require_approval||e.requireApproval||[],approvalTimeoutSecs:e.approvalTimeoutSecs||e.approval_timeout_secs||300,webhook:e.webhook};return(await fetch(`${this.baseUrl}/v1/config/policy`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify(t)})).json()}async listApprovals(){return(await fetch(`${this.baseUrl}/v1/approvals`)).json()}async approveTicket(e,t,r){return(await fetch(`${this.baseUrl}/v1/approvals/${encodeURIComponent(e)}/approve`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({operator:t,modified_args:r})})).json()}async rejectTicket(e,t,r){return(await fetch(`${this.baseUrl}/v1/approvals/${encodeURIComponent(e)}/reject`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({operator:t,reason:r})})).json()}async updateAlias(e,t,r){return(await fetch(`${this.baseUrl}/v1/config/alias`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({kind:e,alias:t,target:r})})).json()}async reloadConfig(){return(await fetch(`${this.baseUrl}/v1/config/reload`,{method:"POST"})).json()}}var c=new P;function O(){let e=l.getState(),t=e.config.mcpServers||{},r=Object.keys(t),i=r.length,a="";if(r.length===0)a=`
      <div style="grid-column: 1 / -1; padding: 32px; text-align: center; color: var(--text-dim); background: var(--surface-card); border-radius: var(--radius-md); border: 1px dashed var(--border);">
        <div style="font-size: 14px; color: var(--text-main); font-weight: 600; margin-bottom: 6px;">No Upstream MCP Servers Connected</div>
        <div style="font-size: 12px; margin-bottom: 16px;">Initialize connections by adding a server or syncing existing IDE configurations.</div>
        <div style="display: flex; gap: 8px; justify-content: center;">
          <button class="btn btn-primary" onclick="window.app.openTemplateCatalog()">✨ Browse Templates</button>
          <button class="btn btn-ghost" onclick="window.app.openAddServerModal()">+ Add Custom</button>
          <button class="btn btn-ghost" onclick="window.app.openImportModal()">Sync from IDEs</button>
        </div>
      </div>
    `;else a=r.map((b)=>{let w=t[b],U=w.command?"stdio":"http / sse",C=w.command?`${w.command} ${(w.args||[]).join(" ")}`:w.url,_=e.serverStatuses[b]||{status:"connected",protocol_version:"2026-07-28"};return`
        <div class="bento-card">
          <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
            <span style="font-weight: 700; color: var(--text-main); display: flex; align-items: center; gap: 6px;">
              <span style="width: 8px; height: 8px; border-radius: 50%; background: var(--green-400); display: inline-block;"></span>
              ${x(b)}
            </span>
            <span class="brand-badge">${U}</span>
          </div>
          <div style="font-family: var(--ff-mono); font-size: 11.5px; color: var(--text-dim); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; margin-bottom: 12px;" title="${x(C||"")}">
            ${x(C||"")}
          </div>
          <div style="display: flex; justify-content: space-between; font-size: 11px; color: var(--text-muted); border-top: 1px solid var(--border); padding-top: 8px;">
            <span>Status: <strong style="color: var(--green-400);">${_.status}</strong></span>
            <span>Protocol: ${_.protocol_version}</span>
          </div>
        </div>
      `}).join("");let o=e.eventLogs.length===0?`
    <div class="feed-row" style="grid-template-columns: 80px 100px 1fr 100px 80px;">
      <span style="color: var(--text-dim);">ready</span>
      <span style="color: var(--cyan-400); font-weight: 600;">SSE</span>
      <span style="color: var(--text-main);">/v1/resources/updates stream active</span>
      <span style="color: var(--green-400);">CONNECTED</span>
      <span style="color: var(--amber-300); text-align: right;">0.0ms</span>
    </div>
  `:e.eventLogs.map((b)=>`
    <div class="feed-row" style="grid-template-columns: 80px 100px 1fr 100px 80px;">
      <span style="color: var(--text-dim);">${x(b.time)}</span>
      <span style="color: var(--cyan-400); font-weight: 600;">${x(b.method)}</span>
      <span style="color: var(--text-main); font-family: var(--ff-mono);">${x(b.target)}</span>
      <span style="color: var(--green-400);">${x(b.status)}</span>
      <span style="color: var(--amber-300); text-align: right;">${x(b.latency)}</span>
    </div>
  `).join(""),s=e.metrics,n=s.totalCatalogRequests,p=s.totalEtagHits,m=n>0?`${(p/n*100).toFixed(1)}%`:"0.0%",v=n>0?`${p} of ${n} requests served via HTTP 304`:"Waiting for client requests",g=s.totalToolCalls,A=g>0?`${(s.totalToolDurationUs/g/1000).toFixed(1)}ms`:"0.0ms",D=g>0?`${g} tool executions processed`:"Local worker task queues warm",S=Object.keys(e.config.capabilityAliases||{}).length+Object.keys(e.config.resourceAliases||{}).length+Object.keys(e.config.promptAliases||{}).length,E=S>0?`${S*18}B / call`:"0B",F=S>0?`${S} active facade aliases pruning prompt size`:"Configure aliases in Studio to reduce prompt size";return`
    <div class="bento-grid">
      <div class="bento-card col-3">
        <div class="stat-label">Token Savings Rate</div>
        <div class="stat-value" style="color: var(--amber-300);">${E}</div>
        <div class="stat-sub">${F}</div>
      </div>
      <div class="bento-card col-3">
        <div class="stat-label">ETag Cache Hit Rate</div>
        <div class="stat-value" style="color: var(--cyan-400);">${m}</div>
        <div class="stat-sub">${v}</div>
      </div>
      <div class="bento-card col-3">
        <div class="stat-label">Connected Upstreams</div>
        <div class="stat-value" style="color: var(--green-400);">${i} Active</div>
        <div class="stat-sub">${i>0?"Persistent worker task channels":"No active upstream servers"}</div>
      </div>
      <div class="bento-card col-3">
        <div class="stat-label">Avg Execution Latency</div>
        <div class="stat-value">${A}</div>
        <div class="stat-sub">${D}</div>
      </div>
    </div>

    <div style="display: flex; justify-content: space-between; align-items: center; margin: 24px 0 12px;">
      <div style="font-size: 15px; font-weight: 700; color: var(--text-main);">Connected Upstream Servers</div>
      <button class="btn btn-ghost" onclick="window.app.switchTab('servers')">Manage All (${i}) →</button>
    </div>

    <div style="display: grid; grid-template-columns: repeat(auto-fill, minmax(280px, 1fr)); gap: 14px; margin-bottom: 24px;">
      ${a}
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
        ${o}
      </div>
    </div>
  `}function x(e){return String(e||"").replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;")}function j(){let e=l.getState(),t=e.config.mcpServers||{},r=Object.keys(t),i="";if(r.length===0)i=`
      <div style="padding: 40px; text-align: center; color: var(--text-dim); background: var(--surface-card); border-radius: var(--radius-md); border: 1px dashed var(--border);">
        <div style="font-size: 15px; color: var(--text-main); font-weight: 600; margin-bottom: 8px;">No Servers Configured in ${h(e.configPath)}</div>
        <p style="font-size: 12px; margin-bottom: 20px; max-width: 480px; margin-left: auto; margin-right: auto;">
          Warmplane bridges local tools and remote MCP servers into one unified facade. Add your first server or import existing configs from Claude Desktop or Cursor.
        </p>
        <div style="display: flex; gap: 8px; justify-content: center;">
          <button class="btn btn-primary" onclick="window.app.openTemplateCatalog()">✨ Browse Templates</button>
          <button class="btn btn-ghost" onclick="window.app.openAddServerModal()">+ Add Custom</button>
          <button class="btn btn-ghost" onclick="window.app.openImportModal()">Sync from IDEs</button>
        </div>
      </div>
    `;else i=r.map((a)=>{let o=t[a],s=o.command?"stdio":"http / sse",n=o.command?`${o.command} ${(o.args||[]).join(" ")}`:o.url,p=e.serverStatuses[a]||{status:"connected",protocol_version:"2026-07-28"},m=o.env?Object.keys(o.env).map((v)=>`${v}=***`).join(", "):"None";return`
        <div class="bento-card" style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px;">
          <div>
            <div style="display: flex; align-items: center; gap: 8px; margin-bottom: 4px;">
              <span style="width: 8px; height: 8px; border-radius: 50%; background: var(--green-400); display: inline-block;"></span>
              <span style="font-size: 15px; font-weight: 700; color: var(--text-main);">${h(a)}</span>
              <span class="brand-badge">${s}</span>
              <span class="brand-badge" style="color: var(--cyan-400); border-color: rgba(34, 211, 238, 0.25);">Protocol: ${p.protocol_version}</span>
            </div>
            <div style="font-family: var(--ff-mono); font-size: 12px; color: var(--text-muted); margin-top: 4px;">
              ${o.command?"Command: ":"URL: "}<code>${h(n||"")}</code>
            </div>
            ${o.env&&Object.keys(o.env).length>0?`<div style="font-family: var(--ff-mono); font-size: 11px; color: var(--text-dim); margin-top: 2px;">Env: ${h(m)}</div>`:""}
          </div>
          <div style="display: flex; gap: 8px;">
            <button class="btn btn-danger" onclick="window.app.deleteServer('${h(a)}')">Remove</button>
          </div>
        </div>
      `}).join("");return`
    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 18px;">
      <div>
        <div style="font-size: 16px; font-weight: 700; color: var(--text-main);">Configured MCP Upstream Servers</div>
        <div style="font-size: 11px; color: var(--text-dim);">Active configuration file: <code>${h(e.configPath)}</code></div>
      </div>
      <div style="display: flex; gap: 8px;">
        <button class="btn btn-ghost" onclick="window.app.reloadFromDisk()">⟳ Reload</button>
        <button class="btn btn-ghost" onclick="window.app.openImportModal()">Sync from IDEs</button>
        <button class="btn btn-ghost" onclick="window.app.openAddServerModal()">+ Custom Server</button>
        <button class="btn btn-primary" onclick="window.app.openTemplateCatalog()">✨ Browse Templates</button>
      </div>
    </div>

    ${i}
  `}function h(e){return String(e||"").replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;")}function R(){let e=l.getState(),t=e.capabilities,r=e.selectedCapabilityId||(t.length>0?t[0].id:null),i=t.find((s)=>s.id===r),a="";if(t.length===0)a=`
      <div style="padding: 24px 16px; text-align: center; color: var(--text-dim); font-size: 11.5px;">
        No tools or capabilities discovered from connected servers.
      </div>
    `;else a=t.map((s)=>{return`
        <div class="cap-item ${s.id===r?"active":""}" onclick="window.app.selectCapability('${y(s.id)}')">
          <div style="display: flex; justify-content: space-between; align-items: center;">
            <span style="font-weight: 600; color: var(--text-main); font-family: var(--ff-mono); font-size: 12px;">${y(s.id)}</span>
            <span style="font-size: 10px; color: var(--green-400);">${y(s.mode||"read")}</span>
          </div>
          <div style="font-size: 11px; color: var(--text-dim); margin-top: 2px;">server: ${y(s.server||"local")}</div>
        </div>
      `}).join("");let o=i&&i.input_schema?JSON.stringify(i.input_schema.properties||{},null,2):"{}";return`
    <div style="display: grid; grid-template-columns: 320px 1fr; gap: 16px; height: calc(100vh - 120px);">
      <!-- Left Sidebar: Capabilities Catalog -->
      <div style="background: var(--surface-card); border: 1px solid var(--border); border-radius: var(--radius-md); display: flex; flex-direction: column; overflow: hidden;">
        <div style="padding: 12px; border-bottom: 1px solid var(--border);">
          <input type="text" class="form-input" placeholder="Search ${t.length} capabilities..." oninput="window.app.filterCapabilities(this.value)">
        </div>
        <div style="flex: 1; overflow-y: auto; padding: 8px;" id="pg-cap-list">
          ${a}
        </div>
      </div>

      <!-- Right Panel: Capability Execution & Envelope Visualizer -->
      <div style="background: var(--surface-card); border: 1px solid var(--border); border-radius: var(--radius-md); display: flex; flex-direction: column; overflow: hidden;">
        <div style="padding: 14px 18px; border-bottom: 1px solid var(--border); display: flex; justify-content: space-between; align-items: center;">
          <div>
            <div style="font-size: 15px; font-weight: 700; color: var(--text-main); font-family: var(--ff-mono);" id="pg-selected-title">
              ${y(i?i.id:"No Capability Selected")}
            </div>
            <div style="font-size: 11.5px; color: var(--text-dim);" id="pg-selected-desc">
              ${y(i?i.summary||i.description:"Connect servers to inspect and execute tools")}
            </div>
          </div>
          <button class="btn btn-primary" onclick="window.app.executePlaygroundTool()" ${i?"":"disabled"}>
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polygon points="5 3 19 12 5 21 5 3"></polygon></svg>
            Execute Capability
          </button>
        </div>

        <div style="flex: 1; display: grid; grid-template-columns: 1fr 1fr; overflow: hidden;">
          <!-- Request Builder -->
          <div style="padding: 16px; border-right: 1px solid var(--border); overflow-y: auto;">
            <div class="form-group">
              <label class="form-label">Arguments JSON (Object)</label>
              <textarea class="form-textarea" rows="8" id="pg-args-input">${y(o)}</textarea>
            </div>
            <div class="form-group">
              <label class="form-label">Request Context / Operation ID (Optional)</label>
              <input type="text" class="form-input" id="pg-context-input" placeholder="e.g. op-dev-test-1">
            </div>
            ${i&&i.input_schema?`
              <div style="margin-top: 14px;">
                <label class="form-label">Input JSON Schema</label>
                <pre style="background: var(--surface); padding: 10px; border-radius: var(--radius-sm); border: 1px solid var(--border); font-size: 11px; color: var(--text-muted); max-height: 180px; overflow-y: auto;">${y(JSON.stringify(i.input_schema,null,2))}</pre>
              </div>
            `:""}
          </div>

          <!-- Response Inspector -->
          <div style="padding: 16px; background: var(--bg-app); display: flex; flex-direction: column; overflow: hidden;">
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
              <span style="font-size: 11px; font-weight: 600; color: var(--text-dim);">NORMALIZED EXECUTION ENVELOPE</span>
              <span id="pg-status-badge" style="font-size: 11px; font-weight: 600; color: ${e.executionResult?e.executionResult.status===200?"var(--green-400)":"var(--red-400)":"var(--text-dim)"}; font-family: var(--ff-mono);">
                ${e.executionResult?`HTTP ${e.executionResult.status} · ${e.executionResult.durationMs.toFixed(1)}ms`:"READY"}
              </span>
            </div>
            <pre id="pg-response-json" style="flex: 1; background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 14px; color: var(--amber-300); font-size: 11.5px; overflow-y: auto; margin: 0; white-space: pre-wrap; word-break: break-word;">${e.executionResult?y(JSON.stringify(e.executionResult.data,null,2)):"// Response envelope output will be formatted here"}</pre>
          </div>
        </div>
      </div>
    </div>
  `}function y(e){return String(e||"").replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;")}function z(e){let t=e.approvals.filter((o)=>o.status==="pending"),r=e.approvals.filter((o)=>o.status!=="pending"),i=t.length===0?`
    <div style="padding: 40px 20px; text-align: center; background: var(--surface-card); border-radius: var(--radius-md); border: 1px dashed var(--border);">
      <div style="width: 44px; height: 44px; border-radius: 50%; background: rgba(52, 211, 153, 0.12); border: 1px solid rgba(52, 211, 153, 0.3); display: flex; align-items: center; justify-content: center; margin: 0 auto 12px; color: var(--green-400); font-size: 18px; font-weight: 700;">
        ✓
      </div>
      <div style="font-size: 14px; font-weight: 600; color: var(--text-main); margin-bottom: 4px;">All Clear — No Pending Approvals</div>
      <div style="font-size: 11.5px; color: var(--text-dim); max-width: 460px; margin: 0 auto;">
        Tool calls intercepted by <code style="color: var(--amber-300); font-family: var(--ff-mono);">require_approval</code> policy rules will appear here for review and execution gating.
      </div>
    </div>
  `:t.map((o)=>`
    <div class="bento-card" style="border: 1px solid rgba(245, 158, 11, 0.35); background: var(--surface-card); margin-bottom: 14px; padding: 18px;">
      <div style="display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 12px;">
        <div>
          <div style="display: flex; align-items: center; gap: 8px;">
            <span class="brand-badge" style="background: rgba(245, 158, 11, 0.2); color: var(--amber-300); border-color: rgba(245, 158, 11, 0.5);">
              PENDING APPROVAL
            </span>
            <span style="font-family: var(--ff-mono); font-size: 12px; color: var(--text-muted);">${u(o.id)}</span>
          </div>
          <div style="margin-top: 6px; display: flex; align-items: center; gap: 8px;">
            <span style="font-size: 14.5px; font-weight: 700; color: var(--text-main); font-family: var(--ff-mono);">${u(o.capability_id)}</span>
            <span style="font-size: 11px; color: var(--text-dim);">via <span style="color: var(--cyan-400); font-family: var(--ff-mono);">${u(o.server_id)}</span></span>
          </div>
        </div>

        <div style="text-align: right; font-family: var(--ff-mono); font-size: 11px; color: var(--text-dim);">
          <div>Created: <span style="color: var(--text-muted);">${new Date(o.created_at*1000).toLocaleTimeString()}</span></div>
          <div style="color: var(--amber-400); margin-top: 2px;">Expires: ${new Date(o.expires_at*1000).toLocaleTimeString()}</div>
        </div>
      </div>

      <!-- Caller Context -->
      ${o.context||o.request_id?`
        <div style="background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 8px 12px; font-family: var(--ff-mono); font-size: 11px; display: flex; flex-wrap: wrap; gap: 16px; margin-bottom: 12px; color: var(--text-muted);">
          ${o.request_id?`<div><span style="color: var(--text-dim);">Request:</span> <span style="color: var(--text-main);">${u(o.request_id)}</span></div>`:""}
          ${o.context?.actor_id?`<div><span style="color: var(--text-dim);">Actor:</span> <span style="color: var(--cyan-400);">${u(o.context.actor_id)}</span></div>`:""}
          ${o.context?.operation_id?`<div><span style="color: var(--text-dim);">Operation:</span> <span style="color: var(--text-main);">${u(o.context.operation_id)}</span></div>`:""}
          ${o.context?.work_item_id?`<div><span style="color: var(--text-dim);">Work Item:</span> <span style="color: var(--text-main);">${u(o.context.work_item_id)}</span></div>`:""}
        </div>
      `:""}

      <!-- Arguments Editor -->
      <div style="margin-bottom: 14px;">
        <div style="font-size: 11px; font-weight: 600; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.5px; margin-bottom: 6px;">
          Parameters (Editable before approval)
        </div>
        <textarea id="appr-args-${o.id}" class="form-textarea" rows="4" style="color: var(--green-400); font-family: var(--ff-mono); font-size: 11.5px; line-height: 1.4;">${u(JSON.stringify(o.sanitized_args,null,2))}</textarea>
      </div>

      <!-- Action Footer -->
      <div style="display: flex; justify-content: space-between; align-items: center; border-top: 1px solid var(--border); padding-top: 12px;">
        <div style="display: flex; align-items: center; gap: 8px;">
          <input id="appr-operator-${o.id}" type="text" class="form-input" placeholder="Operator ID" value="security-operator" style="width: 200px; padding: 5px 10px; font-size: 11px;">
        </div>

        <div style="display: flex; align-items: center; gap: 8px;">
          <button class="btn btn-danger" onclick="window.app.promptReject('${u(o.id)}')">
            ✕ Reject
          </button>
          <button class="btn btn-primary" onclick="window.app.submitApproval('${u(o.id)}')">
            ✓ Approve &amp; Execute
          </button>
        </div>
      </div>
    </div>
  `).join(""),a=r.length===0?"":`
    <div style="margin-top: 32px; border-top: 1px solid var(--border); padding-top: 18px;">
      <div style="font-size: 11px; font-weight: 700; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.5px; margin-bottom: 12px;">
        Recent History (${r.length})
      </div>
      <div style="background: var(--surface-card); border: 1px solid var(--border); border-radius: var(--radius-md); overflow: hidden; font-family: var(--ff-mono); font-size: 11.5px;">
        ${r.map((o)=>`
          <div style="display: flex; justify-content: space-between; align-items: center; padding: 10px 14px; border-bottom: 1px solid var(--border);">
            <div style="display: flex; align-items: center; gap: 10px;">
              <span class="brand-badge" style="${o.status==="approved"?"background: rgba(52, 211, 153, 0.12); color: var(--green-400); border-color: rgba(52, 211, 153, 0.3);":o.status==="rejected"?"background: rgba(248, 113, 113, 0.12); color: var(--red-400); border-color: rgba(248, 113, 113, 0.3);":"background: var(--surface-hover); color: var(--text-dim);"}">
                ${o.status.toUpperCase()}
              </span>
              <span style="font-weight: 600; color: var(--text-main);">${u(o.capability_id)}</span>
              <span style="color: var(--text-dim); font-size: 10.5px;">${u(o.id)}</span>
            </div>
            <div style="display: flex; align-items: center; gap: 16px; color: var(--text-dim);">
              ${o.operator?`<span>Operator: <span style="color: var(--text-muted);">${u(o.operator)}</span></span>`:""}
              ${o.reason?`<span style="color: var(--red-400); font-style: italic;">"${u(o.reason)}"</span>`:""}
              <span>${new Date(o.created_at*1000).toLocaleTimeString()}</span>
            </div>
          </div>
        `).join("")}
      </div>
    </div>
  `;return`
    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px; padding-bottom: 14px; border-bottom: 1px solid var(--border);">
      <div>
        <div style="font-size: 16px; font-weight: 700; color: var(--text-main); display: flex; align-items: center; gap: 8px;">
          <span>\uD83D\uDEE1️ Human-in-the-Loop Review Queue</span>
          <span class="brand-badge" style="font-size: 10px; padding: 2px 8px;">
            ${t.length} Pending
          </span>
        </div>
        <div style="font-size: 11.5px; color: var(--text-dim); margin-top: 3px;">
          Inspect, parameter-tweak, and approve or reject sensitive capability executions in real-time.
        </div>
      </div>
      <button class="btn btn-ghost" onclick="window.app.refreshApprovals()" style="font-size: 11.5px;">
        <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21.5 2v6h-6M21.34 15.57a10 10 0 1 1-.57-8.38l5.67-5.67"/></svg>
        Refresh Queue
      </button>
    </div>

    <div style="font-size: 11px; font-weight: 700; color: var(--amber-400); text-transform: uppercase; letter-spacing: 0.5px; margin-bottom: 12px; display: flex; align-items: center; gap: 6px;">
      <span>⚡ Awaiting Operator Decision (${t.length})</span>
    </div>

    <div>
      ${i}
    </div>

    ${a}
  `}function u(e){return String(e||"").replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;")}function L(){let t=l.getState().config.policy||{},r=t.allow||[],i=t.deny||[],a=t.redact_keys||t.redactKeys||[],o=t.require_approval||t.requireApproval||[],s=r.length===0?`
    <div style="color: var(--text-dim); font-size: 12px;">No allow list (all non-denied operations permitted)</div>
  `:r.map((v,g)=>`
    <div style="display: flex; justify-content: space-between; align-items: center; background: var(--surface); padding: 8px 12px; border-radius: var(--radius-sm); border: 1px solid var(--border);">
      <span style="font-family: var(--ff-mono); font-size: 12px; color: var(--green-400);">✔ ${k(v)}</span>
      <button class="btn btn-ghost" style="padding: 2px 6px; font-size: 11px; color: var(--red-400);" onclick="window.app.removePolicyRule('allow', ${g})">✕</button>
    </div>
  `).join(""),n=i.length===0?`
    <div style="color: var(--text-dim); font-size: 12px;">No deny rules configured</div>
  `:i.map((v,g)=>`
    <div style="display: flex; justify-content: space-between; align-items: center; background: var(--surface); padding: 8px 12px; border-radius: var(--radius-sm); border: 1px solid var(--border);">
      <span style="font-family: var(--ff-mono); font-size: 12px; color: var(--red-400);">✖ ${k(v)}</span>
      <button class="btn btn-ghost" style="padding: 2px 6px; font-size: 11px; color: var(--red-400);" onclick="window.app.removePolicyRule('deny', ${g})">✕</button>
    </div>
  `).join(""),p=o.length===0?`
    <div style="color: var(--text-dim); font-size: 12px;">No human-in-the-loop approval rules configured</div>
  `:o.map((v,g)=>`
    <div style="display: flex; justify-content: space-between; align-items: center; background: var(--surface); padding: 8px 12px; border-radius: var(--radius-sm); border: 1px solid var(--border);">
      <span style="font-family: var(--ff-mono); font-size: 12px; color: var(--amber-400);">\uD83D\uDEE1️ ${k(v)}</span>
      <button class="btn btn-ghost" style="padding: 2px 6px; font-size: 11px; color: var(--red-400);" onclick="window.app.removePolicyRule('requireApproval', ${g})">✕</button>
    </div>
  `).join(""),m=a.length===0?`
    <div style="color: var(--text-dim); font-size: 12px;">No key redaction patterns configured</div>
  `:a.map((v,g)=>`
    <span class="brand-badge" style="color: var(--amber-300); padding: 5px 10px; font-size: 11px; display: inline-flex; align-items: center; gap: 6px;">
      ${k(v)}
      <span style="cursor: pointer; color: var(--red-400); font-weight: bold;" onclick="window.app.removePolicyRule('redact', ${g})">✕</span>
    </span>
  `).join("");return`
    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 18px;">
      <div>
        <div style="font-size: 16px; font-weight: 700; color: var(--text-main);">Security Governance, Approvals &amp; Data Redaction</div>
        <div style="font-size: 11px; color: var(--text-dim);">Wildcard capability access control, human-in-the-loop triggers, and sensitive key masking</div>
      </div>
    </div>

    <div class="bento-grid">
      <!-- Allow Rules -->
      <div class="bento-card col-4">
        <div class="stat-header">
          <span class="stat-label" style="color: var(--green-400);">Allow List Patterns</span>
        </div>
        <div style="display: flex; flex-direction: column; gap: 8px; margin: 12px 0;">
          ${s}
        </div>
        <div style="display: flex; gap: 8px; margin-top: 14px;">
          <input type="text" class="form-input" id="policy-new-allow" placeholder="e.g. github.*, db.read_*" onkeydown="if(event.key==='Enter') window.app.submitPolicyRule('allow')">
          <button class="btn btn-ghost" onclick="window.app.submitPolicyRule('allow')">Add Allow</button>
        </div>
      </div>

      <!-- Deny Rules -->
      <div class="bento-card col-4">
        <div class="stat-header">
          <span class="stat-label" style="color: var(--red-400);">Deny List Patterns (Strict Precedence)</span>
        </div>
        <div style="display: flex; flex-direction: column; gap: 8px; margin: 12px 0;">
          ${n}
        </div>
        <div style="display: flex; gap: 8px; margin-top: 14px;">
          <input type="text" class="form-input" id="policy-new-deny" placeholder="e.g. *.drop_*, filesystem.write_*" onkeydown="if(event.key==='Enter') window.app.submitPolicyRule('deny')">
          <button class="btn btn-ghost" onclick="window.app.submitPolicyRule('deny')">Add Deny</button>
        </div>
      </div>

      <!-- Require Approval Rules -->
      <div class="bento-card col-4">
        <div class="stat-header">
          <span class="stat-label" style="color: var(--amber-400);">Human-in-the-Loop Triggers</span>
        </div>
        <div style="display: flex; flex-direction: column; gap: 8px; margin: 12px 0;">
          ${p}
        </div>
        <div style="display: flex; gap: 8px; margin-top: 14px;">
          <input type="text" class="form-input" id="policy-new-requireApproval" placeholder="e.g. docker.run*, db.write*" onkeydown="if(event.key==='Enter') window.app.submitPolicyRule('requireApproval')">
          <button class="btn btn-ghost" onclick="window.app.submitPolicyRule('requireApproval')">Add Approval</button>
        </div>
      </div>

      <!-- Key Redaction -->
      <div class="bento-card col-12">
        <div class="stat-header">
          <span class="stat-label" style="color: var(--amber-300);">Sensitive Key Redaction Patterns</span>
          <span style="font-size: 11px; color: var(--text-dim); margin-left: 8px;">Keys automatically masked as &lt;redacted&gt; in logs and envelopes</span>
        </div>
        <div style="display: flex; gap: 8px; flex-wrap: wrap; margin: 12px 0;">
          ${m}
        </div>
        <div style="display: flex; gap: 8px; margin-top: 14px; max-width: 420px;">
          <input type="text" class="form-input" id="policy-new-redact" placeholder="e.g. token, api_key, password, secret" onkeydown="if(event.key==='Enter') window.app.submitPolicyRule('redact')">
          <button class="btn btn-ghost" onclick="window.app.submitPolicyRule('redact')">Add Key</button>
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
  `}function k(e){return String(e||"").replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;")}function I(){let e=l.getState(),t=e.config,r=Object.entries(t.capabilityAliases||{}),i=Object.entries(t.resourceAliases||{}),a=Object.entries(t.promptAliases||{}),o="";if(r.length===0&&i.length===0&&a.length===0)o=`
      <div style="padding: 24px; text-align: center; color: var(--text-dim);">
        No facade aliases configured in ${f(e.configPath)}. Add short names to prune token payload sizes.
      </div>
    `;else{for(let[s,n]of r)o+=`
        <div class="feed-row" style="grid-template-columns: 90px 180px 1fr 80px;">
          <span style="color: var(--cyan-400);">Tool</span>
          <span style="font-weight: 700; color: var(--text-main); font-family: var(--ff-mono);">${f(s)}</span>
          <span style="color: var(--text-dim); font-family: var(--ff-mono);">${f(n)}</span>
          <div style="text-align: right;">
            <button class="btn btn-ghost" style="padding: 2px 6px; color: var(--red-400);" onclick="window.app.deleteAlias('tool', '${f(s)}')">✕</button>
          </div>
        </div>
      `;for(let[s,n]of i)o+=`
        <div class="feed-row" style="grid-template-columns: 90px 180px 1fr 80px;">
          <span style="color: var(--green-400);">Resource</span>
          <span style="font-weight: 700; color: var(--text-main); font-family: var(--ff-mono);">${f(s)}</span>
          <span style="color: var(--text-dim); font-family: var(--ff-mono);">${f(n)}</span>
          <div style="text-align: right;">
            <button class="btn btn-ghost" style="padding: 2px 6px; color: var(--red-400);" onclick="window.app.deleteAlias('resource', '${f(s)}')">✕</button>
          </div>
        </div>
      `;for(let[s,n]of a)o+=`
        <div class="feed-row" style="grid-template-columns: 90px 180px 1fr 80px;">
          <span style="color: var(--amber-300);">Prompt</span>
          <span style="font-weight: 700; color: var(--text-main); font-family: var(--ff-mono);">${f(s)}</span>
          <span style="color: var(--text-dim); font-family: var(--ff-mono);">${f(n)}</span>
          <div style="text-align: right;">
            <button class="btn btn-ghost" style="padding: 2px 6px; color: var(--red-400);" onclick="window.app.deleteAlias('prompt', '${f(s)}')">✕</button>
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
    <div class="bento-card" style="margin-bottom: 20px; overflow: visible;">
      <div class="stat-header" style="margin-bottom: 12px;">
        <span class="stat-label">Create New Alias</span>
      </div>
      <div style="display: grid; grid-template-columns: 140px 1fr 1fr 100px; gap: 10px; align-items: center; position: relative;">
        <select class="form-input" id="alias-kind">
          <option value="tool">Tool / Capability</option>
          <option value="resource">Resource</option>
          <option value="prompt">Prompt</option>
        </select>
        <input type="text" class="form-input" id="alias-name" placeholder="Public alias (e.g. db.query)" onkeydown="if(event.key==='Enter') window.app.createAlias()">
        <div style="position: relative; width: 100%;">
          <input type="text" class="form-input" id="alias-target" autocomplete="off" placeholder="Target ID (e.g. docker.list_containers)" style="width: 100%;" oninput="window.app.handleAliasTargetInput(this.value)" onkeydown="if(event.key==='Enter') window.app.createAlias()" onfocus="window.app.handleAliasTargetInput(this.value)" onblur="setTimeout(() => window.app.hideAliasDropdown(), 200)">
          <div id="alias-suggestions-dropdown" style="display: none; position: absolute; top: calc(100% + 4px); left: 0; right: 0; max-height: 240px; overflow-y: auto; background: var(--surface-elevated); border: 1px solid var(--border); border-radius: var(--radius-sm); box-shadow: 0 8px 24px rgba(0,0,0,0.4); z-index: 1000; font-family: var(--ff-mono); font-size: 11.5px;"></div>
        </div>
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
      ${o}
    </div>
  `}function f(e){return String(e||"").replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;")}var $=[{id:"github",name:"GitHub",category:"devtools",description:"Explore repositories, issues, pull requests, branches, and commit histories.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-github"],envFields:[{key:"GITHUB_PERSONAL_ACCESS_TOKEN",label:"GitHub Personal Access Token",placeholder:"ghp_...",required:!0,description:"Classic or fine-grained token with repo scope."}]},{id:"git",name:"Git (Local)",category:"devtools",description:"Read local Git repository status, diffs, log histories, and commit changes.",badge:"Official / uvx",command:"uvx",defaultArgs:["mcp-server-git","--repository","."],argsPlaceholder:"mcp-server-git --repository /path/to/repo",envFields:[]},{id:"filesystem",name:"Filesystem",category:"devtools",description:"Secure, sandboxed access to local files and directories for AI workflows.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-filesystem","."],argsPlaceholder:"-y @modelcontextprotocol/server-filesystem /allowed/dir1 /allowed/dir2",envFields:[]},{id:"memory",name:"Memory Graph",category:"devtools",description:"Persistent knowledge-graph based memory for multi-turn agent learning.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-memory"],envFields:[]},{id:"chrome-devtools",name:"Chrome DevTools",category:"devtools",description:"Inspect live DOM, execute scripts, read console logs, and capture network traces in Chrome.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-puppeteer"],envFields:[]},{id:"sentry",name:"Sentry",category:"devtools",description:"Query production error events, stack traces, and issue frequencies directly from Sentry.",badge:"uvx / Telemetry",command:"uvx",defaultArgs:["mcp-server-sentry"],envFields:[{key:"SENTRY_AUTH_TOKEN",label:"Sentry Auth Token",placeholder:"sntrys_...",required:!0}]},{id:"playwright",name:"Playwright Browser",category:"browser",description:"Headless / headed browser automation for scraping, form filling, and UI interaction.",badge:"Popular #1 / npx",command:"npx",defaultArgs:["-y","@executeautomation/playwright-mcp-server"],envFields:[]},{id:"puppeteer",name:"Puppeteer",category:"browser",description:"Official browser automation server for web page scraping and screenshot capture.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-puppeteer"],envFields:[]},{id:"brave-search",name:"Brave Search",category:"browser",description:"Real-time privacy-preserving web search and local point-of-interest query engine.",badge:"Official / Search",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-brave-search"],envFields:[{key:"BRAVE_API_KEY",label:"Brave Search API Key",placeholder:"BSA...",required:!0}]},{id:"tavily",name:"Tavily Search",category:"browser",description:"AI-optimized web search engine structured specifically for LLM context injection.",badge:"Community / Stdio",command:"npx",defaultArgs:["-y","@tavily/mcp-server"],envFields:[{key:"TAVILY_API_KEY",label:"Tavily API Key",placeholder:"tvly-...",required:!0}]},{id:"fetch",name:"Fetch / Web Markdown",category:"browser",description:"Download web pages, strip clutter, and convert raw HTML to clean markdown text.",badge:"Official / uvx",command:"uvx",defaultArgs:["mcp-server-fetch"],envFields:[]},{id:"postgres",name:"PostgreSQL",category:"database",description:"Read schemas, inspect tables, and execute SQL queries against PostgreSQL databases.",badge:"Official / Database",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-postgres","postgresql://user:pass@localhost:5432/mydb"],argsPlaceholder:"-y @modelcontextprotocol/server-postgres postgresql://user:pass@localhost:5432/dbname",envFields:[]},{id:"sqlite",name:"SQLite",category:"database",description:"Local embedded SQLite query runner and schema inspector.",badge:"Official / uvx",command:"uvx",defaultArgs:["mcp-server-sqlite","--db-path","./app.db"],argsPlaceholder:"mcp-server-sqlite --db-path /path/to/database.sqlite",envFields:[]},{id:"supabase",name:"Supabase",category:"database",description:"Query database tables, manage auth policies, and inspect storage in Supabase.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@supabase/mcp-server"],envFields:[{key:"SUPABASE_ACCESS_TOKEN",label:"Supabase Personal Access Token",placeholder:"sbp_...",required:!0},{key:"SUPABASE_PROJECT_REF",label:"Supabase Project Reference ID",placeholder:"abcdefghijklmnop",required:!1}]},{id:"redis",name:"Redis",category:"database",description:"Inspect cached keys, hash sets, lists, TTLs, and pub/sub channels in Redis.",badge:"uvx / Key-Value",command:"uvx",defaultArgs:["mcp-server-redis","--url","redis://localhost:6379"],argsPlaceholder:"mcp-server-redis --url redis://localhost:6379",envFields:[]},{id:"s3",name:"AWS S3 / Cloud Storage",category:"database",description:"Browse S3 buckets, fetch object metadata, and download files from cloud storage.",badge:"uvx / Cloud Storage",command:"uvx",defaultArgs:["mcp-server-s3","--bucket","my-bucket-name"],argsPlaceholder:"mcp-server-s3 --bucket bucket-name --region us-east-1",envFields:[{key:"AWS_ACCESS_KEY_ID",label:"AWS Access Key ID",placeholder:"AKIA...",required:!0},{key:"AWS_SECRET_ACCESS_KEY",label:"AWS Secret Access Key",placeholder:"...",required:!0},{key:"AWS_REGION",label:"AWS Region",placeholder:"us-east-1",required:!1}]},{id:"linear",name:"Linear",category:"productivity",description:"Search, create, and triage Linear issues, cycles, teams, and project roadmaps.",badge:"Productivity / Stdio",command:"npx",defaultArgs:["-y","mcp-linear"],envFields:[{key:"LINEAR_API_KEY",label:"Linear API Key",placeholder:"lin_api_...",required:!0}]},{id:"slack",name:"Slack",category:"productivity",description:"Read channels, post messages, inspect threads, and search team discussions.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-slack"],envFields:[{key:"SLACK_BOT_TOKEN",label:"Slack Bot User Token",placeholder:"xoxb-...",required:!0},{key:"SLACK_TEAM_ID",label:"Slack Team ID",placeholder:"T01234567",required:!0}]},{id:"notion",name:"Notion",category:"productivity",description:"Search Notion workspace pages, read nested blocks, and query database entries.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-notion"],envFields:[{key:"NOTION_API_KEY",label:"Notion Internal Integration Token",placeholder:"secret_...",required:!0}]},{id:"jira",name:"Jira / Atlassian",category:"productivity",description:"Manage Jira issues, search JQL, read sprint statuses, and inspect boards.",badge:"uvx / Atlassian",command:"uvx",defaultArgs:["mcp-server-jira","--url","https://your-domain.atlassian.net","--email","user@example.com"],argsPlaceholder:"mcp-server-jira --url https://org.atlassian.net --email me@org.com",envFields:[{key:"JIRA_API_TOKEN",label:"Atlassian API Token",placeholder:"ATATT3...",required:!0}]},{id:"google-drive",name:"Google Drive",category:"productivity",description:"Search, list, and read documents, spreadsheets, and drive files.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-gdrive"],envFields:[{key:"GOOGLE_APPLICATION_CREDENTIALS",label:"Google Credentials JSON Path",placeholder:"/path/to/credentials.json",required:!0}]},{id:"docker",name:"Docker",category:"cloud",description:"Inspect running containers, tail container logs, list images, and manage compose services.",badge:"uvx / DevOps",command:"uvx",defaultArgs:["mcp-server-docker"],envFields:[]},{id:"kubernetes",name:"Kubernetes (K8s)",category:"cloud",description:"Query cluster pods, services, deployment status, and inspect Kubernetes logs.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-kubernetes"],envFields:[{key:"KUBECONFIG",label:"Kubeconfig File Path (Optional)",placeholder:"~/.kube/config",required:!1}]},{id:"cloudflare",name:"Cloudflare",category:"cloud",description:"Manage Cloudflare Workers, KV namespaces, D1 databases, Vectorize indexes, and DNS.",badge:"Official / Cloudflare",command:"npx",defaultArgs:["-y","@cloudflare/mcp-server-cloudflare"],envFields:[{key:"CLOUDFLARE_API_TOKEN",label:"Cloudflare API Token",placeholder:"...",required:!0},{key:"CLOUDFLARE_ACCOUNT_ID",label:"Cloudflare Account ID",placeholder:"...",required:!0}]},{id:"terraform",name:"Terraform",category:"cloud",description:"Inspect Terraform state files, resource dependency graphs, and plan previews.",badge:"uvx / IaC",command:"uvx",defaultArgs:["mcp-server-terraform"],envFields:[]}];class N{activeTemplateCategory="all";activeTemplateFilter="";selectedTemplate=null;async init(){let e=window.location.port?`:${window.location.port}`:"",t=document.getElementById("daemon-port-label");if(t)t.textContent=`Daemon ${e}`;await this.refreshData(),this.initSSE(),this.render(),l.subscribe(()=>{this.render()})}async refreshData(){try{let[e,t,r]=await Promise.all([c.getConfig(),c.listCapabilities(),c.listApprovals()]);if(e.ok)l.setState({configPath:e.config_path,config:e.config,serverStatuses:e.server_statuses||{},metrics:{totalCatalogRequests:e.metrics?.total_catalog_requests||0,totalEtagHits:e.metrics?.total_etag_hits||0,totalToolCalls:e.metrics?.total_tool_calls||0,totalToolDurationUs:e.metrics?.total_tool_duration_us||0}});if(t&&Array.isArray(t.capabilities))l.setState({capabilities:t.capabilities});if(r&&Array.isArray(r.approvals))l.setState({approvals:r.approvals})}catch(e){console.error("Failed to fetch daemon state:",e)}}async refreshApprovals(){try{let e=await c.listApprovals();if(e&&Array.isArray(e.approvals))l.setState({approvals:e.approvals})}catch(e){console.error("Failed to refresh approvals:",e)}}initSSE(){try{let e=new EventSource("/v1/resources/updates");e.onmessage=(t)=>{l.addEventLog("SSE","/v1/resources/updates","UPDATED","0.1ms"),this.refreshData()}}catch(e){console.warn("SSE connection unavailable")}}switchTab(e){l.setState({activeTab:e}),this.refreshData()}render(){let e=l.getState(),t=document.getElementById("app-main");if(!t)return;let r=e.approvals.filter((s)=>s.status==="pending").length,i=document.getElementById("nav-approvals-badge");if(i)i.textContent=r>0?`${r}`:"",i.style.display=r>0?"inline-block":"none";document.querySelectorAll(".nav-item").forEach((s)=>{if(s.getAttribute("data-tab")===e.activeTab)s.classList.add("active");else s.classList.remove("active")});let a=document.getElementById("top-title"),o={overview:"Overview Cockpit",servers:"Server Hub & Connections",playground:"MCP Capability Playground",approvals:"Human-in-the-Loop Review Queue",policy:"Security Governance & Redaction",aliases:"Facade & Alias Studio"};if(a)a.textContent=o[e.activeTab];switch(e.activeTab){case"overview":t.innerHTML=O();break;case"servers":t.innerHTML=j();break;case"playground":t.innerHTML=R();break;case"approvals":t.innerHTML=z(e);break;case"policy":t.innerHTML=L();break;case"aliases":t.innerHTML=I();break}}async submitApproval(e){let t=document.getElementById(`appr-operator-${e}`),r=document.getElementById(`appr-args-${e}`),i=t?.value.trim()||"security-operator",a=void 0;if(r&&r.value.trim())try{a=JSON.parse(r.value.trim())}catch{alert("Invalid JSON in arguments editor");return}let o=await c.approveTicket(e,i,a);if(o.ok)await this.refreshApprovals();else alert(`Approval failed: ${o.error||"Unknown error"}`)}async promptReject(e){let t=prompt("Reason for rejection (will be returned to the calling agent):");if(t===null)return;let i=document.getElementById(`appr-operator-${e}`)?.value.trim()||"security-operator",a=await c.rejectTicket(e,i,t);if(a.ok)await this.refreshApprovals();else alert(`Rejection failed: ${a.error||"Unknown error"}`)}selectCapability(e){l.setState({selectedCapabilityId:e})}filterCapabilities(e){let t=e.toLowerCase().trim(),i=l.getState().capabilities.filter((o)=>o.id.toLowerCase().includes(t)||o.summary&&o.summary.toLowerCase().includes(t)||o.server&&o.server.toLowerCase().includes(t)),a=document.getElementById("pg-cap-list");if(a)if(i.length===0)a.innerHTML=`
          <div style="padding: 24px 16px; text-align: center; color: var(--text-dim); font-size: 11.5px;">
            No capabilities match "${d(e)}"
          </div>
        `;else a.innerHTML=i.map((o)=>`
          <div class="cap-item ${o.id===l.getState().selectedCapabilityId?"active":""}" onclick="window.app.selectCapability('${d(o.id)}')">
            <div style="display: flex; justify-content: space-between; align-items: center;">
              <span style="font-weight: 600; color: var(--text-main); font-family: var(--ff-mono); font-size: 12px;">${d(o.id)}</span>
              <span style="font-size: 10px; color: var(--green-400);">${d(o.mode||"read")}</span>
            </div>
            <div style="font-size: 11px; color: var(--text-dim); margin-top: 2px;">server: ${d(o.server||"local")}</div>
          </div>
        `).join("")}async executePlaygroundTool(){let e=l.getState(),t=e.selectedCapabilityId||(e.capabilities[0]?e.capabilities[0].id:null);if(!t)return;let r=document.getElementById("pg-args-input")?.value||"{}",i=document.getElementById("pg-context-input")?.value||void 0,a={};try{a=JSON.parse(r)}catch{alert("Invalid arguments JSON object");return}let o=document.getElementById("pg-status-badge"),s=document.getElementById("pg-response-json");if(o)o.textContent="EXECUTING...",o.style.color="var(--amber-400)";try{let n=await c.callCapability({capability_id:t,args:a,context:i?{operation_id:i}:void 0,request_id:`ui-req-${Date.now()}`});l.setState({executionResult:{status:n.status,durationMs:n.durationMs,data:n.data}}),l.addEventLog("POST",`/v1/tools/call → ${t}`,n.status===200?"200 OK":`HTTP ${n.status}`,`${n.durationMs.toFixed(1)}ms`)}catch(n){if(o)o.textContent="ERROR",o.style.color="var(--red-400)";if(s)s.textContent=n.toString()}}async submitPolicyRule(e){let t=e==="allow"?"policy-new-allow":e==="deny"?"policy-new-deny":"policy-new-redact",r=document.getElementById(t);if(!r)return;let i=r.value.trim();if(!i)return;await this.addPolicyRule(e,i),r.value=""}async addPolicyRule(e,t){let r=(t||"").trim();if(!r)return;let a=l.getState().config.policy||{},o=[...a.allow||[]],s=[...a.deny||[]],n=[...a.redact_keys||a.redactKeys||[]];if(e==="allow"&&!o.includes(r))o.push(r);if(e==="deny"&&!s.includes(r))s.push(r);if(e==="redact"&&!n.includes(r))n.push(r);let p=await c.savePolicy({allow:o,deny:s,redact_keys:n,redactKeys:n});if(!p.ok)alert(`Failed to save policy rule: ${p.error||"Unknown error"}`);await this.refreshData()}async removePolicyRule(e,t){let i=l.getState().config.policy||{},a=[...i.allow||[]],o=[...i.deny||[]],s=[...i.redact_keys||i.redactKeys||[]];if(e==="allow")a.splice(t,1);if(e==="deny")o.splice(t,1);if(e==="redact")s.splice(t,1);let n=await c.savePolicy({allow:a,deny:o,redact_keys:s,redactKeys:s});if(!n.ok)alert(`Failed to update policy: ${n.error||"Unknown error"}`);await this.refreshData()}testPolicySandbox(e){let t=document.getElementById("policy-test-verdict");if(!t)return;let r=e.trim();if(!r){t.textContent="ENTER ID",t.style.color="var(--text-dim)";return}let a=l.getState().config.policy||{},o=a.deny||[],s=a.allow||[],n=(p,m)=>{if(p==="*")return!0;if(p.endsWith("*"))return m.startsWith(p.slice(0,-1));return p===m};if(o.some((p)=>n(p,r))){t.textContent="DENIED (Strict Block)",t.style.color="var(--red-400)";return}if(s.length>0&&!s.some((p)=>n(p,r))){t.textContent="DENIED (Not in Allow List)",t.style.color="var(--red-400)";return}t.textContent="ALLOWED",t.style.color="var(--green-400)"}async deleteServer(e){if(!confirm(`Are you sure you want to remove server '${e}' from config?`))return;await c.deleteServer(e),await this.refreshData()}openAddServerModal(){this.closeModals();let e=document.getElementById("modal-add-server");if(e)e.classList.add("active")}async submitAddServer(){let e=document.getElementById("modal-srv-name")?.value.trim(),t=document.getElementById("modal-srv-transport")?.value;if(!e){alert("Server name is required");return}let r={};if(t==="stdio"){let o=(document.getElementById("modal-srv-command")?.value.trim()).split(/\s+/).filter(Boolean);if(o.length===0){alert("Command is required");return}r.command=o[0],r.args=o.slice(1)}else{let a=document.getElementById("modal-srv-url")?.value.trim();if(!a){alert("URL is required");return}r.url=a}let i=await c.upsertServer(e,r);if(i.ok)this.closeModals(),await this.refreshData();else alert(`Failed to save server: ${i.error}`)}openTemplateCatalog(){this.closeModals();let e=document.getElementById("modal-templates");if(e)e.classList.add("active");this.renderTemplateGrid()}setTemplateCategory(e){this.activeTemplateCategory=e,document.querySelectorAll(".tmpl-cat-btn").forEach((t)=>{if(t.getAttribute("data-category")===e)t.classList.add("active"),t.style.background="var(--surface-elevated)",t.style.color="var(--amber-400)";else t.classList.remove("active"),t.style.background="var(--surface-card)",t.style.color="var(--text-main)"}),this.renderTemplateGrid()}filterTemplates(e){this.activeTemplateFilter=e.toLowerCase().trim(),this.renderTemplateGrid()}renderTemplateGrid(){let e=document.getElementById("tmpl-grid");if(!e)return;let t=$.filter((a)=>{let o=this.activeTemplateCategory==="all"||a.category===this.activeTemplateCategory,s=!this.activeTemplateFilter||a.name.toLowerCase().includes(this.activeTemplateFilter)||a.id.toLowerCase().includes(this.activeTemplateFilter)||a.description.toLowerCase().includes(this.activeTemplateFilter)||a.command.toLowerCase().includes(this.activeTemplateFilter)||a.envFields.some((n)=>n.key.toLowerCase().includes(this.activeTemplateFilter));return o&&s});if(t.length===0){e.innerHTML=`
        <div style="grid-column: span 2; padding: 32px; text-align: center; color: var(--text-dim);">
          No matching MCP server templates found.
        </div>
      `;return}let i=l.getState().config.mcpServers||{};e.innerHTML=t.map((a)=>{let o=!!i[a.id],s=`${a.command} ${a.defaultArgs.join(" ")}`;return`
        <div class="bento-card" style="display: flex; flex-direction: column; justify-content: space-between; padding: 14px; background: var(--surface); border: 1px solid var(--border); transition: transform 0.15s, border-color 0.15s;">
          <div>
            <div style="display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 6px;">
              <div style="display: flex; align-items: center; gap: 8px;">
                <span style="font-weight: 700; font-size: 13.5px; color: var(--text-main);">${d(a.name)}</span>
                <span class="brand-badge" style="font-size: 9.5px; padding: 1px 6px;">${d(a.badge)}</span>
              </div>
              ${o?'<span style="font-size: 10px; color: var(--green-400); font-weight: 600;">CONNECTED</span>':""}
            </div>
            <div style="font-size: 11.5px; color: var(--text-muted); line-height: 1.4; margin-bottom: 8px;">
              ${d(a.description)}
            </div>
            <div style="font-family: var(--ff-mono); font-size: 10.5px; color: var(--text-dim); background: var(--surface-card); padding: 5px 8px; border-radius: var(--radius-sm); border: 1px solid var(--border-subtle); overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">
              <code>${d(s)}</code>
            </div>
            ${a.envFields.length>0?`
              <div style="font-size: 10.5px; color: var(--amber-400); margin-top: 6px; display: flex; align-items: center; gap: 4px;">
                <span>⚡ Needs:</span>
                <code>${a.envFields.map((n)=>d(n.key)).join(", ")}</code>
              </div>
            `:""}
          </div>

          <div style="display: flex; justify-content: flex-end; margin-top: 12px; gap: 6px;">
            <button class="btn btn-primary" style="font-size: 11.5px; padding: 4px 10px;" onclick="window.app.selectTemplate('${d(a.id)}')">
              ${o?"Configure Another":"✨ 1-Click Setup"}
            </button>
          </div>
        </div>
      `}).join("")}selectTemplate(e){let t=$.find((s)=>s.id===e);if(!t)return;this.selectedTemplate=t,this.closeModals();let r=document.getElementById("modal-configure-template");if(r)r.classList.add("active");let i=document.getElementById("cfg-tmpl-title"),a=document.getElementById("cfg-tmpl-desc"),o=document.getElementById("cfg-tmpl-form");if(i)i.textContent=`Configure ${t.name} Server`;if(a)a.textContent=t.description;if(o){let s="";if(t.envFields.length>0)s=`
          <div style="margin-top: 14px; margin-bottom: 6px; font-weight: 700; font-size: 11px; text-transform: uppercase; color: var(--amber-400); letter-spacing: 0.5px;">
            Environment Variables &amp; API Keys
          </div>
          ${t.envFields.map((n)=>`
            <div class="form-group">
              <label class="form-label">${d(n.label)} ${n.required?'<span style="color: var(--red-400);">*</span>':"(Optional)"}</label>
              <input type="password" class="form-input tmpl-env-input" data-key="${d(n.key)}" placeholder="${d(n.placeholder||"")}">
              ${n.description?`<div style="font-size: 10.5px; color: var(--text-dim); margin-top: 3px;">${d(n.description)}</div>`:""}
            </div>
          `).join("")}
        `;o.innerHTML=`
        <div class="form-group">
          <label class="form-label">Server Identifier (Name)</label>
          <input type="text" class="form-input" id="cfg-srv-id" value="${d(t.id)}">
        </div>
        <div class="form-group">
          <label class="form-label">Command Line Arguments</label>
          <input type="text" class="form-input" id="cfg-srv-args" value="${d(t.defaultArgs.join(" "))}" placeholder="${d(t.argsPlaceholder||"")}">
          <div style="font-size: 10.5px; color: var(--text-dim); margin-top: 3px;">Executable: <code>${d(t.command)}</code></div>
        </div>
        ${s}
      `}}async submitTemplateServer(){if(!this.selectedTemplate)return;let e=this.selectedTemplate,t=document.getElementById("cfg-srv-id")?.value.trim(),r=document.getElementById("cfg-srv-args")?.value.trim();if(!t){alert("Server identifier is required");return}let i=r?r.split(/\s+/).filter(Boolean):[],a={},o=document.querySelectorAll(".tmpl-env-input");for(let p of Array.from(o)){let m=p.getAttribute("data-key"),v=p.value.trim(),g=e.envFields.find((A)=>A.key===m);if(g?.required&&!v){alert(`Required field '${g.label}' is missing.`);return}if(m&&v)a[m]=v}let s={command:e.command,args:i};if(Object.keys(a).length>0)s.env=a;let n=await c.upsertServer(t,s);if(n.ok)this.closeModals(),await this.refreshData();else alert(`Failed to save server: ${n.error}`)}async openImportModal(){this.closeModals();let e=document.getElementById("modal-import");if(e)e.classList.add("active");let t=document.getElementById("modal-eco-list");if(!t)return;t.innerHTML='<div style="color: var(--text-dim); padding: 12px; text-align: center;">Scanning IDE configs...</div>';try{let r=await c.getEcosystemSources();if(r.sources&&r.sources.length>0)t.innerHTML=r.sources.map((i)=>`
          <label style="display: flex; align-items: center; gap: 10px; background: var(--surface); padding: 10px 14px; border-radius: var(--radius-sm); border: 1px solid var(--border); cursor: pointer;">
            <input type="checkbox" class="eco-checkbox" value="${i.path}" checked>
            <div>
              <div style="font-weight: 600; color: var(--text-main);">${i.name}</div>
              <div style="font-size: 11px; color: var(--text-dim);">${i.server_count} servers (${i.servers.join(", ")})</div>
            </div>
          </label>
        `).join("");else t.innerHTML='<div style="color: var(--text-dim); padding: 12px; text-align: center;">No external MCP configuration files found on this system.</div>'}catch{t.innerHTML='<div style="color: var(--red-400); padding: 12px; text-align: center;">Failed to scan ecosystem sources.</div>'}}async submitImport(){let e=document.querySelectorAll(".eco-checkbox:checked");if(e.length===0){alert("No sources selected");return}for(let t of Array.from(e))await c.importConfig(t.value,!1);this.closeModals(),await this.refreshData()}handleAliasTargetInput(e){let t=document.getElementById("alias-suggestions-dropdown");if(!t)return;let r=(e||"").trim().toLowerCase();if(r.length<2){t.style.display="none";return}let a=l.getState().capabilities.filter((o)=>o.id.toLowerCase().includes(r)||o.summary&&o.summary.toLowerCase().includes(r)||o.description&&o.description.toLowerCase().includes(r)||o.server&&o.server.toLowerCase().includes(r)).slice(0,8);if(a.length===0){t.style.display="none";return}t.innerHTML=a.map((o)=>`
      <div style="padding: 8px 12px; cursor: pointer; border-bottom: 1px solid var(--border-subtle); display: flex; justify-content: space-between; align-items: center; transition: background 0.1s;"
           onmouseover="this.style.background='var(--surface-hover)'"
           onmouseout="this.style.background='transparent'"
           onmousedown="window.app.selectAliasSuggestion('${d(o.id)}')">
        <div>
          <div style="font-weight: 700; color: var(--text-main);">${d(o.id)}</div>
          <div style="font-size: 10.5px; color: var(--text-dim); margin-top: 2px;">${d(o.summary||o.description||"")}</div>
        </div>
        <span style="font-size: 10px; color: var(--cyan-400);">${d(o.server||"local")}</span>
      </div>
    `).join(""),t.style.display="block"}selectAliasSuggestion(e){let t=document.getElementById("alias-target");if(t)t.value=e;this.hideAliasDropdown()}hideAliasDropdown(){let e=document.getElementById("alias-suggestions-dropdown");if(e)e.style.display="none"}async createAlias(){let e=document.getElementById("alias-kind")?.value,t=document.getElementById("alias-name")?.value.trim(),r=document.getElementById("alias-target")?.value.trim();if(!t||!r){alert("Please provide both alias name and canonical target");return}await c.updateAlias(e,t,r),await this.refreshData()}async deleteAlias(e,t){await c.updateAlias(e,t,void 0),await this.refreshData()}async reloadFromDisk(){try{let e=await c.reloadConfig();if(e.ok){let t="Hot-reload completed successfully!";if(e.mounted&&e.mounted.length>0)t+=`
Mounted: ${e.mounted.join(", ")}`;if(e.unmounted&&e.unmounted.length>0)t+=`
Unmounted: ${e.unmounted.join(", ")}`;if(e.warnings&&e.warnings.length>0)t+=`
Warnings:
${e.warnings.join(`
`)}`;alert(t)}else alert(`Hot-reload failed: ${e.error||"Unknown error"}`)}catch(e){alert(`Error reaching daemon: ${e.message}`)}await this.refreshData()}closeModals(){document.querySelectorAll(".modal-backdrop").forEach((e)=>e.classList.remove("active"))}}function d(e){return String(e||"").replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;")}var q=new N;window.app=q;window.addEventListener("DOMContentLoaded",()=>q.init());
