class N{state={configPath:"mcp_servers.json",config:{mcpServers:{}},serverStatuses:{},circuitBreakers:[],capabilities:[],resources:[],prompts:[],catalogEvents:[],approvals:[],auditEvents:[],auditTotal:0,auditFilters:{search:"",status:"all",eventType:"all",serverId:"all",limit:25,offset:0},auditSelectedEvent:null,auditStats:null,auditVerification:null,selectedCapabilityId:null,selectedResourceId:null,selectedPromptId:null,playgroundMode:"tools",isExecuting:!1,activeRequestId:null,isBatchModalOpen:!1,batchSteps:[{id:"step_1",capability_id:"",argsJson:"{}",continue_on_error:!1},{id:"step_2",capability_id:"",argsJson:"{}",continue_on_error:!0}],activeTab:"overview",eventLogs:[],executionResult:null,resourceReadResult:null,promptGetResult:null,metrics:{totalCatalogRequests:0,totalEtagHits:0,totalToolCalls:0,totalToolDurationUs:0}};listeners=[];getState(){return this.state}setState(e){this.state={...this.state,...e},this.listeners.forEach((t)=>t(this.state))}subscribe(e){return this.listeners.push(e),()=>{this.listeners=this.listeners.filter((t)=>t!==e)}}addEventLog(e,t,i,r){let n=[{time:new Date().toLocaleTimeString(),method:e,target:t,status:i,latency:r},...this.state.eventLogs].slice(0,50);this.setState({eventLogs:n})}}var s=new N;class q{baseUrl;constructor(e=""){this.baseUrl=e}async getConfig(){return(await fetch(`${this.baseUrl}/v1/config`)).json()}async listCapabilities(){return(await fetch(`${this.baseUrl}/v1/capabilities`)).json()}async listResources(){return(await fetch(`${this.baseUrl}/v1/resources`)).json()}async readResource(e){let t=performance.now(),i=await fetch(`${this.baseUrl}/v1/resources/read`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify(e)}),r=performance.now()-t,o=await i.json();return{status:i.status,durationMs:r,data:o}}async listPrompts(){return(await fetch(`${this.baseUrl}/v1/prompts`)).json()}async getPrompt(e){let t=performance.now(),i=await fetch(`${this.baseUrl}/v1/prompts/get`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify(e)}),r=performance.now()-t,o=await i.json();return{status:i.status,durationMs:r,data:o}}async getCatalogEvents(e){let t=e?`?after=${encodeURIComponent(e)}`:"";return(await fetch(`${this.baseUrl}/v1/catalog/events${t}`)).json()}async callCapability(e){let t=performance.now(),i=await fetch(`${this.baseUrl}/v1/tools/call`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify(e)}),r=performance.now()-t,o=await i.json();return{status:i.status,durationMs:r,data:o}}async batchCallCapabilities(e){let t=performance.now(),i=await fetch(`${this.baseUrl}/v1/tools/batch_call`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({steps:e})}),r=performance.now()-t,o=await i.json();return{status:i.status,durationMs:r,data:o}}async cancelOperation(e){return(await fetch(`${this.baseUrl}/v1/operations/${encodeURIComponent(e)}/cancel`,{method:"POST"})).json()}async completeArgument(e){return(await fetch(`${this.baseUrl}/v1/completion/complete`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify(e)})).json()}async upsertServer(e,t){return(await fetch(`${this.baseUrl}/v1/config/servers`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({name:e,server:t})})).json()}async deleteServer(e){return(await fetch(`${this.baseUrl}/v1/config/servers/${encodeURIComponent(e)}`,{method:"DELETE"})).json()}async getEcosystemSources(){return(await fetch(`${this.baseUrl}/v1/config/ecosystem`)).json()}async importConfig(e,t=!1){return(await fetch(`${this.baseUrl}/v1/config/import`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({source_path:e,overwrite:t})})).json()}async savePolicy(e){let t={allow:e.allow||[],deny:e.deny||[],redactKeys:e.redact_keys||e.redactKeys||[],requireApproval:e.require_approval||e.requireApproval||[],approvalTimeoutSecs:e.approvalTimeoutSecs||e.approval_timeout_secs||300,webhook:e.webhook};return(await fetch(`${this.baseUrl}/v1/config/policy`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify(t)})).json()}async listApprovals(){return(await fetch(`${this.baseUrl}/v1/approvals`)).json()}async approveTicket(e,t,i){return(await fetch(`${this.baseUrl}/v1/approvals/${encodeURIComponent(e)}/approve`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({operator:t,modified_args:i})})).json()}async rejectTicket(e,t,i){return(await fetch(`${this.baseUrl}/v1/approvals/${encodeURIComponent(e)}/reject`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({operator:t,reason:i})})).json()}async updateAlias(e,t,i){return(await fetch(`${this.baseUrl}/v1/config/alias`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({kind:e,alias:t,target:i})})).json()}async reloadConfig(){return(await fetch(`${this.baseUrl}/v1/config/reload`,{method:"POST"})).json()}async listAuditEvents(e){let t=new URLSearchParams;if(e?.actor_id)t.set("actor_id",e.actor_id);if(e?.server_id&&e.server_id!=="all")t.set("server_id",e.server_id);if(e?.capability_id)t.set("capability_id",e.capability_id);if(e?.event_type&&e.event_type!=="all")t.set("event_type",e.event_type);if(e?.status&&e.status!=="all")t.set("status",e.status);if(e?.trace_id)t.set("trace_id",e.trace_id);if(e?.request_id)t.set("request_id",e.request_id);if(e?.search)t.set("search",e.search);if(e?.limit)t.set("limit",String(e.limit));if(e?.offset!==void 0)t.set("offset",String(e.offset));let i=t.toString();return(await fetch(`${this.baseUrl}/v1/audit/events${i?`?${i}`:""}`)).json()}getAuditExportUrl(e,t="csv"){let i=new URLSearchParams;if(i.set("format",t),e?.actor_id)i.set("actor_id",e.actor_id);if(e?.server_id&&e.server_id!=="all")i.set("server_id",e.server_id);if(e?.capability_id)i.set("capability_id",e.capability_id);if(e?.event_type&&e.event_type!=="all")i.set("event_type",e.event_type);if(e?.status&&e.status!=="all")i.set("status",e.status);if(e?.trace_id)i.set("trace_id",e.trace_id);if(e?.request_id)i.set("request_id",e.request_id);if(e?.search)i.set("search",e.search);return`${this.baseUrl}/v1/audit/export?${i.toString()}`}async verifyAuditChain(){return(await fetch(`${this.baseUrl}/v1/audit/verify`)).json()}async getAuditStats(){return(await fetch(`${this.baseUrl}/v1/audit/stats`)).json()}}var u=new q;function D(){let e=s.getState(),t=e.config.mcpServers||{},i=Object.keys(t),r=i.length,o="";if(i.length===0)o=`
      <div style="grid-column: 1 / -1; padding: 32px; text-align: center; color: var(--text-dim); background: var(--surface-card); border-radius: var(--radius-md); border: 1px dashed var(--border);">
        <div style="font-size: 14px; color: var(--text-main); font-weight: 600; margin-bottom: 6px;">No Upstream MCP Servers Connected</div>
        <div style="font-size: 12px; margin-bottom: 16px;">Initialize connections by adding a server or syncing existing IDE configurations.</div>
        <div style="display: flex; gap: 8px; justify-content: center;">
          <button class="btn btn-primary" onclick="window.app.openTemplateCatalog()">✨ Browse Templates</button>
          <button class="btn btn-ghost" onclick="window.app.openAddServerModal()">+ Add Custom</button>
          <button class="btn btn-ghost" onclick="window.app.openImportModal()">Sync from IDEs</button>
        </div>
      </div>
    `;else o=i.map((x)=>{let z=t[x],O=z.command?"stdio":"http / sse",R=z.command?`${z.command} ${(z.args||[]).join(" ")}`:z.url,P=e.serverStatuses[x]||{status:"connected",protocol_version:"2026-07-28"},y=P.status==="degraded",L=P.status==="error"||P.status==="disconnected",T=y?"var(--amber-400)":L?"var(--red-400)":"var(--green-400)";return`
        <div class="bento-card col-4" style="background: var(--surface); border: 1px solid var(--border);">
          <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
            <span style="font-weight: 700; color: var(--text-main); display: flex; align-items: center; gap: 6px;">
              <span style="width: 8px; height: 8px; border-radius: 50%; background: ${T}; display: inline-block;"></span>
              ${A(x)}
            </span>
            <span class="brand-badge">${O}</span>
          </div>
          <div style="font-family: var(--ff-mono); font-size: 11.5px; color: var(--text-dim); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; margin-bottom: 12px;" title="${A(R||"")}">
            ${A(R||"")}
          </div>
          <div style="display: flex; justify-content: space-between; font-size: 11px; color: var(--text-muted); border-top: 1px solid var(--border); padding-top: 8px;">
            <span>Status: <strong style="color: ${T};">${A(P.status)}</strong></span>
            <span>Protocol: ${P.protocol_version}</span>
          </div>
        </div>
      `}).join("");let n=e.eventLogs.length===0?`
    <div class="feed-row" style="grid-template-columns: 80px 100px 1fr 100px 80px;">
      <span style="color: var(--text-dim);">ready</span>
      <span style="color: var(--cyan-400); font-weight: 600;">SSE</span>
      <span style="color: var(--text-main);">/v1/resources/updates stream active</span>
      <span style="color: var(--green-400);">CONNECTED</span>
      <span style="color: var(--amber-300); text-align: right;">0.0ms</span>
    </div>
  `:e.eventLogs.map((x)=>`
    <div class="feed-row" style="grid-template-columns: 80px 100px 1fr 100px 80px;">
      <span style="color: var(--text-dim);">${A(x.time)}</span>
      <span style="color: var(--cyan-400); font-weight: 600;">${A(x.method)}</span>
      <span style="color: var(--text-main); font-family: var(--ff-mono);">${A(x.target)}</span>
      <span style="color: var(--green-400);">${A(x.status)}</span>
      <span style="color: var(--amber-300); text-align: right;">${A(x.latency)}</span>
    </div>
  `).join(""),a=e.metrics,d=a.totalCatalogRequests,p=a.totalEtagHits,l=d>0?`${(p/d*100).toFixed(1)}%`:"0.0%",g=d>0?`${p} of ${d} requests served via HTTP 304`:"Waiting for client requests",b=a.totalToolCalls,f=b>0?`${(a.totalToolDurationUs/b/1000).toFixed(1)}ms`:"0.0ms",_=b>0?`${b} tool executions processed`:"Local worker task queues warm",h=Object.keys(e.config.capabilityAliases||{}).length+Object.keys(e.config.resourceAliases||{}).length+Object.keys(e.config.promptAliases||{}).length,k=h>0?`${h*18}B / call`:"0B",$=h>0?`${h} active facade aliases pruning prompt size`:"Configure aliases in Studio to reduce prompt size";return`
    <div class="bento-grid">
      <div class="bento-card col-3">
        <div class="stat-label">Token Savings Rate</div>
        <div class="stat-value" style="color: var(--amber-300);">${k}</div>
        <div class="stat-sub">${$}</div>
      </div>
      <div class="bento-card col-3">
        <div class="stat-label">ETag Cache Hit Rate</div>
        <div class="stat-value" style="color: var(--cyan-400);">${l}</div>
        <div class="stat-sub">${g}</div>
      </div>
      <div class="bento-card col-3">
        <div class="stat-label">Connected Upstreams</div>
        <div class="stat-value" style="color: var(--green-400);">${r} Active</div>
        <div class="stat-sub">${r>0?"Persistent worker task channels":"No active upstream servers"}</div>
      </div>
      <div class="bento-card col-3">
        <div class="stat-label">Avg Execution Latency</div>
        <div class="stat-value">${f}</div>
        <div class="stat-sub">${_}</div>
      </div>
    </div>

    <div style="display: flex; justify-content: space-between; align-items: center; margin: 24px 0 12px;">
      <div style="font-size: 15px; font-weight: 700; color: var(--text-main);">Connected Upstream Servers</div>
      <button class="btn btn-ghost" onclick="window.app.switchTab('servers')">Manage All (${r}) →</button>
    </div>

    <div class="bento-grid" style="margin-bottom: 24px;">
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
        ${n}
      </div>
    </div>
  `}function A(e){return String(e||"").replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;")}function E(){let e=s.getState(),t=e.config.mcpServers||{},i=Object.keys(t),r="";if(i.length===0)r=`
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
    `;else r=i.map((o)=>{let n=t[o],a=n.command?"stdio":"http / sse",d=n.command?`${n.command} ${(n.args||[]).join(" ")}`:n.url,p=e.serverStatuses[o]||{status:"connected",protocol_version:"2026-07-28"},l=n.env?Object.keys(n.env).map((x)=>`${x}=***`).join(", "):"None",g=(e.circuitBreakers||[]).find((x)=>x.server_id===o),b='<span class="brand-badge" style="color: var(--green-400); border-color: rgba(52, 211, 153, 0.25);">Circuit: CLOSED</span>';if(g){if(g.state==="open")b=`<span class="brand-badge" style="color: var(--red-400); border-color: rgba(248, 113, 113, 0.4); background: rgba(248, 113, 113, 0.1);">Circuit: OPEN (${g.consecutive_failures} failures)</span>`;else if(g.state==="half_open")b=`<span class="brand-badge" style="color: var(--amber-300); border-color: rgba(251, 191, 36, 0.4); background: rgba(251, 191, 36, 0.1);">Circuit: HALF-OPEN (${g.consecutive_successes} probe)</span>`}let f=n.resilience||e.config.resilience,_=f?`FT: ${f.failureThreshold||3} · Cooldown: ${(f.cooldownMs||30000)/1000}s · AutoRestart: ${f.autoRestart!==!1?"ON":"OFF"}`:"Default Resilience",h=p.status==="degraded",k=p.status==="error"||p.status==="disconnected",$=h?"var(--amber-400)":k?"var(--red-400)":"var(--green-400)";return`
        <div class="bento-card" style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px;">
          <div>
            <div style="display: flex; align-items: center; gap: 8px; margin-bottom: 4px;">
              <span style="width: 8px; height: 8px; border-radius: 50%; background: ${$}; display: inline-block;"></span>
              <span style="font-size: 15px; font-weight: 700; color: var(--text-main);">${C(o)}</span>
              <span class="brand-badge">${a}</span>
              <span class="brand-badge" style="color: ${$}; border-color: rgba(245, 158, 11, 0.3);">Status: ${C(p.status)}</span>
              <span class="brand-badge" style="color: var(--cyan-400); border-color: rgba(34, 211, 238, 0.25);">Protocol: ${p.protocol_version}</span>
              ${b}
            </div>
            <div style="font-family: var(--ff-mono); font-size: 12px; color: var(--text-muted); margin-top: 4px;">
              ${n.command?"Command: ":"URL: "}<code>${C(d||"")}</code>
            </div>
            <div style="display: flex; gap: 14px; font-family: var(--ff-mono); font-size: 11px; color: var(--text-dim); margin-top: 4px;">
              <span>\uD83D\uDEE1️ ${C(_)}</span>
              ${n.env&&Object.keys(n.env).length>0?`<span>Env: ${C(l)}</span>`:""}
            </div>
          </div>
          <div style="display: flex; gap: 8px;">
            <button class="btn btn-ghost" style="padding: 4px 10px; font-size: 11.5px;" onclick="window.app.openEditServerModal('${C(o)}')">✏️ Edit</button>
            <button class="btn btn-danger" style="padding: 4px 10px; font-size: 11.5px;" onclick="window.app.deleteServer('${C(o)}')">Remove</button>
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

    ${r}
  `}function C(e){return String(e||"").replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;")}function U(){let e=s.getState(),t=e.playgroundMode||"tools",i=e.capabilities||[],r=e.resources||[],o=e.prompts||[],n=`
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
          \uD83D\uDCC4 Resources (${r.length})
        </button>
        <button 
          class="btn ${t==="prompts"?"btn-primary":"btn-ghost"}" 
          style="padding: 4px 12px; font-size: 11.5px; height: 28px;"
          onclick="window.app.setPlaygroundMode('prompts')"
        >
          \uD83D\uDCAC Prompts (${o.length})
        </button>
      </div>

      <div style="font-size: 11.5px; color: var(--text-dim);">
        ${t==="tools"?"Interactive Tool Caller & Context Distillation":t==="resources"?"Live MCP Resource Inspector & Reader":"Prompt Template Studio & Variable Binder"}
      </div>
    </div>
  `;if(t==="resources")return`
      ${n}
      ${H(e)}
    `;if(t==="prompts")return`
      ${n}
      ${ee(e)}
    `;return`
    ${n}
    ${V(e)}
    ${e.isBatchModalOpen?te(e):""}
  `}function V(e){let t=e.capabilities||[],i=e.selectedCapabilityId||(t.length>0?t[0].id:null),r=t.find((d)=>d.id===i),o=!!e.isExecuting,n="";if(t.length===0)n=`
      <div style="padding: 24px 16px; text-align: center; color: var(--text-dim); font-size: 11.5px;">
        No tools or capabilities discovered from connected servers.
      </div>
    `;else n=t.map((d)=>{return`
        <div class="cap-item ${d.id===i?"active":""}" onclick="window.app.selectCapability('${c(d.id)}')">
          <div style="display: flex; justify-content: space-between; align-items: center;">
            <span style="font-weight: 600; color: var(--text-main); font-family: var(--ff-mono); font-size: 12px;">${c(d.id)}</span>
            <span style="font-size: 10px; color: var(--green-400);">${c(d.mode||"read")}</span>
          </div>
          <div style="font-size: 11px; color: var(--text-dim); margin-top: 2px;">server: ${c(d.server||"local")}</div>
        </div>
      `}).join("");let a=r&&r.input_schema?JSON.stringify(r.input_schema.properties||{},null,2):"{}";return`
    <div style="display: grid; grid-template-columns: 320px 1fr; gap: 16px; height: calc(100vh - 165px);">
      <!-- Left Sidebar: Capabilities Catalog -->
      <div style="background: var(--surface-card); border: 1px solid var(--border); border-radius: var(--radius-md); display: flex; flex-direction: column; overflow: hidden;">
        <div style="padding: 12px; border-bottom: 1px solid var(--border);">
          <input type="text" class="form-input" placeholder="Search ${t.length} capabilities..." oninput="window.app.filterCapabilities(this.value)">
        </div>
        <div style="flex: 1; overflow-y: auto; padding: 8px;" id="pg-cap-list">
          ${n}
        </div>
      </div>

      <!-- Right Panel: Capability Execution & Envelope Visualizer -->
      <div style="background: var(--surface-card); border: 1px solid var(--border); border-radius: var(--radius-md); display: flex; flex-direction: column; overflow: hidden;">
        <div style="padding: 14px 18px; border-bottom: 1px solid var(--border); display: flex; justify-content: space-between; align-items: center;">
          <div>
            <div style="font-size: 15px; font-weight: 700; color: var(--text-main); font-family: var(--ff-mono);" id="pg-selected-title">
              ${c(r?r.id:"No Capability Selected")}
            </div>
            <div style="font-size: 11.5px; color: var(--text-dim);" id="pg-selected-desc">
              ${c(r?r.summary||r.description:"Connect servers to inspect and execute tools")}
            </div>
          </div>
          
          <div style="display: flex; align-items: center; gap: 10px;">
            ${o?`
              <div style="display: flex; gap: 8px; align-items: center;">
                <span class="badge" style="background: rgba(234, 179, 8, 0.15); color: var(--amber-400); font-family: var(--ff-mono); font-size: 11px; padding: 4px 8px; display: inline-flex; align-items: center; gap: 6px;">
                  <span style="width: 6px; height: 6px; border-radius: 50%; background: var(--amber-400); display: inline-block;"></span>
                  EXECUTING...
                </span>
                <button class="btn btn-danger" style="background: rgba(239, 68, 68, 0.2); color: var(--red-400); border: 1px solid rgba(239, 68, 68, 0.4); padding: 5px 12px; font-size: 11.5px;" onclick="window.app.cancelActiveOperation()">
                  <svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor" stroke="none"><rect x="4" y="4" width="16" height="16" rx="2"></rect></svg>
                  Cancel Operation
                </button>
              </div>
            `:`
              <button class="btn btn-primary" onclick="window.app.executePlaygroundTool()" ${r?"":"disabled"}>
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polygon points="5 3 19 12 5 21 5 3"></polygon></svg>
                Execute Capability
              </button>
            `}
          </div>
        </div>

        <div style="flex: 1; display: grid; grid-template-columns: 1fr 1fr; overflow: hidden;">
          <!-- Request Builder -->
          <div style="padding: 16px; border-right: 1px solid var(--border); overflow-y: auto;">
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 10px;">
              <label class="form-label" style="margin: 0;">Arguments JSON (Object)</label>
              <div style="display: flex; gap: 8px;">
                <button class="btn btn-ghost" style="padding: 2px 8px; font-size: 11px;" onclick="window.app.openBatchModal()">⚡ Visual Pipeline Builder</button>
              </div>
            </div>
            <textarea class="form-textarea" rows="7" id="pg-args-input">${c(a)}</textarea>

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
            ${r&&r.input_schema?`
              <div style="margin-top: 14px;">
                <label class="form-label">Input JSON Schema</label>
                <pre style="background: var(--surface); padding: 10px; border-radius: var(--radius-sm); border: 1px solid var(--border); font-size: 11px; color: var(--text-muted); max-height: 140px; overflow-y: auto;">${c(JSON.stringify(r.input_schema,null,2))}</pre>
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
            <pre id="pg-response-json" style="flex: 1; background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 14px; color: var(--amber-300); font-size: 11.5px; overflow-y: auto; margin: 0; white-space: pre-wrap; word-break: break-word;">${e.executionResult?c(JSON.stringify(e.executionResult.data,null,2)):"// Response envelope output will be formatted here"}</pre>
          </div>
        </div>
      </div>
    </div>
  `}function H(e){let t=e.resources||[],i=e.selectedResourceId||(t.length>0?t[0].id:null),r=t.find((a)=>a.id===i),o=e.resourceReadResult,n="";if(t.length===0)n=`
      <div style="padding: 24px 16px; text-align: center; color: var(--text-dim); font-size: 11.5px;">
        No resources exposed by connected MCP servers.
      </div>
    `;else n=t.map((a)=>{let d=a.id===i?"active":"",p=a.uri?a.uri.split(":")[0]:"res";return`
        <div class="cap-item ${d}" onclick="window.app.selectResource('${c(a.id)}')">
          <div style="display: flex; justify-content: space-between; align-items: center;">
            <span style="font-weight: 600; color: var(--text-main); font-family: var(--ff-mono); font-size: 12px;">${c(a.name||a.id)}</span>
            <span class="badge" style="font-size: 9.5px; background: rgba(56, 189, 248, 0.15); color: var(--cyan-400);">${c(p)}</span>
          </div>
          <div style="font-size: 11px; color: var(--text-dim); margin-top: 2px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">${c(a.uri)}</div>
          <div style="display: flex; justify-content: space-between; font-size: 10px; color: var(--text-muted); margin-top: 4px;">
            <span>server: ${c(a.server||"local")}</span>
            <span>${c(a.mime_type||"text/plain")}</span>
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
          ${n}
        </div>
      </div>

      <!-- Right Panel: Resource Content Reader & Metadata Inspector -->
      <div style="background: var(--surface-card); border: 1px solid var(--border); border-radius: var(--radius-md); display: flex; flex-direction: column; overflow: hidden;">
        <div style="padding: 14px 18px; border-bottom: 1px solid var(--border); display: flex; justify-content: space-between; align-items: center;">
          <div>
            <div style="font-size: 15px; font-weight: 700; color: var(--text-main); font-family: var(--ff-mono);">
              ${c(r?r.name||r.id:"No Resource Selected")}
            </div>
            <div style="font-size: 11.5px; color: var(--cyan-400); font-family: var(--ff-mono);">
              ${c(r?r.uri:"Select a resource from the list to read live content")}
            </div>
          </div>
          <button class="btn btn-primary" onclick="window.app.executeReadResource()" ${r?"":"disabled"}>
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M2 12h20M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"></path></svg>
            Read Resource Content
          </button>
        </div>

        <div style="flex: 1; display: grid; grid-template-columns: 1fr 1fr; overflow: hidden;">
          <!-- Request / Distillation Parameters -->
          <div style="padding: 16px; border-right: 1px solid var(--border); overflow-y: auto;">
            ${r?`
              <div style="background: rgba(0,0,0,0.25); padding: 12px; border-radius: var(--radius-sm); border: 1px solid var(--border); margin-bottom: 14px;">
                <div style="font-size: 11px; font-weight: 600; color: var(--text-muted); text-transform: uppercase; margin-bottom: 6px;">Resource Metadata</div>
                <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 8px; font-size: 11.5px;">
                  <div><span style="color: var(--text-muted);">Server:</span> <strong style="color: var(--text-main);">${c(r.server)}</strong></div>
                  <div><span style="color: var(--text-muted);">MIME Type:</span> <strong style="color: var(--text-main);">${c(r.mime_type||"text/plain")}</strong></div>
                </div>
                ${r.description?`
                  <div style="margin-top: 8px; font-size: 11.5px; color: var(--text-dim); border-top: 1px solid rgba(255,255,255,0.05); padding-top: 6px;">
                    ${c(r.description)}
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
              <span style="font-size: 11px; font-weight: 600; color: ${o?o.status===200?"var(--green-400)":"var(--red-400)":"var(--text-dim)"}; font-family: var(--ff-mono);">
                ${o?`HTTP ${o.status} · ${o.durationMs.toFixed(1)}ms`:"READY"}
              </span>
            </div>
            <pre style="flex: 1; background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 14px; color: var(--cyan-400); font-size: 11.5px; overflow-y: auto; margin: 0; white-space: pre-wrap; word-break: break-word;">${o?c(JSON.stringify(o.data,null,2)):'// Click "Read Resource Content" to inspect live payload'}</pre>
          </div>
        </div>
      </div>
    </div>
  `}function ee(e){let t=e.prompts||[],i=e.selectedPromptId||(t.length>0?t[0].id:null),r=t.find((d)=>d.id===i),o=e.promptGetResult,n="";if(t.length===0)n=`
      <div style="padding: 24px 16px; text-align: center; color: var(--text-dim); font-size: 11.5px;">
        No prompt templates registered by connected MCP servers.
      </div>
    `;else n=t.map((d)=>{let p=d.id===i?"active":"",l=d.arguments?d.arguments.length:0;return`
        <div class="cap-item ${p}" onclick="window.app.selectPrompt('${c(d.id)}')">
          <div style="display: flex; justify-content: space-between; align-items: center;">
            <span style="font-weight: 600; color: var(--text-main); font-family: var(--ff-mono); font-size: 12px;">${c(d.name||d.id)}</span>
            <span class="badge" style="font-size: 9.5px; background: rgba(168, 85, 247, 0.15); color: var(--purple-400);">${l} args</span>
          </div>
          <div style="font-size: 11px; color: var(--text-dim); margin-top: 2px;">${c(d.description||d.title||"Prompt template")}</div>
          <div style="font-size: 10px; color: var(--text-muted); margin-top: 4px;">server: ${c(d.server||"local")}</div>
        </div>
      `}).join("");let a="";if(r&&r.arguments&&r.arguments.length>0)a=r.arguments.map((d)=>`
      <div class="form-group" style="margin-bottom: 12px;">
        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 4px;">
          <label class="form-label" style="margin: 0; font-family: var(--ff-mono);">${c(d.name)}</label>
          ${d.required?'<span class="badge" style="background: rgba(239, 68, 68, 0.15); color: var(--red-400); font-size: 9px;">REQUIRED</span>':'<span style="font-size: 10px; color: var(--text-dim);">optional</span>'}
        </div>
        ${d.description?`<div style="font-size: 11px; color: var(--text-dim); margin-bottom: 4px;">${c(d.description)}</div>`:""}
        <input type="text" class="form-input prompt-arg-input" data-arg-name="${c(d.name)}" placeholder="Enter ${c(d.name)}..." />
      </div>
    `).join("");else if(r)a=`
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
          ${n}
        </div>
      </div>

      <!-- Right Panel: Prompt Parameter Binder & Message Envelope Preview -->
      <div style="background: var(--surface-card); border: 1px solid var(--border); border-radius: var(--radius-md); display: flex; flex-direction: column; overflow: hidden;">
        <div style="padding: 14px 18px; border-bottom: 1px solid var(--border); display: flex; justify-content: space-between; align-items: center;">
          <div>
            <div style="font-size: 15px; font-weight: 700; color: var(--text-main); font-family: var(--ff-mono);">
              ${c(r?r.name||r.id:"No Prompt Selected")}
            </div>
            <div style="font-size: 11.5px; color: var(--text-dim);">
              ${c(r?r.description||r.title||"Bind variables and render messages":"Select a prompt from the list to test")}
            </div>
          </div>
          <button class="btn btn-primary" onclick="window.app.executeGetPrompt()" ${r?"":"disabled"}>
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
            ${a}
          </div>

          <!-- Rendered Messages Output -->
          <div style="padding: 16px; background: var(--bg-app); display: flex; flex-direction: column; overflow: hidden;">
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
              <span style="font-size: 11px; font-weight: 600; color: var(--text-dim);">RENDERED PROMPT MESSAGES</span>
              <span style="font-size: 11px; font-weight: 600; color: ${o?o.status===200?"var(--green-400)":"var(--red-400)":"var(--text-dim)"}; font-family: var(--ff-mono);">
                ${o?`HTTP ${o.status} · ${o.durationMs.toFixed(1)}ms`:"READY"}
              </span>
            </div>
            <pre style="flex: 1; background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 14px; color: #c084fc; font-size: 11.5px; overflow-y: auto; margin: 0; white-space: pre-wrap; word-break: break-word;">${o?c(JSON.stringify(o.data,null,2)):'// Click "Render Prompt Messages" to view resolved system/user messages'}</pre>
          </div>
        </div>
      </div>
    </div>
  `}function te(e){let t=e.capabilities||[],i=e.batchSteps||[];return`
    <div style="position: fixed; inset: 0; background: rgba(0,0,0,0.75); backdrop-filter: blur(4px); display: flex; align-items: center; justify-content: center; z-index: 1000; padding: 24px;">
      <div style="background: var(--surface-card); border: 1px solid var(--border); border-radius: var(--radius-md); width: 840px; max-width: 95vw; max-height: 90vh; display: flex; flex-direction: column; overflow: hidden; box-shadow: 0 20px 40px rgba(0,0,0,0.5);">
        <div style="padding: 16px 20px; border-bottom: 1px solid var(--border); display: flex; justify-content: space-between; align-items: center;">
          <div>
            <div style="font-size: 16px; font-weight: 700; color: var(--text-main); display: flex; align-items: center; gap: 8px;">
              <span>⚡ Visual Multi-Step Batch Pipeline Builder</span>
            </div>
            <div style="font-size: 11.5px; color: var(--text-dim); margin-top: 2px;">
              Execute chained MCP tools with variable reference interpolation and fault tolerance.
            </div>
          </div>
          <button class="btn btn-ghost" style="font-size: 16px; padding: 4px 8px;" onclick="window.app.closeBatchModal()">✕</button>
        </div>

        <div style="flex: 1; overflow-y: auto; padding: 20px;">
          ${i.map((o,n)=>{let a=t.map((d)=>`
      <option value="${c(d.id)}" ${d.id===o.capability_id?"selected":""}>
        ${c(d.id)} (${c(d.server||"local")})
      </option>
    `).join("");return`
      <div style="background: rgba(0,0,0,0.3); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 14px; margin-bottom: 12px;">
        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 10px;">
          <div style="display: flex; align-items: center; gap: 8px;">
            <span class="badge" style="background: rgba(56, 189, 248, 0.15); color: var(--cyan-400); font-family: var(--ff-mono); font-weight: 700;">STEP ${n+1}</span>
            <span style="font-size: 11px; font-family: var(--ff-mono); color: var(--text-dim);">id: ${c(o.id)}</span>
          </div>
          <button class="btn btn-ghost" style="padding: 2px 8px; font-size: 11px; color: var(--red-400);" onclick="window.app.removeBatchStep(${n})">
            ✕ Remove
          </button>
        </div>

        <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 12px; margin-bottom: 10px;">
          <div class="form-group" style="margin: 0;">
            <label class="form-label" style="font-size: 11px;">Target Capability</label>
            <select class="form-input" style="font-size: 11.5px;" onchange="window.app.updateBatchStepCapability(${n}, this.value)">
              <option value="">-- Select Capability --</option>
              ${a}
            </select>
          </div>
          <div style="display: flex; align-items: flex-end; padding-bottom: 6px;">
            <label style="display: flex; align-items: center; gap: 6px; font-size: 11.5px; color: var(--text-muted); cursor: pointer;">
              <input type="checkbox" ${o.continue_on_error?"checked":""} onchange="window.app.updateBatchStepContinueOnError(${n}, this.checked)" />
              <span>Continue pipeline on step failure</span>
            </label>
          </div>
        </div>

        <div class="form-group" style="margin: 0;">
          <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 4px;">
            <label class="form-label" style="margin: 0; font-size: 11px;">Step Arguments JSON</label>
            <div style="display: flex; gap: 6px; font-size: 10px; color: var(--cyan-400); font-family: var(--ff-mono);">
              <span>Helpers:</span>
              <code style="cursor: pointer; background: rgba(0,0,0,0.3); padding: 1px 4px; border-radius: 2px;" onclick="window.app.appendBatchVariable(${n}, '\${steps[0].result.id}')">\${steps[0].result.id}</code>
              <code style="cursor: pointer; background: rgba(0,0,0,0.3); padding: 1px 4px; border-radius: 2px;" onclick="window.app.appendBatchVariable(${n}, '\${steps[0].result.data}')">\${steps[0].result.data}</code>
            </div>
          </div>
          <textarea 
            id="batch-step-args-${n}"
            class="form-textarea" 
            rows="3" 
            style="font-size: 11px; font-family: var(--ff-mono);" 
            oninput="window.app.updateBatchStepArgs(${n}, this.value)"
          >${c(o.argsJson)}</textarea>
        </div>
      </div>
    `}).join("")}

          <div style="display: flex; gap: 10px; margin-top: 14px;">
            <button class="btn btn-ghost" style="font-size: 11.5px;" onclick="window.app.addBatchStep()">
              + Add Pipeline Step
            </button>
          </div>
        </div>

        <div style="padding: 14px 20px; border-top: 1px solid var(--border); background: var(--bg-app); display: flex; justify-content: space-between; align-items: center;">
          <div style="font-size: 11.5px; color: var(--text-dim);">
            ${i.length} sequential execution steps configured
          </div>
          <div style="display: flex; gap: 10px;">
            <button class="btn btn-ghost" onclick="window.app.closeBatchModal()">Cancel</button>
            <button class="btn btn-primary" onclick="window.app.executeBatchPipeline()">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polygon points="5 3 19 12 5 21 5 3"></polygon></svg>
              Run Batch Pipeline (${i.length} Steps)
            </button>
          </div>
        </div>
      </div>
    </div>
  `}function c(e){return String(e||"").replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;")}function F(e){let t=e.approvals.filter((l)=>l.status==="pending"),i=e.approvals.filter((l)=>l.status!=="pending"),r=e.approvals.filter((l)=>l.status==="approved").length,o=e.approvals.filter((l)=>l.status==="rejected").length,n=e.config.policy?.require_approval||e.config.policy?.requireApproval||[],a=t.length===0?`
    <div style="padding: 48px 24px; text-align: center; background: var(--surface-card); border-radius: var(--radius-md); border: 1px dashed var(--border);">
      <div style="width: 48px; height: 48px; border-radius: 50%; background: rgba(52, 211, 153, 0.12); border: 1px solid rgba(52, 211, 153, 0.3); display: flex; align-items: center; justify-content: center; margin: 0 auto 14px; color: var(--green-400); font-size: 20px; font-weight: 700;">
        ✓
      </div>
      <div style="font-size: 15px; font-weight: 600; color: var(--text-main); margin-bottom: 6px;">All Clear — No Pending Approvals</div>
      <div style="font-size: 12px; color: var(--text-dim); max-width: 520px; margin: 0 auto; line-height: 1.6;">
        Tool invocations intercepted by <code style="color: var(--amber-300); font-family: var(--ff-mono);">require_approval</code> governance rules will suspend execution and appear here for operator inspection, argument modification, and cryptographic gating.
      </div>
    </div>
  `:t.map((l)=>`
    <div class="bento-card" style="border: 1px solid rgba(245, 158, 11, 0.35); background: var(--surface-card); margin-bottom: 14px; padding: 18px;">
      <div style="display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 12px;">
        <div>
          <div style="display: flex; align-items: center; gap: 8px;">
            <span class="brand-badge" style="background: rgba(245, 158, 11, 0.2); color: var(--amber-300); border-color: rgba(245, 158, 11, 0.5);">
              PENDING APPROVAL
            </span>
            <span style="font-family: var(--ff-mono); font-size: 12px; color: var(--text-muted);">${w(l.id)}</span>
          </div>
          <div style="margin-top: 6px; display: flex; align-items: center; gap: 8px;">
            <span style="font-size: 14.5px; font-weight: 700; color: var(--text-main); font-family: var(--ff-mono);">${w(l.capability_id)}</span>
            <span style="font-size: 11px; color: var(--text-dim);">via <span style="color: var(--cyan-400); font-family: var(--ff-mono);">${w(l.server_id)}</span></span>
          </div>
        </div>

        <div style="text-align: right; font-family: var(--ff-mono); font-size: 11px; color: var(--text-dim);">
          <div>Created: <span style="color: var(--text-muted);">${new Date(l.created_at*1000).toLocaleTimeString()}</span></div>
          <div style="color: var(--amber-400); margin-top: 2px;">Expires: ${new Date(l.expires_at*1000).toLocaleTimeString()}</div>
        </div>
      </div>

      <!-- Caller Context -->
      ${l.context||l.request_id?`
        <div style="background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 8px 12px; font-family: var(--ff-mono); font-size: 11px; display: flex; flex-wrap: wrap; gap: 16px; margin-bottom: 12px; color: var(--text-muted);">
          ${l.request_id?`<div><span style="color: var(--text-dim);">Request:</span> <span style="color: var(--text-main);">${w(l.request_id)}</span></div>`:""}
          ${l.context?.actor_id?`<div><span style="color: var(--text-dim);">Actor:</span> <span style="color: var(--cyan-400);">${w(l.context.actor_id)}</span></div>`:""}
          ${l.context?.operation_id?`<div><span style="color: var(--text-dim);">Operation:</span> <span style="color: var(--text-main);">${w(l.context.operation_id)}</span></div>`:""}
          ${l.context?.work_item_id?`<div><span style="color: var(--text-dim);">Work Item:</span> <span style="color: var(--text-main);">${w(l.context.work_item_id)}</span></div>`:""}
        </div>
      `:""}

      <!-- Arguments Editor -->
      <div style="margin-bottom: 14px;">
        <div style="font-size: 11px; font-weight: 600; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.5px; margin-bottom: 6px;">
          Parameters (Editable before approval)
        </div>
        <textarea id="appr-args-${l.id}" class="form-textarea" rows="4" style="color: var(--green-400); font-family: var(--ff-mono); font-size: 11.5px; line-height: 1.4;">${w(JSON.stringify(l.sanitized_args,null,2))}</textarea>
      </div>

      <!-- Action Footer -->
      <div style="display: flex; justify-content: space-between; align-items: center; border-top: 1px solid var(--border); padding-top: 12px;">
        <div style="display: flex; align-items: center; gap: 8px;">
          <input id="appr-operator-${l.id}" type="text" class="form-input" placeholder="Operator ID" value="security-operator" style="width: 200px; padding: 5px 10px; font-size: 11px;">
        </div>

        <div style="display: flex; align-items: center; gap: 8px;">
          <button class="btn btn-danger" onclick="window.app.promptReject('${w(l.id)}')">
            ✕ Reject
          </button>
          <button class="btn btn-primary" onclick="window.app.submitApproval('${w(l.id)}')">
            ✓ Approve &amp; Execute
          </button>
        </div>
      </div>
    </div>
  `).join(""),d=n.length===0?`
    <div style="color: var(--text-dim); font-size: 11.5px; line-height: 1.5; padding: 8px 0;">
      No explicit <code style="color: var(--amber-400);">require_approval</code> rules active. All non-denied capabilities execute immediately.
    </div>
  `:n.map((l)=>`
    <div style="display: flex; justify-content: space-between; align-items: center; background: var(--surface); padding: 8px 10px; border-radius: var(--radius-sm); border: 1px solid var(--border); margin-bottom: 6px;">
      <span style="font-family: var(--ff-mono); font-size: 11.5px; color: var(--amber-300); font-weight: 500;">\uD83D\uDEE1️ ${w(l)}</span>
      <span class="brand-badge" style="font-size: 9.5px; padding: 1px 5px;">GATED</span>
    </div>
  `).join(""),p=i.length===0?`
    <div style="padding: 24px; text-align: center; color: var(--text-dim); font-size: 12px;">
      No historical operator decisions recorded in this session.
    </div>
  `:`
    <div style="overflow-x: auto;">
      <table style="width: 100%; border-collapse: collapse; font-family: var(--ff-mono); font-size: 11.5px; text-align: left;">
        <thead>
          <tr style="border-bottom: 1px solid var(--border); color: var(--text-muted); font-size: 10.5px; text-transform: uppercase;">
            <th style="padding: 10px 14px;">Status</th>
            <th style="padding: 10px 14px;">Capability / Tool</th>
            <th style="padding: 10px 14px;">Approval ID</th>
            <th style="padding: 10px 14px;">Operator</th>
            <th style="padding: 10px 14px;">Reason / Notes</th>
            <th style="padding: 10px 14px; text-align: right;">Timestamp</th>
          </tr>
        </thead>
        <tbody>
          ${i.map((l)=>`
            <tr style="border-bottom: 1px solid rgba(255,255,255,0.03); transition: background 0.15s;" onmouseover="this.style.background='var(--surface-hover)'" onmouseout="this.style.background='transparent'">
              <td style="padding: 10px 14px;">
                <span class="brand-badge" style="${l.status==="approved"?"background: rgba(52, 211, 153, 0.12); color: var(--green-400); border-color: rgba(52, 211, 153, 0.3);":l.status==="rejected"?"background: rgba(248, 113, 113, 0.12); color: var(--red-400); border-color: rgba(248, 113, 113, 0.3);":"background: var(--surface-hover); color: var(--text-dim);"}">
                  ${l.status.toUpperCase()}
                </span>
              </td>
              <td style="padding: 10px 14px; font-weight: 600; color: var(--text-main);">${w(l.capability_id)}</td>
              <td style="padding: 10px 14px; color: var(--text-dim); font-size: 10.5px;">${w(l.id)}</td>
              <td style="padding: 10px 14px; color: var(--text-muted);">${w(l.operator||"system")}</td>
              <td style="padding: 10px 14px; color: ${l.reason?"var(--red-400)":"var(--text-dim)"};">${l.reason?`"${w(l.reason)}"`:"—"}</td>
              <td style="padding: 10px 14px; text-align: right; color: var(--text-dim);">${new Date(l.created_at*1000).toLocaleTimeString()}</td>
            </tr>
          `).join("")}
        </tbody>
      </table>
    </div>
  `;return`
    <!-- Header -->
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

    <!-- Top Bento Metrics (Full 12-column span) -->
    <div class="bento-grid">
      <div class="bento-card col-3">
        <div class="stat-label">Pending Decisions</div>
        <div class="stat-value" style="color: ${t.length>0?"var(--amber-400)":"var(--text-main)"};">${t.length}</div>
        <div class="stat-sub">Suspended in-flight executions</div>
      </div>
      <div class="bento-card col-3">
        <div class="stat-label">Approved Executions</div>
        <div class="stat-value" style="color: var(--green-400);">${r}</div>
        <div class="stat-sub">Operator sanctioned calls</div>
      </div>
      <div class="bento-card col-3">
        <div class="stat-label">Rejected Requests</div>
        <div class="stat-value" style="color: var(--red-400);">${o}</div>
        <div class="stat-sub">Blocked &amp; reported to agent</div>
      </div>
      <div class="bento-card col-3">
        <div class="stat-label">Active Gating Rules</div>
        <div class="stat-value" style="color: var(--cyan-400);">${n.length}</div>
        <div class="stat-sub">require_approval patterns</div>
      </div>
    </div>

    <!-- Main Content Bento Split (8 cols queue / 4 cols rules) -->
    <div class="bento-grid">
      <!-- Left Column: Pending Queue -->
      <div class="col-8">
        <div style="font-size: 11px; font-weight: 700; color: var(--amber-400); text-transform: uppercase; letter-spacing: 0.5px; margin-bottom: 12px; display: flex; align-items: center; gap: 6px;">
          <span>⚡ Awaiting Operator Decision (${t.length})</span>
        </div>
        <div>
          ${a}
        </div>
      </div>

      <!-- Right Column: Active Rules & Guidelines -->
      <div class="col-4">
        <div style="font-size: 11px; font-weight: 700; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.5px; margin-bottom: 12px; display: flex; justify-content: space-between; align-items: center;">
          <span>\uD83D\uDEE1️ Gating Policy Rules</span>
          <button class="btn btn-ghost" style="padding: 2px 8px; font-size: 10.5px;" onclick="window.app.switchTab('policy')">Edit in Policy →</button>
        </div>
        <div class="bento-card" style="margin-bottom: 14px;">
          ${d}
        </div>

        <div class="bento-card">
          <div class="stat-label" style="margin-bottom: 8px;">Security Notice</div>
          <div style="font-size: 11.5px; color: var(--text-dim); line-height: 1.5;">
            Warmplane enforces an HMAC-SHA256 signature on external webhook approval dispatches. Intercepted payloads are securely held until sanctioned by an operator.
          </div>
        </div>
      </div>
    </div>

    <!-- Bottom History (Full 12 columns) -->
    <div class="bento-card col-12" style="margin-top: 10px;">
      <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px;">
        <div style="font-size: 11px; font-weight: 700; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.5px;">
          \uD83D\uDCDC Recent Decision History (${i.length})
        </div>
      </div>
      ${p}
    </div>
  `}function w(e){return String(e||"").replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;")}function I(){let e=s.getState(),t=e.auditEvents||[],i=e.auditStats||{total_events:0,by_status:{success:0,failed:0,denied:0,intercepted:0}},r=e.auditVerification,o=e.auditFilters,n=e.auditTotal??t.length,a=e.auditSelectedEvent,d=Object.keys(e.config?.mcpServers||{}),p=o.limit||25,l=o.offset||0,g=Math.floor(l/p)+1,b=Math.max(1,Math.ceil(n/p)),f=n===0?0:l+1,_=Math.min(l+p,n),h=u.getAuditExportUrl({actor_id:o.search?void 0:void 0,server_id:o.serverId!=="all"?o.serverId:void 0,event_type:o.eventType!=="all"?o.eventType:void 0,status:o.status!=="all"?o.status:void 0,search:o.search.trim()?o.search.trim():void 0},"csv"),k=u.getAuditExportUrl({server_id:o.serverId!=="all"?o.serverId:void 0,event_type:o.eventType!=="all"?o.eventType:void 0,status:o.status!=="all"?o.status:void 0,search:o.search.trim()?o.search.trim():void 0},"jsonl"),$=r?r.is_valid?`
      <div style="display: inline-flex; align-items: center; gap: 6px; padding: 4px 10px; background: rgba(34, 197, 94, 0.1); border: 1px solid rgba(34, 197, 94, 0.3); border-radius: var(--radius-sm); font-size: 11.5px; color: var(--green-400);">
        <span>\uD83D\uDEE1️</span>
        <span style="font-weight: 600;">Chain Verified: 100% Tamper Free (${r.total_records} events)</span>
      </div>
    `:`
      <div style="display: inline-flex; align-items: center; gap: 6px; padding: 4px 10px; background: rgba(239, 68, 68, 0.1); border: 1px solid rgba(239, 68, 68, 0.3); border-radius: var(--radius-sm); font-size: 11.5px; color: var(--red-400);">
        <span>⚠️</span>
        <span style="font-weight: 600;">TAMPER DETECTED at Record #${r.corrupted_at_index}</span>
      </div>
    `:`
    <button class="btn btn-ghost" style="padding: 4px 10px; font-size: 11.5px;" onclick="window.app.verifyAuditChain()">
      \uD83D\uDEE1️ Verify Cryptographic Hash Chain
    </button>
  `,x=d.map((y)=>`<option value="${m(y)}" ${o.serverId===y?"selected":""}>${m(y)}</option>`).join(""),z=`
    <div class="bento-card" style="padding: 14px 16px; margin-bottom: 16px; background: rgba(18, 24, 38, 0.7); border: 1px solid var(--border);">
      <div style="display: grid; grid-template-columns: 2fr 1fr 1.2fr 1.2fr auto auto; gap: 10px; align-items: center;">
        <!-- Full-text search input -->
        <div style="position: relative;">
          <input 
            type="text" 
            id="audit-search-input" 
            class="form-input" 
            style="width: 100%; padding-left: 28px; font-size: 12px; height: 32px;"
            placeholder="Search trace, actor, capability, hash, error..." 
            value="${m(o.search)}"
            oninput="window.app.handleAuditSearchInput(this.value)"
          />
          <span style="position: absolute; left: 8px; top: 7px; font-size: 12px; color: var(--text-dim);">\uD83D\uDD0D</span>
        </div>

        <!-- Status Filter -->
        <div>
          <select 
            class="form-input" 
            style="width: 100%; font-size: 12px; height: 32px;"
            onchange="window.app.handleAuditStatusFilter(this.value)"
          >
            <option value="all" ${o.status==="all"?"selected":""}>All Statuses</option>
            <option value="success" ${o.status==="success"?"selected":""}>\uD83D\uDFE2 Success</option>
            <option value="denied" ${o.status==="denied"?"selected":""}>\uD83D\uDD34 Denied</option>
            <option value="intercepted" ${o.status==="intercepted"?"selected":""}>\uD83D\uDFE1 HITL Intercept</option>
            <option value="failed" ${o.status==="failed"?"selected":""}>❌ Failed</option>
            <option value="cancelled" ${o.status==="cancelled"?"selected":""}>⚪ Cancelled</option>
          </select>
        </div>

        <!-- Event Type Filter -->
        <div>
          <select 
            class="form-input" 
            style="width: 100%; font-size: 12px; height: 32px;"
            onchange="window.app.handleAuditEventTypeFilter(this.value)"
          >
            <option value="all" ${o.eventType==="all"?"selected":""}>All Event Types</option>
            <option value="tool_execution" ${o.eventType==="tool_execution"?"selected":""}>Tool Execution</option>
            <option value="tool_intercepted_hitl" ${o.eventType==="tool_intercepted_hitl"?"selected":""}>HITL Intercept</option>
            <option value="approval_granted" ${o.eventType==="approval_granted"?"selected":""}>Approval Granted</option>
            <option value="approval_rejected" ${o.eventType==="approval_rejected"?"selected":""}>Approval Rejected</option>
            <option value="approval_expired" ${o.eventType==="approval_expired"?"selected":""}>Approval Expired</option>
            <option value="policy_violation" ${o.eventType==="policy_violation"?"selected":""}>Policy Violation</option>
            <option value="config_mutation" ${o.eventType==="config_mutation"?"selected":""}>Config Mutation</option>
            <option value="sampling_call" ${o.eventType==="sampling_call"?"selected":""}>Sampling Call</option>
            <option value="resource_access" ${o.eventType==="resource_access"?"selected":""}>Resource Access</option>
          </select>
        </div>

        <!-- Server Filter -->
        <div>
          <select 
            class="form-input" 
            style="width: 100%; font-size: 12px; height: 32px;"
            onchange="window.app.handleAuditServerFilter(this.value)"
          >
            <option value="all" ${o.serverId==="all"?"selected":""}>All MCP Servers</option>
            ${x}
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
  `,O=`
    <div style="display: flex; justify-content: space-between; align-items: center; padding: 12px 16px; background: rgba(18, 24, 38, 0.5); border-radius: var(--radius-md); border: 1px solid var(--border); margin-top: 16px;">
      <div style="font-size: 12px; color: var(--text-dim); display: flex; align-items: center; gap: 8px;">
        <span>Showing <strong style="color: var(--text-main);">${f}–${_}</strong> of <strong style="color: var(--text-main);">${n}</strong> events</span>
        <span style="color: var(--border);">|</span>
        <span>Page Size:</span>
        <select 
          class="form-input" 
          style="font-size: 11.5px; padding: 2px 24px 2px 8px; height: 28px; width: auto;"
          onchange="window.app.handleAuditPageSize(this.value)"
        >
          <option value="10" ${p===10?"selected":""}>10 / page</option>
          <option value="25" ${p===25?"selected":""}>25 / page</option>
          <option value="50" ${p===50?"selected":""}>50 / page</option>
          <option value="100" ${p===100?"selected":""}>100 / page</option>
        </select>
      </div>

      <div style="display: flex; align-items: center; gap: 6px;">
        <button 
          class="btn btn-ghost" 
          style="padding: 4px 8px; font-size: 11px; height: 28px;"
          ${g<=1?'disabled style="opacity: 0.4; cursor: not-allowed;"':""}
          onclick="window.app.auditGoToPage(1)"
          title="First Page"
        >
          ⏮ First
        </button>
        <button 
          class="btn btn-ghost" 
          style="padding: 4px 8px; font-size: 11px; height: 28px;"
          ${g<=1?'disabled style="opacity: 0.4; cursor: not-allowed;"':""}
          onclick="window.app.auditPrevPage()"
        >
          ◀ Prev
        </button>
        <span style="font-size: 12px; font-weight: 600; color: var(--text-main); padding: 0 8px;">
          Page ${g} of ${b}
        </span>
        <button 
          class="btn btn-ghost" 
          style="padding: 4px 8px; font-size: 11px; height: 28px;"
          ${g>=b?'disabled style="opacity: 0.4; cursor: not-allowed;"':""}
          onclick="window.app.auditNextPage()"
        >
          Next ▶
        </button>
        <button 
          class="btn btn-ghost" 
          style="padding: 4px 8px; font-size: 11px; height: 28px;"
          ${g>=b?'disabled style="opacity: 0.4; cursor: not-allowed;"':""}
          onclick="window.app.auditGoToPage(${b})"
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
    `;else R=t.map((y)=>{let L=new Date(Math.floor(y.timestamp_ns/1e6)).toLocaleString(),T='<span class="badge" style="background: rgba(34, 197, 94, 0.15); color: var(--green-400); font-weight: 600;">SUCCESS</span>';if(y.status==="denied")T='<span class="badge" style="background: rgba(239, 68, 68, 0.15); color: var(--red-400); font-weight: 600;">DENIED</span>';else if(y.status==="intercepted")T='<span class="badge" style="background: rgba(234, 179, 8, 0.15); color: var(--amber-300); font-weight: 600;">HITL INTERCEPT</span>';else if(y.status==="failed")T='<span class="badge" style="background: rgba(239, 68, 68, 0.15); color: var(--red-400); font-weight: 600;">FAILED</span>';else if(y.status==="cancelled")T='<span class="badge" style="background: rgba(148, 163, 184, 0.15); color: var(--text-muted); font-weight: 600;">CANCELLED</span>';let Q=y.sanitized_args?JSON.stringify(y.sanitized_args):"-",W=y.actor_id||y.operator_id||"anonymous",Y=y.server_id||"system",X=y.capability_id||y.event_type,Z=y.execution_latency_us?`${(y.execution_latency_us/1000).toFixed(1)}ms`:"-";return`
        <div class="bento-card" style="margin-bottom: 12px; padding: 16px; border: 1px solid var(--border); transition: border-color 0.15s ease;">
          <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
            <div style="display: flex; align-items: center; gap: 8px;">
              <span style="font-family: var(--ff-mono); font-size: 11px; font-weight: 700; color: var(--text-dim);">${m(y.id)}</span>
              ${T}
              <span style="font-size: 12px; font-weight: 600; color: var(--text-main);">${m(X)}</span>
            </div>
            <div style="display: flex; align-items: center; gap: 10px;">
              <div style="font-family: var(--ff-mono); font-size: 11px; color: var(--text-muted);">${m(L)}</div>
              <button 
                class="btn btn-ghost" 
                style="padding: 2px 8px; font-size: 11px; height: 24px;" 
                onclick="window.app.selectAuditEvent('${m(y.id)}')"
                title="Inspect event details & cryptographic payload"
              >
                Inspect \uD83D\uDD0D
              </button>
            </div>
          </div>
          
          <div style="display: grid; grid-template-columns: repeat(4, 1fr); gap: 8px; font-size: 11.5px; background: rgba(0,0,0,0.2); padding: 8px 12px; border-radius: var(--radius-sm); margin-bottom: 8px;">
            <div><span style="color: var(--text-muted);">Actor:</span> <strong style="color: var(--text-main);">${m(W)}</strong></div>
            <div><span style="color: var(--text-muted);">Server:</span> <strong style="color: var(--cyan-400);">${m(Y)}</strong></div>
            <div><span style="color: var(--text-muted);">Trace:</span> <code style="color: var(--cyan-400); font-size: 10.5px;">${m(y.trace_id)}</code></div>
            <div><span style="color: var(--text-muted);">Latency:</span> <span style="color: var(--amber-300);">${Z}</span></div>
          </div>

          <div style="font-family: var(--ff-mono); font-size: 11px; color: var(--text-dim); margin-bottom: 8px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">
            <span style="color: var(--text-muted);">Args:</span> ${m(Q)}
          </div>

          <div style="display: flex; justify-content: space-between; align-items: center; border-top: 1px solid var(--border); padding-top: 6px; font-size: 10.5px; font-family: var(--ff-mono); color: var(--text-muted);">
            <div><span style="color: var(--text-dim);">prev_hash:</span> ${m(y.prev_hash.slice(0,16))}...</div>
            <div><span style="color: var(--text-dim);">hash:</span> <span style="color: var(--green-400);">${m(y.hash.slice(0,16))}...</span></div>
          </div>
        </div>
      `}).join("");let P="";if(a){let y=new Date(Math.floor(a.timestamp_ns/1e6)).toISOString();P=`
      <div style="position: fixed; inset: 0; background: rgba(0,0,0,0.7); backdrop-filter: blur(4px); z-index: 9999; display: flex; align-items: center; justify-content: center; padding: 24px;" onclick="if (event.target === this) window.app.selectAuditEvent(null)">
        <div class="bento-card" style="width: 100%; max-width: 720px; max-height: 85vh; display: flex; flex-direction: column; overflow: hidden; background: #0f172a; border: 1px solid var(--border); box-shadow: 0 20px 40px rgba(0,0,0,0.5);">
          <!-- Modal Header -->
          <div style="display: flex; justify-content: space-between; align-items: center; padding: 16px 20px; border-bottom: 1px solid var(--border);">
            <div style="display: flex; align-items: center; gap: 8px;">
              <span style="font-size: 16px;">\uD83D\uDD12</span>
              <h2 style="font-size: 15px; font-weight: 700; color: var(--text-main); margin: 0;">Audit Event Details (${m(a.id)})</h2>
            </div>
            <button class="btn btn-ghost" style="padding: 4px 8px; font-size: 14px;" onclick="window.app.selectAuditEvent(null)">✕</button>
          </div>

          <!-- Modal Body -->
          <div style="padding: 20px; overflow-y: auto; display: flex; flex-direction: column; gap: 14px; font-size: 12px;">
            <div style="display: grid; grid-template-columns: repeat(2, 1fr); gap: 10px; background: rgba(0,0,0,0.25); padding: 12px; border-radius: var(--radius-sm);">
              <div><span style="color: var(--text-muted);">Timestamp:</span> <strong style="color: var(--text-main); font-family: var(--ff-mono); font-size: 11px;">${m(y)}</strong></div>
              <div><span style="color: var(--text-muted);">Status:</span> <strong style="color: var(--text-main);">${m(a.status.toUpperCase())}</strong></div>
              <div><span style="color: var(--text-muted);">Event Type:</span> <strong style="color: var(--text-main);">${m(a.event_type)}</strong></div>
              <div><span style="color: var(--text-muted);">Server:</span> <strong style="color: var(--cyan-400);">${m(a.server_id||"system")}</strong></div>
              <div><span style="color: var(--text-muted);">Capability:</span> <strong style="color: var(--text-main);">${m(a.capability_id||"-")}</strong></div>
              <div><span style="color: var(--text-muted);">Actor / Operator:</span> <strong style="color: var(--text-main);">${m(a.actor_id||a.operator_id||"anonymous")}</strong></div>
              <div><span style="color: var(--text-muted);">Trace ID:</span> <code style="color: var(--cyan-400);">${m(a.trace_id)}</code></div>
              <div><span style="color: var(--text-muted);">Request ID:</span> <code style="color: var(--cyan-400);">${m(a.request_id||"-")}</code></div>
              <div><span style="color: var(--text-muted);">Client IP:</span> <span style="color: var(--text-main);">${m(a.client_ip||"-")}</span></div>
              <div><span style="color: var(--text-muted);">Latency:</span> <span style="color: var(--amber-300);">${a.execution_latency_us?`${(a.execution_latency_us/1000).toFixed(2)} ms`:"-"}</span></div>
            </div>

            ${a.error_message?`
              <div style="background: rgba(239, 68, 68, 0.1); border: 1px solid rgba(239, 68, 68, 0.3); border-radius: var(--radius-sm); padding: 10px 12px; color: var(--red-400);">
                <div style="font-weight: 700; margin-bottom: 2px;">Error (${m(a.error_code||"ERROR")}):</div>
                <div style="font-family: var(--ff-mono); font-size: 11px;">${m(a.error_message)}</div>
              </div>
            `:""}

            <!-- Sanitized Arguments -->
            <div>
              <div style="font-weight: 600; color: var(--text-main); margin-bottom: 4px;">Sanitized Arguments</div>
              <pre style="background: rgba(0,0,0,0.4); padding: 10px; border-radius: var(--radius-sm); border: 1px solid var(--border); font-family: var(--ff-mono); font-size: 11px; max-height: 140px; overflow: auto; margin: 0; color: #cbd5e1;">${m(JSON.stringify(a.sanitized_args||{},null,2))}</pre>
            </div>

            <!-- Sanitized Response -->
            ${a.sanitized_response?`
              <div>
                <div style="font-weight: 600; color: var(--text-main); margin-bottom: 4px;">Sanitized Response</div>
                <pre style="background: rgba(0,0,0,0.4); padding: 10px; border-radius: var(--radius-sm); border: 1px solid var(--border); font-family: var(--ff-mono); font-size: 11px; max-height: 140px; overflow: auto; margin: 0; color: #cbd5e1;">${m(JSON.stringify(a.sanitized_response,null,2))}</pre>
              </div>
            `:""}

            <!-- Cryptographic Hashes -->
            <div style="background: rgba(0,0,0,0.3); padding: 12px; border-radius: var(--radius-sm); border: 1px solid var(--border);">
              <div style="font-weight: 600; color: var(--text-main); margin-bottom: 6px;">Tamper-Evidence Cryptographic Hashes</div>
              <div style="margin-bottom: 6px;">
                <span style="color: var(--text-muted); font-size: 10.5px;">Previous Chain Hash (prev_hash):</span>
                <div style="font-family: var(--ff-mono); font-size: 10.5px; color: var(--text-dim); word-break: break-all;">${m(a.prev_hash)}</div>
              </div>
              <div>
                <span style="color: var(--text-muted); font-size: 10.5px;">Record Hash Signature (hash):</span>
                <div style="font-family: var(--ff-mono); font-size: 10.5px; color: var(--green-400); word-break: break-all;">${m(a.hash)}</div>
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
    ${n>0?O:""}

    <!-- Modal Popup for Event Inspection -->
    ${P}
  `}function m(e){if(!e)return"";return String(e).replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;").replace(/'/g,"&#039;")}function B(){let t=s.getState().config.policy||{},i=t.allow||[],r=t.deny||[],o=t.redact_keys||t.redactKeys||[],n=t.require_approval||t.requireApproval||[],a=i.length===0?`
    <div style="color: var(--text-dim); font-size: 12px;">No allow list (all non-denied operations permitted)</div>
  `:i.map((g,b)=>`
    <div style="display: flex; justify-content: space-between; align-items: center; background: var(--surface); padding: 8px 12px; border-radius: var(--radius-sm); border: 1px solid var(--border);">
      <span style="font-family: var(--ff-mono); font-size: 12px; color: var(--green-400);">✔ ${j(g)}</span>
      <button class="btn btn-ghost" style="padding: 2px 6px; font-size: 11px; color: var(--red-400);" onclick="window.app.removePolicyRule('allow', ${b})">✕</button>
    </div>
  `).join(""),d=r.length===0?`
    <div style="color: var(--text-dim); font-size: 12px;">No deny rules configured</div>
  `:r.map((g,b)=>`
    <div style="display: flex; justify-content: space-between; align-items: center; background: var(--surface); padding: 8px 12px; border-radius: var(--radius-sm); border: 1px solid var(--border);">
      <span style="font-family: var(--ff-mono); font-size: 12px; color: var(--red-400);">✖ ${j(g)}</span>
      <button class="btn btn-ghost" style="padding: 2px 6px; font-size: 11px; color: var(--red-400);" onclick="window.app.removePolicyRule('deny', ${b})">✕</button>
    </div>
  `).join(""),p=n.length===0?`
    <div style="color: var(--text-dim); font-size: 12px;">No human-in-the-loop approval rules configured</div>
  `:n.map((g,b)=>`
    <div style="display: flex; justify-content: space-between; align-items: center; background: var(--surface); padding: 8px 12px; border-radius: var(--radius-sm); border: 1px solid var(--border);">
      <span style="font-family: var(--ff-mono); font-size: 12px; color: var(--amber-400);">\uD83D\uDEE1️ ${j(g)}</span>
      <button class="btn btn-ghost" style="padding: 2px 6px; font-size: 11px; color: var(--red-400);" onclick="window.app.removePolicyRule('requireApproval', ${b})">✕</button>
    </div>
  `).join(""),l=o.length===0?`
    <div style="color: var(--text-dim); font-size: 12px;">No key redaction patterns configured</div>
  `:o.map((g,b)=>`
    <span class="brand-badge" style="color: var(--amber-300); padding: 5px 10px; font-size: 11px; display: inline-flex; align-items: center; gap: 6px;">
      ${j(g)}
      <span style="cursor: pointer; color: var(--red-400); font-weight: bold;" onclick="window.app.removePolicyRule('redact', ${b})">✕</span>
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
          ${a}
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
          ${l}
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
  `}function j(e){return String(e||"").replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;")}function K(){let e=s.getState(),t=e.config,i=Object.entries(t.capabilityAliases||{}),r=Object.entries(t.resourceAliases||{}),o=Object.entries(t.promptAliases||{}),n="";if(i.length===0&&r.length===0&&o.length===0)n=`
      <div style="padding: 24px; text-align: center; color: var(--text-dim);">
        No facade aliases configured in ${S(e.configPath)}. Add short names to prune token payload sizes.
      </div>
    `;else{for(let[a,d]of i)n+=`
        <div class="feed-row" style="grid-template-columns: 90px 180px 1fr 80px;">
          <span style="color: var(--cyan-400);">Tool</span>
          <span style="font-weight: 700; color: var(--text-main); font-family: var(--ff-mono);">${S(a)}</span>
          <span style="color: var(--text-dim); font-family: var(--ff-mono);">${S(d)}</span>
          <div style="text-align: right;">
            <button class="btn btn-ghost" style="padding: 2px 6px; color: var(--red-400);" onclick="window.app.deleteAlias('tool', '${S(a)}')">✕</button>
          </div>
        </div>
      `;for(let[a,d]of r)n+=`
        <div class="feed-row" style="grid-template-columns: 90px 180px 1fr 80px;">
          <span style="color: var(--green-400);">Resource</span>
          <span style="font-weight: 700; color: var(--text-main); font-family: var(--ff-mono);">${S(a)}</span>
          <span style="color: var(--text-dim); font-family: var(--ff-mono);">${S(d)}</span>
          <div style="text-align: right;">
            <button class="btn btn-ghost" style="padding: 2px 6px; color: var(--red-400);" onclick="window.app.deleteAlias('resource', '${S(a)}')">✕</button>
          </div>
        </div>
      `;for(let[a,d]of o)n+=`
        <div class="feed-row" style="grid-template-columns: 90px 180px 1fr 80px;">
          <span style="color: var(--amber-300);">Prompt</span>
          <span style="font-weight: 700; color: var(--text-main); font-family: var(--ff-mono);">${S(a)}</span>
          <span style="color: var(--text-dim); font-family: var(--ff-mono);">${S(d)}</span>
          <div style="text-align: right;">
            <button class="btn btn-ghost" style="padding: 2px 6px; color: var(--red-400);" onclick="window.app.deleteAlias('prompt', '${S(a)}')">✕</button>
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
      ${n}
    </div>
  `}function S(e){return String(e||"").replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;")}var M=[{id:"github",name:"GitHub",category:"devtools",description:"Explore repositories, issues, pull requests, branches, and commit histories.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-github"],envFields:[{key:"GITHUB_PERSONAL_ACCESS_TOKEN",label:"GitHub Personal Access Token",placeholder:"ghp_...",required:!0,description:"Classic or fine-grained token with repo scope."}]},{id:"git",name:"Git (Local)",category:"devtools",description:"Read local Git repository status, diffs, log histories, and commit changes.",badge:"Official / uvx",command:"uvx",defaultArgs:["mcp-server-git","--repository","."],argsPlaceholder:"mcp-server-git --repository /path/to/repo",envFields:[]},{id:"filesystem",name:"Filesystem",category:"devtools",description:"Secure, sandboxed access to local files and directories for AI workflows.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-filesystem","."],argsPlaceholder:"-y @modelcontextprotocol/server-filesystem /allowed/dir1 /allowed/dir2",envFields:[]},{id:"memory",name:"Memory Graph",category:"devtools",description:"Persistent knowledge-graph based memory for multi-turn agent learning.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-memory"],envFields:[]},{id:"chrome-devtools",name:"Chrome DevTools",category:"devtools",description:"Inspect live DOM, execute scripts, read console logs, and capture network traces in Chrome.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-puppeteer"],envFields:[]},{id:"sentry",name:"Sentry",category:"devtools",description:"Query production error events, stack traces, and issue frequencies directly from Sentry.",badge:"uvx / Telemetry",command:"uvx",defaultArgs:["mcp-server-sentry"],envFields:[{key:"SENTRY_AUTH_TOKEN",label:"Sentry Auth Token",placeholder:"sntrys_...",required:!0}]},{id:"playwright",name:"Playwright Browser",category:"browser",description:"Headless / headed browser automation for scraping, form filling, and UI interaction.",badge:"Popular #1 / npx",command:"npx",defaultArgs:["-y","@executeautomation/playwright-mcp-server"],envFields:[]},{id:"puppeteer",name:"Puppeteer",category:"browser",description:"Official browser automation server for web page scraping and screenshot capture.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-puppeteer"],envFields:[]},{id:"brave-search",name:"Brave Search",category:"browser",description:"Real-time privacy-preserving web search and local point-of-interest query engine.",badge:"Official / Search",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-brave-search"],envFields:[{key:"BRAVE_API_KEY",label:"Brave Search API Key",placeholder:"BSA...",required:!0}]},{id:"tavily",name:"Tavily Search",category:"browser",description:"AI-optimized web search engine structured specifically for LLM context injection.",badge:"Community / Stdio",command:"npx",defaultArgs:["-y","@tavily/mcp-server"],envFields:[{key:"TAVILY_API_KEY",label:"Tavily API Key",placeholder:"tvly-...",required:!0}]},{id:"fetch",name:"Fetch / Web Markdown",category:"browser",description:"Download web pages, strip clutter, and convert raw HTML to clean markdown text.",badge:"Official / uvx",command:"uvx",defaultArgs:["mcp-server-fetch"],envFields:[]},{id:"postgres",name:"PostgreSQL",category:"database",description:"Read schemas, inspect tables, and execute SQL queries against PostgreSQL databases.",badge:"Official / Database",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-postgres","postgresql://user:pass@localhost:5432/mydb"],argsPlaceholder:"-y @modelcontextprotocol/server-postgres postgresql://user:pass@localhost:5432/dbname",envFields:[]},{id:"sqlite",name:"SQLite",category:"database",description:"Local embedded SQLite query runner and schema inspector.",badge:"Official / uvx",command:"uvx",defaultArgs:["mcp-server-sqlite","--db-path","./app.db"],argsPlaceholder:"mcp-server-sqlite --db-path /path/to/database.sqlite",envFields:[]},{id:"supabase",name:"Supabase",category:"database",description:"Query database tables, manage auth policies, and inspect storage in Supabase.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@supabase/mcp-server"],envFields:[{key:"SUPABASE_ACCESS_TOKEN",label:"Supabase Personal Access Token",placeholder:"sbp_...",required:!0},{key:"SUPABASE_PROJECT_REF",label:"Supabase Project Reference ID",placeholder:"abcdefghijklmnop",required:!1}]},{id:"redis",name:"Redis",category:"database",description:"Inspect cached keys, hash sets, lists, TTLs, and pub/sub channels in Redis.",badge:"uvx / Key-Value",command:"uvx",defaultArgs:["mcp-server-redis","--url","redis://localhost:6379"],argsPlaceholder:"mcp-server-redis --url redis://localhost:6379",envFields:[]},{id:"s3",name:"AWS S3 / Cloud Storage",category:"database",description:"Browse S3 buckets, fetch object metadata, and download files from cloud storage.",badge:"uvx / Cloud Storage",command:"uvx",defaultArgs:["mcp-server-s3","--bucket","my-bucket-name"],argsPlaceholder:"mcp-server-s3 --bucket bucket-name --region us-east-1",envFields:[{key:"AWS_ACCESS_KEY_ID",label:"AWS Access Key ID",placeholder:"AKIA...",required:!0},{key:"AWS_SECRET_ACCESS_KEY",label:"AWS Secret Access Key",placeholder:"...",required:!0},{key:"AWS_REGION",label:"AWS Region",placeholder:"us-east-1",required:!1}]},{id:"linear",name:"Linear",category:"productivity",description:"Search, create, and triage Linear issues, cycles, teams, and project roadmaps.",badge:"Productivity / Stdio",command:"npx",defaultArgs:["-y","mcp-linear"],envFields:[{key:"LINEAR_API_KEY",label:"Linear API Key",placeholder:"lin_api_...",required:!0}]},{id:"slack",name:"Slack",category:"productivity",description:"Read channels, post messages, inspect threads, and search team discussions.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-slack"],envFields:[{key:"SLACK_BOT_TOKEN",label:"Slack Bot User Token",placeholder:"xoxb-...",required:!0},{key:"SLACK_TEAM_ID",label:"Slack Team ID",placeholder:"T01234567",required:!0}]},{id:"notion",name:"Notion",category:"productivity",description:"Search Notion workspace pages, read nested blocks, and query database entries.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-notion"],envFields:[{key:"NOTION_API_KEY",label:"Notion Internal Integration Token",placeholder:"secret_...",required:!0}]},{id:"jira",name:"Jira / Atlassian",category:"productivity",description:"Manage Jira issues, search JQL, read sprint statuses, and inspect boards.",badge:"uvx / Atlassian",command:"uvx",defaultArgs:["mcp-server-jira","--url","https://your-domain.atlassian.net","--email","user@example.com"],argsPlaceholder:"mcp-server-jira --url https://org.atlassian.net --email me@org.com",envFields:[{key:"JIRA_API_TOKEN",label:"Atlassian API Token",placeholder:"ATATT3...",required:!0}]},{id:"google-drive",name:"Google Drive",category:"productivity",description:"Search, list, and read documents, spreadsheets, and drive files.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-gdrive"],envFields:[{key:"GOOGLE_APPLICATION_CREDENTIALS",label:"Google Credentials JSON Path",placeholder:"/path/to/credentials.json",required:!0}]},{id:"docker",name:"Docker",category:"cloud",description:"Inspect running containers, tail container logs, list images, and manage compose services.",badge:"uvx / DevOps",command:"uvx",defaultArgs:["mcp-server-docker"],envFields:[]},{id:"kubernetes",name:"Kubernetes (K8s)",category:"cloud",description:"Query cluster pods, services, deployment status, and inspect Kubernetes logs.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-kubernetes"],envFields:[{key:"KUBECONFIG",label:"Kubeconfig File Path (Optional)",placeholder:"~/.kube/config",required:!1}]},{id:"cloudflare",name:"Cloudflare",category:"cloud",description:"Manage Cloudflare Workers, KV namespaces, D1 databases, Vectorize indexes, and DNS.",badge:"Official / Cloudflare",command:"npx",defaultArgs:["-y","@cloudflare/mcp-server-cloudflare"],envFields:[{key:"CLOUDFLARE_API_TOKEN",label:"Cloudflare API Token",placeholder:"...",required:!0},{key:"CLOUDFLARE_ACCOUNT_ID",label:"Cloudflare Account ID",placeholder:"...",required:!0}]},{id:"terraform",name:"Terraform",category:"cloud",description:"Inspect Terraform state files, resource dependency graphs, and plan previews.",badge:"uvx / IaC",command:"uvx",defaultArgs:["mcp-server-terraform"],envFields:[]}];class J{activeTemplateCategory="all";activeTemplateFilter="";selectedTemplate=null;async init(){let e=window.location.port?`:${window.location.port}`:"",t=document.getElementById("daemon-port-label");if(t)t.textContent=`Daemon ${e}`;await this.refreshData(),this.initSSE(),this.render(),s.subscribe(()=>{this.render()})}auditSearchTimeout=null;async refreshData(){try{let t=s.getState().auditFilters,[i,r,o,n,a,d,p,l]=await Promise.all([u.getConfig(),u.listCapabilities(),u.listResources(),u.listPrompts(),u.getCatalogEvents(),u.listApprovals(),u.listAuditEvents({server_id:t.serverId!=="all"?t.serverId:void 0,event_type:t.eventType!=="all"?t.eventType:void 0,status:t.status!=="all"?t.status:void 0,search:t.search.trim()?t.search.trim():void 0,limit:t.limit,offset:t.offset}),u.getAuditStats()]);if(i.ok)s.setState({configPath:i.config_path,config:i.config,serverStatuses:i.server_statuses||{},circuitBreakers:i.circuit_breakers||[],metrics:{totalCatalogRequests:i.metrics?.total_catalog_requests||0,totalEtagHits:i.metrics?.total_etag_hits||0,totalToolCalls:i.metrics?.total_tool_calls||0,totalToolDurationUs:i.metrics?.total_tool_duration_us||0}});if(r&&Array.isArray(r.capabilities))s.setState({capabilities:r.capabilities});if(o&&Array.isArray(o.resources))s.setState({resources:o.resources});if(n&&Array.isArray(n.prompts))s.setState({prompts:n.prompts});if(a&&Array.isArray(a.events))s.setState({catalogEvents:a.events});if(d&&Array.isArray(d.approvals))s.setState({approvals:d.approvals});if(p&&Array.isArray(p.events))s.setState({auditEvents:p.events,auditTotal:p.total??p.events.length});if(l&&l.ok)s.setState({auditStats:l})}catch(e){console.error("Failed to fetch daemon state:",e)}}async refreshAuditEvents(){try{let t=s.getState().auditFilters,[i,r]=await Promise.all([u.listAuditEvents({server_id:t.serverId!=="all"?t.serverId:void 0,event_type:t.eventType!=="all"?t.eventType:void 0,status:t.status!=="all"?t.status:void 0,search:t.search.trim()?t.search.trim():void 0,limit:t.limit,offset:t.offset}),u.getAuditStats()]);if(i&&Array.isArray(i.events))s.setState({auditEvents:i.events,auditTotal:i.total??i.events.length});if(r&&r.ok)s.setState({auditStats:r})}catch(e){console.error("Failed to refresh audit events:",e)}}handleAuditSearchInput(e){let i={...s.getState().auditFilters,search:e,offset:0};s.setState({auditFilters:i}),clearTimeout(this.auditSearchTimeout),this.auditSearchTimeout=setTimeout(()=>{this.refreshAuditEvents()},250)}handleAuditStatusFilter(e){let t=s.getState();s.setState({auditFilters:{...t.auditFilters,status:e,offset:0}}),this.refreshAuditEvents()}handleAuditEventTypeFilter(e){let t=s.getState();s.setState({auditFilters:{...t.auditFilters,eventType:e,offset:0}}),this.refreshAuditEvents()}handleAuditServerFilter(e){let t=s.getState();s.setState({auditFilters:{...t.auditFilters,serverId:e,offset:0}}),this.refreshAuditEvents()}handleAuditPageSize(e){let t=parseInt(e,10)||25,i=s.getState();s.setState({auditFilters:{...i.auditFilters,limit:t,offset:0}}),this.refreshAuditEvents()}clearAuditFilters(){let e=s.getState();s.setState({auditFilters:{search:"",status:"all",eventType:"all",serverId:"all",limit:e.auditFilters.limit||25,offset:0}}),this.refreshAuditEvents()}auditPrevPage(){let e=s.getState(),{limit:t,offset:i}=e.auditFilters,r=Math.max(0,i-t);if(r!==i)s.setState({auditFilters:{...e.auditFilters,offset:r}}),this.refreshAuditEvents()}auditNextPage(){let e=s.getState(),{limit:t,offset:i}=e.auditFilters,r=e.auditTotal;if(i+t<r)s.setState({auditFilters:{...e.auditFilters,offset:i+t}}),this.refreshAuditEvents()}auditGoToPage(e){let t=s.getState(),{limit:i}=t.auditFilters,r=Math.max(0,(e-1)*i);s.setState({auditFilters:{...t.auditFilters,offset:r}}),this.refreshAuditEvents()}selectAuditEvent(e){if(!e){s.setState({auditSelectedEvent:null});return}let i=s.getState().auditEvents.find((r)=>r.id===e)||null;s.setState({auditSelectedEvent:i})}async verifyAuditChain(){try{let e=await u.verifyAuditChain();if(e&&e.report)s.setState({auditVerification:e.report})}catch(e){console.error("Failed to verify audit chain:",e)}}async refreshApprovals(){try{let e=await u.listApprovals();if(e&&Array.isArray(e.approvals))s.setState({approvals:e.approvals})}catch(e){console.error("Failed to refresh approvals:",e)}}initSSE(){try{let e=new EventSource("/v1/resources/updates");e.onmessage=(t)=>{s.addEventLog("SSE","/v1/resources/updates","UPDATED","0.1ms"),this.refreshData()}}catch(e){console.warn("SSE connection unavailable")}}switchTab(e){s.setState({activeTab:e}),this.refreshData()}render(){let e=s.getState(),t=document.getElementById("app-main");if(!t)return;let i=e.approvals.filter((a)=>a.status==="pending").length,r=document.getElementById("nav-approvals-badge");if(r)r.textContent=i>0?`${i}`:"",r.style.display=i>0?"inline-block":"none";document.querySelectorAll(".nav-item").forEach((a)=>{if(a.getAttribute("data-tab")===e.activeTab)a.classList.add("active");else a.classList.remove("active")});let o=document.getElementById("top-title"),n={overview:"Overview Cockpit",servers:"Server Hub & Connections",playground:"MCP Capability Playground",approvals:"Human-in-the-Loop Review Queue",audit:"WORM Audit & Compliance Ledger",policy:"Security Governance & Redaction",aliases:"Facade & Alias Studio"};if(o)o.textContent=n[e.activeTab];switch(e.activeTab){case"overview":t.innerHTML=D();break;case"servers":t.innerHTML=E();break;case"playground":t.innerHTML=U();break;case"approvals":t.innerHTML=F(e);break;case"audit":t.innerHTML=I();break;case"policy":t.innerHTML=B();break;case"aliases":t.innerHTML=K();break}}async submitApproval(e){let t=document.getElementById(`appr-operator-${e}`),i=document.getElementById(`appr-args-${e}`),r=t?.value.trim()||"security-operator",o=void 0;if(i&&i.value.trim())try{o=JSON.parse(i.value.trim())}catch{alert("Invalid JSON in arguments editor");return}let n=await u.approveTicket(e,r,o);if(n.ok)await this.refreshApprovals();else alert(`Approval failed: ${n.error||"Unknown error"}`)}async promptReject(e){let t=prompt("Reason for rejection (will be returned to the calling agent):");if(t===null)return;let r=document.getElementById(`appr-operator-${e}`)?.value.trim()||"security-operator",o=await u.rejectTicket(e,r,t);if(o.ok)await this.refreshApprovals();else alert(`Rejection failed: ${o.error||"Unknown error"}`)}selectCapability(e){s.setState({selectedCapabilityId:e})}filterCapabilities(e){let t=e.toLowerCase().trim(),r=s.getState().capabilities.filter((n)=>n.id.toLowerCase().includes(t)||n.summary&&n.summary.toLowerCase().includes(t)||n.server&&n.server.toLowerCase().includes(t)),o=document.getElementById("pg-cap-list");if(o)if(r.length===0)o.innerHTML=`
          <div style="padding: 24px 16px; text-align: center; color: var(--text-dim); font-size: 11.5px;">
            No capabilities match "${v(e)}"
          </div>
        `;else o.innerHTML=r.map((n)=>`
          <div class="cap-item ${n.id===s.getState().selectedCapabilityId?"active":""}" onclick="window.app.selectCapability('${v(n.id)}')">
            <div style="display: flex; justify-content: space-between; align-items: center;">
              <span style="font-weight: 600; color: var(--text-main); font-family: var(--ff-mono); font-size: 12px;">${v(n.id)}</span>
              <span style="font-size: 10px; color: var(--green-400);">${v(n.mode||"read")}</span>
            </div>
            <div style="font-size: 11px; color: var(--text-dim); margin-top: 2px;">server: ${v(n.server||"local")}</div>
          </div>
        `).join("")}async executePlaygroundTool(){let e=s.getState(),t=e.selectedCapabilityId||(e.capabilities[0]?e.capabilities[0].id:null);if(!t)return;let i=document.getElementById("pg-args-input")?.value||"{}",r=document.getElementById("pg-context-input")?.value||void 0,o=document.getElementById("pg-jsonpath-input")?.value.trim()||void 0,n=document.getElementById("pg-limit-lines-input")?.value.trim()||void 0,a=document.getElementById("pg-truncate-bytes-input")?.value.trim()||void 0,d={};try{d=JSON.parse(i)}catch{alert("Invalid arguments JSON object");return}if(o)d._jsonpath=o;if(n&&!isNaN(Number(n)))d._limit_lines=Number(n);if(a&&!isNaN(Number(a)))d._truncate_bytes=Number(a);let p=`op-${Date.now()}`;s.setState({isExecuting:!0,activeRequestId:p});try{let l=await u.callCapability({capability_id:t,args:d,request_id:p,context:{operation_id:r||p}});s.setState({isExecuting:!1,activeRequestId:null,executionResult:{status:l.status,durationMs:l.durationMs,data:l.data}}),s.addEventLog("POST",`/v1/tools/call → ${t}`,l.status===200?"200 OK":`HTTP ${l.status}`,`${l.durationMs.toFixed(1)}ms`),u.getConfig().then((g)=>{if(g.ok&&g.circuit_breakers)s.setState({circuitBreakers:g.circuit_breakers})})}catch(l){s.setState({isExecuting:!1,activeRequestId:null,executionResult:{status:500,durationMs:0,data:{error:l.toString()}}})}}async cancelActiveOperation(){let t=s.getState().activeRequestId;if(t)try{await u.cancelOperation(t),s.addEventLog("POST",`/v1/operations/${t}/cancel`,"CANCELLED","0.1ms")}catch(i){console.error("Failed to cancel operation:",i)}s.setState({isExecuting:!1,activeRequestId:null,executionResult:{status:499,durationMs:0,data:{ok:!1,cancelled:!0,message:`Operation '${t||"unknown"}' cancelled by user.`}}})}openBatchModal(){let e=s.getState(),t=e.capabilities||[],i=e.batchSteps;if(!i||i.length===0)i=[{id:"step_1",capability_id:t[0]?t[0].id:"",argsJson:"{}",continue_on_error:!1},{id:"step_2",capability_id:t[1]?t[1].id:t[0]?t[0].id:"",argsJson:`{
  "ref_id": "\${steps[0].result.id}"
}`,continue_on_error:!0}];s.setState({isBatchModalOpen:!0,batchSteps:i})}closeBatchModal(){s.setState({isBatchModalOpen:!1})}addBatchStep(){let e=s.getState(),t=[...e.batchSteps||[]],i=e.capabilities||[];t.push({id:`step_${t.length+1}`,capability_id:i[0]?i[0].id:"",argsJson:"{}",continue_on_error:!1}),s.setState({batchSteps:t})}removeBatchStep(e){let i=[...s.getState().batchSteps||[]];if(i.length<=1){alert("A batch pipeline requires at least one step.");return}i.splice(e,1),s.setState({batchSteps:i})}updateBatchStepCapability(e,t){let r=[...s.getState().batchSteps||[]];if(r[e])r[e].capability_id=t,s.setState({batchSteps:r})}updateBatchStepArgs(e,t){let r=[...s.getState().batchSteps||[]];if(r[e])r[e].argsJson=t}updateBatchStepContinueOnError(e,t){let r=[...s.getState().batchSteps||[]];if(r[e])r[e].continue_on_error=t,s.setState({batchSteps:r})}appendBatchVariable(e,t){let i=document.getElementById(`batch-step-args-${e}`);if(i)i.value+=t,this.updateBatchStepArgs(e,i.value)}async executeBatchPipeline(){let t=s.getState().batchSteps||[],i=[];for(let r=0;r<t.length;r++){let o=t[r];if(!o.capability_id){alert(`Please select a capability for Step ${r+1}`);return}let n={};try{n=JSON.parse(o.argsJson||"{}")}catch{alert(`Invalid JSON in Step ${r+1} arguments`);return}i.push({id:o.id||`step_${r+1}`,capability_id:o.capability_id,args:n,continue_on_error:o.continue_on_error})}s.setState({isBatchModalOpen:!1});try{let r=await u.batchCallCapabilities(i);s.setState({executionResult:{status:r.status,durationMs:r.durationMs,data:r.data}}),s.addEventLog("POST",`/v1/tools/batch_call (${t.length} steps)`,r.status===200?"200 OK":`HTTP ${r.status}`,`${r.durationMs.toFixed(1)}ms`)}catch(r){s.setState({executionResult:{status:500,durationMs:0,data:{error:r.toString()}}})}}setPlaygroundMode(e){s.setState({playgroundMode:e})}selectResource(e){s.setState({selectedResourceId:e,resourceReadResult:null})}filterResources(e){let t=e.toLowerCase().trim(),r=(s.getState().resources||[]).filter((n)=>n.id.toLowerCase().includes(t)||n.name&&n.name.toLowerCase().includes(t)||n.uri&&n.uri.toLowerCase().includes(t)||n.server&&n.server.toLowerCase().includes(t)),o=document.getElementById("pg-res-list");if(o)if(r.length===0)o.innerHTML=`
          <div style="padding: 24px 16px; text-align: center; color: var(--text-dim); font-size: 11.5px;">
            No resources match "${v(e)}"
          </div>
        `;else o.innerHTML=r.map((n)=>{let a=n.uri?n.uri.split(":")[0]:"res";return`
            <div class="cap-item ${n.id===s.getState().selectedResourceId?"active":""}" onclick="window.app.selectResource('${v(n.id)}')">
              <div style="display: flex; justify-content: space-between; align-items: center;">
                <span style="font-weight: 600; color: var(--text-main); font-family: var(--ff-mono); font-size: 12px;">${v(n.name||n.id)}</span>
                <span class="badge" style="font-size: 9.5px; background: rgba(56, 189, 248, 0.15); color: var(--cyan-400);">${v(a)}</span>
              </div>
              <div style="font-size: 11px; color: var(--text-dim); margin-top: 2px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">${v(n.uri)}</div>
              <div style="display: flex; justify-content: space-between; font-size: 10px; color: var(--text-muted); margin-top: 4px;">
                <span>server: ${v(n.server||"local")}</span>
                <span>${v(n.mime_type||"text/plain")}</span>
              </div>
            </div>
          `}).join("")}async executeReadResource(){let e=s.getState(),t=e.selectedResourceId||(e.resources[0]?e.resources[0].id:null);if(!t)return;let i=document.getElementById("pg-res-jsonpath-input")?.value.trim()||void 0,r=document.getElementById("pg-res-lines-input")?.value.trim()||void 0,o=document.getElementById("pg-res-bytes-input")?.value.trim()||void 0,n={resource_id:t};if(i)n._jsonpath=i;if(r&&!isNaN(Number(r)))n._limit_lines=Number(r);if(o&&!isNaN(Number(o)))n._truncate_bytes=Number(o);try{let a=await u.readResource(n);s.setState({resourceReadResult:{status:a.status,durationMs:a.durationMs,data:a.data}}),s.addEventLog("POST",`/v1/resources/read → ${t}`,a.status===200?"200 OK":`HTTP ${a.status}`,`${a.durationMs.toFixed(1)}ms`)}catch(a){s.setState({resourceReadResult:{status:500,durationMs:0,data:{error:a.toString()}}})}}selectPrompt(e){s.setState({selectedPromptId:e,promptGetResult:null})}filterPrompts(e){let t=e.toLowerCase().trim(),r=(s.getState().prompts||[]).filter((n)=>n.id.toLowerCase().includes(t)||n.name&&n.name.toLowerCase().includes(t)||n.description&&n.description.toLowerCase().includes(t)||n.server&&n.server.toLowerCase().includes(t)),o=document.getElementById("pg-prompt-list");if(o)if(r.length===0)o.innerHTML=`
          <div style="padding: 24px 16px; text-align: center; color: var(--text-dim); font-size: 11.5px;">
            No prompts match "${v(e)}"
          </div>
        `;else o.innerHTML=r.map((n)=>{let a=n.arguments?n.arguments.length:0;return`
            <div class="cap-item ${n.id===s.getState().selectedPromptId?"active":""}" onclick="window.app.selectPrompt('${v(n.id)}')">
              <div style="display: flex; justify-content: space-between; align-items: center;">
                <span style="font-weight: 600; color: var(--text-main); font-family: var(--ff-mono); font-size: 12px;">${v(n.name||n.id)}</span>
                <span class="badge" style="font-size: 9.5px; background: rgba(168, 85, 247, 0.15); color: var(--purple-400);">${a} args</span>
              </div>
              <div style="font-size: 11px; color: var(--text-dim); margin-top: 2px;">${v(n.description||n.title||"Prompt template")}</div>
              <div style="font-size: 10px; color: var(--text-muted); margin-top: 4px;">server: ${v(n.server||"local")}</div>
            </div>
          `}).join("")}async executeGetPrompt(){let e=s.getState(),t=e.selectedPromptId||(e.prompts[0]?e.prompts[0].id:null);if(!t)return;let i=document.querySelectorAll(".prompt-arg-input"),r={};i.forEach((o)=>{let n=o,a=n.getAttribute("data-arg-name");if(a&&n.value.trim())r[a]=n.value.trim()});try{let o=await u.getPrompt({prompt_id:t,arguments:r});s.setState({promptGetResult:{status:o.status,durationMs:o.durationMs,data:o.data}}),s.addEventLog("POST",`/v1/prompts/get → ${t}`,o.status===200?"200 OK":`HTTP ${o.status}`,`${o.durationMs.toFixed(1)}ms`)}catch(o){s.setState({promptGetResult:{status:500,durationMs:0,data:{error:o.toString()}}})}}toggleBatchPlayground(){let e=document.getElementById("pg-args-input");if(!e)return;let t=[{id:"step_1",capability_id:"sqlite.read_query",args:{query:"SELECT * FROM users LIMIT 2"}},{id:"step_2",capability_id:"github.issues.search",args:{query:"label:bug"},continue_on_error:!0}];e.value=JSON.stringify(t,null,2)}async submitPolicyRule(e){let t=e==="allow"?"policy-new-allow":e==="deny"?"policy-new-deny":"policy-new-redact",i=document.getElementById(t);if(!i)return;let r=i.value.trim();if(!r)return;await this.addPolicyRule(e,r),i.value=""}async addPolicyRule(e,t){let i=(t||"").trim();if(!i)return;let o=s.getState().config.policy||{},n=[...o.allow||[]],a=[...o.deny||[]],d=[...o.redact_keys||o.redactKeys||[]];if(e==="allow"&&!n.includes(i))n.push(i);if(e==="deny"&&!a.includes(i))a.push(i);if(e==="redact"&&!d.includes(i))d.push(i);let p=await u.savePolicy({allow:n,deny:a,redact_keys:d,redactKeys:d});if(!p.ok)alert(`Failed to save policy rule: ${p.error||"Unknown error"}`);await this.refreshData()}async removePolicyRule(e,t){let r=s.getState().config.policy||{},o=[...r.allow||[]],n=[...r.deny||[]],a=[...r.redact_keys||r.redactKeys||[]];if(e==="allow")o.splice(t,1);if(e==="deny")n.splice(t,1);if(e==="redact")a.splice(t,1);let d=await u.savePolicy({allow:o,deny:n,redact_keys:a,redactKeys:a});if(!d.ok)alert(`Failed to update policy: ${d.error||"Unknown error"}`);await this.refreshData()}testPolicySandbox(e){let t=document.getElementById("policy-test-verdict");if(!t)return;let i=e.trim();if(!i){t.textContent="ENTER ID",t.style.color="var(--text-dim)";return}let o=s.getState().config.policy||{},n=o.deny||[],a=o.allow||[],d=(p,l)=>{if(p==="*")return!0;if(p.endsWith("*"))return l.startsWith(p.slice(0,-1));return p===l};if(n.some((p)=>d(p,i))){t.textContent="DENIED (Strict Block)",t.style.color="var(--red-400)";return}if(a.length>0&&!a.some((p)=>d(p,i))){t.textContent="DENIED (Not in Allow List)",t.style.color="var(--red-400)";return}t.textContent="ALLOWED",t.style.color="var(--green-400)"}async deleteServer(e){if(!confirm(`Are you sure you want to remove server '${e}' from config?`))return;await u.deleteServer(e),await this.refreshData()}openAddServerModal(){this.closeModals();let e=document.getElementById("modal-srv-title"),t=document.getElementById("modal-srv-template-banner"),i=document.getElementById("modal-srv-name"),r=document.getElementById("modal-srv-transport"),o=document.getElementById("modal-srv-command"),n=document.getElementById("modal-srv-url"),a=document.getElementById("modal-srv-ft"),d=document.getElementById("modal-srv-cd"),p=document.getElementById("modal-srv-autorestart"),l=document.getElementById("modal-srv-maxrestarts");if(e)e.textContent="Add Upstream MCP Server";if(t)t.style.display="flex";if(i)i.value="",i.disabled=!1;if(r)r.value="stdio";if(o)o.value="";if(n)n.value="";let g=document.getElementById("modal-group-cmd"),b=document.getElementById("modal-group-url");if(g)g.style.display="block";if(b)b.style.display="none";if(a)a.value="3";if(d)d.value="30000";if(p)p.value="true";if(l)l.value="5";let f=document.getElementById("modal-add-server");if(f)f.classList.add("active")}openEditServerModal(e){this.closeModals();let t=s.getState(),i=t.config.mcpServers?.[e];if(!i){alert(`Server '${e}' not found in configuration.`);return}let r=document.getElementById("modal-srv-title"),o=document.getElementById("modal-srv-template-banner"),n=document.getElementById("modal-srv-name"),a=document.getElementById("modal-srv-transport"),d=document.getElementById("modal-srv-command"),p=document.getElementById("modal-srv-url"),l=document.getElementById("modal-srv-ft"),g=document.getElementById("modal-srv-cd"),b=document.getElementById("modal-srv-autorestart"),f=document.getElementById("modal-srv-maxrestarts");if(r)r.textContent=`Edit Server '${e}'`;if(o)o.style.display="none";if(n)n.value=e,n.disabled=!0;let _=!!i.command;if(a)a.value=_?"stdio":"http";let h=document.getElementById("modal-group-cmd"),k=document.getElementById("modal-group-url");if(h)h.style.display=_?"block":"none";if(k)k.style.display=_?"none":"block";if(d)d.value=_?`${i.command} ${(i.args||[]).join(" ")}`.trim():"";if(p)p.value=i.url||"";let $=i.resilience||t.config.resilience;if(l)l.value=String($?.failureThreshold??3);if(g)g.value=String($?.cooldownMs??30000);if(b)b.value=$?.autoRestart===!1?"false":"true";if(f)f.value=String($?.maxRestarts??5);let x=document.getElementById("modal-add-server");if(x)x.classList.add("active")}async submitAddServer(){let e=document.getElementById("modal-srv-name")?.value.trim(),t=document.getElementById("modal-srv-transport")?.value;if(!e){alert("Server name is required");return}let i={};if(t==="stdio"){let l=(document.getElementById("modal-srv-command")?.value.trim()).split(/\s+/).filter(Boolean);if(l.length===0){alert("Command is required");return}i.command=l[0],i.args=l.slice(1)}else{let p=document.getElementById("modal-srv-url")?.value.trim();if(!p){alert("URL is required");return}i.url=p}let r=document.getElementById("modal-srv-ft")?.value.trim(),o=document.getElementById("modal-srv-cd")?.value.trim(),n=document.getElementById("modal-srv-autorestart")?.value,a=document.getElementById("modal-srv-maxrestarts")?.value.trim();if(r||o||n||a)i.resilience={failureThreshold:r?Number(r):3,cooldownMs:o?Number(o):30000,autoRestart:n!=="false",maxRestarts:a?Number(a):5};let d=await u.upsertServer(e,i);if(d.ok)this.closeModals(),await this.refreshData();else alert(`Failed to save server: ${d.error}`)}openTemplateCatalog(){this.closeModals();let e=document.getElementById("modal-templates");if(e)e.classList.add("active");this.renderTemplateGrid()}setTemplateCategory(e){this.activeTemplateCategory=e,document.querySelectorAll(".tmpl-cat-btn").forEach((t)=>{if(t.getAttribute("data-category")===e)t.classList.add("active"),t.style.background="var(--surface-elevated)",t.style.color="var(--amber-400)";else t.classList.remove("active"),t.style.background="var(--surface-card)",t.style.color="var(--text-main)"}),this.renderTemplateGrid()}filterTemplates(e){this.activeTemplateFilter=e.toLowerCase().trim(),this.renderTemplateGrid()}renderTemplateGrid(){let e=document.getElementById("tmpl-grid");if(!e)return;let t=M.filter((o)=>{let n=this.activeTemplateCategory==="all"||o.category===this.activeTemplateCategory,a=!this.activeTemplateFilter||o.name.toLowerCase().includes(this.activeTemplateFilter)||o.id.toLowerCase().includes(this.activeTemplateFilter)||o.description.toLowerCase().includes(this.activeTemplateFilter)||o.command.toLowerCase().includes(this.activeTemplateFilter)||o.envFields.some((d)=>d.key.toLowerCase().includes(this.activeTemplateFilter));return n&&a});if(t.length===0){e.innerHTML=`
        <div style="grid-column: span 2; padding: 32px; text-align: center; color: var(--text-dim);">
          No matching MCP server templates found.
        </div>
      `;return}let r=s.getState().config.mcpServers||{};e.innerHTML=t.map((o)=>{let n=!!r[o.id],a=`${o.command} ${o.defaultArgs.join(" ")}`;return`
        <div class="bento-card" style="display: flex; flex-direction: column; justify-content: space-between; padding: 14px; background: var(--surface); border: 1px solid var(--border); transition: transform 0.15s, border-color 0.15s;">
          <div>
            <div style="display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 6px;">
              <div style="display: flex; align-items: center; gap: 8px;">
                <span style="font-weight: 700; font-size: 13.5px; color: var(--text-main);">${v(o.name)}</span>
                <span class="brand-badge" style="font-size: 9.5px; padding: 1px 6px;">${v(o.badge)}</span>
              </div>
              ${n?'<span style="font-size: 10px; color: var(--green-400); font-weight: 600;">CONNECTED</span>':""}
            </div>
            <div style="font-size: 11.5px; color: var(--text-muted); line-height: 1.4; margin-bottom: 8px;">
              ${v(o.description)}
            </div>
            <div style="font-family: var(--ff-mono); font-size: 10.5px; color: var(--text-dim); background: var(--surface-card); padding: 5px 8px; border-radius: var(--radius-sm); border: 1px solid var(--border-subtle); overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">
              <code>${v(a)}</code>
            </div>
            ${o.envFields.length>0?`
              <div style="font-size: 10.5px; color: var(--amber-400); margin-top: 6px; display: flex; align-items: center; gap: 4px;">
                <span>⚡ Needs:</span>
                <code>${o.envFields.map((d)=>v(d.key)).join(", ")}</code>
              </div>
            `:""}
          </div>

          <div style="display: flex; justify-content: flex-end; margin-top: 12px; gap: 6px;">
            <button class="btn btn-primary" style="font-size: 11.5px; padding: 4px 10px;" onclick="window.app.selectTemplate('${v(o.id)}')">
              ${n?"Configure Another":"✨ 1-Click Setup"}
            </button>
          </div>
        </div>
      `}).join("")}selectTemplate(e){let t=M.find((a)=>a.id===e);if(!t)return;this.selectedTemplate=t,this.closeModals();let i=document.getElementById("modal-configure-template");if(i)i.classList.add("active");let r=document.getElementById("cfg-tmpl-title"),o=document.getElementById("cfg-tmpl-desc"),n=document.getElementById("cfg-tmpl-form");if(r)r.textContent=`Configure ${t.name} Server`;if(o)o.textContent=t.description;if(n){let a="";if(t.envFields.length>0)a=`
          <div style="margin-top: 14px; margin-bottom: 6px; font-weight: 700; font-size: 11px; text-transform: uppercase; color: var(--amber-400); letter-spacing: 0.5px;">
            Environment Variables &amp; API Keys
          </div>
          ${t.envFields.map((d)=>`
            <div class="form-group">
              <label class="form-label">${v(d.label)} ${d.required?'<span style="color: var(--red-400);">*</span>':"(Optional)"}</label>
              <input type="password" class="form-input tmpl-env-input" data-key="${v(d.key)}" placeholder="${v(d.placeholder||"")}">
              ${d.description?`<div style="font-size: 10.5px; color: var(--text-dim); margin-top: 3px;">${v(d.description)}</div>`:""}
            </div>
          `).join("")}
        `;n.innerHTML=`
        <div class="form-group">
          <label class="form-label">Server Identifier (Name)</label>
          <input type="text" class="form-input" id="cfg-srv-id" value="${v(t.id)}">
        </div>
        <div class="form-group">
          <label class="form-label">Command Line Arguments</label>
          <input type="text" class="form-input" id="cfg-srv-args" value="${v(t.defaultArgs.join(" "))}" placeholder="${v(t.argsPlaceholder||"")}">
          <div style="font-size: 10.5px; color: var(--text-dim); margin-top: 3px;">Executable: <code>${v(t.command)}</code></div>
        </div>
        ${a}
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
      `}}async submitTemplateServer(){if(!this.selectedTemplate)return;let e=this.selectedTemplate,t=document.getElementById("cfg-srv-id")?.value.trim(),i=document.getElementById("cfg-srv-args")?.value.trim();if(!t){alert("Server identifier is required");return}let r=i?i.split(/\s+/).filter(Boolean):[],o={},n=document.querySelectorAll(".tmpl-env-input");for(let f of Array.from(n)){let _=f.getAttribute("data-key"),h=f.value.trim(),k=e.envFields.find(($)=>$.key===_);if(k?.required&&!h){alert(`Required field '${k.label}' is missing.`);return}if(_&&h)o[_]=h}let a={command:e.command,args:r};if(Object.keys(o).length>0)a.env=o;let d=document.getElementById("cfg-srv-ft")?.value.trim(),p=document.getElementById("cfg-srv-cd")?.value.trim(),l=document.getElementById("cfg-srv-autorestart")?.value,g=document.getElementById("cfg-srv-maxrestarts")?.value.trim();if(d||p||l||g)a.resilience={failureThreshold:d?Number(d):3,cooldownMs:p?Number(p):30000,autoRestart:l!=="false",maxRestarts:g?Number(g):5};let b=await u.upsertServer(t,a);if(b.ok)this.closeModals(),await this.refreshData();else alert(`Failed to save server: ${b.error}`)}async openImportModal(){this.closeModals();let e=document.getElementById("modal-import");if(e)e.classList.add("active");let t=document.getElementById("modal-eco-list");if(!t)return;t.innerHTML='<div style="color: var(--text-dim); padding: 12px; text-align: center;">Scanning IDE configs...</div>';try{let i=await u.getEcosystemSources();if(i.sources&&i.sources.length>0)t.innerHTML=i.sources.map((r)=>`
          <label style="display: flex; align-items: center; gap: 10px; background: var(--surface); padding: 10px 14px; border-radius: var(--radius-sm); border: 1px solid var(--border); cursor: pointer;">
            <input type="checkbox" class="eco-checkbox" value="${r.path}" checked>
            <div>
              <div style="font-weight: 600; color: var(--text-main);">${r.name}</div>
              <div style="font-size: 11px; color: var(--text-dim);">${r.server_count} servers (${r.servers.join(", ")})</div>
            </div>
          </label>
        `).join("");else t.innerHTML='<div style="color: var(--text-dim); padding: 12px; text-align: center;">No external MCP configuration files found on this system.</div>'}catch{t.innerHTML='<div style="color: var(--red-400); padding: 12px; text-align: center;">Failed to scan ecosystem sources.</div>'}}async submitImport(){let e=document.querySelectorAll(".eco-checkbox:checked");if(e.length===0){alert("No sources selected");return}for(let t of Array.from(e))await u.importConfig(t.value,!1);this.closeModals(),await this.refreshData()}handleAliasTargetInput(e){let t=document.getElementById("alias-suggestions-dropdown");if(!t)return;let i=(e||"").trim().toLowerCase();if(i.length<2){t.style.display="none";return}let o=s.getState().capabilities.filter((n)=>n.id.toLowerCase().includes(i)||n.summary&&n.summary.toLowerCase().includes(i)||n.description&&n.description.toLowerCase().includes(i)||n.server&&n.server.toLowerCase().includes(i)).slice(0,8);if(o.length===0){t.style.display="none";return}t.innerHTML=o.map((n)=>`
      <div style="padding: 8px 12px; cursor: pointer; border-bottom: 1px solid var(--border-subtle); display: flex; justify-content: space-between; align-items: center; transition: background 0.1s;"
           onmouseover="this.style.background='var(--surface-hover)'"
           onmouseout="this.style.background='transparent'"
           onmousedown="window.app.selectAliasSuggestion('${v(n.id)}')">
        <div>
          <div style="font-weight: 700; color: var(--text-main);">${v(n.id)}</div>
          <div style="font-size: 10.5px; color: var(--text-dim); margin-top: 2px;">${v(n.summary||n.description||"")}</div>
        </div>
        <span style="font-size: 10px; color: var(--cyan-400);">${v(n.server||"local")}</span>
      </div>
    `).join(""),t.style.display="block"}selectAliasSuggestion(e){let t=document.getElementById("alias-target");if(t)t.value=e;this.hideAliasDropdown()}hideAliasDropdown(){let e=document.getElementById("alias-suggestions-dropdown");if(e)e.style.display="none"}async createAlias(){let e=document.getElementById("alias-kind")?.value,t=document.getElementById("alias-name")?.value.trim(),i=document.getElementById("alias-target")?.value.trim();if(!t||!i){alert("Please provide both alias name and canonical target");return}await u.updateAlias(e,t,i),await this.refreshData()}async deleteAlias(e,t){await u.updateAlias(e,t,void 0),await this.refreshData()}async reloadFromDisk(){try{let e=await u.reloadConfig();if(e.ok){let t="Hot-reload completed successfully!";if(e.mounted&&e.mounted.length>0)t+=`
Mounted: ${e.mounted.join(", ")}`;if(e.unmounted&&e.unmounted.length>0)t+=`
Unmounted: ${e.unmounted.join(", ")}`;if(e.warnings&&e.warnings.length>0)t+=`
Warnings:
${e.warnings.join(`
`)}`;alert(t)}else alert(`Hot-reload failed: ${e.error||"Unknown error"}`)}catch(e){alert(`Error reaching daemon: ${e.message}`)}await this.refreshData()}closeModals(){document.querySelectorAll(".modal-backdrop").forEach((e)=>e.classList.remove("active"))}}function v(e){return String(e||"").replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;")}var G=new J;window.app=G;window.addEventListener("DOMContentLoaded",()=>G.init());
