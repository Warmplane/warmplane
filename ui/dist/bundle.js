class M{state={configPath:"mcp_servers.json",config:{mcpServers:{}},serverStatuses:{},circuitBreakers:[],capabilities:[],resources:[],prompts:[],catalogEvents:[],approvals:[],auditEvents:[],auditTotal:0,auditFilters:{search:"",status:"all",eventType:"all",serverId:"all",limit:25,offset:0},auditSelectedEvent:null,auditStats:null,auditVerification:null,selectedCapabilityId:null,selectedResourceId:null,selectedPromptId:null,playgroundMode:"tools",activeTab:"overview",eventLogs:[],executionResult:null,resourceReadResult:null,promptGetResult:null,metrics:{totalCatalogRequests:0,totalEtagHits:0,totalToolCalls:0,totalToolDurationUs:0}};listeners=[];getState(){return this.state}setState(e){this.state={...this.state,...e},this.listeners.forEach((t)=>t(this.state))}subscribe(e){return this.listeners.push(e),()=>{this.listeners=this.listeners.filter((t)=>t!==e)}}addEventLog(e,t,i,a){let o=[{time:new Date().toLocaleTimeString(),method:e,target:t,status:i,latency:a},...this.state.eventLogs].slice(0,50);this.setState({eventLogs:o})}}var s=new M;class D{baseUrl;constructor(e=""){this.baseUrl=e}async getConfig(){return(await fetch(`${this.baseUrl}/v1/config`)).json()}async listCapabilities(){return(await fetch(`${this.baseUrl}/v1/capabilities`)).json()}async listResources(){return(await fetch(`${this.baseUrl}/v1/resources`)).json()}async readResource(e){let t=performance.now(),i=await fetch(`${this.baseUrl}/v1/resources/read`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify(e)}),a=performance.now()-t,r=await i.json();return{status:i.status,durationMs:a,data:r}}async listPrompts(){return(await fetch(`${this.baseUrl}/v1/prompts`)).json()}async getPrompt(e){let t=performance.now(),i=await fetch(`${this.baseUrl}/v1/prompts/get`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify(e)}),a=performance.now()-t,r=await i.json();return{status:i.status,durationMs:a,data:r}}async getCatalogEvents(e){let t=e?`?after=${encodeURIComponent(e)}`:"";return(await fetch(`${this.baseUrl}/v1/catalog/events${t}`)).json()}async callCapability(e){let t=performance.now(),i=await fetch(`${this.baseUrl}/v1/tools/call`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify(e)}),a=performance.now()-t,r=await i.json();return{status:i.status,durationMs:a,data:r}}async batchCallCapabilities(e){let t=performance.now(),i=await fetch(`${this.baseUrl}/v1/tools/batch_call`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({steps:e})}),a=performance.now()-t,r=await i.json();return{status:i.status,durationMs:a,data:r}}async upsertServer(e,t){return(await fetch(`${this.baseUrl}/v1/config/servers`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({name:e,server:t})})).json()}async deleteServer(e){return(await fetch(`${this.baseUrl}/v1/config/servers/${encodeURIComponent(e)}`,{method:"DELETE"})).json()}async getEcosystemSources(){return(await fetch(`${this.baseUrl}/v1/config/ecosystem`)).json()}async importConfig(e,t=!1){return(await fetch(`${this.baseUrl}/v1/config/import`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({source_path:e,overwrite:t})})).json()}async savePolicy(e){let t={allow:e.allow||[],deny:e.deny||[],redactKeys:e.redact_keys||e.redactKeys||[],requireApproval:e.require_approval||e.requireApproval||[],approvalTimeoutSecs:e.approvalTimeoutSecs||e.approval_timeout_secs||300,webhook:e.webhook};return(await fetch(`${this.baseUrl}/v1/config/policy`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify(t)})).json()}async listApprovals(){return(await fetch(`${this.baseUrl}/v1/approvals`)).json()}async approveTicket(e,t,i){return(await fetch(`${this.baseUrl}/v1/approvals/${encodeURIComponent(e)}/approve`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({operator:t,modified_args:i})})).json()}async rejectTicket(e,t,i){return(await fetch(`${this.baseUrl}/v1/approvals/${encodeURIComponent(e)}/reject`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({operator:t,reason:i})})).json()}async updateAlias(e,t,i){return(await fetch(`${this.baseUrl}/v1/config/alias`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({kind:e,alias:t,target:i})})).json()}async reloadConfig(){return(await fetch(`${this.baseUrl}/v1/config/reload`,{method:"POST"})).json()}async listAuditEvents(e){let t=new URLSearchParams;if(e?.actor_id)t.set("actor_id",e.actor_id);if(e?.server_id&&e.server_id!=="all")t.set("server_id",e.server_id);if(e?.capability_id)t.set("capability_id",e.capability_id);if(e?.event_type&&e.event_type!=="all")t.set("event_type",e.event_type);if(e?.status&&e.status!=="all")t.set("status",e.status);if(e?.trace_id)t.set("trace_id",e.trace_id);if(e?.request_id)t.set("request_id",e.request_id);if(e?.search)t.set("search",e.search);if(e?.limit)t.set("limit",String(e.limit));if(e?.offset!==void 0)t.set("offset",String(e.offset));let i=t.toString();return(await fetch(`${this.baseUrl}/v1/audit/events${i?`?${i}`:""}`)).json()}getAuditExportUrl(e,t="csv"){let i=new URLSearchParams;if(i.set("format",t),e?.actor_id)i.set("actor_id",e.actor_id);if(e?.server_id&&e.server_id!=="all")i.set("server_id",e.server_id);if(e?.capability_id)i.set("capability_id",e.capability_id);if(e?.event_type&&e.event_type!=="all")i.set("event_type",e.event_type);if(e?.status&&e.status!=="all")i.set("status",e.status);if(e?.trace_id)i.set("trace_id",e.trace_id);if(e?.request_id)i.set("request_id",e.request_id);if(e?.search)i.set("search",e.search);return`${this.baseUrl}/v1/audit/export?${i.toString()}`}async verifyAuditChain(){return(await fetch(`${this.baseUrl}/v1/audit/verify`)).json()}async getAuditStats(){return(await fetch(`${this.baseUrl}/v1/audit/stats`)).json()}}var u=new D;function q(){let e=s.getState(),t=e.config.mcpServers||{},i=Object.keys(t),a=i.length,r="";if(i.length===0)r=`
      <div style="grid-column: 1 / -1; padding: 32px; text-align: center; color: var(--text-dim); background: var(--surface-card); border-radius: var(--radius-md); border: 1px dashed var(--border);">
        <div style="font-size: 14px; color: var(--text-main); font-weight: 600; margin-bottom: 6px;">No Upstream MCP Servers Connected</div>
        <div style="font-size: 12px; margin-bottom: 16px;">Initialize connections by adding a server or syncing existing IDE configurations.</div>
        <div style="display: flex; gap: 8px; justify-content: center;">
          <button class="btn btn-primary" onclick="window.app.openTemplateCatalog()">✨ Browse Templates</button>
          <button class="btn btn-ghost" onclick="window.app.openAddServerModal()">+ Add Custom</button>
          <button class="btn btn-ghost" onclick="window.app.openImportModal()">Sync from IDEs</button>
        </div>
      </div>
    `;else r=i.map((f)=>{let z=t[f],j=z.command?"stdio":"http / sse",R=z.command?`${z.command} ${(z.args||[]).join(" ")}`:z.url,P=e.serverStatuses[f]||{status:"connected",protocol_version:"2026-07-28"},y=P.status==="degraded",L=P.status==="error"||P.status==="disconnected",T=y?"var(--amber-400)":L?"var(--red-400)":"var(--green-400)";return`
        <div class="bento-card col-4" style="background: var(--surface); border: 1px solid var(--border);">
          <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
            <span style="font-weight: 700; color: var(--text-main); display: flex; align-items: center; gap: 6px;">
              <span style="width: 8px; height: 8px; border-radius: 50%; background: ${T}; display: inline-block;"></span>
              ${A(f)}
            </span>
            <span class="brand-badge">${j}</span>
          </div>
          <div style="font-family: var(--ff-mono); font-size: 11.5px; color: var(--text-dim); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; margin-bottom: 12px;" title="${A(R||"")}">
            ${A(R||"")}
          </div>
          <div style="display: flex; justify-content: space-between; font-size: 11px; color: var(--text-muted); border-top: 1px solid var(--border); padding-top: 8px;">
            <span>Status: <strong style="color: ${T};">${A(P.status)}</strong></span>
            <span>Protocol: ${P.protocol_version}</span>
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
  `:e.eventLogs.map((f)=>`
    <div class="feed-row" style="grid-template-columns: 80px 100px 1fr 100px 80px;">
      <span style="color: var(--text-dim);">${A(f.time)}</span>
      <span style="color: var(--cyan-400); font-weight: 600;">${A(f.method)}</span>
      <span style="color: var(--text-main); font-family: var(--ff-mono);">${A(f.target)}</span>
      <span style="color: var(--green-400);">${A(f.status)}</span>
      <span style="color: var(--amber-300); text-align: right;">${A(f.latency)}</span>
    </div>
  `).join(""),n=e.metrics,d=n.totalCatalogRequests,l=n.totalEtagHits,g=d>0?`${(l/d*100).toFixed(1)}%`:"0.0%",p=d>0?`${l} of ${d} requests served via HTTP 304`:"Waiting for client requests",v=n.totalToolCalls,x=v>0?`${(n.totalToolDurationUs/v/1000).toFixed(1)}ms`:"0.0ms",w=v>0?`${v} tool executions processed`:"Local worker task queues warm",h=Object.keys(e.config.capabilityAliases||{}).length+Object.keys(e.config.resourceAliases||{}).length+Object.keys(e.config.promptAliases||{}).length,k=h>0?`${h*18}B / call`:"0B",$=h>0?`${h} active facade aliases pruning prompt size`:"Configure aliases in Studio to reduce prompt size";return`
    <div class="bento-grid">
      <div class="bento-card col-3">
        <div class="stat-label">Token Savings Rate</div>
        <div class="stat-value" style="color: var(--amber-300);">${k}</div>
        <div class="stat-sub">${$}</div>
      </div>
      <div class="bento-card col-3">
        <div class="stat-label">ETag Cache Hit Rate</div>
        <div class="stat-value" style="color: var(--cyan-400);">${g}</div>
        <div class="stat-sub">${p}</div>
      </div>
      <div class="bento-card col-3">
        <div class="stat-label">Connected Upstreams</div>
        <div class="stat-value" style="color: var(--green-400);">${a} Active</div>
        <div class="stat-sub">${a>0?"Persistent worker task channels":"No active upstream servers"}</div>
      </div>
      <div class="bento-card col-3">
        <div class="stat-label">Avg Execution Latency</div>
        <div class="stat-value">${x}</div>
        <div class="stat-sub">${w}</div>
      </div>
    </div>

    <div style="display: flex; justify-content: space-between; align-items: center; margin: 24px 0 12px;">
      <div style="font-size: 15px; font-weight: 700; color: var(--text-main);">Connected Upstream Servers</div>
      <button class="btn btn-ghost" onclick="window.app.switchTab('servers')">Manage All (${a}) →</button>
    </div>

    <div class="bento-grid" style="margin-bottom: 24px;">
      ${r}
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
  `}function A(e){return String(e||"").replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;")}function I(){let e=s.getState(),t=e.config.mcpServers||{},i=Object.keys(t),a="";if(i.length===0)a=`
      <div style="padding: 40px; text-align: center; color: var(--text-dim); background: var(--surface-card); border-radius: var(--radius-md); border: 1px dashed var(--border);">
        <div style="font-size: 15px; color: var(--text-main); font-weight: 600; margin-bottom: 8px;">No Servers Configured in ${C(e.configPath)}</div>
        <p style="font-size: 12px; margin-bottom: 20px; max-width: 480px; margin-left: auto; margin-right: auto;">
          Warmplane bridges local tools and remote MCP servers into one unified facade. Add your first server or import existing configs from Claude Desktop or Cursor.
        </p>
        <div style="display: flex; gap: 8px; justify-content: center;">
          <button class="btn btn-primary" onclick="window.app.openTemplateCatalog()">✨ Browse Templates</button>
          <button class="btn btn-ghost" onclick="window.app.openAddServerModal()">+ Add Custom</button>
          <button class="btn btn-ghost" onclick="window.app.openImportModal()">Sync from IDEs</button>
        </div>
      </div>
    `;else a=i.map((r)=>{let o=t[r],n=o.command?"stdio":"http / sse",d=o.command?`${o.command} ${(o.args||[]).join(" ")}`:o.url,l=e.serverStatuses[r]||{status:"connected",protocol_version:"2026-07-28"},g=o.env?Object.keys(o.env).map((f)=>`${f}=***`).join(", "):"None",p=(e.circuitBreakers||[]).find((f)=>f.server_id===r),v='<span class="brand-badge" style="color: var(--green-400); border-color: rgba(52, 211, 153, 0.25);">Circuit: CLOSED</span>';if(p){if(p.state==="open")v=`<span class="brand-badge" style="color: var(--red-400); border-color: rgba(248, 113, 113, 0.4); background: rgba(248, 113, 113, 0.1);">Circuit: OPEN (${p.consecutive_failures} failures)</span>`;else if(p.state==="half_open")v=`<span class="brand-badge" style="color: var(--amber-300); border-color: rgba(251, 191, 36, 0.4); background: rgba(251, 191, 36, 0.1);">Circuit: HALF-OPEN (${p.consecutive_successes} probe)</span>`}let x=o.resilience||e.config.resilience,w=x?`FT: ${x.failureThreshold||3} · Cooldown: ${(x.cooldownMs||30000)/1000}s · AutoRestart: ${x.autoRestart!==!1?"ON":"OFF"}`:"Default Resilience",h=l.status==="degraded",k=l.status==="error"||l.status==="disconnected",$=h?"var(--amber-400)":k?"var(--red-400)":"var(--green-400)";return`
        <div class="bento-card" style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px;">
          <div>
            <div style="display: flex; align-items: center; gap: 8px; margin-bottom: 4px;">
              <span style="width: 8px; height: 8px; border-radius: 50%; background: ${$}; display: inline-block;"></span>
              <span style="font-size: 15px; font-weight: 700; color: var(--text-main);">${C(r)}</span>
              <span class="brand-badge">${n}</span>
              <span class="brand-badge" style="color: ${$}; border-color: rgba(245, 158, 11, 0.3);">Status: ${C(l.status)}</span>
              <span class="brand-badge" style="color: var(--cyan-400); border-color: rgba(34, 211, 238, 0.25);">Protocol: ${l.protocol_version}</span>
              ${v}
            </div>
            <div style="font-family: var(--ff-mono); font-size: 12px; color: var(--text-muted); margin-top: 4px;">
              ${o.command?"Command: ":"URL: "}<code>${C(d||"")}</code>
            </div>
            <div style="display: flex; gap: 14px; font-family: var(--ff-mono); font-size: 11px; color: var(--text-dim); margin-top: 4px;">
              <span>\uD83D\uDEE1️ ${C(w)}</span>
              ${o.env&&Object.keys(o.env).length>0?`<span>Env: ${C(g)}</span>`:""}
            </div>
          </div>
          <div style="display: flex; gap: 8px;">
            <button class="btn btn-ghost" style="padding: 4px 10px; font-size: 11.5px;" onclick="window.app.openEditServerModal('${C(r)}')">✏️ Edit</button>
            <button class="btn btn-danger" style="padding: 4px 10px; font-size: 11.5px;" onclick="window.app.deleteServer('${C(r)}')">Remove</button>
          </div>
        </div>
      `}).join("");return`
    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 18px;">
      <div>
        <div style="font-size: 16px; font-weight: 700; color: var(--text-main);">Configured MCP Upstream Servers</div>
        <div style="font-size: 11px; color: var(--text-dim);">Active configuration file: <code>${C(e.configPath)}</code></div>
      </div>
      <div style="display: flex; gap: 8px;">
        <button class="btn btn-ghost" onclick="window.app.reloadFromDisk()">⟳ Reload Config</button>
      </div>
    </div>

    ${a}
  `}function C(e){return String(e||"").replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;")}function U(){let e=s.getState(),t=e.playgroundMode||"tools",i=e.capabilities||[],a=e.resources||[],r=e.prompts||[],o=`
    <div style="display: flex; gap: 8px; margin-bottom: 12px; align-items: center; justify-content: space-between;">
      <div style="display: inline-flex; padding: 3px; background: rgba(0,0,0,0.3); border: 1px solid var(--border); border-radius: var(--radius-sm);">
        <button 
          class="btn ${t==="tools"?"btn-primary":"btn-ghost"}" 
          style="padding: 4px 12px; font-size: 11.5px; height: 28px;"
          onclick="window.app.setPlaygroundMode('tools')"
        >
          \uD83D\uDEE0️ Tools (${i.length})
        </button>
        <button 
          class="btn ${t==="resources"?"btn-primary":"btn-ghost"}" 
          style="padding: 4px 12px; font-size: 11.5px; height: 28px;"
          onclick="window.app.setPlaygroundMode('resources')"
        >
          \uD83D\uDCC4 Resources (${a.length})
        </button>
        <button 
          class="btn ${t==="prompts"?"btn-primary":"btn-ghost"}" 
          style="padding: 4px 12px; font-size: 11.5px; height: 28px;"
          onclick="window.app.setPlaygroundMode('prompts')"
        >
          \uD83D\uDCAC Prompts (${r.length})
        </button>
      </div>

      <div style="font-size: 11.5px; color: var(--text-dim);">
        ${t==="tools"?"Interactive Tool Caller & Context Distillation":t==="resources"?"Live MCP Resource Inspector & Reader":"Prompt Template Studio & Variable Binder"}
      </div>
    </div>
  `;if(t==="resources")return`
      ${o}
      ${H(e)}
    `;if(t==="prompts")return`
      ${o}
      ${ee(e)}
    `;return`
    ${o}
    ${V(e)}
  `}function V(e){let t=e.capabilities||[],i=e.selectedCapabilityId||(t.length>0?t[0].id:null),a=t.find((n)=>n.id===i),r="";if(t.length===0)r=`
      <div style="padding: 24px 16px; text-align: center; color: var(--text-dim); font-size: 11.5px;">
        No tools or capabilities discovered from connected servers.
      </div>
    `;else r=t.map((n)=>{return`
        <div class="cap-item ${n.id===i?"active":""}" onclick="window.app.selectCapability('${m(n.id)}')">
          <div style="display: flex; justify-content: space-between; align-items: center;">
            <span style="font-weight: 600; color: var(--text-main); font-family: var(--ff-mono); font-size: 12px;">${m(n.id)}</span>
            <span style="font-size: 10px; color: var(--green-400);">${m(n.mode||"read")}</span>
          </div>
          <div style="font-size: 11px; color: var(--text-dim); margin-top: 2px;">server: ${m(n.server||"local")}</div>
        </div>
      `}).join("");let o=a&&a.input_schema?JSON.stringify(a.input_schema.properties||{},null,2):"{}";return`
    <div style="display: grid; grid-template-columns: 320px 1fr; gap: 16px; height: calc(100vh - 165px);">
      <!-- Left Sidebar: Capabilities Catalog -->
      <div style="background: var(--surface-card); border: 1px solid var(--border); border-radius: var(--radius-md); display: flex; flex-direction: column; overflow: hidden;">
        <div style="padding: 12px; border-bottom: 1px solid var(--border);">
          <input type="text" class="form-input" placeholder="Search ${t.length} capabilities..." oninput="window.app.filterCapabilities(this.value)">
        </div>
        <div style="flex: 1; overflow-y: auto; padding: 8px;" id="pg-cap-list">
          ${r}
        </div>
      </div>

      <!-- Right Panel: Capability Execution & Envelope Visualizer -->
      <div style="background: var(--surface-card); border: 1px solid var(--border); border-radius: var(--radius-md); display: flex; flex-direction: column; overflow: hidden;">
        <div style="padding: 14px 18px; border-bottom: 1px solid var(--border); display: flex; justify-content: space-between; align-items: center;">
          <div>
            <div style="font-size: 15px; font-weight: 700; color: var(--text-main); font-family: var(--ff-mono);" id="pg-selected-title">
              ${m(a?a.id:"No Capability Selected")}
            </div>
            <div style="font-size: 11.5px; color: var(--text-dim);" id="pg-selected-desc">
              ${m(a?a.summary||a.description:"Connect servers to inspect and execute tools")}
            </div>
          </div>
          <button class="btn btn-primary" onclick="window.app.executePlaygroundTool()" ${a?"":"disabled"}>
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polygon points="5 3 19 12 5 21 5 3"></polygon></svg>
            Execute Capability
          </button>
        </div>

        <div style="flex: 1; display: grid; grid-template-columns: 1fr 1fr; overflow: hidden;">
          <!-- Request Builder -->
          <div style="padding: 16px; border-right: 1px solid var(--border); overflow-y: auto;">
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 10px;">
              <label class="form-label" style="margin: 0;">Arguments JSON (Object)</label>
              <div style="display: flex; gap: 8px;">
                <button class="btn btn-ghost" style="padding: 2px 8px; font-size: 11px;" onclick="window.app.toggleBatchPlayground()">\uD83D\uDD04 Batch Mode</button>
              </div>
            </div>
            <textarea class="form-textarea" rows="7" id="pg-args-input">${m(o)}</textarea>

            <div style="margin-top: 12px; padding: 10px; background: rgba(0,0,0,0.2); border-radius: var(--radius-sm); border: 1px solid var(--border);">
              <div style="font-size: 11px; font-weight: 700; color: var(--cyan-400); margin-bottom: 6px; display: flex; align-items: center; gap: 6px;">
                <span>⚡ Context Distillation Filters</span>
              </div>
              <div class="form-group" style="margin-bottom: 6px;">
                <label class="form-label" style="font-size: 10.5px;">JSONPath Filter (e.g. $.items[*].name)</label>
                <input type="text" class="form-input" id="pg-jsonpath-input" placeholder="$.result">
              </div>
              <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 8px;">
                <div>
                  <label class="form-label" style="font-size: 10.5px;">Max Lines</label>
                  <input type="number" class="form-input" id="pg-limit-lines-input" placeholder="e.g. 50">
                </div>
                <div>
                  <label class="form-label" style="font-size: 10.5px;">Max Bytes</label>
                  <input type="number" class="form-input" id="pg-truncate-bytes-input" placeholder="e.g. 20480">
                </div>
              </div>
            </div>

            <div class="form-group" style="margin-top: 10px;">
              <label class="form-label">Request Context / Operation ID (Optional)</label>
              <input type="text" class="form-input" id="pg-context-input" placeholder="e.g. op-dev-test-1">
            </div>
            ${a&&a.input_schema?`
              <div style="margin-top: 14px;">
                <label class="form-label">Input JSON Schema</label>
                <pre style="background: var(--surface); padding: 10px; border-radius: var(--radius-sm); border: 1px solid var(--border); font-size: 11px; color: var(--text-muted); max-height: 140px; overflow-y: auto;">${m(JSON.stringify(a.input_schema,null,2))}</pre>
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
            <pre id="pg-response-json" style="flex: 1; background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 14px; color: var(--amber-300); font-size: 11.5px; overflow-y: auto; margin: 0; white-space: pre-wrap; word-break: break-word;">${e.executionResult?m(JSON.stringify(e.executionResult.data,null,2)):"// Response envelope output will be formatted here"}</pre>
          </div>
        </div>
      </div>
    </div>
  `}function H(e){let t=e.resources||[],i=e.selectedResourceId||(t.length>0?t[0].id:null),a=t.find((n)=>n.id===i),r=e.resourceReadResult,o="";if(t.length===0)o=`
      <div style="padding: 24px 16px; text-align: center; color: var(--text-dim); font-size: 11.5px;">
        No resources exposed by connected MCP servers.
      </div>
    `;else o=t.map((n)=>{let d=n.id===i?"active":"",l=n.uri?n.uri.split(":")[0]:"res";return`
        <div class="cap-item ${d}" onclick="window.app.selectResource('${m(n.id)}')">
          <div style="display: flex; justify-content: space-between; align-items: center;">
            <span style="font-weight: 600; color: var(--text-main); font-family: var(--ff-mono); font-size: 12px;">${m(n.name||n.id)}</span>
            <span class="badge" style="font-size: 9.5px; background: rgba(56, 189, 248, 0.15); color: var(--cyan-400);">${m(l)}</span>
          </div>
          <div style="font-size: 11px; color: var(--text-dim); margin-top: 2px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">${m(n.uri)}</div>
          <div style="display: flex; justify-content: space-between; font-size: 10px; color: var(--text-muted); margin-top: 4px;">
            <span>server: ${m(n.server||"local")}</span>
            <span>${m(n.mime_type||"text/plain")}</span>
          </div>
        </div>
      `}).join("");return`
    <div style="display: grid; grid-template-columns: 340px 1fr; gap: 16px; height: calc(100vh - 165px);">
      <!-- Left Sidebar: Resources Catalog -->
      <div style="background: var(--surface-card); border: 1px solid var(--border); border-radius: var(--radius-md); display: flex; flex-direction: column; overflow: hidden;">
        <div style="padding: 12px; border-bottom: 1px solid var(--border);">
          <input type="text" class="form-input" placeholder="Search ${t.length} resources..." oninput="window.app.filterResources(this.value)">
        </div>
        <div style="flex: 1; overflow-y: auto; padding: 8px;" id="pg-res-list">
          ${o}
        </div>
      </div>

      <!-- Right Panel: Resource Content Reader & Metadata Inspector -->
      <div style="background: var(--surface-card); border: 1px solid var(--border); border-radius: var(--radius-md); display: flex; flex-direction: column; overflow: hidden;">
        <div style="padding: 14px 18px; border-bottom: 1px solid var(--border); display: flex; justify-content: space-between; align-items: center;">
          <div>
            <div style="font-size: 15px; font-weight: 700; color: var(--text-main); font-family: var(--ff-mono);">
              ${m(a?a.name||a.id:"No Resource Selected")}
            </div>
            <div style="font-size: 11.5px; color: var(--cyan-400); font-family: var(--ff-mono);">
              ${m(a?a.uri:"Select a resource from the list to read live content")}
            </div>
          </div>
          <button class="btn btn-primary" onclick="window.app.executeReadResource()" ${a?"":"disabled"}>
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M2 12h20M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"></path></svg>
            Read Resource Content
          </button>
        </div>

        <div style="flex: 1; display: grid; grid-template-columns: 1fr 1fr; overflow: hidden;">
          <!-- Request / Distillation Parameters -->
          <div style="padding: 16px; border-right: 1px solid var(--border); overflow-y: auto;">
            ${a?`
              <div style="background: rgba(0,0,0,0.25); padding: 12px; border-radius: var(--radius-sm); border: 1px solid var(--border); margin-bottom: 14px;">
                <div style="font-size: 11px; font-weight: 600; color: var(--text-muted); text-transform: uppercase; margin-bottom: 6px;">Resource Metadata</div>
                <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 8px; font-size: 11.5px;">
                  <div><span style="color: var(--text-muted);">Server:</span> <strong style="color: var(--text-main);">${m(a.server)}</strong></div>
                  <div><span style="color: var(--text-muted);">MIME Type:</span> <strong style="color: var(--text-main);">${m(a.mime_type||"text/plain")}</strong></div>
                </div>
                ${a.description?`
                  <div style="margin-top: 8px; font-size: 11.5px; color: var(--text-dim); border-top: 1px solid rgba(255,255,255,0.05); padding-top: 6px;">
                    ${m(a.description)}
                  </div>
                `:""}
              </div>
            `:""}

            <div style="padding: 12px; background: rgba(0,0,0,0.2); border-radius: var(--radius-sm); border: 1px solid var(--border);">
              <div style="font-size: 11px; font-weight: 700; color: var(--cyan-400); margin-bottom: 8px; display: flex; align-items: center; gap: 6px;">
                <span>⚡ Context Distillation Options</span>
              </div>
              <div class="form-group" style="margin-bottom: 8px;">
                <label class="form-label" style="font-size: 10.5px;">JSONPath Expression</label>
                <input type="text" class="form-input" id="pg-res-jsonpath-input" placeholder="$.items[*].data">
              </div>
              <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 8px;">
                <div>
                  <label class="form-label" style="font-size: 10.5px;">Max Lines</label>
                  <input type="number" class="form-input" id="pg-res-lines-input" placeholder="e.g. 100">
                </div>
                <div>
                  <label class="form-label" style="font-size: 10.5px;">Max Bytes</label>
                  <input type="number" class="form-input" id="pg-res-bytes-input" placeholder="e.g. 32768">
                </div>
              </div>
            </div>
          </div>

          <!-- Content Output Preview -->
          <div style="padding: 16px; background: var(--bg-app); display: flex; flex-direction: column; overflow: hidden;">
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
              <span style="font-size: 11px; font-weight: 600; color: var(--text-dim);">RESOURCE CONTENT ENVELOPE</span>
              <span style="font-size: 11px; font-weight: 600; color: ${r?r.status===200?"var(--green-400)":"var(--red-400)":"var(--text-dim)"}; font-family: var(--ff-mono);">
                ${r?`HTTP ${r.status} · ${r.durationMs.toFixed(1)}ms`:"READY"}
              </span>
            </div>
            <pre style="flex: 1; background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 14px; color: var(--cyan-400); font-size: 11.5px; overflow-y: auto; margin: 0; white-space: pre-wrap; word-break: break-word;">${r?m(JSON.stringify(r.data,null,2)):'// Click "Read Resource Content" to inspect live payload'}</pre>
          </div>
        </div>
      </div>
    </div>
  `}function ee(e){let t=e.prompts||[],i=e.selectedPromptId||(t.length>0?t[0].id:null),a=t.find((d)=>d.id===i),r=e.promptGetResult,o="";if(t.length===0)o=`
      <div style="padding: 24px 16px; text-align: center; color: var(--text-dim); font-size: 11.5px;">
        No prompt templates registered by connected MCP servers.
      </div>
    `;else o=t.map((d)=>{let l=d.id===i?"active":"",g=d.arguments?d.arguments.length:0;return`
        <div class="cap-item ${l}" onclick="window.app.selectPrompt('${m(d.id)}')">
          <div style="display: flex; justify-content: space-between; align-items: center;">
            <span style="font-weight: 600; color: var(--text-main); font-family: var(--ff-mono); font-size: 12px;">${m(d.name||d.id)}</span>
            <span class="badge" style="font-size: 9.5px; background: rgba(168, 85, 247, 0.15); color: var(--purple-400);">${g} args</span>
          </div>
          <div style="font-size: 11px; color: var(--text-dim); margin-top: 2px;">${m(d.description||d.title||"Prompt template")}</div>
          <div style="font-size: 10px; color: var(--text-muted); margin-top: 4px;">server: ${m(d.server||"local")}</div>
        </div>
      `}).join("");let n="";if(a&&a.arguments&&a.arguments.length>0)n=a.arguments.map((d)=>`
      <div class="form-group" style="margin-bottom: 12px;">
        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 4px;">
          <label class="form-label" style="margin: 0; font-family: var(--ff-mono);">${m(d.name)}</label>
          ${d.required?'<span class="badge" style="background: rgba(239, 68, 68, 0.15); color: var(--red-400); font-size: 9px;">REQUIRED</span>':'<span style="font-size: 10px; color: var(--text-dim);">optional</span>'}
        </div>
        ${d.description?`<div style="font-size: 11px; color: var(--text-dim); margin-bottom: 4px;">${m(d.description)}</div>`:""}
        <input type="text" class="form-input prompt-arg-input" data-arg-name="${m(d.name)}" placeholder="Enter ${m(d.name)}..." />
      </div>
    `).join("");else if(a)n=`
      <div style="padding: 12px; background: rgba(0,0,0,0.2); border-radius: var(--radius-sm); border: 1px solid var(--border); font-size: 11.5px; color: var(--text-dim);">
        This prompt template does not require any input arguments.
      </div>
    `;return`
    <div style="display: grid; grid-template-columns: 340px 1fr; gap: 16px; height: calc(100vh - 165px);">
      <!-- Left Sidebar: Prompts Catalog -->
      <div style="background: var(--surface-card); border: 1px solid var(--border); border-radius: var(--radius-md); display: flex; flex-direction: column; overflow: hidden;">
        <div style="padding: 12px; border-bottom: 1px solid var(--border);">
          <input type="text" class="form-input" placeholder="Search ${t.length} prompts..." oninput="window.app.filterPrompts(this.value)">
        </div>
        <div style="flex: 1; overflow-y: auto; padding: 8px;" id="pg-prompt-list">
          ${o}
        </div>
      </div>

      <!-- Right Panel: Prompt Parameter Binder & Message Envelope Preview -->
      <div style="background: var(--surface-card); border: 1px solid var(--border); border-radius: var(--radius-md); display: flex; flex-direction: column; overflow: hidden;">
        <div style="padding: 14px 18px; border-bottom: 1px solid var(--border); display: flex; justify-content: space-between; align-items: center;">
          <div>
            <div style="font-size: 15px; font-weight: 700; color: var(--text-main); font-family: var(--ff-mono);">
              ${m(a?a.name||a.id:"No Prompt Selected")}
            </div>
            <div style="font-size: 11.5px; color: var(--text-dim);">
              ${m(a?a.description||a.title||"Bind variables and render messages":"Select a prompt from the list to test")}
            </div>
          </div>
          <button class="btn btn-primary" onclick="window.app.executeGetPrompt()" ${a?"":"disabled"}>
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polygon points="5 3 19 12 5 21 5 3"></polygon></svg>
            Render Prompt Messages
          </button>
        </div>

        <div style="flex: 1; display: grid; grid-template-columns: 1fr 1fr; overflow: hidden;">
          <!-- Arguments Form Builder -->
          <div style="padding: 16px; border-right: 1px solid var(--border); overflow-y: auto;">
            <div style="font-size: 12px; font-weight: 700; color: var(--text-main); margin-bottom: 12px; text-transform: uppercase;">
              Template Arguments
            </div>
            ${n}
          </div>

          <!-- Rendered Messages Output -->
          <div style="padding: 16px; background: var(--bg-app); display: flex; flex-direction: column; overflow: hidden;">
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
              <span style="font-size: 11px; font-weight: 600; color: var(--text-dim);">RENDERED PROMPT MESSAGES</span>
              <span style="font-size: 11px; font-weight: 600; color: ${r?r.status===200?"var(--green-400)":"var(--red-400)":"var(--text-dim)"}; font-family: var(--ff-mono);">
                ${r?`HTTP ${r.status} · ${r.durationMs.toFixed(1)}ms`:"READY"}
              </span>
            </div>
            <pre style="flex: 1; background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 14px; color: #c084fc; font-size: 11.5px; overflow-y: auto; margin: 0; white-space: pre-wrap; word-break: break-word;">${r?m(JSON.stringify(r.data,null,2)):'// Click "Render Prompt Messages" to view resolved system/user messages'}</pre>
          </div>
        </div>
      </div>
    </div>
  `}function m(e){return String(e||"").replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;")}function F(e){let t=e.approvals.filter((o)=>o.status==="pending"),i=e.approvals.filter((o)=>o.status!=="pending"),a=t.length===0?`
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
            <span style="font-family: var(--ff-mono); font-size: 12px; color: var(--text-muted);">${_(o.id)}</span>
          </div>
          <div style="margin-top: 6px; display: flex; align-items: center; gap: 8px;">
            <span style="font-size: 14.5px; font-weight: 700; color: var(--text-main); font-family: var(--ff-mono);">${_(o.capability_id)}</span>
            <span style="font-size: 11px; color: var(--text-dim);">via <span style="color: var(--cyan-400); font-family: var(--ff-mono);">${_(o.server_id)}</span></span>
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
          ${o.request_id?`<div><span style="color: var(--text-dim);">Request:</span> <span style="color: var(--text-main);">${_(o.request_id)}</span></div>`:""}
          ${o.context?.actor_id?`<div><span style="color: var(--text-dim);">Actor:</span> <span style="color: var(--cyan-400);">${_(o.context.actor_id)}</span></div>`:""}
          ${o.context?.operation_id?`<div><span style="color: var(--text-dim);">Operation:</span> <span style="color: var(--text-main);">${_(o.context.operation_id)}</span></div>`:""}
          ${o.context?.work_item_id?`<div><span style="color: var(--text-dim);">Work Item:</span> <span style="color: var(--text-main);">${_(o.context.work_item_id)}</span></div>`:""}
        </div>
      `:""}

      <!-- Arguments Editor -->
      <div style="margin-bottom: 14px;">
        <div style="font-size: 11px; font-weight: 600; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.5px; margin-bottom: 6px;">
          Parameters (Editable before approval)
        </div>
        <textarea id="appr-args-${o.id}" class="form-textarea" rows="4" style="color: var(--green-400); font-family: var(--ff-mono); font-size: 11.5px; line-height: 1.4;">${_(JSON.stringify(o.sanitized_args,null,2))}</textarea>
      </div>

      <!-- Action Footer -->
      <div style="display: flex; justify-content: space-between; align-items: center; border-top: 1px solid var(--border); padding-top: 12px;">
        <div style="display: flex; align-items: center; gap: 8px;">
          <input id="appr-operator-${o.id}" type="text" class="form-input" placeholder="Operator ID" value="security-operator" style="width: 200px; padding: 5px 10px; font-size: 11px;">
        </div>

        <div style="display: flex; align-items: center; gap: 8px;">
          <button class="btn btn-danger" onclick="window.app.promptReject('${_(o.id)}')">
            ✕ Reject
          </button>
          <button class="btn btn-primary" onclick="window.app.submitApproval('${_(o.id)}')">
            ✓ Approve &amp; Execute
          </button>
        </div>
      </div>
    </div>
  `).join(""),r=i.length===0?"":`
    <div style="margin-top: 32px; border-top: 1px solid var(--border); padding-top: 18px;">
      <div style="font-size: 11px; font-weight: 700; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.5px; margin-bottom: 12px;">
        Recent History (${i.length})
      </div>
      <div style="background: var(--surface-card); border: 1px solid var(--border); border-radius: var(--radius-md); overflow: hidden; font-family: var(--ff-mono); font-size: 11.5px;">
        ${i.map((o)=>`
          <div style="display: flex; justify-content: space-between; align-items: center; padding: 10px 14px; border-bottom: 1px solid var(--border);">
            <div style="display: flex; align-items: center; gap: 10px;">
              <span class="brand-badge" style="${o.status==="approved"?"background: rgba(52, 211, 153, 0.12); color: var(--green-400); border-color: rgba(52, 211, 153, 0.3);":o.status==="rejected"?"background: rgba(248, 113, 113, 0.12); color: var(--red-400); border-color: rgba(248, 113, 113, 0.3);":"background: var(--surface-hover); color: var(--text-dim);"}">
                ${o.status.toUpperCase()}
              </span>
              <span style="font-weight: 600; color: var(--text-main);">${_(o.capability_id)}</span>
              <span style="color: var(--text-dim); font-size: 10.5px;">${_(o.id)}</span>
            </div>
            <div style="display: flex; align-items: center; gap: 16px; color: var(--text-dim);">
              ${o.operator?`<span>Operator: <span style="color: var(--text-muted);">${_(o.operator)}</span></span>`:""}
              ${o.reason?`<span style="color: var(--red-400); font-style: italic;">"${_(o.reason)}"</span>`:""}
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
      ${a}
    </div>

    ${r}
  `}function _(e){return String(e||"").replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;")}function E(){let e=s.getState(),t=e.auditEvents||[],i=e.auditStats||{total_events:0,by_status:{success:0,failed:0,denied:0,intercepted:0}},a=e.auditVerification,r=e.auditFilters,o=e.auditTotal??t.length,n=e.auditSelectedEvent,d=Object.keys(e.config?.mcpServers||{}),l=r.limit||25,g=r.offset||0,p=Math.floor(g/l)+1,v=Math.max(1,Math.ceil(o/l)),x=o===0?0:g+1,w=Math.min(g+l,o),h=u.getAuditExportUrl({actor_id:r.search?void 0:void 0,server_id:r.serverId!=="all"?r.serverId:void 0,event_type:r.eventType!=="all"?r.eventType:void 0,status:r.status!=="all"?r.status:void 0,search:r.search.trim()?r.search.trim():void 0},"csv"),k=u.getAuditExportUrl({server_id:r.serverId!=="all"?r.serverId:void 0,event_type:r.eventType!=="all"?r.eventType:void 0,status:r.status!=="all"?r.status:void 0,search:r.search.trim()?r.search.trim():void 0},"jsonl"),$=a?a.is_valid?`
      <div style="display: inline-flex; align-items: center; gap: 6px; padding: 4px 10px; background: rgba(34, 197, 94, 0.1); border: 1px solid rgba(34, 197, 94, 0.3); border-radius: var(--radius-sm); font-size: 11.5px; color: var(--green-400);">
        <span>\uD83D\uDEE1️</span>
        <span style="font-weight: 600;">Chain Verified: 100% Tamper Free (${a.total_records} events)</span>
      </div>
    `:`
      <div style="display: inline-flex; align-items: center; gap: 6px; padding: 4px 10px; background: rgba(239, 68, 68, 0.1); border: 1px solid rgba(239, 68, 68, 0.3); border-radius: var(--radius-sm); font-size: 11.5px; color: var(--red-400);">
        <span>⚠️</span>
        <span style="font-weight: 600;">TAMPER DETECTED at Record #${a.corrupted_at_index}</span>
      </div>
    `:`
    <button class="btn btn-ghost" style="padding: 4px 10px; font-size: 11.5px;" onclick="window.app.verifyAuditChain()">
      \uD83D\uDEE1️ Verify Cryptographic Hash Chain
    </button>
  `,f=d.map((y)=>`<option value="${b(y)}" ${r.serverId===y?"selected":""}>${b(y)}</option>`).join(""),z=`
    <div class="bento-card" style="padding: 14px 16px; margin-bottom: 16px; background: rgba(18, 24, 38, 0.7); border: 1px solid var(--border);">
      <div style="display: grid; grid-template-columns: 2fr 1fr 1.2fr 1.2fr auto auto; gap: 10px; align-items: center;">
        <!-- Full-text search input -->
        <div style="position: relative;">
          <input 
            type="text" 
            id="audit-search-input" 
            class="input-control" 
            style="width: 100%; padding-left: 28px; font-size: 12px; height: 32px; background: rgba(0,0,0,0.3); border: 1px solid var(--border);"
            placeholder="Search trace, actor, capability, hash, error..." 
            value="${b(r.search)}"
            oninput="window.app.handleAuditSearchInput(this.value)"
          />
          <span style="position: absolute; left: 8px; top: 7px; font-size: 12px; color: var(--text-dim);">\uD83D\uDD0D</span>
        </div>

        <!-- Status Filter -->
        <div>
          <select 
            class="input-control" 
            style="width: 100%; font-size: 12px; height: 32px; background: rgba(0,0,0,0.3); border: 1px solid var(--border);"
            onchange="window.app.handleAuditStatusFilter(this.value)"
          >
            <option value="all" ${r.status==="all"?"selected":""}>All Statuses</option>
            <option value="success" ${r.status==="success"?"selected":""}>\uD83D\uDFE2 Success</option>
            <option value="denied" ${r.status==="denied"?"selected":""}>\uD83D\uDD34 Denied</option>
            <option value="intercepted" ${r.status==="intercepted"?"selected":""}>\uD83D\uDFE1 HITL Intercept</option>
            <option value="failed" ${r.status==="failed"?"selected":""}>❌ Failed</option>
            <option value="cancelled" ${r.status==="cancelled"?"selected":""}>⚪ Cancelled</option>
          </select>
        </div>

        <!-- Event Type Filter -->
        <div>
          <select 
            class="input-control" 
            style="width: 100%; font-size: 12px; height: 32px; background: rgba(0,0,0,0.3); border: 1px solid var(--border);"
            onchange="window.app.handleAuditEventTypeFilter(this.value)"
          >
            <option value="all" ${r.eventType==="all"?"selected":""}>All Event Types</option>
            <option value="tool_execution" ${r.eventType==="tool_execution"?"selected":""}>Tool Execution</option>
            <option value="tool_intercepted_hitl" ${r.eventType==="tool_intercepted_hitl"?"selected":""}>HITL Intercept</option>
            <option value="approval_granted" ${r.eventType==="approval_granted"?"selected":""}>Approval Granted</option>
            <option value="approval_rejected" ${r.eventType==="approval_rejected"?"selected":""}>Approval Rejected</option>
            <option value="approval_expired" ${r.eventType==="approval_expired"?"selected":""}>Approval Expired</option>
            <option value="policy_violation" ${r.eventType==="policy_violation"?"selected":""}>Policy Violation</option>
            <option value="config_mutation" ${r.eventType==="config_mutation"?"selected":""}>Config Mutation</option>
            <option value="sampling_call" ${r.eventType==="sampling_call"?"selected":""}>Sampling Call</option>
            <option value="resource_access" ${r.eventType==="resource_access"?"selected":""}>Resource Access</option>
          </select>
        </div>

        <!-- Server Filter -->
        <div>
          <select 
            class="input-control" 
            style="width: 100%; font-size: 12px; height: 32px; background: rgba(0,0,0,0.3); border: 1px solid var(--border);"
            onchange="window.app.handleAuditServerFilter(this.value)"
          >
            <option value="all" ${r.serverId==="all"?"selected":""}>All MCP Servers</option>
            ${f}
          </select>
        </div>

        <!-- Clear Filters Button -->
        <div>
          <button 
            class="btn btn-ghost" 
            style="padding: 6px 12px; font-size: 11.5px; height: 32px;" 
            onclick="window.app.clearAuditFilters()"
            title="Reset all search queries and filters"
          >
            ✕ Reset
          </button>
        </div>
      </div>
    </div>
  `,j=`
    <div style="display: flex; justify-content: space-between; align-items: center; padding: 12px 16px; background: rgba(18, 24, 38, 0.5); border-radius: var(--radius-md); border: 1px solid var(--border); margin-top: 16px;">
      <div style="font-size: 12px; color: var(--text-dim); display: flex; align-items: center; gap: 8px;">
        <span>Showing <strong style="color: var(--text-main);">${x}–${w}</strong> of <strong style="color: var(--text-main);">${o}</strong> events</span>
        <span style="color: var(--border);">|</span>
        <span>Page Size:</span>
        <select 
          class="input-control" 
          style="font-size: 11.5px; padding: 2px 6px; height: 26px; background: rgba(0,0,0,0.3); border: 1px solid var(--border);"
          onchange="window.app.handleAuditPageSize(this.value)"
        >
          <option value="10" ${l===10?"selected":""}>10 / page</option>
          <option value="25" ${l===25?"selected":""}>25 / page</option>
          <option value="50" ${l===50?"selected":""}>50 / page</option>
          <option value="100" ${l===100?"selected":""}>100 / page</option>
        </select>
      </div>

      <div style="display: flex; align-items: center; gap: 6px;">
        <button 
          class="btn btn-ghost" 
          style="padding: 4px 8px; font-size: 11px; height: 28px;"
          ${p<=1?'disabled style="opacity: 0.4; cursor: not-allowed;"':""}
          onclick="window.app.auditGoToPage(1)"
          title="First Page"
        >
          ⏮ First
        </button>
        <button 
          class="btn btn-ghost" 
          style="padding: 4px 8px; font-size: 11px; height: 28px;"
          ${p<=1?'disabled style="opacity: 0.4; cursor: not-allowed;"':""}
          onclick="window.app.auditPrevPage()"
        >
          ◀ Prev
        </button>
        <span style="font-size: 12px; font-weight: 600; color: var(--text-main); padding: 0 8px;">
          Page ${p} of ${v}
        </span>
        <button 
          class="btn btn-ghost" 
          style="padding: 4px 8px; font-size: 11px; height: 28px;"
          ${p>=v?'disabled style="opacity: 0.4; cursor: not-allowed;"':""}
          onclick="window.app.auditNextPage()"
        >
          Next ▶
        </button>
        <button 
          class="btn btn-ghost" 
          style="padding: 4px 8px; font-size: 11px; height: 28px;"
          ${p>=v?'disabled style="opacity: 0.4; cursor: not-allowed;"':""}
          onclick="window.app.auditGoToPage(${v})"
          title="Last Page"
        >
          Last ⏭
        </button>
      </div>
    </div>
  `,R="";if(t.length===0)R=`
      <div style="padding: 48px 24px; text-align: center; color: var(--text-dim); background: var(--surface-card); border-radius: var(--radius-md); border: 1px dashed var(--border);">
        <div style="font-size: 28px; margin-bottom: 8px;">\uD83D\uDD0D</div>
        <div style="font-size: 14px; font-weight: 600; color: var(--text-main); margin-bottom: 4px;">No Matching Audit Events</div>
        <div style="font-size: 12px; max-width: 420px; margin: 0 auto;">No audit records match your currently selected filters. Try broadening your search or resetting filters.</div>
        <button class="btn btn-ghost" style="margin-top: 14px; font-size: 11.5px;" onclick="window.app.clearAuditFilters()">Reset Filters</button>
      </div>
    `;else R=t.map((y)=>{let L=new Date(Math.floor(y.timestamp_ns/1e6)).toLocaleString(),T='<span class="badge" style="background: rgba(34, 197, 94, 0.15); color: var(--green-400); font-weight: 600;">SUCCESS</span>';if(y.status==="denied")T='<span class="badge" style="background: rgba(239, 68, 68, 0.15); color: var(--red-400); font-weight: 600;">DENIED</span>';else if(y.status==="intercepted")T='<span class="badge" style="background: rgba(234, 179, 8, 0.15); color: var(--amber-300); font-weight: 600;">HITL INTERCEPT</span>';else if(y.status==="failed")T='<span class="badge" style="background: rgba(239, 68, 68, 0.15); color: var(--red-400); font-weight: 600;">FAILED</span>';else if(y.status==="cancelled")T='<span class="badge" style="background: rgba(148, 163, 184, 0.15); color: var(--text-muted); font-weight: 600;">CANCELLED</span>';let Q=y.sanitized_args?JSON.stringify(y.sanitized_args):"-",Y=y.actor_id||y.operator_id||"anonymous",W=y.server_id||"system",X=y.capability_id||y.event_type,Z=y.execution_latency_us?`${(y.execution_latency_us/1000).toFixed(1)}ms`:"-";return`
        <div class="bento-card" style="margin-bottom: 12px; padding: 16px; border: 1px solid var(--border); transition: border-color 0.15s ease;">
          <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
            <div style="display: flex; align-items: center; gap: 8px;">
              <span style="font-family: var(--ff-mono); font-size: 11px; font-weight: 700; color: var(--text-dim);">${b(y.id)}</span>
              ${T}
              <span style="font-size: 12px; font-weight: 600; color: var(--text-main);">${b(X)}</span>
            </div>
            <div style="display: flex; align-items: center; gap: 10px;">
              <div style="font-family: var(--ff-mono); font-size: 11px; color: var(--text-muted);">${b(L)}</div>
              <button 
                class="btn btn-ghost" 
                style="padding: 2px 8px; font-size: 11px; height: 24px;" 
                onclick="window.app.selectAuditEvent('${b(y.id)}')"
                title="Inspect event details & cryptographic payload"
              >
                Inspect \uD83D\uDD0D
              </button>
            </div>
          </div>
          
          <div style="display: grid; grid-template-columns: repeat(4, 1fr); gap: 8px; font-size: 11.5px; background: rgba(0,0,0,0.2); padding: 8px 12px; border-radius: var(--radius-sm); margin-bottom: 8px;">
            <div><span style="color: var(--text-muted);">Actor:</span> <strong style="color: var(--text-main);">${b(Y)}</strong></div>
            <div><span style="color: var(--text-muted);">Server:</span> <strong style="color: var(--cyan-400);">${b(W)}</strong></div>
            <div><span style="color: var(--text-muted);">Trace:</span> <code style="color: var(--cyan-400); font-size: 10.5px;">${b(y.trace_id)}</code></div>
            <div><span style="color: var(--text-muted);">Latency:</span> <span style="color: var(--amber-300);">${Z}</span></div>
          </div>

          <div style="font-family: var(--ff-mono); font-size: 11px; color: var(--text-dim); margin-bottom: 8px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">
            <span style="color: var(--text-muted);">Args:</span> ${b(Q)}
          </div>

          <div style="display: flex; justify-content: space-between; align-items: center; border-top: 1px solid var(--border); padding-top: 6px; font-size: 10.5px; font-family: var(--ff-mono); color: var(--text-muted);">
            <div><span style="color: var(--text-dim);">prev_hash:</span> ${b(y.prev_hash.slice(0,16))}...</div>
            <div><span style="color: var(--text-dim);">hash:</span> <span style="color: var(--green-400);">${b(y.hash.slice(0,16))}...</span></div>
          </div>
        </div>
      `}).join("");let P="";if(n){let y=new Date(Math.floor(n.timestamp_ns/1e6)).toISOString();P=`
      <div style="position: fixed; inset: 0; background: rgba(0,0,0,0.7); backdrop-filter: blur(4px); z-index: 9999; display: flex; align-items: center; justify-content: center; padding: 24px;" onclick="if (event.target === this) window.app.selectAuditEvent(null)">
        <div class="bento-card" style="width: 100%; max-width: 720px; max-height: 85vh; display: flex; flex-direction: column; overflow: hidden; background: #0f172a; border: 1px solid var(--border); box-shadow: 0 20px 40px rgba(0,0,0,0.5);">
          <!-- Modal Header -->
          <div style="display: flex; justify-content: space-between; align-items: center; padding: 16px 20px; border-bottom: 1px solid var(--border);">
            <div style="display: flex; align-items: center; gap: 8px;">
              <span style="font-size: 16px;">\uD83D\uDD12</span>
              <h2 style="font-size: 15px; font-weight: 700; color: var(--text-main); margin: 0;">Audit Event Details (${b(n.id)})</h2>
            </div>
            <button class="btn btn-ghost" style="padding: 4px 8px; font-size: 14px;" onclick="window.app.selectAuditEvent(null)">✕</button>
          </div>

          <!-- Modal Body -->
          <div style="padding: 20px; overflow-y: auto; display: flex; flex-direction: column; gap: 14px; font-size: 12px;">
            <div style="display: grid; grid-template-columns: repeat(2, 1fr); gap: 10px; background: rgba(0,0,0,0.25); padding: 12px; border-radius: var(--radius-sm);">
              <div><span style="color: var(--text-muted);">Timestamp:</span> <strong style="color: var(--text-main); font-family: var(--ff-mono); font-size: 11px;">${b(y)}</strong></div>
              <div><span style="color: var(--text-muted);">Status:</span> <strong style="color: var(--text-main);">${b(n.status.toUpperCase())}</strong></div>
              <div><span style="color: var(--text-muted);">Event Type:</span> <strong style="color: var(--text-main);">${b(n.event_type)}</strong></div>
              <div><span style="color: var(--text-muted);">Server:</span> <strong style="color: var(--cyan-400);">${b(n.server_id||"system")}</strong></div>
              <div><span style="color: var(--text-muted);">Capability:</span> <strong style="color: var(--text-main);">${b(n.capability_id||"-")}</strong></div>
              <div><span style="color: var(--text-muted);">Actor / Operator:</span> <strong style="color: var(--text-main);">${b(n.actor_id||n.operator_id||"anonymous")}</strong></div>
              <div><span style="color: var(--text-muted);">Trace ID:</span> <code style="color: var(--cyan-400);">${b(n.trace_id)}</code></div>
              <div><span style="color: var(--text-muted);">Request ID:</span> <code style="color: var(--cyan-400);">${b(n.request_id||"-")}</code></div>
              <div><span style="color: var(--text-muted);">Client IP:</span> <span style="color: var(--text-main);">${b(n.client_ip||"-")}</span></div>
              <div><span style="color: var(--text-muted);">Latency:</span> <span style="color: var(--amber-300);">${n.execution_latency_us?`${(n.execution_latency_us/1000).toFixed(2)} ms`:"-"}</span></div>
            </div>

            ${n.error_message?`
              <div style="background: rgba(239, 68, 68, 0.1); border: 1px solid rgba(239, 68, 68, 0.3); border-radius: var(--radius-sm); padding: 10px 12px; color: var(--red-400);">
                <div style="font-weight: 700; margin-bottom: 2px;">Error (${b(n.error_code||"ERROR")}):</div>
                <div style="font-family: var(--ff-mono); font-size: 11px;">${b(n.error_message)}</div>
              </div>
            `:""}

            <!-- Sanitized Arguments -->
            <div>
              <div style="font-weight: 600; color: var(--text-main); margin-bottom: 4px;">Sanitized Arguments</div>
              <pre style="background: rgba(0,0,0,0.4); padding: 10px; border-radius: var(--radius-sm); border: 1px solid var(--border); font-family: var(--ff-mono); font-size: 11px; max-height: 140px; overflow: auto; margin: 0; color: #cbd5e1;">${b(JSON.stringify(n.sanitized_args||{},null,2))}</pre>
            </div>

            <!-- Sanitized Response -->
            ${n.sanitized_response?`
              <div>
                <div style="font-weight: 600; color: var(--text-main); margin-bottom: 4px;">Sanitized Response</div>
                <pre style="background: rgba(0,0,0,0.4); padding: 10px; border-radius: var(--radius-sm); border: 1px solid var(--border); font-family: var(--ff-mono); font-size: 11px; max-height: 140px; overflow: auto; margin: 0; color: #cbd5e1;">${b(JSON.stringify(n.sanitized_response,null,2))}</pre>
              </div>
            `:""}

            <!-- Cryptographic Hashes -->
            <div style="background: rgba(0,0,0,0.3); padding: 12px; border-radius: var(--radius-sm); border: 1px solid var(--border);">
              <div style="font-weight: 600; color: var(--text-main); margin-bottom: 6px;">Tamper-Evidence Cryptographic Hashes</div>
              <div style="margin-bottom: 6px;">
                <span style="color: var(--text-muted); font-size: 10.5px;">Previous Chain Hash (prev_hash):</span>
                <div style="font-family: var(--ff-mono); font-size: 10.5px; color: var(--text-dim); word-break: break-all;">${b(n.prev_hash)}</div>
              </div>
              <div>
                <span style="color: var(--text-muted); font-size: 10.5px;">Record Hash Signature (hash):</span>
                <div style="font-family: var(--ff-mono); font-size: 10.5px; color: var(--green-400); word-break: break-all;">${b(n.hash)}</div>
              </div>
            </div>
          </div>

          <!-- Modal Footer -->
          <div style="padding: 12px 20px; border-top: 1px solid var(--border); display: flex; justify-content: flex-end;">
            <button class="btn btn-primary" style="font-size: 12px;" onclick="window.app.selectAuditEvent(null)">Close</button>
          </div>
        </div>
      </div>
    `}return`
    <div class="content-header" style="margin-bottom: 20px; display: flex; justify-content: space-between; align-items: center;">
      <div>
        <h1 style="font-size: 20px; font-weight: 700; margin-bottom: 4px; display: flex; align-items: center; gap: 8px;">
          <span>\uD83D\uDD12 WORM Audit Trail & Compliance Log</span>
        </h1>
        <p style="font-size: 12.5px; color: var(--text-dim);">Cryptographically tamper-evident, append-only execution log for SOC2 & ISO 27001 compliance.</p>
      </div>
      <div style="display: flex; gap: 8px; align-items: center;">
        ${$}
        <a href="${h}" download class="btn btn-ghost" style="font-size: 11.5px; text-decoration: none;" title="Export current filtered view as CSV">\uD83D\uDCE5 Export CSV</a>
        <a href="${k}" download class="btn btn-ghost" style="font-size: 11.5px; text-decoration: none;" title="Export current filtered view as JSONL">\uD83D\uDCE5 Export JSONL</a>
        <button class="btn btn-primary" style="font-size: 11.5px;" onclick="window.app.refreshAuditEvents()">\uD83D\uDD04 Refresh</button>
      </div>
    </div>

    <!-- Stats summary cards -->
    <div style="display: grid; grid-template-columns: repeat(4, 1fr); gap: 12px; margin-bottom: 20px;">
      <div class="bento-card" style="padding: 14px; text-align: center;">
        <div style="font-size: 11px; color: var(--text-muted); text-transform: uppercase; font-weight: 600;">Total Events</div>
        <div style="font-size: 22px; font-weight: 800; color: var(--text-main); margin-top: 4px;">${i.total_events}</div>
      </div>
      <div class="bento-card" style="padding: 14px; text-align: center;">
        <div style="font-size: 11px; color: var(--text-muted); text-transform: uppercase; font-weight: 600;">Successful Calls</div>
        <div style="font-size: 22px; font-weight: 800; color: var(--green-400); margin-top: 4px;">${i.by_status.success}</div>
      </div>
      <div class="bento-card" style="padding: 14px; text-align: center;">
        <div style="font-size: 11px; color: var(--text-muted); text-transform: uppercase; font-weight: 600;">HITL Intercepts</div>
        <div style="font-size: 22px; font-weight: 800; color: var(--amber-300); margin-top: 4px;">${i.by_status.intercepted}</div>
      </div>
      <div class="bento-card" style="padding: 14px; text-align: center;">
        <div style="font-size: 11px; color: var(--text-muted); text-transform: uppercase; font-weight: 600;">Policy Denials</div>
        <div style="font-size: 22px; font-weight: 800; color: var(--red-400); margin-top: 4px;">${i.by_status.denied}</div>
      </div>
    </div>

    <!-- Search & Filter Toolbar -->
    ${z}

    <!-- Event Timeline List Header -->
    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px;">
      <h2 style="font-size: 14px; font-weight: 600; color: var(--text-main);">Sequential Audit Ledger (SHA-256 Hash Chained)</h2>
      <span style="font-size: 11.5px; color: var(--text-dim);">${t.length} events loaded on this page</span>
    </div>

    <!-- Event Rows -->
    <div>
      ${R}
    </div>

    <!-- Pagination Footer -->
    ${o>0?j:""}

    <!-- Modal Popup for Event Inspection -->
    ${P}
  `}function b(e){if(!e)return"";return String(e).replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;").replace(/'/g,"&#039;")}function K(){let t=s.getState().config.policy||{},i=t.allow||[],a=t.deny||[],r=t.redact_keys||t.redactKeys||[],o=t.require_approval||t.requireApproval||[],n=i.length===0?`
    <div style="color: var(--text-dim); font-size: 12px;">No allow list (all non-denied operations permitted)</div>
  `:i.map((p,v)=>`
    <div style="display: flex; justify-content: space-between; align-items: center; background: var(--surface); padding: 8px 12px; border-radius: var(--radius-sm); border: 1px solid var(--border);">
      <span style="font-family: var(--ff-mono); font-size: 12px; color: var(--green-400);">✔ ${O(p)}</span>
      <button class="btn btn-ghost" style="padding: 2px 6px; font-size: 11px; color: var(--red-400);" onclick="window.app.removePolicyRule('allow', ${v})">✕</button>
    </div>
  `).join(""),d=a.length===0?`
    <div style="color: var(--text-dim); font-size: 12px;">No deny rules configured</div>
  `:a.map((p,v)=>`
    <div style="display: flex; justify-content: space-between; align-items: center; background: var(--surface); padding: 8px 12px; border-radius: var(--radius-sm); border: 1px solid var(--border);">
      <span style="font-family: var(--ff-mono); font-size: 12px; color: var(--red-400);">✖ ${O(p)}</span>
      <button class="btn btn-ghost" style="padding: 2px 6px; font-size: 11px; color: var(--red-400);" onclick="window.app.removePolicyRule('deny', ${v})">✕</button>
    </div>
  `).join(""),l=o.length===0?`
    <div style="color: var(--text-dim); font-size: 12px;">No human-in-the-loop approval rules configured</div>
  `:o.map((p,v)=>`
    <div style="display: flex; justify-content: space-between; align-items: center; background: var(--surface); padding: 8px 12px; border-radius: var(--radius-sm); border: 1px solid var(--border);">
      <span style="font-family: var(--ff-mono); font-size: 12px; color: var(--amber-400);">\uD83D\uDEE1️ ${O(p)}</span>
      <button class="btn btn-ghost" style="padding: 2px 6px; font-size: 11px; color: var(--red-400);" onclick="window.app.removePolicyRule('requireApproval', ${v})">✕</button>
    </div>
  `).join(""),g=r.length===0?`
    <div style="color: var(--text-dim); font-size: 12px;">No key redaction patterns configured</div>
  `:r.map((p,v)=>`
    <span class="brand-badge" style="color: var(--amber-300); padding: 5px 10px; font-size: 11px; display: inline-flex; align-items: center; gap: 6px;">
      ${O(p)}
      <span style="cursor: pointer; color: var(--red-400); font-weight: bold;" onclick="window.app.removePolicyRule('redact', ${v})">✕</span>
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
          ${n}
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
          ${d}
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
          ${l}
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
          ${g}
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
  `}function O(e){return String(e||"").replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;")}function J(){let e=s.getState(),t=e.config,i=Object.entries(t.capabilityAliases||{}),a=Object.entries(t.resourceAliases||{}),r=Object.entries(t.promptAliases||{}),o="";if(i.length===0&&a.length===0&&r.length===0)o=`
      <div style="padding: 24px; text-align: center; color: var(--text-dim);">
        No facade aliases configured in ${S(e.configPath)}. Add short names to prune token payload sizes.
      </div>
    `;else{for(let[n,d]of i)o+=`
        <div class="feed-row" style="grid-template-columns: 90px 180px 1fr 80px;">
          <span style="color: var(--cyan-400);">Tool</span>
          <span style="font-weight: 700; color: var(--text-main); font-family: var(--ff-mono);">${S(n)}</span>
          <span style="color: var(--text-dim); font-family: var(--ff-mono);">${S(d)}</span>
          <div style="text-align: right;">
            <button class="btn btn-ghost" style="padding: 2px 6px; color: var(--red-400);" onclick="window.app.deleteAlias('tool', '${S(n)}')">✕</button>
          </div>
        </div>
      `;for(let[n,d]of a)o+=`
        <div class="feed-row" style="grid-template-columns: 90px 180px 1fr 80px;">
          <span style="color: var(--green-400);">Resource</span>
          <span style="font-weight: 700; color: var(--text-main); font-family: var(--ff-mono);">${S(n)}</span>
          <span style="color: var(--text-dim); font-family: var(--ff-mono);">${S(d)}</span>
          <div style="text-align: right;">
            <button class="btn btn-ghost" style="padding: 2px 6px; color: var(--red-400);" onclick="window.app.deleteAlias('resource', '${S(n)}')">✕</button>
          </div>
        </div>
      `;for(let[n,d]of r)o+=`
        <div class="feed-row" style="grid-template-columns: 90px 180px 1fr 80px;">
          <span style="color: var(--amber-300);">Prompt</span>
          <span style="font-weight: 700; color: var(--text-main); font-family: var(--ff-mono);">${S(n)}</span>
          <span style="color: var(--text-dim); font-family: var(--ff-mono);">${S(d)}</span>
          <div style="text-align: right;">
            <button class="btn btn-ghost" style="padding: 2px 6px; color: var(--red-400);" onclick="window.app.deleteAlias('prompt', '${S(n)}')">✕</button>
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
  `}function S(e){return String(e||"").replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;")}var N=[{id:"github",name:"GitHub",category:"devtools",description:"Explore repositories, issues, pull requests, branches, and commit histories.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-github"],envFields:[{key:"GITHUB_PERSONAL_ACCESS_TOKEN",label:"GitHub Personal Access Token",placeholder:"ghp_...",required:!0,description:"Classic or fine-grained token with repo scope."}]},{id:"git",name:"Git (Local)",category:"devtools",description:"Read local Git repository status, diffs, log histories, and commit changes.",badge:"Official / uvx",command:"uvx",defaultArgs:["mcp-server-git","--repository","."],argsPlaceholder:"mcp-server-git --repository /path/to/repo",envFields:[]},{id:"filesystem",name:"Filesystem",category:"devtools",description:"Secure, sandboxed access to local files and directories for AI workflows.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-filesystem","."],argsPlaceholder:"-y @modelcontextprotocol/server-filesystem /allowed/dir1 /allowed/dir2",envFields:[]},{id:"memory",name:"Memory Graph",category:"devtools",description:"Persistent knowledge-graph based memory for multi-turn agent learning.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-memory"],envFields:[]},{id:"chrome-devtools",name:"Chrome DevTools",category:"devtools",description:"Inspect live DOM, execute scripts, read console logs, and capture network traces in Chrome.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-puppeteer"],envFields:[]},{id:"sentry",name:"Sentry",category:"devtools",description:"Query production error events, stack traces, and issue frequencies directly from Sentry.",badge:"uvx / Telemetry",command:"uvx",defaultArgs:["mcp-server-sentry"],envFields:[{key:"SENTRY_AUTH_TOKEN",label:"Sentry Auth Token",placeholder:"sntrys_...",required:!0}]},{id:"playwright",name:"Playwright Browser",category:"browser",description:"Headless / headed browser automation for scraping, form filling, and UI interaction.",badge:"Popular #1 / npx",command:"npx",defaultArgs:["-y","@executeautomation/playwright-mcp-server"],envFields:[]},{id:"puppeteer",name:"Puppeteer",category:"browser",description:"Official browser automation server for web page scraping and screenshot capture.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-puppeteer"],envFields:[]},{id:"brave-search",name:"Brave Search",category:"browser",description:"Real-time privacy-preserving web search and local point-of-interest query engine.",badge:"Official / Search",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-brave-search"],envFields:[{key:"BRAVE_API_KEY",label:"Brave Search API Key",placeholder:"BSA...",required:!0}]},{id:"tavily",name:"Tavily Search",category:"browser",description:"AI-optimized web search engine structured specifically for LLM context injection.",badge:"Community / Stdio",command:"npx",defaultArgs:["-y","@tavily/mcp-server"],envFields:[{key:"TAVILY_API_KEY",label:"Tavily API Key",placeholder:"tvly-...",required:!0}]},{id:"fetch",name:"Fetch / Web Markdown",category:"browser",description:"Download web pages, strip clutter, and convert raw HTML to clean markdown text.",badge:"Official / uvx",command:"uvx",defaultArgs:["mcp-server-fetch"],envFields:[]},{id:"postgres",name:"PostgreSQL",category:"database",description:"Read schemas, inspect tables, and execute SQL queries against PostgreSQL databases.",badge:"Official / Database",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-postgres","postgresql://user:pass@localhost:5432/mydb"],argsPlaceholder:"-y @modelcontextprotocol/server-postgres postgresql://user:pass@localhost:5432/dbname",envFields:[]},{id:"sqlite",name:"SQLite",category:"database",description:"Local embedded SQLite query runner and schema inspector.",badge:"Official / uvx",command:"uvx",defaultArgs:["mcp-server-sqlite","--db-path","./app.db"],argsPlaceholder:"mcp-server-sqlite --db-path /path/to/database.sqlite",envFields:[]},{id:"supabase",name:"Supabase",category:"database",description:"Query database tables, manage auth policies, and inspect storage in Supabase.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@supabase/mcp-server"],envFields:[{key:"SUPABASE_ACCESS_TOKEN",label:"Supabase Personal Access Token",placeholder:"sbp_...",required:!0},{key:"SUPABASE_PROJECT_REF",label:"Supabase Project Reference ID",placeholder:"abcdefghijklmnop",required:!1}]},{id:"redis",name:"Redis",category:"database",description:"Inspect cached keys, hash sets, lists, TTLs, and pub/sub channels in Redis.",badge:"uvx / Key-Value",command:"uvx",defaultArgs:["mcp-server-redis","--url","redis://localhost:6379"],argsPlaceholder:"mcp-server-redis --url redis://localhost:6379",envFields:[]},{id:"s3",name:"AWS S3 / Cloud Storage",category:"database",description:"Browse S3 buckets, fetch object metadata, and download files from cloud storage.",badge:"uvx / Cloud Storage",command:"uvx",defaultArgs:["mcp-server-s3","--bucket","my-bucket-name"],argsPlaceholder:"mcp-server-s3 --bucket bucket-name --region us-east-1",envFields:[{key:"AWS_ACCESS_KEY_ID",label:"AWS Access Key ID",placeholder:"AKIA...",required:!0},{key:"AWS_SECRET_ACCESS_KEY",label:"AWS Secret Access Key",placeholder:"...",required:!0},{key:"AWS_REGION",label:"AWS Region",placeholder:"us-east-1",required:!1}]},{id:"linear",name:"Linear",category:"productivity",description:"Search, create, and triage Linear issues, cycles, teams, and project roadmaps.",badge:"Productivity / Stdio",command:"npx",defaultArgs:["-y","mcp-linear"],envFields:[{key:"LINEAR_API_KEY",label:"Linear API Key",placeholder:"lin_api_...",required:!0}]},{id:"slack",name:"Slack",category:"productivity",description:"Read channels, post messages, inspect threads, and search team discussions.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-slack"],envFields:[{key:"SLACK_BOT_TOKEN",label:"Slack Bot User Token",placeholder:"xoxb-...",required:!0},{key:"SLACK_TEAM_ID",label:"Slack Team ID",placeholder:"T01234567",required:!0}]},{id:"notion",name:"Notion",category:"productivity",description:"Search Notion workspace pages, read nested blocks, and query database entries.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-notion"],envFields:[{key:"NOTION_API_KEY",label:"Notion Internal Integration Token",placeholder:"secret_...",required:!0}]},{id:"jira",name:"Jira / Atlassian",category:"productivity",description:"Manage Jira issues, search JQL, read sprint statuses, and inspect boards.",badge:"uvx / Atlassian",command:"uvx",defaultArgs:["mcp-server-jira","--url","https://your-domain.atlassian.net","--email","user@example.com"],argsPlaceholder:"mcp-server-jira --url https://org.atlassian.net --email me@org.com",envFields:[{key:"JIRA_API_TOKEN",label:"Atlassian API Token",placeholder:"ATATT3...",required:!0}]},{id:"google-drive",name:"Google Drive",category:"productivity",description:"Search, list, and read documents, spreadsheets, and drive files.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-gdrive"],envFields:[{key:"GOOGLE_APPLICATION_CREDENTIALS",label:"Google Credentials JSON Path",placeholder:"/path/to/credentials.json",required:!0}]},{id:"docker",name:"Docker",category:"cloud",description:"Inspect running containers, tail container logs, list images, and manage compose services.",badge:"uvx / DevOps",command:"uvx",defaultArgs:["mcp-server-docker"],envFields:[]},{id:"kubernetes",name:"Kubernetes (K8s)",category:"cloud",description:"Query cluster pods, services, deployment status, and inspect Kubernetes logs.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-kubernetes"],envFields:[{key:"KUBECONFIG",label:"Kubeconfig File Path (Optional)",placeholder:"~/.kube/config",required:!1}]},{id:"cloudflare",name:"Cloudflare",category:"cloud",description:"Manage Cloudflare Workers, KV namespaces, D1 databases, Vectorize indexes, and DNS.",badge:"Official / Cloudflare",command:"npx",defaultArgs:["-y","@cloudflare/mcp-server-cloudflare"],envFields:[{key:"CLOUDFLARE_API_TOKEN",label:"Cloudflare API Token",placeholder:"...",required:!0},{key:"CLOUDFLARE_ACCOUNT_ID",label:"Cloudflare Account ID",placeholder:"...",required:!0}]},{id:"terraform",name:"Terraform",category:"cloud",description:"Inspect Terraform state files, resource dependency graphs, and plan previews.",badge:"uvx / IaC",command:"uvx",defaultArgs:["mcp-server-terraform"],envFields:[]}];class B{activeTemplateCategory="all";activeTemplateFilter="";selectedTemplate=null;async init(){let e=window.location.port?`:${window.location.port}`:"",t=document.getElementById("daemon-port-label");if(t)t.textContent=`Daemon ${e}`;await this.refreshData(),this.initSSE(),this.render(),s.subscribe(()=>{this.render()})}auditSearchTimeout=null;async refreshData(){try{let t=s.getState().auditFilters,[i,a,r,o,n,d,l,g]=await Promise.all([u.getConfig(),u.listCapabilities(),u.listResources(),u.listPrompts(),u.getCatalogEvents(),u.listApprovals(),u.listAuditEvents({server_id:t.serverId!=="all"?t.serverId:void 0,event_type:t.eventType!=="all"?t.eventType:void 0,status:t.status!=="all"?t.status:void 0,search:t.search.trim()?t.search.trim():void 0,limit:t.limit,offset:t.offset}),u.getAuditStats()]);if(i.ok)s.setState({configPath:i.config_path,config:i.config,serverStatuses:i.server_statuses||{},circuitBreakers:i.circuit_breakers||[],metrics:{totalCatalogRequests:i.metrics?.total_catalog_requests||0,totalEtagHits:i.metrics?.total_etag_hits||0,totalToolCalls:i.metrics?.total_tool_calls||0,totalToolDurationUs:i.metrics?.total_tool_duration_us||0}});if(a&&Array.isArray(a.capabilities))s.setState({capabilities:a.capabilities});if(r&&Array.isArray(r.resources))s.setState({resources:r.resources});if(o&&Array.isArray(o.prompts))s.setState({prompts:o.prompts});if(n&&Array.isArray(n.events))s.setState({catalogEvents:n.events});if(d&&Array.isArray(d.approvals))s.setState({approvals:d.approvals});if(l&&Array.isArray(l.events))s.setState({auditEvents:l.events,auditTotal:l.total??l.events.length});if(g&&g.ok)s.setState({auditStats:g})}catch(e){console.error("Failed to fetch daemon state:",e)}}async refreshAuditEvents(){try{let t=s.getState().auditFilters,[i,a]=await Promise.all([u.listAuditEvents({server_id:t.serverId!=="all"?t.serverId:void 0,event_type:t.eventType!=="all"?t.eventType:void 0,status:t.status!=="all"?t.status:void 0,search:t.search.trim()?t.search.trim():void 0,limit:t.limit,offset:t.offset}),u.getAuditStats()]);if(i&&Array.isArray(i.events))s.setState({auditEvents:i.events,auditTotal:i.total??i.events.length});if(a&&a.ok)s.setState({auditStats:a})}catch(e){console.error("Failed to refresh audit events:",e)}}handleAuditSearchInput(e){let i={...s.getState().auditFilters,search:e,offset:0};s.setState({auditFilters:i}),clearTimeout(this.auditSearchTimeout),this.auditSearchTimeout=setTimeout(()=>{this.refreshAuditEvents()},250)}handleAuditStatusFilter(e){let t=s.getState();s.setState({auditFilters:{...t.auditFilters,status:e,offset:0}}),this.refreshAuditEvents()}handleAuditEventTypeFilter(e){let t=s.getState();s.setState({auditFilters:{...t.auditFilters,eventType:e,offset:0}}),this.refreshAuditEvents()}handleAuditServerFilter(e){let t=s.getState();s.setState({auditFilters:{...t.auditFilters,serverId:e,offset:0}}),this.refreshAuditEvents()}handleAuditPageSize(e){let t=parseInt(e,10)||25,i=s.getState();s.setState({auditFilters:{...i.auditFilters,limit:t,offset:0}}),this.refreshAuditEvents()}clearAuditFilters(){let e=s.getState();s.setState({auditFilters:{search:"",status:"all",eventType:"all",serverId:"all",limit:e.auditFilters.limit||25,offset:0}}),this.refreshAuditEvents()}auditPrevPage(){let e=s.getState(),{limit:t,offset:i}=e.auditFilters,a=Math.max(0,i-t);if(a!==i)s.setState({auditFilters:{...e.auditFilters,offset:a}}),this.refreshAuditEvents()}auditNextPage(){let e=s.getState(),{limit:t,offset:i}=e.auditFilters,a=e.auditTotal;if(i+t<a)s.setState({auditFilters:{...e.auditFilters,offset:i+t}}),this.refreshAuditEvents()}auditGoToPage(e){let t=s.getState(),{limit:i}=t.auditFilters,a=Math.max(0,(e-1)*i);s.setState({auditFilters:{...t.auditFilters,offset:a}}),this.refreshAuditEvents()}selectAuditEvent(e){if(!e){s.setState({auditSelectedEvent:null});return}let i=s.getState().auditEvents.find((a)=>a.id===e)||null;s.setState({auditSelectedEvent:i})}async verifyAuditChain(){try{let e=await u.verifyAuditChain();if(e&&e.report)s.setState({auditVerification:e.report})}catch(e){console.error("Failed to verify audit chain:",e)}}async refreshApprovals(){try{let e=await u.listApprovals();if(e&&Array.isArray(e.approvals))s.setState({approvals:e.approvals})}catch(e){console.error("Failed to refresh approvals:",e)}}initSSE(){try{let e=new EventSource("/v1/resources/updates");e.onmessage=(t)=>{s.addEventLog("SSE","/v1/resources/updates","UPDATED","0.1ms"),this.refreshData()}}catch(e){console.warn("SSE connection unavailable")}}switchTab(e){s.setState({activeTab:e}),this.refreshData()}render(){let e=s.getState(),t=document.getElementById("app-main");if(!t)return;let i=e.approvals.filter((n)=>n.status==="pending").length,a=document.getElementById("nav-approvals-badge");if(a)a.textContent=i>0?`${i}`:"",a.style.display=i>0?"inline-block":"none";document.querySelectorAll(".nav-item").forEach((n)=>{if(n.getAttribute("data-tab")===e.activeTab)n.classList.add("active");else n.classList.remove("active")});let r=document.getElementById("top-title"),o={overview:"Overview Cockpit",servers:"Server Hub & Connections",playground:"MCP Capability Playground",approvals:"Human-in-the-Loop Review Queue",audit:"WORM Audit & Compliance Ledger",policy:"Security Governance & Redaction",aliases:"Facade & Alias Studio"};if(r)r.textContent=o[e.activeTab];switch(e.activeTab){case"overview":t.innerHTML=q();break;case"servers":t.innerHTML=I();break;case"playground":t.innerHTML=U();break;case"approvals":t.innerHTML=F(e);break;case"audit":t.innerHTML=E();break;case"policy":t.innerHTML=K();break;case"aliases":t.innerHTML=J();break}}async submitApproval(e){let t=document.getElementById(`appr-operator-${e}`),i=document.getElementById(`appr-args-${e}`),a=t?.value.trim()||"security-operator",r=void 0;if(i&&i.value.trim())try{r=JSON.parse(i.value.trim())}catch{alert("Invalid JSON in arguments editor");return}let o=await u.approveTicket(e,a,r);if(o.ok)await this.refreshApprovals();else alert(`Approval failed: ${o.error||"Unknown error"}`)}async promptReject(e){let t=prompt("Reason for rejection (will be returned to the calling agent):");if(t===null)return;let a=document.getElementById(`appr-operator-${e}`)?.value.trim()||"security-operator",r=await u.rejectTicket(e,a,t);if(r.ok)await this.refreshApprovals();else alert(`Rejection failed: ${r.error||"Unknown error"}`)}selectCapability(e){s.setState({selectedCapabilityId:e})}filterCapabilities(e){let t=e.toLowerCase().trim(),a=s.getState().capabilities.filter((o)=>o.id.toLowerCase().includes(t)||o.summary&&o.summary.toLowerCase().includes(t)||o.server&&o.server.toLowerCase().includes(t)),r=document.getElementById("pg-cap-list");if(r)if(a.length===0)r.innerHTML=`
          <div style="padding: 24px 16px; text-align: center; color: var(--text-dim); font-size: 11.5px;">
            No capabilities match "${c(e)}"
          </div>
        `;else r.innerHTML=a.map((o)=>`
          <div class="cap-item ${o.id===s.getState().selectedCapabilityId?"active":""}" onclick="window.app.selectCapability('${c(o.id)}')">
            <div style="display: flex; justify-content: space-between; align-items: center;">
              <span style="font-weight: 600; color: var(--text-main); font-family: var(--ff-mono); font-size: 12px;">${c(o.id)}</span>
              <span style="font-size: 10px; color: var(--green-400);">${c(o.mode||"read")}</span>
            </div>
            <div style="font-size: 11px; color: var(--text-dim); margin-top: 2px;">server: ${c(o.server||"local")}</div>
          </div>
        `).join("")}async executePlaygroundTool(){let e=s.getState(),t=e.selectedCapabilityId||(e.capabilities[0]?e.capabilities[0].id:null);if(!t)return;let i=document.getElementById("pg-args-input")?.value||"{}",a=document.getElementById("pg-context-input")?.value||void 0,r=document.getElementById("pg-jsonpath-input")?.value.trim()||void 0,o=document.getElementById("pg-limit-lines-input")?.value.trim()||void 0,n=document.getElementById("pg-truncate-bytes-input")?.value.trim()||void 0,d={};try{d=JSON.parse(i)}catch{alert("Invalid arguments JSON object");return}if(r)d._jsonpath=r;if(o&&!isNaN(Number(o)))d._limit_lines=Number(o);if(n&&!isNaN(Number(n)))d._truncate_bytes=Number(n);let l=document.getElementById("pg-status-badge"),g=document.getElementById("pg-response-json");if(l)l.textContent="EXECUTING...",l.style.color="var(--amber-400)";try{let p=await u.callCapability({capability_id:t,args:d,context:a?{operation_id:a}:void 0});s.setState({executionResult:{status:p.status,durationMs:p.durationMs,data:p.data}}),s.addEventLog("POST",`/v1/tools/call → ${t}`,p.status===200?"200 OK":`HTTP ${p.status}`,`${p.durationMs.toFixed(1)}ms`),u.getConfig().then((v)=>{if(v.ok&&v.circuit_breakers)s.setState({circuitBreakers:v.circuit_breakers})})}catch(p){if(l)l.textContent="ERROR",l.style.color="var(--red-400)";if(g)g.textContent=p.toString()}}setPlaygroundMode(e){s.setState({playgroundMode:e})}selectResource(e){s.setState({selectedResourceId:e,resourceReadResult:null})}filterResources(e){let t=e.toLowerCase().trim(),a=(s.getState().resources||[]).filter((o)=>o.id.toLowerCase().includes(t)||o.name&&o.name.toLowerCase().includes(t)||o.uri&&o.uri.toLowerCase().includes(t)||o.server&&o.server.toLowerCase().includes(t)),r=document.getElementById("pg-res-list");if(r)if(a.length===0)r.innerHTML=`
          <div style="padding: 24px 16px; text-align: center; color: var(--text-dim); font-size: 11.5px;">
            No resources match "${c(e)}"
          </div>
        `;else r.innerHTML=a.map((o)=>{let n=o.uri?o.uri.split(":")[0]:"res";return`
            <div class="cap-item ${o.id===s.getState().selectedResourceId?"active":""}" onclick="window.app.selectResource('${c(o.id)}')">
              <div style="display: flex; justify-content: space-between; align-items: center;">
                <span style="font-weight: 600; color: var(--text-main); font-family: var(--ff-mono); font-size: 12px;">${c(o.name||o.id)}</span>
                <span class="badge" style="font-size: 9.5px; background: rgba(56, 189, 248, 0.15); color: var(--cyan-400);">${c(n)}</span>
              </div>
              <div style="font-size: 11px; color: var(--text-dim); margin-top: 2px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">${c(o.uri)}</div>
              <div style="display: flex; justify-content: space-between; font-size: 10px; color: var(--text-muted); margin-top: 4px;">
                <span>server: ${c(o.server||"local")}</span>
                <span>${c(o.mime_type||"text/plain")}</span>
              </div>
            </div>
          `}).join("")}async executeReadResource(){let e=s.getState(),t=e.selectedResourceId||(e.resources[0]?e.resources[0].id:null);if(!t)return;let i=document.getElementById("pg-res-jsonpath-input")?.value.trim()||void 0,a=document.getElementById("pg-res-lines-input")?.value.trim()||void 0,r=document.getElementById("pg-res-bytes-input")?.value.trim()||void 0,o={resource_id:t};if(i)o._jsonpath=i;if(a&&!isNaN(Number(a)))o._limit_lines=Number(a);if(r&&!isNaN(Number(r)))o._truncate_bytes=Number(r);try{let n=await u.readResource(o);s.setState({resourceReadResult:{status:n.status,durationMs:n.durationMs,data:n.data}}),s.addEventLog("POST",`/v1/resources/read → ${t}`,n.status===200?"200 OK":`HTTP ${n.status}`,`${n.durationMs.toFixed(1)}ms`)}catch(n){s.setState({resourceReadResult:{status:500,durationMs:0,data:{error:n.toString()}}})}}selectPrompt(e){s.setState({selectedPromptId:e,promptGetResult:null})}filterPrompts(e){let t=e.toLowerCase().trim(),a=(s.getState().prompts||[]).filter((o)=>o.id.toLowerCase().includes(t)||o.name&&o.name.toLowerCase().includes(t)||o.description&&o.description.toLowerCase().includes(t)||o.server&&o.server.toLowerCase().includes(t)),r=document.getElementById("pg-prompt-list");if(r)if(a.length===0)r.innerHTML=`
          <div style="padding: 24px 16px; text-align: center; color: var(--text-dim); font-size: 11.5px;">
            No prompts match "${c(e)}"
          </div>
        `;else r.innerHTML=a.map((o)=>{let n=o.arguments?o.arguments.length:0;return`
            <div class="cap-item ${o.id===s.getState().selectedPromptId?"active":""}" onclick="window.app.selectPrompt('${c(o.id)}')">
              <div style="display: flex; justify-content: space-between; align-items: center;">
                <span style="font-weight: 600; color: var(--text-main); font-family: var(--ff-mono); font-size: 12px;">${c(o.name||o.id)}</span>
                <span class="badge" style="font-size: 9.5px; background: rgba(168, 85, 247, 0.15); color: var(--purple-400);">${n} args</span>
              </div>
              <div style="font-size: 11px; color: var(--text-dim); margin-top: 2px;">${c(o.description||o.title||"Prompt template")}</div>
              <div style="font-size: 10px; color: var(--text-muted); margin-top: 4px;">server: ${c(o.server||"local")}</div>
            </div>
          `}).join("")}async executeGetPrompt(){let e=s.getState(),t=e.selectedPromptId||(e.prompts[0]?e.prompts[0].id:null);if(!t)return;let i=document.querySelectorAll(".prompt-arg-input"),a={};i.forEach((r)=>{let o=r,n=o.getAttribute("data-arg-name");if(n&&o.value.trim())a[n]=o.value.trim()});try{let r=await u.getPrompt({prompt_id:t,arguments:a});s.setState({promptGetResult:{status:r.status,durationMs:r.durationMs,data:r.data}}),s.addEventLog("POST",`/v1/prompts/get → ${t}`,r.status===200?"200 OK":`HTTP ${r.status}`,`${r.durationMs.toFixed(1)}ms`)}catch(r){s.setState({promptGetResult:{status:500,durationMs:0,data:{error:r.toString()}}})}}toggleBatchPlayground(){let e=document.getElementById("pg-args-input");if(!e)return;let t=[{id:"step_1",capability_id:"sqlite.read_query",args:{query:"SELECT * FROM users LIMIT 2"}},{id:"step_2",capability_id:"github.issues.search",args:{query:"label:bug"},continue_on_error:!0}];e.value=JSON.stringify(t,null,2)}async submitPolicyRule(e){let t=e==="allow"?"policy-new-allow":e==="deny"?"policy-new-deny":"policy-new-redact",i=document.getElementById(t);if(!i)return;let a=i.value.trim();if(!a)return;await this.addPolicyRule(e,a),i.value=""}async addPolicyRule(e,t){let i=(t||"").trim();if(!i)return;let r=s.getState().config.policy||{},o=[...r.allow||[]],n=[...r.deny||[]],d=[...r.redact_keys||r.redactKeys||[]];if(e==="allow"&&!o.includes(i))o.push(i);if(e==="deny"&&!n.includes(i))n.push(i);if(e==="redact"&&!d.includes(i))d.push(i);let l=await u.savePolicy({allow:o,deny:n,redact_keys:d,redactKeys:d});if(!l.ok)alert(`Failed to save policy rule: ${l.error||"Unknown error"}`);await this.refreshData()}async removePolicyRule(e,t){let a=s.getState().config.policy||{},r=[...a.allow||[]],o=[...a.deny||[]],n=[...a.redact_keys||a.redactKeys||[]];if(e==="allow")r.splice(t,1);if(e==="deny")o.splice(t,1);if(e==="redact")n.splice(t,1);let d=await u.savePolicy({allow:r,deny:o,redact_keys:n,redactKeys:n});if(!d.ok)alert(`Failed to update policy: ${d.error||"Unknown error"}`);await this.refreshData()}testPolicySandbox(e){let t=document.getElementById("policy-test-verdict");if(!t)return;let i=e.trim();if(!i){t.textContent="ENTER ID",t.style.color="var(--text-dim)";return}let r=s.getState().config.policy||{},o=r.deny||[],n=r.allow||[],d=(l,g)=>{if(l==="*")return!0;if(l.endsWith("*"))return g.startsWith(l.slice(0,-1));return l===g};if(o.some((l)=>d(l,i))){t.textContent="DENIED (Strict Block)",t.style.color="var(--red-400)";return}if(n.length>0&&!n.some((l)=>d(l,i))){t.textContent="DENIED (Not in Allow List)",t.style.color="var(--red-400)";return}t.textContent="ALLOWED",t.style.color="var(--green-400)"}async deleteServer(e){if(!confirm(`Are you sure you want to remove server '${e}' from config?`))return;await u.deleteServer(e),await this.refreshData()}openAddServerModal(){this.closeModals();let e=document.getElementById("modal-srv-title"),t=document.getElementById("modal-srv-template-banner"),i=document.getElementById("modal-srv-name"),a=document.getElementById("modal-srv-transport"),r=document.getElementById("modal-srv-command"),o=document.getElementById("modal-srv-url"),n=document.getElementById("modal-srv-ft"),d=document.getElementById("modal-srv-cd"),l=document.getElementById("modal-srv-autorestart"),g=document.getElementById("modal-srv-maxrestarts");if(e)e.textContent="Add Upstream MCP Server";if(t)t.style.display="flex";if(i)i.value="",i.disabled=!1;if(a)a.value="stdio";if(r)r.value="";if(o)o.value="";let p=document.getElementById("modal-group-cmd"),v=document.getElementById("modal-group-url");if(p)p.style.display="block";if(v)v.style.display="none";if(n)n.value="3";if(d)d.value="30000";if(l)l.value="true";if(g)g.value="5";let x=document.getElementById("modal-add-server");if(x)x.classList.add("active")}openEditServerModal(e){this.closeModals();let t=s.getState(),i=t.config.mcpServers?.[e];if(!i){alert(`Server '${e}' not found in configuration.`);return}let a=document.getElementById("modal-srv-title"),r=document.getElementById("modal-srv-template-banner"),o=document.getElementById("modal-srv-name"),n=document.getElementById("modal-srv-transport"),d=document.getElementById("modal-srv-command"),l=document.getElementById("modal-srv-url"),g=document.getElementById("modal-srv-ft"),p=document.getElementById("modal-srv-cd"),v=document.getElementById("modal-srv-autorestart"),x=document.getElementById("modal-srv-maxrestarts");if(a)a.textContent=`Edit Server '${e}'`;if(r)r.style.display="none";if(o)o.value=e,o.disabled=!0;let w=!!i.command;if(n)n.value=w?"stdio":"http";let h=document.getElementById("modal-group-cmd"),k=document.getElementById("modal-group-url");if(h)h.style.display=w?"block":"none";if(k)k.style.display=w?"none":"block";if(d)d.value=w?`${i.command} ${(i.args||[]).join(" ")}`.trim():"";if(l)l.value=i.url||"";let $=i.resilience||t.config.resilience;if(g)g.value=String($?.failureThreshold??3);if(p)p.value=String($?.cooldownMs??30000);if(v)v.value=$?.autoRestart===!1?"false":"true";if(x)x.value=String($?.maxRestarts??5);let f=document.getElementById("modal-add-server");if(f)f.classList.add("active")}async submitAddServer(){let e=document.getElementById("modal-srv-name")?.value.trim(),t=document.getElementById("modal-srv-transport")?.value;if(!e){alert("Server name is required");return}let i={};if(t==="stdio"){let g=(document.getElementById("modal-srv-command")?.value.trim()).split(/\s+/).filter(Boolean);if(g.length===0){alert("Command is required");return}i.command=g[0],i.args=g.slice(1)}else{let l=document.getElementById("modal-srv-url")?.value.trim();if(!l){alert("URL is required");return}i.url=l}let a=document.getElementById("modal-srv-ft")?.value.trim(),r=document.getElementById("modal-srv-cd")?.value.trim(),o=document.getElementById("modal-srv-autorestart")?.value,n=document.getElementById("modal-srv-maxrestarts")?.value.trim();if(a||r||o||n)i.resilience={failureThreshold:a?Number(a):3,cooldownMs:r?Number(r):30000,autoRestart:o!=="false",maxRestarts:n?Number(n):5};let d=await u.upsertServer(e,i);if(d.ok)this.closeModals(),await this.refreshData();else alert(`Failed to save server: ${d.error}`)}openTemplateCatalog(){this.closeModals();let e=document.getElementById("modal-templates");if(e)e.classList.add("active");this.renderTemplateGrid()}setTemplateCategory(e){this.activeTemplateCategory=e,document.querySelectorAll(".tmpl-cat-btn").forEach((t)=>{if(t.getAttribute("data-category")===e)t.classList.add("active"),t.style.background="var(--surface-elevated)",t.style.color="var(--amber-400)";else t.classList.remove("active"),t.style.background="var(--surface-card)",t.style.color="var(--text-main)"}),this.renderTemplateGrid()}filterTemplates(e){this.activeTemplateFilter=e.toLowerCase().trim(),this.renderTemplateGrid()}renderTemplateGrid(){let e=document.getElementById("tmpl-grid");if(!e)return;let t=N.filter((r)=>{let o=this.activeTemplateCategory==="all"||r.category===this.activeTemplateCategory,n=!this.activeTemplateFilter||r.name.toLowerCase().includes(this.activeTemplateFilter)||r.id.toLowerCase().includes(this.activeTemplateFilter)||r.description.toLowerCase().includes(this.activeTemplateFilter)||r.command.toLowerCase().includes(this.activeTemplateFilter)||r.envFields.some((d)=>d.key.toLowerCase().includes(this.activeTemplateFilter));return o&&n});if(t.length===0){e.innerHTML=`
        <div style="grid-column: span 2; padding: 32px; text-align: center; color: var(--text-dim);">
          No matching MCP server templates found.
        </div>
      `;return}let a=s.getState().config.mcpServers||{};e.innerHTML=t.map((r)=>{let o=!!a[r.id],n=`${r.command} ${r.defaultArgs.join(" ")}`;return`
        <div class="bento-card" style="display: flex; flex-direction: column; justify-content: space-between; padding: 14px; background: var(--surface); border: 1px solid var(--border); transition: transform 0.15s, border-color 0.15s;">
          <div>
            <div style="display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 6px;">
              <div style="display: flex; align-items: center; gap: 8px;">
                <span style="font-weight: 700; font-size: 13.5px; color: var(--text-main);">${c(r.name)}</span>
                <span class="brand-badge" style="font-size: 9.5px; padding: 1px 6px;">${c(r.badge)}</span>
              </div>
              ${o?'<span style="font-size: 10px; color: var(--green-400); font-weight: 600;">CONNECTED</span>':""}
            </div>
            <div style="font-size: 11.5px; color: var(--text-muted); line-height: 1.4; margin-bottom: 8px;">
              ${c(r.description)}
            </div>
            <div style="font-family: var(--ff-mono); font-size: 10.5px; color: var(--text-dim); background: var(--surface-card); padding: 5px 8px; border-radius: var(--radius-sm); border: 1px solid var(--border-subtle); overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">
              <code>${c(n)}</code>
            </div>
            ${r.envFields.length>0?`
              <div style="font-size: 10.5px; color: var(--amber-400); margin-top: 6px; display: flex; align-items: center; gap: 4px;">
                <span>⚡ Needs:</span>
                <code>${r.envFields.map((d)=>c(d.key)).join(", ")}</code>
              </div>
            `:""}
          </div>

          <div style="display: flex; justify-content: flex-end; margin-top: 12px; gap: 6px;">
            <button class="btn btn-primary" style="font-size: 11.5px; padding: 4px 10px;" onclick="window.app.selectTemplate('${c(r.id)}')">
              ${o?"Configure Another":"✨ 1-Click Setup"}
            </button>
          </div>
        </div>
      `}).join("")}selectTemplate(e){let t=N.find((n)=>n.id===e);if(!t)return;this.selectedTemplate=t,this.closeModals();let i=document.getElementById("modal-configure-template");if(i)i.classList.add("active");let a=document.getElementById("cfg-tmpl-title"),r=document.getElementById("cfg-tmpl-desc"),o=document.getElementById("cfg-tmpl-form");if(a)a.textContent=`Configure ${t.name} Server`;if(r)r.textContent=t.description;if(o){let n="";if(t.envFields.length>0)n=`
          <div style="margin-top: 14px; margin-bottom: 6px; font-weight: 700; font-size: 11px; text-transform: uppercase; color: var(--amber-400); letter-spacing: 0.5px;">
            Environment Variables &amp; API Keys
          </div>
          ${t.envFields.map((d)=>`
            <div class="form-group">
              <label class="form-label">${c(d.label)} ${d.required?'<span style="color: var(--red-400);">*</span>':"(Optional)"}</label>
              <input type="password" class="form-input tmpl-env-input" data-key="${c(d.key)}" placeholder="${c(d.placeholder||"")}">
              ${d.description?`<div style="font-size: 10.5px; color: var(--text-dim); margin-top: 3px;">${c(d.description)}</div>`:""}
            </div>
          `).join("")}
        `;o.innerHTML=`
        <div class="form-group">
          <label class="form-label">Server Identifier (Name)</label>
          <input type="text" class="form-input" id="cfg-srv-id" value="${c(t.id)}">
        </div>
        <div class="form-group">
          <label class="form-label">Command Line Arguments</label>
          <input type="text" class="form-input" id="cfg-srv-args" value="${c(t.defaultArgs.join(" "))}" placeholder="${c(t.argsPlaceholder||"")}">
          <div style="font-size: 10.5px; color: var(--text-dim); margin-top: 3px;">Executable: <code>${c(t.command)}</code></div>
        </div>
        ${n}
        <details style="margin-top: 14px; background: rgba(0,0,0,0.2); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 8px 12px;">
          <summary style="font-size: 11.5px; font-weight: 600; color: var(--amber-400); cursor: pointer;">
            \uD83D\uDEE1️ Fault Tolerance &amp; Process Supervision (Optional)
          </summary>
          <div style="margin-top: 10px; display: grid; grid-template-columns: 1fr 1fr; gap: 8px;">
            <div>
              <label class="form-label" style="font-size: 10.5px;">Failure Threshold</label>
              <input type="number" class="form-input" id="cfg-srv-ft" placeholder="3" value="3">
            </div>
            <div>
              <label class="form-label" style="font-size: 10.5px;">Cooldown (ms)</label>
              <input type="number" class="form-input" id="cfg-srv-cd" placeholder="30000" value="30000">
            </div>
            <div>
              <label class="form-label" style="font-size: 10.5px;">Auto-Restart</label>
              <select class="form-input" id="cfg-srv-autorestart">
                <option value="true">Enabled (Default)</option>
                <option value="false">Disabled</option>
              </select>
            </div>
            <div>
              <label class="form-label" style="font-size: 10.5px;">Max Restarts</label>
              <input type="number" class="form-input" id="cfg-srv-maxrestarts" placeholder="5" value="5">
            </div>
          </div>
        </details>
      `}}async submitTemplateServer(){if(!this.selectedTemplate)return;let e=this.selectedTemplate,t=document.getElementById("cfg-srv-id")?.value.trim(),i=document.getElementById("cfg-srv-args")?.value.trim();if(!t){alert("Server identifier is required");return}let a=i?i.split(/\s+/).filter(Boolean):[],r={},o=document.querySelectorAll(".tmpl-env-input");for(let x of Array.from(o)){let w=x.getAttribute("data-key"),h=x.value.trim(),k=e.envFields.find(($)=>$.key===w);if(k?.required&&!h){alert(`Required field '${k.label}' is missing.`);return}if(w&&h)r[w]=h}let n={command:e.command,args:a};if(Object.keys(r).length>0)n.env=r;let d=document.getElementById("cfg-srv-ft")?.value.trim(),l=document.getElementById("cfg-srv-cd")?.value.trim(),g=document.getElementById("cfg-srv-autorestart")?.value,p=document.getElementById("cfg-srv-maxrestarts")?.value.trim();if(d||l||g||p)n.resilience={failureThreshold:d?Number(d):3,cooldownMs:l?Number(l):30000,autoRestart:g!=="false",maxRestarts:p?Number(p):5};let v=await u.upsertServer(t,n);if(v.ok)this.closeModals(),await this.refreshData();else alert(`Failed to save server: ${v.error}`)}async openImportModal(){this.closeModals();let e=document.getElementById("modal-import");if(e)e.classList.add("active");let t=document.getElementById("modal-eco-list");if(!t)return;t.innerHTML='<div style="color: var(--text-dim); padding: 12px; text-align: center;">Scanning IDE configs...</div>';try{let i=await u.getEcosystemSources();if(i.sources&&i.sources.length>0)t.innerHTML=i.sources.map((a)=>`
          <label style="display: flex; align-items: center; gap: 10px; background: var(--surface); padding: 10px 14px; border-radius: var(--radius-sm); border: 1px solid var(--border); cursor: pointer;">
            <input type="checkbox" class="eco-checkbox" value="${a.path}" checked>
            <div>
              <div style="font-weight: 600; color: var(--text-main);">${a.name}</div>
              <div style="font-size: 11px; color: var(--text-dim);">${a.server_count} servers (${a.servers.join(", ")})</div>
            </div>
          </label>
        `).join("");else t.innerHTML='<div style="color: var(--text-dim); padding: 12px; text-align: center;">No external MCP configuration files found on this system.</div>'}catch{t.innerHTML='<div style="color: var(--red-400); padding: 12px; text-align: center;">Failed to scan ecosystem sources.</div>'}}async submitImport(){let e=document.querySelectorAll(".eco-checkbox:checked");if(e.length===0){alert("No sources selected");return}for(let t of Array.from(e))await u.importConfig(t.value,!1);this.closeModals(),await this.refreshData()}handleAliasTargetInput(e){let t=document.getElementById("alias-suggestions-dropdown");if(!t)return;let i=(e||"").trim().toLowerCase();if(i.length<2){t.style.display="none";return}let r=s.getState().capabilities.filter((o)=>o.id.toLowerCase().includes(i)||o.summary&&o.summary.toLowerCase().includes(i)||o.description&&o.description.toLowerCase().includes(i)||o.server&&o.server.toLowerCase().includes(i)).slice(0,8);if(r.length===0){t.style.display="none";return}t.innerHTML=r.map((o)=>`
      <div style="padding: 8px 12px; cursor: pointer; border-bottom: 1px solid var(--border-subtle); display: flex; justify-content: space-between; align-items: center; transition: background 0.1s;"
           onmouseover="this.style.background='var(--surface-hover)'"
           onmouseout="this.style.background='transparent'"
           onmousedown="window.app.selectAliasSuggestion('${c(o.id)}')">
        <div>
          <div style="font-weight: 700; color: var(--text-main);">${c(o.id)}</div>
          <div style="font-size: 10.5px; color: var(--text-dim); margin-top: 2px;">${c(o.summary||o.description||"")}</div>
        </div>
        <span style="font-size: 10px; color: var(--cyan-400);">${c(o.server||"local")}</span>
      </div>
    `).join(""),t.style.display="block"}selectAliasSuggestion(e){let t=document.getElementById("alias-target");if(t)t.value=e;this.hideAliasDropdown()}hideAliasDropdown(){let e=document.getElementById("alias-suggestions-dropdown");if(e)e.style.display="none"}async createAlias(){let e=document.getElementById("alias-kind")?.value,t=document.getElementById("alias-name")?.value.trim(),i=document.getElementById("alias-target")?.value.trim();if(!t||!i){alert("Please provide both alias name and canonical target");return}await u.updateAlias(e,t,i),await this.refreshData()}async deleteAlias(e,t){await u.updateAlias(e,t,void 0),await this.refreshData()}async reloadFromDisk(){try{let e=await u.reloadConfig();if(e.ok){let t="Hot-reload completed successfully!";if(e.mounted&&e.mounted.length>0)t+=`
Mounted: ${e.mounted.join(", ")}`;if(e.unmounted&&e.unmounted.length>0)t+=`
Unmounted: ${e.unmounted.join(", ")}`;if(e.warnings&&e.warnings.length>0)t+=`
Warnings:
${e.warnings.join(`
`)}`;alert(t)}else alert(`Hot-reload failed: ${e.error||"Unknown error"}`)}catch(e){alert(`Error reaching daemon: ${e.message}`)}await this.refreshData()}closeModals(){document.querySelectorAll(".modal-backdrop").forEach((e)=>e.classList.remove("active"))}}function c(e){return String(e||"").replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;")}var G=new B;window.app=G;window.addEventListener("DOMContentLoaded",()=>G.init());
