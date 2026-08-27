class Z{state={configPath:"mcp_servers.json",config:{mcpServers:{}},serverStatuses:{},circuitBreakers:[],clients:[],secrets:[],clientsCollapsed:!1,capabilities:[],capabilitiesHiddenByPolicy:0,resources:[],resourcesHiddenByPolicy:0,prompts:[],promptsHiddenByPolicy:0,catalogEvents:[],tasks:[],selectedTaskId:null,taskFilterStatus:"all",approvals:[],auditEvents:[],auditTotal:0,auditFilters:{search:"",status:"all",eventType:"all",serverId:"all",limit:25,offset:0},auditSelectedEvent:null,auditStats:null,auditVerification:null,selectedCapabilityId:null,selectedResourceId:null,selectedPromptId:null,playgroundMode:"tools",playgroundArgs:{},isExecuting:!1,playgroundAsyncTask:!1,activeRequestId:null,isBatchModalOpen:!1,batchSteps:[{id:"step_1",capability_id:"",argsJson:"{}",continue_on_error:!1},{id:"step_2",capability_id:"",argsJson:"{}",continue_on_error:!0}],activeTab:"overview",activeProfile:null,eventLogs:[],executionResult:null,resourceReadResult:null,promptGetResult:null,metrics:{totalCatalogRequests:0,totalEtagHits:0,totalToolCalls:0,totalToolDurationUs:0}};listeners=[];getState(){return this.state}setState(e){this.state={...this.state,...e},this.listeners.forEach((t)=>t(this.state))}subscribe(e){return this.listeners.push(e),()=>{this.listeners=this.listeners.filter((t)=>t!==e)}}addEventLog(e,t,a,n){let s=[{time:new Date().toLocaleTimeString(),method:e,target:t,status:a,latency:n},...this.state.eventLogs].slice(0,50);this.setState({eventLogs:s})}}var d=new Z;class ee{baseUrl;constructor(e=""){this.baseUrl=e}async getConfig(){return(await fetch(`${this.baseUrl}/v1/config`)).json()}async listCapabilities(e){let t={};if(e)t["X-Warmplane-Profile"]=e;return(await fetch(`${this.baseUrl}/v1/capabilities`,{headers:t})).json()}async listResources(e){let t={};if(e)t["X-Warmplane-Profile"]=e;return(await fetch(`${this.baseUrl}/v1/resources`,{headers:t})).json()}async readResource(e,t){let a=performance.now(),n={"Content-Type":"application/json"};if(t)n["X-Warmplane-Profile"]=t;let r=await fetch(`${this.baseUrl}/v1/resources/read`,{method:"POST",headers:n,body:JSON.stringify(e)}),s=performance.now()-a,o=await r.json();return{status:r.status,durationMs:s,data:o}}async listPrompts(e){let t={};if(e)t["X-Warmplane-Profile"]=e;return(await fetch(`${this.baseUrl}/v1/prompts`,{headers:t})).json()}async getPrompt(e,t){let a=performance.now(),n={"Content-Type":"application/json"};if(t)n["X-Warmplane-Profile"]=t;let r=await fetch(`${this.baseUrl}/v1/prompts/get`,{method:"POST",headers:n,body:JSON.stringify(e)}),s=performance.now()-a,o=await r.json();return{status:r.status,durationMs:s,data:o}}async getCatalogEvents(e){let t=e?`?after=${encodeURIComponent(e)}`:"";return(await fetch(`${this.baseUrl}/v1/catalog/events${t}`)).json()}async callCapability(e,t){let a=performance.now(),n={"Content-Type":"application/json"};if(t)n["X-Warmplane-Profile"]=t;let r=await fetch(`${this.baseUrl}/v1/tools/call`,{method:"POST",headers:n,body:JSON.stringify(e)}),s=performance.now()-a,o=await r.json();return{status:r.status,durationMs:s,data:o}}async batchCallCapabilities(e,t){let a=performance.now(),n={"Content-Type":"application/json"};if(t)n["X-Warmplane-Profile"]=t;let r=await fetch(`${this.baseUrl}/v1/tools/batch_call`,{method:"POST",headers:n,body:JSON.stringify({steps:e})}),s=performance.now()-a,o=await r.json();return{status:r.status,durationMs:s,data:o}}async cancelOperation(e){return(await fetch(`${this.baseUrl}/v1/operations/${encodeURIComponent(e)}/cancel`,{method:"POST"})).json()}async completeArgument(e){return(await fetch(`${this.baseUrl}/v1/completion/complete`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify(e)})).json()}async upsertServer(e,t){return(await fetch(`${this.baseUrl}/v1/config/servers`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({name:e,server:t})})).json()}async deleteServer(e){return(await fetch(`${this.baseUrl}/v1/config/servers/${encodeURIComponent(e)}`,{method:"DELETE"})).json()}async restartServer(e){return(await fetch(`${this.baseUrl}/v1/config/servers/${encodeURIComponent(e)}/restart`,{method:"POST"})).json()}async upsertProfile(e,t,a,n){return(await fetch(`${this.baseUrl}/v1/config/profiles`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({name:e,servers:t,description:a,policy:n})})).json()}async deleteProfile(e){return(await fetch(`${this.baseUrl}/v1/config/profiles/${encodeURIComponent(e)}`,{method:"DELETE"})).json()}async getEcosystemSources(){return(await fetch(`${this.baseUrl}/v1/config/ecosystem`)).json()}async importConfig(e,t=!1){return(await fetch(`${this.baseUrl}/v1/config/import`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({source_path:e,overwrite:t})})).json()}async savePolicy(e){let t={allow:e.allow||[],deny:e.deny||[],redactKeys:e.redact_keys||e.redactKeys||[],requireApproval:e.require_approval||e.requireApproval||[],approvalTimeoutSecs:e.approvalTimeoutSecs||e.approval_timeout_secs||300,webhook:e.webhook};return(await fetch(`${this.baseUrl}/v1/config/policy`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify(t)})).json()}async listTasks(){return(await fetch(`${this.baseUrl}/v1/tasks`)).json()}async getTask(e){return(await fetch(`${this.baseUrl}/v1/tasks/${encodeURIComponent(e)}`)).json()}async updateTask(e,t){return(await fetch(`${this.baseUrl}/v1/tasks/${encodeURIComponent(e)}/update`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({inputResponses:t})})).json()}async cancelTask(e,t){return(await fetch(`${this.baseUrl}/v1/tasks/${encodeURIComponent(e)}/cancel`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({reason:t})})).json()}async listApprovals(){return(await fetch(`${this.baseUrl}/v1/approvals`)).json()}async approveTicket(e,t,a){return(await fetch(`${this.baseUrl}/v1/approvals/${encodeURIComponent(e)}/approve`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({operator:t,modified_args:a})})).json()}async rejectTicket(e,t,a){return(await fetch(`${this.baseUrl}/v1/approvals/${encodeURIComponent(e)}/reject`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({operator:t,reason:a})})).json()}async updateAlias(e,t,a){return(await fetch(`${this.baseUrl}/v1/config/alias`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({kind:e,alias:t,target:a})})).json()}async reloadConfig(){return(await fetch(`${this.baseUrl}/v1/config/reload`,{method:"POST"})).json()}async listAuditEvents(e){let t=new URLSearchParams;if(e?.actor_id)t.set("actor_id",e.actor_id);if(e?.server_id&&e.server_id!=="all")t.set("server_id",e.server_id);if(e?.capability_id)t.set("capability_id",e.capability_id);if(e?.event_type&&e.event_type!=="all")t.set("event_type",e.event_type);if(e?.status&&e.status!=="all")t.set("status",e.status);if(e?.trace_id)t.set("trace_id",e.trace_id);if(e?.request_id)t.set("request_id",e.request_id);if(e?.search)t.set("search",e.search);if(e?.limit)t.set("limit",String(e.limit));if(e?.offset!==void 0)t.set("offset",String(e.offset));let a=t.toString();return(await fetch(`${this.baseUrl}/v1/audit/events${a?`?${a}`:""}`)).json()}getAuditExportUrl(e,t="csv"){let a=new URLSearchParams;if(a.set("format",t),e?.actor_id)a.set("actor_id",e.actor_id);if(e?.server_id&&e.server_id!=="all")a.set("server_id",e.server_id);if(e?.capability_id)a.set("capability_id",e.capability_id);if(e?.event_type&&e.event_type!=="all")a.set("event_type",e.event_type);if(e?.status&&e.status!=="all")a.set("status",e.status);if(e?.trace_id)a.set("trace_id",e.trace_id);if(e?.request_id)a.set("request_id",e.request_id);if(e?.search)a.set("search",e.search);return`${this.baseUrl}/v1/audit/export?${a.toString()}`}async verifyAuditChain(){return(await fetch(`${this.baseUrl}/v1/audit/verify`)).json()}async getAuditStats(){return(await fetch(`${this.baseUrl}/v1/audit/stats`)).json()}async getClients(){return(await fetch(`${this.baseUrl}/v1/clients`)).json()}async attachClient(e,t){return(await fetch(`${this.baseUrl}/v1/clients/${encodeURIComponent(e)}/attach`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({profile:t||void 0})})).json()}async detachClient(e){return(await fetch(`${this.baseUrl}/v1/clients/${encodeURIComponent(e)}/detach`,{method:"POST",headers:{"Content-Type":"application/json"}})).json()}async testWebhook(e,t){return(await fetch(`${this.baseUrl}/v1/webhooks/test`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({url:e||void 0,format:t||void 0})})).json()}async getSecrets(){return(await fetch(`${this.baseUrl}/v1/secrets`)).json()}async saveSecret(e,t,a){return(await fetch(`${this.baseUrl}/v1/secrets`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({key:e,value:t,service:a})})).json()}async deleteSecret(e){return(await fetch(`${this.baseUrl}/v1/secrets/${encodeURIComponent(e)}`,{method:"DELETE"})).json()}}var m=new ee;function te(){let e=d.getState(),t=e.config.mcpServers||{},a=Object.keys(t),n=a.length,r="";if(a.length===0)r=`
      <div style="grid-column: 1 / -1; padding: 32px; text-align: center; color: var(--text-dim); background: var(--surface-card); border-radius: var(--radius-md); border: 1px dashed var(--border);">
        <div style="font-size: 14px; color: var(--text-main); font-weight: 600; margin-bottom: 6px;">No Upstream MCP Servers Connected</div>
        <div style="font-size: 12px; margin-bottom: 16px;">Initialize connections by adding a server or syncing existing IDE configurations.</div>
        <div style="display: flex; gap: 8px; justify-content: center;">
          <button class="btn btn-primary" onclick="window.app.openTemplateCatalog()">✨ Browse Templates</button>
          <button class="btn btn-ghost" onclick="window.app.openAddServerModal()">+ Add Custom</button>
          <button class="btn btn-ghost" onclick="window.app.openImportModal()">Sync from IDEs</button>
        </div>
      </div>
    `;else r=a.map((E)=>{let j=t[E],K=j.command?"stdio":"http / sse",F=j.command?`${j.command} ${(j.args||[]).join(" ")}`:j.url,R=e.serverStatuses[E]||{status:"connected",protocol_version:"2026-07-28"},q=R.status==="degraded",Y=R.status==="error"||R.status==="disconnected",J=q?"var(--amber-400)":Y?"var(--red-400)":"var(--green-400)";return`
        <div class="bento-card col-4" style="background: var(--surface); border: 1px solid var(--border);">
          <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
            <span style="font-weight: 700; color: var(--text-main); display: flex; align-items: center; gap: 6px;">
              <span style="width: 8px; height: 8px; border-radius: 50%; background: ${J}; display: inline-block;"></span>
              ${P(E)}
            </span>
            <span class="brand-badge">${K}</span>
          </div>
          <div style="font-family: var(--ff-mono); font-size: 11.5px; color: var(--text-dim); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; margin-bottom: 12px;" title="${P(F||"")}">
            ${P(F||"")}
          </div>
          <div style="display: flex; justify-content: space-between; font-size: 11px; color: var(--text-muted); border-top: 1px solid var(--border); padding-top: 8px;">
            <span>Status: <strong style="color: ${J};">${P(R.status)}</strong></span>
            <span>Protocol: ${R.protocol_version}</span>
          </div>
        </div>
      `}).join("");let s=e.eventLogs.length===0?`
    <div class="feed-row" style="grid-template-columns: 80px 100px 1fr 100px 80px;">
      <span style="color: var(--text-dim);">ready</span>
      <span style="color: var(--cyan-400); font-weight: 600;">SSE</span>
      <span style="color: var(--text-main);">/v1/resources/updates stream active</span>
      <span style="color: var(--green-400);">CONNECTED</span>
      <span style="color: var(--amber-300); text-align: right;">0.0ms</span>
    </div>
  `:e.eventLogs.map((E)=>`
    <div class="feed-row" style="grid-template-columns: 80px 100px 1fr 100px 80px;">
      <span style="color: var(--text-dim);">${P(E.time)}</span>
      <span style="color: var(--cyan-400); font-weight: 600;">${P(E.method)}</span>
      <span style="color: var(--text-main); font-family: var(--ff-mono);">${P(E.target)}</span>
      <span style="color: var(--green-400);">${P(E.status)}</span>
      <span style="color: var(--amber-300); text-align: right;">${P(E.latency)}</span>
    </div>
  `).join(""),o=e.metrics,i=o.totalCatalogRequests,l=o.totalEtagHits,p=i>0?`${(l/i*100).toFixed(1)}%`:"0.0%",u=i>0?`${l} of ${i} requests served via HTTP 304`:"Waiting for client requests",g=o.totalToolCalls,f=g>0?`${(o.totalToolDurationUs/g/1000).toFixed(1)}ms`:"0.0ms",c=g>0?`${g} tool executions processed`:"Local worker task queues warm",y=Object.keys(e.config.capabilityAliases||{}).length+Object.keys(e.config.resourceAliases||{}).length+Object.keys(e.config.promptAliases||{}).length,h=y>0?`${y*18}B / call`:"0B",v=y>0?`${y} active facade aliases pruning prompt size`:"Configure aliases in Studio to reduce prompt size",w=e.tasks||[],C=w.filter((E)=>E.status==="input_required").length,I=w.filter((E)=>E.status==="working"||E.status==="input_required").length,A=e.clients||[],L=A.filter((E)=>E.is_attached).length,k=A.filter((E)=>E.config_exists&&!E.is_attached).length,M=e.clientsCollapsed,B=L>0?`<span class="brand-badge" style="color: var(--green-400); border-color: rgba(52, 211, 153, 0.3); background: rgba(52, 211, 153, 0.1);">⚡ ${L} Connected</span>`:k>0?`<span class="brand-badge" style="color: var(--amber-300); border-color: rgba(251, 191, 36, 0.3); background: rgba(251, 191, 36, 0.1);">○ ${k} Ready to Connect</span>`:'<span class="brand-badge" style="color: var(--text-dim);">No Apps Detected</span>',D=Object.keys(e.config.profiles||{}),U=e.activeProfile,G=A.map((E)=>{let{is_attached:j,config_exists:K,app_installed:F}=E,R="rgba(255, 255, 255, 0.2)",q="Not Found";if(j)R="var(--green-400)",q=E.attached_profile?`Connected (${E.attached_profile})`:"Connected (All Tools)";else if(K)R="var(--amber-300)",q="Ready to Attach";else if(F)R="var(--cyan-400)",q="Installed";let Y=D.map((V)=>`
      <option value="${P(V)}" ${U===V||E.attached_profile===V?"selected":""}>${P(V)}</option>
    `).join(""),J=j?`<button class="btn btn-ghost" style="padding: 2px 7px; font-size: 10px; color: var(--red-400);" onclick="event.stopPropagation(); window.app.detachClient('${P(E.id)}')">Detach</button>`:K||F?`
        <div style="display: flex; align-items: center; gap: 4px;" onclick="event.stopPropagation();">
          ${D.length>0?`
            <select id="overview-client-prof-${P(E.id)}" class="form-input" style="font-size: 10px; padding: 1px 4px; height: 22px; width: 85px;" title="Select constellation profile">
              <option value="" ${!U?"selected":""}>All Tools</option>
              ${Y}
            </select>
          `:""}
          <button class="btn btn-primary" style="padding: 2px 7px; font-size: 10px;" onclick="window.app.attachClient('${P(E.id)}')">⚡ Connect</button>
        </div>
      `:"";return`
      <div style="background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 8px 12px; display: flex; align-items: center; justify-content: space-between; gap: 8px;">
        <div style="display: flex; align-items: center; gap: 8px; overflow: hidden;">
          <span style="width: 7px; height: 7px; border-radius: 50%; background: ${R}; flex-shrink: 0;"></span>
          <div style="overflow: hidden;">
            <div style="font-weight: 600; font-size: 12px; color: var(--text-main); white-space: nowrap; overflow: hidden; text-overflow: ellipsis;">${P(E.name)}</div>
            <div style="font-size: 10px; color: var(--text-dim);">${P(q)}</div>
          </div>
        </div>
        ${J}
      </div>
    `}).join(""),Q=`
    <div class="bento-card" style="margin-top: 18px; padding: 12px 16px; border-color: rgba(245, 158, 11, 0.25); background: rgba(18, 24, 38, 0.4);">
      <div style="display: flex; justify-content: space-between; align-items: center; cursor: pointer; user-select: none;" onclick="window.app.toggleClientsCollapse()">
        <div style="display: flex; align-items: center; gap: 10px;">
          <span style="font-size: 13.5px; font-weight: 700; color: var(--text-main); display: flex; align-items: center; gap: 6px;">
            <span>⚡ 1-Click AI Client Integrations</span>
          </span>
          ${B}
        </div>
        <div style="display: flex; align-items: center; gap: 8px;">
          <button class="btn btn-ghost" style="padding: 2px 8px; font-size: 11px;" onclick="event.stopPropagation(); window.app.refreshClients()">⟳ Scan</button>
          <span style="font-size: 12px; color: var(--text-dim);">${M?"▼ Show":"▲ Hide"}</span>
        </div>
      </div>

      ${!M?`
        <div style="display: grid; grid-template-columns: repeat(auto-fill, minmax(270px, 1fr)); gap: 10px; margin-top: 12px; padding-top: 10px; border-top: 1px solid var(--border-subtle);">
          ${G}
        </div>
      `:""}
    </div>
  `;return`
    <div class="bento-grid">
      <div class="bento-card col-3">
        <div class="stat-label">Token Savings Rate</div>
        <div class="stat-value" style="color: var(--amber-300);">${h}</div>
        <div class="stat-sub">${v}</div>
      </div>
      <div class="bento-card col-3">
        <div class="stat-label">ETag Cache Hit Rate</div>
        <div class="stat-value" style="color: var(--cyan-400);">${p}</div>
        <div class="stat-sub">${u}</div>
      </div>
      <div class="bento-card col-3">
        <div class="stat-label">Tasks &amp; HITL State</div>
        <div class="stat-value" style="color: ${C>0?"var(--amber-400)":"var(--green-400)"};">${C>0?`${C} Action Req`:`${I} Active`}</div>
        <div class="stat-sub">${C>0?"Awaiting Human-in-the-Loop decision":`${w.length} total registered tasks`}</div>
      </div>
      <div class="bento-card col-3">
        <div class="stat-label">Connected Upstreams</div>
        <div class="stat-value" style="color: var(--green-400);">${n} Active</div>
        <div class="stat-sub">${n>0?"Persistent worker task channels":"No active upstream servers"}</div>
      </div>
    </div>

    ${Q}

    <div style="display: flex; justify-content: space-between; align-items: center; margin: 24px 0 12px;">
      <div style="font-size: 15px; font-weight: 700; color: var(--text-main);">Connected Upstream Servers</div>
      <button class="btn btn-ghost" onclick="window.app.switchTab('servers')">Manage All (${n}) →</button>
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
        ${s}
      </div>
    </div>
  `}function P(e){return String(e||"").replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;")}function se(){let e=d.getState(),t=e.config.mcpServers||{},a=Object.keys(t),n="";if(a.length===0)n=`
      <div style="padding: 40px; text-align: center; color: var(--text-dim); background: var(--surface-card); border-radius: var(--radius-md); border: 1px dashed var(--border);">
        <div style="font-size: 15px; color: var(--text-main); font-weight: 600; margin-bottom: 8px;">No Servers Configured in ${_(e.configPath)}</div>
        <p style="font-size: 12px; margin-bottom: 20px; max-width: 480px; margin-left: auto; margin-right: auto;">
          Warmplane bridges local tools and remote MCP servers into one unified facade. Add your first server or import existing configs from Claude Desktop or Cursor.
        </p>
        <div style="display: flex; gap: 8px; justify-content: center;">
          <button class="btn btn-primary" onclick="window.app.openTemplateCatalog()">✨ Browse Templates</button>
          <button class="btn btn-ghost" onclick="window.app.openAddServerModal()">+ Add Custom</button>
          <button class="btn btn-ghost" onclick="window.app.openImportModal()">Sync from IDEs</button>
        </div>
      </div>
    `;else n=a.map((r)=>{let s=t[r],o=s.command?"stdio":"http / sse",i=s.command?`${s.command} ${(s.args||[]).join(" ")}`:s.url,l=e.serverStatuses[r]||{status:"connected",protocol_version:"2026-07-28"},p=s.env?Object.entries(s.env).map(([C,I])=>{if(I.startsWith("keychain://"))return`<span class="brand-badge" style="color: var(--cyan-400); border-color: rgba(34, 211, 238, 0.3);">\uD83D\uDD12 ${_(C)} (Keychain)</span>`;if(I.startsWith("op://"))return`<span class="brand-badge" style="color: var(--cyan-400); border-color: rgba(34, 211, 238, 0.3);">\uD83D\uDD12 ${_(C)} (1Password)</span>`;if(I.startsWith("env://"))return`<span class="brand-badge" style="color: var(--amber-300); border-color: rgba(251, 191, 36, 0.3);">\uD83D\uDD12 ${_(C)} (Env)</span>`;return`<span style="color: var(--text-dim);">${_(C)}=***</span>`}).join(" "):"None",u=(e.circuitBreakers||[]).find((C)=>C.server_id===r),g='<span class="brand-badge" style="color: var(--green-400); border-color: rgba(52, 211, 153, 0.25);">Circuit: CLOSED</span>';if(u){if(u.state==="open")g=`<span class="brand-badge" style="color: var(--red-400); border-color: rgba(248, 113, 113, 0.4); background: rgba(248, 113, 113, 0.1);">Circuit: OPEN (${u.consecutive_failures} failures)</span>`;else if(u.state==="half_open")g=`<span class="brand-badge" style="color: var(--amber-300); border-color: rgba(251, 191, 36, 0.4); background: rgba(251, 191, 36, 0.1);">Circuit: HALF-OPEN (${u.consecutive_successes} probe)</span>`}let f=s.resilience||e.config.resilience,c=f?`FT: ${f.failureThreshold||3} · Cooldown: ${(f.cooldownMs||30000)/1000}s · AutoRestart: ${f.autoRestart!==!1?"ON":"OFF"}`:"Default Resilience",y=l.status==="degraded",h=l.status==="error"||l.status==="disconnected",v=y?"var(--amber-400)":h?"var(--red-400)":"var(--green-400)",w=(y||h)&&l.error?`
        <div style="background: rgba(239, 68, 68, 0.08); border-left: 3px solid var(--amber-400); border-radius: var(--radius-xs); padding: 8px 12px; margin-top: 8px; display: flex; justify-content: space-between; align-items: center; gap: 8px;">
          <div style="font-size: 11px; color: var(--amber-300); font-family: var(--ff-mono); overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">
            <span style="font-weight: 700; color: var(--amber-400);">⚠️ Diagnostics:</span> ${_(l.error)}
          </div>
          <button class="btn btn-ghost" style="padding: 2px 8px; font-size: 10.5px; color: var(--amber-300); border-color: rgba(251, 191, 36, 0.3);" onclick="window.app.openServerDiagnosticsModal('${_(r)}')">Details</button>
        </div>
      `:"";return`
        <div class="bento-card" style="margin-bottom: 12px; border-color: ${y?"rgba(251, 191, 36, 0.3)":h?"rgba(248, 113, 113, 0.3)":"var(--border)"};">
          <div style="display: flex; justify-content: space-between; align-items: center;">
            <div>
              <div style="display: flex; align-items: center; gap: 8px; margin-bottom: 4px;">
                <span style="width: 8px; height: 8px; border-radius: 50%; background: ${v}; display: inline-block;"></span>
                <span style="font-size: 15px; font-weight: 700; color: var(--text-main);">${_(r)}</span>
                <span class="brand-badge">${o}</span>
                <span class="brand-badge" style="color: ${v}; border-color: rgba(245, 158, 11, 0.3);">Status: ${_(l.status)}</span>
                <span class="brand-badge" style="color: var(--cyan-400); border-color: rgba(34, 211, 238, 0.25);">Protocol: ${l.protocol_version}</span>
                ${g}
              </div>
              <div style="font-family: var(--ff-mono); font-size: 12px; color: var(--text-muted); margin-top: 4px;">
                ${s.command?"Command: ":"URL: "}<code>${_(i||"")}</code>
              </div>
              <div style="display: flex; gap: 14px; font-family: var(--ff-mono); font-size: 11px; color: var(--text-dim); margin-top: 4px; align-items: center; flex-wrap: wrap;">
                <span>\uD83D\uDEE1️ ${_(c)}</span>
                ${s.env&&Object.keys(s.env).length>0?`<span>Env: ${p}</span>`:""}
              </div>
            </div>
            <div style="display: flex; gap: 8px; align-items: center;">
              <button class="btn btn-ghost" style="padding: 4px 10px; font-size: 11.5px; color: var(--cyan-400); border-color: rgba(34, 211, 238, 0.3);" onclick="window.app.restartServer('${_(r)}')">⚡ Restart</button>
              <button class="btn btn-ghost" style="padding: 4px 10px; font-size: 11.5px;" onclick="window.app.openServerDiagnosticsModal('${_(r)}')">\uD83D\uDD0D Diagnostics</button>
              <button class="btn btn-ghost" style="padding: 4px 10px; font-size: 11.5px;" onclick="window.app.openEditServerModal('${_(r)}')">✏️ Edit</button>
              <button class="btn btn-danger" style="padding: 4px 10px; font-size: 11.5px;" onclick="window.app.deleteServer('${_(r)}')">Remove</button>
            </div>
          </div>
          ${w}
        </div>
      `}).join("");return`
    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 18px;">
      <div>
        <div style="font-size: 16px; font-weight: 700; color: var(--text-main);">Configured MCP Upstream Servers</div>
        <div style="font-size: 11px; color: var(--text-dim);">Active configuration file: <code>${_(e.configPath)}</code></div>
      </div>
      <div style="display: flex; gap: 8px;">
        <button class="btn btn-ghost" onclick="window.app.reloadFromDisk()">⟳ Reload Config</button>
      </div>
    </div>

    ${n}

    ${ue()}
  `}function ue(){let e=d.getState(),t=e.clients||[],a=Object.keys(e.config.profiles||{}),n=e.clientsCollapsed;if(t.length===0)return"";let r=t.filter((o)=>o.is_attached).length,s=t.map((o)=>{let{is_attached:i,config_exists:l,app_installed:p}=o,u='<span class="brand-badge" style="color: var(--text-dim); border-color: rgba(255, 255, 255, 0.1);">Not Found</span>';if(i){let y=o.attached_profile?` · Profile: ${o.attached_profile}`:"";u=`<span class="brand-badge" style="color: var(--green-400); border-color: rgba(52, 211, 153, 0.3); background: rgba(52, 211, 153, 0.1);">⚡ Connected${_(y)}</span>`}else if(l)u='<span class="brand-badge" style="color: var(--amber-300); border-color: rgba(251, 191, 36, 0.3); background: rgba(251, 191, 36, 0.08);">○ Ready to Connect</span>';else if(p)u='<span class="brand-badge" style="color: var(--cyan-400); border-color: rgba(34, 211, 238, 0.25);">○ App Installed</span>';let g=e.activeProfile,f=a.map((y)=>`
      <option value="${_(y)}" ${g===y||o.attached_profile===y?"selected":""}>Profile: ${_(y)}</option>
    `).join(""),c=i?`<button class="btn btn-ghost" style="padding: 4px 10px; font-size: 11px; color: var(--red-400); border-color: rgba(248, 113, 113, 0.3);" onclick="window.app.detachClient('${_(o.id)}')">Disconnect</button>`:`<button class="btn btn-primary" style="padding: 4px 10px; font-size: 11px;" onclick="window.app.attachClient('${_(o.id)}')">⚡ Connect</button>`;return`
      <div style="background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 12px 14px; display: flex; flex-direction: column; justify-content: space-between; gap: 8px;">
        <div style="display: flex; justify-content: space-between; align-items: flex-start;">
          <div>
            <div style="font-weight: 700; font-size: 13.5px; color: var(--text-main); display: flex; align-items: center; gap: 6px;">
              <span>${_(o.name)}</span>
            </div>
            <div style="font-size: 10.5px; color: var(--text-dim); margin-top: 2px;">${_(o.category)}</div>
          </div>
          ${u}
        </div>
        
        <div style="font-family: var(--ff-mono); font-size: 10px; color: var(--text-muted); overflow: hidden; text-overflow: ellipsis; white-space: nowrap;" title="${_(o.config_path)}">
          ${_(o.config_path)}
        </div>

        <div style="display: flex; align-items: center; justify-content: space-between; gap: 6px; margin-top: 4px; padding-top: 6px; border-top: 1px solid var(--border-subtle);">
          ${a.length>0&&!i?`
            <select id="client-prof-${_(o.id)}" class="form-input" style="font-size: 10.5px; padding: 2px 6px; height: 26px; width: 130px;">
              <option value="">All Tools (Default)</option>
              ${f}
            </select>
          `:`<div style="font-size: 10.5px; color: var(--text-dim);">${o.other_servers_count>0?`${o.other_servers_count} other tools`:"Single tool facade"}</div>`}
          ${c}
        </div>
      </div>
    `}).join("");return`
    <div class="bento-card" style="margin-top: 28px; padding: 14px 18px; border-color: rgba(245, 158, 11, 0.2); background: rgba(18, 24, 38, 0.4);">
      <div style="display: flex; justify-content: space-between; align-items: center; cursor: pointer; user-select: none;" onclick="window.app.toggleClientsCollapse()">
        <div>
          <div style="font-size: 14px; font-weight: 700; color: var(--text-main); display: flex; align-items: center; gap: 8px;">
            <span>⚡ 1-Click AI Client Integrations</span>
            <span class="brand-badge" style="color: var(--amber-400); border-color: rgba(245, 158, 11, 0.3);">${r>0?`${r} Connected`:"Auto-Sync"}</span>
          </div>
          <div style="font-size: 11px; color: var(--text-dim); margin-top: 2px;">
            Attach Warmplane's unified facade to desktop IDEs and agents without editing JSON files.
          </div>
        </div>
        <div style="display: flex; align-items: center; gap: 8px;">
          <button class="btn btn-ghost" style="padding: 3px 8px; font-size: 11px;" onclick="event.stopPropagation(); window.app.refreshClients()">⟳ Scan IDEs</button>
          <span style="font-size: 12px; color: var(--text-dim);">${n?"▼ Show":"▲ Hide"}</span>
        </div>
      </div>

      ${!n?`
        <div style="display: grid; grid-template-columns: repeat(auto-fill, minmax(280px, 1fr)); gap: 12px; margin-top: 14px; padding-top: 12px; border-top: 1px solid var(--border-subtle);">
          ${s}
        </div>
      `:""}
    </div>
  `}function _(e){return String(e||"").replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;")}function re(){let e=d.getState(),t=e.playgroundMode||"tools",a=e.capabilities||[],n=e.resources||[],r=e.prompts||[],s=e.capabilitiesHiddenByPolicy||0,o=e.resourcesHiddenByPolicy||0,i=e.promptsHiddenByPolicy||0,l=`
    <div style="display: flex; gap: 8px; margin-bottom: 12px; align-items: center; justify-content: space-between; flex-wrap: wrap;">
      <div style="display: inline-flex; padding: 3px; background: rgba(0,0,0,0.3); border: 1px solid var(--border); border-radius: var(--radius-sm); align-items: center;">
        <button 
          class="btn ${t==="tools"?"btn-primary":"btn-ghost"}" 
          style="padding: 4px 12px; font-size: 11.5px; height: 28px; display: inline-flex; align-items: center; gap: 6px;"
          onclick="window.app.setPlaygroundMode('tools')"
        >
          <span>\uD83D\uDEE0️ Tools (${a.length})</span>
          ${s>0?`<span class="badge" style="background: rgba(245, 158, 11, 0.2); color: var(--amber-300); font-size: 9.5px; padding: 1px 5px;" title="${s} tools hidden by policy/profile">+${s} hidden</span>`:""}
        </button>
        <button 
          class="btn ${t==="resources"?"btn-primary":"btn-ghost"}" 
          style="padding: 4px 12px; font-size: 11.5px; height: 28px; display: inline-flex; align-items: center; gap: 6px;"
          onclick="window.app.setPlaygroundMode('resources')"
        >
          <span>\uD83D\uDCC4 Resources (${n.length})</span>
          ${o>0?`<span class="badge" style="background: rgba(245, 158, 11, 0.2); color: var(--amber-300); font-size: 9.5px; padding: 1px 5px;" title="${o} resources hidden by policy/profile">+${o} hidden</span>`:""}
        </button>
        <button 
          class="btn ${t==="prompts"?"btn-primary":"btn-ghost"}" 
          style="padding: 4px 12px; font-size: 11.5px; height: 28px; display: inline-flex; align-items: center; gap: 6px;"
          onclick="window.app.setPlaygroundMode('prompts')"
        >
          <span>\uD83D\uDCAC Prompts (${r.length})</span>
          ${i>0?`<span class="badge" style="background: rgba(245, 158, 11, 0.2); color: var(--amber-300); font-size: 9.5px; padding: 1px 5px;" title="${i} prompts hidden by policy/profile">+${i} hidden</span>`:""}
        </button>
      </div>

      <div style="display: flex; align-items: center; gap: 12px;">
        ${t==="tools"&&s>0?`
          <div style="font-size: 11px; color: var(--amber-300); background: rgba(245, 158, 11, 0.1); border: 1px solid rgba(245, 158, 11, 0.25); padding: 3px 8px; border-radius: var(--radius-sm); display: flex; align-items: center; gap: 6px;">
            <span>\uD83D\uDEE1️ ${s} tool${s>1?"s":""} filtered by policy/profile</span>
            <a href="javascript:void(0)" onclick="window.app.switchTab('policy')" style="color: var(--amber-400); text-decoration: underline; font-weight: 600;">View Policy</a>
          </div>
        `:t==="resources"&&o>0?`
          <div style="font-size: 11px; color: var(--amber-300); background: rgba(245, 158, 11, 0.1); border: 1px solid rgba(245, 158, 11, 0.25); padding: 3px 8px; border-radius: var(--radius-sm); display: flex; align-items: center; gap: 6px;">
            <span>\uD83D\uDEE1️ ${o} resource${o>1?"s":""} filtered by policy/profile</span>
            <a href="javascript:void(0)" onclick="window.app.switchTab('policy')" style="color: var(--amber-400); text-decoration: underline; font-weight: 600;">View Policy</a>
          </div>
        `:t==="prompts"&&i>0?`
          <div style="font-size: 11px; color: var(--amber-300); background: rgba(245, 158, 11, 0.1); border: 1px solid rgba(245, 158, 11, 0.25); padding: 3px 8px; border-radius: var(--radius-sm); display: flex; align-items: center; gap: 6px;">
            <span>\uD83D\uDEE1️ ${i} prompt${i>1?"s":""} filtered by policy/profile</span>
            <a href="javascript:void(0)" onclick="window.app.switchTab('policy')" style="color: var(--amber-400); text-decoration: underline; font-weight: 600;">View Policy</a>
          </div>
        `:`
          <div style="font-size: 11.5px; color: var(--text-dim);">
            ${t==="tools"?"Interactive Tool Caller & Context Distillation":t==="resources"?"Live MCP Resource Inspector & Reader":"Prompt Template Studio & Variable Binder"}
          </div>
        `}
      </div>
    </div>
  `;if(t==="resources")return`
      ${l}
      ${ve(e)}
    `;if(t==="prompts")return`
      ${l}
      ${me(e)}
    `;return`
    ${l}
    ${ge(e)}
    ${e.isBatchModalOpen?fe(e):""}
  `}function W(e,t=!1){if(!e||!e.properties)return{};let a=e.properties||{},n=Array.isArray(e.required)?e.required:[],r={};for(let[s,o]of Object.entries(a)){let i=n.includes(s);if(t&&!i)continue;if(o.default!==void 0)r[s]=o.default;else if(Array.isArray(o.enum)&&o.enum.length>0)r[s]=o.enum[0];else if(o.examples&&Array.isArray(o.examples)&&o.examples.length>0)r[s]=o.examples[0];else if(o.example!==void 0)r[s]=o.example;else switch(o.type||"string"){case"string":r[s]=i?`sample_${s}`:"";break;case"number":case"integer":r[s]=0;break;case"boolean":r[s]=!0;break;case"array":r[s]=[];break;case"object":r[s]={};break;default:r[s]=`sample_${s}`}}return r}function ge(e){let t=e.capabilities||[],a=e.capabilitiesHiddenByPolicy||0,n=e.selectedCapabilityId||(t.length>0?t[0].id:null),r=t.find((c)=>c.id===n),s=!!e.isExecuting,o="";if(t.length===0)o=`
      <div style="padding: 24px 16px; text-align: center; color: var(--text-dim); font-size: 11.5px;">
        No tools or capabilities discovered from connected servers.
      </div>
    `;else o=t.map((c)=>`
        <div class="cap-item ${c.id===n?"active":""}" onclick="window.app.selectCapability('${x(c.id)}')">
          <div style="display: flex; justify-content: space-between; align-items: center;">
            <span style="font-weight: 600; color: var(--text-main); font-family: var(--ff-mono); font-size: 12px;">${x(c.id)}</span>
            <span style="font-size: 10px; color: var(--green-400);">${x(c.mode||"read")}</span>
          </div>
          <div style="font-size: 11px; color: var(--text-dim); margin-top: 2px;">server: ${x(c.server||"local")}</div>
        </div>
      `).join("");let i=r?.input_schema,l=i?.properties||{},p=Array.isArray(i?.required)?i.required:[],u=Object.entries(l),g="";if(u.length>0)g=`
      <div style="display: flex; flex-wrap: wrap; gap: 6px; margin-bottom: 8px; align-items: center;">
        <span style="font-size: 10px; font-weight: 700; color: var(--text-dim); text-transform: uppercase;">Fields:</span>
        ${u.map(([c,y])=>{let h=p.includes(c),v=y.type||(y.enum?"enum":"any"),w=h?"rgba(239, 68, 68, 0.15)":"rgba(148, 163, 184, 0.1)",C=h?"var(--red-400)":"var(--text-muted)",I=h?"rgba(239, 68, 68, 0.3)":"var(--border)",A=y.description?` - ${y.description}`:"";return`
            <button 
              type="button" 
              class="btn" 
              style="padding: 2px 7px; font-size: 10.5px; font-family: var(--ff-mono); background: ${w}; color: ${C}; border: 1px solid ${I}; border-radius: var(--radius-sm);" 
              title="Click to insert '${c}' (${v}${A})" 
              onclick="window.app.insertPlaygroundArgKey('${x(c)}', '${x(v)}', ${x(JSON.stringify(y.default??null))})"
            >
              + ${x(c)} <span style="font-size: 9px; opacity: 0.7;">(${v}${h?" *":""})</span>
            </button>
          `}).join("")}
      </div>
    `;let f="{}";if(n&&e.playgroundArgs&&e.playgroundArgs[n]!==void 0)f=e.playgroundArgs[n];else{let c=W(i,!1);f=JSON.stringify(c,null,2)}return`
    <div style="display: grid; grid-template-columns: 320px 1fr; gap: 16px; height: calc(100vh - 165px);">
      <!-- Left Sidebar: Capabilities Catalog -->
      <div style="background: var(--surface-card); border: 1px solid var(--border); border-radius: var(--radius-md); display: flex; flex-direction: column; overflow: hidden;">
        <div style="padding: 12px; border-bottom: 1px solid var(--border);">
          <input type="text" class="form-input" placeholder="Search ${t.length} capabilities..." oninput="window.app.filterCapabilities(this.value)">
        </div>
        <div style="flex: 1; overflow-y: auto; padding: 8px;" id="pg-cap-list">
          ${o}
        </div>
        ${a>0?`
          <div style="padding: 8px 12px; background: rgba(245, 158, 11, 0.08); border-top: 1px solid rgba(245, 158, 11, 0.2); font-size: 11px; color: var(--amber-300); display: flex; justify-content: space-between; align-items: center;">
            <span>\uD83D\uDEE1️ ${a} tool${a>1?"s":""} hidden by policy</span>
            <a href="javascript:void(0)" onclick="window.app.switchTab('policy')" style="color: var(--amber-400); text-decoration: underline; font-weight: 600; font-size: 10.5px;">Edit Policy</a>
          </div>
        `:""}
      </div>

      <!-- Right Panel: Capability Execution & Envelope Visualizer -->
      <div style="background: var(--surface-card); border: 1px solid var(--border); border-radius: var(--radius-md); display: flex; flex-direction: column; overflow: hidden;">
        <div style="padding: 14px 18px; border-bottom: 1px solid var(--border); display: flex; justify-content: space-between; align-items: center;">
          <div>
            <div style="font-size: 15px; font-weight: 700; color: var(--text-main); font-family: var(--ff-mono);" id="pg-selected-title">
              ${x(r?r.id:"No Capability Selected")}
            </div>
            <div style="font-size: 11.5px; color: var(--text-dim);" id="pg-selected-desc">
              ${x(r?r.summary||r.description:"Connect servers to inspect and execute tools")}
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
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
              <label class="form-label" style="margin: 0;">Arguments JSON</label>
              <div style="display: flex; gap: 6px;">
                <button type="button" class="btn btn-ghost" style="padding: 2px 7px; font-size: 10.5px;" title="Fill sample payload from schema" onclick="window.app.fillPlaygroundSampleArgs(false)">✨ Sample Template</button>
                ${p.length>0?`
                  <button type="button" class="btn btn-ghost" style="padding: 2px 7px; font-size: 10.5px;" title="Fill only required schema fields" onclick="window.app.fillPlaygroundSampleArgs(true)">\uD83E\uDDF9 Required Only</button>
                `:""}
                <button type="button" class="btn btn-ghost" style="padding: 2px 7px; font-size: 10.5px;" title="Format JSON" onclick="window.app.formatPlaygroundArgs()">\uD83D\uDCCB Format</button>
                <button type="button" class="btn btn-ghost" style="padding: 2px 8px; font-size: 11px;" onclick="window.app.openBatchModal()">⚡ Pipeline Builder</button>
              </div>
            </div>

            ${g}

            <textarea class="form-textarea" rows="7" id="pg-args-input" oninput="window.app.updatePlaygroundArgs(this.value)">${x(f)}</textarea>

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

            <div style="margin-top: 10px; display: flex; align-items: center; justify-content: space-between; padding: 8px 12px; background: rgba(0,0,0,0.25); border-radius: var(--radius-sm); border: 1px solid var(--border);">
              <div>
                <div style="font-size: 11.5px; font-weight: 600; color: var(--amber-300); display: flex; align-items: center; gap: 6px;">
                  <span>⚡ Async Task Mode (SEP-2663)</span>
                </div>
                <div style="font-size: 10.5px; color: var(--text-dim);">Execute tool asynchronously returning HTTP 202 Accepted Task</div>
              </div>
              <label style="position: relative; display: inline-block; width: 36px; height: 20px; margin: 0; cursor: pointer;">
                <input type="checkbox" id="pg-async-task-toggle" ${e.playgroundAsyncTask?"checked":""} onchange="window.app.togglePlaygroundAsyncTask(this.checked)" style="opacity: 0; width: 0; height: 0;">
                <span style="position: absolute; cursor: pointer; top: 0; left: 0; right: 0; bottom: 0; background-color: ${e.playgroundAsyncTask?"var(--amber-400)":"var(--border)"}; transition: .3s; border-radius: 20px;">
                  <span style="position: absolute; content: ''; height: 14px; width: 14px; left: ${e.playgroundAsyncTask?"19px":"3px"}; bottom: 3px; background-color: white; transition: .3s; border-radius: 50%;"></span>
                </span>
              </label>
            </div>

            <div class="form-group" style="margin-top: 10px;">
              <label class="form-label">Request Context / Operation ID (Optional)</label>
              <input type="text" class="form-input" id="pg-context-input" placeholder="e.g. op-dev-test-1">
            </div>
            ${r&&r.input_schema?`
              <div style="margin-top: 14px;">
                <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 4px;">
                  <label class="form-label" style="margin: 0;">Input JSON Schema</label>
                  <span style="font-size: 10px; color: var(--text-dim); font-family: var(--ff-mono);">${u.length} field${u.length===1?"":"s"} (${p.length} required)</span>
                </div>
                <pre style="background: var(--surface); padding: 10px; border-radius: var(--radius-sm); border: 1px solid var(--border); font-size: 11px; color: var(--text-muted); max-height: 140px; overflow-y: auto;">${x(JSON.stringify(r.input_schema,null,2))}</pre>
              </div>
            `:""}
          </div>

          <!-- Response Inspector -->
          <div style="padding: 16px; background: var(--bg-app); display: flex; flex-direction: column; overflow: hidden;">
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
              <span style="font-size: 11px; font-weight: 600; color: var(--text-dim);">
                ${e.executionResult&&(e.executionResult.status===202||e.executionResult.data?.resultType==="task")?"SEP-2663 TASK RESPONSE":"NORMALIZED EXECUTION ENVELOPE"}
              </span>
              <span id="pg-status-badge" style="font-size: 11px; font-weight: 600; color: ${e.executionResult?e.executionResult.status===200?"var(--green-400)":e.executionResult.status===202?"var(--amber-300)":"var(--red-400)":"var(--text-dim)"}; font-family: var(--ff-mono);">
                ${e.executionResult?`HTTP ${e.executionResult.status} · ${e.executionResult.durationMs.toFixed(1)}ms`:"READY"}
              </span>
            </div>

            ${e.executionResult&&(e.executionResult.status===202||e.executionResult.data?.resultType==="task")?`
              <div style="margin-bottom: 12px; padding: 12px 14px; background: rgba(245, 158, 11, 0.1); border: 1px solid rgba(245, 158, 11, 0.35); border-radius: var(--radius-sm); display: flex; justify-content: space-between; align-items: center;">
                <div>
                  <div style="display: flex; align-items: center; gap: 8px;">
                    <span class="brand-badge" style="background: rgba(245, 158, 11, 0.2); color: var(--amber-300); border-color: rgba(245, 158, 11, 0.5);">
                      ${x(e.executionResult.data?.task?.status||e.executionResult.data?.status||"TASK_CREATED").toUpperCase()}
                    </span>
                    <span style="font-family: var(--ff-mono); font-size: 12px; font-weight: 700; color: var(--text-main);">${x(e.executionResult.data?.task?.taskId||e.executionResult.data?.taskId||"")}</span>
                  </div>
                  <div style="font-size: 11px; color: var(--text-dim); margin-top: 4px;">
                    Execution suspended for Human-in-the-Loop approval or async resolution.
                  </div>
                </div>
                <button class="btn btn-primary" style="padding: 4px 10px; font-size: 11px;" onclick="window.app.switchTab('tasks')">
                  Go to Tasks &amp; Approvals →
                </button>
              </div>
            `:""}

            <pre id="pg-response-json" style="flex: 1; background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 14px; color: var(--amber-300); font-size: 11.5px; overflow-y: auto; margin: 0; white-space: pre-wrap; word-break: break-word;">${e.executionResult?x(JSON.stringify(e.executionResult.data,null,2)):"// Response envelope output will be formatted here"}</pre>
          </div>
        </div>
      </div>
    </div>
  `}function ve(e){let t=e.resources||[],a=e.resourcesHiddenByPolicy||0,n=e.selectedResourceId||(t.length>0?t[0].id:null),r=t.find((i)=>i.id===n),s=e.resourceReadResult,o="";if(t.length===0)o=`
      <div style="padding: 24px 16px; text-align: center; color: var(--text-dim); font-size: 11.5px;">
        No resources exposed by connected MCP servers.
      </div>
    `;else o=t.map((i)=>{let l=i.id===n?"active":"",p=i.uri?i.uri.split(":")[0]:"res";return`
        <div class="cap-item ${l}" onclick="window.app.selectResource('${x(i.id)}')">
          <div style="display: flex; justify-content: space-between; align-items: center;">
            <span style="font-weight: 600; color: var(--text-main); font-family: var(--ff-mono); font-size: 12px;">${x(i.name||i.id)}</span>
            <span class="badge" style="font-size: 9.5px; background: rgba(56, 189, 248, 0.15); color: var(--cyan-400);">${x(p)}</span>
          </div>
          <div style="font-size: 11px; color: var(--text-dim); margin-top: 2px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">${x(i.uri)}</div>
          <div style="display: flex; justify-content: space-between; font-size: 10px; color: var(--text-muted); margin-top: 4px;">
            <span>server: ${x(i.server||"local")}</span>
            <span>${x(i.mime_type||"text/plain")}</span>
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
        ${a>0?`
          <div style="padding: 8px 12px; background: rgba(245, 158, 11, 0.08); border-top: 1px solid rgba(245, 158, 11, 0.2); font-size: 11px; color: var(--amber-300); display: flex; justify-content: space-between; align-items: center;">
            <span>\uD83D\uDEE1️ ${a} resource${a>1?"s":""} hidden by policy</span>
            <a href="javascript:void(0)" onclick="window.app.switchTab('policy')" style="color: var(--amber-400); text-decoration: underline; font-weight: 600; font-size: 10.5px;">Edit Policy</a>
          </div>
        `:""}
      </div>

      <!-- Right Panel: Resource Content Reader & Metadata Inspector -->
      <div style="background: var(--surface-card); border: 1px solid var(--border); border-radius: var(--radius-md); display: flex; flex-direction: column; overflow: hidden;">
        <div style="padding: 14px 18px; border-bottom: 1px solid var(--border); display: flex; justify-content: space-between; align-items: center;">
          <div>
            <div style="font-size: 15px; font-weight: 700; color: var(--text-main); font-family: var(--ff-mono);">
              ${x(r?r.name||r.id:"No Resource Selected")}
            </div>
            <div style="font-size: 11.5px; color: var(--cyan-400); font-family: var(--ff-mono);">
              ${x(r?r.uri:"Select a resource from the list to read live content")}
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
                  <div><span style="color: var(--text-muted);">Server:</span> <strong style="color: var(--text-main);">${x(r.server)}</strong></div>
                  <div><span style="color: var(--text-muted);">MIME Type:</span> <strong style="color: var(--text-main);">${x(r.mime_type||"text/plain")}</strong></div>
                </div>
                ${r.description?`
                  <div style="margin-top: 8px; font-size: 11.5px; color: var(--text-dim); border-top: 1px solid rgba(255,255,255,0.05); padding-top: 6px;">
                    ${x(r.description)}
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
            <pre style="flex: 1; background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 14px; color: var(--cyan-400); font-size: 11.5px; overflow-y: auto; margin: 0; white-space: pre-wrap; word-break: break-word;">${s?x(JSON.stringify(s.data,null,2)):'// Click "Read Resource Content" to inspect live payload'}</pre>
          </div>
        </div>
      </div>
    </div>
  `}function me(e){let t=e.prompts||[],a=e.promptsHiddenByPolicy||0,n=e.selectedPromptId||(t.length>0?t[0].id:null),r=t.find((l)=>l.id===n),s=e.promptGetResult,o="";if(t.length===0)o=`
      <div style="padding: 24px 16px; text-align: center; color: var(--text-dim); font-size: 11.5px;">
        No prompt templates registered by connected MCP servers.
      </div>
    `;else o=t.map((l)=>{let p=l.id===n?"active":"",u=l.arguments?l.arguments.length:0;return`
        <div class="cap-item ${p}" onclick="window.app.selectPrompt('${x(l.id)}')">
          <div style="display: flex; justify-content: space-between; align-items: center;">
            <span style="font-weight: 600; color: var(--text-main); font-family: var(--ff-mono); font-size: 12px;">${x(l.name||l.id)}</span>
            <span class="badge" style="font-size: 9.5px; background: rgba(168, 85, 247, 0.15); color: var(--purple-400);">${u} args</span>
          </div>
          <div style="font-size: 11px; color: var(--text-dim); margin-top: 2px;">${x(l.description||l.title||"Prompt template")}</div>
          <div style="font-size: 10px; color: var(--text-muted); margin-top: 4px;">server: ${x(l.server||"local")}</div>
        </div>
      `}).join("");let i="";if(r&&r.arguments&&r.arguments.length>0)i=r.arguments.map((l)=>`
      <div class="form-group" style="margin-bottom: 12px;">
        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 4px;">
          <label class="form-label" style="margin: 0; font-family: var(--ff-mono);">${x(l.name)}</label>
          ${l.required?'<span class="badge" style="background: rgba(239, 68, 68, 0.15); color: var(--red-400); font-size: 9px;">REQUIRED</span>':'<span style="font-size: 10px; color: var(--text-dim);">optional</span>'}
        </div>
        ${l.description?`<div style="font-size: 11px; color: var(--text-dim); margin-bottom: 4px;">${x(l.description)}</div>`:""}
        <input type="text" class="form-input prompt-arg-input" data-arg-name="${x(l.name)}" placeholder="Enter ${x(l.name)}..." />
      </div>
    `).join("");else if(r)i=`
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
        ${a>0?`
          <div style="padding: 8px 12px; background: rgba(245, 158, 11, 0.08); border-top: 1px solid rgba(245, 158, 11, 0.2); font-size: 11px; color: var(--amber-300); display: flex; justify-content: space-between; align-items: center;">
            <span>\uD83D\uDEE1️ ${a} prompt${a>1?"s":""} hidden by policy</span>
            <a href="javascript:void(0)" onclick="window.app.switchTab('policy')" style="color: var(--amber-400); text-decoration: underline; font-weight: 600; font-size: 10.5px;">Edit Policy</a>
          </div>
        `:""}
      </div>

      <!-- Right Panel: Prompt Parameter Binder & Message Envelope Preview -->
      <div style="background: var(--surface-card); border: 1px solid var(--border); border-radius: var(--radius-md); display: flex; flex-direction: column; overflow: hidden;">
        <div style="padding: 14px 18px; border-bottom: 1px solid var(--border); display: flex; justify-content: space-between; align-items: center;">
          <div>
            <div style="font-size: 15px; font-weight: 700; color: var(--text-main); font-family: var(--ff-mono);">
              ${x(r?r.name||r.id:"No Prompt Selected")}
            </div>
            <div style="font-size: 11.5px; color: var(--text-dim);">
              ${x(r?r.description||r.title||"Bind variables and render messages":"Select a prompt from the list to test")}
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
            ${i}
          </div>

          <!-- Rendered Messages Output -->
          <div style="padding: 16px; background: var(--bg-app); display: flex; flex-direction: column; overflow: hidden;">
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
              <span style="font-size: 11px; font-weight: 600; color: var(--text-dim);">RENDERED PROMPT MESSAGES</span>
              <span style="font-size: 11px; font-weight: 600; color: ${s?s.status===200?"var(--green-400)":"var(--red-400)":"var(--text-dim)"}; font-family: var(--ff-mono);">
                ${s?`HTTP ${s.status} · ${s.durationMs.toFixed(1)}ms`:"READY"}
              </span>
            </div>
            <pre style="flex: 1; background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 14px; color: #c084fc; font-size: 11.5px; overflow-y: auto; margin: 0; white-space: pre-wrap; word-break: break-word;">${s?x(JSON.stringify(s.data,null,2)):'// Click "Render Prompt Messages" to view resolved system/user messages'}</pre>
          </div>
        </div>
      </div>
    </div>
  `}function fe(e){let t=e.capabilities||[],a=e.batchSteps||[];return`
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
          ${a.map((r,s)=>{let o=t.find((c)=>c.id===r.capability_id),i=o?.input_schema,l=i?.properties||{},p=Array.isArray(i?.required)?i.required:[],u=Object.entries(l),g=t.map((c)=>`
      <option value="${x(c.id)}" ${c.id===r.capability_id?"selected":""}>
        ${x(c.id)} (${x(c.server||"local")})
      </option>
    `).join(""),f="";if(u.length>0)f=`
        <div style="display: flex; flex-wrap: wrap; gap: 4px; margin-top: 6px; margin-bottom: 6px; align-items: center;">
          <span style="font-size: 9.5px; font-weight: 700; color: var(--text-dim); text-transform: uppercase;">Parameters:</span>
          ${u.map(([c,y])=>{let h=p.includes(c),v=y.type||(y.enum?"enum":"any");return`
              <span style="font-size: 9.5px; font-family: var(--ff-mono); padding: 1px 5px; background: ${h?"rgba(239, 68, 68, 0.15)":"rgba(148, 163, 184, 0.1)"}; color: ${h?"var(--red-400)":"var(--text-muted)"}; border: 1px solid ${h?"rgba(239, 68, 68, 0.3)":"var(--border)"}; border-radius: 3px;" title="${x(y.description||"")}">
                ${x(c)} (${v}${h?" *":""})
              </span>
            `}).join("")}
        </div>
      `;return`
      <div style="background: rgba(0,0,0,0.3); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 14px; margin-bottom: 12px;">
        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 10px;">
          <div style="display: flex; align-items: center; gap: 8px;">
            <span class="badge" style="background: rgba(56, 189, 248, 0.15); color: var(--cyan-400); font-family: var(--ff-mono); font-weight: 700;">STEP ${s+1}</span>
            <span style="font-size: 11px; font-family: var(--ff-mono); color: var(--text-dim);">id: ${x(r.id)}</span>
          </div>
          <button class="btn btn-ghost" style="padding: 2px 8px; font-size: 11px; color: var(--red-400);" onclick="window.app.removeBatchStep(${s})">
            ✕ Remove
          </button>
        </div>

        <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 12px; margin-bottom: 6px;">
          <div class="form-group" style="margin: 0;">
            <label class="form-label" style="font-size: 11px;">Target Capability</label>
            <select class="form-input" style="font-size: 11.5px;" onchange="window.app.updateBatchStepCapability(${s}, this.value)">
              <option value="">-- Select Capability --</option>
              ${g}
            </select>
          </div>
          <div style="display: flex; align-items: flex-end; padding-bottom: 6px;">
            <label style="display: flex; align-items: center; gap: 6px; font-size: 11.5px; color: var(--text-muted); cursor: pointer;">
              <input type="checkbox" ${r.continue_on_error?"checked":""} onchange="window.app.updateBatchStepContinueOnError(${s}, this.checked)" />
              <span>Continue pipeline on step failure</span>
            </label>
          </div>
        </div>

        ${f}

        <div class="form-group" style="margin: 0;">
          <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 4px;">
            <div style="display: flex; align-items: center; gap: 8px;">
              <label class="form-label" style="margin: 0; font-size: 11px;">Step Arguments JSON</label>
              ${o?`
                <button type="button" class="btn btn-ghost" style="padding: 1px 6px; font-size: 9.5px;" onclick="window.app.fillBatchStepSampleArgs(${s})">✨ Sample Args</button>
              `:""}
            </div>
            <div style="display: flex; gap: 6px; font-size: 10px; color: var(--cyan-400); font-family: var(--ff-mono);">
              <span>Helpers:</span>
              <code style="cursor: pointer; background: rgba(0,0,0,0.3); padding: 1px 4px; border-radius: 2px;" onclick="window.app.appendBatchVariable(${s}, '\${steps[0].result.id}')">\${steps[0].result.id}</code>
              <code style="cursor: pointer; background: rgba(0,0,0,0.3); padding: 1px 4px; border-radius: 2px;" onclick="window.app.appendBatchVariable(${s}, '\${steps[0].result.data}')">\${steps[0].result.data}</code>
            </div>
          </div>
          <textarea 
            id="batch-step-args-${s}"
            class="form-textarea" 
            rows="3" 
            style="font-size: 11px; font-family: var(--ff-mono);" 
            oninput="window.app.updateBatchStepArgs(${s}, this.value)"
          >${x(r.argsJson)}</textarea>
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
  `}function x(e){return String(e||"").replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;")}function ae(e){let t=e.tasks||[],a=e.taskFilterStatus||"all",n=t.filter((c)=>c.status==="input_required"),r=t.filter((c)=>c.status==="working"),s=t.filter((c)=>c.status==="completed"),o=t.filter((c)=>c.status==="cancelled"),i=t.filter((c)=>c.status==="failed"),l=a==="all"?t:t.filter((c)=>c.status===a),p=e.config.policy?.require_approval||e.config.policy?.requireApproval||[],u=n.length===0?`
    <div style="padding: 36px 24px; text-align: center; background: var(--surface-card); border-radius: var(--radius-md); border: 1px dashed var(--border);">
      <div style="width: 44px; height: 44px; border-radius: 50%; background: rgba(52, 211, 153, 0.12); border: 1px solid rgba(52, 211, 153, 0.3); display: flex; align-items: center; justify-content: center; margin: 0 auto 12px; color: var(--green-400); font-size: 18px; font-weight: 700;">
        ✓
      </div>
      <div style="font-size: 14.5px; font-weight: 600; color: var(--text-main); margin-bottom: 5px;">No Tasks Awaiting Input or Approval</div>
      <div style="font-size: 11.5px; color: var(--text-dim); max-width: 520px; margin: 0 auto; line-height: 1.6;">
        Tool calls requiring Human-in-the-Loop approval or returning asynchronous <code style="color: var(--amber-300); font-family: var(--ff-mono);">input_required</code> tasks will suspend here for operator inspection, parameter editing, and response submission.
      </div>
    </div>
  `:n.map((c)=>{let y=c.inputRequests||{},h=Object.keys(y),v=h.length>0,w=Math.floor(Date.now()/1000),C=c.expiresAtEpochSecs?Math.max(0,c.expiresAtEpochSecs-w):c.ttlSeconds||300;return`
      <div class="bento-card" style="border: 1px solid rgba(245, 158, 11, 0.35); background: var(--surface-card); margin-bottom: 14px; padding: 18px;">
        <div style="display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 12px;">
          <div>
            <div style="display: flex; align-items: center; gap: 8px;">
              <span class="brand-badge" style="background: rgba(245, 158, 11, 0.2); color: var(--amber-300); border-color: rgba(245, 158, 11, 0.5);">
                INPUT REQUIRED
              </span>
              <span style="font-family: var(--ff-mono); font-size: 12px; color: var(--text-muted);">${S(c.taskId)}</span>
            </div>
            <div style="margin-top: 6px; display: flex; align-items: center; gap: 8px;">
              <span style="font-size: 14.5px; font-weight: 700; color: var(--text-main); font-family: var(--ff-mono);">
                ${S(c.capabilityId||"Tool Execution")}
              </span>
              ${c.serverId?`<span style="font-size: 11px; color: var(--text-dim);">via <span style="color: var(--cyan-400); font-family: var(--ff-mono);">${S(c.serverId)}</span></span>`:""}
            </div>
          </div>

          <div style="text-align: right; font-family: var(--ff-mono); font-size: 11px; color: var(--text-dim);">
            ${c.createdAtEpochSecs?`<div>Created: <span style="color: var(--text-muted);">${new Date(c.createdAtEpochSecs*1000).toLocaleTimeString()}</span></div>`:""}
            <div style="color: var(--amber-400); margin-top: 2px;">TTL Remaining: ${C}s</div>
          </div>
        </div>

        <!-- Caller Context -->
        ${c.context?`
          <div style="background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 8px 12px; font-family: var(--ff-mono); font-size: 11px; display: flex; flex-wrap: wrap; gap: 16px; margin-bottom: 12px; color: var(--text-muted);">
            ${c.context.actor_id?`<div><span style="color: var(--text-dim);">Actor:</span> <span style="color: var(--cyan-400);">${S(c.context.actor_id)}</span></div>`:""}
            ${c.context.operation_id?`<div><span style="color: var(--text-dim);">Operation:</span> <span style="color: var(--text-main);">${S(c.context.operation_id)}</span></div>`:""}
            ${c.context.grant_id?`<div><span style="color: var(--text-dim);">Grant:</span> <span style="color: var(--text-main);">${S(c.context.grant_id)}</span></div>`:""}
          </div>
        `:""}

        <!-- Dynamic Input Requests Form -->
        <div style="margin-bottom: 14px;">
          <div style="font-size: 11px; font-weight: 600; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.5px; margin-bottom: 8px;">
            ${v?"Required Input Responses (MRTR / HITL)":"Input Responses Payload (JSON)"}
          </div>

          ${v?`
            <div style="display: flex; flex-direction: column; gap: 10px;">
              ${h.map((I)=>{let A=y[I]||{},L=typeof A==="string"?A:A.prompt||A.description||A.title||I,k=A.type||"text",M=A.default!==void 0?JSON.stringify(A.default):A.value!==void 0?JSON.stringify(A.value):A.sanitized_args?JSON.stringify(A.sanitized_args,null,2):"";if(k==="approval_review")return`
                    <div style="background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 10px 12px;">
                      <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 4px;">
                        <label style="font-size: 11.5px; font-weight: 600; color: var(--amber-300); font-family: var(--ff-mono);">${S(I)}</label>
                        <span class="brand-badge" style="font-size: 9.5px; padding: 1px 5px;">APPROVAL GATED</span>
                      </div>
                      <div style="font-size: 11px; color: var(--text-dim); margin-bottom: 6px;">${S(L)}</div>
                      <div style="margin-bottom: 8px;">
                        <label style="font-size: 10.5px; color: var(--text-dim); display: block; margin-bottom: 2px;">Decision:</label>
                        <select id="task-input-${S(c.taskId)}-${S(I)}-decision" class="form-input" style="font-size: 11.5px; font-family: var(--ff-mono); padding: 4px 8px;">
                          <option value="true" selected>Approve &amp; Execute</option>
                          <option value="false">Reject Execution</option>
                        </select>
                      </div>
                      <div>
                        <label style="font-size: 10.5px; color: var(--text-dim); display: block; margin-bottom: 2px;">Parameters (Editable):</label>
                        <textarea id="task-input-${S(c.taskId)}-${S(I)}" class="form-textarea" rows="3" style="color: var(--green-400); font-family: var(--ff-mono); font-size: 11.5px;">${S(M)}</textarea>
                      </div>
                    </div>
                  `;return`
                  <div style="background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 10px 12px;">
                    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 4px;">
                      <label style="font-size: 11.5px; font-weight: 600; color: var(--amber-300); font-family: var(--ff-mono);">${S(I)}</label>
                      <span class="brand-badge" style="font-size: 9.5px; padding: 1px 5px;">${S(k)}</span>
                    </div>
                    <div style="font-size: 11px; color: var(--text-dim); margin-bottom: 6px;">${S(L)}</div>
                    ${k==="confirmation"||k==="boolean"?`
                      <select id="task-input-${S(c.taskId)}-${S(I)}" class="form-input" style="font-size: 11.5px; font-family: var(--ff-mono); padding: 4px 8px;">
                        <option value="true" selected>true (Approve / Confirm)</option>
                        <option value="false">false (Reject / Deny)</option>
                      </select>
                    `:A.sanitized_args||k==="object"||k==="json"?`
                      <textarea id="task-input-${S(c.taskId)}-${S(I)}" class="form-textarea" rows="3" style="color: var(--green-400); font-family: var(--ff-mono); font-size: 11.5px;">${S(M)}</textarea>
                    `:`
                      <input id="task-input-${S(c.taskId)}-${S(I)}" type="text" class="form-input" value="${S(M)}" placeholder="Enter ${S(I)} response..." style="font-size: 11.5px; font-family: var(--ff-mono);">
                    `}
                  </div>
                `}).join("")}
            </div>
          `:`
            <textarea id="task-raw-input-${S(c.taskId)}" class="form-textarea" rows="3" style="color: var(--green-400); font-family: var(--ff-mono); font-size: 11.5px;">{}</textarea>
          `}
        </div>

        <!-- Action Footer -->
        <div style="display: flex; justify-content: space-between; align-items: center; border-top: 1px solid var(--border); padding-top: 12px;">
          <div style="display: flex; align-items: center; gap: 8px;">
            <input id="task-operator-${S(c.taskId)}" type="text" class="form-input" placeholder="Operator ID" value="security-operator" style="width: 180px; padding: 5px 10px; font-size: 11px;">
          </div>

          <div style="display: flex; align-items: center; gap: 8px;">
            <button class="btn btn-danger" onclick="window.app.promptCancelTask('${S(c.taskId)}')">
              ✕ Cancel Task
            </button>
            <button class="btn btn-primary" onclick="window.app.submitTaskInputResponses('${S(c.taskId)}')">
              ✓ Submit &amp; Resume
            </button>
          </div>
        </div>
      </div>
    `}).join(""),g=p.length===0?`
    <div style="color: var(--text-dim); font-size: 11.5px; line-height: 1.5; padding: 8px 0;">
      No explicit <code style="color: var(--amber-400);">require_approval</code> rules active. Gated execution rules convert matching tool calls into tasks in real-time.
    </div>
  `:p.map((c)=>`
    <div style="display: flex; justify-content: space-between; align-items: center; background: var(--surface); padding: 8px 10px; border-radius: var(--radius-sm); border: 1px solid var(--border); margin-bottom: 6px;">
      <span style="font-family: var(--ff-mono); font-size: 11.5px; color: var(--amber-300); font-weight: 500;">\uD83D\uDEE1️ ${S(c)}</span>
      <span class="brand-badge" style="font-size: 9.5px; padding: 1px 5px;">GATED</span>
    </div>
  `).join(""),f=l.length===0?`
    <tr>
      <td colspan="6" style="padding: 24px; text-align: center; color: var(--text-dim); font-size: 12px;">
        No tasks found matching filter "${S(a)}".
      </td>
    </tr>
  `:l.map((c)=>{let y=c.status==="completed"?"background: rgba(52, 211, 153, 0.12); color: var(--green-400); border-color: rgba(52, 211, 153, 0.3);":c.status==="working"?"background: rgba(56, 189, 248, 0.15); color: var(--cyan-400); border-color: rgba(56, 189, 248, 0.4);":c.status==="input_required"?"background: rgba(245, 158, 11, 0.2); color: var(--amber-300); border-color: rgba(245, 158, 11, 0.5);":c.status==="cancelled"?"background: rgba(148, 163, 184, 0.15); color: var(--text-muted); border-color: rgba(148, 163, 184, 0.3);":"background: rgba(248, 113, 113, 0.12); color: var(--red-400); border-color: rgba(248, 113, 113, 0.3);",h=c.progress!==void 0?Math.round(c.progress*100):c.status==="completed"?100:c.status==="working"?50:0,v=c.createdAtEpochSecs?new Date(c.createdAtEpochSecs*1000).toLocaleTimeString():"—";return`
      <tr style="border-bottom: 1px solid rgba(255,255,255,0.03); transition: background 0.15s;" onmouseover="this.style.background='var(--surface-hover)'" onmouseout="this.style.background='transparent'">
        <td style="padding: 10px 14px;">
          <span class="brand-badge" style="${y}">
            ${c.status.toUpperCase()}
          </span>
        </td>
        <td style="padding: 10px 14px; font-family: var(--ff-mono); font-weight: 600; color: var(--text-main); font-size: 11.5px;">
          ${S(c.capabilityId||"Tool Execution")}
        </td>
        <td style="padding: 10px 14px; font-family: var(--ff-mono); color: var(--text-dim); font-size: 11px;">
          ${S(c.taskId)}
        </td>
        <td style="padding: 10px 14px; width: 140px;">
          <div style="display: flex; align-items: center; gap: 8px;">
            <div style="flex: 1; height: 6px; background: var(--surface-card); border-radius: 3px; overflow: hidden; border: 1px solid var(--border);">
              <div style="height: 100%; width: ${h}%; background: ${c.status==="completed"?"var(--green-400)":"var(--amber-400)"}; transition: width 0.3s;"></div>
            </div>
            <span style="font-size: 10.5px; font-family: var(--ff-mono); color: var(--text-muted);">${h}%</span>
          </div>
        </td>
        <td style="padding: 10px 14px; color: var(--text-dim); font-size: 11px; text-align: right;">
          ${v}
        </td>
        <td style="padding: 10px 14px; text-align: right;">
          ${c.status==="input_required"||c.status==="working"?`
            <button class="btn btn-danger" style="padding: 2px 8px; font-size: 10.5px;" onclick="window.app.promptCancelTask('${S(c.taskId)}')">Cancel</button>
          `:`
            <button class="btn btn-ghost" style="padding: 2px 8px; font-size: 10.5px;" onclick="window.app.inspectTaskDetails('${S(c.taskId)}')">Inspect</button>
          `}
        </td>
      </tr>
    `}).join("");return`
    <!-- Sub-header & Actions -->
    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px;">
      <div style="display: flex; align-items: center; gap: 10px;">
        <span class="brand-badge" style="font-size: 11px; padding: 3px 10px; color: ${n.length>0?"var(--amber-300)":"var(--green-400)"}; border-color: ${n.length>0?"rgba(245, 158, 11, 0.4)":"rgba(52, 211, 153, 0.4)"}; background: ${n.length>0?"rgba(245, 158, 11, 0.1)":"rgba(52, 211, 153, 0.1)"};">
          ${n.length} ACTION REQUIRED
        </span>
        <span style="font-size: 12px; color: var(--text-dim);">
          SEP-2663 Tasks Extension (<code style="color: var(--amber-300); font-family: var(--ff-mono);">io.modelcontextprotocol/tasks</code>) and Unified Human-in-the-Loop execution control.
        </span>
      </div>
      <div style="display: flex; gap: 8px;">
        <button class="btn btn-ghost" onclick="window.app.refreshTasks()" style="font-size: 11.5px;">
          <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21.5 2v6h-6M21.34 15.57a10 10 0 1 1-.57-8.38l5.67-5.67"/></svg>
          Refresh Tasks
        </button>
      </div>
    </div>

    <!-- Top Bento Metrics (Full 12-column span) -->
    <div class="bento-grid">
      <div class="bento-card col-3">
        <div class="stat-label">Input Required (HITL)</div>
        <div class="stat-value" style="color: ${n.length>0?"var(--amber-400)":"var(--text-main)"};">${n.length}</div>
        <div class="stat-sub">Awaiting operator decision or response</div>
      </div>
      <div class="bento-card col-3">
        <div class="stat-label">Working / In-Flight</div>
        <div class="stat-value" style="color: var(--cyan-400);">${r.length}</div>
        <div class="stat-sub">Asynchronous active executions</div>
      </div>
      <div class="bento-card col-3">
        <div class="stat-label">Completed Tasks</div>
        <div class="stat-value" style="color: var(--green-400);">${s.length}</div>
        <div class="stat-sub">Finished successfully</div>
      </div>
      <div class="bento-card col-3">
        <div class="stat-label">Cancelled / Failed</div>
        <div class="stat-value" style="color: ${i.length>0?"var(--red-400)":"var(--text-muted)"};">${o.length+i.length}</div>
        <div class="stat-sub">Terminated or errored</div>
      </div>
    </div>

    <!-- Main Content Bento Split (8 cols queue / 4 cols rules) -->
    <div class="bento-grid">
      <!-- Left Column: Input Required Action Queue -->
      <div class="col-8">
        <div style="font-size: 11px; font-weight: 700; color: var(--amber-400); text-transform: uppercase; letter-spacing: 0.5px; margin-bottom: 12px; display: flex; align-items: center; gap: 6px;">
          <span>⚡ Awaiting Operator Action (${n.length})</span>
        </div>
        <div>
          ${u}
        </div>
      </div>

      <!-- Right Column: Active Governance Rules & Architecture -->
      <div class="col-4">
        <div style="font-size: 11px; font-weight: 700; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.5px; margin-bottom: 12px; display: flex; justify-content: space-between; align-items: center;">
          <span>\uD83D\uDEE1️ Gating Policy Rules</span>
          <button class="btn btn-ghost" style="padding: 2px 8px; font-size: 10.5px;" onclick="window.app.switchTab('policy')">Edit in Policy →</button>
        </div>
        <div class="bento-card" style="margin-bottom: 14px;">
          ${g}
        </div>

        <div class="bento-card">
          <div class="stat-label" style="margin-bottom: 8px;">SEP-2663 Protocol Standard</div>
          <div style="font-size: 11.5px; color: var(--text-dim); line-height: 1.5;">
            Warmplane exposes compliant <code>task_get</code>, <code>task_update</code>, and <code>task_cancel</code> tools directly on the MCP facade, enabling seamless agent delegation with non-blocking lifecycle management.
          </div>
        </div>
      </div>
    </div>

    <!-- Bottom History & Task Registry Table (Full 12 columns) -->
    <div class="bento-card col-12" style="margin-top: 10px;">
      <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px;">
        <div style="font-size: 11px; font-weight: 700; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.5px;">
          \uD83D\uDCDC Task Registry (${l.length})
        </div>
        <div style="display: flex; gap: 6px; align-items: center;">
          <span style="font-size: 11px; color: var(--text-dim);">Filter Status:</span>
          <select class="form-input" style="padding: 3px 8px; font-size: 11px; width: 140px;" onchange="window.app.filterTasksByStatus(this.value)">
            <option value="all" ${a==="all"?"selected":""}>All Statuses</option>
            <option value="input_required" ${a==="input_required"?"selected":""}>input_required</option>
            <option value="working" ${a==="working"?"selected":""}>working</option>
            <option value="completed" ${a==="completed"?"selected":""}>completed</option>
            <option value="cancelled" ${a==="cancelled"?"selected":""}>cancelled</option>
            <option value="failed" ${a==="failed"?"selected":""}>failed</option>
          </select>
        </div>
      </div>

      <div style="overflow-x: auto;">
        <table style="width: 100%; border-collapse: collapse; font-family: var(--ff-mono); font-size: 11.5px; text-align: left;">
          <thead>
            <tr style="border-bottom: 1px solid var(--border); color: var(--text-muted); font-size: 10.5px; text-transform: uppercase;">
              <th style="padding: 10px 14px;">Status</th>
              <th style="padding: 10px 14px;">Capability / Tool</th>
              <th style="padding: 10px 14px;">Task ID</th>
              <th style="padding: 10px 14px;">Progress</th>
              <th style="padding: 10px 14px; text-align: right;">Created</th>
              <th style="padding: 10px 14px; text-align: right;">Actions</th>
            </tr>
          </thead>
          <tbody>
            ${f}
          </tbody>
        </table>
      </div>
    </div>
  `}function S(e){return String(e||"").replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;")}function oe(){let e=d.getState(),t=e.auditEvents||[],a=e.auditStats||{total_events:0,by_status:{success:0,failed:0,denied:0,intercepted:0}},n=e.auditVerification,r=e.auditFilters,s=e.auditTotal??t.length,o=e.auditSelectedEvent,i=Object.keys(e.config?.mcpServers||{}),l=r.limit||25,p=r.offset||0,u=Math.floor(p/l)+1,g=Math.max(1,Math.ceil(s/l)),f=s===0?0:p+1,c=Math.min(p+l,s),y=m.getAuditExportUrl({actor_id:r.search?void 0:void 0,server_id:r.serverId!=="all"?r.serverId:void 0,event_type:r.eventType!=="all"?r.eventType:void 0,status:r.status!=="all"?r.status:void 0,search:r.search.trim()?r.search.trim():void 0},"csv"),h=m.getAuditExportUrl({server_id:r.serverId!=="all"?r.serverId:void 0,event_type:r.eventType!=="all"?r.eventType:void 0,status:r.status!=="all"?r.status:void 0,search:r.search.trim()?r.search.trim():void 0},"jsonl"),v=n?n.is_valid?`
      <div style="display: inline-flex; align-items: center; gap: 6px; padding: 4px 10px; background: rgba(34, 197, 94, 0.1); border: 1px solid rgba(34, 197, 94, 0.3); border-radius: var(--radius-sm); font-size: 11.5px; color: var(--green-400);">
        <span>\uD83D\uDEE1️</span>
        <span style="font-weight: 600;">Chain Verified: 100% Tamper Free (${n.total_records} events)</span>
      </div>
    `:`
      <div style="display: inline-flex; align-items: center; gap: 6px; padding: 4px 10px; background: rgba(239, 68, 68, 0.1); border: 1px solid rgba(239, 68, 68, 0.3); border-radius: var(--radius-sm); font-size: 11.5px; color: var(--red-400);">
        <span>⚠️</span>
        <span style="font-weight: 600;">TAMPER DETECTED at Record #${n.corrupted_at_index}</span>
      </div>
    `:`
    <button class="btn btn-ghost" style="padding: 4px 10px; font-size: 11.5px;" onclick="window.app.verifyAuditChain()">
      \uD83D\uDEE1️ Verify Cryptographic Hash Chain
    </button>
  `,w=i.map((k)=>`<option value="${T(k)}" ${r.serverId===k?"selected":""}>${T(k)}</option>`).join(""),C=`
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
            value="${T(r.search)}"
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
            class="form-input" 
            style="width: 100%; font-size: 12px; height: 32px;"
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
            class="form-input" 
            style="width: 100%; font-size: 12px; height: 32px;"
            onchange="window.app.handleAuditServerFilter(this.value)"
          >
            <option value="all" ${r.serverId==="all"?"selected":""}>All MCP Servers</option>
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
  `,I=`
    <div style="display: flex; justify-content: space-between; align-items: center; padding: 12px 16px; background: rgba(18, 24, 38, 0.5); border-radius: var(--radius-md); border: 1px solid var(--border); margin-top: 16px;">
      <div style="font-size: 12px; color: var(--text-dim); display: flex; align-items: center; gap: 8px;">
        <span>Showing <strong style="color: var(--text-main);">${f}–${c}</strong> of <strong style="color: var(--text-main);">${s}</strong> events</span>
        <span style="color: var(--border);">|</span>
        <span>Page Size:</span>
        <select 
          class="form-input" 
          style="font-size: 11.5px; padding: 2px 24px 2px 8px; height: 28px; width: auto;"
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
          ${u<=1?'disabled style="opacity: 0.4; cursor: not-allowed;"':""}
          onclick="window.app.auditGoToPage(1)"
          title="First Page"
        >
          ⏮ First
        </button>
        <button 
          class="btn btn-ghost" 
          style="padding: 4px 8px; font-size: 11px; height: 28px;"
          ${u<=1?'disabled style="opacity: 0.4; cursor: not-allowed;"':""}
          onclick="window.app.auditPrevPage()"
        >
          ◀ Prev
        </button>
        <span style="font-size: 12px; font-weight: 600; color: var(--text-main); padding: 0 8px;">
          Page ${u} of ${g}
        </span>
        <button 
          class="btn btn-ghost" 
          style="padding: 4px 8px; font-size: 11px; height: 28px;"
          ${u>=g?'disabled style="opacity: 0.4; cursor: not-allowed;"':""}
          onclick="window.app.auditNextPage()"
        >
          Next ▶
        </button>
        <button 
          class="btn btn-ghost" 
          style="padding: 4px 8px; font-size: 11px; height: 28px;"
          ${u>=g?'disabled style="opacity: 0.4; cursor: not-allowed;"':""}
          onclick="window.app.auditGoToPage(${g})"
          title="Last Page"
        >
          Last ⏭
        </button>
      </div>
    </div>
  `,A="";if(t.length===0)A=`
      <div style="padding: 48px 24px; text-align: center; color: var(--text-dim); background: var(--surface-card); border-radius: var(--radius-md); border: 1px dashed var(--border);">
        <div style="font-size: 28px; margin-bottom: 8px;">\uD83D\uDD0D</div>
        <div style="font-size: 14px; font-weight: 600; color: var(--text-main); margin-bottom: 4px;">No Matching Audit Events</div>
        <div style="font-size: 12px; max-width: 420px; margin: 0 auto;">No audit records match your currently selected filters. Try broadening your search or resetting filters.</div>
        <button class="btn btn-ghost" style="margin-top: 14px; font-size: 11.5px;" onclick="window.app.clearAuditFilters()">Reset Filters</button>
      </div>
    `;else A=t.map((k)=>{let M=new Date(Math.floor(k.timestamp_ns/1e6)).toLocaleString(),B='<span class="badge" style="background: rgba(34, 197, 94, 0.15); color: var(--green-400); font-weight: 600;">SUCCESS</span>';if(k.status==="denied")B='<span class="badge" style="background: rgba(239, 68, 68, 0.15); color: var(--red-400); font-weight: 600;">DENIED</span>';else if(k.status==="intercepted")B='<span class="badge" style="background: rgba(234, 179, 8, 0.15); color: var(--amber-300); font-weight: 600;">HITL INTERCEPT</span>';else if(k.status==="failed")B='<span class="badge" style="background: rgba(239, 68, 68, 0.15); color: var(--red-400); font-weight: 600;">FAILED</span>';else if(k.status==="cancelled")B='<span class="badge" style="background: rgba(148, 163, 184, 0.15); color: var(--text-muted); font-weight: 600;">CANCELLED</span>';let D=k.sanitized_args?JSON.stringify(k.sanitized_args):"-",U=k.actor_id||k.operator_id||"anonymous",G=k.server_id||"system",Q=k.capability_id||k.event_type,E=k.execution_latency_us?`${(k.execution_latency_us/1000).toFixed(1)}ms`:"-";return`
        <div class="bento-card" style="margin-bottom: 12px; padding: 16px; border: 1px solid var(--border); transition: border-color 0.15s ease;">
          <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
            <div style="display: flex; align-items: center; gap: 8px;">
              <span style="font-family: var(--ff-mono); font-size: 11px; font-weight: 700; color: var(--text-dim);">${T(k.id)}</span>
              ${B}
              <span style="font-size: 12px; font-weight: 600; color: var(--text-main);">${T(Q)}</span>
            </div>
            <div style="display: flex; align-items: center; gap: 10px;">
              <div style="font-family: var(--ff-mono); font-size: 11px; color: var(--text-muted);">${T(M)}</div>
              <button 
                class="btn btn-ghost" 
                style="padding: 2px 8px; font-size: 11px; height: 24px;" 
                onclick="window.app.selectAuditEvent('${T(k.id)}')"
                title="Inspect event details & cryptographic payload"
              >
                Inspect \uD83D\uDD0D
              </button>
            </div>
          </div>
          
          <div style="display: grid; grid-template-columns: repeat(4, 1fr); gap: 8px; font-size: 11.5px; background: rgba(0,0,0,0.2); padding: 8px 12px; border-radius: var(--radius-sm); margin-bottom: 8px;">
            <div><span style="color: var(--text-muted);">Actor:</span> <strong style="color: var(--text-main);">${T(U)}</strong></div>
            <div><span style="color: var(--text-muted);">Server:</span> <strong style="color: var(--cyan-400);">${T(G)}</strong></div>
            <div><span style="color: var(--text-muted);">Trace:</span> <code style="color: var(--cyan-400); font-size: 10.5px;">${T(k.trace_id)}</code></div>
            <div><span style="color: var(--text-muted);">Latency:</span> <span style="color: var(--amber-300);">${E}</span></div>
          </div>

          <div style="font-family: var(--ff-mono); font-size: 11px; color: var(--text-dim); margin-bottom: 8px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">
            <span style="color: var(--text-muted);">Args:</span> ${T(D)}
          </div>

          <div style="display: flex; justify-content: space-between; align-items: center; border-top: 1px solid var(--border); padding-top: 6px; font-size: 10.5px; font-family: var(--ff-mono); color: var(--text-muted);">
            <div><span style="color: var(--text-dim);">prev_hash:</span> ${T(k.prev_hash.slice(0,16))}...</div>
            <div><span style="color: var(--text-dim);">hash:</span> <span style="color: var(--green-400);">${T(k.hash.slice(0,16))}...</span></div>
          </div>
        </div>
      `}).join("");let L="";if(o){let k=new Date(Math.floor(o.timestamp_ns/1e6)).toISOString();L=`
      <div style="position: fixed; inset: 0; background: rgba(0,0,0,0.7); backdrop-filter: blur(4px); z-index: 9999; display: flex; align-items: center; justify-content: center; padding: 24px;" onclick="if (event.target === this) window.app.selectAuditEvent(null)">
        <div class="bento-card" style="width: 100%; max-width: 720px; max-height: 85vh; display: flex; flex-direction: column; overflow: hidden; background: #0f172a; border: 1px solid var(--border); box-shadow: 0 20px 40px rgba(0,0,0,0.5);">
          <!-- Modal Header -->
          <div style="display: flex; justify-content: space-between; align-items: center; padding: 16px 20px; border-bottom: 1px solid var(--border);">
            <div style="display: flex; align-items: center; gap: 8px;">
              <span style="font-size: 16px;">\uD83D\uDD12</span>
              <h2 style="font-size: 15px; font-weight: 700; color: var(--text-main); margin: 0;">Audit Event Details (${T(o.id)})</h2>
            </div>
            <button class="btn btn-ghost" style="padding: 4px 8px; font-size: 14px;" onclick="window.app.selectAuditEvent(null)">✕</button>
          </div>

          <!-- Modal Body -->
          <div style="padding: 20px; overflow-y: auto; display: flex; flex-direction: column; gap: 14px; font-size: 12px;">
            <div style="display: grid; grid-template-columns: repeat(2, 1fr); gap: 10px; background: rgba(0,0,0,0.25); padding: 12px; border-radius: var(--radius-sm);">
              <div><span style="color: var(--text-muted);">Timestamp:</span> <strong style="color: var(--text-main); font-family: var(--ff-mono); font-size: 11px;">${T(k)}</strong></div>
              <div><span style="color: var(--text-muted);">Status:</span> <strong style="color: var(--text-main);">${T(o.status.toUpperCase())}</strong></div>
              <div><span style="color: var(--text-muted);">Event Type:</span> <strong style="color: var(--text-main);">${T(o.event_type)}</strong></div>
              <div><span style="color: var(--text-muted);">Server:</span> <strong style="color: var(--cyan-400);">${T(o.server_id||"system")}</strong></div>
              <div><span style="color: var(--text-muted);">Capability:</span> <strong style="color: var(--text-main);">${T(o.capability_id||"-")}</strong></div>
              <div><span style="color: var(--text-muted);">Actor / Operator:</span> <strong style="color: var(--text-main);">${T(o.actor_id||o.operator_id||"anonymous")}</strong></div>
              <div><span style="color: var(--text-muted);">Trace ID:</span> <code style="color: var(--cyan-400);">${T(o.trace_id)}</code></div>
              <div><span style="color: var(--text-muted);">Request ID:</span> <code style="color: var(--cyan-400);">${T(o.request_id||"-")}</code></div>
              <div><span style="color: var(--text-muted);">Client IP:</span> <span style="color: var(--text-main);">${T(o.client_ip||"-")}</span></div>
              <div><span style="color: var(--text-muted);">Latency:</span> <span style="color: var(--amber-300);">${o.execution_latency_us?`${(o.execution_latency_us/1000).toFixed(2)} ms`:"-"}</span></div>
            </div>

            ${o.error_message?`
              <div style="background: rgba(239, 68, 68, 0.1); border: 1px solid rgba(239, 68, 68, 0.3); border-radius: var(--radius-sm); padding: 10px 12px; color: var(--red-400);">
                <div style="font-weight: 700; margin-bottom: 2px;">Error (${T(o.error_code||"ERROR")}):</div>
                <div style="font-family: var(--ff-mono); font-size: 11px;">${T(o.error_message)}</div>
              </div>
            `:""}

            <!-- Sanitized Arguments -->
            <div>
              <div style="font-weight: 600; color: var(--text-main); margin-bottom: 4px;">Sanitized Arguments</div>
              <pre style="background: rgba(0,0,0,0.4); padding: 10px; border-radius: var(--radius-sm); border: 1px solid var(--border); font-family: var(--ff-mono); font-size: 11px; max-height: 140px; overflow: auto; margin: 0; color: #cbd5e1;">${T(JSON.stringify(o.sanitized_args||{},null,2))}</pre>
            </div>

            <!-- Sanitized Response -->
            ${o.sanitized_response?`
              <div>
                <div style="font-weight: 600; color: var(--text-main); margin-bottom: 4px;">Sanitized Response</div>
                <pre style="background: rgba(0,0,0,0.4); padding: 10px; border-radius: var(--radius-sm); border: 1px solid var(--border); font-family: var(--ff-mono); font-size: 11px; max-height: 140px; overflow: auto; margin: 0; color: #cbd5e1;">${T(JSON.stringify(o.sanitized_response,null,2))}</pre>
              </div>
            `:""}

            <!-- Cryptographic Hashes -->
            <div style="background: rgba(0,0,0,0.3); padding: 12px; border-radius: var(--radius-sm); border: 1px solid var(--border);">
              <div style="font-weight: 600; color: var(--text-main); margin-bottom: 6px;">Tamper-Evidence Cryptographic Hashes</div>
              <div style="margin-bottom: 6px;">
                <span style="color: var(--text-muted); font-size: 10.5px;">Previous Chain Hash (prev_hash):</span>
                <div style="font-family: var(--ff-mono); font-size: 10.5px; color: var(--text-dim); word-break: break-all;">${T(o.prev_hash)}</div>
              </div>
              <div>
                <span style="color: var(--text-muted); font-size: 10.5px;">Record Hash Signature (hash):</span>
                <div style="font-family: var(--ff-mono); font-size: 10.5px; color: var(--green-400); word-break: break-all;">${T(o.hash)}</div>
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
        ${v}
        <a href="${y}" download class="btn btn-ghost" style="font-size: 11.5px; text-decoration: none;" title="Export current filtered view as CSV">\uD83D\uDCE5 Export CSV</a>
        <a href="${h}" download class="btn btn-ghost" style="font-size: 11.5px; text-decoration: none;" title="Export current filtered view as JSONL">\uD83D\uDCE5 Export JSONL</a>
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
    ${C}

    <!-- Event Timeline List Header -->
    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px;">
      <h2 style="font-size: 14px; font-weight: 600; color: var(--text-main);">Sequential Audit Ledger (SHA-256 Hash Chained)</h2>
      <span style="font-size: 11.5px; color: var(--text-dim);">${t.length} events loaded on this page</span>
    </div>

    <!-- Event Rows -->
    <div>
      ${A}
    </div>

    <!-- Pagination Footer -->
    ${s>0?I:""}

    <!-- Modal Popup for Event Inspection -->
    ${L}
  `}function T(e){if(!e)return"";return String(e).replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;").replace(/'/g,"&#039;")}function ne(){let e=d.getState(),t=e.activeProfile,a=t?e.config.profiles?.[t]:void 0,n=!!a,r=e.config.policy||{},s=a?.policy,o=n?s||{}:r,i=o.allow||[],l=o.deny||[],p=o.redact_keys||o.redactKeys||[],u=o.require_approval||o.requireApproval||[],g=i.length===0?`
    <div style="color: var(--text-dim); font-size: 12px;">${n?"No profile allow list (inherits global rules)":"No allow list (all non-denied operations permitted)"}</div>
  `:i.map((v,w)=>`
    <div style="display: flex; justify-content: space-between; align-items: center; background: var(--surface); padding: 8px 12px; border-radius: var(--radius-sm); border: 1px solid var(--border);">
      <span style="font-family: var(--ff-mono); font-size: 12px; color: var(--green-400);">✔ ${H(v)}</span>
      <button class="btn btn-ghost" style="padding: 2px 6px; font-size: 11px; color: var(--red-400);" onclick="window.app.removePolicyRule('allow', ${w})">✕</button>
    </div>
  `).join(""),f=l.length===0?`
    <div style="color: var(--text-dim); font-size: 12px;">${n?"No profile deny rules configured":"No deny rules configured"}</div>
  `:l.map((v,w)=>`
    <div style="display: flex; justify-content: space-between; align-items: center; background: var(--surface); padding: 8px 12px; border-radius: var(--radius-sm); border: 1px solid var(--border);">
      <span style="font-family: var(--ff-mono); font-size: 12px; color: var(--red-400);">✖ ${H(v)}</span>
      <button class="btn btn-ghost" style="padding: 2px 6px; font-size: 11px; color: var(--red-400);" onclick="window.app.removePolicyRule('deny', ${w})">✕</button>
    </div>
  `).join(""),c=u.length===0?`
    <div style="color: var(--text-dim); font-size: 12px;">${n?"No profile human-in-the-loop triggers configured":"No human-in-the-loop approval rules configured"}</div>
  `:u.map((v,w)=>`
    <div style="display: flex; justify-content: space-between; align-items: center; background: var(--surface); padding: 8px 12px; border-radius: var(--radius-sm); border: 1px solid var(--border);">
      <span style="font-family: var(--ff-mono); font-size: 12px; color: var(--amber-400);">\uD83D\uDEE1️ ${H(v)}</span>
      <button class="btn btn-ghost" style="padding: 2px 6px; font-size: 11px; color: var(--red-400);" onclick="window.app.removePolicyRule('requireApproval', ${w})">✕</button>
    </div>
  `).join(""),y=p.length===0?`
    <div style="color: var(--text-dim); font-size: 12px;">${n?"No profile key redaction patterns configured":"No key redaction patterns configured"}</div>
  `:p.map((v,w)=>`
    <span class="brand-badge" style="color: var(--amber-300); padding: 5px 10px; font-size: 11px; display: inline-flex; align-items: center; gap: 6px;">
      ${H(v)}
      <span style="cursor: pointer; color: var(--red-400); font-weight: bold;" onclick="window.app.removePolicyRule('redact', ${w})">✕</span>
    </span>
  `).join("");return`
    ${n?`
    <div class="bento-card" style="margin-bottom: 16px; background: rgba(245, 158, 11, 0.05); border: 1px solid rgba(245, 158, 11, 0.3); display: flex; justify-content: space-between; align-items: center;">
      <div style="display: flex; align-items: center; gap: 10px;">
        <span style="font-size: 18px;">\uD83D\uDEE1️</span>
        <div>
          <div style="font-size: 13px; font-weight: 700; color: var(--amber-400);">
            Viewing &amp; Editing Policy for Profile Constellation: <code style="font-size: 13px; color: var(--text-main);">${H(t)}</code>
          </div>
          <div style="font-size: 11.5px; color: var(--text-muted); margin-top: 2px;">
            Rules defined here apply specifically when requests target this profile. Deny and HITL rules are strictly additive with global rules.
          </div>
        </div>
      </div>
      <button class="btn btn-ghost" style="font-size: 11px; padding: 4px 10px;" onclick="window.app.setActiveProfile(null)">Switch to Global Policy</button>
    </div>
  `:`
    <div style="margin-bottom: 16px; font-size: 12px; color: var(--text-dim);">
      Global security policy rules governing wildcard access control, human-in-the-loop triggers, and sensitive key masking. (Select an active profile in the top bar to edit per-profile rules).
    </div>
  `}

    <div class="bento-grid">
      <!-- Allow Rules -->
      <div class="bento-card col-4">
        <div class="stat-header">
          <span class="stat-label" style="color: var(--green-400);">Allow List Patterns</span>
        </div>
        <div style="display: flex; flex-direction: column; gap: 8px; margin: 12px 0;">
          ${g}
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
          ${f}
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
          ${y}
        </div>
        <div style="display: flex; gap: 8px; margin-top: 14px; max-width: 420px;">
          <input type="text" class="form-input" id="policy-new-redact" placeholder="e.g. token, api_key, password, secret" onkeydown="if(event.key==='Enter') window.app.submitPolicyRule('redact')">
          <button class="btn btn-ghost" onclick="window.app.submitPolicyRule('redact')">Add Key</button>
        </div>
      </div>

      <!-- Webhooks & ChatOps Alerts -->
      <div class="bento-card col-12" style="border-color: rgba(59, 130, 246, 0.3);">
        <div class="stat-header" style="display: flex; justify-content: space-between; align-items: center;">
          <div>
            <span class="stat-label" style="color: var(--cyan-400);">⚡ ChatOps &amp; Outbound Webhooks</span>
            <span style="font-size: 11px; color: var(--text-dim); margin-left: 8px;">Push actionable HITL approval cards and alerts directly to Slack, Discord, or Teams</span>
          </div>
          <span class="brand-badge" style="color: var(--cyan-400); border-color: rgba(34, 211, 238, 0.3);">Bidirectional</span>
        </div>

        <div style="display: grid; grid-template-columns: repeat(3, 1fr); gap: 14px; margin-top: 14px;">
          <div>
            <label class="form-label" style="font-size: 11px;">Target Webhook URL</label>
            <input type="text" class="form-input" id="policy-webhook-url" placeholder="https://hooks.slack.com/services/... or Discord webhook URL" value="${H(typeof o.webhook==="object"&&o.webhook?o.webhook.url||"":"")}">
          </div>
          <div>
            <label class="form-label" style="font-size: 11px;">Payload Layout Format</label>
            <select class="form-input" id="policy-webhook-format">
              <option value="slack" ${typeof o.webhook==="object"&&o.webhook?.format==="slack"?"selected":""}>Slack Block Kit (Interactive)</option>
              <option value="discord" ${typeof o.webhook==="object"&&o.webhook?.format==="discord"?"selected":""}>Discord Embed &amp; Actions</option>
              <option value="teams" ${typeof o.webhook==="object"&&o.webhook?.format==="teams"?"selected":""}>Microsoft Teams Adaptive Cards</option>
              <option value="generic" ${typeof o.webhook==="object"&&o.webhook?.format==="generic"||!o.webhook?"selected":""}>Generic JSON (Standard)</option>
            </select>
          </div>
          <div>
            <label class="form-label" style="font-size: 11px;">HMAC Secret (or Env Var)</label>
            <input type="text" class="form-input" id="policy-webhook-secret" placeholder="e.g. WARMPLANE_WEBHOOK_SECRET" value="${H(typeof o.webhook==="object"&&o.webhook?o.webhook.secret_env||o.webhook.secretEnv||o.webhook.secret||"":"")}">
          </div>
        </div>

        <div style="display: flex; justify-content: space-between; align-items: center; margin-top: 14px; padding-top: 12px; border-top: 1px solid var(--border-subtle);">
          <div style="display: flex; gap: 8px; align-items: center;">
            <button class="btn btn-primary" onclick="window.app.saveWebhookConfig()">Save Webhook Settings</button>
            <button class="btn btn-ghost" onclick="window.app.testWebhook()">⚡ Send Test Event</button>
          </div>
          <div id="policy-webhook-status" style="font-size: 11px; font-family: var(--ff-mono); color: var(--text-dim);">
            ${typeof o.webhook==="object"&&o.webhook?.url?`Active Target: ${H(o.webhook.url)}`:"No webhook configured"}
          </div>
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
  `}function H(e){return String(e||"").replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;")}function ie(){let e=d.getState(),t=e.config,a=Object.entries(t.capabilityAliases||{}),n=Object.entries(t.resourceAliases||{}),r=Object.entries(t.promptAliases||{}),s="";if(a.length===0&&n.length===0&&r.length===0)s=`
      <div style="padding: 24px; text-align: center; color: var(--text-dim);">
        No facade aliases configured in ${z(e.configPath)}. Add short names to prune token payload sizes.
      </div>
    `;else{for(let[o,i]of a)s+=`
        <div class="feed-row" style="grid-template-columns: 90px 180px 1fr 80px;">
          <span style="color: var(--cyan-400);">Tool</span>
          <span style="font-weight: 700; color: var(--text-main); font-family: var(--ff-mono);">${z(o)}</span>
          <span style="color: var(--text-dim); font-family: var(--ff-mono);">${z(i)}</span>
          <div style="text-align: right;">
            <button class="btn btn-ghost" style="padding: 2px 6px; color: var(--red-400);" onclick="window.app.deleteAlias('tool', '${z(o)}')">✕</button>
          </div>
        </div>
      `;for(let[o,i]of n)s+=`
        <div class="feed-row" style="grid-template-columns: 90px 180px 1fr 80px;">
          <span style="color: var(--green-400);">Resource</span>
          <span style="font-weight: 700; color: var(--text-main); font-family: var(--ff-mono);">${z(o)}</span>
          <span style="color: var(--text-dim); font-family: var(--ff-mono);">${z(i)}</span>
          <div style="text-align: right;">
            <button class="btn btn-ghost" style="padding: 2px 6px; color: var(--red-400);" onclick="window.app.deleteAlias('resource', '${z(o)}')">✕</button>
          </div>
        </div>
      `;for(let[o,i]of r)s+=`
        <div class="feed-row" style="grid-template-columns: 90px 180px 1fr 80px;">
          <span style="color: var(--amber-300);">Prompt</span>
          <span style="font-weight: 700; color: var(--text-main); font-family: var(--ff-mono);">${z(o)}</span>
          <span style="color: var(--text-dim); font-family: var(--ff-mono);">${z(i)}</span>
          <div style="text-align: right;">
            <button class="btn btn-ghost" style="padding: 2px 6px; color: var(--red-400);" onclick="window.app.deleteAlias('prompt', '${z(o)}')">✕</button>
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
      ${s}
    </div>
  `}function z(e){return String(e||"").replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;")}function le(){let e=d.getState(),t=e.config,a=t.profiles||{},n=Object.entries(a),r=t.mcpServers||{},s=e.activeProfile,o="";if(n.length===0)o=`
      <div style="padding: 40px; text-align: center; color: var(--text-dim); background: var(--surface-card); border-radius: var(--radius-md); border: 1px dashed var(--border);">
        <div style="font-size: 15px; color: var(--text-main); font-weight: 600; margin-bottom: 8px;">No Profiles Configured</div>
        <p style="font-size: 12px; margin-bottom: 20px; max-width: 480px; margin-left: auto; margin-right: auto;">
          Profiles allow Warmplane to serve multiple task-relevant server constellations (e.g. <code>coding</code>, <code>support</code>, <code>data</code>) from one running daemon process.
        </p>
        <button class="btn btn-primary" onclick="window.app.openAddProfileModal()">+ Create First Profile</button>
      </div>
    `;else o=n.map(([i,l])=>{let p=s===i,u=l.servers.map((w)=>`<span class="brand-badge" style="${r[w]?"color: var(--cyan-400); border-color: rgba(34, 211, 238, 0.25); background: rgba(34, 211, 238, 0.05);":"color: var(--red-400); border-color: rgba(248, 113, 113, 0.3); background: rgba(248, 113, 113, 0.05);"}">${N(w)}</span>`).join(" "),g=(e.capabilities||[]).filter((w)=>l.servers.includes(w.server)).length,f=!!l.policy,c=l.policy?.allow?.length||0,y=l.policy?.deny?.length||0,h=(l.policy?.require_approval||l.policy?.requireApproval||[]).length,v=(l.policy?.redact_keys||l.policy?.redactKeys||[]).length;return`
        <div class="bento-card" style="margin-bottom: 14px; border-left: ${p?"3px solid var(--amber-400)":"1px solid var(--border)"};">
          <div style="display: flex; justify-content: space-between; align-items: flex-start;">
            <div>
              <div style="display: flex; align-items: center; gap: 8px; margin-bottom: 6px;">
                <span style="font-size: 16px; font-weight: 700; color: var(--text-main); font-family: var(--ff-mono);">${N(i)}</span>
                ${p?'<span class="brand-badge" style="color: var(--amber-400); border-color: rgba(245, 158, 11, 0.4); background: rgba(245, 158, 11, 0.1);">ACTIVE IN UI</span>':""}
                <span class="brand-badge">${l.servers.length} server${l.servers.length===1?"":"s"}</span>
                <span class="brand-badge" style="color: var(--text-dim);">${g} capabilities</span>
                ${f?'<span class="brand-badge" style="color: var(--green-400); border-color: rgba(34, 197, 94, 0.3); background: rgba(34, 197, 94, 0.08);">CUSTOM POLICY</span>':""}
              </div>
              <div style="font-size: 12px; color: var(--text-muted); margin-bottom: 10px;">
                ${N(l.description||"No description provided")}
              </div>
              <div style="display: flex; flex-wrap: wrap; gap: 6px; align-items: center; margin-bottom: ${f?"8px":"0"};">
                <span style="font-size: 11px; color: var(--text-dim); font-weight: 600; text-transform: uppercase;">Servers:</span>
                ${u||'<span style="font-size: 11px; color: var(--text-dim);">None</span>'}
              </div>
              ${f?`
                <div style="display: flex; flex-wrap: wrap; gap: 6px; align-items: center; font-size: 11px;">
                  <span style="color: var(--text-dim); font-weight: 600; text-transform: uppercase;">Policy Overlay:</span>
                  ${c>0?`<span class="brand-badge" style="color: var(--green-400);">Allow: ${c}</span>`:""}
                  ${y>0?`<span class="brand-badge" style="color: var(--red-400);">Deny: ${y}</span>`:""}
                  ${h>0?`<span class="brand-badge" style="color: var(--amber-400);">HITL: ${h}</span>`:""}
                  ${v>0?`<span class="brand-badge" style="color: var(--text-muted);">Redact: ${v}</span>`:""}
                  ${c===0&&y===0&&h===0&&v===0?'<span style="color: var(--text-dim);">Configured</span>':""}
                </div>
              `:""}
            </div>
            
            <div style="display: flex; gap: 8px; align-items: center;">
              ${p?`
                <button class="btn btn-ghost" style="padding: 4px 10px; font-size: 11.5px;" onclick="window.app.setActiveProfile(null)">
                  Deselect
                </button>
              `:`
                <button class="btn btn-primary" style="padding: 4px 10px; font-size: 11.5px;" onclick="window.app.setActiveProfile('${N(i)}')">
                  Activate in UI
                </button>
              `}
              <button class="btn btn-ghost" style="padding: 4px 10px; font-size: 11.5px;" onclick="window.app.openEditProfileModal('${N(i)}')">
                ✏️ Edit
              </button>
              <button class="btn btn-danger" style="padding: 4px 10px; font-size: 11.5px;" onclick="window.app.deleteProfile('${N(i)}')">
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

    ${o}
  `}function N(e){return String(e||"").replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;")}function de(){let t=d.getState().secrets||[],a=t.length,n=t.filter((o)=>o.is_vault).length,r=a-n,s=t.length===0?`
    <div style="padding: 32px; text-align: center; color: var(--text-dim);">
      No environment variables or secrets configured in active servers.
    </div>
  `:t.map((o)=>{let i='<span class="brand-badge" style="color: var(--red-400); border-color: rgba(248, 113, 113, 0.4); background: rgba(248, 113, 113, 0.1);">Plaintext (Unsecured)</span>';if(o.is_vault)i=`<span class="brand-badge" style="color: var(--green-400); border-color: rgba(52, 211, 153, 0.3); background: rgba(52, 211, 153, 0.1);">\uD83D\uDD12 ${O(o.backend)}</span>`;return`
      <div style="display: grid; grid-template-columns: 140px 180px 1fr 180px auto; padding: 10px 16px; border-bottom: 1px solid var(--border-subtle); align-items: center; font-size: 12px;">
        <span style="font-weight: 700; color: var(--text-main);">${O(o.server)}</span>
        <span style="font-family: var(--ff-mono); color: var(--amber-300);">${O(o.key)}</span>
        <span style="font-family: var(--ff-mono); font-size: 11px; color: var(--text-muted); overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">${O(o.display)}</span>
        <div>${i}</div>
        <div style="display: flex; gap: 6px; justify-content: flex-end;">
          ${!o.is_vault?`
            <button class="btn btn-primary" style="padding: 2px 8px; font-size: 11px;" onclick="window.app.quickVaultEnv('${O(o.server)}', '${O(o.key)}')">\uD83D\uDD12 Move to Keychain</button>
          `:`
            <button class="btn btn-ghost" style="padding: 2px 8px; font-size: 11px; color: var(--red-400);" onclick="window.app.deleteVaultSecret('${O(o.key)}')">Delete Key</button>
          `}
        </div>
      </div>
    `}).join("");return`
    <div style="margin-bottom: 16px; font-size: 12px; color: var(--text-dim);">
      Manage native OS Keychain credentials (macOS Keychain, Linux Secret Service, 1Password). Secrets are injected directly in-memory at process launch and never saved to disk in plaintext.
    </div>

    <!-- Stat Header Cards -->
    <div class="bento-grid" style="margin-bottom: 20px;">
      <div class="bento-card col-4">
        <div class="stat-label">Total Secret References</div>
        <div class="stat-value" style="color: var(--cyan-400);">${a}</div>
        <div class="stat-sub">Across all configured MCP servers</div>
      </div>
      <div class="bento-card col-4">
        <div class="stat-label">Secured via Vault / Keychain</div>
        <div class="stat-value" style="color: var(--green-400);">${n}</div>
        <div class="stat-sub">Zero-disk plaintext exposure</div>
      </div>
      <div class="bento-card col-4">
        <div class="stat-label">Plaintext Secrets</div>
        <div class="stat-value" style="color: ${r>0?"var(--red-400)":"var(--green-400)"};">${r}</div>
        <div class="stat-sub">${r>0?"Recommend migrating to Keychain":"All credentials protected"}</div>
      </div>
    </div>

    <!-- Action Drawer / Store New Secret -->
    <div class="bento-card" style="margin-bottom: 20px; padding: 14px 18px; border-color: rgba(59, 130, 246, 0.3);">
      <div style="font-size: 13.5px; font-weight: 700; color: var(--text-main); margin-bottom: 10px;">
        \uD83D\uDD11 Store New Secret in OS Keychain
      </div>
      <div style="display: grid; grid-template-columns: 200px 1fr 140px auto; gap: 10px; align-items: center;">
        <input type="text" class="form-input" id="vault-new-key" placeholder="Key identifier, e.g. github_token" style="font-size: 12px;">
        <input type="password" class="form-input" id="vault-new-val" placeholder="Secret value (will be written to OS Keychain)" style="font-size: 12px;">
        <input type="text" class="form-input" id="vault-new-service" placeholder="Service (warmplane)" value="warmplane" style="font-size: 12px;">
        <button class="btn btn-primary" onclick="window.app.saveNewVaultSecret()">Save to Keychain</button>
      </div>
    </div>

    <!-- Secrets Ledger Table -->
    <div style="background: var(--surface-card); border: 1px solid var(--border); border-radius: var(--radius-md); overflow: hidden;">
      <div style="display: grid; grid-template-columns: 140px 180px 1fr 180px auto; padding: 8px 16px; background: var(--surface-hover); border-bottom: 1px solid var(--border); color: var(--text-muted); font-size: 11px; font-weight: 600;">
        <span>SERVER</span>
        <span>VARIABLE KEY</span>
        <span>VALUE / URI SCHEME</span>
        <span>SECURITY STATUS</span>
        <span style="text-align: right;">ACTION</span>
      </div>
      <div id="secrets-table-rows">
        ${s}
      </div>
    </div>
  `}function O(e){return String(e||"").replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;")}var X=[{id:"github",name:"GitHub",category:"devtools",description:"Explore repositories, issues, pull requests, branches, and commit histories.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-github"],envFields:[{key:"GITHUB_PERSONAL_ACCESS_TOKEN",label:"GitHub Personal Access Token",placeholder:"ghp_...",required:!0,description:"Classic or fine-grained token with repo scope."}]},{id:"git",name:"Git (Local)",category:"devtools",description:"Read local Git repository status, diffs, log histories, and commit changes.",badge:"Official / uvx",command:"uvx",defaultArgs:["--with","mcp<2","mcp-server-git","--repository","."],argsPlaceholder:"--with mcp<2 mcp-server-git --repository /path/to/repo",envFields:[]},{id:"filesystem",name:"Filesystem",category:"devtools",description:"Secure, sandboxed access to local files and directories for AI workflows.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-filesystem","."],argsPlaceholder:"-y @modelcontextprotocol/server-filesystem /allowed/dir1 /allowed/dir2",envFields:[]},{id:"memory",name:"Memory Graph",category:"devtools",description:"Persistent knowledge-graph based memory for multi-turn agent learning.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-memory"],envFields:[]},{id:"chrome-devtools",name:"Chrome DevTools",category:"devtools",description:"Inspect live DOM, execute scripts, read console logs, and capture network traces in Chrome.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-puppeteer"],envFields:[]},{id:"sentry",name:"Sentry",category:"devtools",description:"Query production error events, stack traces, and issue frequencies directly from Sentry.",badge:"uvx / Telemetry",command:"uvx",defaultArgs:["--with","mcp<2","--with","httpx","mcp-server-sentry","--auth-token","sntrys_token"],argsPlaceholder:"--with mcp<2 --with httpx mcp-server-sentry --auth-token YOUR_SENTRY_TOKEN",envFields:[{key:"SENTRY_AUTH_TOKEN",label:"Sentry Auth Token",placeholder:"sntrys_...",required:!0}]},{id:"playwright",name:"Playwright Browser",category:"browser",description:"Headless / headed browser automation for scraping, form filling, and UI interaction.",badge:"Popular #1 / npx",command:"npx",defaultArgs:["-y","@executeautomation/playwright-mcp-server"],envFields:[]},{id:"puppeteer",name:"Puppeteer",category:"browser",description:"Official browser automation server for web page scraping and screenshot capture.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-puppeteer"],envFields:[]},{id:"brave-search",name:"Brave Search",category:"browser",description:"Real-time privacy-preserving web search and local point-of-interest query engine.",badge:"Official / Search",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-brave-search"],envFields:[{key:"BRAVE_API_KEY",label:"Brave Search API Key",placeholder:"BSA...",required:!0}]},{id:"tavily",name:"Tavily Search",category:"browser",description:"AI-optimized web search engine structured specifically for LLM context injection.",badge:"Community / Stdio",command:"npx",defaultArgs:["-y","tavily-mcp"],envFields:[{key:"TAVILY_API_KEY",label:"Tavily API Key",placeholder:"tvly-...",required:!0}]},{id:"fetch",name:"Fetch / Web Markdown",category:"browser",description:"Download web pages, strip clutter, and convert raw HTML to clean markdown text.",badge:"Official / uvx",command:"uvx",defaultArgs:["mcp-server-fetch"],envFields:[]},{id:"postgres",name:"PostgreSQL",category:"database",description:"Read schemas, inspect tables, and execute SQL queries against PostgreSQL databases.",badge:"Official / Database",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-postgres","postgresql://user:pass@localhost:5432/mydb"],argsPlaceholder:"-y @modelcontextprotocol/server-postgres postgresql://user:pass@localhost:5432/dbname",envFields:[]},{id:"sqlite",name:"SQLite",category:"database",description:"Local embedded SQLite query runner and schema inspector.",badge:"Official / uvx",command:"uvx",defaultArgs:["--with","mcp<2","mcp-server-sqlite","--db-path","./app.db"],argsPlaceholder:"--with mcp<2 mcp-server-sqlite --db-path /path/to/database.sqlite",envFields:[]},{id:"supabase",name:"Supabase",category:"database",description:"Query database tables, manage auth policies, and inspect storage in Supabase.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@supabase/mcp-server-supabase@latest"],envFields:[{key:"SUPABASE_ACCESS_TOKEN",label:"Supabase Personal Access Token",placeholder:"sbp_...",required:!0},{key:"SUPABASE_PROJECT_REF",label:"Supabase Project Reference ID",placeholder:"abcdefghijklmnop",required:!1}]},{id:"redis",name:"Redis",category:"database",description:"Inspect cached keys, hash sets, lists, TTLs, and pub/sub channels in Redis.",badge:"Official / Key-Value",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-redis","redis://localhost:6379"],argsPlaceholder:"-y @modelcontextprotocol/server-redis redis://localhost:6379",envFields:[]},{id:"s3",name:"AWS S3 / Cloud Storage",category:"database",description:"Browse S3 buckets, fetch object metadata, and download files from cloud storage.",badge:"Community / Stdio",command:"npx",defaultArgs:["-y","@geunoh/s3-mcp-server"],argsPlaceholder:"-y @geunoh/s3-mcp-server",envFields:[{key:"AWS_ACCESS_KEY_ID",label:"AWS Access Key ID",placeholder:"AKIA...",required:!0},{key:"AWS_SECRET_ACCESS_KEY",label:"AWS Secret Access Key",placeholder:"...",required:!0},{key:"AWS_REGION",label:"AWS Region",placeholder:"us-east-1",required:!1}]},{id:"linear",name:"Linear",category:"productivity",description:"Search, create, and triage Linear issues, cycles, teams, and project roadmaps.",badge:"Productivity / Stdio",command:"npx",defaultArgs:["-y","mcp-linear"],envFields:[{key:"LINEAR_API_KEY",label:"Linear API Key",placeholder:"lin_api_...",required:!0}]},{id:"slack",name:"Slack",category:"productivity",description:"Read channels, post messages, inspect threads, and search team discussions.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-slack"],envFields:[{key:"SLACK_BOT_TOKEN",label:"Slack Bot User Token",placeholder:"xoxb-...",required:!0},{key:"SLACK_TEAM_ID",label:"Slack Team ID",placeholder:"T01234567",required:!0}]},{id:"notion",name:"Notion",category:"productivity",description:"Search Notion workspace pages, read nested blocks, and query database entries.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@notionhq/notion-mcp-server"],envFields:[{key:"NOTION_TOKEN",label:"Notion Internal Integration Token",placeholder:"secret_...",required:!0}]},{id:"jira",name:"Jira / Atlassian",category:"productivity",description:"Manage Jira issues, search JQL, read sprint statuses, and inspect boards.",badge:"uvx / Atlassian",command:"uvx",defaultArgs:["--with","mcp<2","mcp-server-jira","--jira-base-url","https://your-domain.atlassian.net"],argsPlaceholder:"--with mcp<2 mcp-server-jira --jira-base-url https://org.atlassian.net",envFields:[{key:"JIRA_TOKEN",label:"Atlassian API Token",placeholder:"ATATT3...",required:!0}]},{id:"google-drive",name:"Google Drive",category:"productivity",description:"Search, list, and read documents, spreadsheets, and drive files.",badge:"Community / Stdio",command:"npx",defaultArgs:["-y","@piotr-agier/google-drive-mcp"],envFields:[{key:"GOOGLE_APPLICATION_CREDENTIALS",label:"Google Credentials JSON Path",placeholder:"/path/to/credentials.json",required:!0}]},{id:"docker",name:"Docker",category:"cloud",description:"Inspect running containers, tail container logs, list images, and manage compose services.",badge:"uvx / DevOps",command:"uvx",defaultArgs:["mcp-server-docker"],envFields:[]},{id:"kubernetes",name:"Kubernetes (K8s)",category:"cloud",description:"Query cluster pods, services, deployment status, and inspect Kubernetes logs.",badge:"Popular / Stdio",command:"npx",defaultArgs:["-y","@strowk/mcp-k8s"],envFields:[{key:"KUBECONFIG",label:"Kubeconfig File Path (Optional)",placeholder:"~/.kube/config",required:!1}]},{id:"cloudflare",name:"Cloudflare",category:"cloud",description:"Manage Cloudflare Workers, KV namespaces, D1 databases, Vectorize indexes, and DNS.",badge:"Official / Cloudflare",command:"npx",defaultArgs:["-y","@cloudflare/mcp-server-cloudflare","run","dummy_account_id"],argsPlaceholder:"-y @cloudflare/mcp-server-cloudflare run YOUR_ACCOUNT_ID",envFields:[{key:"CLOUDFLARE_API_TOKEN",label:"Cloudflare API Token",placeholder:"...",required:!0},{key:"CLOUDFLARE_ACCOUNT_ID",label:"Cloudflare Account ID",placeholder:"...",required:!0}]},{id:"terraform",name:"Terraform",category:"cloud",description:"Inspect Terraform state files, resource dependency graphs, and plan previews.",badge:"Community / IaC",command:"npx",defaultArgs:["-y","@mseep/terraform-mcp-server"],envFields:[]}];class ce{activeTemplateCategory="all";activeTemplateFilter="";selectedTemplate=null;async init(){let e=window.location.port?`:${window.location.port}`:"",t=document.getElementById("daemon-port-label");if(t)t.textContent=`Daemon ${e}`;await this.refreshData(),this.initSSE(),this.render(),d.subscribe(()=>{this.render()})}auditSearchTimeout=null;async refreshData(){try{let e=d.getState(),t=e.auditFilters,a=e.activeProfile||void 0,[n,r,s,o,i,l,p,u,g,f,c]=await Promise.all([m.getConfig(),m.listCapabilities(a),m.listResources(a),m.listPrompts(a),m.getCatalogEvents(),m.listApprovals(),m.listTasks(),m.listAuditEvents({server_id:t.serverId!=="all"?t.serverId:void 0,event_type:t.eventType!=="all"?t.eventType:void 0,status:t.status!=="all"?t.status:void 0,search:t.search.trim()?t.search.trim():void 0,limit:t.limit,offset:t.offset}),m.getAuditStats(),m.getClients().catch(()=>({ok:!1,clients:[]})),m.getSecrets().catch(()=>({ok:!1,secrets:[],keychain_service:"warmplane"}))]);if(f&&f.ok&&Array.isArray(f.clients))d.setState({clients:f.clients});if(c&&c.ok&&Array.isArray(c.secrets))d.setState({secrets:c.secrets});if(n.ok)d.setState({configPath:n.config_path,config:n.config,serverStatuses:n.server_statuses||{},circuitBreakers:n.circuit_breakers||[],metrics:{totalCatalogRequests:n.metrics?.total_catalog_requests||0,totalEtagHits:n.metrics?.total_etag_hits||0,totalToolCalls:n.metrics?.total_tool_calls||0,totalToolDurationUs:n.metrics?.total_tool_duration_us||0}});if(r&&Array.isArray(r.capabilities))d.setState({capabilities:r.capabilities,capabilitiesHiddenByPolicy:r.hidden_by_policy||0});if(s&&Array.isArray(s.resources))d.setState({resources:s.resources,resourcesHiddenByPolicy:s.hidden_by_policy||0});if(o&&Array.isArray(o.prompts))d.setState({prompts:o.prompts,promptsHiddenByPolicy:o.hidden_by_policy||0});if(i&&Array.isArray(i.events))d.setState({catalogEvents:i.events});if(l&&Array.isArray(l.approvals))d.setState({approvals:l.approvals});if(p&&Array.isArray(p.tasks))d.setState({tasks:p.tasks});if(u&&Array.isArray(u.events))d.setState({auditEvents:u.events,auditTotal:u.total??u.events.length});if(g&&g.ok)d.setState({auditStats:g})}catch(e){console.error("Failed to fetch daemon state:",e)}}async refreshAuditEvents(){try{let t=d.getState().auditFilters,[a,n]=await Promise.all([m.listAuditEvents({server_id:t.serverId!=="all"?t.serverId:void 0,event_type:t.eventType!=="all"?t.eventType:void 0,status:t.status!=="all"?t.status:void 0,search:t.search.trim()?t.search.trim():void 0,limit:t.limit,offset:t.offset}),m.getAuditStats()]);if(a&&Array.isArray(a.events))d.setState({auditEvents:a.events,auditTotal:a.total??a.events.length});if(n&&n.ok)d.setState({auditStats:n})}catch(e){console.error("Failed to refresh audit events:",e)}}handleAuditSearchInput(e){let a={...d.getState().auditFilters,search:e,offset:0};d.setState({auditFilters:a}),clearTimeout(this.auditSearchTimeout),this.auditSearchTimeout=setTimeout(()=>{this.refreshAuditEvents()},250)}handleAuditStatusFilter(e){let t=d.getState();d.setState({auditFilters:{...t.auditFilters,status:e,offset:0}}),this.refreshAuditEvents()}handleAuditEventTypeFilter(e){let t=d.getState();d.setState({auditFilters:{...t.auditFilters,eventType:e,offset:0}}),this.refreshAuditEvents()}handleAuditServerFilter(e){let t=d.getState();d.setState({auditFilters:{...t.auditFilters,serverId:e,offset:0}}),this.refreshAuditEvents()}handleAuditPageSize(e){let t=parseInt(e,10)||25,a=d.getState();d.setState({auditFilters:{...a.auditFilters,limit:t,offset:0}}),this.refreshAuditEvents()}clearAuditFilters(){let e=d.getState();d.setState({auditFilters:{search:"",status:"all",eventType:"all",serverId:"all",limit:e.auditFilters.limit||25,offset:0}}),this.refreshAuditEvents()}auditPrevPage(){let e=d.getState(),{limit:t,offset:a}=e.auditFilters,n=Math.max(0,a-t);if(n!==a)d.setState({auditFilters:{...e.auditFilters,offset:n}}),this.refreshAuditEvents()}auditNextPage(){let e=d.getState(),{limit:t,offset:a}=e.auditFilters,n=e.auditTotal;if(a+t<n)d.setState({auditFilters:{...e.auditFilters,offset:a+t}}),this.refreshAuditEvents()}auditGoToPage(e){let t=d.getState(),{limit:a}=t.auditFilters,n=Math.max(0,(e-1)*a);d.setState({auditFilters:{...t.auditFilters,offset:n}}),this.refreshAuditEvents()}selectAuditEvent(e){if(!e){d.setState({auditSelectedEvent:null});return}let a=d.getState().auditEvents.find((n)=>n.id===e)||null;d.setState({auditSelectedEvent:a})}async verifyAuditChain(){try{let e=await m.verifyAuditChain();if(e&&e.report)d.setState({auditVerification:e.report})}catch(e){console.error("Failed to verify audit chain:",e)}}async refreshApprovals(){try{let e=await m.listApprovals();if(e&&Array.isArray(e.approvals))d.setState({approvals:e.approvals})}catch(e){console.error("Failed to refresh approvals:",e)}}initSSE(){try{let e=new EventSource("/v1/resources/updates");e.onmessage=(t)=>{d.addEventLog("SSE","/v1/resources/updates","UPDATED","0.1ms"),this.refreshData()}}catch(e){console.warn("SSE connection unavailable")}}switchTab(e){d.setState({activeTab:e}),this.refreshData()}render(){let e=d.getState(),t=document.getElementById("app-main");if(!t)return;let a=(e.tasks||[]).filter((l)=>l.status==="input_required").length,n=(e.approvals||[]).filter((l)=>l.status==="pending").length,r=Math.max(a,n),s=document.getElementById("nav-approvals-badge");if(s)s.textContent=r>0?`${r}`:"",s.style.display=r>0?"inline-block":"none";document.querySelectorAll(".nav-item").forEach((l)=>{let p=l.getAttribute("data-tab");if(p===e.activeTab||e.activeTab==="tasks"&&p==="approvals"||e.activeTab==="approvals"&&p==="tasks")l.classList.add("active");else l.classList.remove("active")});let o=document.getElementById("top-title"),i={overview:"Overview Cockpit",servers:"Server Hub & Connections",playground:"MCP Capability Playground",tasks:"SEP-2663 Tasks & HITL Review",approvals:"SEP-2663 Tasks & HITL Review",audit:"WORM Audit & Compliance Ledger",policy:"Security Governance & Redaction",secrets:"Native OS Keychain & Secrets Vault",aliases:"Facade & Alias Studio",profiles:"Server Constellation Profiles"};if(o)o.textContent=i[e.activeTab]||"Control Deck";switch(this.renderTopProfileSelector(),e.activeTab){case"overview":t.innerHTML=te();break;case"servers":t.innerHTML=se();break;case"playground":t.innerHTML=re();break;case"tasks":case"approvals":t.innerHTML=ae(e);break;case"audit":t.innerHTML=oe();break;case"policy":t.innerHTML=ne();break;case"secrets":t.innerHTML=de();break;case"aliases":t.innerHTML=ie();break;case"profiles":t.innerHTML=le();break}}toggleClientsCollapse(){let e=d.getState().clientsCollapsed;d.setState({clientsCollapsed:!e}),this.render()}async saveNewVaultSecret(){let e=document.getElementById("vault-new-key"),t=document.getElementById("vault-new-val"),a=document.getElementById("vault-new-service"),n=e?.value.trim(),r=t?.value.trim(),s=a?.value.trim()||"warmplane";if(!n||!r){alert("Key and secret value are required");return}try{let o=await m.saveSecret(n,r,s);if(o.ok){if(alert(`Secret '${n}' saved securely into OS Keychain!
Reference: ${o.uri}`),e)e.value="";if(t)t.value="";await this.refreshData()}else alert(`Failed to save secret: ${o.error}`)}catch(o){alert(`Error saving secret: ${o.message}`)}}async deleteVaultSecret(e){if(!confirm(`Are you sure you want to remove secret '${e}' from OS Keychain?`))return;try{let t=await m.deleteSecret(e);if(t.ok)await this.refreshData();else alert(`Failed to delete secret: ${t.error}`)}catch(t){alert(`Error deleting secret: ${t.message}`)}}async quickVaultEnv(e,t){let a=prompt(`Enter secret value to store in OS Keychain for ${e}.${t}:`);if(!a)return;try{let n=await m.saveSecret(t,a,"warmplane");if(!n.ok){alert(`Failed to save to Keychain: ${n.error}`);return}let o=(d.getState().config.mcpServers||{})[e];if(o){let i={...o.env||{},[t]:`keychain://warmplane/${t}`},l={...o,env:i},p=await m.upsertServer(e,l);if(p.ok)await this.refreshData(),alert(`Successfully migrated ${e}.${t} to OS Keychain!`);else alert(`Failed to update server config: ${p.error}`)}}catch(n){alert(`Error during migration: ${n.message}`)}}async refreshTasks(){try{let e=await m.listTasks();if(e&&Array.isArray(e.tasks))d.setState({tasks:e.tasks})}catch(e){console.error("Failed to refresh tasks:",e)}}filterTasksByStatus(e){d.setState({taskFilterStatus:e})}togglePlaygroundAsyncTask(e){d.setState({playgroundAsyncTask:e})}async submitTaskInputResponses(e){let a=d.getState().tasks.find((s)=>s.taskId===e)?.inputRequests||{},n=Object.keys(a),r={};if(n.length>0)for(let s of n){let o=a[s];if(o&&o.type==="approval_review"){let i=document.getElementById(`task-input-${e}-${s}-decision`),l=document.getElementById(`task-input-${e}-${s}`),p=i?i.value==="true":!0,u=void 0;if(l&&l.value.trim())try{u=JSON.parse(l.value.trim())}catch{alert("Invalid JSON in parameters editor");return}r[s]={approved:p,modified_args:u,reason:p?void 0:"Operator rejected execution via Tasks review"}}else{let i=document.getElementById(`task-input-${e}-${s}`);if(i){let l=i.value.trim();try{r[s]=JSON.parse(l)}catch{r[s]=l}}}}else{let s=document.getElementById(`task-raw-input-${e}`);if(s&&s.value.trim())try{Object.assign(r,JSON.parse(s.value.trim()))}catch{alert("Invalid JSON in raw input responses");return}}try{let s=await m.updateTask(e,r);if(s.ok)await this.refreshTasks();else alert(`Task update failed: ${s.error?.message||s.error||"Unknown error"}`)}catch(s){alert(`Error updating task: ${s.message}`)}}async promptCancelTask(e){let t=prompt("Reason for cancelling task:");if(t===null)return;try{let a=await m.cancelTask(e,t||void 0);if(a.ok)await this.refreshTasks();else alert(`Task cancellation failed: ${a.error?.message||a.error||"Unknown error"}`)}catch(a){alert(`Error cancelling task: ${a.message}`)}}async inspectTaskDetails(e){try{let t=await m.getTask(e);if(t.ok&&t.task)alert(`Task [${t.task.taskId}]
Status: ${t.task.status}
Progress: ${Math.round((t.task.progress||0)*100)}%
Payload: ${JSON.stringify(t.task.result||t.task.error||t.task.inputRequests||{},null,2)}`)}catch(t){alert(`Failed to fetch task: ${t.message}`)}}async submitApproval(e){let t=document.getElementById(`appr-operator-${e}`),a=document.getElementById(`appr-args-${e}`),n=t?.value.trim()||"security-operator",r=void 0;if(a&&a.value.trim())try{r=JSON.parse(a.value.trim())}catch{alert("Invalid JSON in arguments editor");return}let s=await m.approveTicket(e,n,r);if(s.ok)await this.refreshApprovals(),await this.refreshTasks();else alert(`Approval failed: ${s.error||"Unknown error"}`)}async promptReject(e){let t=prompt("Reason for rejection (will be returned to the calling agent):");if(t===null)return;let n=document.getElementById(`appr-operator-${e}`)?.value.trim()||"security-operator",r=await m.rejectTicket(e,n,t);if(r.ok)await this.refreshApprovals(),await this.refreshTasks();else alert(`Rejection failed: ${r.error||"Unknown error"}`)}setPlaygroundMode(e){d.setState({playgroundMode:e})}selectCapability(e){d.setState({selectedCapabilityId:e});let t=d.getState().capabilities.find((n)=>n.id===e),a=document.getElementById("pg-args-input");if(t){let n=W(t.input_schema,!1),r=JSON.stringify(n,null,2);if(a)a.value=r;let s={...d.getState().playgroundArgs||{}};s[e]=r,d.getState().playgroundArgs=s}}selectResource(e){d.setState({selectedResourceId:e})}selectPrompt(e){d.setState({selectedPromptId:e})}filterResources(e){let t=e.toLowerCase().trim(),n=(d.getState().resources||[]).filter((s)=>s.id.toLowerCase().includes(t)||s.name&&s.name.toLowerCase().includes(t)||s.uri&&s.uri.toLowerCase().includes(t)||s.server&&s.server.toLowerCase().includes(t)),r=document.getElementById("pg-res-list");if(r)if(n.length===0)r.innerHTML=`
          <div style="padding: 24px 16px; text-align: center; color: var(--text-dim); font-size: 11.5px;">
            No resources match "${b(e)}"
          </div>
        `;else r.innerHTML=n.map((s)=>{let o=s.id===d.getState().selectedResourceId?"active":"",i=s.uri?s.uri.split(":")[0]:"res";return`
            <div class="cap-item ${o}" onclick="window.app.selectResource('${b(s.id)}')">
              <div style="display: flex; justify-content: space-between; align-items: center;">
                <span style="font-weight: 600; color: var(--text-main); font-family: var(--ff-mono); font-size: 12px;">${b(s.name||s.id)}</span>
                <span class="badge" style="font-size: 9.5px; background: rgba(56, 189, 248, 0.15); color: var(--cyan-400);">${b(i)}</span>
              </div>
              <div style="font-size: 11px; color: var(--text-dim); margin-top: 2px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">${b(s.uri)}</div>
              <div style="display: flex; justify-content: space-between; font-size: 10px; color: var(--text-muted); margin-top: 4px;">
                <span>server: ${b(s.server||"local")}</span>
                <span>${b(s.mime_type||"text/plain")}</span>
              </div>
            </div>
          `}).join("")}filterPrompts(e){let t=e.toLowerCase().trim(),n=(d.getState().prompts||[]).filter((s)=>s.id.toLowerCase().includes(t)||s.name&&s.name.toLowerCase().includes(t)||s.description&&s.description.toLowerCase().includes(t)||s.server&&s.server.toLowerCase().includes(t)),r=document.getElementById("pg-prompt-list");if(r)if(n.length===0)r.innerHTML=`
          <div style="padding: 24px 16px; text-align: center; color: var(--text-dim); font-size: 11.5px;">
            No prompts match "${b(e)}"
          </div>
        `;else r.innerHTML=n.map((s)=>{let o=s.id===d.getState().selectedPromptId?"active":"",i=s.arguments?s.arguments.length:0;return`
            <div class="cap-item ${o}" onclick="window.app.selectPrompt('${b(s.id)}')">
              <div style="display: flex; justify-content: space-between; align-items: center;">
                <span style="font-weight: 600; color: var(--text-main); font-family: var(--ff-mono); font-size: 12px;">${b(s.name||s.id)}</span>
                <span class="badge" style="font-size: 9.5px; background: rgba(168, 85, 247, 0.15); color: var(--purple-400);">${i} args</span>
              </div>
              <div style="font-size: 11px; color: var(--text-dim); margin-top: 2px;">${b(s.description||s.title||"Prompt template")}</div>
              <div style="font-size: 10px; color: var(--text-muted); margin-top: 4px;">server: ${b(s.server||"local")}</div>
            </div>
          `}).join("")}updatePlaygroundArgs(e){let t=d.getState(),a=t.selectedCapabilityId||(t.capabilities[0]?t.capabilities[0].id:null);if(!a)return;let n={...t.playgroundArgs||{}};n[a]=e,t.playgroundArgs=n}fillPlaygroundSampleArgs(e=!1){let t=d.getState(),a=t.selectedCapabilityId||(t.capabilities[0]?t.capabilities[0].id:null),n=t.capabilities.find((i)=>i.id===a),r=document.getElementById("pg-args-input");if(!r)return;if(!n||!n.input_schema){if(r.value="{}",a){let i={...t.playgroundArgs||{}};i[a]="{}",t.playgroundArgs=i}return}let s=W(n.input_schema,e),o=JSON.stringify(s,null,2);if(r.value=o,a){let i={...t.playgroundArgs||{}};i[a]=o,t.playgroundArgs=i}}formatPlaygroundArgs(){let e=d.getState(),t=e.selectedCapabilityId||(e.capabilities[0]?e.capabilities[0].id:null),a=document.getElementById("pg-args-input");if(a)try{let n=JSON.parse(a.value||"{}"),r=JSON.stringify(n,null,2);if(a.value=r,t){let s={...e.playgroundArgs||{}};s[t]=r,e.playgroundArgs=s}}catch(n){alert(`Cannot format JSON: ${n.message}`)}}insertPlaygroundArgKey(e,t,a){let n=d.getState(),r=n.selectedCapabilityId||(n.capabilities[0]?n.capabilities[0].id:null),s=document.getElementById("pg-args-input");if(s){let o={};try{o=JSON.parse(s.value||"{}")}catch{o={}}if(o[e]===void 0)if(a!==null&&a!==void 0)o[e]=a;else switch(t){case"string":o[e]=`sample_${e}`;break;case"number":case"integer":o[e]=0;break;case"boolean":o[e]=!0;break;case"array":o[e]=[];break;case"object":o[e]={};break;default:o[e]=`sample_${e}`}let i=JSON.stringify(o,null,2);if(s.value=i,r){let l={...n.playgroundArgs||{}};l[r]=i,n.playgroundArgs=l}}}fillBatchStepSampleArgs(e){let t=d.getState(),a=[...t.batchSteps||[]],n=a[e];if(!n||!n.capability_id)return;let r=t.capabilities.find((l)=>l.id===n.capability_id);if(!r||!r.input_schema)return;let s=r.input_schema.properties||{},o={};for(let[l,p]of Object.entries(s))if(p.default!==void 0)o[l]=p.default;else if(Array.isArray(p.enum)&&p.enum.length>0)o[l]=p.enum[0];else switch(p.type||"string"){case"string":o[l]=`sample_${l}`;break;case"number":case"integer":o[l]=0;break;case"boolean":o[l]=!0;break;case"array":o[l]=[];break;case"object":o[l]={};break;default:o[l]=`sample_${l}`}let i=JSON.stringify(o,null,2);a[e]={...a[e],argsJson:i},d.setState({batchSteps:a})}filterCapabilities(e){let t=e.toLowerCase().trim(),n=d.getState().capabilities.filter((s)=>s.id.toLowerCase().includes(t)||s.summary&&s.summary.toLowerCase().includes(t)||s.server&&s.server.toLowerCase().includes(t)),r=document.getElementById("pg-cap-list");if(r)if(n.length===0)r.innerHTML=`
          <div style="padding: 24px 16px; text-align: center; color: var(--text-dim); font-size: 11.5px;">
            No capabilities match "${b(e)}"
          </div>
        `;else r.innerHTML=n.map((s)=>`
          <div class="cap-item ${s.id===d.getState().selectedCapabilityId?"active":""}" onclick="window.app.selectCapability('${b(s.id)}')">
            <div style="display: flex; justify-content: space-between; align-items: center;">
              <span style="font-weight: 600; color: var(--text-main); font-family: var(--ff-mono); font-size: 12px;">${b(s.id)}</span>
              <span style="font-size: 10px; color: var(--green-400);">${b(s.mode||"read")}</span>
            </div>
            <div style="font-size: 11px; color: var(--text-dim); margin-top: 2px;">server: ${b(s.server||"local")}</div>
          </div>
        `).join("")}async executePlaygroundTool(){let e=d.getState(),t=e.selectedCapabilityId||(e.capabilities[0]?e.capabilities[0].id:null);if(!t)return;let a=document.getElementById("pg-args-input")?.value||"{}",n=document.getElementById("pg-context-input")?.value||void 0,r=document.getElementById("pg-jsonpath-input")?.value.trim()||void 0,s=document.getElementById("pg-limit-lines-input")?.value.trim()||void 0,o=document.getElementById("pg-truncate-bytes-input")?.value.trim()||void 0,i={};try{i=JSON.parse(a)}catch{alert("Invalid arguments JSON object");return}if(r)i._jsonpath=r;if(s&&!isNaN(Number(s)))i._limit_lines=Number(s);if(o&&!isNaN(Number(o)))i._truncate_bytes=Number(o);let l=`op-${Date.now()}`;d.setState({isExecuting:!0,activeRequestId:l});let p=e.activeProfile||void 0,u=e.playgroundAsyncTask||!1;try{let g=await m.callCapability({capability_id:t,args:i,request_id:l,async_task:u?!0:void 0,context:{operation_id:n||l}},p);if(d.setState({isExecuting:!1,activeRequestId:null,executionResult:{status:g.status,durationMs:g.durationMs,data:g.data}}),g.status===202||g.data?.resultType==="task")this.refreshTasks();d.addEventLog("POST",`/v1/tools/call → ${t}`,g.status===200?"200 OK":`HTTP ${g.status}`,`${g.durationMs.toFixed(1)}ms`),m.getConfig().then((f)=>{if(f.ok&&f.circuit_breakers)d.setState({circuitBreakers:f.circuit_breakers})})}catch(g){d.setState({isExecuting:!1,activeRequestId:null,executionResult:{status:500,durationMs:0,data:{error:g.toString()}}})}}async cancelActiveOperation(){let t=d.getState().activeRequestId;if(t)try{await m.cancelOperation(t)}catch(a){console.warn("Failed to send cancel signal:",a)}d.setState({isExecuting:!1,activeRequestId:null,executionResult:{status:499,durationMs:0,data:{ok:!1,error:{code:"CANCELLED",message:"Operation cancelled by operator"}}}})}openBatchModal(){let e=d.getState(),t=e.batchSteps;if(!t||t.length===0)t=[{id:"step_1",capability_id:e.selectedCapabilityId||(e.capabilities[0]?e.capabilities[0].id:""),argsJson:"{}",continue_on_error:!1},{id:"step_2",capability_id:"",argsJson:"{}",continue_on_error:!0}],d.setState({batchSteps:t});d.setState({isBatchModalOpen:!0})}closeBatchModal(){d.setState({isBatchModalOpen:!1})}addBatchStep(){let t=[...d.getState().batchSteps||[]],a=t.length+1;t.push({id:`step_${a}`,capability_id:"",argsJson:"{}",continue_on_error:!1}),d.setState({batchSteps:t})}removeBatchStep(e){let a=[...d.getState().batchSteps||[]];if(a.length<=1){alert("Pipeline must contain at least one execution step.");return}a.splice(e,1);let n=a.map((r,s)=>({...r,id:`step_${s+1}`}));d.setState({batchSteps:n})}updateBatchStepCapability(e,t){let n=[...d.getState().batchSteps||[]];if(n[e])n[e]={...n[e],capability_id:t},d.setState({batchSteps:n})}updateBatchStepContinueOnError(e,t){let n=[...d.getState().batchSteps||[]];if(n[e])n[e]={...n[e],continue_on_error:t},d.setState({batchSteps:n})}updateBatchStepArgs(e,t){let a=d.getState(),n=[...a.batchSteps||[]];if(n[e])n[e]={...n[e],argsJson:t},a.batchSteps[e].argsJson=t}appendBatchVariable(e,t){let n=[...d.getState().batchSteps||[]],r=document.getElementById(`batch-step-args-${e}`);if(r){let s=r.value,o=r.selectionStart||s.length,i=r.selectionEnd||s.length,l=s.substring(0,o)+t+s.substring(i);if(r.value=l,n[e])n[e]={...n[e],argsJson:l},d.setState({batchSteps:n})}}async executeBatchPipeline(){let e=d.getState(),t=e.batchSteps||[],a=[];for(let r=0;r<t.length;r++){let s=t[r];if(!s.capability_id){alert(`Please select a capability for Step ${r+1}`);return}let o={};try{o=JSON.parse(s.argsJson||"{}")}catch{alert(`Invalid JSON in Step ${r+1} arguments`);return}a.push({id:s.id||`step_${r+1}`,capability_id:s.capability_id,args:o,continue_on_error:s.continue_on_error})}d.setState({isBatchModalOpen:!1});let n=e.activeProfile||void 0;try{let r=await m.batchCallCapabilities(a,n);d.setState({executionResult:{status:r.status,durationMs:r.durationMs,data:r.data}}),d.addEventLog("POST",`/v1/tools/batch_call (${t.length} steps)`,r.status===200?"200 OK":`HTTP ${r.status}`,`${r.durationMs.toFixed(1)}ms`)}catch(r){d.setState({executionResult:{status:500,durationMs:0,data:{error:r.toString()}}})}}async executeReadResource(){let e=d.getState(),t=e.selectedResourceId||(e.resources[0]?e.resources[0].id:null);if(!t)return;let a=document.getElementById("pg-res-jsonpath-input")?.value.trim()||void 0,n=document.getElementById("pg-res-lines-input")?.value.trim()||void 0,r=document.getElementById("pg-res-bytes-input")?.value.trim()||void 0,s={resource_id:t};if(a)s._jsonpath=a;if(n&&!isNaN(Number(n)))s._limit_lines=Number(n);if(r&&!isNaN(Number(r)))s._truncate_bytes=Number(r);let o=e.activeProfile||void 0;try{let i=await m.readResource({resource_id:t,input_responses:s},o);d.setState({resourceReadResult:{status:i.status,durationMs:i.durationMs,data:i.data}}),d.addEventLog("POST",`/v1/resources/read → ${t}`,i.status===200?"200 OK":`HTTP ${i.status}`,`${i.durationMs.toFixed(1)}ms`)}catch(i){d.setState({resourceReadResult:{status:500,durationMs:0,data:{error:i.toString()}}})}}async executeGetPrompt(){let e=d.getState(),t=e.selectedPromptId||(e.prompts[0]?e.prompts[0].id:null);if(!t)return;let a=document.querySelectorAll(".prompt-arg-input"),n={};a.forEach((s)=>{let o=s,i=o.getAttribute("data-arg-name");if(i&&o.value.trim())n[i]=o.value.trim()});let r=e.activeProfile||void 0;try{let s=await m.getPrompt({prompt_id:t,arguments:n},r);d.setState({promptGetResult:{status:s.status,durationMs:s.durationMs,data:s.data}}),d.addEventLog("POST",`/v1/prompts/get → ${t}`,s.status===200?"200 OK":`HTTP ${s.status}`,`${s.durationMs.toFixed(1)}ms`)}catch(s){d.setState({promptGetResult:{status:500,durationMs:0,data:{error:s.toString()}}})}}toggleBatchPlayground(){let e=document.getElementById("pg-args-input");if(!e)return;let t=[{id:"step_1",capability_id:"sqlite.read_query",args:{query:"SELECT * FROM users LIMIT 2"}},{id:"step_2",capability_id:"github.issues.search",args:{query:"label:bug"},continue_on_error:!0}];e.value=JSON.stringify(t,null,2)}async submitPolicyRule(e){let t=e==="allow"?"policy-new-allow":e==="deny"?"policy-new-deny":e==="redact"?"policy-new-redact":"policy-new-requireApproval",a=document.getElementById(t);if(!a)return;let n=a.value.trim();if(!n)return;await this.addPolicyRule(e,n),a.value=""}async addPolicyRule(e,t){let a=(t||"").trim();if(!a)return;let n=d.getState(),r=n.activeProfile,s=r?n.config.profiles?.[r]:void 0;if(s&&r){let o=s.policy||{},i=[...o.allow||[]],l=[...o.deny||[]],p=[...o.redact_keys||o.redactKeys||[]],u=[...o.require_approval||o.requireApproval||[]];if(e==="allow"&&!i.includes(a))i.push(a);if(e==="deny"&&!l.includes(a))l.push(a);if(e==="redact"&&!p.includes(a))p.push(a);if(e==="requireApproval"&&!u.includes(a))u.push(a);let g={...o,allow:i,deny:l,redactKeys:p,requireApproval:u},f=await m.upsertProfile(r,s.servers,s.description,g);if(!f.ok)alert(`Failed to save profile policy rule: ${f.error||"Unknown error"}`)}else{let o=n.config.policy||{},i=[...o.allow||[]],l=[...o.deny||[]],p=[...o.redact_keys||o.redactKeys||[]],u=[...o.require_approval||o.requireApproval||[]];if(e==="allow"&&!i.includes(a))i.push(a);if(e==="deny"&&!l.includes(a))l.push(a);if(e==="redact"&&!p.includes(a))p.push(a);if(e==="requireApproval"&&!u.includes(a))u.push(a);let g=await m.savePolicy({...o,allow:i,deny:l,redact_keys:p,redactKeys:p,require_approval:u,requireApproval:u});if(!g.ok)alert(`Failed to save policy rule: ${g.error||"Unknown error"}`)}await this.refreshData()}async removePolicyRule(e,t){let a=d.getState(),n=a.activeProfile,r=n?a.config.profiles?.[n]:void 0;if(r&&n){let s=r.policy||{},o=[...s.allow||[]],i=[...s.deny||[]],l=[...s.redact_keys||s.redactKeys||[]],p=[...s.require_approval||s.requireApproval||[]];if(e==="allow")o.splice(t,1);if(e==="deny")i.splice(t,1);if(e==="redact")l.splice(t,1);if(e==="requireApproval")p.splice(t,1);let u={...s,allow:o,deny:i,redactKeys:l,requireApproval:p},g=await m.upsertProfile(n,r.servers,r.description,u);if(!g.ok)alert(`Failed to update profile policy: ${g.error||"Unknown error"}`)}else{let s=a.config.policy||{},o=[...s.allow||[]],i=[...s.deny||[]],l=[...s.redact_keys||s.redactKeys||[]],p=[...s.require_approval||s.requireApproval||[]];if(e==="allow")o.splice(t,1);if(e==="deny")i.splice(t,1);if(e==="redact")l.splice(t,1);if(e==="requireApproval")p.splice(t,1);let u=await m.savePolicy({...s,allow:o,deny:i,redact_keys:l,redactKeys:l,require_approval:p,requireApproval:p});if(!u.ok)alert(`Failed to update policy: ${u.error||"Unknown error"}`)}await this.refreshData()}async saveWebhookConfig(){let e=document.getElementById("policy-webhook-url"),t=document.getElementById("policy-webhook-format"),a=document.getElementById("policy-webhook-secret"),n=e?e.value.trim():"",r=t?t.value:"generic",s=a?a.value.trim():"",i=d.getState().config.policy||{},l=n?{url:n,format:r,secret:s&&!s.startsWith("WARMPLANE_")&&!s.includes("_")?s:void 0,secret_env:s&&(s.startsWith("WARMPLANE_")||s.includes("_"))?s:void 0,events:["approval.requested","circuit_breaker.tripped","policy.violation"]}:void 0,p=await m.savePolicy({...i,webhook:l});if(p.ok)alert("Webhook settings saved successfully");else alert(`Failed to save webhook settings: ${p.error||"Unknown error"}`);await this.refreshData()}async testWebhook(){let e=document.getElementById("policy-webhook-url"),t=document.getElementById("policy-webhook-format"),a=e?e.value.trim():void 0,n=t?t.value:void 0,r=document.getElementById("policy-webhook-status");if(r)r.textContent="Sending test event...",r.style.color="var(--cyan-400)";try{let s=await m.testWebhook(a,n);if(s.ok){if(alert(`Test webhook sent successfully! (${s.message})`),r)r.textContent=`✔ Test sent (HTTP ${s.status_code||200})`,r.style.color="var(--green-400)"}else if(alert(`Test webhook failed: ${s.error||"Unknown error"}`),r)r.textContent=`✖ Failed: ${s.error}`,r.style.color="var(--red-400)"}catch(s){alert(`Error sending test webhook: ${s.message}`)}}testPolicySandbox(e){let t=document.getElementById("policy-test-verdict");if(!t)return;let a=e.trim();if(!a){t.textContent="ENTER ID",t.style.color="var(--text-dim)";return}let n=d.getState(),r=n.activeProfile,s=r?n.config.profiles?.[r]:void 0,o=n.config.policy||{},i=s?.policy,l=o.deny||[],p=i?.deny||[],u=Array.from(new Set([...l,...p])),g=o.require_approval||o.requireApproval||[],f=i?.require_approval||i?.requireApproval||[],c=Array.from(new Set([...g,...f])),y=i&&i.allow&&i.allow.length>0?i.allow:o.allow||[],h=(v,w)=>{if(v==="*")return!0;if(v.endsWith("*"))return w.startsWith(v.slice(0,-1));return v===w};if(u.some((v)=>h(v,a))){t.textContent="DENIED (Strict Block)",t.style.color="var(--red-400)";return}if(y.length>0&&!y.some((v)=>h(v,a))){t.textContent="DENIED (Not in Allow List)",t.style.color="var(--red-400)";return}if(c.some((v)=>h(v,a))){t.textContent="REQUIRE APPROVAL (HITL Gate)",t.style.color="var(--amber-400)";return}t.textContent="ALLOWED",t.style.color="var(--green-400)"}async deleteServer(e){if(!confirm(`Are you sure you want to remove server '${e}' from config?`))return;await m.deleteServer(e),await this.refreshData()}async restartServer(e){try{let t=await m.restartServer(e);if(t.ok)await this.refreshData();else alert(`Failed to restart server '${e}': ${t.error||"Unknown error"}`)}catch(t){alert(`Error restarting server '${e}': ${t.message}`)}}openServerDiagnosticsModal(e){this.closeModals();let t=d.getState(),a=t.config.mcpServers?.[e],n=t.serverStatuses?.[e],r=(t.circuitBreakers||[]).find((l)=>l.server_id===e),s=document.getElementById("modal-server-diagnostics");if(!s)return;let o=document.getElementById("modal-diag-title"),i=document.getElementById("modal-diag-body");if(o)o.textContent=`Live Diagnostics: ${e}`;if(i){let l=n?.status==="degraded",p=l?"var(--amber-400)":n?.status==="connected"?"var(--green-400)":"var(--red-400)",u=n?.error||"No active crash or error reported. Server is healthy.";i.innerHTML=`
        <div style="display: flex; gap: 10px; align-items: center; margin-bottom: 16px;">
          <span style="width: 10px; height: 10px; border-radius: 50%; background: ${p};"></span>
          <span style="font-weight: 700; font-size: 14px; color: var(--text-main);">Current Status: <span style="color: ${p}; text-transform: uppercase;">${b(n?.status||"unknown")}</span></span>
          <span class="brand-badge" style="color: var(--cyan-400);">Protocol: ${b(n?.protocol_version||"2026-07-28")}</span>
        </div>

        <div style="background: rgba(0,0,0,0.3); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 12px; margin-bottom: 14px;">
          <div style="font-size: 11px; font-weight: 700; color: var(--amber-400); text-transform: uppercase; margin-bottom: 6px;">
            ⚠️ Diagnostic Details / Failure Root Cause
          </div>
          <pre style="font-family: var(--ff-mono); font-size: 11.5px; color: ${l?"var(--red-300)":"var(--text-dim)"}; white-space: pre-wrap; word-break: break-word; margin: 0;">${b(u)}</pre>
        </div>

        <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 10px; margin-bottom: 14px;">
          <div style="background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 10px;">
            <div style="font-size: 10.5px; color: var(--text-dim);">Circuit Breaker State</div>
            <div style="font-weight: 700; font-size: 13px; color: var(--text-main); margin-top: 2px;">
              ${r?`${r.state.toUpperCase()} (${r.consecutive_failures} failures)`:"CLOSED (Healthy)"}
            </div>
          </div>
          <div style="background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 10px;">
            <div style="font-size: 10.5px; color: var(--text-dim);">Process Supervision</div>
            <div style="font-weight: 700; font-size: 13px; color: var(--text-main); margin-top: 2px;">
              Auto-Restart: ${a?.resilience?.autoRestart!==!1?"ENABLED":"DISABLED"}
            </div>
          </div>
        </div>

        <div style="background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 10px; margin-bottom: 16px;">
          <div style="font-size: 10.5px; color: var(--text-dim); margin-bottom: 4px;">Configured Execution Target</div>
          <code style="font-family: var(--ff-mono); font-size: 11px; color: var(--cyan-400); display: block; word-break: break-all;">
            ${a?.command?`${b(a.command)} ${b((a.args||[]).join(" "))}`:b(a?.url||"")}
          </code>
        </div>

        <div style="display: flex; justify-content: flex-end; gap: 8px;">
          <button class="btn btn-primary" onclick="window.app.restartServer('${b(e)}'); window.app.closeModals();">⚡ Restart &amp; Probe Now</button>
          <button class="btn btn-ghost" onclick="window.app.closeModals()">Close</button>
        </div>
      `}s.classList.add("active")}openAddServerModal(){this.closeModals();let e=document.getElementById("modal-srv-title"),t=document.getElementById("modal-srv-template-banner"),a=document.getElementById("modal-srv-name"),n=document.getElementById("modal-srv-transport"),r=document.getElementById("modal-srv-command"),s=document.getElementById("modal-srv-url"),o=document.getElementById("modal-srv-ft"),i=document.getElementById("modal-srv-cd"),l=document.getElementById("modal-srv-autorestart"),p=document.getElementById("modal-srv-maxrestarts");if(e)e.textContent="Add Upstream MCP Server";if(t)t.style.display="flex";if(a)a.value="",a.disabled=!1;if(n)n.value="stdio";if(r)r.value="";if(s)s.value="";let u=document.getElementById("modal-group-cmd"),g=document.getElementById("modal-group-url");if(u)u.style.display="block";if(g)g.style.display="none";if(o)o.value="3";if(i)i.value="30000";if(l)l.value="true";if(p)p.value="5";let f=document.getElementById("modal-add-server");if(f)f.classList.add("active")}openEditServerModal(e){this.closeModals();let t=d.getState(),a=t.config.mcpServers?.[e];if(!a){alert(`Server '${e}' not found in configuration.`);return}let n=document.getElementById("modal-srv-title"),r=document.getElementById("modal-srv-template-banner"),s=document.getElementById("modal-srv-name"),o=document.getElementById("modal-srv-transport"),i=document.getElementById("modal-srv-command"),l=document.getElementById("modal-srv-url"),p=document.getElementById("modal-srv-ft"),u=document.getElementById("modal-srv-cd"),g=document.getElementById("modal-srv-autorestart"),f=document.getElementById("modal-srv-maxrestarts");if(n)n.textContent=`Edit Server '${e}'`;if(r)r.style.display="none";if(s)s.value=e,s.disabled=!0;let c=!!a.command;if(o)o.value=c?"stdio":"http";let y=document.getElementById("modal-group-cmd"),h=document.getElementById("modal-group-url");if(y)y.style.display=c?"block":"none";if(h)h.style.display=c?"none":"block";if(i)i.value=c?`${a.command} ${(a.args||[]).join(" ")}`.trim():"";if(l)l.value=a.url||"";let v=a.resilience||t.config.resilience;if(p)p.value=String(v?.failureThreshold??3);if(u)u.value=String(v?.cooldownMs??30000);if(g)g.value=v?.autoRestart===!1?"false":"true";if(f)f.value=String(v?.maxRestarts??5);let w=document.getElementById("modal-add-server");if(w)w.classList.add("active")}async submitAddServer(){let e=document.getElementById("modal-srv-name"),t=e?.value.trim(),a=document.getElementById("modal-srv-transport")?.value;if(!t){alert("Server name is required");return}if(e&&!e.disabled){if((d.getState().config.mcpServers||{})[t]){if(!confirm(`Server '${t}' already exists in configuration. Do you want to overwrite it?`))return}}let n={};if(a==="stdio"){let u=(document.getElementById("modal-srv-command")?.value.trim()).split(/\s+/).filter(Boolean);if(u.length===0){alert("Command is required");return}n.command=u[0],n.args=u.slice(1)}else{let p=document.getElementById("modal-srv-url")?.value.trim();if(!p){alert("URL is required");return}n.url=p}let r=document.getElementById("modal-srv-ft")?.value.trim(),s=document.getElementById("modal-srv-cd")?.value.trim(),o=document.getElementById("modal-srv-autorestart")?.value,i=document.getElementById("modal-srv-maxrestarts")?.value.trim();if(r||s||o||i)n.resilience={failureThreshold:r?Number(r):3,cooldownMs:s?Number(s):30000,autoRestart:o!=="false",maxRestarts:i?Number(i):5};let l=await m.upsertServer(t,n);if(l.ok)this.closeModals(),await this.refreshData();else alert(`Failed to save server: ${l.error}`)}openTemplateCatalog(){this.closeModals();let e=document.getElementById("modal-templates");if(e)e.classList.add("active");this.renderTemplateGrid()}setTemplateCategory(e){this.activeTemplateCategory=e,document.querySelectorAll(".tmpl-cat-btn").forEach((t)=>{if(t.getAttribute("data-category")===e)t.classList.add("active"),t.style.background="var(--surface-elevated)",t.style.color="var(--amber-400)";else t.classList.remove("active"),t.style.background="var(--surface-card)",t.style.color="var(--text-main)"}),this.renderTemplateGrid()}filterTemplates(e){this.activeTemplateFilter=e.toLowerCase().trim(),this.renderTemplateGrid()}renderTemplateGrid(){let e=document.getElementById("tmpl-grid");if(!e)return;let t=X.filter((r)=>{let s=this.activeTemplateCategory==="all"||r.category===this.activeTemplateCategory,o=!this.activeTemplateFilter||r.name.toLowerCase().includes(this.activeTemplateFilter)||r.id.toLowerCase().includes(this.activeTemplateFilter)||r.description.toLowerCase().includes(this.activeTemplateFilter)||r.command.toLowerCase().includes(this.activeTemplateFilter)||r.envFields.some((i)=>i.key.toLowerCase().includes(this.activeTemplateFilter));return s&&o});if(t.length===0){e.innerHTML=`
        <div style="grid-column: span 2; padding: 32px; text-align: center; color: var(--text-dim);">
          No matching MCP server templates found.
        </div>
      `;return}let n=d.getState().config.mcpServers||{};e.innerHTML=t.map((r)=>{let s=!!n[r.id],o=`${r.command} ${r.defaultArgs.join(" ")}`;return`
        <div class="bento-card" style="display: flex; flex-direction: column; justify-content: space-between; padding: 14px; background: var(--surface); border: 1px solid var(--border); min-width: 0; transition: transform 0.15s, border-color 0.15s;">
          <div>
            <div style="display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 6px;">
              <div style="display: flex; align-items: center; gap: 8px; min-width: 0;">
                <span style="font-weight: 700; font-size: 13.5px; color: var(--text-main); white-space: nowrap; overflow: hidden; text-overflow: ellipsis;">${b(r.name)}</span>
                <span class="brand-badge" style="font-size: 9.5px; padding: 1px 6px; flex-shrink: 0;">${b(r.badge)}</span>
              </div>
              ${s?'<span style="font-size: 10px; color: var(--green-400); font-weight: 600; flex-shrink: 0;">CONNECTED</span>':""}
            </div>
            <div style="font-size: 11.5px; color: var(--text-muted); line-height: 1.4; margin-bottom: 8px;">
              ${b(r.description)}
            </div>
            <div style="font-family: var(--ff-mono); font-size: 10.5px; color: var(--text-dim); background: var(--surface-card); padding: 5px 8px; border-radius: var(--radius-sm); border: 1px solid var(--border-subtle); overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">
              <code>${b(o)}</code>
            </div>
            ${r.envFields.length>0?`
              <div style="font-size: 10.5px; color: var(--amber-400); margin-top: 6px; display: flex; align-items: center; gap: 4px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">
                <span>⚡ Needs:</span>
                <code style="overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">${r.envFields.map((i)=>b(i.key)).join(", ")}</code>
              </div>
            `:""}
          </div>

          <div style="display: flex; justify-content: flex-end; margin-top: 12px; gap: 6px;">
            <button class="btn btn-primary" style="font-size: 11.5px; padding: 4px 10px;" onclick="window.app.selectTemplate('${b(r.id)}')">
              ${s?"Configure Another":"✨ 1-Click Setup"}
            </button>
          </div>
        </div>
      `}).join("")}selectTemplate(e){let t=X.find((l)=>l.id===e);if(!t)return;this.selectedTemplate=t,this.closeModals();let a=document.getElementById("modal-configure-template");if(a)a.classList.add("active");let n=document.getElementById("cfg-tmpl-title"),r=document.getElementById("cfg-tmpl-desc"),s=document.getElementById("cfg-tmpl-form");if(n)n.textContent=`Configure ${t.name} Server`;if(r)r.textContent=t.description;let o=d.getState().config.mcpServers||{},i=t.id;if(o[i]){let l=2;while(o[`${t.id}-${l}`])l++;i=`${t.id}-${l}`}if(s){let l="";if(t.envFields.length>0)l=`
          <div style="margin-top: 14px; margin-bottom: 6px; font-weight: 700; font-size: 11px; text-transform: uppercase; color: var(--amber-400); letter-spacing: 0.5px;">
            Environment Variables &amp; API Keys
          </div>
          ${t.envFields.map((p)=>`
            <div class="form-group">
              <label class="form-label">${b(p.label)} ${p.required?'<span style="color: var(--red-400);">*</span>':"(Optional)"}</label>
              <input type="password" class="form-input tmpl-env-input" data-key="${b(p.key)}" placeholder="${b(p.placeholder||"")}">
              ${p.description?`<div style="font-size: 10.5px; color: var(--text-dim); margin-top: 3px;">${b(p.description)}</div>`:""}
            </div>
          `).join("")}
        `;s.innerHTML=`
        <div class="form-group">
          <label class="form-label">Server Identifier (Name)</label>
          <input type="text" class="form-input" id="cfg-srv-id" value="${b(i)}">
          <div style="font-size: 10.5px; color: var(--text-dim); margin-top: 3px;">Must be unique across all configured servers.</div>
        </div>
        <div class="form-group">
          <label class="form-label">Command Line Arguments</label>
          <input type="text" class="form-input" id="cfg-srv-args" value="${b(t.defaultArgs.join(" "))}" placeholder="${b(t.argsPlaceholder||"")}">
          <div style="font-size: 10.5px; color: var(--text-dim); margin-top: 3px;">Executable: <code>${b(t.command)}</code></div>
        </div>
        ${l}
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
      `}}async submitTemplateServer(){if(!this.selectedTemplate)return;let e=this.selectedTemplate,t=document.getElementById("cfg-srv-id")?.value.trim(),a=document.getElementById("cfg-srv-args")?.value.trim();if(!t){alert("Server identifier is required");return}if((d.getState().config.mcpServers||{})[t]){if(!confirm(`Server '${t}' already exists. Do you want to overwrite its configuration?`))return}let s=a?a.split(/\s+/).filter(Boolean):[],o={},i=document.querySelectorAll(".tmpl-env-input");for(let y of Array.from(i)){let h=y.getAttribute("data-key"),v=y.value.trim(),w=e.envFields.find((C)=>C.key===h);if(w?.required&&!v){alert(`Required field '${w.label}' is missing.`);return}if(h&&v)o[h]=v}let l={command:e.command,args:s};if(Object.keys(o).length>0)l.env=o;let p=document.getElementById("cfg-srv-ft")?.value.trim(),u=document.getElementById("cfg-srv-cd")?.value.trim(),g=document.getElementById("cfg-srv-autorestart")?.value,f=document.getElementById("cfg-srv-maxrestarts")?.value.trim();if(p||u||g||f)l.resilience={failureThreshold:p?Number(p):3,cooldownMs:u?Number(u):30000,autoRestart:g!=="false",maxRestarts:f?Number(f):5};let c=await m.upsertServer(t,l);if(c.ok)this.closeModals(),await this.refreshData();else alert(`Failed to save server: ${c.error}`)}async openImportModal(){this.closeModals();let e=document.getElementById("modal-import");if(e)e.classList.add("active");let t=document.getElementById("modal-eco-list");if(!t)return;t.innerHTML='<div style="color: var(--text-dim); padding: 12px; text-align: center;">Scanning IDE configs...</div>';try{let a=await m.getEcosystemSources();if(a.sources&&a.sources.length>0)t.innerHTML=a.sources.map((n)=>`
          <label style="display: flex; align-items: center; gap: 10px; background: var(--surface); padding: 10px 14px; border-radius: var(--radius-sm); border: 1px solid var(--border); cursor: pointer;">
            <input type="checkbox" class="eco-checkbox" value="${n.path}" checked>
            <div>
              <div style="font-weight: 600; color: var(--text-main);">${n.name}</div>
              <div style="font-size: 11px; color: var(--text-dim);">${n.server_count} servers (${n.servers.join(", ")})</div>
            </div>
          </label>
        `).join("");else t.innerHTML='<div style="color: var(--text-dim); padding: 12px; text-align: center;">No external MCP configuration files found on this system.</div>'}catch{t.innerHTML='<div style="color: var(--red-400); padding: 12px; text-align: center;">Failed to scan ecosystem sources.</div>'}}async submitImport(){let e=document.querySelectorAll(".eco-checkbox:checked");if(e.length===0){alert("No sources selected");return}for(let t of Array.from(e))await m.importConfig(t.value,!1);this.closeModals(),await this.refreshData()}async refreshClients(){try{let e=await m.getClients();if(e.ok&&Array.isArray(e.clients))d.setState({clients:e.clients})}catch(e){console.error("Failed to scan clients:",e)}}async attachClient(e,t){let a=t;if(!a){let r=document.getElementById(`client-prof-${e}`)||document.getElementById(`overview-client-prof-${e}`);if(r)a=r.value||void 0;else a=d.getState().activeProfile||void 0}let n=await m.attachClient(e,a);if(!n.ok)alert(`Failed to attach client: ${n.error||n.message||"Unknown error"}`);else await this.refreshData()}async detachClient(e){if(!confirm("Disconnect Warmplane from this client?"))return;let t=await m.detachClient(e);if(!t.ok)alert(`Failed to detach client: ${t.error||t.message||"Unknown error"}`);else await this.refreshData()}handleAliasTargetInput(e){let t=document.getElementById("alias-suggestions-dropdown");if(!t)return;let a=(e||"").trim().toLowerCase();if(a.length<2){t.style.display="none";return}let r=d.getState().capabilities.filter((s)=>s.id.toLowerCase().includes(a)||s.summary&&s.summary.toLowerCase().includes(a)||s.description&&s.description.toLowerCase().includes(a)||s.server&&s.server.toLowerCase().includes(a)).slice(0,8);if(r.length===0){t.style.display="none";return}t.innerHTML=r.map((s)=>`
      <div style="padding: 8px 12px; cursor: pointer; border-bottom: 1px solid var(--border-subtle); display: flex; justify-content: space-between; align-items: center; transition: background 0.1s;"
           onmouseover="this.style.background='var(--surface-hover)'"
           onmouseout="this.style.background='transparent'"
           onmousedown="window.app.selectAliasSuggestion('${b(s.id)}')">
        <div>
          <div style="font-weight: 700; color: var(--text-main);">${b(s.id)}</div>
          <div style="font-size: 10.5px; color: var(--text-dim); margin-top: 2px;">${b(s.summary||s.description||"")}</div>
        </div>
        <span style="font-size: 10px; color: var(--cyan-400);">${b(s.server||"local")}</span>
      </div>
    `).join(""),t.style.display="block"}selectAliasSuggestion(e){let t=document.getElementById("alias-target");if(t)t.value=e;this.hideAliasDropdown()}hideAliasDropdown(){let e=document.getElementById("alias-suggestions-dropdown");if(e)e.style.display="none"}async createAlias(){let e=document.getElementById("alias-kind")?.value,t=document.getElementById("alias-name")?.value.trim(),a=document.getElementById("alias-target")?.value.trim();if(!t||!a){alert("Please provide both alias name and canonical target");return}await m.updateAlias(e,t,a),await this.refreshData()}async deleteAlias(e,t){await m.updateAlias(e,t,void 0),await this.refreshData()}async reloadFromDisk(){try{let e=await m.reloadConfig();if(e.ok){let t="Hot-reload completed successfully!";if(e.mounted&&e.mounted.length>0)t+=`
Mounted: ${e.mounted.join(", ")}`;if(e.unmounted&&e.unmounted.length>0)t+=`
Unmounted: ${e.unmounted.join(", ")}`;if(e.warnings&&e.warnings.length>0)t+=`
Warnings:
${e.warnings.join(`
`)}`;alert(t)}else alert(`Hot-reload failed: ${e.error||"Unknown error"}`)}catch(e){alert(`Error reaching daemon: ${e.message}`)}await this.refreshData()}renderTopProfileSelector(){let e=document.getElementById("top-profile-selector");if(!e)return;let t=d.getState(),a=t.config.profiles||{},n=Object.keys(a),r=t.activeProfile,s='<option value="">All Servers (Unrestricted)</option>';for(let o of n){let i=r===o?"selected":"";s+=`<option value="${b(o)}" ${i}>Profile: ${b(o)}</option>`}e.innerHTML=s}async setActiveProfile(e){d.setState({activeProfile:e||null}),await this.refreshData()}openAddProfileModal(){let e=document.getElementById("modal-prof-title");if(e)e.textContent="Create Server Constellation Profile";let t=document.getElementById("modal-prof-name"),a=document.getElementById("modal-prof-desc"),n=document.getElementById("modal-prof-mode");if(t)t.value="",t.disabled=!1;if(a)a.value="";if(n)n.value="create";let r=document.getElementById("modal-prof-allow"),s=document.getElementById("modal-prof-deny"),o=document.getElementById("modal-prof-hitl"),i=document.getElementById("modal-prof-redact");if(r)r.value="";if(s)s.value="";if(o)o.value="";if(i)i.value="";this.renderProfileServerCheckboxes([]);let l=document.getElementById("modal-add-profile");if(l)l.classList.add("active")}openEditProfileModal(e){let a=d.getState().config.profiles?.[e];if(!a)return;let n=document.getElementById("modal-prof-title");if(n)n.textContent=`Edit Profile: ${e}`;let r=document.getElementById("modal-prof-name"),s=document.getElementById("modal-prof-desc"),o=document.getElementById("modal-prof-mode");if(r)r.value=e,r.disabled=!0;if(s)s.value=a.description||"";if(o)o.value="edit";let i=document.getElementById("modal-prof-allow"),l=document.getElementById("modal-prof-deny"),p=document.getElementById("modal-prof-hitl"),u=document.getElementById("modal-prof-redact"),g=a.policy;if(i)i.value=(g?.allow||[]).join(", ");if(l)l.value=(g?.deny||[]).join(", ");if(p)p.value=(g?.require_approval||g?.requireApproval||[]).join(", ");if(u)u.value=(g?.redact_keys||g?.redactKeys||[]).join(", ");this.renderProfileServerCheckboxes(a.servers||[]);let f=document.getElementById("modal-add-profile");if(f)f.classList.add("active")}renderProfileServerCheckboxes(e){let t=document.getElementById("modal-prof-servers-list");if(!t)return;let a=d.getState(),n=Object.keys(a.config.mcpServers||{});if(n.length===0){t.innerHTML='<div style="font-size: 11.5px; color: var(--text-dim);">No MCP servers configured yet. Add servers first.</div>';return}t.innerHTML=n.map((r)=>{let s=e.includes(r)?"checked":"";return`
        <label style="display: flex; align-items: center; gap: 8px; font-size: 12px; cursor: pointer; padding: 4px 6px; border-radius: var(--radius-sm); transition: background 0.15s;" onmouseover="this.style.background='var(--surface-hover)'" onmouseout="this.style.background='transparent'">
          <input type="checkbox" class="prof-server-checkbox" value="${b(r)}" ${s} style="accent-color: var(--amber-400);">
          <span style="font-family: var(--ff-mono); font-weight: 600; color: var(--text-main);">${b(r)}</span>
        </label>
      `}).join("")}async saveProfile(){let e=document.getElementById("modal-prof-name"),t=document.getElementById("modal-prof-desc"),a=e?.value.trim(),n=t?.value.trim();if(!a){alert("Please enter a profile name");return}let r=document.querySelectorAll(".prof-server-checkbox:checked"),s=[];if(r.forEach((v)=>{s.push(v.value)}),s.length===0){alert("Please select at least one server to include in this constellation");return}let o=(v)=>{if(!v)return[];return v.split(",").map((w)=>w.trim()).filter((w)=>w.length>0)},i=document.getElementById("modal-prof-allow"),l=document.getElementById("modal-prof-deny"),p=document.getElementById("modal-prof-hitl"),u=document.getElementById("modal-prof-redact"),g=o(i?.value),f=o(l?.value),c=o(p?.value),y=o(u?.value),h=void 0;if(g.length>0||f.length>0||c.length>0||y.length>0)h={allow:g,deny:f,requireApproval:c,redactKeys:y};try{let v=await m.upsertProfile(a,s,n||void 0,h);if(v.ok)this.closeModals(),await this.refreshData();else alert(`Failed to save profile: ${v.error||"Unknown error"}`)}catch(v){alert(`Error saving profile: ${v.message}`)}}async deleteProfile(e){if(!confirm(`Are you sure you want to delete profile '${e}'?`))return;try{let t=await m.deleteProfile(e);if(t.ok){if(d.getState().activeProfile===e)d.setState({activeProfile:null});await this.refreshData()}else alert(`Failed to delete profile: ${t.error||"Unknown error"}`)}catch(t){alert(`Error deleting profile: ${t.message}`)}}closeModals(){document.querySelectorAll(".modal-backdrop").forEach((e)=>e.classList.remove("active"))}}function b(e){return String(e||"").replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;")}var pe=new ce;window.app=pe;window.addEventListener("DOMContentLoaded",()=>pe.init());
