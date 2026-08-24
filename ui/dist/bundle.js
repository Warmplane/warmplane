class H{state={configPath:"mcp_servers.json",config:{mcpServers:{}},serverStatuses:{},circuitBreakers:[],capabilities:[],resources:[],prompts:[],catalogEvents:[],approvals:[],auditEvents:[],auditTotal:0,auditFilters:{search:"",status:"all",eventType:"all",serverId:"all",limit:25,offset:0},auditSelectedEvent:null,auditStats:null,auditVerification:null,selectedCapabilityId:null,selectedResourceId:null,selectedPromptId:null,playgroundMode:"tools",playgroundArgs:{},isExecuting:!1,activeRequestId:null,isBatchModalOpen:!1,batchSteps:[{id:"step_1",capability_id:"",argsJson:"{}",continue_on_error:!1},{id:"step_2",capability_id:"",argsJson:"{}",continue_on_error:!0}],activeTab:"overview",activeProfile:null,eventLogs:[],executionResult:null,resourceReadResult:null,promptGetResult:null,metrics:{totalCatalogRequests:0,totalEtagHits:0,totalToolCalls:0,totalToolDurationUs:0}};listeners=[];getState(){return this.state}setState(e){this.state={...this.state,...e},this.listeners.forEach((t)=>t(this.state))}subscribe(e){return this.listeners.push(e),()=>{this.listeners=this.listeners.filter((t)=>t!==e)}}addEventLog(e,t,a,i){let r=[{time:new Date().toLocaleTimeString(),method:e,target:t,status:a,latency:i},...this.state.eventLogs].slice(0,50);this.setState({eventLogs:r})}}var l=new H;class q{baseUrl;constructor(e=""){this.baseUrl=e}async getConfig(){return(await fetch(`${this.baseUrl}/v1/config`)).json()}async listCapabilities(e){let t={};if(e)t["X-Warmplane-Profile"]=e;return(await fetch(`${this.baseUrl}/v1/capabilities`,{headers:t})).json()}async listResources(e){let t={};if(e)t["X-Warmplane-Profile"]=e;return(await fetch(`${this.baseUrl}/v1/resources`,{headers:t})).json()}async readResource(e,t){let a=performance.now(),i={"Content-Type":"application/json"};if(t)i["X-Warmplane-Profile"]=t;let s=await fetch(`${this.baseUrl}/v1/resources/read`,{method:"POST",headers:i,body:JSON.stringify(e)}),r=performance.now()-a,n=await s.json();return{status:s.status,durationMs:r,data:n}}async listPrompts(e){let t={};if(e)t["X-Warmplane-Profile"]=e;return(await fetch(`${this.baseUrl}/v1/prompts`,{headers:t})).json()}async getPrompt(e,t){let a=performance.now(),i={"Content-Type":"application/json"};if(t)i["X-Warmplane-Profile"]=t;let s=await fetch(`${this.baseUrl}/v1/prompts/get`,{method:"POST",headers:i,body:JSON.stringify(e)}),r=performance.now()-a,n=await s.json();return{status:s.status,durationMs:r,data:n}}async getCatalogEvents(e){let t=e?`?after=${encodeURIComponent(e)}`:"";return(await fetch(`${this.baseUrl}/v1/catalog/events${t}`)).json()}async callCapability(e,t){let a=performance.now(),i={"Content-Type":"application/json"};if(t)i["X-Warmplane-Profile"]=t;let s=await fetch(`${this.baseUrl}/v1/tools/call`,{method:"POST",headers:i,body:JSON.stringify(e)}),r=performance.now()-a,n=await s.json();return{status:s.status,durationMs:r,data:n}}async batchCallCapabilities(e,t){let a=performance.now(),i={"Content-Type":"application/json"};if(t)i["X-Warmplane-Profile"]=t;let s=await fetch(`${this.baseUrl}/v1/tools/batch_call`,{method:"POST",headers:i,body:JSON.stringify({steps:e})}),r=performance.now()-a,n=await s.json();return{status:s.status,durationMs:r,data:n}}async cancelOperation(e){return(await fetch(`${this.baseUrl}/v1/operations/${encodeURIComponent(e)}/cancel`,{method:"POST"})).json()}async completeArgument(e){return(await fetch(`${this.baseUrl}/v1/completion/complete`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify(e)})).json()}async upsertServer(e,t){return(await fetch(`${this.baseUrl}/v1/config/servers`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({name:e,server:t})})).json()}async deleteServer(e){return(await fetch(`${this.baseUrl}/v1/config/servers/${encodeURIComponent(e)}`,{method:"DELETE"})).json()}async upsertProfile(e,t,a){return(await fetch(`${this.baseUrl}/v1/config/profiles`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({name:e,servers:t,description:a})})).json()}async deleteProfile(e){return(await fetch(`${this.baseUrl}/v1/config/profiles/${encodeURIComponent(e)}`,{method:"DELETE"})).json()}async getEcosystemSources(){return(await fetch(`${this.baseUrl}/v1/config/ecosystem`)).json()}async importConfig(e,t=!1){return(await fetch(`${this.baseUrl}/v1/config/import`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({source_path:e,overwrite:t})})).json()}async savePolicy(e){let t={allow:e.allow||[],deny:e.deny||[],redactKeys:e.redact_keys||e.redactKeys||[],requireApproval:e.require_approval||e.requireApproval||[],approvalTimeoutSecs:e.approvalTimeoutSecs||e.approval_timeout_secs||300,webhook:e.webhook};return(await fetch(`${this.baseUrl}/v1/config/policy`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify(t)})).json()}async listApprovals(){return(await fetch(`${this.baseUrl}/v1/approvals`)).json()}async approveTicket(e,t,a){return(await fetch(`${this.baseUrl}/v1/approvals/${encodeURIComponent(e)}/approve`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({operator:t,modified_args:a})})).json()}async rejectTicket(e,t,a){return(await fetch(`${this.baseUrl}/v1/approvals/${encodeURIComponent(e)}/reject`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({operator:t,reason:a})})).json()}async updateAlias(e,t,a){return(await fetch(`${this.baseUrl}/v1/config/alias`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({kind:e,alias:t,target:a})})).json()}async reloadConfig(){return(await fetch(`${this.baseUrl}/v1/config/reload`,{method:"POST"})).json()}async listAuditEvents(e){let t=new URLSearchParams;if(e?.actor_id)t.set("actor_id",e.actor_id);if(e?.server_id&&e.server_id!=="all")t.set("server_id",e.server_id);if(e?.capability_id)t.set("capability_id",e.capability_id);if(e?.event_type&&e.event_type!=="all")t.set("event_type",e.event_type);if(e?.status&&e.status!=="all")t.set("status",e.status);if(e?.trace_id)t.set("trace_id",e.trace_id);if(e?.request_id)t.set("request_id",e.request_id);if(e?.search)t.set("search",e.search);if(e?.limit)t.set("limit",String(e.limit));if(e?.offset!==void 0)t.set("offset",String(e.offset));let a=t.toString();return(await fetch(`${this.baseUrl}/v1/audit/events${a?`?${a}`:""}`)).json()}getAuditExportUrl(e,t="csv"){let a=new URLSearchParams;if(a.set("format",t),e?.actor_id)a.set("actor_id",e.actor_id);if(e?.server_id&&e.server_id!=="all")a.set("server_id",e.server_id);if(e?.capability_id)a.set("capability_id",e.capability_id);if(e?.event_type&&e.event_type!=="all")a.set("event_type",e.event_type);if(e?.status&&e.status!=="all")a.set("status",e.status);if(e?.trace_id)a.set("trace_id",e.trace_id);if(e?.request_id)a.set("request_id",e.request_id);if(e?.search)a.set("search",e.search);return`${this.baseUrl}/v1/audit/export?${a.toString()}`}async verifyAuditChain(){return(await fetch(`${this.baseUrl}/v1/audit/verify`)).json()}async getAuditStats(){return(await fetch(`${this.baseUrl}/v1/audit/stats`)).json()}}var f=new q;function N(){let e=l.getState(),t=e.config.mcpServers||{},a=Object.keys(t),i=a.length,s="";if(a.length===0)s=`
      <div style="grid-column: 1 / -1; padding: 32px; text-align: center; color: var(--text-dim); background: var(--surface-card); border-radius: var(--radius-md); border: 1px dashed var(--border);">
        <div style="font-size: 14px; color: var(--text-main); font-weight: 600; margin-bottom: 6px;">No Upstream MCP Servers Connected</div>
        <div style="font-size: 12px; margin-bottom: 16px;">Initialize connections by adding a server or syncing existing IDE configurations.</div>
        <div style="display: flex; gap: 8px; justify-content: center;">
          <button class="btn btn-primary" onclick="window.app.openTemplateCatalog()">✨ Browse Templates</button>
          <button class="btn btn-ghost" onclick="window.app.openAddServerModal()">+ Add Custom</button>
          <button class="btn btn-ghost" onclick="window.app.openImportModal()">Sync from IDEs</button>
        </div>
      </div>
    `;else s=a.map((w)=>{let k=t[w],R=k.command?"stdio":"http / sse",z=k.command?`${k.command} ${(k.args||[]).join(" ")}`:k.url,C=e.serverStatuses[w]||{status:"connected",protocol_version:"2026-07-28"},x=C.status==="degraded",O=C.status==="error"||C.status==="disconnected",P=x?"var(--amber-400)":O?"var(--red-400)":"var(--green-400)";return`
        <div class="bento-card col-4" style="background: var(--surface); border: 1px solid var(--border);">
          <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
            <span style="font-weight: 700; color: var(--text-main); display: flex; align-items: center; gap: 6px;">
              <span style="width: 8px; height: 8px; border-radius: 50%; background: ${P}; display: inline-block;"></span>
              ${T(w)}
            </span>
            <span class="brand-badge">${R}</span>
          </div>
          <div style="font-family: var(--ff-mono); font-size: 11.5px; color: var(--text-dim); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; margin-bottom: 12px;" title="${T(z||"")}">
            ${T(z||"")}
          </div>
          <div style="display: flex; justify-content: space-between; font-size: 11px; color: var(--text-muted); border-top: 1px solid var(--border); padding-top: 8px;">
            <span>Status: <strong style="color: ${P};">${T(C.status)}</strong></span>
            <span>Protocol: ${C.protocol_version}</span>
          </div>
        </div>
      `}).join("");let r=e.eventLogs.length===0?`
    <div class="feed-row" style="grid-template-columns: 80px 100px 1fr 100px 80px;">
      <span style="color: var(--text-dim);">ready</span>
      <span style="color: var(--cyan-400); font-weight: 600;">SSE</span>
      <span style="color: var(--text-main);">/v1/resources/updates stream active</span>
      <span style="color: var(--green-400);">CONNECTED</span>
      <span style="color: var(--amber-300); text-align: right;">0.0ms</span>
    </div>
  `:e.eventLogs.map((w)=>`
    <div class="feed-row" style="grid-template-columns: 80px 100px 1fr 100px 80px;">
      <span style="color: var(--text-dim);">${T(w.time)}</span>
      <span style="color: var(--cyan-400); font-weight: 600;">${T(w.method)}</span>
      <span style="color: var(--text-main); font-family: var(--ff-mono);">${T(w.target)}</span>
      <span style="color: var(--green-400);">${T(w.status)}</span>
      <span style="color: var(--amber-300); text-align: right;">${T(w.latency)}</span>
    </div>
  `).join(""),n=e.metrics,o=n.totalCatalogRequests,c=n.totalEtagHits,d=o>0?`${(c/o*100).toFixed(1)}%`:"0.0%",p=o>0?`${c} of ${o} requests served via HTTP 304`:"Waiting for client requests",g=n.totalToolCalls,m=g>0?`${(n.totalToolDurationUs/g/1000).toFixed(1)}ms`:"0.0ms",y=g>0?`${g} tool executions processed`:"Local worker task queues warm",h=Object.keys(e.config.capabilityAliases||{}).length+Object.keys(e.config.resourceAliases||{}).length+Object.keys(e.config.promptAliases||{}).length,S=h>0?`${h*18}B / call`:"0B",E=h>0?`${h} active facade aliases pruning prompt size`:"Configure aliases in Studio to reduce prompt size";return`
    <div class="bento-grid">
      <div class="bento-card col-3">
        <div class="stat-label">Token Savings Rate</div>
        <div class="stat-value" style="color: var(--amber-300);">${S}</div>
        <div class="stat-sub">${E}</div>
      </div>
      <div class="bento-card col-3">
        <div class="stat-label">ETag Cache Hit Rate</div>
        <div class="stat-value" style="color: var(--cyan-400);">${d}</div>
        <div class="stat-sub">${p}</div>
      </div>
      <div class="bento-card col-3">
        <div class="stat-label">Connected Upstreams</div>
        <div class="stat-value" style="color: var(--green-400);">${i} Active</div>
        <div class="stat-sub">${i>0?"Persistent worker task channels":"No active upstream servers"}</div>
      </div>
      <div class="bento-card col-3">
        <div class="stat-label">Avg Execution Latency</div>
        <div class="stat-value">${m}</div>
        <div class="stat-sub">${y}</div>
      </div>
    </div>

    <div style="display: flex; justify-content: space-between; align-items: center; margin: 24px 0 12px;">
      <div style="font-size: 15px; font-weight: 700; color: var(--text-main);">Connected Upstream Servers</div>
      <button class="btn btn-ghost" onclick="window.app.switchTab('servers')">Manage All (${i}) →</button>
    </div>

    <div class="bento-grid" style="margin-bottom: 24px;">
      ${s}
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
        ${r}
      </div>
    </div>
  `}function T(e){return String(e||"").replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;")}function F(){let e=l.getState(),t=e.config.mcpServers||{},a=Object.keys(t),i="";if(a.length===0)i=`
      <div style="padding: 40px; text-align: center; color: var(--text-dim); background: var(--surface-card); border-radius: var(--radius-md); border: 1px dashed var(--border);">
        <div style="font-size: 15px; color: var(--text-main); font-weight: 600; margin-bottom: 8px;">No Servers Configured in ${I(e.configPath)}</div>
        <p style="font-size: 12px; margin-bottom: 20px; max-width: 480px; margin-left: auto; margin-right: auto;">
          Warmplane bridges local tools and remote MCP servers into one unified facade. Add your first server or import existing configs from Claude Desktop or Cursor.
        </p>
        <div style="display: flex; gap: 8px; justify-content: center;">
          <button class="btn btn-primary" onclick="window.app.openTemplateCatalog()">✨ Browse Templates</button>
          <button class="btn btn-ghost" onclick="window.app.openAddServerModal()">+ Add Custom</button>
          <button class="btn btn-ghost" onclick="window.app.openImportModal()">Sync from IDEs</button>
        </div>
      </div>
    `;else i=a.map((s)=>{let r=t[s],n=r.command?"stdio":"http / sse",o=r.command?`${r.command} ${(r.args||[]).join(" ")}`:r.url,c=e.serverStatuses[s]||{status:"connected",protocol_version:"2026-07-28"},d=r.env?Object.keys(r.env).map((w)=>`${w}=***`).join(", "):"None",p=(e.circuitBreakers||[]).find((w)=>w.server_id===s),g='<span class="brand-badge" style="color: var(--green-400); border-color: rgba(52, 211, 153, 0.25);">Circuit: CLOSED</span>';if(p){if(p.state==="open")g=`<span class="brand-badge" style="color: var(--red-400); border-color: rgba(248, 113, 113, 0.4); background: rgba(248, 113, 113, 0.1);">Circuit: OPEN (${p.consecutive_failures} failures)</span>`;else if(p.state==="half_open")g=`<span class="brand-badge" style="color: var(--amber-300); border-color: rgba(251, 191, 36, 0.4); background: rgba(251, 191, 36, 0.1);">Circuit: HALF-OPEN (${p.consecutive_successes} probe)</span>`}let m=r.resilience||e.config.resilience,y=m?`FT: ${m.failureThreshold||3} · Cooldown: ${(m.cooldownMs||30000)/1000}s · AutoRestart: ${m.autoRestart!==!1?"ON":"OFF"}`:"Default Resilience",h=c.status==="degraded",S=c.status==="error"||c.status==="disconnected",E=h?"var(--amber-400)":S?"var(--red-400)":"var(--green-400)";return`
        <div class="bento-card" style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px;">
          <div>
            <div style="display: flex; align-items: center; gap: 8px; margin-bottom: 4px;">
              <span style="width: 8px; height: 8px; border-radius: 50%; background: ${E}; display: inline-block;"></span>
              <span style="font-size: 15px; font-weight: 700; color: var(--text-main);">${I(s)}</span>
              <span class="brand-badge">${n}</span>
              <span class="brand-badge" style="color: ${E}; border-color: rgba(245, 158, 11, 0.3);">Status: ${I(c.status)}</span>
              <span class="brand-badge" style="color: var(--cyan-400); border-color: rgba(34, 211, 238, 0.25);">Protocol: ${c.protocol_version}</span>
              ${g}
            </div>
            <div style="font-family: var(--ff-mono); font-size: 12px; color: var(--text-muted); margin-top: 4px;">
              ${r.command?"Command: ":"URL: "}<code>${I(o||"")}</code>
            </div>
            <div style="display: flex; gap: 14px; font-family: var(--ff-mono); font-size: 11px; color: var(--text-dim); margin-top: 4px;">
              <span>\uD83D\uDEE1️ ${I(y)}</span>
              ${r.env&&Object.keys(r.env).length>0?`<span>Env: ${I(d)}</span>`:""}
            </div>
          </div>
          <div style="display: flex; gap: 8px;">
            <button class="btn btn-ghost" style="padding: 4px 10px; font-size: 11.5px;" onclick="window.app.openEditServerModal('${I(s)}')">✏️ Edit</button>
            <button class="btn btn-danger" style="padding: 4px 10px; font-size: 11.5px;" onclick="window.app.deleteServer('${I(s)}')">Remove</button>
          </div>
        </div>
      `}).join("");return`
    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 18px;">
      <div>
        <div style="font-size: 16px; font-weight: 700; color: var(--text-main);">Configured MCP Upstream Servers</div>
        <div style="font-size: 11px; color: var(--text-dim);">Active configuration file: <code>${I(e.configPath)}</code></div>
      </div>
      <div style="display: flex; gap: 8px;">
        <button class="btn btn-ghost" onclick="window.app.reloadFromDisk()">⟳ Reload Config</button>
      </div>
    </div>

    ${i}
  `}function I(e){return String(e||"").replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;")}function D(){let e=l.getState(),t=e.playgroundMode||"tools",a=e.capabilities||[],i=e.resources||[],s=e.prompts||[],r=`
    <div style="display: flex; gap: 8px; margin-bottom: 12px; align-items: center; justify-content: space-between;">
      <div style="display: inline-flex; padding: 3px; background: rgba(0,0,0,0.3); border: 1px solid var(--border); border-radius: var(--radius-sm);">
        <button 
          class="btn ${t==="tools"?"btn-primary":"btn-ghost"}" 
          style="padding: 4px 12px; font-size: 11.5px; height: 28px;"
          onclick="window.app.setPlaygroundMode('tools')"
        >
          \uD83D\uDEE0️ Tools (${a.length})
        </button>
        <button 
          class="btn ${t==="resources"?"btn-primary":"btn-ghost"}" 
          style="padding: 4px 12px; font-size: 11.5px; height: 28px;"
          onclick="window.app.setPlaygroundMode('resources')"
        >
          \uD83D\uDCC4 Resources (${i.length})
        </button>
        <button 
          class="btn ${t==="prompts"?"btn-primary":"btn-ghost"}" 
          style="padding: 4px 12px; font-size: 11.5px; height: 28px;"
          onclick="window.app.setPlaygroundMode('prompts')"
        >
          \uD83D\uDCAC Prompts (${s.length})
        </button>
      </div>

      <div style="font-size: 11.5px; color: var(--text-dim);">
        ${t==="tools"?"Interactive Tool Caller & Context Distillation":t==="resources"?"Live MCP Resource Inspector & Reader":"Prompt Template Studio & Variable Binder"}
      </div>
    </div>
  `;if(t==="resources")return`
      ${r}
      ${re(e)}
    `;if(t==="prompts")return`
      ${r}
      ${ae(e)}
    `;return`
    ${r}
    ${se(e)}
    ${e.isBatchModalOpen?ie(e):""}
  `}function M(e,t=!1){if(!e||!e.properties)return{};let a=e.properties||{},i=Array.isArray(e.required)?e.required:[],s={};for(let[r,n]of Object.entries(a)){let o=i.includes(r);if(t&&!o)continue;if(n.default!==void 0)s[r]=n.default;else if(Array.isArray(n.enum)&&n.enum.length>0)s[r]=n.enum[0];else if(n.examples&&Array.isArray(n.examples)&&n.examples.length>0)s[r]=n.examples[0];else if(n.example!==void 0)s[r]=n.example;else switch(n.type||"string"){case"string":s[r]=o?`sample_${r}`:"";break;case"number":case"integer":s[r]=0;break;case"boolean":s[r]=!0;break;case"array":s[r]=[];break;case"object":s[r]={};break;default:s[r]=`sample_${r}`}}return s}function se(e){let t=e.capabilities||[],a=e.selectedCapabilityId||(t.length>0?t[0].id:null),i=t.find((m)=>m.id===a),s=!!e.isExecuting,r="";if(t.length===0)r=`
      <div style="padding: 24px 16px; text-align: center; color: var(--text-dim); font-size: 11.5px;">
        No tools or capabilities discovered from connected servers.
      </div>
    `;else r=t.map((m)=>`
        <div class="cap-item ${m.id===a?"active":""}" onclick="window.app.selectCapability('${u(m.id)}')">
          <div style="display: flex; justify-content: space-between; align-items: center;">
            <span style="font-weight: 600; color: var(--text-main); font-family: var(--ff-mono); font-size: 12px;">${u(m.id)}</span>
            <span style="font-size: 10px; color: var(--green-400);">${u(m.mode||"read")}</span>
          </div>
          <div style="font-size: 11px; color: var(--text-dim); margin-top: 2px;">server: ${u(m.server||"local")}</div>
        </div>
      `).join("");let n=i?.input_schema,o=n?.properties||{},c=Array.isArray(n?.required)?n.required:[],d=Object.entries(o),p="";if(d.length>0)p=`
      <div style="display: flex; flex-wrap: wrap; gap: 6px; margin-bottom: 8px; align-items: center;">
        <span style="font-size: 10px; font-weight: 700; color: var(--text-dim); text-transform: uppercase;">Fields:</span>
        ${d.map(([m,y])=>{let h=c.includes(m),S=y.type||(y.enum?"enum":"any"),E=h?"rgba(239, 68, 68, 0.15)":"rgba(148, 163, 184, 0.1)",w=h?"var(--red-400)":"var(--text-muted)",k=h?"rgba(239, 68, 68, 0.3)":"var(--border)",R=y.description?` - ${y.description}`:"";return`
            <button 
              type="button" 
              class="btn" 
              style="padding: 2px 7px; font-size: 10.5px; font-family: var(--ff-mono); background: ${E}; color: ${w}; border: 1px solid ${k}; border-radius: var(--radius-sm);" 
              title="Click to insert '${m}' (${S}${R})" 
              onclick="window.app.insertPlaygroundArgKey('${u(m)}', '${u(S)}', ${u(JSON.stringify(y.default??null))})"
            >
              + ${u(m)} <span style="font-size: 9px; opacity: 0.7;">(${S}${h?" *":""})</span>
            </button>
          `}).join("")}
      </div>
    `;let g="{}";if(a&&e.playgroundArgs&&e.playgroundArgs[a]!==void 0)g=e.playgroundArgs[a];else{let m=M(n,!1);g=JSON.stringify(m,null,2)}return`
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
              ${u(i?i.id:"No Capability Selected")}
            </div>
            <div style="font-size: 11.5px; color: var(--text-dim);" id="pg-selected-desc">
              ${u(i?i.summary||i.description:"Connect servers to inspect and execute tools")}
            </div>
          </div>
          
          <div style="display: flex; align-items: center; gap: 10px;">
            ${s?`
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
              <button class="btn btn-primary" onclick="window.app.executePlaygroundTool()" ${i?"":"disabled"}>
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polygon points="5 3 19 12 5 21 5 3"></polygon></svg>
                Execute Capability
              </button>
            `}
          </div>
        </div>

        <div style="flex: 1; display: grid; grid-template-columns: 1fr 1fr; overflow: hidden;">
          <!-- Request Builder -->
          <div style="padding: 16px; border-right: 1px solid var(--border); overflow-y: auto;">
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
              <label class="form-label" style="margin: 0;">Arguments JSON</label>
              <div style="display: flex; gap: 6px;">
                <button type="button" class="btn btn-ghost" style="padding: 2px 7px; font-size: 10.5px;" title="Fill sample payload from schema" onclick="window.app.fillPlaygroundSampleArgs(false)">✨ Sample Template</button>
                ${c.length>0?`
                  <button type="button" class="btn btn-ghost" style="padding: 2px 7px; font-size: 10.5px;" title="Fill only required schema fields" onclick="window.app.fillPlaygroundSampleArgs(true)">\uD83E\uDDF9 Required Only</button>
                `:""}
                <button type="button" class="btn btn-ghost" style="padding: 2px 7px; font-size: 10.5px;" title="Format JSON" onclick="window.app.formatPlaygroundArgs()">\uD83D\uDCCB Format</button>
                <button type="button" class="btn btn-ghost" style="padding: 2px 8px; font-size: 11px;" onclick="window.app.openBatchModal()">⚡ Pipeline Builder</button>
              </div>
            </div>

            ${p}

            <textarea class="form-textarea" rows="7" id="pg-args-input" oninput="window.app.updatePlaygroundArgs(this.value)">${u(g)}</textarea>

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
            ${i&&i.input_schema?`
              <div style="margin-top: 14px;">
                <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 4px;">
                  <label class="form-label" style="margin: 0;">Input JSON Schema</label>
                  <span style="font-size: 10px; color: var(--text-dim); font-family: var(--ff-mono);">${d.length} field${d.length===1?"":"s"} (${c.length} required)</span>
                </div>
                <pre style="background: var(--surface); padding: 10px; border-radius: var(--radius-sm); border: 1px solid var(--border); font-size: 11px; color: var(--text-muted); max-height: 140px; overflow-y: auto;">${u(JSON.stringify(i.input_schema,null,2))}</pre>
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
            <pre id="pg-response-json" style="flex: 1; background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 14px; color: var(--amber-300); font-size: 11.5px; overflow-y: auto; margin: 0; white-space: pre-wrap; word-break: break-word;">${e.executionResult?u(JSON.stringify(e.executionResult.data,null,2)):"// Response envelope output will be formatted here"}</pre>
          </div>
        </div>
      </div>
    </div>
  `}function re(e){let t=e.resources||[],a=e.selectedResourceId||(t.length>0?t[0].id:null),i=t.find((n)=>n.id===a),s=e.resourceReadResult,r="";if(t.length===0)r=`
      <div style="padding: 24px 16px; text-align: center; color: var(--text-dim); font-size: 11.5px;">
        No resources exposed by connected MCP servers.
      </div>
    `;else r=t.map((n)=>{let o=n.id===a?"active":"",c=n.uri?n.uri.split(":")[0]:"res";return`
        <div class="cap-item ${o}" onclick="window.app.selectResource('${u(n.id)}')">
          <div style="display: flex; justify-content: space-between; align-items: center;">
            <span style="font-weight: 600; color: var(--text-main); font-family: var(--ff-mono); font-size: 12px;">${u(n.name||n.id)}</span>
            <span class="badge" style="font-size: 9.5px; background: rgba(56, 189, 248, 0.15); color: var(--cyan-400);">${u(c)}</span>
          </div>
          <div style="font-size: 11px; color: var(--text-dim); margin-top: 2px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">${u(n.uri)}</div>
          <div style="display: flex; justify-content: space-between; font-size: 10px; color: var(--text-muted); margin-top: 4px;">
            <span>server: ${u(n.server||"local")}</span>
            <span>${u(n.mime_type||"text/plain")}</span>
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
          ${r}
        </div>
      </div>

      <!-- Right Panel: Resource Content Reader & Metadata Inspector -->
      <div style="background: var(--surface-card); border: 1px solid var(--border); border-radius: var(--radius-md); display: flex; flex-direction: column; overflow: hidden;">
        <div style="padding: 14px 18px; border-bottom: 1px solid var(--border); display: flex; justify-content: space-between; align-items: center;">
          <div>
            <div style="font-size: 15px; font-weight: 700; color: var(--text-main); font-family: var(--ff-mono);">
              ${u(i?i.name||i.id:"No Resource Selected")}
            </div>
            <div style="font-size: 11.5px; color: var(--cyan-400); font-family: var(--ff-mono);">
              ${u(i?i.uri:"Select a resource from the list to read live content")}
            </div>
          </div>
          <button class="btn btn-primary" onclick="window.app.executeReadResource()" ${i?"":"disabled"}>
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M2 12h20M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"></path></svg>
            Read Resource Content
          </button>
        </div>

        <div style="flex: 1; display: grid; grid-template-columns: 1fr 1fr; overflow: hidden;">
          <!-- Request / Distillation Parameters -->
          <div style="padding: 16px; border-right: 1px solid var(--border); overflow-y: auto;">
            ${i?`
              <div style="background: rgba(0,0,0,0.25); padding: 12px; border-radius: var(--radius-sm); border: 1px solid var(--border); margin-bottom: 14px;">
                <div style="font-size: 11px; font-weight: 600; color: var(--text-muted); text-transform: uppercase; margin-bottom: 6px;">Resource Metadata</div>
                <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 8px; font-size: 11.5px;">
                  <div><span style="color: var(--text-muted);">Server:</span> <strong style="color: var(--text-main);">${u(i.server)}</strong></div>
                  <div><span style="color: var(--text-muted);">MIME Type:</span> <strong style="color: var(--text-main);">${u(i.mime_type||"text/plain")}</strong></div>
                </div>
                ${i.description?`
                  <div style="margin-top: 8px; font-size: 11.5px; color: var(--text-dim); border-top: 1px solid rgba(255,255,255,0.05); padding-top: 6px;">
                    ${u(i.description)}
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
              <span style="font-size: 11px; font-weight: 600; color: ${s?s.status===200?"var(--green-400)":"var(--red-400)":"var(--text-dim)"}; font-family: var(--ff-mono);">
                ${s?`HTTP ${s.status} · ${s.durationMs.toFixed(1)}ms`:"READY"}
              </span>
            </div>
            <pre style="flex: 1; background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 14px; color: var(--cyan-400); font-size: 11.5px; overflow-y: auto; margin: 0; white-space: pre-wrap; word-break: break-word;">${s?u(JSON.stringify(s.data,null,2)):'// Click "Read Resource Content" to inspect live payload'}</pre>
          </div>
        </div>
      </div>
    </div>
  `}function ae(e){let t=e.prompts||[],a=e.selectedPromptId||(t.length>0?t[0].id:null),i=t.find((o)=>o.id===a),s=e.promptGetResult,r="";if(t.length===0)r=`
      <div style="padding: 24px 16px; text-align: center; color: var(--text-dim); font-size: 11.5px;">
        No prompt templates registered by connected MCP servers.
      </div>
    `;else r=t.map((o)=>{let c=o.id===a?"active":"",d=o.arguments?o.arguments.length:0;return`
        <div class="cap-item ${c}" onclick="window.app.selectPrompt('${u(o.id)}')">
          <div style="display: flex; justify-content: space-between; align-items: center;">
            <span style="font-weight: 600; color: var(--text-main); font-family: var(--ff-mono); font-size: 12px;">${u(o.name||o.id)}</span>
            <span class="badge" style="font-size: 9.5px; background: rgba(168, 85, 247, 0.15); color: var(--purple-400);">${d} args</span>
          </div>
          <div style="font-size: 11px; color: var(--text-dim); margin-top: 2px;">${u(o.description||o.title||"Prompt template")}</div>
          <div style="font-size: 10px; color: var(--text-muted); margin-top: 4px;">server: ${u(o.server||"local")}</div>
        </div>
      `}).join("");let n="";if(i&&i.arguments&&i.arguments.length>0)n=i.arguments.map((o)=>`
      <div class="form-group" style="margin-bottom: 12px;">
        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 4px;">
          <label class="form-label" style="margin: 0; font-family: var(--ff-mono);">${u(o.name)}</label>
          ${o.required?'<span class="badge" style="background: rgba(239, 68, 68, 0.15); color: var(--red-400); font-size: 9px;">REQUIRED</span>':'<span style="font-size: 10px; color: var(--text-dim);">optional</span>'}
        </div>
        ${o.description?`<div style="font-size: 11px; color: var(--text-dim); margin-bottom: 4px;">${u(o.description)}</div>`:""}
        <input type="text" class="form-input prompt-arg-input" data-arg-name="${u(o.name)}" placeholder="Enter ${u(o.name)}..." />
      </div>
    `).join("");else if(i)n=`
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
          ${r}
        </div>
      </div>

      <!-- Right Panel: Prompt Parameter Binder & Message Envelope Preview -->
      <div style="background: var(--surface-card); border: 1px solid var(--border); border-radius: var(--radius-md); display: flex; flex-direction: column; overflow: hidden;">
        <div style="padding: 14px 18px; border-bottom: 1px solid var(--border); display: flex; justify-content: space-between; align-items: center;">
          <div>
            <div style="font-size: 15px; font-weight: 700; color: var(--text-main); font-family: var(--ff-mono);">
              ${u(i?i.name||i.id:"No Prompt Selected")}
            </div>
            <div style="font-size: 11.5px; color: var(--text-dim);">
              ${u(i?i.description||i.title||"Bind variables and render messages":"Select a prompt from the list to test")}
            </div>
          </div>
          <button class="btn btn-primary" onclick="window.app.executeGetPrompt()" ${i?"":"disabled"}>
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
              <span style="font-size: 11px; font-weight: 600; color: ${s?s.status===200?"var(--green-400)":"var(--red-400)":"var(--text-dim)"}; font-family: var(--ff-mono);">
                ${s?`HTTP ${s.status} · ${s.durationMs.toFixed(1)}ms`:"READY"}
              </span>
            </div>
            <pre style="flex: 1; background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 14px; color: #c084fc; font-size: 11.5px; overflow-y: auto; margin: 0; white-space: pre-wrap; word-break: break-word;">${s?u(JSON.stringify(s.data,null,2)):'// Click "Render Prompt Messages" to view resolved system/user messages'}</pre>
          </div>
        </div>
      </div>
    </div>
  `}function ie(e){let t=e.capabilities||[],a=e.batchSteps||[];return`
    <div style="position: fixed; inset: 0; background: rgba(0,0,0,0.75); backdrop-filter: blur(4px); display: flex; align-items: center; justify-content: center; z-index: 1000; padding: 24px;" onclick="if(event.target === this) window.app.closeBatchModal()">
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
          ${a.map((s,r)=>{let n=t.find((y)=>y.id===s.capability_id),o=n?.input_schema,c=o?.properties||{},d=Array.isArray(o?.required)?o.required:[],p=Object.entries(c),g=t.map((y)=>`
      <option value="${u(y.id)}" ${y.id===s.capability_id?"selected":""}>
        ${u(y.id)} (${u(y.server||"local")})
      </option>
    `).join(""),m="";if(p.length>0)m=`
        <div style="display: flex; flex-wrap: wrap; gap: 4px; margin-top: 6px; margin-bottom: 6px; align-items: center;">
          <span style="font-size: 9.5px; font-weight: 700; color: var(--text-dim); text-transform: uppercase;">Parameters:</span>
          ${p.map(([y,h])=>{let S=d.includes(y),E=h.type||(h.enum?"enum":"any");return`
              <span style="font-size: 9.5px; font-family: var(--ff-mono); padding: 1px 5px; background: ${S?"rgba(239, 68, 68, 0.15)":"rgba(148, 163, 184, 0.1)"}; color: ${S?"var(--red-400)":"var(--text-muted)"}; border: 1px solid ${S?"rgba(239, 68, 68, 0.3)":"var(--border)"}; border-radius: 3px;" title="${u(h.description||"")}">
                ${u(y)} (${E}${S?" *":""})
              </span>
            `}).join("")}
        </div>
      `;return`
      <div style="background: rgba(0,0,0,0.3); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 14px; margin-bottom: 12px;">
        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 10px;">
          <div style="display: flex; align-items: center; gap: 8px;">
            <span class="badge" style="background: rgba(56, 189, 248, 0.15); color: var(--cyan-400); font-family: var(--ff-mono); font-weight: 700;">STEP ${r+1}</span>
            <span style="font-size: 11px; font-family: var(--ff-mono); color: var(--text-dim);">id: ${u(s.id)}</span>
          </div>
          <button class="btn btn-ghost" style="padding: 2px 8px; font-size: 11px; color: var(--red-400);" onclick="window.app.removeBatchStep(${r})">
            ✕ Remove
          </button>
        </div>

        <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 12px; margin-bottom: 6px;">
          <div class="form-group" style="margin: 0;">
            <label class="form-label" style="font-size: 11px;">Target Capability</label>
            <select class="form-input" style="font-size: 11.5px;" onchange="window.app.updateBatchStepCapability(${r}, this.value)">
              <option value="">-- Select Capability --</option>
              ${g}
            </select>
          </div>
          <div style="display: flex; align-items: flex-end; padding-bottom: 6px;">
            <label style="display: flex; align-items: center; gap: 6px; font-size: 11.5px; color: var(--text-muted); cursor: pointer;">
              <input type="checkbox" ${s.continue_on_error?"checked":""} onchange="window.app.updateBatchStepContinueOnError(${r}, this.checked)" />
              <span>Continue pipeline on step failure</span>
            </label>
          </div>
        </div>

        ${m}

        <div class="form-group" style="margin: 0;">
          <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 4px;">
            <div style="display: flex; align-items: center; gap: 8px;">
              <label class="form-label" style="margin: 0; font-size: 11px;">Step Arguments JSON</label>
              ${n?`
                <button type="button" class="btn btn-ghost" style="padding: 1px 6px; font-size: 9.5px;" onclick="window.app.fillBatchStepSampleArgs(${r})">✨ Sample Args</button>
              `:""}
            </div>
            <div style="display: flex; gap: 6px; font-size: 10px; color: var(--cyan-400); font-family: var(--ff-mono);">
              <span>Helpers:</span>
              <code style="cursor: pointer; background: rgba(0,0,0,0.3); padding: 1px 4px; border-radius: 2px;" onclick="window.app.appendBatchVariable(${r}, '\${steps[0].result.id}')">\${steps[0].result.id}</code>
              <code style="cursor: pointer; background: rgba(0,0,0,0.3); padding: 1px 4px; border-radius: 2px;" onclick="window.app.appendBatchVariable(${r}, '\${steps[0].result.data}')">\${steps[0].result.data}</code>
            </div>
          </div>
          <textarea 
            id="batch-step-args-${r}"
            class="form-textarea" 
            rows="3" 
            style="font-size: 11px; font-family: var(--ff-mono);" 
            oninput="window.app.updateBatchStepArgs(${r}, this.value)"
          >${u(s.argsJson)}</textarea>
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
            ${a.length} sequential execution steps configured
          </div>
          <div style="display: flex; gap: 10px;">
            <button class="btn btn-ghost" onclick="window.app.closeBatchModal()">Cancel</button>
            <button class="btn btn-primary" onclick="window.app.executeBatchPipeline()">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polygon points="5 3 19 12 5 21 5 3"></polygon></svg>
              Run Batch Pipeline (${a.length} Steps)
            </button>
          </div>
        </div>
      </div>
    </div>
  `}function u(e){return String(e||"").replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;")}function U(e){let t=e.approvals.filter((d)=>d.status==="pending"),a=e.approvals.filter((d)=>d.status!=="pending"),i=e.approvals.filter((d)=>d.status==="approved").length,s=e.approvals.filter((d)=>d.status==="rejected").length,r=e.config.policy?.require_approval||e.config.policy?.requireApproval||[],n=t.length===0?`
    <div style="padding: 48px 24px; text-align: center; background: var(--surface-card); border-radius: var(--radius-md); border: 1px dashed var(--border);">
      <div style="width: 48px; height: 48px; border-radius: 50%; background: rgba(52, 211, 153, 0.12); border: 1px solid rgba(52, 211, 153, 0.3); display: flex; align-items: center; justify-content: center; margin: 0 auto 14px; color: var(--green-400); font-size: 20px; font-weight: 700;">
        ✓
      </div>
      <div style="font-size: 15px; font-weight: 600; color: var(--text-main); margin-bottom: 6px;">All Clear — No Pending Approvals</div>
      <div style="font-size: 12px; color: var(--text-dim); max-width: 520px; margin: 0 auto; line-height: 1.6;">
        Tool invocations intercepted by <code style="color: var(--amber-300); font-family: var(--ff-mono);">require_approval</code> governance rules will suspend execution and appear here for operator inspection, argument modification, and cryptographic gating.
      </div>
    </div>
  `:t.map((d)=>`
    <div class="bento-card" style="border: 1px solid rgba(245, 158, 11, 0.35); background: var(--surface-card); margin-bottom: 14px; padding: 18px;">
      <div style="display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 12px;">
        <div>
          <div style="display: flex; align-items: center; gap: 8px;">
            <span class="brand-badge" style="background: rgba(245, 158, 11, 0.2); color: var(--amber-300); border-color: rgba(245, 158, 11, 0.5);">
              PENDING APPROVAL
            </span>
            <span style="font-family: var(--ff-mono); font-size: 12px; color: var(--text-muted);">${_(d.id)}</span>
          </div>
          <div style="margin-top: 6px; display: flex; align-items: center; gap: 8px;">
            <span style="font-size: 14.5px; font-weight: 700; color: var(--text-main); font-family: var(--ff-mono);">${_(d.capability_id)}</span>
            <span style="font-size: 11px; color: var(--text-dim);">via <span style="color: var(--cyan-400); font-family: var(--ff-mono);">${_(d.server_id)}</span></span>
          </div>
        </div>

        <div style="text-align: right; font-family: var(--ff-mono); font-size: 11px; color: var(--text-dim);">
          <div>Created: <span style="color: var(--text-muted);">${new Date(d.created_at*1000).toLocaleTimeString()}</span></div>
          <div style="color: var(--amber-400); margin-top: 2px;">Expires: ${new Date(d.expires_at*1000).toLocaleTimeString()}</div>
        </div>
      </div>

      <!-- Caller Context -->
      ${d.context||d.request_id?`
        <div style="background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 8px 12px; font-family: var(--ff-mono); font-size: 11px; display: flex; flex-wrap: wrap; gap: 16px; margin-bottom: 12px; color: var(--text-muted);">
          ${d.request_id?`<div><span style="color: var(--text-dim);">Request:</span> <span style="color: var(--text-main);">${_(d.request_id)}</span></div>`:""}
          ${d.context?.actor_id?`<div><span style="color: var(--text-dim);">Actor:</span> <span style="color: var(--cyan-400);">${_(d.context.actor_id)}</span></div>`:""}
          ${d.context?.operation_id?`<div><span style="color: var(--text-dim);">Operation:</span> <span style="color: var(--text-main);">${_(d.context.operation_id)}</span></div>`:""}
          ${d.context?.work_item_id?`<div><span style="color: var(--text-dim);">Work Item:</span> <span style="color: var(--text-main);">${_(d.context.work_item_id)}</span></div>`:""}
        </div>
      `:""}

      <!-- Arguments Editor -->
      <div style="margin-bottom: 14px;">
        <div style="font-size: 11px; font-weight: 600; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.5px; margin-bottom: 6px;">
          Parameters (Editable before approval)
        </div>
        <textarea id="appr-args-${d.id}" class="form-textarea" rows="4" style="color: var(--green-400); font-family: var(--ff-mono); font-size: 11.5px; line-height: 1.4;">${_(JSON.stringify(d.sanitized_args,null,2))}</textarea>
      </div>

      <!-- Action Footer -->
      <div style="display: flex; justify-content: space-between; align-items: center; border-top: 1px solid var(--border); padding-top: 12px;">
        <div style="display: flex; align-items: center; gap: 8px;">
          <input id="appr-operator-${d.id}" type="text" class="form-input" placeholder="Operator ID" value="security-operator" style="width: 200px; padding: 5px 10px; font-size: 11px;">
        </div>

        <div style="display: flex; align-items: center; gap: 8px;">
          <button class="btn btn-danger" onclick="window.app.promptReject('${_(d.id)}')">
            ✕ Reject
          </button>
          <button class="btn btn-primary" onclick="window.app.submitApproval('${_(d.id)}')">
            ✓ Approve &amp; Execute
          </button>
        </div>
      </div>
    </div>
  `).join(""),o=r.length===0?`
    <div style="color: var(--text-dim); font-size: 11.5px; line-height: 1.5; padding: 8px 0;">
      No explicit <code style="color: var(--amber-400);">require_approval</code> rules active. All non-denied capabilities execute immediately.
    </div>
  `:r.map((d)=>`
    <div style="display: flex; justify-content: space-between; align-items: center; background: var(--surface); padding: 8px 10px; border-radius: var(--radius-sm); border: 1px solid var(--border); margin-bottom: 6px;">
      <span style="font-family: var(--ff-mono); font-size: 11.5px; color: var(--amber-300); font-weight: 500;">\uD83D\uDEE1️ ${_(d)}</span>
      <span class="brand-badge" style="font-size: 9.5px; padding: 1px 5px;">GATED</span>
    </div>
  `).join(""),c=a.length===0?`
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
          ${a.map((d)=>`
            <tr style="border-bottom: 1px solid rgba(255,255,255,0.03); transition: background 0.15s;" onmouseover="this.style.background='var(--surface-hover)'" onmouseout="this.style.background='transparent'">
              <td style="padding: 10px 14px;">
                <span class="brand-badge" style="${d.status==="approved"?"background: rgba(52, 211, 153, 0.12); color: var(--green-400); border-color: rgba(52, 211, 153, 0.3);":d.status==="rejected"?"background: rgba(248, 113, 113, 0.12); color: var(--red-400); border-color: rgba(248, 113, 113, 0.3);":"background: var(--surface-hover); color: var(--text-dim);"}">
                  ${d.status.toUpperCase()}
                </span>
              </td>
              <td style="padding: 10px 14px; font-weight: 600; color: var(--text-main);">${_(d.capability_id)}</td>
              <td style="padding: 10px 14px; color: var(--text-dim); font-size: 10.5px;">${_(d.id)}</td>
              <td style="padding: 10px 14px; color: var(--text-muted);">${_(d.operator||"system")}</td>
              <td style="padding: 10px 14px; color: ${d.reason?"var(--red-400)":"var(--text-dim)"};">${d.reason?`"${_(d.reason)}"`:"—"}</td>
              <td style="padding: 10px 14px; text-align: right; color: var(--text-dim);">${new Date(d.created_at*1000).toLocaleTimeString()}</td>
            </tr>
          `).join("")}
        </tbody>
      </table>
    </div>
  `;return`
    <!-- Sub-header & Actions -->
    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px;">
      <div style="display: flex; align-items: center; gap: 10px;">
        <span class="brand-badge" style="font-size: 11px; padding: 3px 10px; color: ${t.length>0?"var(--amber-300)":"var(--green-400)"}; border-color: ${t.length>0?"rgba(245, 158, 11, 0.4)":"rgba(52, 211, 153, 0.4)"}; background: ${t.length>0?"rgba(245, 158, 11, 0.1)":"rgba(52, 211, 153, 0.1)"};">
          ${t.length} PENDING DECISION${t.length===1?"":"S"}
        </span>
        <span style="font-size: 12px; color: var(--text-dim);">
          Inspect, parameter-tweak, and approve or reject sensitive capability executions in real-time.
        </span>
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
        <div class="stat-value" style="color: var(--green-400);">${i}</div>
        <div class="stat-sub">Operator sanctioned calls</div>
      </div>
      <div class="bento-card col-3">
        <div class="stat-label">Rejected Requests</div>
        <div class="stat-value" style="color: var(--red-400);">${s}</div>
        <div class="stat-sub">Blocked &amp; reported to agent</div>
      </div>
      <div class="bento-card col-3">
        <div class="stat-label">Active Gating Rules</div>
        <div class="stat-value" style="color: var(--cyan-400);">${r.length}</div>
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
          ${n}
        </div>
      </div>

      <!-- Right Column: Active Rules & Guidelines -->
      <div class="col-4">
        <div style="font-size: 11px; font-weight: 700; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.5px; margin-bottom: 12px; display: flex; justify-content: space-between; align-items: center;">
          <span>\uD83D\uDEE1️ Gating Policy Rules</span>
          <button class="btn btn-ghost" style="padding: 2px 8px; font-size: 10.5px;" onclick="window.app.switchTab('policy')">Edit in Policy →</button>
        </div>
        <div class="bento-card" style="margin-bottom: 14px;">
          ${o}
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
          \uD83D\uDCDC Recent Decision History (${a.length})
        </div>
      </div>
      ${c}
    </div>
  `}function _(e){return String(e||"").replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;")}function J(){let e=l.getState(),t=e.auditEvents||[],a=e.auditStats||{total_events:0,by_status:{success:0,failed:0,denied:0,intercepted:0}},i=e.auditVerification,s=e.auditFilters,r=e.auditTotal??t.length,n=e.auditSelectedEvent,o=Object.keys(e.config?.mcpServers||{}),c=s.limit||25,d=s.offset||0,p=Math.floor(d/c)+1,g=Math.max(1,Math.ceil(r/c)),m=r===0?0:d+1,y=Math.min(d+c,r),h=f.getAuditExportUrl({actor_id:s.search?void 0:void 0,server_id:s.serverId!=="all"?s.serverId:void 0,event_type:s.eventType!=="all"?s.eventType:void 0,status:s.status!=="all"?s.status:void 0,search:s.search.trim()?s.search.trim():void 0},"csv"),S=f.getAuditExportUrl({server_id:s.serverId!=="all"?s.serverId:void 0,event_type:s.eventType!=="all"?s.eventType:void 0,status:s.status!=="all"?s.status:void 0,search:s.search.trim()?s.search.trim():void 0},"jsonl"),E=i?i.is_valid?`
      <div style="display: inline-flex; align-items: center; gap: 6px; padding: 4px 10px; background: rgba(34, 197, 94, 0.1); border: 1px solid rgba(34, 197, 94, 0.3); border-radius: var(--radius-sm); font-size: 11.5px; color: var(--green-400);">
        <span>\uD83D\uDEE1️</span>
        <span style="font-weight: 600;">Chain Verified: 100% Tamper Free (${i.total_records} events)</span>
      </div>
    `:`
      <div style="display: inline-flex; align-items: center; gap: 6px; padding: 4px 10px; background: rgba(239, 68, 68, 0.1); border: 1px solid rgba(239, 68, 68, 0.3); border-radius: var(--radius-sm); font-size: 11.5px; color: var(--red-400);">
        <span>⚠️</span>
        <span style="font-weight: 600;">TAMPER DETECTED at Record #${i.corrupted_at_index}</span>
      </div>
    `:`
    <button class="btn btn-ghost" style="padding: 4px 10px; font-size: 11.5px;" onclick="window.app.verifyAuditChain()">
      \uD83D\uDEE1️ Verify Cryptographic Hash Chain
    </button>
  `,w=o.map((x)=>`<option value="${b(x)}" ${s.serverId===x?"selected":""}>${b(x)}</option>`).join(""),k=`
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
            value="${b(s.search)}"
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
            <option value="all" ${s.status==="all"?"selected":""}>All Statuses</option>
            <option value="success" ${s.status==="success"?"selected":""}>\uD83D\uDFE2 Success</option>
            <option value="denied" ${s.status==="denied"?"selected":""}>\uD83D\uDD34 Denied</option>
            <option value="intercepted" ${s.status==="intercepted"?"selected":""}>\uD83D\uDFE1 HITL Intercept</option>
            <option value="failed" ${s.status==="failed"?"selected":""}>❌ Failed</option>
            <option value="cancelled" ${s.status==="cancelled"?"selected":""}>⚪ Cancelled</option>
          </select>
        </div>

        <!-- Event Type Filter -->
        <div>
          <select 
            class="form-input" 
            style="width: 100%; font-size: 12px; height: 32px;"
            onchange="window.app.handleAuditEventTypeFilter(this.value)"
          >
            <option value="all" ${s.eventType==="all"?"selected":""}>All Event Types</option>
            <option value="tool_execution" ${s.eventType==="tool_execution"?"selected":""}>Tool Execution</option>
            <option value="tool_intercepted_hitl" ${s.eventType==="tool_intercepted_hitl"?"selected":""}>HITL Intercept</option>
            <option value="approval_granted" ${s.eventType==="approval_granted"?"selected":""}>Approval Granted</option>
            <option value="approval_rejected" ${s.eventType==="approval_rejected"?"selected":""}>Approval Rejected</option>
            <option value="approval_expired" ${s.eventType==="approval_expired"?"selected":""}>Approval Expired</option>
            <option value="policy_violation" ${s.eventType==="policy_violation"?"selected":""}>Policy Violation</option>
            <option value="config_mutation" ${s.eventType==="config_mutation"?"selected":""}>Config Mutation</option>
            <option value="sampling_call" ${s.eventType==="sampling_call"?"selected":""}>Sampling Call</option>
            <option value="resource_access" ${s.eventType==="resource_access"?"selected":""}>Resource Access</option>
          </select>
        </div>

        <!-- Server Filter -->
        <div>
          <select 
            class="form-input" 
            style="width: 100%; font-size: 12px; height: 32px;"
            onchange="window.app.handleAuditServerFilter(this.value)"
          >
            <option value="all" ${s.serverId==="all"?"selected":""}>All MCP Servers</option>
            ${w}
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
  `,R=`
    <div style="display: flex; justify-content: space-between; align-items: center; padding: 12px 16px; background: rgba(18, 24, 38, 0.5); border-radius: var(--radius-md); border: 1px solid var(--border); margin-top: 16px;">
      <div style="font-size: 12px; color: var(--text-dim); display: flex; align-items: center; gap: 8px;">
        <span>Showing <strong style="color: var(--text-main);">${m}–${y}</strong> of <strong style="color: var(--text-main);">${r}</strong> events</span>
        <span style="color: var(--border);">|</span>
        <span>Page Size:</span>
        <select 
          class="form-input" 
          style="font-size: 11.5px; padding: 2px 24px 2px 8px; height: 28px; width: auto;"
          onchange="window.app.handleAuditPageSize(this.value)"
        >
          <option value="10" ${c===10?"selected":""}>10 / page</option>
          <option value="25" ${c===25?"selected":""}>25 / page</option>
          <option value="50" ${c===50?"selected":""}>50 / page</option>
          <option value="100" ${c===100?"selected":""}>100 / page</option>
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
          Page ${p} of ${g}
        </span>
        <button 
          class="btn btn-ghost" 
          style="padding: 4px 8px; font-size: 11px; height: 28px;"
          ${p>=g?'disabled style="opacity: 0.4; cursor: not-allowed;"':""}
          onclick="window.app.auditNextPage()"
        >
          Next ▶
        </button>
        <button 
          class="btn btn-ghost" 
          style="padding: 4px 8px; font-size: 11px; height: 28px;"
          ${p>=g?'disabled style="opacity: 0.4; cursor: not-allowed;"':""}
          onclick="window.app.auditGoToPage(${g})"
          title="Last Page"
        >
          Last ⏭
        </button>
      </div>
    </div>
  `,z="";if(t.length===0)z=`
      <div style="padding: 48px 24px; text-align: center; color: var(--text-dim); background: var(--surface-card); border-radius: var(--radius-md); border: 1px dashed var(--border);">
        <div style="font-size: 28px; margin-bottom: 8px;">\uD83D\uDD0D</div>
        <div style="font-size: 14px; font-weight: 600; color: var(--text-main); margin-bottom: 4px;">No Matching Audit Events</div>
        <div style="font-size: 12px; max-width: 420px; margin: 0 auto;">No audit records match your currently selected filters. Try broadening your search or resetting filters.</div>
        <button class="btn btn-ghost" style="margin-top: 14px; font-size: 11.5px;" onclick="window.app.clearAuditFilters()">Reset Filters</button>
      </div>
    `;else z=t.map((x)=>{let O=new Date(Math.floor(x.timestamp_ns/1e6)).toLocaleString(),P='<span class="badge" style="background: rgba(34, 197, 94, 0.15); color: var(--green-400); font-weight: 600;">SUCCESS</span>';if(x.status==="denied")P='<span class="badge" style="background: rgba(239, 68, 68, 0.15); color: var(--red-400); font-weight: 600;">DENIED</span>';else if(x.status==="intercepted")P='<span class="badge" style="background: rgba(234, 179, 8, 0.15); color: var(--amber-300); font-weight: 600;">HITL INTERCEPT</span>';else if(x.status==="failed")P='<span class="badge" style="background: rgba(239, 68, 68, 0.15); color: var(--red-400); font-weight: 600;">FAILED</span>';else if(x.status==="cancelled")P='<span class="badge" style="background: rgba(148, 163, 184, 0.15); color: var(--text-muted); font-weight: 600;">CANCELLED</span>';let Y=x.sanitized_args?JSON.stringify(x.sanitized_args):"-",X=x.actor_id||x.operator_id||"anonymous",Z=x.server_id||"system",ee=x.capability_id||x.event_type,te=x.execution_latency_us?`${(x.execution_latency_us/1000).toFixed(1)}ms`:"-";return`
        <div class="bento-card" style="margin-bottom: 12px; padding: 16px; border: 1px solid var(--border); transition: border-color 0.15s ease;">
          <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
            <div style="display: flex; align-items: center; gap: 8px;">
              <span style="font-family: var(--ff-mono); font-size: 11px; font-weight: 700; color: var(--text-dim);">${b(x.id)}</span>
              ${P}
              <span style="font-size: 12px; font-weight: 600; color: var(--text-main);">${b(ee)}</span>
            </div>
            <div style="display: flex; align-items: center; gap: 10px;">
              <div style="font-family: var(--ff-mono); font-size: 11px; color: var(--text-muted);">${b(O)}</div>
              <button 
                class="btn btn-ghost" 
                style="padding: 2px 8px; font-size: 11px; height: 24px;" 
                onclick="window.app.selectAuditEvent('${b(x.id)}')"
                title="Inspect event details & cryptographic payload"
              >
                Inspect \uD83D\uDD0D
              </button>
            </div>
          </div>
          
          <div style="display: grid; grid-template-columns: repeat(4, 1fr); gap: 8px; font-size: 11.5px; background: rgba(0,0,0,0.2); padding: 8px 12px; border-radius: var(--radius-sm); margin-bottom: 8px;">
            <div><span style="color: var(--text-muted);">Actor:</span> <strong style="color: var(--text-main);">${b(X)}</strong></div>
            <div><span style="color: var(--text-muted);">Server:</span> <strong style="color: var(--cyan-400);">${b(Z)}</strong></div>
            <div><span style="color: var(--text-muted);">Trace:</span> <code style="color: var(--cyan-400); font-size: 10.5px;">${b(x.trace_id)}</code></div>
            <div><span style="color: var(--text-muted);">Latency:</span> <span style="color: var(--amber-300);">${te}</span></div>
          </div>

          <div style="font-family: var(--ff-mono); font-size: 11px; color: var(--text-dim); margin-bottom: 8px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">
            <span style="color: var(--text-muted);">Args:</span> ${b(Y)}
          </div>

          <div style="display: flex; justify-content: space-between; align-items: center; border-top: 1px solid var(--border); padding-top: 6px; font-size: 10.5px; font-family: var(--ff-mono); color: var(--text-muted);">
            <div><span style="color: var(--text-dim);">prev_hash:</span> ${b(x.prev_hash.slice(0,16))}...</div>
            <div><span style="color: var(--text-dim);">hash:</span> <span style="color: var(--green-400);">${b(x.hash.slice(0,16))}...</span></div>
          </div>
        </div>
      `}).join("");let C="";if(n){let x=new Date(Math.floor(n.timestamp_ns/1e6)).toISOString();C=`
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
              <div><span style="color: var(--text-muted);">Timestamp:</span> <strong style="color: var(--text-main); font-family: var(--ff-mono); font-size: 11px;">${b(x)}</strong></div>
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
    <!-- Sub-header & Actions -->
    <div style="margin-bottom: 18px; display: flex; justify-content: space-between; align-items: center;">
      <div style="font-size: 12px; color: var(--text-dim);">
        Cryptographically tamper-evident, append-only execution log for SOC2 & ISO 27001 compliance.
      </div>
      <div style="display: flex; gap: 8px; align-items: center;">
        ${E}
        <a href="${h}" download class="btn btn-ghost" style="font-size: 11.5px; text-decoration: none;" title="Export current filtered view as CSV">\uD83D\uDCE5 Export CSV</a>
        <a href="${S}" download class="btn btn-ghost" style="font-size: 11.5px; text-decoration: none;" title="Export current filtered view as JSONL">\uD83D\uDCE5 Export JSONL</a>
        <button class="btn btn-primary" style="font-size: 11.5px;" onclick="window.app.refreshAuditEvents()">\uD83D\uDD04 Refresh</button>
      </div>
    </div>

    <!-- Stats summary cards -->
    <div style="display: grid; grid-template-columns: repeat(4, 1fr); gap: 12px; margin-bottom: 20px;">
      <div class="bento-card" style="padding: 14px; text-align: center;">
        <div style="font-size: 11px; color: var(--text-muted); text-transform: uppercase; font-weight: 600;">Total Events</div>
        <div style="font-size: 22px; font-weight: 800; color: var(--text-main); margin-top: 4px;">${a.total_events}</div>
      </div>
      <div class="bento-card" style="padding: 14px; text-align: center;">
        <div style="font-size: 11px; color: var(--text-muted); text-transform: uppercase; font-weight: 600;">Successful Calls</div>
        <div style="font-size: 22px; font-weight: 800; color: var(--green-400); margin-top: 4px;">${a.by_status.success}</div>
      </div>
      <div class="bento-card" style="padding: 14px; text-align: center;">
        <div style="font-size: 11px; color: var(--text-muted); text-transform: uppercase; font-weight: 600;">HITL Intercepts</div>
        <div style="font-size: 22px; font-weight: 800; color: var(--amber-300); margin-top: 4px;">${a.by_status.intercepted}</div>
      </div>
      <div class="bento-card" style="padding: 14px; text-align: center;">
        <div style="font-size: 11px; color: var(--text-muted); text-transform: uppercase; font-weight: 600;">Policy Denials</div>
        <div style="font-size: 22px; font-weight: 800; color: var(--red-400); margin-top: 4px;">${a.by_status.denied}</div>
      </div>
    </div>

    <!-- Search & Filter Toolbar -->
    ${k}

    <!-- Event Timeline List Header -->
    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px;">
      <h2 style="font-size: 14px; font-weight: 600; color: var(--text-main);">Sequential Audit Ledger (SHA-256 Hash Chained)</h2>
      <span style="font-size: 11.5px; color: var(--text-dim);">${t.length} events loaded on this page</span>
    </div>

    <!-- Event Rows -->
    <div>
      ${z}
    </div>

    <!-- Pagination Footer -->
    ${r>0?R:""}

    <!-- Modal Popup for Event Inspection -->
    ${C}
  `}function b(e){if(!e)return"";return String(e).replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;").replace(/'/g,"&#039;")}function K(){let t=l.getState().config.policy||{},a=t.allow||[],i=t.deny||[],s=t.redact_keys||t.redactKeys||[],r=t.require_approval||t.requireApproval||[],n=a.length===0?`
    <div style="color: var(--text-dim); font-size: 12px;">No allow list (all non-denied operations permitted)</div>
  `:a.map((p,g)=>`
    <div style="display: flex; justify-content: space-between; align-items: center; background: var(--surface); padding: 8px 12px; border-radius: var(--radius-sm); border: 1px solid var(--border);">
      <span style="font-family: var(--ff-mono); font-size: 12px; color: var(--green-400);">✔ ${j(p)}</span>
      <button class="btn btn-ghost" style="padding: 2px 6px; font-size: 11px; color: var(--red-400);" onclick="window.app.removePolicyRule('allow', ${g})">✕</button>
    </div>
  `).join(""),o=i.length===0?`
    <div style="color: var(--text-dim); font-size: 12px;">No deny rules configured</div>
  `:i.map((p,g)=>`
    <div style="display: flex; justify-content: space-between; align-items: center; background: var(--surface); padding: 8px 12px; border-radius: var(--radius-sm); border: 1px solid var(--border);">
      <span style="font-family: var(--ff-mono); font-size: 12px; color: var(--red-400);">✖ ${j(p)}</span>
      <button class="btn btn-ghost" style="padding: 2px 6px; font-size: 11px; color: var(--red-400);" onclick="window.app.removePolicyRule('deny', ${g})">✕</button>
    </div>
  `).join(""),c=r.length===0?`
    <div style="color: var(--text-dim); font-size: 12px;">No human-in-the-loop approval rules configured</div>
  `:r.map((p,g)=>`
    <div style="display: flex; justify-content: space-between; align-items: center; background: var(--surface); padding: 8px 12px; border-radius: var(--radius-sm); border: 1px solid var(--border);">
      <span style="font-family: var(--ff-mono); font-size: 12px; color: var(--amber-400);">\uD83D\uDEE1️ ${j(p)}</span>
      <button class="btn btn-ghost" style="padding: 2px 6px; font-size: 11px; color: var(--red-400);" onclick="window.app.removePolicyRule('requireApproval', ${g})">✕</button>
    </div>
  `).join(""),d=s.length===0?`
    <div style="color: var(--text-dim); font-size: 12px;">No key redaction patterns configured</div>
  `:s.map((p,g)=>`
    <span class="brand-badge" style="color: var(--amber-300); padding: 5px 10px; font-size: 11px; display: inline-flex; align-items: center; gap: 6px;">
      ${j(p)}
      <span style="cursor: pointer; color: var(--red-400); font-weight: bold;" onclick="window.app.removePolicyRule('redact', ${g})">✕</span>
    </span>
  `).join("");return`
    <!-- Sub-header -->
    <div style="margin-bottom: 16px; font-size: 12px; color: var(--text-dim);">
      Wildcard capability access control, human-in-the-loop triggers, and sensitive key masking.
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
          ${o}
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
          ${c}
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
          ${d}
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
  `}function j(e){return String(e||"").replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;")}function G(){let e=l.getState(),t=e.config,a=Object.entries(t.capabilityAliases||{}),i=Object.entries(t.resourceAliases||{}),s=Object.entries(t.promptAliases||{}),r="";if(a.length===0&&i.length===0&&s.length===0)r=`
      <div style="padding: 24px; text-align: center; color: var(--text-dim);">
        No facade aliases configured in ${A(e.configPath)}. Add short names to prune token payload sizes.
      </div>
    `;else{for(let[n,o]of a)r+=`
        <div class="feed-row" style="grid-template-columns: 90px 180px 1fr 80px;">
          <span style="color: var(--cyan-400);">Tool</span>
          <span style="font-weight: 700; color: var(--text-main); font-family: var(--ff-mono);">${A(n)}</span>
          <span style="color: var(--text-dim); font-family: var(--ff-mono);">${A(o)}</span>
          <div style="text-align: right;">
            <button class="btn btn-ghost" style="padding: 2px 6px; color: var(--red-400);" onclick="window.app.deleteAlias('tool', '${A(n)}')">✕</button>
          </div>
        </div>
      `;for(let[n,o]of i)r+=`
        <div class="feed-row" style="grid-template-columns: 90px 180px 1fr 80px;">
          <span style="color: var(--green-400);">Resource</span>
          <span style="font-weight: 700; color: var(--text-main); font-family: var(--ff-mono);">${A(n)}</span>
          <span style="color: var(--text-dim); font-family: var(--ff-mono);">${A(o)}</span>
          <div style="text-align: right;">
            <button class="btn btn-ghost" style="padding: 2px 6px; color: var(--red-400);" onclick="window.app.deleteAlias('resource', '${A(n)}')">✕</button>
          </div>
        </div>
      `;for(let[n,o]of s)r+=`
        <div class="feed-row" style="grid-template-columns: 90px 180px 1fr 80px;">
          <span style="color: var(--amber-300);">Prompt</span>
          <span style="font-weight: 700; color: var(--text-main); font-family: var(--ff-mono);">${A(n)}</span>
          <span style="color: var(--text-dim); font-family: var(--ff-mono);">${A(o)}</span>
          <div style="text-align: right;">
            <button class="btn btn-ghost" style="padding: 2px 6px; color: var(--red-400);" onclick="window.app.deleteAlias('prompt', '${A(n)}')">✕</button>
          </div>
        </div>
      `}return`
    <!-- Sub-header -->
    <div style="margin-bottom: 16px; font-size: 12px; color: var(--text-dim);">
      Shorten capability IDs to prune prompt tokens and create stable public interfaces.
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
      ${r}
    </div>
  `}function A(e){return String(e||"").replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;")}function V(){let e=l.getState(),t=e.config,a=t.profiles||{},i=Object.entries(a),s=t.mcpServers||{},r=e.activeProfile,n="";if(i.length===0)n=`
      <div style="padding: 40px; text-align: center; color: var(--text-dim); background: var(--surface-card); border-radius: var(--radius-md); border: 1px dashed var(--border);">
        <div style="font-size: 15px; color: var(--text-main); font-weight: 600; margin-bottom: 8px;">No Profiles Configured</div>
        <p style="font-size: 12px; margin-bottom: 20px; max-width: 480px; margin-left: auto; margin-right: auto;">
          Profiles allow Warmplane to serve multiple task-relevant server constellations (e.g. <code>coding</code>, <code>support</code>, <code>data</code>) from one running daemon process.
        </p>
        <button class="btn btn-primary" onclick="window.app.openAddProfileModal()">+ Create First Profile</button>
      </div>
    `;else n=i.map(([o,c])=>{let d=r===o,p=c.servers.map((m)=>`<span class="brand-badge" style="${s[m]?"color: var(--cyan-400); border-color: rgba(34, 211, 238, 0.25); background: rgba(34, 211, 238, 0.05);":"color: var(--red-400); border-color: rgba(248, 113, 113, 0.3); background: rgba(248, 113, 113, 0.05);"}">${L(m)}</span>`).join(" "),g=(e.capabilities||[]).filter((m)=>c.servers.includes(m.server)).length;return`
        <div class="bento-card" style="margin-bottom: 14px; border-left: ${d?"3px solid var(--amber-400)":"1px solid var(--border)"};">
          <div style="display: flex; justify-content: space-between; align-items: flex-start;">
            <div>
              <div style="display: flex; align-items: center; gap: 8px; margin-bottom: 6px;">
                <span style="font-size: 16px; font-weight: 700; color: var(--text-main); font-family: var(--ff-mono);">${L(o)}</span>
                ${d?'<span class="brand-badge" style="color: var(--amber-400); border-color: rgba(245, 158, 11, 0.4); background: rgba(245, 158, 11, 0.1);">ACTIVE IN UI</span>':""}
                <span class="brand-badge">${c.servers.length} server${c.servers.length===1?"":"s"}</span>
                <span class="brand-badge" style="color: var(--text-dim);">${g} capabilities</span>
              </div>
              <div style="font-size: 12px; color: var(--text-muted); margin-bottom: 10px;">
                ${L(c.description||"No description provided")}
              </div>
              <div style="display: flex; flex-wrap: wrap; gap: 6px; align-items: center;">
                <span style="font-size: 11px; color: var(--text-dim); font-weight: 600; text-transform: uppercase;">Servers:</span>
                ${p||'<span style="font-size: 11px; color: var(--text-dim);">None</span>'}
              </div>
            </div>
            
            <div style="display: flex; gap: 8px; align-items: center;">
              ${d?`
                <button class="btn btn-ghost" style="padding: 4px 10px; font-size: 11.5px;" onclick="window.app.setActiveProfile(null)">
                  Deselect
                </button>
              `:`
                <button class="btn btn-primary" style="padding: 4px 10px; font-size: 11.5px;" onclick="window.app.setActiveProfile('${L(o)}')">
                  Activate in UI
                </button>
              `}
              <button class="btn btn-ghost" style="padding: 4px 10px; font-size: 11.5px;" onclick="window.app.openEditProfileModal('${L(o)}')">
                ✏️ Edit
              </button>
              <button class="btn btn-danger" style="padding: 4px 10px; font-size: 11.5px;" onclick="window.app.deleteProfile('${L(o)}')">
                Remove
              </button>
            </div>
          </div>
        </div>
      `}).join("");return`
    <!-- Sub-header & Actions -->
    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px;">
      <div style="font-size: 12px; color: var(--text-dim);">
        Define named subsets of servers for task-specific agent interactions, dynamic per-request switching, and scoped ETag caching.
      </div>
      <div style="display: flex; gap: 8px;">
        <button class="btn btn-primary" onclick="window.app.openAddProfileModal()">+ Create Profile</button>
      </div>
    </div>

    <!-- Quick Info Box -->
    <div class="bento-card" style="margin-bottom: 18px; background: rgba(245, 158, 11, 0.03); border: 1px solid rgba(245, 158, 11, 0.15);">
      <div style="display: flex; gap: 12px; align-items: center;">
        <span style="font-size: 20px;">\uD83D\uDCA1</span>
        <div style="font-size: 12px; color: var(--text-muted); line-height: 1.5;">
          HTTP clients can select profiles dynamically using the <code style="color: var(--amber-400);">X-Warmplane-Profile: &lt;name&gt;</code> header or <code style="color: var(--amber-400);">?profile=&lt;name&gt;</code> query parameter. MCP stdio clients can pass <code style="color: var(--amber-400);">--profile &lt;name&gt;</code>.
        </div>
      </div>
    </div>

    ${n}
  `}function L(e){return String(e||"").replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;")}var B=[{id:"github",name:"GitHub",category:"devtools",description:"Explore repositories, issues, pull requests, branches, and commit histories.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-github"],envFields:[{key:"GITHUB_PERSONAL_ACCESS_TOKEN",label:"GitHub Personal Access Token",placeholder:"ghp_...",required:!0,description:"Classic or fine-grained token with repo scope."}]},{id:"git",name:"Git (Local)",category:"devtools",description:"Read local Git repository status, diffs, log histories, and commit changes.",badge:"Official / uvx",command:"uvx",defaultArgs:["mcp-server-git","--repository","."],argsPlaceholder:"mcp-server-git --repository /path/to/repo",envFields:[]},{id:"filesystem",name:"Filesystem",category:"devtools",description:"Secure, sandboxed access to local files and directories for AI workflows.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-filesystem","."],argsPlaceholder:"-y @modelcontextprotocol/server-filesystem /allowed/dir1 /allowed/dir2",envFields:[]},{id:"memory",name:"Memory Graph",category:"devtools",description:"Persistent knowledge-graph based memory for multi-turn agent learning.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-memory"],envFields:[]},{id:"chrome-devtools",name:"Chrome DevTools",category:"devtools",description:"Inspect live DOM, execute scripts, read console logs, and capture network traces in Chrome.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-puppeteer"],envFields:[]},{id:"sentry",name:"Sentry",category:"devtools",description:"Query production error events, stack traces, and issue frequencies directly from Sentry.",badge:"uvx / Telemetry",command:"uvx",defaultArgs:["mcp-server-sentry"],envFields:[{key:"SENTRY_AUTH_TOKEN",label:"Sentry Auth Token",placeholder:"sntrys_...",required:!0}]},{id:"playwright",name:"Playwright Browser",category:"browser",description:"Headless / headed browser automation for scraping, form filling, and UI interaction.",badge:"Popular #1 / npx",command:"npx",defaultArgs:["-y","@executeautomation/playwright-mcp-server"],envFields:[]},{id:"puppeteer",name:"Puppeteer",category:"browser",description:"Official browser automation server for web page scraping and screenshot capture.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-puppeteer"],envFields:[]},{id:"brave-search",name:"Brave Search",category:"browser",description:"Real-time privacy-preserving web search and local point-of-interest query engine.",badge:"Official / Search",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-brave-search"],envFields:[{key:"BRAVE_API_KEY",label:"Brave Search API Key",placeholder:"BSA...",required:!0}]},{id:"tavily",name:"Tavily Search",category:"browser",description:"AI-optimized web search engine structured specifically for LLM context injection.",badge:"Community / Stdio",command:"npx",defaultArgs:["-y","@tavily/mcp-server"],envFields:[{key:"TAVILY_API_KEY",label:"Tavily API Key",placeholder:"tvly-...",required:!0}]},{id:"fetch",name:"Fetch / Web Markdown",category:"browser",description:"Download web pages, strip clutter, and convert raw HTML to clean markdown text.",badge:"Official / uvx",command:"uvx",defaultArgs:["mcp-server-fetch"],envFields:[]},{id:"postgres",name:"PostgreSQL",category:"database",description:"Read schemas, inspect tables, and execute SQL queries against PostgreSQL databases.",badge:"Official / Database",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-postgres","postgresql://user:pass@localhost:5432/mydb"],argsPlaceholder:"-y @modelcontextprotocol/server-postgres postgresql://user:pass@localhost:5432/dbname",envFields:[]},{id:"sqlite",name:"SQLite",category:"database",description:"Local embedded SQLite query runner and schema inspector.",badge:"Official / uvx",command:"uvx",defaultArgs:["mcp-server-sqlite","--db-path","./app.db"],argsPlaceholder:"mcp-server-sqlite --db-path /path/to/database.sqlite",envFields:[]},{id:"supabase",name:"Supabase",category:"database",description:"Query database tables, manage auth policies, and inspect storage in Supabase.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@supabase/mcp-server"],envFields:[{key:"SUPABASE_ACCESS_TOKEN",label:"Supabase Personal Access Token",placeholder:"sbp_...",required:!0},{key:"SUPABASE_PROJECT_REF",label:"Supabase Project Reference ID",placeholder:"abcdefghijklmnop",required:!1}]},{id:"redis",name:"Redis",category:"database",description:"Inspect cached keys, hash sets, lists, TTLs, and pub/sub channels in Redis.",badge:"uvx / Key-Value",command:"uvx",defaultArgs:["mcp-server-redis","--url","redis://localhost:6379"],argsPlaceholder:"mcp-server-redis --url redis://localhost:6379",envFields:[]},{id:"s3",name:"AWS S3 / Cloud Storage",category:"database",description:"Browse S3 buckets, fetch object metadata, and download files from cloud storage.",badge:"uvx / Cloud Storage",command:"uvx",defaultArgs:["mcp-server-s3","--bucket","my-bucket-name"],argsPlaceholder:"mcp-server-s3 --bucket bucket-name --region us-east-1",envFields:[{key:"AWS_ACCESS_KEY_ID",label:"AWS Access Key ID",placeholder:"AKIA...",required:!0},{key:"AWS_SECRET_ACCESS_KEY",label:"AWS Secret Access Key",placeholder:"...",required:!0},{key:"AWS_REGION",label:"AWS Region",placeholder:"us-east-1",required:!1}]},{id:"linear",name:"Linear",category:"productivity",description:"Search, create, and triage Linear issues, cycles, teams, and project roadmaps.",badge:"Productivity / Stdio",command:"npx",defaultArgs:["-y","mcp-linear"],envFields:[{key:"LINEAR_API_KEY",label:"Linear API Key",placeholder:"lin_api_...",required:!0}]},{id:"slack",name:"Slack",category:"productivity",description:"Read channels, post messages, inspect threads, and search team discussions.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-slack"],envFields:[{key:"SLACK_BOT_TOKEN",label:"Slack Bot User Token",placeholder:"xoxb-...",required:!0},{key:"SLACK_TEAM_ID",label:"Slack Team ID",placeholder:"T01234567",required:!0}]},{id:"notion",name:"Notion",category:"productivity",description:"Search Notion workspace pages, read nested blocks, and query database entries.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-notion"],envFields:[{key:"NOTION_API_KEY",label:"Notion Internal Integration Token",placeholder:"secret_...",required:!0}]},{id:"jira",name:"Jira / Atlassian",category:"productivity",description:"Manage Jira issues, search JQL, read sprint statuses, and inspect boards.",badge:"uvx / Atlassian",command:"uvx",defaultArgs:["mcp-server-jira","--url","https://your-domain.atlassian.net","--email","user@example.com"],argsPlaceholder:"mcp-server-jira --url https://org.atlassian.net --email me@org.com",envFields:[{key:"JIRA_API_TOKEN",label:"Atlassian API Token",placeholder:"ATATT3...",required:!0}]},{id:"google-drive",name:"Google Drive",category:"productivity",description:"Search, list, and read documents, spreadsheets, and drive files.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-gdrive"],envFields:[{key:"GOOGLE_APPLICATION_CREDENTIALS",label:"Google Credentials JSON Path",placeholder:"/path/to/credentials.json",required:!0}]},{id:"docker",name:"Docker",category:"cloud",description:"Inspect running containers, tail container logs, list images, and manage compose services.",badge:"uvx / DevOps",command:"uvx",defaultArgs:["mcp-server-docker"],envFields:[]},{id:"kubernetes",name:"Kubernetes (K8s)",category:"cloud",description:"Query cluster pods, services, deployment status, and inspect Kubernetes logs.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-kubernetes"],envFields:[{key:"KUBECONFIG",label:"Kubeconfig File Path (Optional)",placeholder:"~/.kube/config",required:!1}]},{id:"cloudflare",name:"Cloudflare",category:"cloud",description:"Manage Cloudflare Workers, KV namespaces, D1 databases, Vectorize indexes, and DNS.",badge:"Official / Cloudflare",command:"npx",defaultArgs:["-y","@cloudflare/mcp-server-cloudflare"],envFields:[{key:"CLOUDFLARE_API_TOKEN",label:"Cloudflare API Token",placeholder:"...",required:!0},{key:"CLOUDFLARE_ACCOUNT_ID",label:"Cloudflare Account ID",placeholder:"...",required:!0}]},{id:"terraform",name:"Terraform",category:"cloud",description:"Inspect Terraform state files, resource dependency graphs, and plan previews.",badge:"uvx / IaC",command:"uvx",defaultArgs:["mcp-server-terraform"],envFields:[]}];class W{activeTemplateCategory="all";activeTemplateFilter="";selectedTemplate=null;async init(){let e=window.location.port?`:${window.location.port}`:"",t=document.getElementById("daemon-port-label");if(t)t.textContent=`Daemon ${e}`;await this.refreshData(),this.initSSE(),this.render(),l.subscribe(()=>{this.render()})}auditSearchTimeout=null;async refreshData(){try{let e=l.getState(),t=e.auditFilters,a=e.activeProfile||void 0,[i,s,r,n,o,c,d,p]=await Promise.all([f.getConfig(),f.listCapabilities(a),f.listResources(a),f.listPrompts(a),f.getCatalogEvents(),f.listApprovals(),f.listAuditEvents({server_id:t.serverId!=="all"?t.serverId:void 0,event_type:t.eventType!=="all"?t.eventType:void 0,status:t.status!=="all"?t.status:void 0,search:t.search.trim()?t.search.trim():void 0,limit:t.limit,offset:t.offset}),f.getAuditStats()]);if(i.ok)l.setState({configPath:i.config_path,config:i.config,serverStatuses:i.server_statuses||{},circuitBreakers:i.circuit_breakers||[],metrics:{totalCatalogRequests:i.metrics?.total_catalog_requests||0,totalEtagHits:i.metrics?.total_etag_hits||0,totalToolCalls:i.metrics?.total_tool_calls||0,totalToolDurationUs:i.metrics?.total_tool_duration_us||0}});if(s&&Array.isArray(s.capabilities))l.setState({capabilities:s.capabilities});if(r&&Array.isArray(r.resources))l.setState({resources:r.resources});if(n&&Array.isArray(n.prompts))l.setState({prompts:n.prompts});if(o&&Array.isArray(o.events))l.setState({catalogEvents:o.events});if(c&&Array.isArray(c.approvals))l.setState({approvals:c.approvals});if(d&&Array.isArray(d.events))l.setState({auditEvents:d.events,auditTotal:d.total??d.events.length});if(p&&p.ok)l.setState({auditStats:p})}catch(e){console.error("Failed to fetch daemon state:",e)}}async refreshAuditEvents(){try{let t=l.getState().auditFilters,[a,i]=await Promise.all([f.listAuditEvents({server_id:t.serverId!=="all"?t.serverId:void 0,event_type:t.eventType!=="all"?t.eventType:void 0,status:t.status!=="all"?t.status:void 0,search:t.search.trim()?t.search.trim():void 0,limit:t.limit,offset:t.offset}),f.getAuditStats()]);if(a&&Array.isArray(a.events))l.setState({auditEvents:a.events,auditTotal:a.total??a.events.length});if(i&&i.ok)l.setState({auditStats:i})}catch(e){console.error("Failed to refresh audit events:",e)}}handleAuditSearchInput(e){let a={...l.getState().auditFilters,search:e,offset:0};l.setState({auditFilters:a}),clearTimeout(this.auditSearchTimeout),this.auditSearchTimeout=setTimeout(()=>{this.refreshAuditEvents()},250)}handleAuditStatusFilter(e){let t=l.getState();l.setState({auditFilters:{...t.auditFilters,status:e,offset:0}}),this.refreshAuditEvents()}handleAuditEventTypeFilter(e){let t=l.getState();l.setState({auditFilters:{...t.auditFilters,eventType:e,offset:0}}),this.refreshAuditEvents()}handleAuditServerFilter(e){let t=l.getState();l.setState({auditFilters:{...t.auditFilters,serverId:e,offset:0}}),this.refreshAuditEvents()}handleAuditPageSize(e){let t=parseInt(e,10)||25,a=l.getState();l.setState({auditFilters:{...a.auditFilters,limit:t,offset:0}}),this.refreshAuditEvents()}clearAuditFilters(){let e=l.getState();l.setState({auditFilters:{search:"",status:"all",eventType:"all",serverId:"all",limit:e.auditFilters.limit||25,offset:0}}),this.refreshAuditEvents()}auditPrevPage(){let e=l.getState(),{limit:t,offset:a}=e.auditFilters,i=Math.max(0,a-t);if(i!==a)l.setState({auditFilters:{...e.auditFilters,offset:i}}),this.refreshAuditEvents()}auditNextPage(){let e=l.getState(),{limit:t,offset:a}=e.auditFilters,i=e.auditTotal;if(a+t<i)l.setState({auditFilters:{...e.auditFilters,offset:a+t}}),this.refreshAuditEvents()}auditGoToPage(e){let t=l.getState(),{limit:a}=t.auditFilters,i=Math.max(0,(e-1)*a);l.setState({auditFilters:{...t.auditFilters,offset:i}}),this.refreshAuditEvents()}selectAuditEvent(e){if(!e){l.setState({auditSelectedEvent:null});return}let a=l.getState().auditEvents.find((i)=>i.id===e)||null;l.setState({auditSelectedEvent:a})}async verifyAuditChain(){try{let e=await f.verifyAuditChain();if(e&&e.report)l.setState({auditVerification:e.report})}catch(e){console.error("Failed to verify audit chain:",e)}}async refreshApprovals(){try{let e=await f.listApprovals();if(e&&Array.isArray(e.approvals))l.setState({approvals:e.approvals})}catch(e){console.error("Failed to refresh approvals:",e)}}initSSE(){try{let e=new EventSource("/v1/resources/updates");e.onmessage=(t)=>{l.addEventLog("SSE","/v1/resources/updates","UPDATED","0.1ms"),this.refreshData()}}catch(e){console.warn("SSE connection unavailable")}}switchTab(e){l.setState({activeTab:e}),this.refreshData()}render(){let e=l.getState(),t=document.getElementById("app-main");if(!t)return;let a=e.approvals.filter((n)=>n.status==="pending").length,i=document.getElementById("nav-approvals-badge");if(i)i.textContent=a>0?`${a}`:"",i.style.display=a>0?"inline-block":"none";document.querySelectorAll(".nav-item").forEach((n)=>{if(n.getAttribute("data-tab")===e.activeTab)n.classList.add("active");else n.classList.remove("active")});let s=document.getElementById("top-title"),r={overview:"Overview Cockpit",servers:"Server Hub & Connections",playground:"MCP Capability Playground",approvals:"Human-in-the-Loop Review Queue",audit:"WORM Audit & Compliance Ledger",policy:"Security Governance & Redaction",aliases:"Facade & Alias Studio",profiles:"Server Constellation Profiles"};if(s)s.textContent=r[e.activeTab];switch(this.renderTopProfileSelector(),e.activeTab){case"overview":t.innerHTML=N();break;case"servers":t.innerHTML=F();break;case"playground":t.innerHTML=D();break;case"approvals":t.innerHTML=U(e);break;case"audit":t.innerHTML=J();break;case"policy":t.innerHTML=K();break;case"aliases":t.innerHTML=G();break;case"profiles":t.innerHTML=V();break}}async submitApproval(e){let t=document.getElementById(`appr-operator-${e}`),a=document.getElementById(`appr-args-${e}`),i=t?.value.trim()||"security-operator",s=void 0;if(a&&a.value.trim())try{s=JSON.parse(a.value.trim())}catch{alert("Invalid JSON in arguments editor");return}let r=await f.approveTicket(e,i,s);if(r.ok)await this.refreshApprovals();else alert(`Approval failed: ${r.error||"Unknown error"}`)}async promptReject(e){let t=prompt("Reason for rejection (will be returned to the calling agent):");if(t===null)return;let i=document.getElementById(`appr-operator-${e}`)?.value.trim()||"security-operator",s=await f.rejectTicket(e,i,t);if(s.ok)await this.refreshApprovals();else alert(`Rejection failed: ${s.error||"Unknown error"}`)}setPlaygroundMode(e){l.setState({playgroundMode:e})}selectCapability(e){l.setState({selectedCapabilityId:e});let t=l.getState().capabilities.find((i)=>i.id===e),a=document.getElementById("pg-args-input");if(t){let i=M(t.input_schema,!1),s=JSON.stringify(i,null,2);if(a)a.value=s;let r={...l.getState().playgroundArgs||{}};r[e]=s,l.getState().playgroundArgs=r}}selectResource(e){l.setState({selectedResourceId:e})}selectPrompt(e){l.setState({selectedPromptId:e})}filterResources(e){let t=e.toLowerCase().trim(),i=(l.getState().resources||[]).filter((r)=>r.id.toLowerCase().includes(t)||r.name&&r.name.toLowerCase().includes(t)||r.uri&&r.uri.toLowerCase().includes(t)||r.server&&r.server.toLowerCase().includes(t)),s=document.getElementById("pg-res-list");if(s)if(i.length===0)s.innerHTML=`
          <div style="padding: 24px 16px; text-align: center; color: var(--text-dim); font-size: 11.5px;">
            No resources match "${v(e)}"
          </div>
        `;else s.innerHTML=i.map((r)=>{let n=r.id===l.getState().selectedResourceId?"active":"",o=r.uri?r.uri.split(":")[0]:"res";return`
            <div class="cap-item ${n}" onclick="window.app.selectResource('${v(r.id)}')">
              <div style="display: flex; justify-content: space-between; align-items: center;">
                <span style="font-weight: 600; color: var(--text-main); font-family: var(--ff-mono); font-size: 12px;">${v(r.name||r.id)}</span>
                <span class="badge" style="font-size: 9.5px; background: rgba(56, 189, 248, 0.15); color: var(--cyan-400);">${v(o)}</span>
              </div>
              <div style="font-size: 11px; color: var(--text-dim); margin-top: 2px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">${v(r.uri)}</div>
              <div style="display: flex; justify-content: space-between; font-size: 10px; color: var(--text-muted); margin-top: 4px;">
                <span>server: ${v(r.server||"local")}</span>
                <span>${v(r.mime_type||"text/plain")}</span>
              </div>
            </div>
          `}).join("")}filterPrompts(e){let t=e.toLowerCase().trim(),i=(l.getState().prompts||[]).filter((r)=>r.id.toLowerCase().includes(t)||r.name&&r.name.toLowerCase().includes(t)||r.description&&r.description.toLowerCase().includes(t)||r.server&&r.server.toLowerCase().includes(t)),s=document.getElementById("pg-prompt-list");if(s)if(i.length===0)s.innerHTML=`
          <div style="padding: 24px 16px; text-align: center; color: var(--text-dim); font-size: 11.5px;">
            No prompts match "${v(e)}"
          </div>
        `;else s.innerHTML=i.map((r)=>{let n=r.id===l.getState().selectedPromptId?"active":"",o=r.arguments?r.arguments.length:0;return`
            <div class="cap-item ${n}" onclick="window.app.selectPrompt('${v(r.id)}')">
              <div style="display: flex; justify-content: space-between; align-items: center;">
                <span style="font-weight: 600; color: var(--text-main); font-family: var(--ff-mono); font-size: 12px;">${v(r.name||r.id)}</span>
                <span class="badge" style="font-size: 9.5px; background: rgba(168, 85, 247, 0.15); color: var(--purple-400);">${o} args</span>
              </div>
              <div style="font-size: 11px; color: var(--text-dim); margin-top: 2px;">${v(r.description||r.title||"Prompt template")}</div>
              <div style="font-size: 10px; color: var(--text-muted); margin-top: 4px;">server: ${v(r.server||"local")}</div>
            </div>
          `}).join("")}updatePlaygroundArgs(e){let t=l.getState(),a=t.selectedCapabilityId||(t.capabilities[0]?t.capabilities[0].id:null);if(!a)return;let i={...t.playgroundArgs||{}};i[a]=e,t.playgroundArgs=i}fillPlaygroundSampleArgs(e=!1){let t=l.getState(),a=t.selectedCapabilityId||(t.capabilities[0]?t.capabilities[0].id:null),i=t.capabilities.find((o)=>o.id===a),s=document.getElementById("pg-args-input");if(!s)return;if(!i||!i.input_schema){if(s.value="{}",a){let o={...t.playgroundArgs||{}};o[a]="{}",t.playgroundArgs=o}return}let r=M(i.input_schema,e),n=JSON.stringify(r,null,2);if(s.value=n,a){let o={...t.playgroundArgs||{}};o[a]=n,t.playgroundArgs=o}}formatPlaygroundArgs(){let e=l.getState(),t=e.selectedCapabilityId||(e.capabilities[0]?e.capabilities[0].id:null),a=document.getElementById("pg-args-input");if(a)try{let i=JSON.parse(a.value||"{}"),s=JSON.stringify(i,null,2);if(a.value=s,t){let r={...e.playgroundArgs||{}};r[t]=s,e.playgroundArgs=r}}catch(i){alert(`Cannot format JSON: ${i.message}`)}}insertPlaygroundArgKey(e,t,a){let i=l.getState(),s=i.selectedCapabilityId||(i.capabilities[0]?i.capabilities[0].id:null),r=document.getElementById("pg-args-input");if(r){let n={};try{n=JSON.parse(r.value||"{}")}catch{n={}}if(n[e]===void 0)if(a!==null&&a!==void 0)n[e]=a;else switch(t){case"string":n[e]=`sample_${e}`;break;case"number":case"integer":n[e]=0;break;case"boolean":n[e]=!0;break;case"array":n[e]=[];break;case"object":n[e]={};break;default:n[e]=`sample_${e}`}let o=JSON.stringify(n,null,2);if(r.value=o,s){let c={...i.playgroundArgs||{}};c[s]=o,i.playgroundArgs=c}}}fillBatchStepSampleArgs(e){let t=l.getState(),a=[...t.batchSteps||[]],i=a[e];if(!i||!i.capability_id)return;let s=t.capabilities.find((c)=>c.id===i.capability_id);if(!s||!s.input_schema)return;let r=s.input_schema.properties||{},n={};for(let[c,d]of Object.entries(r))if(d.default!==void 0)n[c]=d.default;else if(Array.isArray(d.enum)&&d.enum.length>0)n[c]=d.enum[0];else switch(d.type||"string"){case"string":n[c]=`sample_${c}`;break;case"number":case"integer":n[c]=0;break;case"boolean":n[c]=!0;break;case"array":n[c]=[];break;case"object":n[c]={};break;default:n[c]=`sample_${c}`}let o=JSON.stringify(n,null,2);a[e]={...a[e],argsJson:o},l.setState({batchSteps:a})}filterCapabilities(e){let t=e.toLowerCase().trim(),i=l.getState().capabilities.filter((r)=>r.id.toLowerCase().includes(t)||r.summary&&r.summary.toLowerCase().includes(t)||r.server&&r.server.toLowerCase().includes(t)),s=document.getElementById("pg-cap-list");if(s)if(i.length===0)s.innerHTML=`
          <div style="padding: 24px 16px; text-align: center; color: var(--text-dim); font-size: 11.5px;">
            No capabilities match "${v(e)}"
          </div>
        `;else s.innerHTML=i.map((r)=>`
          <div class="cap-item ${r.id===l.getState().selectedCapabilityId?"active":""}" onclick="window.app.selectCapability('${v(r.id)}')">
            <div style="display: flex; justify-content: space-between; align-items: center;">
              <span style="font-weight: 600; color: var(--text-main); font-family: var(--ff-mono); font-size: 12px;">${v(r.id)}</span>
              <span style="font-size: 10px; color: var(--green-400);">${v(r.mode||"read")}</span>
            </div>
            <div style="font-size: 11px; color: var(--text-dim); margin-top: 2px;">server: ${v(r.server||"local")}</div>
          </div>
        `).join("")}async executePlaygroundTool(){let e=l.getState(),t=e.selectedCapabilityId||(e.capabilities[0]?e.capabilities[0].id:null);if(!t)return;let a=document.getElementById("pg-args-input")?.value||"{}",i=document.getElementById("pg-context-input")?.value||void 0,s=document.getElementById("pg-jsonpath-input")?.value.trim()||void 0,r=document.getElementById("pg-limit-lines-input")?.value.trim()||void 0,n=document.getElementById("pg-truncate-bytes-input")?.value.trim()||void 0,o={};try{o=JSON.parse(a)}catch{alert("Invalid arguments JSON object");return}if(s)o._jsonpath=s;if(r&&!isNaN(Number(r)))o._limit_lines=Number(r);if(n&&!isNaN(Number(n)))o._truncate_bytes=Number(n);let c=`op-${Date.now()}`;l.setState({isExecuting:!0,activeRequestId:c});let d=e.activeProfile||void 0;try{let p=await f.callCapability({capability_id:t,args:o,request_id:c,context:{operation_id:i||c}},d);l.setState({isExecuting:!1,activeRequestId:null,executionResult:{status:p.status,durationMs:p.durationMs,data:p.data}}),l.addEventLog("POST",`/v1/tools/call → ${t}`,p.status===200?"200 OK":`HTTP ${p.status}`,`${p.durationMs.toFixed(1)}ms`),f.getConfig().then((g)=>{if(g.ok&&g.circuit_breakers)l.setState({circuitBreakers:g.circuit_breakers})})}catch(p){l.setState({isExecuting:!1,activeRequestId:null,executionResult:{status:500,durationMs:0,data:{error:p.toString()}}})}}openBatchModal(){let e=l.getState(),t=e.batchSteps;if(!t||t.length===0)t=[{id:"step_1",capability_id:e.selectedCapabilityId||(e.capabilities[0]?e.capabilities[0].id:""),argsJson:"{}",continue_on_error:!1},{id:"step_2",capability_id:"",argsJson:"{}",continue_on_error:!0}],l.setState({batchSteps:t});l.setState({isBatchModalOpen:!0})}closeBatchModal(){l.setState({isBatchModalOpen:!1})}addBatchStep(){let t=[...l.getState().batchSteps||[]],a=t.length+1;t.push({id:`step_${a}`,capability_id:"",argsJson:"{}",continue_on_error:!1}),l.setState({batchSteps:t})}removeBatchStep(e){let a=[...l.getState().batchSteps||[]];if(a.length<=1){alert("Pipeline must contain at least one execution step.");return}a.splice(e,1);let i=a.map((s,r)=>({...s,id:`step_${r+1}`}));l.setState({batchSteps:i})}updateBatchStepCapability(e,t){let i=[...l.getState().batchSteps||[]];if(i[e])i[e]={...i[e],capability_id:t},l.setState({batchSteps:i})}updateBatchStepContinueOnError(e,t){let i=[...l.getState().batchSteps||[]];if(i[e])i[e]={...i[e],continue_on_error:t},l.setState({batchSteps:i})}updateBatchStepArgs(e,t){let a=l.getState(),i=[...a.batchSteps||[]];if(i[e])i[e]={...i[e],argsJson:t},a.batchSteps[e].argsJson=t}appendBatchVariable(e,t){let i=[...l.getState().batchSteps||[]],s=document.getElementById(`batch-step-args-${e}`);if(s){let r=s.value,n=s.selectionStart||r.length,o=s.selectionEnd||r.length,c=r.substring(0,n)+t+r.substring(o);if(s.value=c,i[e])i[e]={...i[e],argsJson:c},l.setState({batchSteps:i})}}async executeBatchPipeline(){let e=l.getState(),t=e.batchSteps||[],a=[];for(let s=0;s<t.length;s++){let r=t[s];if(!r.capability_id){alert(`Please select a capability for Step ${s+1}`);return}let n={};try{n=JSON.parse(r.argsJson||"{}")}catch{alert(`Invalid JSON in Step ${s+1} arguments`);return}a.push({id:r.id||`step_${s+1}`,capability_id:r.capability_id,args:n,continue_on_error:r.continue_on_error})}l.setState({isBatchModalOpen:!1});let i=e.activeProfile||void 0;try{let s=await f.batchCallCapabilities(a,i);l.setState({executionResult:{status:s.status,durationMs:s.durationMs,data:s.data}}),l.addEventLog("POST",`/v1/tools/batch_call (${t.length} steps)`,s.status===200?"200 OK":`HTTP ${s.status}`,`${s.durationMs.toFixed(1)}ms`)}catch(s){l.setState({executionResult:{status:500,durationMs:0,data:{error:s.toString()}}})}}async executeReadResource(){let e=l.getState(),t=e.selectedResourceId||(e.resources[0]?e.resources[0].id:null);if(!t)return;let a=document.getElementById("pg-res-jsonpath-input")?.value.trim()||void 0,i=document.getElementById("pg-res-lines-input")?.value.trim()||void 0,s=document.getElementById("pg-res-bytes-input")?.value.trim()||void 0,r={resource_id:t};if(a)r._jsonpath=a;if(i&&!isNaN(Number(i)))r._limit_lines=Number(i);if(s&&!isNaN(Number(s)))r._truncate_bytes=Number(s);let n=e.activeProfile||void 0;try{let o=await f.readResource({resource_id:t,input_responses:r},n);l.setState({resourceReadResult:{status:o.status,durationMs:o.durationMs,data:o.data}}),l.addEventLog("POST",`/v1/resources/read → ${t}`,o.status===200?"200 OK":`HTTP ${o.status}`,`${o.durationMs.toFixed(1)}ms`)}catch(o){l.setState({resourceReadResult:{status:500,durationMs:0,data:{error:o.toString()}}})}}async executeGetPrompt(){let e=l.getState(),t=e.selectedPromptId||(e.prompts[0]?e.prompts[0].id:null);if(!t)return;let a=document.querySelectorAll(".prompt-arg-input"),i={};a.forEach((r)=>{let n=r,o=n.getAttribute("data-arg-name");if(o&&n.value.trim())i[o]=n.value.trim()});let s=e.activeProfile||void 0;try{let r=await f.getPrompt({prompt_id:t,arguments:i},s);l.setState({promptGetResult:{status:r.status,durationMs:r.durationMs,data:r.data}}),l.addEventLog("POST",`/v1/prompts/get → ${t}`,r.status===200?"200 OK":`HTTP ${r.status}`,`${r.durationMs.toFixed(1)}ms`)}catch(r){l.setState({promptGetResult:{status:500,durationMs:0,data:{error:r.toString()}}})}}toggleBatchPlayground(){let e=document.getElementById("pg-args-input");if(!e)return;let t=[{id:"step_1",capability_id:"sqlite.read_query",args:{query:"SELECT * FROM users LIMIT 2"}},{id:"step_2",capability_id:"github.issues.search",args:{query:"label:bug"},continue_on_error:!0}];e.value=JSON.stringify(t,null,2)}async submitPolicyRule(e){let t=e==="allow"?"policy-new-allow":e==="deny"?"policy-new-deny":"policy-new-redact",a=document.getElementById(t);if(!a)return;let i=a.value.trim();if(!i)return;await this.addPolicyRule(e,i),a.value=""}async addPolicyRule(e,t){let a=(t||"").trim();if(!a)return;let s=l.getState().config.policy||{},r=[...s.allow||[]],n=[...s.deny||[]],o=[...s.redact_keys||s.redactKeys||[]];if(e==="allow"&&!r.includes(a))r.push(a);if(e==="deny"&&!n.includes(a))n.push(a);if(e==="redact"&&!o.includes(a))o.push(a);let c=await f.savePolicy({allow:r,deny:n,redact_keys:o,redactKeys:o});if(!c.ok)alert(`Failed to save policy rule: ${c.error||"Unknown error"}`);await this.refreshData()}async removePolicyRule(e,t){let i=l.getState().config.policy||{},s=[...i.allow||[]],r=[...i.deny||[]],n=[...i.redact_keys||i.redactKeys||[]];if(e==="allow")s.splice(t,1);if(e==="deny")r.splice(t,1);if(e==="redact")n.splice(t,1);let o=await f.savePolicy({allow:s,deny:r,redact_keys:n,redactKeys:n});if(!o.ok)alert(`Failed to update policy: ${o.error||"Unknown error"}`);await this.refreshData()}testPolicySandbox(e){let t=document.getElementById("policy-test-verdict");if(!t)return;let a=e.trim();if(!a){t.textContent="ENTER ID",t.style.color="var(--text-dim)";return}let s=l.getState().config.policy||{},r=s.deny||[],n=s.allow||[],o=(c,d)=>{if(c==="*")return!0;if(c.endsWith("*"))return d.startsWith(c.slice(0,-1));return c===d};if(r.some((c)=>o(c,a))){t.textContent="DENIED (Strict Block)",t.style.color="var(--red-400)";return}if(n.length>0&&!n.some((c)=>o(c,a))){t.textContent="DENIED (Not in Allow List)",t.style.color="var(--red-400)";return}t.textContent="ALLOWED",t.style.color="var(--green-400)"}async deleteServer(e){if(!confirm(`Are you sure you want to remove server '${e}' from config?`))return;await f.deleteServer(e),await this.refreshData()}openAddServerModal(){this.closeModals();let e=document.getElementById("modal-srv-title"),t=document.getElementById("modal-srv-template-banner"),a=document.getElementById("modal-srv-name"),i=document.getElementById("modal-srv-transport"),s=document.getElementById("modal-srv-command"),r=document.getElementById("modal-srv-url"),n=document.getElementById("modal-srv-ft"),o=document.getElementById("modal-srv-cd"),c=document.getElementById("modal-srv-autorestart"),d=document.getElementById("modal-srv-maxrestarts");if(e)e.textContent="Add Upstream MCP Server";if(t)t.style.display="flex";if(a)a.value="",a.disabled=!1;if(i)i.value="stdio";if(s)s.value="";if(r)r.value="";let p=document.getElementById("modal-group-cmd"),g=document.getElementById("modal-group-url");if(p)p.style.display="block";if(g)g.style.display="none";if(n)n.value="3";if(o)o.value="30000";if(c)c.value="true";if(d)d.value="5";let m=document.getElementById("modal-add-server");if(m)m.classList.add("active")}openEditServerModal(e){this.closeModals();let t=l.getState(),a=t.config.mcpServers?.[e];if(!a){alert(`Server '${e}' not found in configuration.`);return}let i=document.getElementById("modal-srv-title"),s=document.getElementById("modal-srv-template-banner"),r=document.getElementById("modal-srv-name"),n=document.getElementById("modal-srv-transport"),o=document.getElementById("modal-srv-command"),c=document.getElementById("modal-srv-url"),d=document.getElementById("modal-srv-ft"),p=document.getElementById("modal-srv-cd"),g=document.getElementById("modal-srv-autorestart"),m=document.getElementById("modal-srv-maxrestarts");if(i)i.textContent=`Edit Server '${e}'`;if(s)s.style.display="none";if(r)r.value=e,r.disabled=!0;let y=!!a.command;if(n)n.value=y?"stdio":"http";let h=document.getElementById("modal-group-cmd"),S=document.getElementById("modal-group-url");if(h)h.style.display=y?"block":"none";if(S)S.style.display=y?"none":"block";if(o)o.value=y?`${a.command} ${(a.args||[]).join(" ")}`.trim():"";if(c)c.value=a.url||"";let E=a.resilience||t.config.resilience;if(d)d.value=String(E?.failureThreshold??3);if(p)p.value=String(E?.cooldownMs??30000);if(g)g.value=E?.autoRestart===!1?"false":"true";if(m)m.value=String(E?.maxRestarts??5);let w=document.getElementById("modal-add-server");if(w)w.classList.add("active")}async submitAddServer(){let e=document.getElementById("modal-srv-name"),t=e?.value.trim(),a=document.getElementById("modal-srv-transport")?.value;if(!t){alert("Server name is required");return}if(e&&!e.disabled){if((l.getState().config.mcpServers||{})[t]){if(!confirm(`Server '${t}' already exists in configuration. Do you want to overwrite it?`))return}}let i={};if(a==="stdio"){let p=(document.getElementById("modal-srv-command")?.value.trim()).split(/\s+/).filter(Boolean);if(p.length===0){alert("Command is required");return}i.command=p[0],i.args=p.slice(1)}else{let d=document.getElementById("modal-srv-url")?.value.trim();if(!d){alert("URL is required");return}i.url=d}let s=document.getElementById("modal-srv-ft")?.value.trim(),r=document.getElementById("modal-srv-cd")?.value.trim(),n=document.getElementById("modal-srv-autorestart")?.value,o=document.getElementById("modal-srv-maxrestarts")?.value.trim();if(s||r||n||o)i.resilience={failureThreshold:s?Number(s):3,cooldownMs:r?Number(r):30000,autoRestart:n!=="false",maxRestarts:o?Number(o):5};let c=await f.upsertServer(t,i);if(c.ok)this.closeModals(),await this.refreshData();else alert(`Failed to save server: ${c.error}`)}openTemplateCatalog(){this.closeModals();let e=document.getElementById("modal-templates");if(e)e.classList.add("active");this.renderTemplateGrid()}setTemplateCategory(e){this.activeTemplateCategory=e,document.querySelectorAll(".tmpl-cat-btn").forEach((t)=>{if(t.getAttribute("data-category")===e)t.classList.add("active"),t.style.background="var(--surface-elevated)",t.style.color="var(--amber-400)";else t.classList.remove("active"),t.style.background="var(--surface-card)",t.style.color="var(--text-main)"}),this.renderTemplateGrid()}filterTemplates(e){this.activeTemplateFilter=e.toLowerCase().trim(),this.renderTemplateGrid()}renderTemplateGrid(){let e=document.getElementById("tmpl-grid");if(!e)return;let t=B.filter((s)=>{let r=this.activeTemplateCategory==="all"||s.category===this.activeTemplateCategory,n=!this.activeTemplateFilter||s.name.toLowerCase().includes(this.activeTemplateFilter)||s.id.toLowerCase().includes(this.activeTemplateFilter)||s.description.toLowerCase().includes(this.activeTemplateFilter)||s.command.toLowerCase().includes(this.activeTemplateFilter)||s.envFields.some((o)=>o.key.toLowerCase().includes(this.activeTemplateFilter));return r&&n});if(t.length===0){e.innerHTML=`
        <div style="grid-column: span 2; padding: 32px; text-align: center; color: var(--text-dim);">
          No matching MCP server templates found.
        </div>
      `;return}let i=l.getState().config.mcpServers||{};e.innerHTML=t.map((s)=>{let r=!!i[s.id],n=`${s.command} ${s.defaultArgs.join(" ")}`;return`
        <div class="bento-card" style="display: flex; flex-direction: column; justify-content: space-between; padding: 14px; background: var(--surface); border: 1px solid var(--border); min-width: 0; transition: transform 0.15s, border-color 0.15s;">
          <div>
            <div style="display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 6px;">
              <div style="display: flex; align-items: center; gap: 8px; min-width: 0;">
                <span style="font-weight: 700; font-size: 13.5px; color: var(--text-main); white-space: nowrap; overflow: hidden; text-overflow: ellipsis;">${v(s.name)}</span>
                <span class="brand-badge" style="font-size: 9.5px; padding: 1px 6px; flex-shrink: 0;">${v(s.badge)}</span>
              </div>
              ${r?'<span style="font-size: 10px; color: var(--green-400); font-weight: 600; flex-shrink: 0;">CONNECTED</span>':""}
            </div>
            <div style="font-size: 11.5px; color: var(--text-muted); line-height: 1.4; margin-bottom: 8px;">
              ${v(s.description)}
            </div>
            <div style="font-family: var(--ff-mono); font-size: 10.5px; color: var(--text-dim); background: var(--surface-card); padding: 5px 8px; border-radius: var(--radius-sm); border: 1px solid var(--border-subtle); overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">
              <code>${v(n)}</code>
            </div>
            ${s.envFields.length>0?`
              <div style="font-size: 10.5px; color: var(--amber-400); margin-top: 6px; display: flex; align-items: center; gap: 4px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">
                <span>⚡ Needs:</span>
                <code style="overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">${s.envFields.map((o)=>v(o.key)).join(", ")}</code>
              </div>
            `:""}
          </div>

          <div style="display: flex; justify-content: flex-end; margin-top: 12px; gap: 6px;">
            <button class="btn btn-primary" style="font-size: 11.5px; padding: 4px 10px;" onclick="window.app.selectTemplate('${v(s.id)}')">
              ${r?"Configure Another":"✨ 1-Click Setup"}
            </button>
          </div>
        </div>
      `}).join("")}selectTemplate(e){let t=B.find((c)=>c.id===e);if(!t)return;this.selectedTemplate=t,this.closeModals();let a=document.getElementById("modal-configure-template");if(a)a.classList.add("active");let i=document.getElementById("cfg-tmpl-title"),s=document.getElementById("cfg-tmpl-desc"),r=document.getElementById("cfg-tmpl-form");if(i)i.textContent=`Configure ${t.name} Server`;if(s)s.textContent=t.description;let n=l.getState().config.mcpServers||{},o=t.id;if(n[o]){let c=2;while(n[`${t.id}-${c}`])c++;o=`${t.id}-${c}`}if(r){let c="";if(t.envFields.length>0)c=`
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
        `;r.innerHTML=`
        <div class="form-group">
          <label class="form-label">Server Identifier (Name)</label>
          <input type="text" class="form-input" id="cfg-srv-id" value="${v(o)}">
          <div style="font-size: 10.5px; color: var(--text-dim); margin-top: 3px;">Must be unique across all configured servers.</div>
        </div>
        <div class="form-group">
          <label class="form-label">Command Line Arguments</label>
          <input type="text" class="form-input" id="cfg-srv-args" value="${v(t.defaultArgs.join(" "))}" placeholder="${v(t.argsPlaceholder||"")}">
          <div style="font-size: 10.5px; color: var(--text-dim); margin-top: 3px;">Executable: <code>${v(t.command)}</code></div>
        </div>
        ${c}
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
      `}}async submitTemplateServer(){if(!this.selectedTemplate)return;let e=this.selectedTemplate,t=document.getElementById("cfg-srv-id")?.value.trim(),a=document.getElementById("cfg-srv-args")?.value.trim();if(!t){alert("Server identifier is required");return}if((l.getState().config.mcpServers||{})[t]){if(!confirm(`Server '${t}' already exists. Do you want to overwrite its configuration?`))return}let r=a?a.split(/\s+/).filter(Boolean):[],n={},o=document.querySelectorAll(".tmpl-env-input");for(let h of Array.from(o)){let S=h.getAttribute("data-key"),E=h.value.trim(),w=e.envFields.find((k)=>k.key===S);if(w?.required&&!E){alert(`Required field '${w.label}' is missing.`);return}if(S&&E)n[S]=E}let c={command:e.command,args:r};if(Object.keys(n).length>0)c.env=n;let d=document.getElementById("cfg-srv-ft")?.value.trim(),p=document.getElementById("cfg-srv-cd")?.value.trim(),g=document.getElementById("cfg-srv-autorestart")?.value,m=document.getElementById("cfg-srv-maxrestarts")?.value.trim();if(d||p||g||m)c.resilience={failureThreshold:d?Number(d):3,cooldownMs:p?Number(p):30000,autoRestart:g!=="false",maxRestarts:m?Number(m):5};let y=await f.upsertServer(t,c);if(y.ok)this.closeModals(),await this.refreshData();else alert(`Failed to save server: ${y.error}`)}async openImportModal(){this.closeModals();let e=document.getElementById("modal-import");if(e)e.classList.add("active");let t=document.getElementById("modal-eco-list");if(!t)return;t.innerHTML='<div style="color: var(--text-dim); padding: 12px; text-align: center;">Scanning IDE configs...</div>';try{let a=await f.getEcosystemSources();if(a.sources&&a.sources.length>0)t.innerHTML=a.sources.map((i)=>`
          <label style="display: flex; align-items: center; gap: 10px; background: var(--surface); padding: 10px 14px; border-radius: var(--radius-sm); border: 1px solid var(--border); cursor: pointer;">
            <input type="checkbox" class="eco-checkbox" value="${i.path}" checked>
            <div>
              <div style="font-weight: 600; color: var(--text-main);">${i.name}</div>
              <div style="font-size: 11px; color: var(--text-dim);">${i.server_count} servers (${i.servers.join(", ")})</div>
            </div>
          </label>
        `).join("");else t.innerHTML='<div style="color: var(--text-dim); padding: 12px; text-align: center;">No external MCP configuration files found on this system.</div>'}catch{t.innerHTML='<div style="color: var(--red-400); padding: 12px; text-align: center;">Failed to scan ecosystem sources.</div>'}}async submitImport(){let e=document.querySelectorAll(".eco-checkbox:checked");if(e.length===0){alert("No sources selected");return}for(let t of Array.from(e))await f.importConfig(t.value,!1);this.closeModals(),await this.refreshData()}handleAliasTargetInput(e){let t=document.getElementById("alias-suggestions-dropdown");if(!t)return;let a=(e||"").trim().toLowerCase();if(a.length<2){t.style.display="none";return}let s=l.getState().capabilities.filter((r)=>r.id.toLowerCase().includes(a)||r.summary&&r.summary.toLowerCase().includes(a)||r.description&&r.description.toLowerCase().includes(a)||r.server&&r.server.toLowerCase().includes(a)).slice(0,8);if(s.length===0){t.style.display="none";return}t.innerHTML=s.map((r)=>`
      <div style="padding: 8px 12px; cursor: pointer; border-bottom: 1px solid var(--border-subtle); display: flex; justify-content: space-between; align-items: center; transition: background 0.1s;"
           onmouseover="this.style.background='var(--surface-hover)'"
           onmouseout="this.style.background='transparent'"
           onmousedown="window.app.selectAliasSuggestion('${v(r.id)}')">
        <div>
          <div style="font-weight: 700; color: var(--text-main);">${v(r.id)}</div>
          <div style="font-size: 10.5px; color: var(--text-dim); margin-top: 2px;">${v(r.summary||r.description||"")}</div>
        </div>
        <span style="font-size: 10px; color: var(--cyan-400);">${v(r.server||"local")}</span>
      </div>
    `).join(""),t.style.display="block"}selectAliasSuggestion(e){let t=document.getElementById("alias-target");if(t)t.value=e;this.hideAliasDropdown()}hideAliasDropdown(){let e=document.getElementById("alias-suggestions-dropdown");if(e)e.style.display="none"}async createAlias(){let e=document.getElementById("alias-kind")?.value,t=document.getElementById("alias-name")?.value.trim(),a=document.getElementById("alias-target")?.value.trim();if(!t||!a){alert("Please provide both alias name and canonical target");return}await f.updateAlias(e,t,a),await this.refreshData()}async deleteAlias(e,t){await f.updateAlias(e,t,void 0),await this.refreshData()}async reloadFromDisk(){try{let e=await f.reloadConfig();if(e.ok){let t="Hot-reload completed successfully!";if(e.mounted&&e.mounted.length>0)t+=`
Mounted: ${e.mounted.join(", ")}`;if(e.unmounted&&e.unmounted.length>0)t+=`
Unmounted: ${e.unmounted.join(", ")}`;if(e.warnings&&e.warnings.length>0)t+=`
Warnings:
${e.warnings.join(`
`)}`;alert(t)}else alert(`Hot-reload failed: ${e.error||"Unknown error"}`)}catch(e){alert(`Error reaching daemon: ${e.message}`)}await this.refreshData()}renderTopProfileSelector(){let e=document.getElementById("top-profile-selector");if(!e)return;let t=l.getState(),a=t.config.profiles||{},i=Object.keys(a),s=t.activeProfile,r='<option value="">All Servers (Unrestricted)</option>';for(let n of i){let o=s===n?"selected":"";r+=`<option value="${v(n)}" ${o}>Profile: ${v(n)}</option>`}e.innerHTML=r}async setActiveProfile(e){l.setState({activeProfile:e||null}),await this.refreshData()}openAddProfileModal(){let e=document.getElementById("modal-prof-title");if(e)e.textContent="Create Server Constellation Profile";let t=document.getElementById("modal-prof-name"),a=document.getElementById("modal-prof-desc"),i=document.getElementById("modal-prof-mode");if(t)t.value="",t.disabled=!1;if(a)a.value="";if(i)i.value="create";this.renderProfileServerCheckboxes([]);let s=document.getElementById("modal-add-profile");if(s)s.classList.add("active")}openEditProfileModal(e){let a=l.getState().config.profiles?.[e];if(!a)return;let i=document.getElementById("modal-prof-title");if(i)i.textContent=`Edit Profile: ${e}`;let s=document.getElementById("modal-prof-name"),r=document.getElementById("modal-prof-desc"),n=document.getElementById("modal-prof-mode");if(s)s.value=e,s.disabled=!0;if(r)r.value=a.description||"";if(n)n.value="edit";this.renderProfileServerCheckboxes(a.servers||[]);let o=document.getElementById("modal-add-profile");if(o)o.classList.add("active")}renderProfileServerCheckboxes(e){let t=document.getElementById("modal-prof-servers-list");if(!t)return;let a=l.getState(),i=Object.keys(a.config.mcpServers||{});if(i.length===0){t.innerHTML='<div style="font-size: 11.5px; color: var(--text-dim);">No MCP servers configured yet. Add servers first.</div>';return}t.innerHTML=i.map((s)=>{let r=e.includes(s)?"checked":"";return`
        <label style="display: flex; align-items: center; gap: 8px; font-size: 12px; cursor: pointer; padding: 4px 6px; border-radius: var(--radius-sm); transition: background 0.15s;" onmouseover="this.style.background='var(--surface-hover)'" onmouseout="this.style.background='transparent'">
          <input type="checkbox" class="prof-server-checkbox" value="${v(s)}" ${r} style="accent-color: var(--amber-400);">
          <span style="font-family: var(--ff-mono); font-weight: 600; color: var(--text-main);">${v(s)}</span>
        </label>
      `}).join("")}async saveProfile(){let e=document.getElementById("modal-prof-name"),t=document.getElementById("modal-prof-desc"),a=e?.value.trim(),i=t?.value.trim();if(!a){alert("Please enter a profile name");return}let s=document.querySelectorAll(".prof-server-checkbox:checked"),r=[];if(s.forEach((n)=>{r.push(n.value)}),r.length===0){alert("Please select at least one server to include in this constellation");return}try{let n=await f.upsertProfile(a,r,i||void 0);if(n.ok)this.closeModals(),await this.refreshData();else alert(`Failed to save profile: ${n.error||"Unknown error"}`)}catch(n){alert(`Error saving profile: ${n.message}`)}}async deleteProfile(e){if(!confirm(`Are you sure you want to delete profile '${e}'?`))return;try{let t=await f.deleteProfile(e);if(t.ok){if(l.getState().activeProfile===e)l.setState({activeProfile:null});await this.refreshData()}else alert(`Failed to delete profile: ${t.error||"Unknown error"}`)}catch(t){alert(`Error deleting profile: ${t.message}`)}}closeModals(){document.querySelectorAll(".modal-backdrop").forEach((e)=>e.classList.remove("active"))}}function v(e){return String(e||"").replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;")}var Q=new W;window.app=Q;window.addEventListener("DOMContentLoaded",()=>Q.init());
