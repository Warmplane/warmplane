class Z{state={configPath:"mcp_servers.json",config:{mcpServers:{}},serverStatuses:{},circuitBreakers:[],clients:[],secrets:[],clientsCollapsed:!1,capabilities:[],capabilitiesHiddenByPolicy:0,resources:[],resourcesHiddenByPolicy:0,prompts:[],promptsHiddenByPolicy:0,catalogEvents:[],tasks:[],selectedTaskId:null,taskFilterStatus:"all",approvals:[],auditEvents:[],auditTotal:0,auditFilters:{search:"",status:"all",eventType:"all",serverId:"all",limit:25,offset:0},auditSelectedEvent:null,auditStats:null,auditVerification:null,selectedCapabilityId:null,selectedResourceId:null,selectedPromptId:null,playgroundMode:"tools",playgroundArgs:{},isExecuting:!1,playgroundAsyncTask:!1,activeRequestId:null,isBatchModalOpen:!1,batchSteps:[{id:"step_1",capability_id:"",argsJson:"{}",continue_on_error:!1},{id:"step_2",capability_id:"",argsJson:"{}",continue_on_error:!0}],activeTab:"overview",activeProfile:null,eventLogs:[],executionResult:null,resourceReadResult:null,promptGetResult:null,metrics:{totalCatalogRequests:0,totalEtagHits:0,totalToolCalls:0,totalToolDurationUs:0}};listeners=[];getState(){return this.state}setState(e){this.state={...this.state,...e},this.listeners.forEach((t)=>t(this.state))}subscribe(e){return this.listeners.push(e),()=>{this.listeners=this.listeners.filter((t)=>t!==e)}}addEventLog(e,t,r,n){let s=[{time:new Date().toLocaleTimeString(),method:e,target:t,status:r,latency:n},...this.state.eventLogs].slice(0,50);this.setState({eventLogs:s})}}var d=new Z;class ee{baseUrl;constructor(e=""){this.baseUrl=e}async getConfig(){return(await fetch(`${this.baseUrl}/v1/config`)).json()}async listCapabilities(e){let t={};if(e)t["X-Warmplane-Profile"]=e;return(await fetch(`${this.baseUrl}/v1/capabilities`,{headers:t})).json()}async listResources(e){let t={};if(e)t["X-Warmplane-Profile"]=e;return(await fetch(`${this.baseUrl}/v1/resources`,{headers:t})).json()}async readResource(e,t){let r=performance.now(),n={"Content-Type":"application/json"};if(t)n["X-Warmplane-Profile"]=t;let a=await fetch(`${this.baseUrl}/v1/resources/read`,{method:"POST",headers:n,body:JSON.stringify(e)}),s=performance.now()-r,o=await a.json();return{status:a.status,durationMs:s,data:o}}async listPrompts(e){let t={};if(e)t["X-Warmplane-Profile"]=e;return(await fetch(`${this.baseUrl}/v1/prompts`,{headers:t})).json()}async getPrompt(e,t){let r=performance.now(),n={"Content-Type":"application/json"};if(t)n["X-Warmplane-Profile"]=t;let a=await fetch(`${this.baseUrl}/v1/prompts/get`,{method:"POST",headers:n,body:JSON.stringify(e)}),s=performance.now()-r,o=await a.json();return{status:a.status,durationMs:s,data:o}}async getCatalogEvents(e){let t=e?`?after=${encodeURIComponent(e)}`:"";return(await fetch(`${this.baseUrl}/v1/catalog/events${t}`)).json()}async callCapability(e,t){let r=performance.now(),n={"Content-Type":"application/json"};if(t)n["X-Warmplane-Profile"]=t;let a=await fetch(`${this.baseUrl}/v1/tools/call`,{method:"POST",headers:n,body:JSON.stringify(e)}),s=performance.now()-r,o=await a.json();return{status:a.status,durationMs:s,data:o}}async batchCallCapabilities(e,t){let r=performance.now(),n={"Content-Type":"application/json"};if(t)n["X-Warmplane-Profile"]=t;let a=await fetch(`${this.baseUrl}/v1/tools/batch_call`,{method:"POST",headers:n,body:JSON.stringify({steps:e})}),s=performance.now()-r,o=await a.json();return{status:a.status,durationMs:s,data:o}}async cancelOperation(e){return(await fetch(`${this.baseUrl}/v1/operations/${encodeURIComponent(e)}/cancel`,{method:"POST"})).json()}async completeArgument(e){return(await fetch(`${this.baseUrl}/v1/completion/complete`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify(e)})).json()}async upsertServer(e,t){return(await fetch(`${this.baseUrl}/v1/config/servers`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({name:e,server:t})})).json()}async deleteServer(e){return(await fetch(`${this.baseUrl}/v1/config/servers/${encodeURIComponent(e)}`,{method:"DELETE"})).json()}async restartServer(e){return(await fetch(`${this.baseUrl}/v1/config/servers/${encodeURIComponent(e)}/restart`,{method:"POST"})).json()}async upsertProfile(e,t,r,n){return(await fetch(`${this.baseUrl}/v1/config/profiles`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({name:e,servers:t,description:r,policy:n})})).json()}async deleteProfile(e){return(await fetch(`${this.baseUrl}/v1/config/profiles/${encodeURIComponent(e)}`,{method:"DELETE"})).json()}async getEcosystemSources(){return(await fetch(`${this.baseUrl}/v1/config/ecosystem`)).json()}async importConfig(e,t=!1){return(await fetch(`${this.baseUrl}/v1/config/import`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({source_path:e,overwrite:t})})).json()}async savePolicy(e){let t={allow:e.allow||[],deny:e.deny||[],redactKeys:e.redact_keys||e.redactKeys||[],requireApproval:e.require_approval||e.requireApproval||[],approvalTimeoutSecs:e.approvalTimeoutSecs||e.approval_timeout_secs||300,webhook:e.webhook};return(await fetch(`${this.baseUrl}/v1/config/policy`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify(t)})).json()}async listTasks(){return(await fetch(`${this.baseUrl}/v1/tasks`)).json()}async getTask(e){return(await fetch(`${this.baseUrl}/v1/tasks/${encodeURIComponent(e)}`)).json()}async updateTask(e,t){return(await fetch(`${this.baseUrl}/v1/tasks/${encodeURIComponent(e)}/update`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({inputResponses:t})})).json()}async cancelTask(e,t){return(await fetch(`${this.baseUrl}/v1/tasks/${encodeURIComponent(e)}/cancel`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({reason:t})})).json()}async listApprovals(){return(await fetch(`${this.baseUrl}/v1/approvals`)).json()}async approveTicket(e,t,r){return(await fetch(`${this.baseUrl}/v1/approvals/${encodeURIComponent(e)}/approve`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({operator:t,modified_args:r})})).json()}async rejectTicket(e,t,r){return(await fetch(`${this.baseUrl}/v1/approvals/${encodeURIComponent(e)}/reject`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({operator:t,reason:r})})).json()}async updateAlias(e,t,r){return(await fetch(`${this.baseUrl}/v1/config/alias`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({kind:e,alias:t,target:r})})).json()}async reloadConfig(){return(await fetch(`${this.baseUrl}/v1/config/reload`,{method:"POST"})).json()}async listAuditEvents(e){let t=new URLSearchParams;if(e?.actor_id)t.set("actor_id",e.actor_id);if(e?.server_id&&e.server_id!=="all")t.set("server_id",e.server_id);if(e?.capability_id)t.set("capability_id",e.capability_id);if(e?.event_type&&e.event_type!=="all")t.set("event_type",e.event_type);if(e?.status&&e.status!=="all")t.set("status",e.status);if(e?.trace_id)t.set("trace_id",e.trace_id);if(e?.request_id)t.set("request_id",e.request_id);if(e?.search)t.set("search",e.search);if(e?.limit)t.set("limit",String(e.limit));if(e?.offset!==void 0)t.set("offset",String(e.offset));let r=t.toString();return(await fetch(`${this.baseUrl}/v1/audit/events${r?`?${r}`:""}`)).json()}getAuditExportUrl(e,t="csv"){let r=new URLSearchParams;if(r.set("format",t),e?.actor_id)r.set("actor_id",e.actor_id);if(e?.server_id&&e.server_id!=="all")r.set("server_id",e.server_id);if(e?.capability_id)r.set("capability_id",e.capability_id);if(e?.event_type&&e.event_type!=="all")r.set("event_type",e.event_type);if(e?.status&&e.status!=="all")r.set("status",e.status);if(e?.trace_id)r.set("trace_id",e.trace_id);if(e?.request_id)r.set("request_id",e.request_id);if(e?.search)r.set("search",e.search);return`${this.baseUrl}/v1/audit/export?${r.toString()}`}async verifyAuditChain(){return(await fetch(`${this.baseUrl}/v1/audit/verify`)).json()}async getAuditStats(){return(await fetch(`${this.baseUrl}/v1/audit/stats`)).json()}async getClients(){return(await fetch(`${this.baseUrl}/v1/clients`)).json()}async attachClient(e,t){return(await fetch(`${this.baseUrl}/v1/clients/${encodeURIComponent(e)}/attach`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({profile:t||void 0})})).json()}async detachClient(e){return(await fetch(`${this.baseUrl}/v1/clients/${encodeURIComponent(e)}/detach`,{method:"POST",headers:{"Content-Type":"application/json"}})).json()}async testWebhook(e,t){return(await fetch(`${this.baseUrl}/v1/webhooks/test`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({url:e||void 0,format:t||void 0})})).json()}async getSecrets(){return(await fetch(`${this.baseUrl}/v1/secrets`)).json()}async saveSecret(e,t,r){return(await fetch(`${this.baseUrl}/v1/secrets`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({key:e,value:t,service:r})})).json()}async deleteSecret(e){return(await fetch(`${this.baseUrl}/v1/secrets/${encodeURIComponent(e)}`,{method:"DELETE"})).json()}}var f=new ee;function te(){let e=d.getState(),t=e.config.mcpServers||{},r=Object.keys(t),n=r.length,a="";if(r.length===0)a=`
      <div style="grid-column: 1 / -1; padding: 32px; text-align: center; color: var(--text-dim); background: var(--surface-card); border-radius: var(--radius-md); border: 1px dashed var(--border);">
        <div style="font-size: 14px; color: var(--text-main); font-weight: 600; margin-bottom: 6px;">No Upstream MCP Servers Connected</div>
        <div style="font-size: 12px; margin-bottom: 16px;">Initialize connections by adding a server or syncing existing IDE configurations.</div>
        <div style="display: flex; gap: 8px; justify-content: center;">
          <button class="btn btn-primary" onclick="window.app.openTemplateCatalog()">✨ Browse Templates</button>
          <button class="btn btn-ghost" onclick="window.app.openAddServerModal()">+ Add Custom</button>
          <button class="btn btn-ghost" onclick="window.app.openImportModal()">Sync from IDEs</button>
        </div>
      </div>
    `;else a=r.map((E)=>{let O=t[E],J=O.command?"stdio":"http / sse",K=O.command?`${O.command} ${(O.args||[]).join(" ")}`:O.url,j=e.serverStatuses[E]||{status:"connected",protocol_version:"2026-07-28"},F=j.status==="degraded",Y=j.status==="error"||j.status==="disconnected",V=F?"var(--amber-400)":Y?"var(--red-400)":"var(--green-400)";return`
        <div class="bento-card col-4" style="background: var(--surface); border: 1px solid var(--border);">
          <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
            <span style="font-weight: 700; color: var(--text-main); display: flex; align-items: center; gap: 6px;">
              <span style="width: 8px; height: 8px; border-radius: 50%; background: ${V}; display: inline-block;"></span>
              ${R(E)}
            </span>
            <span class="brand-badge">${J}</span>
          </div>
          <div style="font-family: var(--ff-mono); font-size: 11.5px; color: var(--text-dim); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; margin-bottom: 12px;" title="${R(K||"")}">
            ${R(K||"")}
          </div>
          <div style="display: flex; justify-content: space-between; font-size: 11px; color: var(--text-muted); border-top: 1px solid var(--border); padding-top: 8px;">
            <span>Status: <strong style="color: ${V};">${R(j.status)}</strong></span>
            <span>Protocol: ${j.protocol_version}</span>
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
      <span style="color: var(--text-dim);">${R(E.time)}</span>
      <span style="color: var(--cyan-400); font-weight: 600;">${R(E.method)}</span>
      <span style="color: var(--text-main); font-family: var(--ff-mono);">${R(E.target)}</span>
      <span style="color: var(--green-400);">${R(E.status)}</span>
      <span style="color: var(--amber-300); text-align: right;">${R(E.latency)}</span>
    </div>
  `).join(""),o=e.metrics,i=o.totalCatalogRequests,l=o.totalEtagHits,c=i>0?`${(l/i*100).toFixed(1)}%`:"0.0%",u=i>0?`${l} of ${i} requests served via HTTP 304`:"Waiting for client requests",g=o.totalToolCalls,b=g>0?`${(o.totalToolDurationUs/g/1000).toFixed(1)}ms`:"0.0ms",p=g>0?`${g} tool executions processed`:"Local worker task queues warm",m=Object.keys(e.config.capabilityAliases||{}).length+Object.keys(e.config.resourceAliases||{}).length+Object.keys(e.config.promptAliases||{}).length,y=m>0?`${m*18}B / call`:"0B",v=m>0?`${m} active facade aliases pruning prompt size`:"Configure aliases in Studio to reduce prompt size",x=e.tasks||[],C=x.filter((E)=>E.status==="input_required").length,P=x.filter((E)=>E.status==="working"||E.status==="input_required").length,_=e.clients||[],A=_.filter((E)=>E.is_attached).length,w=_.filter((E)=>E.config_exists&&!E.is_attached).length,M=e.clientsCollapsed,B=A>0?`<span class="brand-badge" style="color: var(--green-400); border-color: rgba(52, 211, 153, 0.3); background: rgba(52, 211, 153, 0.1);">⚡ ${A} Connected</span>`:w>0?`<span class="brand-badge" style="color: var(--amber-300); border-color: rgba(251, 191, 36, 0.3); background: rgba(251, 191, 36, 0.1);">○ ${w} Ready to Connect</span>`:'<span class="brand-badge" style="color: var(--text-dim);">No Apps Detected</span>',U=Object.keys(e.config.profiles||{}),L=e.activeProfile,q=_.map((E)=>{let{is_attached:O,config_exists:J,app_installed:K}=E,j="rgba(255, 255, 255, 0.2)",F="Not Found";if(O)j="var(--green-400)",F=E.attached_profile?`Connected (${E.attached_profile})`:"Connected (All Tools)";else if(J)j="var(--amber-300)",F="Ready to Attach";else if(K)j="var(--cyan-400)",F="Installed";let Y=U.map((W)=>`
      <option value="${R(W)}" ${L===W||E.attached_profile===W?"selected":""}>${R(W)}</option>
    `).join(""),V=O?`<button class="btn btn-ghost" style="padding: 2px 7px; font-size: 10px; color: var(--red-400);" onclick="event.stopPropagation(); window.app.detachClient('${R(E.id)}')">Detach</button>`:J||K?`
        <div style="display: flex; align-items: center; gap: 4px;" onclick="event.stopPropagation();">
          ${U.length>0?`
            <select id="overview-client-prof-${R(E.id)}" class="form-input" style="font-size: 10px; padding: 1px 4px; height: 22px; width: 85px;" title="Select constellation profile">
              <option value="" ${!L?"selected":""}>All Tools</option>
              ${Y}
            </select>
          `:""}
          <button class="btn btn-primary" style="padding: 2px 7px; font-size: 10px;" onclick="window.app.attachClient('${R(E.id)}')">⚡ Connect</button>
        </div>
      `:"";return`
      <div style="background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 8px 12px; display: flex; align-items: center; justify-content: space-between; gap: 8px;">
        <div style="display: flex; align-items: center; gap: 8px; overflow: hidden;">
          <span style="width: 7px; height: 7px; border-radius: 50%; background: ${j}; flex-shrink: 0;"></span>
          <div style="overflow: hidden;">
            <div style="font-weight: 600; font-size: 12px; color: var(--text-main); white-space: nowrap; overflow: hidden; text-overflow: ellipsis;">${R(E.name)}</div>
            <div style="font-size: 10px; color: var(--text-dim);">${R(F)}</div>
          </div>
        </div>
        ${V}
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
          ${q}
        </div>
      `:""}
    </div>
  `;return`
    <div class="bento-grid">
      <div class="bento-card col-3">
        <div class="stat-label">Token Savings Rate</div>
        <div class="stat-value" style="color: var(--amber-300);">${y}</div>
        <div class="stat-sub">${v}</div>
      </div>
      <div class="bento-card col-3">
        <div class="stat-label">ETag Cache Hit Rate</div>
        <div class="stat-value" style="color: var(--cyan-400);">${c}</div>
        <div class="stat-sub">${u}</div>
      </div>
      <div class="bento-card col-3">
        <div class="stat-label">Tasks &amp; HITL State</div>
        <div class="stat-value" style="color: ${C>0?"var(--amber-400)":"var(--green-400)"};">${C>0?`${C} Action Req`:`${P} Active`}</div>
        <div class="stat-sub">${C>0?"Awaiting Human-in-the-Loop decision":`${x.length} total registered tasks`}</div>
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
        ${s}
      </div>
    </div>
  `}function R(e){return String(e||"").replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;")}function se(){let e=d.getState(),t=e.config.mcpServers||{},r=Object.keys(t),n=e.activeProfile,a=n?e.config.profiles?.[n]:void 0,s=!!a,o=a?.servers||[],i="";if(r.length===0)i=`
      <div style="padding: 40px; text-align: center; color: var(--text-dim); background: var(--surface-card); border-radius: var(--radius-md); border: 1px dashed var(--border);">
        <div style="font-size: 15px; color: var(--text-main); font-weight: 600; margin-bottom: 8px;">No Servers Configured in ${T(e.configPath)}</div>
        <p style="font-size: 12px; margin-bottom: 20px; max-width: 480px; margin-left: auto; margin-right: auto;">
          Warmplane bridges local tools and remote MCP servers into one unified facade. Add your first server or import existing configs from Claude Desktop or Cursor.
        </p>
        <div style="display: flex; gap: 8px; justify-content: center;">
          <button class="btn btn-primary" onclick="window.app.openTemplateCatalog()">✨ Browse Templates</button>
          <button class="btn btn-ghost" onclick="window.app.openAddServerModal()">+ Add Custom</button>
          <button class="btn btn-ghost" onclick="window.app.openImportModal()">Sync from IDEs</button>
        </div>
      </div>
    `;else i=r.map((c)=>{let u=t[c],g=u.command?"stdio":"http / sse",b=u.command?`${u.command} ${(u.args||[]).join(" ")}`:u.url,p=e.serverStatuses[c]||{status:"connected",protocol_version:"2026-07-28"},m=!s||o.includes(c),y=u.env?Object.entries(u.env).map(([L,q])=>{if(q.startsWith("keychain://"))return`<span class="brand-badge" style="color: var(--cyan-400); border-color: rgba(34, 211, 238, 0.3);">\uD83D\uDD12 ${T(L)} (Keychain)</span>`;if(q.startsWith("op://"))return`<span class="brand-badge" style="color: var(--cyan-400); border-color: rgba(34, 211, 238, 0.3);">\uD83D\uDD12 ${T(L)} (1Password)</span>`;if(q.startsWith("env://"))return`<span class="brand-badge" style="color: var(--amber-300); border-color: rgba(251, 191, 36, 0.3);">\uD83D\uDD12 ${T(L)} (Env)</span>`;return`<span style="color: var(--text-dim);">${T(L)}=***</span>`}).join(" "):"None",v=(e.circuitBreakers||[]).find((L)=>L.server_id===c),x='<span class="brand-badge" style="color: var(--green-400); border-color: rgba(52, 211, 153, 0.25);">Circuit: CLOSED</span>';if(v){if(v.state==="open")x=`<span class="brand-badge" style="color: var(--red-400); border-color: rgba(248, 113, 113, 0.4); background: rgba(248, 113, 113, 0.1);">Circuit: OPEN (${v.consecutive_failures} failures)</span>`;else if(v.state==="half_open")x=`<span class="brand-badge" style="color: var(--amber-300); border-color: rgba(251, 191, 36, 0.4); background: rgba(251, 191, 36, 0.1);">Circuit: HALF-OPEN (${v.consecutive_successes} probe)</span>`}let C=u.resilience||e.config.resilience,P=C?`FT: ${C.failureThreshold||3} · Cooldown: ${(C.cooldownMs||30000)/1000}s · AutoRestart: ${C.autoRestart!==!1?"ON":"OFF"}`:"Default Resilience",_=p.status==="degraded",A=p.status==="error"||p.status==="disconnected",w=_?"var(--amber-400)":A?"var(--red-400)":"var(--green-400)",M=(_||A)&&p.error?`
        <div style="background: rgba(239, 68, 68, 0.08); border-left: 3px solid var(--amber-400); border-radius: var(--radius-xs); padding: 8px 12px; margin-top: 8px; display: flex; justify-content: space-between; align-items: center; gap: 8px;">
          <div style="font-size: 11px; color: var(--amber-300); font-family: var(--ff-mono); overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">
            <span style="font-weight: 700; color: var(--amber-400);">⚠️ Diagnostics:</span> ${T(p.error)}
          </div>
          <button class="btn btn-ghost" style="padding: 2px 8px; font-size: 10.5px; color: var(--amber-300); border-color: rgba(251, 191, 36, 0.3);" onclick="window.app.openServerDiagnosticsModal('${T(c)}')">Details</button>
        </div>
      `:"",B=s?m?'<span class="brand-badge" style="color: var(--green-400); border-color: rgba(34, 197, 94, 0.3); background: rgba(34, 197, 94, 0.08);">✔ IN CONSTELLATION</span>':`<span class="brand-badge" style="color: var(--amber-400); border-color: rgba(245, 158, 11, 0.35); background: rgba(245, 158, 11, 0.08);">\uD83D\uDEAB EXCLUDED FROM PROFILE: ${T(n)}</span>`:"";return`
        <div class="bento-card" style="${s&&!m?"margin-bottom: 12px; opacity: 0.65; border: 1px dashed rgba(245, 158, 11, 0.4); background: rgba(0, 0, 0, 0.2);":`margin-bottom: 12px; border-color: ${_?"rgba(251, 191, 36, 0.3)":A?"rgba(248, 113, 113, 0.3)":"var(--border)"};`}">
          <div style="display: flex; justify-content: space-between; align-items: center;">
            <div>
              <div style="display: flex; align-items: center; gap: 8px; margin-bottom: 4px; flex-wrap: wrap;">
                <span style="width: 8px; height: 8px; border-radius: 50%; background: ${w}; display: inline-block;"></span>
                <span style="font-size: 15px; font-weight: 700; color: var(--text-main);">${T(c)}</span>
                <span class="brand-badge">${g}</span>
                <span class="brand-badge" style="color: ${w}; border-color: rgba(245, 158, 11, 0.3);">Status: ${T(p.status)}</span>
                <span class="brand-badge" style="color: var(--cyan-400); border-color: rgba(34, 211, 238, 0.25);">Protocol: ${p.protocol_version}</span>
                ${x}
                ${B}
              </div>
              <div style="font-family: var(--ff-mono); font-size: 12px; color: var(--text-muted); margin-top: 4px;">
                ${u.command?"Command: ":"URL: "}<code>${T(b||"")}</code>
              </div>
              <div style="display: flex; gap: 14px; font-family: var(--ff-mono); font-size: 11px; color: var(--text-dim); margin-top: 4px; align-items: center; flex-wrap: wrap;">
                <span>\uD83D\uDEE1️ ${T(P)}</span>
                ${u.env&&Object.keys(u.env).length>0?`<span>Env: ${y}</span>`:""}
              </div>
            </div>
            <div style="display: flex; gap: 8px; align-items: center;">
              ${s?m?`
                  <button class="btn btn-ghost" style="padding: 4px 10px; font-size: 11.5px; color: var(--amber-400); border-color: rgba(245, 158, 11, 0.3);" onclick="window.app.toggleServerInProfile('${T(n)}', '${T(c)}', false)">
                    Exclude from Profile
                  </button>
                `:`
                  <button class="btn btn-primary" style="padding: 4px 10px; font-size: 11.5px;" onclick="window.app.toggleServerInProfile('${T(n)}', '${T(c)}', true)">
                    + Include in Profile
                  </button>
                `:""}
              <button class="btn btn-ghost" style="padding: 4px 10px; font-size: 11.5px; color: var(--cyan-400); border-color: rgba(34, 211, 238, 0.3);" onclick="window.app.restartServer('${T(c)}')">⚡ Restart</button>
              <button class="btn btn-ghost" style="padding: 4px 10px; font-size: 11.5px;" onclick="window.app.openServerDiagnosticsModal('${T(c)}')">\uD83D\uDD0D Diagnostics</button>
              <button class="btn btn-ghost" style="padding: 4px 10px; font-size: 11.5px;" onclick="window.app.openEditServerModal('${T(c)}')">✏️ Edit</button>
              <button class="btn btn-danger" style="padding: 4px 10px; font-size: 11.5px;" onclick="window.app.deleteServer('${T(c)}')">Remove</button>
            </div>
          </div>
          ${M}
        </div>
      `}).join("");let l=s?`
    <div class="bento-card" style="margin-bottom: 16px; background: rgba(245, 158, 11, 0.05); border: 1px solid rgba(245, 158, 11, 0.3); display: flex; justify-content: space-between; align-items: center;">
      <div style="display: flex; align-items: center; gap: 10px;">
        <span style="font-size: 18px;">\uD83C\uDF0C</span>
        <div>
          <div style="font-size: 13px; font-weight: 700; color: var(--amber-400);">
            Active Profile Constellation: <code style="font-size: 13px; color: var(--text-main);">${T(n)}</code>
            <span class="brand-badge" style="margin-left: 8px; color: var(--text-main);">${o.length} of ${r.length} servers included</span>
          </div>
          <div style="font-size: 11.5px; color: var(--text-muted); margin-top: 2px;">
            Excluded servers are unavailable to clients connected via this profile. Tools from excluded servers are automatically hidden.
          </div>
        </div>
      </div>
      <div style="display: flex; gap: 8px;">
        <button class="btn btn-ghost" style="font-size: 11px; padding: 4px 10px;" onclick="window.app.switchTab('profiles')">Manage Profiles</button>
        <button class="btn btn-ghost" style="font-size: 11px; padding: 4px 10px;" onclick="window.app.setActiveProfile(null)">View All Servers</button>
      </div>
    </div>
  `:"";return`
    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 18px;">
      <div>
        <div style="font-size: 16px; font-weight: 700; color: var(--text-main);">Configured MCP Upstream Servers</div>
        <div style="font-size: 11px; color: var(--text-dim);">Active configuration file: <code>${T(e.configPath)}</code></div>
      </div>
      <div style="display: flex; gap: 8px;">
        <button class="btn btn-ghost" onclick="window.app.reloadFromDisk()">⟳ Reload Config</button>
      </div>
    </div>

    ${l}

    ${i}

    ${ue()}
  `}function ue(){let e=d.getState(),t=e.clients||[],r=Object.keys(e.config.profiles||{}),n=e.clientsCollapsed;if(t.length===0)return"";let a=t.filter((o)=>o.is_attached).length,s=t.map((o)=>{let{is_attached:i,config_exists:l,app_installed:c}=o,u='<span class="brand-badge" style="color: var(--text-dim); border-color: rgba(255, 255, 255, 0.1);">Not Found</span>';if(i){let m=o.attached_profile?` · Profile: ${o.attached_profile}`:"";u=`<span class="brand-badge" style="color: var(--green-400); border-color: rgba(52, 211, 153, 0.3); background: rgba(52, 211, 153, 0.1);">⚡ Connected${T(m)}</span>`}else if(l)u='<span class="brand-badge" style="color: var(--amber-300); border-color: rgba(251, 191, 36, 0.3); background: rgba(251, 191, 36, 0.08);">○ Ready to Connect</span>';else if(c)u='<span class="brand-badge" style="color: var(--cyan-400); border-color: rgba(34, 211, 238, 0.25);">○ App Installed</span>';let g=e.activeProfile,b=r.map((m)=>`
      <option value="${T(m)}" ${g===m||o.attached_profile===m?"selected":""}>Profile: ${T(m)}</option>
    `).join(""),p=i?`<button class="btn btn-ghost" style="padding: 4px 10px; font-size: 11px; color: var(--red-400); border-color: rgba(248, 113, 113, 0.3);" onclick="window.app.detachClient('${T(o.id)}')">Disconnect</button>`:`<button class="btn btn-primary" style="padding: 4px 10px; font-size: 11px;" onclick="window.app.attachClient('${T(o.id)}')">⚡ Connect</button>`;return`
      <div style="background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 12px 14px; display: flex; flex-direction: column; justify-content: space-between; gap: 8px;">
        <div style="display: flex; justify-content: space-between; align-items: flex-start;">
          <div>
            <div style="font-weight: 700; font-size: 13.5px; color: var(--text-main); display: flex; align-items: center; gap: 6px;">
              <span>${T(o.name)}</span>
            </div>
            <div style="font-size: 10.5px; color: var(--text-dim); margin-top: 2px;">${T(o.category)}</div>
          </div>
          ${u}
        </div>
        
        <div style="font-family: var(--ff-mono); font-size: 10px; color: var(--text-muted); overflow: hidden; text-overflow: ellipsis; white-space: nowrap;" title="${T(o.config_path)}">
          ${T(o.config_path)}
        </div>

        <div style="display: flex; align-items: center; justify-content: space-between; gap: 6px; margin-top: 4px; padding-top: 6px; border-top: 1px solid var(--border-subtle);">
          ${r.length>0&&!i?`
            <select id="client-prof-${T(o.id)}" class="form-input" style="font-size: 10.5px; padding: 2px 6px; height: 26px; width: 130px;">
              <option value="">All Tools (Default)</option>
              ${b}
            </select>
          `:`<div style="font-size: 10.5px; color: var(--text-dim);">${o.other_servers_count>0?`${o.other_servers_count} other tools`:"Single tool facade"}</div>`}
          ${p}
        </div>
      </div>
    `}).join("");return`
    <div class="bento-card" style="margin-top: 28px; padding: 14px 18px; border-color: rgba(245, 158, 11, 0.2); background: rgba(18, 24, 38, 0.4);">
      <div style="display: flex; justify-content: space-between; align-items: center; cursor: pointer; user-select: none;" onclick="window.app.toggleClientsCollapse()">
        <div>
          <div style="font-size: 14px; font-weight: 700; color: var(--text-main); display: flex; align-items: center; gap: 8px;">
            <span>⚡ 1-Click AI Client Integrations</span>
            <span class="brand-badge" style="color: var(--amber-400); border-color: rgba(245, 158, 11, 0.3);">${a>0?`${a} Connected`:"Auto-Sync"}</span>
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
  `}function T(e){return String(e||"").replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;")}function re(){let e=d.getState(),t=e.playgroundMode||"tools",r=e.capabilities||[],n=e.resources||[],a=e.prompts||[],s=e.capabilitiesHiddenByPolicy||0,o=e.resourcesHiddenByPolicy||0,i=e.promptsHiddenByPolicy||0,l=e.activeProfile,u=!!(l?e.config.profiles?.[l]:void 0),g=`
    <div style="display: flex; gap: 8px; margin-bottom: 12px; align-items: center; justify-content: space-between; flex-wrap: wrap;">
      <div style="display: inline-flex; padding: 3px; background: rgba(0,0,0,0.3); border: 1px solid var(--border); border-radius: var(--radius-sm); align-items: center;">
        <button 
          class="btn ${t==="tools"?"btn-primary":"btn-ghost"}" 
          style="padding: 4px 12px; font-size: 11.5px; height: 28px; display: inline-flex; align-items: center; gap: 6px;"
          onclick="window.app.setPlaygroundMode('tools')"
        >
          <span>\uD83D\uDEE0️ Tools (${r.length})</span>
          ${s>0?`<span class="badge" style="background: rgba(245, 158, 11, 0.2); color: var(--amber-300); font-size: 9.5px; padding: 1px 5px;" title="${s} tools hidden by constellation/policy">+${s} hidden</span>`:""}
        </button>
        <button 
          class="btn ${t==="resources"?"btn-primary":"btn-ghost"}" 
          style="padding: 4px 12px; font-size: 11.5px; height: 28px; display: inline-flex; align-items: center; gap: 6px;"
          onclick="window.app.setPlaygroundMode('resources')"
        >
          <span>\uD83D\uDCC4 Resources (${n.length})</span>
          ${o>0?`<span class="badge" style="background: rgba(245, 158, 11, 0.2); color: var(--amber-300); font-size: 9.5px; padding: 1px 5px;" title="${o} resources hidden by constellation/policy">+${o} hidden</span>`:""}
        </button>
        <button 
          class="btn ${t==="prompts"?"btn-primary":"btn-ghost"}" 
          style="padding: 4px 12px; font-size: 11.5px; height: 28px; display: inline-flex; align-items: center; gap: 6px;"
          onclick="window.app.setPlaygroundMode('prompts')"
        >
          <span>\uD83D\uDCAC Prompts (${a.length})</span>
          ${i>0?`<span class="badge" style="background: rgba(245, 158, 11, 0.2); color: var(--amber-300); font-size: 9.5px; padding: 1px 5px;" title="${i} prompts hidden by constellation/policy">+${i} hidden</span>`:""}
        </button>
      </div>

      <div style="display: flex; align-items: center; gap: 12px;">
        ${t==="tools"&&s>0?`
          <div style="font-size: 11px; color: var(--amber-300); background: rgba(245, 158, 11, 0.1); border: 1px solid rgba(245, 158, 11, 0.25); padding: 3px 8px; border-radius: var(--radius-sm); display: flex; align-items: center; gap: 6px;">
            <span>\uD83D\uDEE1️ ${s} tool${s>1?"s":""} filtered ${u?`(Profile: ${h(l)})`:"by policy"}</span>
            <a href="javascript:void(0)" onclick="window.app.switchTab('policy')" style="color: var(--amber-400); text-decoration: underline; font-weight: 600;">View Policy</a>
            ${u?`<a href="javascript:void(0)" onclick="window.app.switchTab('servers')" style="color: var(--cyan-400); text-decoration: underline; font-weight: 600; margin-left: 4px;">Server Hub</a>`:""}
          </div>
        `:t==="resources"&&o>0?`
          <div style="font-size: 11px; color: var(--amber-300); background: rgba(245, 158, 11, 0.1); border: 1px solid rgba(245, 158, 11, 0.25); padding: 3px 8px; border-radius: var(--radius-sm); display: flex; align-items: center; gap: 6px;">
            <span>\uD83D\uDEE1️ ${o} resource${o>1?"s":""} filtered ${u?`(Profile: ${h(l)})`:"by policy"}</span>
            <a href="javascript:void(0)" onclick="window.app.switchTab('policy')" style="color: var(--amber-400); text-decoration: underline; font-weight: 600;">View Policy</a>
            ${u?`<a href="javascript:void(0)" onclick="window.app.switchTab('servers')" style="color: var(--cyan-400); text-decoration: underline; font-weight: 600; margin-left: 4px;">Server Hub</a>`:""}
          </div>
        `:t==="prompts"&&i>0?`
          <div style="font-size: 11px; color: var(--amber-300); background: rgba(245, 158, 11, 0.1); border: 1px solid rgba(245, 158, 11, 0.25); padding: 3px 8px; border-radius: var(--radius-sm); display: flex; align-items: center; gap: 6px;">
            <span>\uD83D\uDEE1️ ${i} prompt${i>1?"s":""} filtered ${u?`(Profile: ${h(l)})`:"by policy"}</span>
            <a href="javascript:void(0)" onclick="window.app.switchTab('policy')" style="color: var(--amber-400); text-decoration: underline; font-weight: 600;">View Policy</a>
            ${u?`<a href="javascript:void(0)" onclick="window.app.switchTab('servers')" style="color: var(--cyan-400); text-decoration: underline; font-weight: 600; margin-left: 4px;">Server Hub</a>`:""}
          </div>
        `:`
          <div style="font-size: 11.5px; color: var(--text-dim);">
            ${t==="tools"?"Interactive Tool Caller & Context Distillation":t==="resources"?"Live MCP Resource Inspector & Reader":"Prompt Template Studio & Variable Binder"}
          </div>
        `}
      </div>
    </div>
  `;if(t==="resources")return`
      ${g}
      ${ve(e)}
    `;if(t==="prompts")return`
      ${g}
      ${me(e)}
    `;return`
    ${g}
    ${ge(e)}
    ${e.isBatchModalOpen?fe(e):""}
  `}function G(e,t=!1){if(!e||!e.properties)return{};let r=e.properties||{},n=Array.isArray(e.required)?e.required:[],a={};for(let[s,o]of Object.entries(r)){let i=n.includes(s);if(t&&!i)continue;if(o.default!==void 0)a[s]=o.default;else if(Array.isArray(o.enum)&&o.enum.length>0)a[s]=o.enum[0];else if(o.examples&&Array.isArray(o.examples)&&o.examples.length>0)a[s]=o.examples[0];else if(o.example!==void 0)a[s]=o.example;else switch(o.type||"string"){case"string":a[s]=i?`sample_${s}`:"";break;case"number":case"integer":a[s]=0;break;case"boolean":a[s]=!0;break;case"array":a[s]=[];break;case"object":a[s]={};break;default:a[s]=`sample_${s}`}}return a}function ge(e){let t=e.capabilities||[],r=e.selectedCapabilityId||(t.length>0?t[0].id:null),n=t.find((y)=>y.id===r),a=e.isExecutingCapability,s=e.capabilitiesHiddenByPolicy||0,o=e.activeProfile,i=!!(o&&e.config.profiles?.[o]),l="";if(t.length===0)l=`
      <div style="padding: 24px 16px; text-align: center; color: var(--text-dim); font-size: 11.5px;">
        No tools or capabilities discovered from connected servers.
      </div>
    `;else l=t.map((y)=>`
        <div class="cap-item ${y.id===r?"active":""}" onclick="window.app.selectCapability('${h(y.id)}')">
          <div style="display: flex; justify-content: space-between; align-items: center;">
            <span style="font-weight: 600; color: var(--text-main); font-family: var(--ff-mono); font-size: 12px;">${h(y.id)}</span>
            <span style="font-size: 10px; color: var(--green-400);">${h(y.mode||"read")}</span>
          </div>
          <div style="font-size: 11px; color: var(--text-dim); margin-top: 2px;">server: ${h(y.server||"local")}</div>
        </div>
      `).join("");let c=n?.input_schema,u=c?.properties||{},g=Array.isArray(c?.required)?c.required:[],b=Object.entries(u),p="";if(b.length>0)p=`
      <div style="display: flex; flex-wrap: wrap; gap: 6px; margin-bottom: 8px; align-items: center;">
        <span style="font-size: 10px; font-weight: 700; color: var(--text-dim); text-transform: uppercase;">Fields:</span>
        ${b.map(([y,v])=>{let x=g.includes(y),C=v.type||(v.enum?"enum":"any"),P=x?"rgba(239, 68, 68, 0.15)":"rgba(148, 163, 184, 0.1)",_=x?"var(--red-400)":"var(--text-muted)",A=x?"rgba(239, 68, 68, 0.3)":"var(--border)",w=v.description?` - ${v.description}`:"";return`
            <button 
              type="button" 
              class="btn" 
              style="padding: 2px 7px; font-size: 10.5px; font-family: var(--ff-mono); background: ${P}; color: ${_}; border: 1px solid ${A}; border-radius: var(--radius-sm);" 
              title="Click to insert '${y}' (${C}${w})" 
              onclick="window.app.insertPlaygroundArgKey('${h(y)}', '${h(C)}', ${h(JSON.stringify(v.default??null))})"
            >
              + ${h(y)} <span style="font-size: 9px; opacity: 0.7;">(${C}${x?" *":""})</span>
            </button>
          `}).join("")}
      </div>
    `;let m="{}";if(r&&e.playgroundArgs&&e.playgroundArgs[r]!==void 0)m=e.playgroundArgs[r];else{let y=G(c,!1);m=JSON.stringify(y,null,2)}return`
    <div style="display: grid; grid-template-columns: 320px 1fr; gap: 16px; height: calc(100vh - 165px);">
      <!-- Left Sidebar: Capabilities Catalog -->
      <div style="background: var(--surface-card); border: 1px solid var(--border); border-radius: var(--radius-md); display: flex; flex-direction: column; overflow: hidden;">
        <div style="padding: 12px; border-bottom: 1px solid var(--border);">
          <input type="text" class="form-input" placeholder="Search ${t.length} capabilities..." oninput="window.app.filterCapabilities(this.value)">
        </div>
        <div style="flex: 1; overflow-y: auto; padding: 8px;" id="pg-cap-list">
          ${l}
        </div>
        ${s>0?`
          <div style="padding: 8px 12px; background: rgba(245, 158, 11, 0.08); border-top: 1px solid rgba(245, 158, 11, 0.2); font-size: 11px; color: var(--amber-300); display: flex; justify-content: space-between; align-items: center;">
            <span>\uD83D\uDEE1️ ${s} tool${s>1?"s":""} filtered ${i?`(${h(o)})`:""}</span>
            <div style="display: flex; gap: 6px;">
              <a href="javascript:void(0)" onclick="window.app.switchTab('policy')" style="color: var(--amber-400); text-decoration: underline; font-weight: 600; font-size: 10.5px;">Policy</a>
              ${i?`<a href="javascript:void(0)" onclick="window.app.switchTab('servers')" style="color: var(--cyan-400); text-decoration: underline; font-weight: 600; font-size: 10.5px;">Servers</a>`:""}
            </div>
          </div>
        `:""}
      </div>

      <!-- Right Panel: Capability Execution & Envelope Visualizer -->
      <div style="background: var(--surface-card); border: 1px solid var(--border); border-radius: var(--radius-md); display: flex; flex-direction: column; overflow: hidden;">
        <div style="padding: 14px 18px; border-bottom: 1px solid var(--border); display: flex; justify-content: space-between; align-items: center;">
          <div>
            <div style="font-size: 15px; font-weight: 700; color: var(--text-main); font-family: var(--ff-mono);" id="pg-selected-title">
              ${h(n?n.id:"No Capability Selected")}
            </div>
            <div style="font-size: 11.5px; color: var(--text-dim);" id="pg-selected-desc">
              ${h(n?n.summary||n.description:"Connect servers to inspect and execute tools")}
            </div>
          </div>
          
          <div style="display: flex; align-items: center; gap: 10px;">
            ${a?`
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
              <button class="btn btn-primary" onclick="window.app.executePlaygroundTool()" ${n?"":"disabled"}>
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
                ${g.length>0?`
                  <button type="button" class="btn btn-ghost" style="padding: 2px 7px; font-size: 10.5px;" title="Fill only required schema fields" onclick="window.app.fillPlaygroundSampleArgs(true)">\uD83E\uDDF9 Required Only</button>
                `:""}
                <button type="button" class="btn btn-ghost" style="padding: 2px 7px; font-size: 10.5px;" title="Format JSON" onclick="window.app.formatPlaygroundArgs()">\uD83D\uDCCB Format</button>
                <button type="button" class="btn btn-ghost" style="padding: 2px 8px; font-size: 11px;" onclick="window.app.openBatchModal()">⚡ Pipeline Builder</button>
              </div>
            </div>

            ${p}

            <textarea class="form-textarea" rows="7" id="pg-args-input" oninput="window.app.updatePlaygroundArgs(this.value)">${h(m)}</textarea>

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
            ${n&&n.input_schema?`
              <div style="margin-top: 14px;">
                <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 4px;">
                  <label class="form-label" style="margin: 0;">Input JSON Schema</label>
                  <span style="font-size: 10px; color: var(--text-dim); font-family: var(--ff-mono);">${b.length} field${b.length===1?"":"s"} (${g.length} required)</span>
                </div>
                <pre style="background: var(--surface); padding: 10px; border-radius: var(--radius-sm); border: 1px solid var(--border); font-size: 11px; color: var(--text-muted); max-height: 140px; overflow-y: auto;">${h(JSON.stringify(n.input_schema,null,2))}</pre>
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
                      ${h(e.executionResult.data?.task?.status||e.executionResult.data?.status||"TASK_CREATED").toUpperCase()}
                    </span>
                    <span style="font-family: var(--ff-mono); font-size: 12px; font-weight: 700; color: var(--text-main);">${h(e.executionResult.data?.task?.taskId||e.executionResult.data?.taskId||"")}</span>
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

            <pre id="pg-response-json" style="flex: 1; background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 14px; color: var(--amber-300); font-size: 11.5px; overflow-y: auto; margin: 0; white-space: pre-wrap; word-break: break-word;">${e.executionResult?h(JSON.stringify(e.executionResult.data,null,2)):"// Response envelope output will be formatted here"}</pre>
          </div>
        </div>
      </div>
    </div>
  `}function ve(e){let t=e.resources||[],r=e.resourcesHiddenByPolicy||0,n=e.selectedResourceId||(t.length>0?t[0].id:null),a=t.find((i)=>i.id===n),s=e.resourceReadResult,o="";if(t.length===0)o=`
      <div style="padding: 24px 16px; text-align: center; color: var(--text-dim); font-size: 11.5px;">
        No resources exposed by connected MCP servers.
      </div>
    `;else o=t.map((i)=>{let l=i.id===n?"active":"",c=i.uri?i.uri.split(":")[0]:"res";return`
        <div class="cap-item ${l}" onclick="window.app.selectResource('${h(i.id)}')">
          <div style="display: flex; justify-content: space-between; align-items: center;">
            <span style="font-weight: 600; color: var(--text-main); font-family: var(--ff-mono); font-size: 12px;">${h(i.name||i.id)}</span>
            <span class="badge" style="font-size: 9.5px; background: rgba(56, 189, 248, 0.15); color: var(--cyan-400);">${h(c)}</span>
          </div>
          <div style="font-size: 11px; color: var(--text-dim); margin-top: 2px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">${h(i.uri)}</div>
          <div style="display: flex; justify-content: space-between; font-size: 10px; color: var(--text-muted); margin-top: 4px;">
            <span>server: ${h(i.server||"local")}</span>
            <span>${h(i.mime_type||"text/plain")}</span>
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
        ${r>0?`
          <div style="padding: 8px 12px; background: rgba(245, 158, 11, 0.08); border-top: 1px solid rgba(245, 158, 11, 0.2); font-size: 11px; color: var(--amber-300); display: flex; justify-content: space-between; align-items: center;">
            <span>\uD83D\uDEE1️ ${r} resource${r>1?"s":""} hidden by policy</span>
            <a href="javascript:void(0)" onclick="window.app.switchTab('policy')" style="color: var(--amber-400); text-decoration: underline; font-weight: 600; font-size: 10.5px;">Edit Policy</a>
          </div>
        `:""}
      </div>

      <!-- Right Panel: Resource Content Reader & Metadata Inspector -->
      <div style="background: var(--surface-card); border: 1px solid var(--border); border-radius: var(--radius-md); display: flex; flex-direction: column; overflow: hidden;">
        <div style="padding: 14px 18px; border-bottom: 1px solid var(--border); display: flex; justify-content: space-between; align-items: center;">
          <div>
            <div style="font-size: 15px; font-weight: 700; color: var(--text-main); font-family: var(--ff-mono);">
              ${h(a?a.name||a.id:"No Resource Selected")}
            </div>
            <div style="font-size: 11.5px; color: var(--cyan-400); font-family: var(--ff-mono);">
              ${h(a?a.uri:"Select a resource from the list to read live content")}
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
                  <div><span style="color: var(--text-muted);">Server:</span> <strong style="color: var(--text-main);">${h(a.server)}</strong></div>
                  <div><span style="color: var(--text-muted);">MIME Type:</span> <strong style="color: var(--text-main);">${h(a.mime_type||"text/plain")}</strong></div>
                </div>
                ${a.description?`
                  <div style="margin-top: 8px; font-size: 11.5px; color: var(--text-dim); border-top: 1px solid rgba(255,255,255,0.05); padding-top: 6px;">
                    ${h(a.description)}
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
            <pre style="flex: 1; background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 14px; color: var(--cyan-400); font-size: 11.5px; overflow-y: auto; margin: 0; white-space: pre-wrap; word-break: break-word;">${s?h(JSON.stringify(s.data,null,2)):'// Click "Read Resource Content" to inspect live payload'}</pre>
          </div>
        </div>
      </div>
    </div>
  `}function me(e){let t=e.prompts||[],r=e.promptsHiddenByPolicy||0,n=e.selectedPromptId||(t.length>0?t[0].id:null),a=t.find((l)=>l.id===n),s=e.promptGetResult,o="";if(t.length===0)o=`
      <div style="padding: 24px 16px; text-align: center; color: var(--text-dim); font-size: 11.5px;">
        No prompt templates registered by connected MCP servers.
      </div>
    `;else o=t.map((l)=>{let c=l.id===n?"active":"",u=l.arguments?l.arguments.length:0;return`
        <div class="cap-item ${c}" onclick="window.app.selectPrompt('${h(l.id)}')">
          <div style="display: flex; justify-content: space-between; align-items: center;">
            <span style="font-weight: 600; color: var(--text-main); font-family: var(--ff-mono); font-size: 12px;">${h(l.name||l.id)}</span>
            <span class="badge" style="font-size: 9.5px; background: rgba(168, 85, 247, 0.15); color: var(--purple-400);">${u} args</span>
          </div>
          <div style="font-size: 11px; color: var(--text-dim); margin-top: 2px;">${h(l.description||l.title||"Prompt template")}</div>
          <div style="font-size: 10px; color: var(--text-muted); margin-top: 4px;">server: ${h(l.server||"local")}</div>
        </div>
      `}).join("");let i="";if(a&&a.arguments&&a.arguments.length>0)i=a.arguments.map((l)=>`
      <div class="form-group" style="margin-bottom: 12px;">
        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 4px;">
          <label class="form-label" style="margin: 0; font-family: var(--ff-mono);">${h(l.name)}</label>
          ${l.required?'<span class="badge" style="background: rgba(239, 68, 68, 0.15); color: var(--red-400); font-size: 9px;">REQUIRED</span>':'<span style="font-size: 10px; color: var(--text-dim);">optional</span>'}
        </div>
        ${l.description?`<div style="font-size: 11px; color: var(--text-dim); margin-bottom: 4px;">${h(l.description)}</div>`:""}
        <input type="text" class="form-input prompt-arg-input" data-arg-name="${h(l.name)}" placeholder="Enter ${h(l.name)}..." />
      </div>
    `).join("");else if(a)i=`
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
        ${r>0?`
          <div style="padding: 8px 12px; background: rgba(245, 158, 11, 0.08); border-top: 1px solid rgba(245, 158, 11, 0.2); font-size: 11px; color: var(--amber-300); display: flex; justify-content: space-between; align-items: center;">
            <span>\uD83D\uDEE1️ ${r} prompt${r>1?"s":""} hidden by policy</span>
            <a href="javascript:void(0)" onclick="window.app.switchTab('policy')" style="color: var(--amber-400); text-decoration: underline; font-weight: 600; font-size: 10.5px;">Edit Policy</a>
          </div>
        `:""}
      </div>

      <!-- Right Panel: Prompt Parameter Binder & Message Envelope Preview -->
      <div style="background: var(--surface-card); border: 1px solid var(--border); border-radius: var(--radius-md); display: flex; flex-direction: column; overflow: hidden;">
        <div style="padding: 14px 18px; border-bottom: 1px solid var(--border); display: flex; justify-content: space-between; align-items: center;">
          <div>
            <div style="font-size: 15px; font-weight: 700; color: var(--text-main); font-family: var(--ff-mono);">
              ${h(a?a.name||a.id:"No Prompt Selected")}
            </div>
            <div style="font-size: 11.5px; color: var(--text-dim);">
              ${h(a?a.description||a.title||"Bind variables and render messages":"Select a prompt from the list to test")}
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
            <pre style="flex: 1; background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 14px; color: #c084fc; font-size: 11.5px; overflow-y: auto; margin: 0; white-space: pre-wrap; word-break: break-word;">${s?h(JSON.stringify(s.data,null,2)):'// Click "Render Prompt Messages" to view resolved system/user messages'}</pre>
          </div>
        </div>
      </div>
    </div>
  `}function fe(e){let t=e.capabilities||[],r=e.batchSteps||[];return`
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
          ${r.map((a,s)=>{let o=t.find((p)=>p.id===a.capability_id),i=o?.input_schema,l=i?.properties||{},c=Array.isArray(i?.required)?i.required:[],u=Object.entries(l),g=t.map((p)=>`
      <option value="${h(p.id)}" ${p.id===a.capability_id?"selected":""}>
        ${h(p.id)} (${h(p.server||"local")})
      </option>
    `).join(""),b="";if(u.length>0)b=`
        <div style="display: flex; flex-wrap: wrap; gap: 4px; margin-top: 6px; margin-bottom: 6px; align-items: center;">
          <span style="font-size: 9.5px; font-weight: 700; color: var(--text-dim); text-transform: uppercase;">Parameters:</span>
          ${u.map(([p,m])=>{let y=c.includes(p),v=m.type||(m.enum?"enum":"any");return`
              <span style="font-size: 9.5px; font-family: var(--ff-mono); padding: 1px 5px; background: ${y?"rgba(239, 68, 68, 0.15)":"rgba(148, 163, 184, 0.1)"}; color: ${y?"var(--red-400)":"var(--text-muted)"}; border: 1px solid ${y?"rgba(239, 68, 68, 0.3)":"var(--border)"}; border-radius: 3px;" title="${h(m.description||"")}">
                ${h(p)} (${v}${y?" *":""})
              </span>
            `}).join("")}
        </div>
      `;return`
      <div style="background: rgba(0,0,0,0.3); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 14px; margin-bottom: 12px;">
        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 10px;">
          <div style="display: flex; align-items: center; gap: 8px;">
            <span class="badge" style="background: rgba(56, 189, 248, 0.15); color: var(--cyan-400); font-family: var(--ff-mono); font-weight: 700;">STEP ${s+1}</span>
            <span style="font-size: 11px; font-family: var(--ff-mono); color: var(--text-dim);">id: ${h(a.id)}</span>
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
              <input type="checkbox" ${a.continue_on_error?"checked":""} onchange="window.app.updateBatchStepContinueOnError(${s}, this.checked)" />
              <span>Continue pipeline on step failure</span>
            </label>
          </div>
        </div>

        ${b}

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
          >${h(a.argsJson)}</textarea>
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
            ${r.length} sequential execution steps configured
          </div>
          <div style="display: flex; gap: 10px;">
            <button class="btn btn-ghost" onclick="window.app.closeBatchModal()">Cancel</button>
            <button class="btn btn-primary" onclick="window.app.executeBatchPipeline()">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polygon points="5 3 19 12 5 21 5 3"></polygon></svg>
              Run Batch Pipeline (${r.length} Steps)
            </button>
          </div>
        </div>
      </div>
    </div>
  `}function h(e){return String(e||"").replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;")}function ae(e){let t=e.tasks||[],r=e.taskFilterStatus||"all",n=t.filter((p)=>p.status==="input_required"),a=t.filter((p)=>p.status==="working"),s=t.filter((p)=>p.status==="completed"),o=t.filter((p)=>p.status==="cancelled"),i=t.filter((p)=>p.status==="failed"),l=r==="all"?t:t.filter((p)=>p.status===r),c=e.config.policy?.require_approval||e.config.policy?.requireApproval||[],u=n.length===0?`
    <div style="padding: 36px 24px; text-align: center; background: var(--surface-card); border-radius: var(--radius-md); border: 1px dashed var(--border);">
      <div style="width: 44px; height: 44px; border-radius: 50%; background: rgba(52, 211, 153, 0.12); border: 1px solid rgba(52, 211, 153, 0.3); display: flex; align-items: center; justify-content: center; margin: 0 auto 12px; color: var(--green-400); font-size: 18px; font-weight: 700;">
        ✓
      </div>
      <div style="font-size: 14.5px; font-weight: 600; color: var(--text-main); margin-bottom: 5px;">No Tasks Awaiting Input or Approval</div>
      <div style="font-size: 11.5px; color: var(--text-dim); max-width: 520px; margin: 0 auto; line-height: 1.6;">
        Tool calls requiring Human-in-the-Loop approval or returning asynchronous <code style="color: var(--amber-300); font-family: var(--ff-mono);">input_required</code> tasks will suspend here for operator inspection, parameter editing, and response submission.
      </div>
    </div>
  `:n.map((p)=>{let m=p.inputRequests||{},y=Object.keys(m),v=y.length>0,x=Math.floor(Date.now()/1000),C=p.expiresAtEpochSecs?Math.max(0,p.expiresAtEpochSecs-x):p.ttlSeconds||300;return`
      <div class="bento-card" style="border: 1px solid rgba(245, 158, 11, 0.35); background: var(--surface-card); margin-bottom: 14px; padding: 18px;">
        <div style="display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 12px;">
          <div>
            <div style="display: flex; align-items: center; gap: 8px;">
              <span class="brand-badge" style="background: rgba(245, 158, 11, 0.2); color: var(--amber-300); border-color: rgba(245, 158, 11, 0.5);">
                INPUT REQUIRED
              </span>
              <span style="font-family: var(--ff-mono); font-size: 12px; color: var(--text-muted);">${k(p.taskId)}</span>
            </div>
            <div style="margin-top: 6px; display: flex; align-items: center; gap: 8px;">
              <span style="font-size: 14.5px; font-weight: 700; color: var(--text-main); font-family: var(--ff-mono);">
                ${k(p.capabilityId||"Tool Execution")}
              </span>
              ${p.serverId?`<span style="font-size: 11px; color: var(--text-dim);">via <span style="color: var(--cyan-400); font-family: var(--ff-mono);">${k(p.serverId)}</span></span>`:""}
            </div>
          </div>

          <div style="text-align: right; font-family: var(--ff-mono); font-size: 11px; color: var(--text-dim);">
            ${p.createdAtEpochSecs?`<div>Created: <span style="color: var(--text-muted);">${new Date(p.createdAtEpochSecs*1000).toLocaleTimeString()}</span></div>`:""}
            <div style="color: var(--amber-400); margin-top: 2px;">TTL Remaining: ${C}s</div>
          </div>
        </div>

        <!-- Caller Context -->
        ${p.context?`
          <div style="background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 8px 12px; font-family: var(--ff-mono); font-size: 11px; display: flex; flex-wrap: wrap; gap: 16px; margin-bottom: 12px; color: var(--text-muted);">
            ${p.context.actor_id?`<div><span style="color: var(--text-dim);">Actor:</span> <span style="color: var(--cyan-400);">${k(p.context.actor_id)}</span></div>`:""}
            ${p.context.operation_id?`<div><span style="color: var(--text-dim);">Operation:</span> <span style="color: var(--text-main);">${k(p.context.operation_id)}</span></div>`:""}
            ${p.context.grant_id?`<div><span style="color: var(--text-dim);">Grant:</span> <span style="color: var(--text-main);">${k(p.context.grant_id)}</span></div>`:""}
          </div>
        `:""}

        <!-- Dynamic Input Requests Form -->
        <div style="margin-bottom: 14px;">
          <div style="font-size: 11px; font-weight: 600; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.5px; margin-bottom: 8px;">
            ${v?"Required Input Responses (MRTR / HITL)":"Input Responses Payload (JSON)"}
          </div>

          ${v?`
            <div style="display: flex; flex-direction: column; gap: 10px;">
              ${y.map((P)=>{let _=m[P]||{},A=typeof _==="string"?_:_.prompt||_.description||_.title||P,w=_.type||"text",M=_.default!==void 0?JSON.stringify(_.default):_.value!==void 0?JSON.stringify(_.value):_.sanitized_args?JSON.stringify(_.sanitized_args,null,2):"";if(w==="approval_review")return`
                    <div style="background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 10px 12px;">
                      <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 4px;">
                        <label style="font-size: 11.5px; font-weight: 600; color: var(--amber-300); font-family: var(--ff-mono);">${k(P)}</label>
                        <span class="brand-badge" style="font-size: 9.5px; padding: 1px 5px;">APPROVAL GATED</span>
                      </div>
                      <div style="font-size: 11px; color: var(--text-dim); margin-bottom: 6px;">${k(A)}</div>
                      <div style="margin-bottom: 8px;">
                        <label style="font-size: 10.5px; color: var(--text-dim); display: block; margin-bottom: 2px;">Decision:</label>
                        <select id="task-input-${k(p.taskId)}-${k(P)}-decision" class="form-input" style="font-size: 11.5px; font-family: var(--ff-mono); padding: 4px 8px;">
                          <option value="true" selected>Approve &amp; Execute</option>
                          <option value="false">Reject Execution</option>
                        </select>
                      </div>
                      <div>
                        <label style="font-size: 10.5px; color: var(--text-dim); display: block; margin-bottom: 2px;">Parameters (Editable):</label>
                        <textarea id="task-input-${k(p.taskId)}-${k(P)}" class="form-textarea" rows="3" style="color: var(--green-400); font-family: var(--ff-mono); font-size: 11.5px;">${k(M)}</textarea>
                      </div>
                    </div>
                  `;return`
                  <div style="background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 10px 12px;">
                    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 4px;">
                      <label style="font-size: 11.5px; font-weight: 600; color: var(--amber-300); font-family: var(--ff-mono);">${k(P)}</label>
                      <span class="brand-badge" style="font-size: 9.5px; padding: 1px 5px;">${k(w)}</span>
                    </div>
                    <div style="font-size: 11px; color: var(--text-dim); margin-bottom: 6px;">${k(A)}</div>
                    ${w==="confirmation"||w==="boolean"?`
                      <select id="task-input-${k(p.taskId)}-${k(P)}" class="form-input" style="font-size: 11.5px; font-family: var(--ff-mono); padding: 4px 8px;">
                        <option value="true" selected>true (Approve / Confirm)</option>
                        <option value="false">false (Reject / Deny)</option>
                      </select>
                    `:_.sanitized_args||w==="object"||w==="json"?`
                      <textarea id="task-input-${k(p.taskId)}-${k(P)}" class="form-textarea" rows="3" style="color: var(--green-400); font-family: var(--ff-mono); font-size: 11.5px;">${k(M)}</textarea>
                    `:`
                      <input id="task-input-${k(p.taskId)}-${k(P)}" type="text" class="form-input" value="${k(M)}" placeholder="Enter ${k(P)} response..." style="font-size: 11.5px; font-family: var(--ff-mono);">
                    `}
                  </div>
                `}).join("")}
            </div>
          `:`
            <textarea id="task-raw-input-${k(p.taskId)}" class="form-textarea" rows="3" style="color: var(--green-400); font-family: var(--ff-mono); font-size: 11.5px;">{}</textarea>
          `}
        </div>

        <!-- Action Footer -->
        <div style="display: flex; justify-content: space-between; align-items: center; border-top: 1px solid var(--border); padding-top: 12px;">
          <div style="display: flex; align-items: center; gap: 8px;">
            <input id="task-operator-${k(p.taskId)}" type="text" class="form-input" placeholder="Operator ID" value="security-operator" style="width: 180px; padding: 5px 10px; font-size: 11px;">
          </div>

          <div style="display: flex; align-items: center; gap: 8px;">
            <button class="btn btn-danger" onclick="window.app.promptCancelTask('${k(p.taskId)}')">
              ✕ Cancel Task
            </button>
            <button class="btn btn-primary" onclick="window.app.submitTaskInputResponses('${k(p.taskId)}')">
              ✓ Submit &amp; Resume
            </button>
          </div>
        </div>
      </div>
    `}).join(""),g=c.length===0?`
    <div style="color: var(--text-dim); font-size: 11.5px; line-height: 1.5; padding: 8px 0;">
      No explicit <code style="color: var(--amber-400);">require_approval</code> rules active. Gated execution rules convert matching tool calls into tasks in real-time.
    </div>
  `:c.map((p)=>`
    <div style="display: flex; justify-content: space-between; align-items: center; background: var(--surface); padding: 8px 10px; border-radius: var(--radius-sm); border: 1px solid var(--border); margin-bottom: 6px;">
      <span style="font-family: var(--ff-mono); font-size: 11.5px; color: var(--amber-300); font-weight: 500;">\uD83D\uDEE1️ ${k(p)}</span>
      <span class="brand-badge" style="font-size: 9.5px; padding: 1px 5px;">GATED</span>
    </div>
  `).join(""),b=l.length===0?`
    <tr>
      <td colspan="6" style="padding: 24px; text-align: center; color: var(--text-dim); font-size: 12px;">
        No tasks found matching filter "${k(r)}".
      </td>
    </tr>
  `:l.map((p)=>{let m=p.status==="completed"?"background: rgba(52, 211, 153, 0.12); color: var(--green-400); border-color: rgba(52, 211, 153, 0.3);":p.status==="working"?"background: rgba(56, 189, 248, 0.15); color: var(--cyan-400); border-color: rgba(56, 189, 248, 0.4);":p.status==="input_required"?"background: rgba(245, 158, 11, 0.2); color: var(--amber-300); border-color: rgba(245, 158, 11, 0.5);":p.status==="cancelled"?"background: rgba(148, 163, 184, 0.15); color: var(--text-muted); border-color: rgba(148, 163, 184, 0.3);":"background: rgba(248, 113, 113, 0.12); color: var(--red-400); border-color: rgba(248, 113, 113, 0.3);",y=p.progress!==void 0?Math.round(p.progress*100):p.status==="completed"?100:p.status==="working"?50:0,v=p.createdAtEpochSecs?new Date(p.createdAtEpochSecs*1000).toLocaleTimeString():"—";return`
      <tr style="border-bottom: 1px solid rgba(255,255,255,0.03); transition: background 0.15s;" onmouseover="this.style.background='var(--surface-hover)'" onmouseout="this.style.background='transparent'">
        <td style="padding: 10px 14px;">
          <span class="brand-badge" style="${m}">
            ${p.status.toUpperCase()}
          </span>
        </td>
        <td style="padding: 10px 14px; font-family: var(--ff-mono); font-weight: 600; color: var(--text-main); font-size: 11.5px;">
          ${k(p.capabilityId||"Tool Execution")}
        </td>
        <td style="padding: 10px 14px; font-family: var(--ff-mono); color: var(--text-dim); font-size: 11px;">
          ${k(p.taskId)}
        </td>
        <td style="padding: 10px 14px; width: 140px;">
          <div style="display: flex; align-items: center; gap: 8px;">
            <div style="flex: 1; height: 6px; background: var(--surface-card); border-radius: 3px; overflow: hidden; border: 1px solid var(--border);">
              <div style="height: 100%; width: ${y}%; background: ${p.status==="completed"?"var(--green-400)":"var(--amber-400)"}; transition: width 0.3s;"></div>
            </div>
            <span style="font-size: 10.5px; font-family: var(--ff-mono); color: var(--text-muted);">${y}%</span>
          </div>
        </td>
        <td style="padding: 10px 14px; color: var(--text-dim); font-size: 11px; text-align: right;">
          ${v}
        </td>
        <td style="padding: 10px 14px; text-align: right;">
          ${p.status==="input_required"||p.status==="working"?`
            <button class="btn btn-danger" style="padding: 2px 8px; font-size: 10.5px;" onclick="window.app.promptCancelTask('${k(p.taskId)}')">Cancel</button>
          `:`
            <button class="btn btn-ghost" style="padding: 2px 8px; font-size: 10.5px;" onclick="window.app.inspectTaskDetails('${k(p.taskId)}')">Inspect</button>
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
        <div class="stat-value" style="color: var(--cyan-400);">${a.length}</div>
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
            <option value="all" ${r==="all"?"selected":""}>All Statuses</option>
            <option value="input_required" ${r==="input_required"?"selected":""}>input_required</option>
            <option value="working" ${r==="working"?"selected":""}>working</option>
            <option value="completed" ${r==="completed"?"selected":""}>completed</option>
            <option value="cancelled" ${r==="cancelled"?"selected":""}>cancelled</option>
            <option value="failed" ${r==="failed"?"selected":""}>failed</option>
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
            ${b}
          </tbody>
        </table>
      </div>
    </div>
  `}function k(e){return String(e||"").replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;")}function oe(){let e=d.getState(),t=e.auditEvents||[],r=e.auditStats||{total_events:0,by_status:{success:0,failed:0,denied:0,intercepted:0}},n=e.auditVerification,a=e.auditFilters,s=e.auditTotal??t.length,o=e.auditSelectedEvent,i=Object.keys(e.config?.mcpServers||{}),l=a.limit||25,c=a.offset||0,u=Math.floor(c/l)+1,g=Math.max(1,Math.ceil(s/l)),b=s===0?0:c+1,p=Math.min(c+l,s),m=f.getAuditExportUrl({actor_id:a.search?void 0:void 0,server_id:a.serverId!=="all"?a.serverId:void 0,event_type:a.eventType!=="all"?a.eventType:void 0,status:a.status!=="all"?a.status:void 0,search:a.search.trim()?a.search.trim():void 0},"csv"),y=f.getAuditExportUrl({server_id:a.serverId!=="all"?a.serverId:void 0,event_type:a.eventType!=="all"?a.eventType:void 0,status:a.status!=="all"?a.status:void 0,search:a.search.trim()?a.search.trim():void 0},"jsonl"),v=n?n.is_valid?`
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
  `,x=i.map((w)=>`<option value="${I(w)}" ${a.serverId===w?"selected":""}>${I(w)}</option>`).join(""),C=`
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
            value="${I(a.search)}"
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
            <option value="all" ${a.status==="all"?"selected":""}>All Statuses</option>
            <option value="success" ${a.status==="success"?"selected":""}>\uD83D\uDFE2 Success</option>
            <option value="denied" ${a.status==="denied"?"selected":""}>\uD83D\uDD34 Denied</option>
            <option value="intercepted" ${a.status==="intercepted"?"selected":""}>\uD83D\uDFE1 HITL Intercept</option>
            <option value="failed" ${a.status==="failed"?"selected":""}>❌ Failed</option>
            <option value="cancelled" ${a.status==="cancelled"?"selected":""}>⚪ Cancelled</option>
          </select>
        </div>

        <!-- Event Type Filter -->
        <div>
          <select 
            class="form-input" 
            style="width: 100%; font-size: 12px; height: 32px;"
            onchange="window.app.handleAuditEventTypeFilter(this.value)"
          >
            <option value="all" ${a.eventType==="all"?"selected":""}>All Event Types</option>
            <option value="tool_execution" ${a.eventType==="tool_execution"?"selected":""}>Tool Execution</option>
            <option value="tool_intercepted_hitl" ${a.eventType==="tool_intercepted_hitl"?"selected":""}>HITL Intercept</option>
            <option value="approval_granted" ${a.eventType==="approval_granted"?"selected":""}>Approval Granted</option>
            <option value="approval_rejected" ${a.eventType==="approval_rejected"?"selected":""}>Approval Rejected</option>
            <option value="approval_expired" ${a.eventType==="approval_expired"?"selected":""}>Approval Expired</option>
            <option value="policy_violation" ${a.eventType==="policy_violation"?"selected":""}>Policy Violation</option>
            <option value="config_mutation" ${a.eventType==="config_mutation"?"selected":""}>Config Mutation</option>
            <option value="sampling_call" ${a.eventType==="sampling_call"?"selected":""}>Sampling Call</option>
            <option value="resource_access" ${a.eventType==="resource_access"?"selected":""}>Resource Access</option>
          </select>
        </div>

        <!-- Server Filter -->
        <div>
          <select 
            class="form-input" 
            style="width: 100%; font-size: 12px; height: 32px;"
            onchange="window.app.handleAuditServerFilter(this.value)"
          >
            <option value="all" ${a.serverId==="all"?"selected":""}>All MCP Servers</option>
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
  `,P=`
    <div style="display: flex; justify-content: space-between; align-items: center; padding: 12px 16px; background: rgba(18, 24, 38, 0.5); border-radius: var(--radius-md); border: 1px solid var(--border); margin-top: 16px;">
      <div style="font-size: 12px; color: var(--text-dim); display: flex; align-items: center; gap: 8px;">
        <span>Showing <strong style="color: var(--text-main);">${b}–${p}</strong> of <strong style="color: var(--text-main);">${s}</strong> events</span>
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
  `,_="";if(t.length===0)_=`
      <div style="padding: 48px 24px; text-align: center; color: var(--text-dim); background: var(--surface-card); border-radius: var(--radius-md); border: 1px dashed var(--border);">
        <div style="font-size: 28px; margin-bottom: 8px;">\uD83D\uDD0D</div>
        <div style="font-size: 14px; font-weight: 600; color: var(--text-main); margin-bottom: 4px;">No Matching Audit Events</div>
        <div style="font-size: 12px; max-width: 420px; margin: 0 auto;">No audit records match your currently selected filters. Try broadening your search or resetting filters.</div>
        <button class="btn btn-ghost" style="margin-top: 14px; font-size: 11.5px;" onclick="window.app.clearAuditFilters()">Reset Filters</button>
      </div>
    `;else _=t.map((w)=>{let M=new Date(Math.floor(w.timestamp_ns/1e6)).toLocaleString(),B='<span class="badge" style="background: rgba(34, 197, 94, 0.15); color: var(--green-400); font-weight: 600;">SUCCESS</span>';if(w.status==="denied")B='<span class="badge" style="background: rgba(239, 68, 68, 0.15); color: var(--red-400); font-weight: 600;">DENIED</span>';else if(w.status==="intercepted")B='<span class="badge" style="background: rgba(234, 179, 8, 0.15); color: var(--amber-300); font-weight: 600;">HITL INTERCEPT</span>';else if(w.status==="failed")B='<span class="badge" style="background: rgba(239, 68, 68, 0.15); color: var(--red-400); font-weight: 600;">FAILED</span>';else if(w.status==="cancelled")B='<span class="badge" style="background: rgba(148, 163, 184, 0.15); color: var(--text-muted); font-weight: 600;">CANCELLED</span>';let U=w.sanitized_args?JSON.stringify(w.sanitized_args):"-",L=w.actor_id||w.operator_id||"anonymous",q=w.server_id||"system",Q=w.capability_id||w.event_type,E=w.execution_latency_us?`${(w.execution_latency_us/1000).toFixed(1)}ms`:"-";return`
        <div class="bento-card" style="margin-bottom: 12px; padding: 16px; border: 1px solid var(--border); transition: border-color 0.15s ease;">
          <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
            <div style="display: flex; align-items: center; gap: 8px;">
              <span style="font-family: var(--ff-mono); font-size: 11px; font-weight: 700; color: var(--text-dim);">${I(w.id)}</span>
              ${B}
              <span style="font-size: 12px; font-weight: 600; color: var(--text-main);">${I(Q)}</span>
            </div>
            <div style="display: flex; align-items: center; gap: 10px;">
              <div style="font-family: var(--ff-mono); font-size: 11px; color: var(--text-muted);">${I(M)}</div>
              <button 
                class="btn btn-ghost" 
                style="padding: 2px 8px; font-size: 11px; height: 24px;" 
                onclick="window.app.selectAuditEvent('${I(w.id)}')"
                title="Inspect event details & cryptographic payload"
              >
                Inspect \uD83D\uDD0D
              </button>
            </div>
          </div>
          
          <div style="display: grid; grid-template-columns: repeat(4, 1fr); gap: 8px; font-size: 11.5px; background: rgba(0,0,0,0.2); padding: 8px 12px; border-radius: var(--radius-sm); margin-bottom: 8px;">
            <div><span style="color: var(--text-muted);">Actor:</span> <strong style="color: var(--text-main);">${I(L)}</strong></div>
            <div><span style="color: var(--text-muted);">Server:</span> <strong style="color: var(--cyan-400);">${I(q)}</strong></div>
            <div><span style="color: var(--text-muted);">Trace:</span> <code style="color: var(--cyan-400); font-size: 10.5px;">${I(w.trace_id)}</code></div>
            <div><span style="color: var(--text-muted);">Latency:</span> <span style="color: var(--amber-300);">${E}</span></div>
          </div>

          <div style="font-family: var(--ff-mono); font-size: 11px; color: var(--text-dim); margin-bottom: 8px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">
            <span style="color: var(--text-muted);">Args:</span> ${I(U)}
          </div>

          <div style="display: flex; justify-content: space-between; align-items: center; border-top: 1px solid var(--border); padding-top: 6px; font-size: 10.5px; font-family: var(--ff-mono); color: var(--text-muted);">
            <div><span style="color: var(--text-dim);">prev_hash:</span> ${I(w.prev_hash.slice(0,16))}...</div>
            <div><span style="color: var(--text-dim);">hash:</span> <span style="color: var(--green-400);">${I(w.hash.slice(0,16))}...</span></div>
          </div>
        </div>
      `}).join("");let A="";if(o){let w=new Date(Math.floor(o.timestamp_ns/1e6)).toISOString();A=`
      <div style="position: fixed; inset: 0; background: rgba(0,0,0,0.7); backdrop-filter: blur(4px); z-index: 9999; display: flex; align-items: center; justify-content: center; padding: 24px;" onclick="if (event.target === this) window.app.selectAuditEvent(null)">
        <div class="bento-card" style="width: 100%; max-width: 720px; max-height: 85vh; display: flex; flex-direction: column; overflow: hidden; background: #0f172a; border: 1px solid var(--border); box-shadow: 0 20px 40px rgba(0,0,0,0.5);">
          <!-- Modal Header -->
          <div style="display: flex; justify-content: space-between; align-items: center; padding: 16px 20px; border-bottom: 1px solid var(--border);">
            <div style="display: flex; align-items: center; gap: 8px;">
              <span style="font-size: 16px;">\uD83D\uDD12</span>
              <h2 style="font-size: 15px; font-weight: 700; color: var(--text-main); margin: 0;">Audit Event Details (${I(o.id)})</h2>
            </div>
            <button class="btn btn-ghost" style="padding: 4px 8px; font-size: 14px;" onclick="window.app.selectAuditEvent(null)">✕</button>
          </div>

          <!-- Modal Body -->
          <div style="padding: 20px; overflow-y: auto; display: flex; flex-direction: column; gap: 14px; font-size: 12px;">
            <div style="display: grid; grid-template-columns: repeat(2, 1fr); gap: 10px; background: rgba(0,0,0,0.25); padding: 12px; border-radius: var(--radius-sm);">
              <div><span style="color: var(--text-muted);">Timestamp:</span> <strong style="color: var(--text-main); font-family: var(--ff-mono); font-size: 11px;">${I(w)}</strong></div>
              <div><span style="color: var(--text-muted);">Status:</span> <strong style="color: var(--text-main);">${I(o.status.toUpperCase())}</strong></div>
              <div><span style="color: var(--text-muted);">Event Type:</span> <strong style="color: var(--text-main);">${I(o.event_type)}</strong></div>
              <div><span style="color: var(--text-muted);">Server:</span> <strong style="color: var(--cyan-400);">${I(o.server_id||"system")}</strong></div>
              <div><span style="color: var(--text-muted);">Capability:</span> <strong style="color: var(--text-main);">${I(o.capability_id||"-")}</strong></div>
              <div><span style="color: var(--text-muted);">Actor / Operator:</span> <strong style="color: var(--text-main);">${I(o.actor_id||o.operator_id||"anonymous")}</strong></div>
              <div><span style="color: var(--text-muted);">Trace ID:</span> <code style="color: var(--cyan-400);">${I(o.trace_id)}</code></div>
              <div><span style="color: var(--text-muted);">Request ID:</span> <code style="color: var(--cyan-400);">${I(o.request_id||"-")}</code></div>
              <div><span style="color: var(--text-muted);">Client IP:</span> <span style="color: var(--text-main);">${I(o.client_ip||"-")}</span></div>
              <div><span style="color: var(--text-muted);">Latency:</span> <span style="color: var(--amber-300);">${o.execution_latency_us?`${(o.execution_latency_us/1000).toFixed(2)} ms`:"-"}</span></div>
            </div>

            ${o.error_message?`
              <div style="background: rgba(239, 68, 68, 0.1); border: 1px solid rgba(239, 68, 68, 0.3); border-radius: var(--radius-sm); padding: 10px 12px; color: var(--red-400);">
                <div style="font-weight: 700; margin-bottom: 2px;">Error (${I(o.error_code||"ERROR")}):</div>
                <div style="font-family: var(--ff-mono); font-size: 11px;">${I(o.error_message)}</div>
              </div>
            `:""}

            <!-- Sanitized Arguments -->
            <div>
              <div style="font-weight: 600; color: var(--text-main); margin-bottom: 4px;">Sanitized Arguments</div>
              <pre style="background: rgba(0,0,0,0.4); padding: 10px; border-radius: var(--radius-sm); border: 1px solid var(--border); font-family: var(--ff-mono); font-size: 11px; max-height: 140px; overflow: auto; margin: 0; color: #cbd5e1;">${I(JSON.stringify(o.sanitized_args||{},null,2))}</pre>
            </div>

            <!-- Sanitized Response -->
            ${o.sanitized_response?`
              <div>
                <div style="font-weight: 600; color: var(--text-main); margin-bottom: 4px;">Sanitized Response</div>
                <pre style="background: rgba(0,0,0,0.4); padding: 10px; border-radius: var(--radius-sm); border: 1px solid var(--border); font-family: var(--ff-mono); font-size: 11px; max-height: 140px; overflow: auto; margin: 0; color: #cbd5e1;">${I(JSON.stringify(o.sanitized_response,null,2))}</pre>
              </div>
            `:""}

            <!-- Cryptographic Hashes -->
            <div style="background: rgba(0,0,0,0.3); padding: 12px; border-radius: var(--radius-sm); border: 1px solid var(--border);">
              <div style="font-weight: 600; color: var(--text-main); margin-bottom: 6px;">Tamper-Evidence Cryptographic Hashes</div>
              <div style="margin-bottom: 6px;">
                <span style="color: var(--text-muted); font-size: 10.5px;">Previous Chain Hash (prev_hash):</span>
                <div style="font-family: var(--ff-mono); font-size: 10.5px; color: var(--text-dim); word-break: break-all;">${I(o.prev_hash)}</div>
              </div>
              <div>
                <span style="color: var(--text-muted); font-size: 10.5px;">Record Hash Signature (hash):</span>
                <div style="font-family: var(--ff-mono); font-size: 10.5px; color: var(--green-400); word-break: break-all;">${I(o.hash)}</div>
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
        <a href="${m}" download class="btn btn-ghost" style="font-size: 11.5px; text-decoration: none;" title="Export current filtered view as CSV">\uD83D\uDCE5 Export CSV</a>
        <a href="${y}" download class="btn btn-ghost" style="font-size: 11.5px; text-decoration: none;" title="Export current filtered view as JSONL">\uD83D\uDCE5 Export JSONL</a>
        <button class="btn btn-primary" style="font-size: 11.5px;" onclick="window.app.refreshAuditEvents()">\uD83D\uDD04 Refresh</button>
      </div>
    </div>

    <!-- Stats summary cards -->
    <div style="display: grid; grid-template-columns: repeat(4, 1fr); gap: 12px; margin-bottom: 20px;">
      <div class="bento-card" style="padding: 14px; text-align: center;">
        <div style="font-size: 11px; color: var(--text-muted); text-transform: uppercase; font-weight: 600;">Total Events</div>
        <div style="font-size: 22px; font-weight: 800; color: var(--text-main); margin-top: 4px;">${r.total_events}</div>
      </div>
      <div class="bento-card" style="padding: 14px; text-align: center;">
        <div style="font-size: 11px; color: var(--text-muted); text-transform: uppercase; font-weight: 600;">Successful Calls</div>
        <div style="font-size: 22px; font-weight: 800; color: var(--green-400); margin-top: 4px;">${r.by_status.success}</div>
      </div>
      <div class="bento-card" style="padding: 14px; text-align: center;">
        <div style="font-size: 11px; color: var(--text-muted); text-transform: uppercase; font-weight: 600;">HITL Intercepts</div>
        <div style="font-size: 22px; font-weight: 800; color: var(--amber-300); margin-top: 4px;">${r.by_status.intercepted}</div>
      </div>
      <div class="bento-card" style="padding: 14px; text-align: center;">
        <div style="font-size: 11px; color: var(--text-muted); text-transform: uppercase; font-weight: 600;">Policy Denials</div>
        <div style="font-size: 22px; font-weight: 800; color: var(--red-400); margin-top: 4px;">${r.by_status.denied}</div>
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
      ${_}
    </div>

    <!-- Pagination Footer -->
    ${s>0?P:""}

    <!-- Modal Popup for Event Inspection -->
    ${A}
  `}function I(e){if(!e)return"";return String(e).replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;").replace(/'/g,"&#039;")}function ne(){let e=d.getState(),t=e.activeProfile,r=t?e.config.profiles?.[t]:void 0,n=!!r,a=e.config.policy||{},s=r?.policy,o=n?s||{}:a,i=o.allow||[],l=o.deny||[],c=o.redact_keys||o.redactKeys||[],u=o.require_approval||o.requireApproval||[],g=i.length===0?`
    <div style="color: var(--text-dim); font-size: 12px;">${n?"No profile allow list (inherits global rules)":"No allow list (all non-denied operations permitted)"}</div>
  `:i.map((A,w)=>`
    <div style="display: flex; justify-content: space-between; align-items: center; background: var(--surface); padding: 8px 12px; border-radius: var(--radius-sm); border: 1px solid var(--border);">
      <span style="font-family: var(--ff-mono); font-size: 12px; color: var(--green-400);">✔ ${z(A)}</span>
      <button class="btn btn-ghost" style="padding: 2px 6px; font-size: 11px; color: var(--red-400);" onclick="window.app.removePolicyRule('allow', ${w})">✕</button>
    </div>
  `).join(""),b=l.length===0?`
    <div style="color: var(--text-dim); font-size: 12px;">${n?"No profile deny rules configured":"No deny rules configured"}</div>
  `:l.map((A,w)=>`
    <div style="display: flex; justify-content: space-between; align-items: center; background: var(--surface); padding: 8px 12px; border-radius: var(--radius-sm); border: 1px solid var(--border);">
      <span style="font-family: var(--ff-mono); font-size: 12px; color: var(--red-400);">✖ ${z(A)}</span>
      <button class="btn btn-ghost" style="padding: 2px 6px; font-size: 11px; color: var(--red-400);" onclick="window.app.removePolicyRule('deny', ${w})">✕</button>
    </div>
  `).join(""),p=u.length===0?`
    <div style="color: var(--text-dim); font-size: 12px;">${n?"No profile human-in-the-loop triggers configured":"No human-in-the-loop approval rules configured"}</div>
  `:u.map((A,w)=>`
    <div style="display: flex; justify-content: space-between; align-items: center; background: var(--surface); padding: 8px 12px; border-radius: var(--radius-sm); border: 1px solid var(--border);">
      <span style="font-family: var(--ff-mono); font-size: 12px; color: var(--amber-400);">\uD83D\uDEE1️ ${z(A)}</span>
      <button class="btn btn-ghost" style="padding: 2px 6px; font-size: 11px; color: var(--red-400);" onclick="window.app.removePolicyRule('requireApproval', ${w})">✕</button>
    </div>
  `).join(""),m=c.length===0?`
    <div style="color: var(--text-dim); font-size: 12px;">${n?"No profile key redaction patterns configured":"No key redaction patterns configured"}</div>
  `:c.map((A,w)=>`
    <span class="brand-badge" style="color: var(--amber-300); padding: 5px 10px; font-size: 11px; display: inline-flex; align-items: center; gap: 6px;">
      ${z(A)}
      <span style="cursor: pointer; color: var(--red-400); font-weight: bold;" onclick="window.app.removePolicyRule('redact', ${w})">✕</span>
    </span>
  `).join(""),y=n?`
    <div class="bento-card" style="margin-bottom: 16px; background: rgba(245, 158, 11, 0.05); border: 1px solid rgba(245, 158, 11, 0.3); display: flex; justify-content: space-between; align-items: center;">
      <div style="display: flex; align-items: center; gap: 10px;">
        <span style="font-size: 18px;">\uD83D\uDEE1️</span>
        <div>
          <div style="font-size: 13px; font-weight: 700; color: var(--amber-400);">
            Viewing &amp; Editing Policy for Profile Constellation: <code style="font-size: 13px; color: var(--text-main);">${z(t)}</code>
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
  `,v=Object.keys(e.config.mcpServers||{}),x=r?.servers||[],C=n?v.filter((A)=>!x.includes(A)):[],P=n?`
    <div class="bento-card" style="margin-bottom: 16px; border: 1px solid rgba(245, 158, 11, 0.2); background: rgba(0, 0, 0, 0.2);">
      <div class="stat-header" style="display: flex; justify-content: space-between; align-items: center;">
        <span class="stat-label" style="color: var(--amber-400);">Constellation Server Boundaries (Profile: ${z(t)})</span>
        <span style="font-size: 11px; color: var(--text-dim);">${x.length} of ${v.length} servers active</span>
      </div>
      <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 14px; margin-top: 10px;">
        <div style="background: var(--surface); padding: 10px; border-radius: var(--radius-sm); border: 1px solid var(--border);">
          <div style="font-size: 11px; font-weight: 600; color: var(--green-400); text-transform: uppercase; margin-bottom: 6px;">
            ✔ Included Servers (${x.length})
          </div>
          <div style="display: flex; flex-wrap: wrap; gap: 6px;">
            ${x.length>0?x.map((A)=>`
              <span class="brand-badge" style="color: var(--cyan-400); border-color: rgba(34, 211, 238, 0.25); background: rgba(34, 211, 238, 0.05);">
                ${z(A)}
              </span>
            `).join(""):'<span style="font-size: 11px; color: var(--text-dim);">No servers included</span>'}
          </div>
        </div>

        <div style="background: var(--surface); padding: 10px; border-radius: var(--radius-sm); border: 1px solid var(--border);">
          <div style="font-size: 11px; font-weight: 600; color: var(--amber-400); text-transform: uppercase; margin-bottom: 6px;">
            \uD83D\uDEAB Excluded Servers (${C.length}) · Implicitly Denied
          </div>
          <div style="display: flex; flex-wrap: wrap; gap: 6px; align-items: center;">
            ${C.length>0?C.map((A)=>`
              <span class="brand-badge" style="color: var(--text-muted); border-color: rgba(245, 158, 11, 0.2); background: rgba(245, 158, 11, 0.04); display: inline-flex; align-items: center; gap: 4px;">
                ${z(A)}
                <button style="background: none; border: none; color: var(--amber-400); font-size: 10px; cursor: pointer; padding: 0 2px;" title="Include in profile" onclick="window.app.toggleServerInProfile('${z(t)}', '${z(A)}', true)">+</button>
              </span>
            `).join(""):'<span style="font-size: 11px; color: var(--text-dim);">All servers included in constellation</span>'}
          </div>
        </div>
      </div>
    </div>
  `:"",_=n&&C.length>0?`
    <div style="border-top: 1px dashed var(--border); padding-top: 8px; margin-top: 8px;">
      <div style="font-size: 10.5px; color: var(--text-dim); text-transform: uppercase; font-weight: 600; margin-bottom: 6px;">
        Implicit Boundary Denials (${C.length})
      </div>
      <div style="display: flex; flex-direction: column; gap: 4px;">
        ${C.map((A)=>`
          <div style="display: flex; justify-content: space-between; align-items: center; background: rgba(245, 158, 11, 0.03); padding: 5px 8px; border-radius: var(--radius-xs); border: 1px dashed rgba(245, 158, 11, 0.2);">
            <span style="font-family: var(--ff-mono); font-size: 11px; color: var(--text-dim);">✖ ${z(A)}.*</span>
            <span style="font-size: 9.5px; color: var(--amber-400); font-family: var(--ff-mono);">server excluded</span>
          </div>
        `).join("")}
      </div>
    </div>
  `:"";return`
    ${y}

    ${P}

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
          ${b}
          ${_}
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
            <input type="text" class="form-input" id="policy-webhook-url" placeholder="https://hooks.slack.com/services/... or Discord webhook URL" value="${z(typeof o.webhook==="object"&&o.webhook?o.webhook.url||"":"")}">
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
            <input type="text" class="form-input" id="policy-webhook-secret" placeholder="e.g. WARMPLANE_WEBHOOK_SECRET" value="${z(typeof o.webhook==="object"&&o.webhook?o.webhook.secret_env||o.webhook.secretEnv||o.webhook.secret||"":"")}">
          </div>
        </div>

        <div style="display: flex; justify-content: space-between; align-items: center; margin-top: 14px; padding-top: 12px; border-top: 1px solid var(--border-subtle);">
          <div style="display: flex; gap: 8px; align-items: center;">
            <button class="btn btn-primary" onclick="window.app.saveWebhookConfig()">Save Webhook Settings</button>
            <button class="btn btn-ghost" onclick="window.app.testWebhook()">⚡ Send Test Event</button>
          </div>
          <div id="policy-webhook-status" style="font-size: 11px; font-family: var(--ff-mono); color: var(--text-dim);">
            ${typeof o.webhook==="object"&&o.webhook?.url?`Active Target: ${z(o.webhook.url)}`:"No webhook configured"}
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
  `}function z(e){return String(e||"").replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;")}function ie(){let e=d.getState(),t=e.config,r=Object.entries(t.capabilityAliases||{}),n=Object.entries(t.resourceAliases||{}),a=Object.entries(t.promptAliases||{}),s="";if(r.length===0&&n.length===0&&a.length===0)s=`
      <div style="padding: 24px; text-align: center; color: var(--text-dim);">
        No facade aliases configured in ${H(e.configPath)}. Add short names to prune token payload sizes.
      </div>
    `;else{for(let[o,i]of r)s+=`
        <div class="feed-row" style="grid-template-columns: 90px 180px 1fr 80px;">
          <span style="color: var(--cyan-400);">Tool</span>
          <span style="font-weight: 700; color: var(--text-main); font-family: var(--ff-mono);">${H(o)}</span>
          <span style="color: var(--text-dim); font-family: var(--ff-mono);">${H(i)}</span>
          <div style="text-align: right;">
            <button class="btn btn-ghost" style="padding: 2px 6px; color: var(--red-400);" onclick="window.app.deleteAlias('tool', '${H(o)}')">✕</button>
          </div>
        </div>
      `;for(let[o,i]of n)s+=`
        <div class="feed-row" style="grid-template-columns: 90px 180px 1fr 80px;">
          <span style="color: var(--green-400);">Resource</span>
          <span style="font-weight: 700; color: var(--text-main); font-family: var(--ff-mono);">${H(o)}</span>
          <span style="color: var(--text-dim); font-family: var(--ff-mono);">${H(i)}</span>
          <div style="text-align: right;">
            <button class="btn btn-ghost" style="padding: 2px 6px; color: var(--red-400);" onclick="window.app.deleteAlias('resource', '${H(o)}')">✕</button>
          </div>
        </div>
      `;for(let[o,i]of a)s+=`
        <div class="feed-row" style="grid-template-columns: 90px 180px 1fr 80px;">
          <span style="color: var(--amber-300);">Prompt</span>
          <span style="font-weight: 700; color: var(--text-main); font-family: var(--ff-mono);">${H(o)}</span>
          <span style="color: var(--text-dim); font-family: var(--ff-mono);">${H(i)}</span>
          <div style="text-align: right;">
            <button class="btn btn-ghost" style="padding: 2px 6px; color: var(--red-400);" onclick="window.app.deleteAlias('prompt', '${H(o)}')">✕</button>
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
  `}function H(e){return String(e||"").replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;")}function le(){let e=d.getState(),t=e.config,r=t.profiles||{},n=Object.entries(r),a=t.mcpServers||{},s=e.activeProfile,o="";if(n.length===0)o=`
      <div style="padding: 40px; text-align: center; color: var(--text-dim); background: var(--surface-card); border-radius: var(--radius-md); border: 1px dashed var(--border);">
        <div style="font-size: 15px; color: var(--text-main); font-weight: 600; margin-bottom: 8px;">No Profiles Configured</div>
        <p style="font-size: 12px; margin-bottom: 20px; max-width: 480px; margin-left: auto; margin-right: auto;">
          Profiles allow Warmplane to serve multiple task-relevant server constellations (e.g. <code>coding</code>, <code>support</code>, <code>data</code>) from one running daemon process.
        </p>
        <button class="btn btn-primary" onclick="window.app.openAddProfileModal()">+ Create First Profile</button>
      </div>
    `;else o=n.map(([i,l])=>{let c=s===i,u=l.servers.map((x)=>`<span class="brand-badge" style="${a[x]?"color: var(--cyan-400); border-color: rgba(34, 211, 238, 0.25); background: rgba(34, 211, 238, 0.05);":"color: var(--red-400); border-color: rgba(248, 113, 113, 0.3); background: rgba(248, 113, 113, 0.05);"}">${D(x)}</span>`).join(" "),g=(e.capabilities||[]).filter((x)=>l.servers.includes(x.server)).length,b=!!l.policy,p=l.policy?.allow?.length||0,m=l.policy?.deny?.length||0,y=(l.policy?.require_approval||l.policy?.requireApproval||[]).length,v=(l.policy?.redact_keys||l.policy?.redactKeys||[]).length;return`
        <div class="bento-card" style="margin-bottom: 14px; border-left: ${c?"3px solid var(--amber-400)":"1px solid var(--border)"};">
          <div style="display: flex; justify-content: space-between; align-items: flex-start;">
            <div>
              <div style="display: flex; align-items: center; gap: 8px; margin-bottom: 6px;">
                <span style="font-size: 16px; font-weight: 700; color: var(--text-main); font-family: var(--ff-mono);">${D(i)}</span>
                ${c?'<span class="brand-badge" style="color: var(--amber-400); border-color: rgba(245, 158, 11, 0.4); background: rgba(245, 158, 11, 0.1);">ACTIVE IN UI</span>':""}
                <span class="brand-badge">${l.servers.length} server${l.servers.length===1?"":"s"}</span>
                <span class="brand-badge" style="color: var(--text-dim);">${g} capabilities</span>
                ${b?'<span class="brand-badge" style="color: var(--green-400); border-color: rgba(34, 197, 94, 0.3); background: rgba(34, 197, 94, 0.08);">CUSTOM POLICY</span>':""}
              </div>
              <div style="font-size: 12px; color: var(--text-muted); margin-bottom: 10px;">
                ${D(l.description||"No description provided")}
              </div>
              <div style="display: flex; flex-wrap: wrap; gap: 6px; align-items: center; margin-bottom: ${b?"8px":"0"};">
                <span style="font-size: 11px; color: var(--text-dim); font-weight: 600; text-transform: uppercase;">Servers:</span>
                ${u||'<span style="font-size: 11px; color: var(--text-dim);">None</span>'}
              </div>
              ${b?`
                <div style="display: flex; flex-wrap: wrap; gap: 6px; align-items: center; font-size: 11px;">
                  <span style="color: var(--text-dim); font-weight: 600; text-transform: uppercase;">Policy Overlay:</span>
                  ${p>0?`<span class="brand-badge" style="color: var(--green-400);">Allow: ${p}</span>`:""}
                  ${m>0?`<span class="brand-badge" style="color: var(--red-400);">Deny: ${m}</span>`:""}
                  ${y>0?`<span class="brand-badge" style="color: var(--amber-400);">HITL: ${y}</span>`:""}
                  ${v>0?`<span class="brand-badge" style="color: var(--text-muted);">Redact: ${v}</span>`:""}
                  ${p===0&&m===0&&y===0&&v===0?'<span style="color: var(--text-dim);">Configured</span>':""}
                </div>
              `:""}
            </div>
            
            <div style="display: flex; gap: 8px; align-items: center;">
              ${c?`
                <button class="btn btn-ghost" style="padding: 4px 10px; font-size: 11.5px;" onclick="window.app.setActiveProfile(null)">
                  Deselect
                </button>
              `:`
                <button class="btn btn-primary" style="padding: 4px 10px; font-size: 11.5px;" onclick="window.app.setActiveProfile('${D(i)}')">
                  Activate in UI
                </button>
              `}
              <button class="btn btn-ghost" style="padding: 4px 10px; font-size: 11.5px;" onclick="window.app.openEditProfileModal('${D(i)}')">
                ✏️ Edit
              </button>
              <button class="btn btn-danger" style="padding: 4px 10px; font-size: 11.5px;" onclick="window.app.deleteProfile('${D(i)}')">
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
  `}function D(e){return String(e||"").replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;")}function de(){let t=d.getState().secrets||[],r=t.length,n=t.filter((o)=>o.is_vault).length,a=r-n,s=t.length===0?`
    <div style="padding: 32px; text-align: center; color: var(--text-dim);">
      No environment variables or secrets configured in active servers.
    </div>
  `:t.map((o)=>{let i='<span class="brand-badge" style="color: var(--red-400); border-color: rgba(248, 113, 113, 0.4); background: rgba(248, 113, 113, 0.1);">Plaintext (Unsecured)</span>';if(o.is_vault)i=`<span class="brand-badge" style="color: var(--green-400); border-color: rgba(52, 211, 153, 0.3); background: rgba(52, 211, 153, 0.1);">\uD83D\uDD12 ${N(o.backend)}</span>`;return`
      <div style="display: grid; grid-template-columns: 140px 180px 1fr 180px auto; padding: 10px 16px; border-bottom: 1px solid var(--border-subtle); align-items: center; font-size: 12px;">
        <span style="font-weight: 700; color: var(--text-main);">${N(o.server)}</span>
        <span style="font-family: var(--ff-mono); color: var(--amber-300);">${N(o.key)}</span>
        <span style="font-family: var(--ff-mono); font-size: 11px; color: var(--text-muted); overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">${N(o.display)}</span>
        <div>${i}</div>
        <div style="display: flex; gap: 6px; justify-content: flex-end;">
          ${!o.is_vault?`
            <button class="btn btn-primary" style="padding: 2px 8px; font-size: 11px;" onclick="window.app.quickVaultEnv('${N(o.server)}', '${N(o.key)}')">\uD83D\uDD12 Move to Keychain</button>
          `:`
            <button class="btn btn-ghost" style="padding: 2px 8px; font-size: 11px; color: var(--red-400);" onclick="window.app.deleteVaultSecret('${N(o.key)}')">Delete Key</button>
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
        <div class="stat-value" style="color: var(--cyan-400);">${r}</div>
        <div class="stat-sub">Across all configured MCP servers</div>
      </div>
      <div class="bento-card col-4">
        <div class="stat-label">Secured via Vault / Keychain</div>
        <div class="stat-value" style="color: var(--green-400);">${n}</div>
        <div class="stat-sub">Zero-disk plaintext exposure</div>
      </div>
      <div class="bento-card col-4">
        <div class="stat-label">Plaintext Secrets</div>
        <div class="stat-value" style="color: ${a>0?"var(--red-400)":"var(--green-400)"};">${a}</div>
        <div class="stat-sub">${a>0?"Recommend migrating to Keychain":"All credentials protected"}</div>
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
  `}function N(e){return String(e||"").replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;")}var X=[{id:"github",name:"GitHub",category:"devtools",description:"Explore repositories, issues, pull requests, branches, and commit histories.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-github"],envFields:[{key:"GITHUB_PERSONAL_ACCESS_TOKEN",label:"GitHub Personal Access Token",placeholder:"ghp_...",required:!0,description:"Classic or fine-grained token with repo scope."}]},{id:"git",name:"Git (Local)",category:"devtools",description:"Read local Git repository status, diffs, log histories, and commit changes.",badge:"Official / uvx",command:"uvx",defaultArgs:["--with","mcp<2","mcp-server-git","--repository","."],argsPlaceholder:"--with mcp<2 mcp-server-git --repository /path/to/repo",envFields:[]},{id:"filesystem",name:"Filesystem",category:"devtools",description:"Secure, sandboxed access to local files and directories for AI workflows.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-filesystem","."],argsPlaceholder:"-y @modelcontextprotocol/server-filesystem /allowed/dir1 /allowed/dir2",envFields:[]},{id:"memory",name:"Memory Graph",category:"devtools",description:"Persistent knowledge-graph based memory for multi-turn agent learning.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-memory"],envFields:[]},{id:"chrome-devtools",name:"Chrome DevTools",category:"devtools",description:"Inspect live DOM, execute scripts, read console logs, and capture network traces in Chrome.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-puppeteer"],envFields:[]},{id:"sentry",name:"Sentry",category:"devtools",description:"Query production error events, stack traces, and issue frequencies directly from Sentry.",badge:"uvx / Telemetry",command:"uvx",defaultArgs:["--with","mcp<2","--with","httpx","mcp-server-sentry","--auth-token","sntrys_token"],argsPlaceholder:"--with mcp<2 --with httpx mcp-server-sentry --auth-token YOUR_SENTRY_TOKEN",envFields:[{key:"SENTRY_AUTH_TOKEN",label:"Sentry Auth Token",placeholder:"sntrys_...",required:!0}]},{id:"playwright",name:"Playwright Browser",category:"browser",description:"Headless / headed browser automation for scraping, form filling, and UI interaction.",badge:"Popular #1 / npx",command:"npx",defaultArgs:["-y","@executeautomation/playwright-mcp-server"],envFields:[]},{id:"puppeteer",name:"Puppeteer",category:"browser",description:"Official browser automation server for web page scraping and screenshot capture.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-puppeteer"],envFields:[]},{id:"brave-search",name:"Brave Search",category:"browser",description:"Real-time privacy-preserving web search and local point-of-interest query engine.",badge:"Official / Search",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-brave-search"],envFields:[{key:"BRAVE_API_KEY",label:"Brave Search API Key",placeholder:"BSA...",required:!0}]},{id:"tavily",name:"Tavily Search",category:"browser",description:"AI-optimized web search engine structured specifically for LLM context injection.",badge:"Community / Stdio",command:"npx",defaultArgs:["-y","tavily-mcp"],envFields:[{key:"TAVILY_API_KEY",label:"Tavily API Key",placeholder:"tvly-...",required:!0}]},{id:"fetch",name:"Fetch / Web Markdown",category:"browser",description:"Download web pages, strip clutter, and convert raw HTML to clean markdown text.",badge:"Official / uvx",command:"uvx",defaultArgs:["mcp-server-fetch"],envFields:[]},{id:"postgres",name:"PostgreSQL",category:"database",description:"Read schemas, inspect tables, and execute SQL queries against PostgreSQL databases.",badge:"Official / Database",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-postgres","postgresql://user:pass@localhost:5432/mydb"],argsPlaceholder:"-y @modelcontextprotocol/server-postgres postgresql://user:pass@localhost:5432/dbname",envFields:[]},{id:"sqlite",name:"SQLite",category:"database",description:"Local embedded SQLite query runner and schema inspector.",badge:"Official / uvx",command:"uvx",defaultArgs:["--with","mcp<2","mcp-server-sqlite","--db-path","./app.db"],argsPlaceholder:"--with mcp<2 mcp-server-sqlite --db-path /path/to/database.sqlite",envFields:[]},{id:"supabase",name:"Supabase",category:"database",description:"Query database tables, manage auth policies, and inspect storage in Supabase.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@supabase/mcp-server-supabase@latest"],envFields:[{key:"SUPABASE_ACCESS_TOKEN",label:"Supabase Personal Access Token",placeholder:"sbp_...",required:!0},{key:"SUPABASE_PROJECT_REF",label:"Supabase Project Reference ID",placeholder:"abcdefghijklmnop",required:!1}]},{id:"redis",name:"Redis",category:"database",description:"Inspect cached keys, hash sets, lists, TTLs, and pub/sub channels in Redis.",badge:"Official / Key-Value",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-redis","redis://localhost:6379"],argsPlaceholder:"-y @modelcontextprotocol/server-redis redis://localhost:6379",envFields:[]},{id:"s3",name:"AWS S3 / Cloud Storage",category:"database",description:"Browse S3 buckets, fetch object metadata, and download files from cloud storage.",badge:"Community / Stdio",command:"npx",defaultArgs:["-y","@geunoh/s3-mcp-server"],argsPlaceholder:"-y @geunoh/s3-mcp-server",envFields:[{key:"AWS_ACCESS_KEY_ID",label:"AWS Access Key ID",placeholder:"AKIA...",required:!0},{key:"AWS_SECRET_ACCESS_KEY",label:"AWS Secret Access Key",placeholder:"...",required:!0},{key:"AWS_REGION",label:"AWS Region",placeholder:"us-east-1",required:!1}]},{id:"linear",name:"Linear",category:"productivity",description:"Search, create, and triage Linear issues, cycles, teams, and project roadmaps.",badge:"Productivity / Stdio",command:"npx",defaultArgs:["-y","mcp-linear"],envFields:[{key:"LINEAR_API_KEY",label:"Linear API Key",placeholder:"lin_api_...",required:!0}]},{id:"slack",name:"Slack",category:"productivity",description:"Read channels, post messages, inspect threads, and search team discussions.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-slack"],envFields:[{key:"SLACK_BOT_TOKEN",label:"Slack Bot User Token",placeholder:"xoxb-...",required:!0},{key:"SLACK_TEAM_ID",label:"Slack Team ID",placeholder:"T01234567",required:!0}]},{id:"notion",name:"Notion",category:"productivity",description:"Search Notion workspace pages, read nested blocks, and query database entries.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@notionhq/notion-mcp-server"],envFields:[{key:"NOTION_TOKEN",label:"Notion Internal Integration Token",placeholder:"secret_...",required:!0}]},{id:"jira",name:"Jira / Atlassian",category:"productivity",description:"Manage Jira issues, search JQL, read sprint statuses, and inspect boards.",badge:"uvx / Atlassian",command:"uvx",defaultArgs:["--with","mcp<2","mcp-server-jira","--jira-base-url","https://your-domain.atlassian.net"],argsPlaceholder:"--with mcp<2 mcp-server-jira --jira-base-url https://org.atlassian.net",envFields:[{key:"JIRA_TOKEN",label:"Atlassian API Token",placeholder:"ATATT3...",required:!0}]},{id:"google-drive",name:"Google Drive",category:"productivity",description:"Search, list, and read documents, spreadsheets, and drive files.",badge:"Community / Stdio",command:"npx",defaultArgs:["-y","@piotr-agier/google-drive-mcp"],envFields:[{key:"GOOGLE_APPLICATION_CREDENTIALS",label:"Google Credentials JSON Path",placeholder:"/path/to/credentials.json",required:!0}]},{id:"docker",name:"Docker",category:"cloud",description:"Inspect running containers, tail container logs, list images, and manage compose services.",badge:"uvx / DevOps",command:"uvx",defaultArgs:["mcp-server-docker"],envFields:[]},{id:"kubernetes",name:"Kubernetes (K8s)",category:"cloud",description:"Query cluster pods, services, deployment status, and inspect Kubernetes logs.",badge:"Popular / Stdio",command:"npx",defaultArgs:["-y","@strowk/mcp-k8s"],envFields:[{key:"KUBECONFIG",label:"Kubeconfig File Path (Optional)",placeholder:"~/.kube/config",required:!1}]},{id:"cloudflare",name:"Cloudflare",category:"cloud",description:"Manage Cloudflare Workers, KV namespaces, D1 databases, Vectorize indexes, and DNS.",badge:"Official / Cloudflare",command:"npx",defaultArgs:["-y","@cloudflare/mcp-server-cloudflare","run","dummy_account_id"],argsPlaceholder:"-y @cloudflare/mcp-server-cloudflare run YOUR_ACCOUNT_ID",envFields:[{key:"CLOUDFLARE_API_TOKEN",label:"Cloudflare API Token",placeholder:"...",required:!0},{key:"CLOUDFLARE_ACCOUNT_ID",label:"Cloudflare Account ID",placeholder:"...",required:!0}]},{id:"terraform",name:"Terraform",category:"cloud",description:"Inspect Terraform state files, resource dependency graphs, and plan previews.",badge:"Community / IaC",command:"npx",defaultArgs:["-y","@mseep/terraform-mcp-server"],envFields:[]}];class ce{activeTemplateCategory="all";activeTemplateFilter="";selectedTemplate=null;async init(){let e=window.location.port?`:${window.location.port}`:"",t=document.getElementById("daemon-port-label");if(t)t.textContent=`Daemon ${e}`;await this.refreshData(),this.initSSE(),this.render(),d.subscribe(()=>{this.render()})}auditSearchTimeout=null;async refreshData(){try{let e=d.getState(),t=e.auditFilters,r=e.activeProfile||void 0,[n,a,s,o,i,l,c,u,g,b,p]=await Promise.all([f.getConfig(),f.listCapabilities(r),f.listResources(r),f.listPrompts(r),f.getCatalogEvents(),f.listApprovals(),f.listTasks(),f.listAuditEvents({server_id:t.serverId!=="all"?t.serverId:void 0,event_type:t.eventType!=="all"?t.eventType:void 0,status:t.status!=="all"?t.status:void 0,search:t.search.trim()?t.search.trim():void 0,limit:t.limit,offset:t.offset}),f.getAuditStats(),f.getClients().catch(()=>({ok:!1,clients:[]})),f.getSecrets().catch(()=>({ok:!1,secrets:[],keychain_service:"warmplane"}))]);if(b&&b.ok&&Array.isArray(b.clients))d.setState({clients:b.clients});if(p&&p.ok&&Array.isArray(p.secrets))d.setState({secrets:p.secrets});if(n.ok)d.setState({configPath:n.config_path,config:n.config,serverStatuses:n.server_statuses||{},circuitBreakers:n.circuit_breakers||[],metrics:{totalCatalogRequests:n.metrics?.total_catalog_requests||0,totalEtagHits:n.metrics?.total_etag_hits||0,totalToolCalls:n.metrics?.total_tool_calls||0,totalToolDurationUs:n.metrics?.total_tool_duration_us||0}});if(a&&Array.isArray(a.capabilities)){let m=d.getState().selectedCapabilityId,v=a.capabilities.some((x)=>x.id===m)?m:a.capabilities.length>0?a.capabilities[0].id:null;d.setState({capabilities:a.capabilities,capabilitiesHiddenByPolicy:a.hidden_by_policy||0,selectedCapabilityId:v})}if(s&&Array.isArray(s.resources)){let m=d.getState().selectedResourceId,v=s.resources.some((x)=>x.uri===m||x.id===m)?m:s.resources.length>0?s.resources[0].uri||s.resources[0].id||null:null;d.setState({resources:s.resources,resourcesHiddenByPolicy:s.hidden_by_policy||0,selectedResourceId:v})}if(o&&Array.isArray(o.prompts)){let m=d.getState().selectedPromptId,v=o.prompts.some((x)=>x.name===m||x.id===m)?m:o.prompts.length>0?o.prompts[0].name||o.prompts[0].id||null:null;d.setState({prompts:o.prompts,promptsHiddenByPolicy:o.hidden_by_policy||0,selectedPromptId:v})}if(i&&Array.isArray(i.events))d.setState({catalogEvents:i.events});if(l&&Array.isArray(l.approvals))d.setState({approvals:l.approvals});if(c&&Array.isArray(c.tasks))d.setState({tasks:c.tasks});if(u&&Array.isArray(u.events))d.setState({auditEvents:u.events,auditTotal:u.total??u.events.length});if(g&&g.ok)d.setState({auditStats:g})}catch(e){console.error("Failed to fetch daemon state:",e)}}async refreshAuditEvents(){try{let t=d.getState().auditFilters,[r,n]=await Promise.all([f.listAuditEvents({server_id:t.serverId!=="all"?t.serverId:void 0,event_type:t.eventType!=="all"?t.eventType:void 0,status:t.status!=="all"?t.status:void 0,search:t.search.trim()?t.search.trim():void 0,limit:t.limit,offset:t.offset}),f.getAuditStats()]);if(r&&Array.isArray(r.events))d.setState({auditEvents:r.events,auditTotal:r.total??r.events.length});if(n&&n.ok)d.setState({auditStats:n})}catch(e){console.error("Failed to refresh audit events:",e)}}handleAuditSearchInput(e){let r={...d.getState().auditFilters,search:e,offset:0};d.setState({auditFilters:r}),clearTimeout(this.auditSearchTimeout),this.auditSearchTimeout=setTimeout(()=>{this.refreshAuditEvents()},250)}handleAuditStatusFilter(e){let t=d.getState();d.setState({auditFilters:{...t.auditFilters,status:e,offset:0}}),this.refreshAuditEvents()}handleAuditEventTypeFilter(e){let t=d.getState();d.setState({auditFilters:{...t.auditFilters,eventType:e,offset:0}}),this.refreshAuditEvents()}handleAuditServerFilter(e){let t=d.getState();d.setState({auditFilters:{...t.auditFilters,serverId:e,offset:0}}),this.refreshAuditEvents()}handleAuditPageSize(e){let t=parseInt(e,10)||25,r=d.getState();d.setState({auditFilters:{...r.auditFilters,limit:t,offset:0}}),this.refreshAuditEvents()}clearAuditFilters(){let e=d.getState();d.setState({auditFilters:{search:"",status:"all",eventType:"all",serverId:"all",limit:e.auditFilters.limit||25,offset:0}}),this.refreshAuditEvents()}auditPrevPage(){let e=d.getState(),{limit:t,offset:r}=e.auditFilters,n=Math.max(0,r-t);if(n!==r)d.setState({auditFilters:{...e.auditFilters,offset:n}}),this.refreshAuditEvents()}auditNextPage(){let e=d.getState(),{limit:t,offset:r}=e.auditFilters,n=e.auditTotal;if(r+t<n)d.setState({auditFilters:{...e.auditFilters,offset:r+t}}),this.refreshAuditEvents()}auditGoToPage(e){let t=d.getState(),{limit:r}=t.auditFilters,n=Math.max(0,(e-1)*r);d.setState({auditFilters:{...t.auditFilters,offset:n}}),this.refreshAuditEvents()}selectAuditEvent(e){if(!e){d.setState({auditSelectedEvent:null});return}let r=d.getState().auditEvents.find((n)=>n.id===e)||null;d.setState({auditSelectedEvent:r})}async verifyAuditChain(){try{let e=await f.verifyAuditChain();if(e&&e.report)d.setState({auditVerification:e.report})}catch(e){console.error("Failed to verify audit chain:",e)}}async refreshApprovals(){try{let e=await f.listApprovals();if(e&&Array.isArray(e.approvals))d.setState({approvals:e.approvals})}catch(e){console.error("Failed to refresh approvals:",e)}}initSSE(){try{let e=new EventSource("/v1/resources/updates");e.onmessage=(t)=>{d.addEventLog("SSE","/v1/resources/updates","UPDATED","0.1ms"),this.refreshData()}}catch(e){console.warn("SSE connection unavailable")}}switchTab(e){d.setState({activeTab:e}),this.refreshData()}render(){let e=d.getState(),t=document.getElementById("app-main");if(!t)return;let r=(e.tasks||[]).filter((l)=>l.status==="input_required").length,n=(e.approvals||[]).filter((l)=>l.status==="pending").length,a=Math.max(r,n),s=document.getElementById("nav-approvals-badge");if(s)s.textContent=a>0?`${a}`:"",s.style.display=a>0?"inline-block":"none";document.querySelectorAll(".nav-item").forEach((l)=>{let c=l.getAttribute("data-tab");if(c===e.activeTab||e.activeTab==="tasks"&&c==="approvals"||e.activeTab==="approvals"&&c==="tasks")l.classList.add("active");else l.classList.remove("active")});let o=document.getElementById("top-title"),i={overview:"Overview Cockpit",servers:"Server Hub & Connections",playground:"MCP Capability Playground",tasks:"SEP-2663 Tasks & HITL Review",approvals:"SEP-2663 Tasks & HITL Review",audit:"WORM Audit & Compliance Ledger",policy:"Security Governance & Redaction",secrets:"Native OS Keychain & Secrets Vault",aliases:"Facade & Alias Studio",profiles:"Server Constellation Profiles"};if(o)o.textContent=i[e.activeTab]||"Control Deck";switch(this.renderTopProfileSelector(),e.activeTab){case"overview":t.innerHTML=te();break;case"servers":t.innerHTML=se();break;case"playground":t.innerHTML=re();break;case"tasks":case"approvals":t.innerHTML=ae(e);break;case"audit":t.innerHTML=oe();break;case"policy":t.innerHTML=ne();break;case"secrets":t.innerHTML=de();break;case"aliases":t.innerHTML=ie();break;case"profiles":t.innerHTML=le();break}}toggleClientsCollapse(){let e=d.getState().clientsCollapsed;d.setState({clientsCollapsed:!e}),this.render()}async saveNewVaultSecret(){let e=document.getElementById("vault-new-key"),t=document.getElementById("vault-new-val"),r=document.getElementById("vault-new-service"),n=e?.value.trim(),a=t?.value.trim(),s=r?.value.trim()||"warmplane";if(!n||!a){alert("Key and secret value are required");return}try{let o=await f.saveSecret(n,a,s);if(o.ok){if(alert(`Secret '${n}' saved securely into OS Keychain!
Reference: ${o.uri}`),e)e.value="";if(t)t.value="";await this.refreshData()}else alert(`Failed to save secret: ${o.error}`)}catch(o){alert(`Error saving secret: ${o.message}`)}}async deleteVaultSecret(e){if(!confirm(`Are you sure you want to remove secret '${e}' from OS Keychain?`))return;try{let t=await f.deleteSecret(e);if(t.ok)await this.refreshData();else alert(`Failed to delete secret: ${t.error}`)}catch(t){alert(`Error deleting secret: ${t.message}`)}}async quickVaultEnv(e,t){let r=prompt(`Enter secret value to store in OS Keychain for ${e}.${t}:`);if(!r)return;try{let n=await f.saveSecret(t,r,"warmplane");if(!n.ok){alert(`Failed to save to Keychain: ${n.error}`);return}let o=(d.getState().config.mcpServers||{})[e];if(o){let i={...o.env||{},[t]:`keychain://warmplane/${t}`},l={...o,env:i},c=await f.upsertServer(e,l);if(c.ok)await this.refreshData(),alert(`Successfully migrated ${e}.${t} to OS Keychain!`);else alert(`Failed to update server config: ${c.error}`)}}catch(n){alert(`Error during migration: ${n.message}`)}}async refreshTasks(){try{let e=await f.listTasks();if(e&&Array.isArray(e.tasks))d.setState({tasks:e.tasks})}catch(e){console.error("Failed to refresh tasks:",e)}}filterTasksByStatus(e){d.setState({taskFilterStatus:e})}togglePlaygroundAsyncTask(e){d.setState({playgroundAsyncTask:e})}async submitTaskInputResponses(e){let r=d.getState().tasks.find((s)=>s.taskId===e)?.inputRequests||{},n=Object.keys(r),a={};if(n.length>0)for(let s of n){let o=r[s];if(o&&o.type==="approval_review"){let i=document.getElementById(`task-input-${e}-${s}-decision`),l=document.getElementById(`task-input-${e}-${s}`),c=i?i.value==="true":!0,u=void 0;if(l&&l.value.trim())try{u=JSON.parse(l.value.trim())}catch{alert("Invalid JSON in parameters editor");return}a[s]={approved:c,modified_args:u,reason:c?void 0:"Operator rejected execution via Tasks review"}}else{let i=document.getElementById(`task-input-${e}-${s}`);if(i){let l=i.value.trim();try{a[s]=JSON.parse(l)}catch{a[s]=l}}}}else{let s=document.getElementById(`task-raw-input-${e}`);if(s&&s.value.trim())try{Object.assign(a,JSON.parse(s.value.trim()))}catch{alert("Invalid JSON in raw input responses");return}}try{let s=await f.updateTask(e,a);if(s.ok)await this.refreshTasks();else alert(`Task update failed: ${s.error?.message||s.error||"Unknown error"}`)}catch(s){alert(`Error updating task: ${s.message}`)}}async promptCancelTask(e){let t=prompt("Reason for cancelling task:");if(t===null)return;try{let r=await f.cancelTask(e,t||void 0);if(r.ok)await this.refreshTasks();else alert(`Task cancellation failed: ${r.error?.message||r.error||"Unknown error"}`)}catch(r){alert(`Error cancelling task: ${r.message}`)}}async inspectTaskDetails(e){try{let t=await f.getTask(e);if(t.ok&&t.task)alert(`Task [${t.task.taskId}]
Status: ${t.task.status}
Progress: ${Math.round((t.task.progress||0)*100)}%
Payload: ${JSON.stringify(t.task.result||t.task.error||t.task.inputRequests||{},null,2)}`)}catch(t){alert(`Failed to fetch task: ${t.message}`)}}async submitApproval(e){let t=document.getElementById(`appr-operator-${e}`),r=document.getElementById(`appr-args-${e}`),n=t?.value.trim()||"security-operator",a=void 0;if(r&&r.value.trim())try{a=JSON.parse(r.value.trim())}catch{alert("Invalid JSON in arguments editor");return}let s=await f.approveTicket(e,n,a);if(s.ok)await this.refreshApprovals(),await this.refreshTasks();else alert(`Approval failed: ${s.error||"Unknown error"}`)}async promptReject(e){let t=prompt("Reason for rejection (will be returned to the calling agent):");if(t===null)return;let n=document.getElementById(`appr-operator-${e}`)?.value.trim()||"security-operator",a=await f.rejectTicket(e,n,t);if(a.ok)await this.refreshApprovals(),await this.refreshTasks();else alert(`Rejection failed: ${a.error||"Unknown error"}`)}setPlaygroundMode(e){d.setState({playgroundMode:e})}selectCapability(e){d.setState({selectedCapabilityId:e});let t=d.getState().capabilities.find((n)=>n.id===e),r=document.getElementById("pg-args-input");if(t){let n=G(t.input_schema,!1),a=JSON.stringify(n,null,2);if(r)r.value=a;let s={...d.getState().playgroundArgs||{}};s[e]=a,d.getState().playgroundArgs=s}}selectResource(e){d.setState({selectedResourceId:e})}selectPrompt(e){d.setState({selectedPromptId:e})}filterResources(e){let t=e.toLowerCase().trim(),n=(d.getState().resources||[]).filter((s)=>s.id.toLowerCase().includes(t)||s.name&&s.name.toLowerCase().includes(t)||s.uri&&s.uri.toLowerCase().includes(t)||s.server&&s.server.toLowerCase().includes(t)),a=document.getElementById("pg-res-list");if(a)if(n.length===0)a.innerHTML=`
          <div style="padding: 24px 16px; text-align: center; color: var(--text-dim); font-size: 11.5px;">
            No resources match "${S(e)}"
          </div>
        `;else a.innerHTML=n.map((s)=>{let o=s.id===d.getState().selectedResourceId?"active":"",i=s.uri?s.uri.split(":")[0]:"res";return`
            <div class="cap-item ${o}" onclick="window.app.selectResource('${S(s.id)}')">
              <div style="display: flex; justify-content: space-between; align-items: center;">
                <span style="font-weight: 600; color: var(--text-main); font-family: var(--ff-mono); font-size: 12px;">${S(s.name||s.id)}</span>
                <span class="badge" style="font-size: 9.5px; background: rgba(56, 189, 248, 0.15); color: var(--cyan-400);">${S(i)}</span>
              </div>
              <div style="font-size: 11px; color: var(--text-dim); margin-top: 2px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">${S(s.uri)}</div>
              <div style="display: flex; justify-content: space-between; font-size: 10px; color: var(--text-muted); margin-top: 4px;">
                <span>server: ${S(s.server||"local")}</span>
                <span>${S(s.mime_type||"text/plain")}</span>
              </div>
            </div>
          `}).join("")}filterPrompts(e){let t=e.toLowerCase().trim(),n=(d.getState().prompts||[]).filter((s)=>s.id.toLowerCase().includes(t)||s.name&&s.name.toLowerCase().includes(t)||s.description&&s.description.toLowerCase().includes(t)||s.server&&s.server.toLowerCase().includes(t)),a=document.getElementById("pg-prompt-list");if(a)if(n.length===0)a.innerHTML=`
          <div style="padding: 24px 16px; text-align: center; color: var(--text-dim); font-size: 11.5px;">
            No prompts match "${S(e)}"
          </div>
        `;else a.innerHTML=n.map((s)=>{let o=s.id===d.getState().selectedPromptId?"active":"",i=s.arguments?s.arguments.length:0;return`
            <div class="cap-item ${o}" onclick="window.app.selectPrompt('${S(s.id)}')">
              <div style="display: flex; justify-content: space-between; align-items: center;">
                <span style="font-weight: 600; color: var(--text-main); font-family: var(--ff-mono); font-size: 12px;">${S(s.name||s.id)}</span>
                <span class="badge" style="font-size: 9.5px; background: rgba(168, 85, 247, 0.15); color: var(--purple-400);">${i} args</span>
              </div>
              <div style="font-size: 11px; color: var(--text-dim); margin-top: 2px;">${S(s.description||s.title||"Prompt template")}</div>
              <div style="font-size: 10px; color: var(--text-muted); margin-top: 4px;">server: ${S(s.server||"local")}</div>
            </div>
          `}).join("")}updatePlaygroundArgs(e){let t=d.getState(),r=t.selectedCapabilityId||(t.capabilities[0]?t.capabilities[0].id:null);if(!r)return;let n={...t.playgroundArgs||{}};n[r]=e,t.playgroundArgs=n}fillPlaygroundSampleArgs(e=!1){let t=d.getState(),r=t.selectedCapabilityId||(t.capabilities[0]?t.capabilities[0].id:null),n=t.capabilities.find((i)=>i.id===r),a=document.getElementById("pg-args-input");if(!a)return;if(!n||!n.input_schema){if(a.value="{}",r){let i={...t.playgroundArgs||{}};i[r]="{}",t.playgroundArgs=i}return}let s=G(n.input_schema,e),o=JSON.stringify(s,null,2);if(a.value=o,r){let i={...t.playgroundArgs||{}};i[r]=o,t.playgroundArgs=i}}formatPlaygroundArgs(){let e=d.getState(),t=e.selectedCapabilityId||(e.capabilities[0]?e.capabilities[0].id:null),r=document.getElementById("pg-args-input");if(r)try{let n=JSON.parse(r.value||"{}"),a=JSON.stringify(n,null,2);if(r.value=a,t){let s={...e.playgroundArgs||{}};s[t]=a,e.playgroundArgs=s}}catch(n){alert(`Cannot format JSON: ${n.message}`)}}insertPlaygroundArgKey(e,t,r){let n=d.getState(),a=n.selectedCapabilityId||(n.capabilities[0]?n.capabilities[0].id:null),s=document.getElementById("pg-args-input");if(s){let o={};try{o=JSON.parse(s.value||"{}")}catch{o={}}if(o[e]===void 0)if(r!==null&&r!==void 0)o[e]=r;else switch(t){case"string":o[e]=`sample_${e}`;break;case"number":case"integer":o[e]=0;break;case"boolean":o[e]=!0;break;case"array":o[e]=[];break;case"object":o[e]={};break;default:o[e]=`sample_${e}`}let i=JSON.stringify(o,null,2);if(s.value=i,a){let l={...n.playgroundArgs||{}};l[a]=i,n.playgroundArgs=l}}}fillBatchStepSampleArgs(e){let t=d.getState(),r=[...t.batchSteps||[]],n=r[e];if(!n||!n.capability_id)return;let a=t.capabilities.find((l)=>l.id===n.capability_id);if(!a||!a.input_schema)return;let s=a.input_schema.properties||{},o={};for(let[l,c]of Object.entries(s))if(c.default!==void 0)o[l]=c.default;else if(Array.isArray(c.enum)&&c.enum.length>0)o[l]=c.enum[0];else switch(c.type||"string"){case"string":o[l]=`sample_${l}`;break;case"number":case"integer":o[l]=0;break;case"boolean":o[l]=!0;break;case"array":o[l]=[];break;case"object":o[l]={};break;default:o[l]=`sample_${l}`}let i=JSON.stringify(o,null,2);r[e]={...r[e],argsJson:i},d.setState({batchSteps:r})}filterCapabilities(e){let t=e.toLowerCase().trim(),n=d.getState().capabilities.filter((s)=>s.id.toLowerCase().includes(t)||s.summary&&s.summary.toLowerCase().includes(t)||s.server&&s.server.toLowerCase().includes(t)),a=document.getElementById("pg-cap-list");if(a)if(n.length===0)a.innerHTML=`
          <div style="padding: 24px 16px; text-align: center; color: var(--text-dim); font-size: 11.5px;">
            No capabilities match "${S(e)}"
          </div>
        `;else a.innerHTML=n.map((s)=>`
          <div class="cap-item ${s.id===d.getState().selectedCapabilityId?"active":""}" onclick="window.app.selectCapability('${S(s.id)}')">
            <div style="display: flex; justify-content: space-between; align-items: center;">
              <span style="font-weight: 600; color: var(--text-main); font-family: var(--ff-mono); font-size: 12px;">${S(s.id)}</span>
              <span style="font-size: 10px; color: var(--green-400);">${S(s.mode||"read")}</span>
            </div>
            <div style="font-size: 11px; color: var(--text-dim); margin-top: 2px;">server: ${S(s.server||"local")}</div>
          </div>
        `).join("")}async executePlaygroundTool(){let e=d.getState(),t=e.selectedCapabilityId||(e.capabilities[0]?e.capabilities[0].id:null);if(!t)return;let r=document.getElementById("pg-args-input")?.value||"{}",n=document.getElementById("pg-context-input")?.value||void 0,a=document.getElementById("pg-jsonpath-input")?.value.trim()||void 0,s=document.getElementById("pg-limit-lines-input")?.value.trim()||void 0,o=document.getElementById("pg-truncate-bytes-input")?.value.trim()||void 0,i={};try{i=JSON.parse(r)}catch{alert("Invalid arguments JSON object");return}if(a)i._jsonpath=a;if(s&&!isNaN(Number(s)))i._limit_lines=Number(s);if(o&&!isNaN(Number(o)))i._truncate_bytes=Number(o);let l=`op-${Date.now()}`;d.setState({isExecuting:!0,activeRequestId:l});let c=e.activeProfile||void 0,u=e.playgroundAsyncTask||!1;try{let g=await f.callCapability({capability_id:t,args:i,request_id:l,async_task:u?!0:void 0,context:{operation_id:n||l}},c);if(d.setState({isExecuting:!1,activeRequestId:null,executionResult:{status:g.status,durationMs:g.durationMs,data:g.data}}),g.status===202||g.data?.resultType==="task")this.refreshTasks();d.addEventLog("POST",`/v1/tools/call → ${t}`,g.status===200?"200 OK":`HTTP ${g.status}`,`${g.durationMs.toFixed(1)}ms`),f.getConfig().then((b)=>{if(b.ok&&b.circuit_breakers)d.setState({circuitBreakers:b.circuit_breakers})})}catch(g){d.setState({isExecuting:!1,activeRequestId:null,executionResult:{status:500,durationMs:0,data:{error:g.toString()}}})}}async cancelActiveOperation(){let t=d.getState().activeRequestId;if(t)try{await f.cancelOperation(t)}catch(r){console.warn("Failed to send cancel signal:",r)}d.setState({isExecuting:!1,activeRequestId:null,executionResult:{status:499,durationMs:0,data:{ok:!1,error:{code:"CANCELLED",message:"Operation cancelled by operator"}}}})}openBatchModal(){let e=d.getState(),t=e.batchSteps;if(!t||t.length===0)t=[{id:"step_1",capability_id:e.selectedCapabilityId||(e.capabilities[0]?e.capabilities[0].id:""),argsJson:"{}",continue_on_error:!1},{id:"step_2",capability_id:"",argsJson:"{}",continue_on_error:!0}],d.setState({batchSteps:t});d.setState({isBatchModalOpen:!0})}closeBatchModal(){d.setState({isBatchModalOpen:!1})}addBatchStep(){let t=[...d.getState().batchSteps||[]],r=t.length+1;t.push({id:`step_${r}`,capability_id:"",argsJson:"{}",continue_on_error:!1}),d.setState({batchSteps:t})}removeBatchStep(e){let r=[...d.getState().batchSteps||[]];if(r.length<=1){alert("Pipeline must contain at least one execution step.");return}r.splice(e,1);let n=r.map((a,s)=>({...a,id:`step_${s+1}`}));d.setState({batchSteps:n})}updateBatchStepCapability(e,t){let n=[...d.getState().batchSteps||[]];if(n[e])n[e]={...n[e],capability_id:t},d.setState({batchSteps:n})}updateBatchStepContinueOnError(e,t){let n=[...d.getState().batchSteps||[]];if(n[e])n[e]={...n[e],continue_on_error:t},d.setState({batchSteps:n})}updateBatchStepArgs(e,t){let r=d.getState(),n=[...r.batchSteps||[]];if(n[e])n[e]={...n[e],argsJson:t},r.batchSteps[e].argsJson=t}appendBatchVariable(e,t){let n=[...d.getState().batchSteps||[]],a=document.getElementById(`batch-step-args-${e}`);if(a){let s=a.value,o=a.selectionStart||s.length,i=a.selectionEnd||s.length,l=s.substring(0,o)+t+s.substring(i);if(a.value=l,n[e])n[e]={...n[e],argsJson:l},d.setState({batchSteps:n})}}async executeBatchPipeline(){let e=d.getState(),t=e.batchSteps||[],r=[];for(let a=0;a<t.length;a++){let s=t[a];if(!s.capability_id){alert(`Please select a capability for Step ${a+1}`);return}let o={};try{o=JSON.parse(s.argsJson||"{}")}catch{alert(`Invalid JSON in Step ${a+1} arguments`);return}r.push({id:s.id||`step_${a+1}`,capability_id:s.capability_id,args:o,continue_on_error:s.continue_on_error})}d.setState({isBatchModalOpen:!1});let n=e.activeProfile||void 0;try{let a=await f.batchCallCapabilities(r,n);d.setState({executionResult:{status:a.status,durationMs:a.durationMs,data:a.data}}),d.addEventLog("POST",`/v1/tools/batch_call (${t.length} steps)`,a.status===200?"200 OK":`HTTP ${a.status}`,`${a.durationMs.toFixed(1)}ms`)}catch(a){d.setState({executionResult:{status:500,durationMs:0,data:{error:a.toString()}}})}}async executeReadResource(){let e=d.getState(),t=e.selectedResourceId||(e.resources[0]?e.resources[0].id:null);if(!t)return;let r=document.getElementById("pg-res-jsonpath-input")?.value.trim()||void 0,n=document.getElementById("pg-res-lines-input")?.value.trim()||void 0,a=document.getElementById("pg-res-bytes-input")?.value.trim()||void 0,s={resource_id:t};if(r)s._jsonpath=r;if(n&&!isNaN(Number(n)))s._limit_lines=Number(n);if(a&&!isNaN(Number(a)))s._truncate_bytes=Number(a);let o=e.activeProfile||void 0;try{let i=await f.readResource({resource_id:t,input_responses:s},o);d.setState({resourceReadResult:{status:i.status,durationMs:i.durationMs,data:i.data}}),d.addEventLog("POST",`/v1/resources/read → ${t}`,i.status===200?"200 OK":`HTTP ${i.status}`,`${i.durationMs.toFixed(1)}ms`)}catch(i){d.setState({resourceReadResult:{status:500,durationMs:0,data:{error:i.toString()}}})}}async executeGetPrompt(){let e=d.getState(),t=e.selectedPromptId||(e.prompts[0]?e.prompts[0].id:null);if(!t)return;let r=document.querySelectorAll(".prompt-arg-input"),n={};r.forEach((s)=>{let o=s,i=o.getAttribute("data-arg-name");if(i&&o.value.trim())n[i]=o.value.trim()});let a=e.activeProfile||void 0;try{let s=await f.getPrompt({prompt_id:t,arguments:n},a);d.setState({promptGetResult:{status:s.status,durationMs:s.durationMs,data:s.data}}),d.addEventLog("POST",`/v1/prompts/get → ${t}`,s.status===200?"200 OK":`HTTP ${s.status}`,`${s.durationMs.toFixed(1)}ms`)}catch(s){d.setState({promptGetResult:{status:500,durationMs:0,data:{error:s.toString()}}})}}toggleBatchPlayground(){let e=document.getElementById("pg-args-input");if(!e)return;let t=[{id:"step_1",capability_id:"sqlite.read_query",args:{query:"SELECT * FROM users LIMIT 2"}},{id:"step_2",capability_id:"github.issues.search",args:{query:"label:bug"},continue_on_error:!0}];e.value=JSON.stringify(t,null,2)}async submitPolicyRule(e){let t=e==="allow"?"policy-new-allow":e==="deny"?"policy-new-deny":e==="redact"?"policy-new-redact":"policy-new-requireApproval",r=document.getElementById(t);if(!r)return;let n=r.value.trim();if(!n)return;await this.addPolicyRule(e,n),r.value=""}async addPolicyRule(e,t){let r=(t||"").trim();if(!r)return;let n=d.getState(),a=n.activeProfile,s=a?n.config.profiles?.[a]:void 0;if(s&&a){let o=s.policy||{},i=[...o.allow||[]],l=[...o.deny||[]],c=[...o.redact_keys||o.redactKeys||[]],u=[...o.require_approval||o.requireApproval||[]];if(e==="allow"&&!i.includes(r))i.push(r);if(e==="deny"&&!l.includes(r))l.push(r);if(e==="redact"&&!c.includes(r))c.push(r);if(e==="requireApproval"&&!u.includes(r))u.push(r);let g={...o,allow:i,deny:l,redactKeys:c,requireApproval:u},b=await f.upsertProfile(a,s.servers,s.description,g);if(!b.ok)alert(`Failed to save profile policy rule: ${b.error||"Unknown error"}`)}else{let o=n.config.policy||{},i=[...o.allow||[]],l=[...o.deny||[]],c=[...o.redact_keys||o.redactKeys||[]],u=[...o.require_approval||o.requireApproval||[]];if(e==="allow"&&!i.includes(r))i.push(r);if(e==="deny"&&!l.includes(r))l.push(r);if(e==="redact"&&!c.includes(r))c.push(r);if(e==="requireApproval"&&!u.includes(r))u.push(r);let g=await f.savePolicy({...o,allow:i,deny:l,redact_keys:c,redactKeys:c,require_approval:u,requireApproval:u});if(!g.ok)alert(`Failed to save policy rule: ${g.error||"Unknown error"}`)}await this.refreshData()}async removePolicyRule(e,t){let r=d.getState(),n=r.activeProfile,a=n?r.config.profiles?.[n]:void 0;if(a&&n){let s=a.policy||{},o=[...s.allow||[]],i=[...s.deny||[]],l=[...s.redact_keys||s.redactKeys||[]],c=[...s.require_approval||s.requireApproval||[]];if(e==="allow")o.splice(t,1);if(e==="deny")i.splice(t,1);if(e==="redact")l.splice(t,1);if(e==="requireApproval")c.splice(t,1);let u={...s,allow:o,deny:i,redactKeys:l,requireApproval:c},g=await f.upsertProfile(n,a.servers,a.description,u);if(!g.ok)alert(`Failed to update profile policy: ${g.error||"Unknown error"}`)}else{let s=r.config.policy||{},o=[...s.allow||[]],i=[...s.deny||[]],l=[...s.redact_keys||s.redactKeys||[]],c=[...s.require_approval||s.requireApproval||[]];if(e==="allow")o.splice(t,1);if(e==="deny")i.splice(t,1);if(e==="redact")l.splice(t,1);if(e==="requireApproval")c.splice(t,1);let u=await f.savePolicy({...s,allow:o,deny:i,redact_keys:l,redactKeys:l,require_approval:c,requireApproval:c});if(!u.ok)alert(`Failed to update policy: ${u.error||"Unknown error"}`)}await this.refreshData()}async saveWebhookConfig(){let e=document.getElementById("policy-webhook-url"),t=document.getElementById("policy-webhook-format"),r=document.getElementById("policy-webhook-secret"),n=e?e.value.trim():"",a=t?t.value:"generic",s=r?r.value.trim():"",i=d.getState().config.policy||{},l=n?{url:n,format:a,secret:s&&!s.startsWith("WARMPLANE_")&&!s.includes("_")?s:void 0,secret_env:s&&(s.startsWith("WARMPLANE_")||s.includes("_"))?s:void 0,events:["approval.requested","circuit_breaker.tripped","policy.violation"]}:void 0,c=await f.savePolicy({...i,webhook:l});if(c.ok)alert("Webhook settings saved successfully");else alert(`Failed to save webhook settings: ${c.error||"Unknown error"}`);await this.refreshData()}async testWebhook(){let e=document.getElementById("policy-webhook-url"),t=document.getElementById("policy-webhook-format"),r=e?e.value.trim():void 0,n=t?t.value:void 0,a=document.getElementById("policy-webhook-status");if(a)a.textContent="Sending test event...",a.style.color="var(--cyan-400)";try{let s=await f.testWebhook(r,n);if(s.ok){if(alert(`Test webhook sent successfully! (${s.message})`),a)a.textContent=`✔ Test sent (HTTP ${s.status_code||200})`,a.style.color="var(--green-400)"}else if(alert(`Test webhook failed: ${s.error||"Unknown error"}`),a)a.textContent=`✖ Failed: ${s.error}`,a.style.color="var(--red-400)"}catch(s){alert(`Error sending test webhook: ${s.message}`)}}testPolicySandbox(e){let t=document.getElementById("policy-test-verdict");if(!t)return;let r=e.trim();if(!r){t.textContent="ENTER ID",t.style.color="var(--text-dim)";return}let n=d.getState(),a=n.activeProfile,s=a?n.config.profiles?.[a]:void 0,o=n.config.policy||{},i=s?.policy,l=o.deny||[],c=i?.deny||[],u=Array.from(new Set([...l,...c])),g=o.require_approval||o.requireApproval||[],b=i?.require_approval||i?.requireApproval||[],p=Array.from(new Set([...g,...b])),m=i&&i.allow&&i.allow.length>0?i.allow:o.allow||[],y=(v,x)=>{if(v==="*")return!0;if(v.endsWith("*"))return x.startsWith(v.slice(0,-1));return v===x};if(u.some((v)=>y(v,r))){t.textContent="DENIED (Strict Block)",t.style.color="var(--red-400)";return}if(m.length>0&&!m.some((v)=>y(v,r))){t.textContent="DENIED (Not in Allow List)",t.style.color="var(--red-400)";return}if(p.some((v)=>y(v,r))){t.textContent="REQUIRE APPROVAL (HITL Gate)",t.style.color="var(--amber-400)";return}t.textContent="ALLOWED",t.style.color="var(--green-400)"}async deleteServer(e){if(!confirm(`Are you sure you want to remove server '${e}' from config?`))return;await f.deleteServer(e),await this.refreshData()}async restartServer(e){try{let t=await f.restartServer(e);if(t.ok)await this.refreshData();else alert(`Failed to restart server '${e}': ${t.error||"Unknown error"}`)}catch(t){alert(`Error restarting server '${e}': ${t.message}`)}}openServerDiagnosticsModal(e){this.closeModals();let t=d.getState(),r=t.config.mcpServers?.[e],n=t.serverStatuses?.[e],a=(t.circuitBreakers||[]).find((l)=>l.server_id===e),s=document.getElementById("modal-server-diagnostics");if(!s)return;let o=document.getElementById("modal-diag-title"),i=document.getElementById("modal-diag-body");if(o)o.textContent=`Live Diagnostics: ${e}`;if(i){let l=n?.status==="degraded",c=l?"var(--amber-400)":n?.status==="connected"?"var(--green-400)":"var(--red-400)",u=n?.error||"No active crash or error reported. Server is healthy.";i.innerHTML=`
        <div style="display: flex; gap: 10px; align-items: center; margin-bottom: 16px;">
          <span style="width: 10px; height: 10px; border-radius: 50%; background: ${c};"></span>
          <span style="font-weight: 700; font-size: 14px; color: var(--text-main);">Current Status: <span style="color: ${c}; text-transform: uppercase;">${S(n?.status||"unknown")}</span></span>
          <span class="brand-badge" style="color: var(--cyan-400);">Protocol: ${S(n?.protocol_version||"2026-07-28")}</span>
        </div>

        <div style="background: rgba(0,0,0,0.3); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 12px; margin-bottom: 14px;">
          <div style="font-size: 11px; font-weight: 700; color: var(--amber-400); text-transform: uppercase; margin-bottom: 6px;">
            ⚠️ Diagnostic Details / Failure Root Cause
          </div>
          <pre style="font-family: var(--ff-mono); font-size: 11.5px; color: ${l?"var(--red-300)":"var(--text-dim)"}; white-space: pre-wrap; word-break: break-word; margin: 0;">${S(u)}</pre>
        </div>

        <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 10px; margin-bottom: 14px;">
          <div style="background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 10px;">
            <div style="font-size: 10.5px; color: var(--text-dim);">Circuit Breaker State</div>
            <div style="font-weight: 700; font-size: 13px; color: var(--text-main); margin-top: 2px;">
              ${a?`${a.state.toUpperCase()} (${a.consecutive_failures} failures)`:"CLOSED (Healthy)"}
            </div>
          </div>
          <div style="background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 10px;">
            <div style="font-size: 10.5px; color: var(--text-dim);">Process Supervision</div>
            <div style="font-weight: 700; font-size: 13px; color: var(--text-main); margin-top: 2px;">
              Auto-Restart: ${r?.resilience?.autoRestart!==!1?"ENABLED":"DISABLED"}
            </div>
          </div>
        </div>

        <div style="background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 10px; margin-bottom: 16px;">
          <div style="font-size: 10.5px; color: var(--text-dim); margin-bottom: 4px;">Configured Execution Target</div>
          <code style="font-family: var(--ff-mono); font-size: 11px; color: var(--cyan-400); display: block; word-break: break-all;">
            ${r?.command?`${S(r.command)} ${S((r.args||[]).join(" "))}`:S(r?.url||"")}
          </code>
        </div>

        <div style="display: flex; justify-content: flex-end; gap: 8px;">
          <button class="btn btn-primary" onclick="window.app.restartServer('${S(e)}'); window.app.closeModals();">⚡ Restart &amp; Probe Now</button>
          <button class="btn btn-ghost" onclick="window.app.closeModals()">Close</button>
        </div>
      `}s.classList.add("active")}openAddServerModal(){this.closeModals();let e=document.getElementById("modal-srv-title"),t=document.getElementById("modal-srv-template-banner"),r=document.getElementById("modal-srv-name"),n=document.getElementById("modal-srv-transport"),a=document.getElementById("modal-srv-command"),s=document.getElementById("modal-srv-url"),o=document.getElementById("modal-srv-ft"),i=document.getElementById("modal-srv-cd"),l=document.getElementById("modal-srv-autorestart"),c=document.getElementById("modal-srv-maxrestarts");if(e)e.textContent="Add Upstream MCP Server";if(t)t.style.display="flex";if(r)r.value="",r.disabled=!1;if(n)n.value="stdio";if(a)a.value="";if(s)s.value="";let u=document.getElementById("modal-group-cmd"),g=document.getElementById("modal-group-url");if(u)u.style.display="block";if(g)g.style.display="none";if(o)o.value="3";if(i)i.value="30000";if(l)l.value="true";if(c)c.value="5";let b=document.getElementById("modal-add-server");if(b)b.classList.add("active")}openEditServerModal(e){this.closeModals();let t=d.getState(),r=t.config.mcpServers?.[e];if(!r){alert(`Server '${e}' not found in configuration.`);return}let n=document.getElementById("modal-srv-title"),a=document.getElementById("modal-srv-template-banner"),s=document.getElementById("modal-srv-name"),o=document.getElementById("modal-srv-transport"),i=document.getElementById("modal-srv-command"),l=document.getElementById("modal-srv-url"),c=document.getElementById("modal-srv-ft"),u=document.getElementById("modal-srv-cd"),g=document.getElementById("modal-srv-autorestart"),b=document.getElementById("modal-srv-maxrestarts");if(n)n.textContent=`Edit Server '${e}'`;if(a)a.style.display="none";if(s)s.value=e,s.disabled=!0;let p=!!r.command;if(o)o.value=p?"stdio":"http";let m=document.getElementById("modal-group-cmd"),y=document.getElementById("modal-group-url");if(m)m.style.display=p?"block":"none";if(y)y.style.display=p?"none":"block";if(i)i.value=p?`${r.command} ${(r.args||[]).join(" ")}`.trim():"";if(l)l.value=r.url||"";let v=r.resilience||t.config.resilience;if(c)c.value=String(v?.failureThreshold??3);if(u)u.value=String(v?.cooldownMs??30000);if(g)g.value=v?.autoRestart===!1?"false":"true";if(b)b.value=String(v?.maxRestarts??5);let x=document.getElementById("modal-add-server");if(x)x.classList.add("active")}async submitAddServer(){let e=document.getElementById("modal-srv-name"),t=e?.value.trim(),r=document.getElementById("modal-srv-transport")?.value;if(!t){alert("Server name is required");return}if(e&&!e.disabled){if((d.getState().config.mcpServers||{})[t]){if(!confirm(`Server '${t}' already exists in configuration. Do you want to overwrite it?`))return}}let n={};if(r==="stdio"){let u=(document.getElementById("modal-srv-command")?.value.trim()).split(/\s+/).filter(Boolean);if(u.length===0){alert("Command is required");return}n.command=u[0],n.args=u.slice(1)}else{let c=document.getElementById("modal-srv-url")?.value.trim();if(!c){alert("URL is required");return}n.url=c}let a=document.getElementById("modal-srv-ft")?.value.trim(),s=document.getElementById("modal-srv-cd")?.value.trim(),o=document.getElementById("modal-srv-autorestart")?.value,i=document.getElementById("modal-srv-maxrestarts")?.value.trim();if(a||s||o||i)n.resilience={failureThreshold:a?Number(a):3,cooldownMs:s?Number(s):30000,autoRestart:o!=="false",maxRestarts:i?Number(i):5};let l=await f.upsertServer(t,n);if(l.ok)this.closeModals(),await this.refreshData();else alert(`Failed to save server: ${l.error}`)}openTemplateCatalog(){this.closeModals();let e=document.getElementById("modal-templates");if(e)e.classList.add("active");this.renderTemplateGrid()}setTemplateCategory(e){this.activeTemplateCategory=e,document.querySelectorAll(".tmpl-cat-btn").forEach((t)=>{if(t.getAttribute("data-category")===e)t.classList.add("active"),t.style.background="var(--surface-elevated)",t.style.color="var(--amber-400)";else t.classList.remove("active"),t.style.background="var(--surface-card)",t.style.color="var(--text-main)"}),this.renderTemplateGrid()}filterTemplates(e){this.activeTemplateFilter=e.toLowerCase().trim(),this.renderTemplateGrid()}renderTemplateGrid(){let e=document.getElementById("tmpl-grid");if(!e)return;let t=X.filter((a)=>{let s=this.activeTemplateCategory==="all"||a.category===this.activeTemplateCategory,o=!this.activeTemplateFilter||a.name.toLowerCase().includes(this.activeTemplateFilter)||a.id.toLowerCase().includes(this.activeTemplateFilter)||a.description.toLowerCase().includes(this.activeTemplateFilter)||a.command.toLowerCase().includes(this.activeTemplateFilter)||a.envFields.some((i)=>i.key.toLowerCase().includes(this.activeTemplateFilter));return s&&o});if(t.length===0){e.innerHTML=`
        <div style="grid-column: span 2; padding: 32px; text-align: center; color: var(--text-dim);">
          No matching MCP server templates found.
        </div>
      `;return}let n=d.getState().config.mcpServers||{};e.innerHTML=t.map((a)=>{let s=!!n[a.id],o=`${a.command} ${a.defaultArgs.join(" ")}`;return`
        <div class="bento-card" style="display: flex; flex-direction: column; justify-content: space-between; padding: 14px; background: var(--surface); border: 1px solid var(--border); min-width: 0; transition: transform 0.15s, border-color 0.15s;">
          <div>
            <div style="display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 6px;">
              <div style="display: flex; align-items: center; gap: 8px; min-width: 0;">
                <span style="font-weight: 700; font-size: 13.5px; color: var(--text-main); white-space: nowrap; overflow: hidden; text-overflow: ellipsis;">${S(a.name)}</span>
                <span class="brand-badge" style="font-size: 9.5px; padding: 1px 6px; flex-shrink: 0;">${S(a.badge)}</span>
              </div>
              ${s?'<span style="font-size: 10px; color: var(--green-400); font-weight: 600; flex-shrink: 0;">CONNECTED</span>':""}
            </div>
            <div style="font-size: 11.5px; color: var(--text-muted); line-height: 1.4; margin-bottom: 8px;">
              ${S(a.description)}
            </div>
            <div style="font-family: var(--ff-mono); font-size: 10.5px; color: var(--text-dim); background: var(--surface-card); padding: 5px 8px; border-radius: var(--radius-sm); border: 1px solid var(--border-subtle); overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">
              <code>${S(o)}</code>
            </div>
            ${a.envFields.length>0?`
              <div style="font-size: 10.5px; color: var(--amber-400); margin-top: 6px; display: flex; align-items: center; gap: 4px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">
                <span>⚡ Needs:</span>
                <code style="overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">${a.envFields.map((i)=>S(i.key)).join(", ")}</code>
              </div>
            `:""}
          </div>

          <div style="display: flex; justify-content: flex-end; margin-top: 12px; gap: 6px;">
            <button class="btn btn-primary" style="font-size: 11.5px; padding: 4px 10px;" onclick="window.app.selectTemplate('${S(a.id)}')">
              ${s?"Configure Another":"✨ 1-Click Setup"}
            </button>
          </div>
        </div>
      `}).join("")}selectTemplate(e){let t=X.find((l)=>l.id===e);if(!t)return;this.selectedTemplate=t,this.closeModals();let r=document.getElementById("modal-configure-template");if(r)r.classList.add("active");let n=document.getElementById("cfg-tmpl-title"),a=document.getElementById("cfg-tmpl-desc"),s=document.getElementById("cfg-tmpl-form");if(n)n.textContent=`Configure ${t.name} Server`;if(a)a.textContent=t.description;let o=d.getState().config.mcpServers||{},i=t.id;if(o[i]){let l=2;while(o[`${t.id}-${l}`])l++;i=`${t.id}-${l}`}if(s){let l="";if(t.envFields.length>0)l=`
          <div style="margin-top: 14px; margin-bottom: 6px; font-weight: 700; font-size: 11px; text-transform: uppercase; color: var(--amber-400); letter-spacing: 0.5px;">
            Environment Variables &amp; API Keys
          </div>
          ${t.envFields.map((c)=>`
            <div class="form-group">
              <label class="form-label">${S(c.label)} ${c.required?'<span style="color: var(--red-400);">*</span>':"(Optional)"}</label>
              <input type="password" class="form-input tmpl-env-input" data-key="${S(c.key)}" placeholder="${S(c.placeholder||"")}">
              ${c.description?`<div style="font-size: 10.5px; color: var(--text-dim); margin-top: 3px;">${S(c.description)}</div>`:""}
            </div>
          `).join("")}
        `;s.innerHTML=`
        <div class="form-group">
          <label class="form-label">Server Identifier (Name)</label>
          <input type="text" class="form-input" id="cfg-srv-id" value="${S(i)}">
          <div style="font-size: 10.5px; color: var(--text-dim); margin-top: 3px;">Must be unique across all configured servers.</div>
        </div>
        <div class="form-group">
          <label class="form-label">Command Line Arguments</label>
          <input type="text" class="form-input" id="cfg-srv-args" value="${S(t.defaultArgs.join(" "))}" placeholder="${S(t.argsPlaceholder||"")}">
          <div style="font-size: 10.5px; color: var(--text-dim); margin-top: 3px;">Executable: <code>${S(t.command)}</code></div>
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
      `}}async submitTemplateServer(){if(!this.selectedTemplate)return;let e=this.selectedTemplate,t=document.getElementById("cfg-srv-id")?.value.trim(),r=document.getElementById("cfg-srv-args")?.value.trim();if(!t){alert("Server identifier is required");return}if((d.getState().config.mcpServers||{})[t]){if(!confirm(`Server '${t}' already exists. Do you want to overwrite its configuration?`))return}let s=r?r.split(/\s+/).filter(Boolean):[],o={},i=document.querySelectorAll(".tmpl-env-input");for(let m of Array.from(i)){let y=m.getAttribute("data-key"),v=m.value.trim(),x=e.envFields.find((C)=>C.key===y);if(x?.required&&!v){alert(`Required field '${x.label}' is missing.`);return}if(y&&v)o[y]=v}let l={command:e.command,args:s};if(Object.keys(o).length>0)l.env=o;let c=document.getElementById("cfg-srv-ft")?.value.trim(),u=document.getElementById("cfg-srv-cd")?.value.trim(),g=document.getElementById("cfg-srv-autorestart")?.value,b=document.getElementById("cfg-srv-maxrestarts")?.value.trim();if(c||u||g||b)l.resilience={failureThreshold:c?Number(c):3,cooldownMs:u?Number(u):30000,autoRestart:g!=="false",maxRestarts:b?Number(b):5};let p=await f.upsertServer(t,l);if(p.ok)this.closeModals(),await this.refreshData();else alert(`Failed to save server: ${p.error}`)}async openImportModal(){this.closeModals();let e=document.getElementById("modal-import");if(e)e.classList.add("active");let t=document.getElementById("modal-eco-list");if(!t)return;t.innerHTML='<div style="color: var(--text-dim); padding: 12px; text-align: center;">Scanning IDE configs...</div>';try{let r=await f.getEcosystemSources();if(r.sources&&r.sources.length>0)t.innerHTML=r.sources.map((n)=>`
          <label style="display: flex; align-items: center; gap: 10px; background: var(--surface); padding: 10px 14px; border-radius: var(--radius-sm); border: 1px solid var(--border); cursor: pointer;">
            <input type="checkbox" class="eco-checkbox" value="${n.path}" checked>
            <div>
              <div style="font-weight: 600; color: var(--text-main);">${n.name}</div>
              <div style="font-size: 11px; color: var(--text-dim);">${n.server_count} servers (${n.servers.join(", ")})</div>
            </div>
          </label>
        `).join("");else t.innerHTML='<div style="color: var(--text-dim); padding: 12px; text-align: center;">No external MCP configuration files found on this system.</div>'}catch{t.innerHTML='<div style="color: var(--red-400); padding: 12px; text-align: center;">Failed to scan ecosystem sources.</div>'}}async submitImport(){let e=document.querySelectorAll(".eco-checkbox:checked");if(e.length===0){alert("No sources selected");return}for(let t of Array.from(e))await f.importConfig(t.value,!1);this.closeModals(),await this.refreshData()}async refreshClients(){try{let e=await f.getClients();if(e.ok&&Array.isArray(e.clients))d.setState({clients:e.clients})}catch(e){console.error("Failed to scan clients:",e)}}async attachClient(e,t){let r=t;if(!r){let a=document.getElementById(`client-prof-${e}`)||document.getElementById(`overview-client-prof-${e}`);if(a)r=a.value||void 0;else r=d.getState().activeProfile||void 0}let n=await f.attachClient(e,r);if(!n.ok)alert(`Failed to attach client: ${n.error||n.message||"Unknown error"}`);else await this.refreshData()}async detachClient(e){if(!confirm("Disconnect Warmplane from this client?"))return;let t=await f.detachClient(e);if(!t.ok)alert(`Failed to detach client: ${t.error||t.message||"Unknown error"}`);else await this.refreshData()}handleAliasTargetInput(e){let t=document.getElementById("alias-suggestions-dropdown");if(!t)return;let r=(e||"").trim().toLowerCase();if(r.length<2){t.style.display="none";return}let a=d.getState().capabilities.filter((s)=>s.id.toLowerCase().includes(r)||s.summary&&s.summary.toLowerCase().includes(r)||s.description&&s.description.toLowerCase().includes(r)||s.server&&s.server.toLowerCase().includes(r)).slice(0,8);if(a.length===0){t.style.display="none";return}t.innerHTML=a.map((s)=>`
      <div style="padding: 8px 12px; cursor: pointer; border-bottom: 1px solid var(--border-subtle); display: flex; justify-content: space-between; align-items: center; transition: background 0.1s;"
           onmouseover="this.style.background='var(--surface-hover)'"
           onmouseout="this.style.background='transparent'"
           onmousedown="window.app.selectAliasSuggestion('${S(s.id)}')">
        <div>
          <div style="font-weight: 700; color: var(--text-main);">${S(s.id)}</div>
          <div style="font-size: 10.5px; color: var(--text-dim); margin-top: 2px;">${S(s.summary||s.description||"")}</div>
        </div>
        <span style="font-size: 10px; color: var(--cyan-400);">${S(s.server||"local")}</span>
      </div>
    `).join(""),t.style.display="block"}selectAliasSuggestion(e){let t=document.getElementById("alias-target");if(t)t.value=e;this.hideAliasDropdown()}hideAliasDropdown(){let e=document.getElementById("alias-suggestions-dropdown");if(e)e.style.display="none"}async createAlias(){let e=document.getElementById("alias-kind")?.value,t=document.getElementById("alias-name")?.value.trim(),r=document.getElementById("alias-target")?.value.trim();if(!t||!r){alert("Please provide both alias name and canonical target");return}await f.updateAlias(e,t,r),await this.refreshData()}async deleteAlias(e,t){await f.updateAlias(e,t,void 0),await this.refreshData()}async reloadFromDisk(){try{let e=await f.reloadConfig();if(e.ok){let t="Hot-reload completed successfully!";if(e.mounted&&e.mounted.length>0)t+=`
Mounted: ${e.mounted.join(", ")}`;if(e.unmounted&&e.unmounted.length>0)t+=`
Unmounted: ${e.unmounted.join(", ")}`;if(e.warnings&&e.warnings.length>0)t+=`
Warnings:
${e.warnings.join(`
`)}`;alert(t)}else alert(`Hot-reload failed: ${e.error||"Unknown error"}`)}catch(e){alert(`Error reaching daemon: ${e.message}`)}await this.refreshData()}renderTopProfileSelector(){let e=document.getElementById("top-profile-selector");if(!e)return;let t=d.getState(),r=t.config.profiles||{},n=Object.keys(r),a=t.activeProfile,s='<option value="">All Servers (Unrestricted)</option>';for(let o of n){let i=a===o?"selected":"";s+=`<option value="${S(o)}" ${i}>Profile: ${S(o)}</option>`}e.innerHTML=s}async setActiveProfile(e){d.setState({activeProfile:e||null}),await this.refreshData()}openAddProfileModal(){let e=document.getElementById("modal-prof-title");if(e)e.textContent="Create Server Constellation Profile";let t=document.getElementById("modal-prof-name"),r=document.getElementById("modal-prof-desc"),n=document.getElementById("modal-prof-mode");if(t)t.value="",t.disabled=!1;if(r)r.value="";if(n)n.value="create";let a=document.getElementById("modal-prof-allow"),s=document.getElementById("modal-prof-deny"),o=document.getElementById("modal-prof-hitl"),i=document.getElementById("modal-prof-redact");if(a)a.value="";if(s)s.value="";if(o)o.value="";if(i)i.value="";this.renderProfileServerCheckboxes([]);let l=document.getElementById("modal-add-profile");if(l)l.classList.add("active")}openEditProfileModal(e){let r=d.getState().config.profiles?.[e];if(!r)return;let n=document.getElementById("modal-prof-title");if(n)n.textContent=`Edit Profile: ${e}`;let a=document.getElementById("modal-prof-name"),s=document.getElementById("modal-prof-desc"),o=document.getElementById("modal-prof-mode");if(a)a.value=e,a.disabled=!0;if(s)s.value=r.description||"";if(o)o.value="edit";let i=document.getElementById("modal-prof-allow"),l=document.getElementById("modal-prof-deny"),c=document.getElementById("modal-prof-hitl"),u=document.getElementById("modal-prof-redact"),g=r.policy;if(i)i.value=(g?.allow||[]).join(", ");if(l)l.value=(g?.deny||[]).join(", ");if(c)c.value=(g?.require_approval||g?.requireApproval||[]).join(", ");if(u)u.value=(g?.redact_keys||g?.redactKeys||[]).join(", ");this.renderProfileServerCheckboxes(r.servers||[]);let b=document.getElementById("modal-add-profile");if(b)b.classList.add("active")}renderProfileServerCheckboxes(e){let t=document.getElementById("modal-prof-servers-list");if(!t)return;let r=d.getState(),n=Object.keys(r.config.mcpServers||{});if(n.length===0){t.innerHTML='<div style="font-size: 11.5px; color: var(--text-dim);">No MCP servers configured yet. Add servers first.</div>';return}t.innerHTML=n.map((a)=>{let s=e.includes(a)?"checked":"";return`
        <label style="display: flex; align-items: center; gap: 8px; font-size: 12px; cursor: pointer; padding: 4px 6px; border-radius: var(--radius-sm); transition: background 0.15s;" onmouseover="this.style.background='var(--surface-hover)'" onmouseout="this.style.background='transparent'">
          <input type="checkbox" class="prof-server-checkbox" value="${S(a)}" ${s} style="accent-color: var(--amber-400);">
          <span style="font-family: var(--ff-mono); font-weight: 600; color: var(--text-main);">${S(a)}</span>
        </label>
      `}).join("")}async saveProfile(){let e=document.getElementById("modal-prof-name"),t=document.getElementById("modal-prof-desc"),r=e?.value.trim(),n=t?.value.trim();if(!r){alert("Please enter a profile name");return}let a=document.querySelectorAll(".prof-server-checkbox:checked"),s=[];if(a.forEach((v)=>{s.push(v.value)}),s.length===0){alert("Please select at least one server to include in this constellation");return}let o=(v)=>{if(!v)return[];return v.split(",").map((x)=>x.trim()).filter((x)=>x.length>0)},i=document.getElementById("modal-prof-allow"),l=document.getElementById("modal-prof-deny"),c=document.getElementById("modal-prof-hitl"),u=document.getElementById("modal-prof-redact"),g=o(i?.value),b=o(l?.value),p=o(c?.value),m=o(u?.value),y=void 0;if(g.length>0||b.length>0||p.length>0||m.length>0)y={allow:g,deny:b,requireApproval:p,redactKeys:m};try{let v=await f.upsertProfile(r,s,n||void 0,y);if(v.ok)this.closeModals(),await this.refreshData();else alert(`Failed to save profile: ${v.error||"Unknown error"}`)}catch(v){alert(`Error saving profile: ${v.message}`)}}async deleteProfile(e){if(!confirm(`Are you sure you want to delete profile '${e}'?`))return;try{let t=await f.deleteProfile(e);if(t.ok){if(d.getState().activeProfile===e)d.setState({activeProfile:null});await this.refreshData()}else alert(`Failed to delete profile: ${t.error||"Unknown error"}`)}catch(t){alert(`Error deleting profile: ${t.message}`)}}async toggleServerInProfile(e,t,r){let a=d.getState().config.profiles?.[e];if(!a)return;let s=[...a.servers||[]];if(r){if(!s.includes(t))s.push(t)}else if(s=s.filter((o)=>o!==t),s.length===0){alert("A profile must contain at least one server. To remove the profile, delete it in the Profiles tab.");return}try{let o=await f.upsertProfile(e,s,a.description,a.policy);if(o.ok)await this.refreshData();else alert(`Failed to update profile constellation: ${o.error||"Unknown error"}`)}catch(o){alert(`Error updating profile constellation: ${o.message}`)}}closeModals(){document.querySelectorAll(".modal-backdrop").forEach((e)=>e.classList.remove("active"))}}function S(e){return String(e||"").replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;")}var pe=new ce;window.app=pe;window.addEventListener("DOMContentLoaded",()=>pe.init());
