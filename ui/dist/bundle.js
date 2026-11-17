function o(e){if(e===null||e===void 0)return"";return String(e).replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;").replace(/'/g,"&#039;")}class G{state={configPath:"mcp_servers.json",config:{mcpServers:{}},serverStatuses:{},circuitBreakers:[],clients:[],clientFilterCategory:"all",clientSearchQuery:"",secrets:[],clientsCollapsed:!1,capabilities:[],capabilitiesHiddenByPolicy:0,resources:[],resourcesHiddenByPolicy:0,prompts:[],promptsHiddenByPolicy:0,catalogEvents:[],tasks:[],undoEntries:[],undoError:null,selectedTaskId:null,taskFilterStatus:"all",approvals:[],auditEvents:[],auditTotal:0,auditFilters:{search:"",status:"all",eventType:"all",serverId:"all",limit:25,offset:0},auditSelectedEvent:null,auditStats:null,auditVerification:null,selectedCapabilityId:null,selectedResourceId:null,selectedPromptId:null,playgroundMode:"tools",playgroundArgs:{},isExecuting:!1,playgroundAsyncTask:!1,activeRequestId:null,isBatchModalOpen:!1,batchSteps:[{id:"step_1",capability_id:"",argsJson:"{}",continue_on_error:!1},{id:"step_2",capability_id:"",argsJson:"{}",continue_on_error:!0}],activeTab:"overview",activeProfile:null,eventLogs:[],executionResult:null,resourceReadResult:null,promptGetResult:null,metrics:{totalCatalogRequests:0,totalEtagHits:0,totalToolCalls:0,totalToolDurationUs:0}};listeners=[];getState(){return this.state}setState(e){this.state={...this.state,...e},this.listeners.forEach((t)=>t(this.state))}subscribe(e){return this.listeners.push(e),()=>{this.listeners=this.listeners.filter((t)=>t!==e)}}addEventLog(e,t,n,i){let r=[{time:new Date().toLocaleTimeString(),method:e,target:t,status:n,latency:i},...this.state.eventLogs].slice(0,50);this.setState({eventLogs:r})}}var p=new G;class Q{baseUrl;constructor(e=""){this.baseUrl=e}async getConfig(){return(await fetch(`${this.baseUrl}/v1/config`)).json()}async listCapabilities(e){let t={};if(e)t["X-Warmplane-Profile"]=e;return(await fetch(`${this.baseUrl}/v1/capabilities`,{headers:t})).json()}async listResources(e){let t={};if(e)t["X-Warmplane-Profile"]=e;return(await fetch(`${this.baseUrl}/v1/resources`,{headers:t})).json()}async readResource(e,t){let n=performance.now(),i={"Content-Type":"application/json"};if(t)i["X-Warmplane-Profile"]=t;let a=await fetch(`${this.baseUrl}/v1/resources/read`,{method:"POST",headers:i,body:JSON.stringify(e)}),r=performance.now()-n,s=await a.json();return{status:a.status,durationMs:r,data:s}}async listPrompts(e){let t={};if(e)t["X-Warmplane-Profile"]=e;return(await fetch(`${this.baseUrl}/v1/prompts`,{headers:t})).json()}async getPrompt(e,t){let n=performance.now(),i={"Content-Type":"application/json"};if(t)i["X-Warmplane-Profile"]=t;let a=await fetch(`${this.baseUrl}/v1/prompts/get`,{method:"POST",headers:i,body:JSON.stringify(e)}),r=performance.now()-n,s=await a.json();return{status:a.status,durationMs:r,data:s}}async getCatalogEvents(e){let t=e?`?after=${encodeURIComponent(e)}`:"";return(await fetch(`${this.baseUrl}/v1/catalog/events${t}`)).json()}async callCapability(e,t){let n=performance.now(),i={"Content-Type":"application/json"};if(t)i["X-Warmplane-Profile"]=t;let a=await fetch(`${this.baseUrl}/v1/tools/call`,{method:"POST",headers:i,body:JSON.stringify(e)}),r=performance.now()-n,s=await a.json();return{status:a.status,durationMs:r,data:s}}async batchCallCapabilities(e,t){let n=performance.now(),i={"Content-Type":"application/json"};if(t)i["X-Warmplane-Profile"]=t;let a=await fetch(`${this.baseUrl}/v1/tools/batch_call`,{method:"POST",headers:i,body:JSON.stringify({steps:e})}),r=performance.now()-n,s=await a.json();return{status:a.status,durationMs:r,data:s}}async cancelOperation(e){return(await fetch(`${this.baseUrl}/v1/operations/${encodeURIComponent(e)}/cancel`,{method:"POST"})).json()}async completeArgument(e){return(await fetch(`${this.baseUrl}/v1/completion/complete`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify(e)})).json()}async upsertServer(e,t){return(await fetch(`${this.baseUrl}/v1/config/servers`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({name:e,server:t})})).json()}async deleteServer(e){return(await fetch(`${this.baseUrl}/v1/config/servers/${encodeURIComponent(e)}`,{method:"DELETE"})).json()}async restartServer(e){return(await fetch(`${this.baseUrl}/v1/config/servers/${encodeURIComponent(e)}/restart`,{method:"POST"})).json()}async upsertProfile(e,t,n,i){return(await fetch(`${this.baseUrl}/v1/config/profiles`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({name:e,servers:t,description:n,policy:i})})).json()}async deleteProfile(e){return(await fetch(`${this.baseUrl}/v1/config/profiles/${encodeURIComponent(e)}`,{method:"DELETE"})).json()}async getEcosystemSources(){return(await fetch(`${this.baseUrl}/v1/config/ecosystem`)).json()}async importConfig(e,t=!1){return(await fetch(`${this.baseUrl}/v1/config/import`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({source_path:e,overwrite:t})})).json()}async savePolicy(e){let t={allow:e.allow||[],deny:e.deny||[],redactKeys:e.redact_keys||e.redactKeys||[],requireApproval:e.require_approval||e.requireApproval||[],approvalTimeoutSecs:e.approvalTimeoutSecs||e.approval_timeout_secs||300,webhook:e.webhook};return(await fetch(`${this.baseUrl}/v1/config/policy`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify(t)})).json()}async listTasks(){return(await fetch(`${this.baseUrl}/v1/tasks`)).json()}async listUndo(){return(await fetch(`${this.baseUrl}/v1/undo`)).json()}async testUndoContract(e){return(await fetch(`${this.baseUrl}/v1/undo/test`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({alias:e})})).json()}async executeUndo(e){return(await fetch(`${this.baseUrl}/v1/undo/${encodeURIComponent(e)}/execute`,{method:"POST",headers:{"Content-Type":"application/json"},body:"{}"})).json()}async rollbackWorkItem(e){return(await fetch(`${this.baseUrl}/v1/undo/work-items/${encodeURIComponent(e)}/rollback`,{method:"POST",headers:{"Content-Type":"application/json"},body:"{}"})).json()}async getTask(e){return(await fetch(`${this.baseUrl}/v1/tasks/${encodeURIComponent(e)}`)).json()}async updateTask(e,t){return(await fetch(`${this.baseUrl}/v1/tasks/${encodeURIComponent(e)}/update`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({inputResponses:t})})).json()}async cancelTask(e,t){return(await fetch(`${this.baseUrl}/v1/tasks/${encodeURIComponent(e)}/cancel`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({reason:t})})).json()}async listApprovals(){return(await fetch(`${this.baseUrl}/v1/approvals`)).json()}async approveTicket(e,t,n){return(await fetch(`${this.baseUrl}/v1/approvals/${encodeURIComponent(e)}/approve`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({operator:t,modified_args:n})})).json()}async rejectTicket(e,t,n){return(await fetch(`${this.baseUrl}/v1/approvals/${encodeURIComponent(e)}/reject`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({operator:t,reason:n})})).json()}async updateAlias(e,t,n,i,a,r){return(await fetch(`${this.baseUrl}/v1/config/alias`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({kind:e,alias:t,target:n,summary:i,description:a,passthrough:r})})).json()}async reloadConfig(){return(await fetch(`${this.baseUrl}/v1/config/reload`,{method:"POST"})).json()}async listAuditEvents(e){let t=new URLSearchParams;if(e?.actor_id)t.set("actor_id",e.actor_id);if(e?.server_id&&e.server_id!=="all")t.set("server_id",e.server_id);if(e?.capability_id)t.set("capability_id",e.capability_id);if(e?.event_type&&e.event_type!=="all")t.set("event_type",e.event_type);if(e?.status&&e.status!=="all")t.set("status",e.status);if(e?.trace_id)t.set("trace_id",e.trace_id);if(e?.request_id)t.set("request_id",e.request_id);if(e?.search)t.set("search",e.search);if(e?.limit)t.set("limit",String(e.limit));if(e?.offset!==void 0)t.set("offset",String(e.offset));let n=t.toString();return(await fetch(`${this.baseUrl}/v1/audit/events${n?`?${n}`:""}`)).json()}getAuditExportUrl(e,t="csv"){let n=new URLSearchParams;if(n.set("format",t),e?.actor_id)n.set("actor_id",e.actor_id);if(e?.server_id&&e.server_id!=="all")n.set("server_id",e.server_id);if(e?.capability_id)n.set("capability_id",e.capability_id);if(e?.event_type&&e.event_type!=="all")n.set("event_type",e.event_type);if(e?.status&&e.status!=="all")n.set("status",e.status);if(e?.trace_id)n.set("trace_id",e.trace_id);if(e?.request_id)n.set("request_id",e.request_id);if(e?.search)n.set("search",e.search);return`${this.baseUrl}/v1/audit/export?${n.toString()}`}async verifyAuditChain(){return(await fetch(`${this.baseUrl}/v1/audit/verify`)).json()}async getAuditStats(){return(await fetch(`${this.baseUrl}/v1/audit/stats`)).json()}async getClients(){return(await fetch(`${this.baseUrl}/v1/clients`)).json()}async attachClient(e,t,n="stdio",i){return(await fetch(`${this.baseUrl}/v1/clients/${encodeURIComponent(e)}/attach`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({profile:t||void 0,transport:n,http_url:i||void 0})})).json()}async detachClient(e){return(await fetch(`${this.baseUrl}/v1/clients/${encodeURIComponent(e)}/detach`,{method:"POST",headers:{"Content-Type":"application/json"}})).json()}async testWebhook(e,t){return(await fetch(`${this.baseUrl}/v1/webhooks/test`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({url:e||void 0,format:t||void 0})})).json()}async getSecrets(){return(await fetch(`${this.baseUrl}/v1/secrets`)).json()}async saveSecret(e,t,n){return(await fetch(`${this.baseUrl}/v1/secrets`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({key:e,value:t,service:n})})).json()}async deleteSecret(e){return(await fetch(`${this.baseUrl}/v1/secrets/${encodeURIComponent(e)}`,{method:"DELETE"})).json()}}var b=new Q;function Y(){let e=p.getState(),t=e.config.mcpServers||{},n=Object.keys(t),i=n.length,a="";if(n.length===0)a=`
      <div style="grid-column: 1 / -1; padding: 32px; text-align: center; color: var(--text-dim); background: var(--surface-card); border-radius: var(--radius-md); border: 1px dashed var(--border);">
        <div style="font-size: 14px; color: var(--text-main); font-weight: 600; margin-bottom: 6px;">No Upstream MCP Servers Connected</div>
        <div style="font-size: 12px; margin-bottom: 16px;">Initialize connections by adding a server or syncing existing IDE configurations.</div>
        <div style="display: flex; gap: 8px; justify-content: center;">
          <button class="btn btn-primary" onclick="window.app.openTemplateCatalog()">✨ Browse Templates</button>
          <button class="btn btn-ghost" onclick="window.app.openAddServerModal()">+ Add Custom</button>
          <button class="btn btn-ghost" onclick="window.app.openImportModal()">Sync from IDEs</button>
        </div>
      </div>
    `;else a=n.map((S)=>{let P=t[S],q=P.command?"stdio":"http / sse",H=P.command?`${P.command} ${(P.args||[]).join(" ")}`:P.url,L=e.serverStatuses[S]||{status:"connected",protocol_version:"2026-07-28"},F=L.status==="degraded",V=L.status==="error"||L.status==="disconnected",U=F?"var(--amber-400)":V?"var(--red-400)":"var(--green-400)";return`
        <div class="bento-card col-4" style="background: var(--surface); border: 1px solid var(--border);">
          <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
            <span style="font-weight: 700; color: var(--text-main); display: flex; align-items: center; gap: 6px;">
              <span style="width: 8px; height: 8px; border-radius: 50%; background: ${U}; display: inline-block;"></span>
              ${o(S)}
            </span>
            <span class="brand-badge">${q}</span>
          </div>
          <div style="font-family: var(--ff-mono); font-size: 11.5px; color: var(--text-dim); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; margin-bottom: 12px;" title="${o(H||"")}">
            ${o(H||"")}
          </div>
          <div style="display: flex; justify-content: space-between; font-size: 11px; color: var(--text-muted); border-top: 1px solid var(--border); padding-top: 8px;">
            <span>Status: <strong style="color: ${U};">${o(L.status)}</strong></span>
            <span>Protocol: ${L.protocol_version}</span>
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
  `:e.eventLogs.map((S)=>`
    <div class="feed-row" style="grid-template-columns: 80px 100px 1fr 100px 80px;">
      <span style="color: var(--text-dim);">${o(S.time)}</span>
      <span style="color: var(--cyan-400); font-weight: 600;">${o(S.method)}</span>
      <span style="color: var(--text-main); font-family: var(--ff-mono);">${o(S.target)}</span>
      <span style="color: var(--green-400);">${o(S.status)}</span>
      <span style="color: var(--amber-300); text-align: right;">${o(S.latency)}</span>
    </div>
  `).join(""),s=e.metrics,{totalCatalogRequests:l,totalEtagHits:d}=s,g=l>0?`${(d/l*100).toFixed(1)}%`:"0.0%",u=l>0?`${d} of ${l} requests served via HTTP 304`:"Waiting for client requests",m=s.totalToolCalls,v=m>0?`${(s.totalToolDurationUs/m/1000).toFixed(1)}ms`:"0.0ms",c=m>0?`${m} tool executions processed`:"Local worker task queues warm",f=Object.keys(e.config.capabilityAliases||{}).length+Object.keys(e.config.resourceAliases||{}).length+Object.keys(e.config.promptAliases||{}).length,h=f>0?`${f*18}B / call`:"0B",y=f>0?`${f} active facade aliases pruning prompt size`:"Configure aliases in Studio to reduce prompt size",x=e.tasks||[],T=x.filter((S)=>S.status==="input_required").length,I=x.filter((S)=>S.status==="working"||S.status==="input_required").length,E=e.clients||[],w=E.filter((S)=>S.is_attached).length,k=E.filter((S)=>S.config_exists&&!S.is_attached).length,C=e.clientsCollapsed,_=w>0?`<span class="brand-badge" style="color: var(--green-400); border-color: rgba(52, 211, 153, 0.3); background: rgba(52, 211, 153, 0.1);">⚡ ${w} Connected</span>`:k>0?`<span class="brand-badge" style="color: var(--amber-300); border-color: rgba(251, 191, 36, 0.3); background: rgba(251, 191, 36, 0.1);">○ ${k} Ready to Connect</span>`:'<span class="brand-badge" style="color: var(--text-dim);">No Apps Detected</span>',R=Object.keys(e.config.profiles||{}),M=e.activeProfile,z=e.clientFilterCategory||"all",j=(e.clientSearchQuery||"").toLowerCase().trim(),O=E.filter((S)=>{if(j){let P=S.name.toLowerCase().includes(j),q=S.id.toLowerCase().includes(j),H=S.category.toLowerCase().includes(j);if(!P&&!q&&!H)return!1}if(z==="connected")return S.is_attached;if(z==="ready")return S.config_exists||S.app_installed;if(z==="ides")return S.category.toLowerCase().includes("ide")||S.category.toLowerCase().includes("extension");if(z==="agents")return S.category.toLowerCase().includes("agent")||S.category.toLowerCase().includes("cli")||S.category.toLowerCase().includes("platform");return!0}),J=O.length===0?'<div style="padding: 16px; text-align: center; color: var(--text-dim); font-size: 11.5px; grid-column: 1 / -1;">No AI clients match current filter.</div>':O.map((S)=>{let{is_attached:P,config_exists:q,app_installed:H}=S,L="rgba(255, 255, 255, 0.2)",F="Not Found";if(P){L="var(--green-400)";let B=S.attached_transport==="http"?"HTTP":"stdio";F=S.attached_profile?`Connected · ${B} (${S.attached_profile})`:`Connected · ${B} (All Tools)`}else if(q)L="var(--amber-300)",F="Ready to Attach";else if(H)L="var(--cyan-400)",F="Installed";let V=R.map((B)=>`
      <option value="${o(B)}" ${M===B||S.attached_profile===B?"selected":""}>${o(B)}</option>
    `).join(""),U=P?`<button class="btn btn-ghost" style="padding: 2px 7px; font-size: 10px; color: var(--red-400);" onclick="event.stopPropagation(); window.app.detachClient('${o(S.id)}')">Detach</button>`:q||H?`
        <div style="display: flex; align-items: center; gap: 4px;" onclick="event.stopPropagation();">
          ${R.length>0?`
            <select id="overview-client-prof-${o(S.id)}" class="form-input" style="font-size: 10px; padding: 1px 4px; height: 22px; width: 85px;" title="Select constellation profile">
              <option value="" ${!M?"selected":""}>All Tools</option>
              ${V}
            </select>
          `:""}
          <button class="btn btn-primary" style="padding: 2px 7px; font-size: 10px;" onclick="window.app.attachClient('${o(S.id)}')">⚡ Connect</button>
        </div>
      `:"";return`
      <div style="background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 8px 12px; display: flex; align-items: center; justify-content: space-between; gap: 8px;">
        <div style="display: flex; align-items: center; gap: 8px; overflow: hidden;">
          <span style="width: 7px; height: 7px; border-radius: 50%; background: ${L}; flex-shrink: 0;"></span>
          <div style="overflow: hidden;">
            <div style="font-weight: 600; font-size: 12px; color: var(--text-main); white-space: nowrap; overflow: hidden; text-overflow: ellipsis;">${o(S.name)}</div>
            <div style="font-size: 10px; color: var(--text-dim);">${o(F)}</div>
          </div>
        </div>
        ${U}
      </div>
    `}).join(""),W=[{id:"all",label:"All"},{id:"ready",label:"Ready / Installed"},{id:"connected",label:"Connected"},{id:"ides",label:"IDEs"},{id:"agents",label:"Agents & CLIs"}].map((S)=>`<button class="btn btn-ghost" style="padding: 2px 8px; font-size: 10.5px; border-radius: 100px; ${z===S.id?"background: var(--amber-400); color: #000; font-weight: 700;":"background: var(--surface); color: var(--text-muted);"}" onclick="event.stopPropagation(); window.app.setClientCategoryFilter('${S.id}')">${S.label}</button>`).join(""),A=`
    <div class="bento-card" style="margin-top: 18px; padding: 12px 16px; border-color: rgba(245, 158, 11, 0.25); background: rgba(18, 24, 38, 0.4);">
      <div style="display: flex; justify-content: space-between; align-items: center; cursor: pointer; user-select: none;" onclick="window.app.toggleClientsCollapse()">
        <div style="display: flex; align-items: center; gap: 10px;">
          <span style="font-size: 13.5px; font-weight: 700; color: var(--text-main); display: flex; align-items: center; gap: 6px;">
            <span>⚡ 1-Click AI Client Integrations</span>
          </span>
          ${_}
        </div>
        <div style="display: flex; align-items: center; gap: 8px;">
          <button class="btn btn-ghost" style="padding: 2px 8px; font-size: 11px;" onclick="event.stopPropagation(); window.app.refreshClients()">⟳ Scan</button>
          <span style="font-size: 12px; color: var(--text-dim);">${C?"▼ Show":"▲ Hide"}</span>
        </div>
      </div>

      ${!C?`
        <div style="margin-top: 10px; padding-top: 8px; border-top: 1px solid var(--border-subtle);">
          <div style="display: flex; justify-content: space-between; align-items: center; gap: 8px; margin-bottom: 8px;">
            <div style="display: flex; gap: 4px; flex-wrap: wrap;">
              ${W}
            </div>
            <input type="text" class="form-input" style="font-size: 10.5px; padding: 2px 8px; width: 140px; height: 22px; border-radius: 100px;" placeholder="\uD83D\uDD0D Search..." value="${o(e.clientSearchQuery||"")}" onclick="event.stopPropagation();" oninput="window.app.setClientSearchQuery(this.value)">
          </div>
          <div style="display: grid; grid-template-columns: repeat(auto-fill, minmax(270px, 1fr)); gap: 8px;">
            ${J}
          </div>
        </div>
      `:""}
    </div>
  `;return`
    <div class="bento-grid">
      <div class="bento-card col-3">
        <div class="stat-label">Token Savings Rate</div>
        <div class="stat-value" style="color: var(--amber-300);">${h}</div>
        <div class="stat-sub">${y}</div>
      </div>
      <div class="bento-card col-3">
        <div class="stat-label">ETag Cache Hit Rate</div>
        <div class="stat-value" style="color: var(--cyan-400);">${g}</div>
        <div class="stat-sub">${u}</div>
      </div>
      <div class="bento-card col-3">
        <div class="stat-label">Tasks &amp; HITL State</div>
        <div class="stat-value" style="color: ${T>0?"var(--amber-400)":"var(--green-400)"};">${T>0?`${T} Action Req`:`${I} Active`}</div>
        <div class="stat-sub">${T>0?"Awaiting Human-in-the-Loop decision":`${x.length} total registered tasks`}</div>
      </div>
      <div class="bento-card col-3">
        <div class="stat-label">Connected Upstreams</div>
        <div class="stat-value" style="color: var(--green-400);">${i} Active</div>
        <div class="stat-sub">${i>0?"Persistent worker task channels":"No active upstream servers"}</div>
      </div>
    </div>

    ${A}

    <div style="display: flex; justify-content: space-between; align-items: center; margin: 24px 0 12px;">
      <div style="font-size: 15px; font-weight: 700; color: var(--text-main);">Connected Upstream Servers</div>
      <button class="btn btn-ghost" onclick="window.app.switchTab('servers')">Manage All (${i}) →</button>
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
        ${r}
      </div>
    </div>
  `}var D=[{id:"github",name:"GitHub",category:"devtools",description:"Explore repositories, issues, pull requests, branches, and commit histories.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-github"],envFields:[{key:"GITHUB_PERSONAL_ACCESS_TOKEN",label:"GitHub Personal Access Token",placeholder:"ghp_...",required:!0,description:"Classic or fine-grained token with repo scope."}]},{id:"git",name:"Git (Local)",category:"devtools",description:"Read local Git repository status, diffs, log histories, and commit changes.",badge:"Official / uvx",command:"uvx",defaultArgs:["--with","mcp<2","mcp-server-git","--repository","."],argsPlaceholder:"--with mcp<2 mcp-server-git --repository /path/to/repo",envFields:[]},{id:"filesystem",name:"Filesystem",category:"devtools",description:"Secure, sandboxed access to local files and directories for AI workflows.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-filesystem","."],argsPlaceholder:"-y @modelcontextprotocol/server-filesystem /allowed/dir1 /allowed/dir2",envFields:[]},{id:"memory",name:"Memory Graph",category:"devtools",description:"Persistent knowledge-graph based memory for multi-turn agent learning.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-memory"],envFields:[]},{id:"chrome-devtools",name:"Chrome DevTools",category:"devtools",description:"Inspect live DOM, execute scripts, read console logs, and capture network traces in Chrome.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-puppeteer"],envFields:[]},{id:"sentry",name:"Sentry",category:"devtools",description:"Query production error events, stack traces, and issue frequencies directly from Sentry.",badge:"uvx / Telemetry",command:"uvx",defaultArgs:["--with","mcp<2","--with","httpx","mcp-server-sentry","--auth-token","sntrys_token"],argsPlaceholder:"--with mcp<2 --with httpx mcp-server-sentry --auth-token YOUR_SENTRY_TOKEN",envFields:[{key:"SENTRY_AUTH_TOKEN",label:"Sentry Auth Token",placeholder:"sntrys_...",required:!0}]},{id:"playwright",name:"Playwright Browser",category:"browser",description:"Headless / headed browser automation for scraping, form filling, and UI interaction.",badge:"Popular #1 / npx",command:"npx",defaultArgs:["-y","@executeautomation/playwright-mcp-server"],envFields:[]},{id:"puppeteer",name:"Puppeteer",category:"browser",description:"Official browser automation server for web page scraping and screenshot capture.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-puppeteer"],envFields:[]},{id:"brave-search",name:"Brave Search",category:"browser",description:"Real-time privacy-preserving web search and local point-of-interest query engine.",badge:"Official / Search",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-brave-search"],envFields:[{key:"BRAVE_API_KEY",label:"Brave Search API Key",placeholder:"BSA...",required:!0}]},{id:"tavily",name:"Tavily Search",category:"browser",description:"AI-optimized web search engine structured specifically for LLM context injection.",badge:"Community / Stdio",command:"npx",defaultArgs:["-y","tavily-mcp"],envFields:[{key:"TAVILY_API_KEY",label:"Tavily API Key",placeholder:"tvly-...",required:!0}]},{id:"fetch",name:"Fetch / Web Markdown",category:"browser",description:"Download web pages, strip clutter, and convert raw HTML to clean markdown text.",badge:"Official / uvx",command:"uvx",defaultArgs:["mcp-server-fetch"],envFields:[]},{id:"postgres",name:"PostgreSQL",category:"database",description:"Read schemas, inspect tables, and execute SQL queries against PostgreSQL databases.",badge:"Official / Database",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-postgres","postgresql://user:pass@localhost:5432/mydb"],argsPlaceholder:"-y @modelcontextprotocol/server-postgres postgresql://user:pass@localhost:5432/dbname",envFields:[]},{id:"sqlite",name:"SQLite",category:"database",description:"Local embedded SQLite query runner and schema inspector.",badge:"Official / uvx",command:"uvx",defaultArgs:["--with","mcp<2","mcp-server-sqlite","--db-path","./app.db"],argsPlaceholder:"--with mcp<2 mcp-server-sqlite --db-path /path/to/database.sqlite",envFields:[]},{id:"supabase",name:"Supabase",category:"database",description:"Query database tables, manage auth policies, and inspect storage in Supabase.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@supabase/mcp-server-supabase@latest"],envFields:[{key:"SUPABASE_ACCESS_TOKEN",label:"Supabase Personal Access Token",placeholder:"sbp_...",required:!0},{key:"SUPABASE_PROJECT_REF",label:"Supabase Project Reference ID",placeholder:"abcdefghijklmnop",required:!1}]},{id:"redis",name:"Redis",category:"database",description:"Inspect cached keys, hash sets, lists, TTLs, and pub/sub channels in Redis.",badge:"Official / Key-Value",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-redis","redis://localhost:6379"],argsPlaceholder:"-y @modelcontextprotocol/server-redis redis://localhost:6379",envFields:[]},{id:"s3",name:"AWS S3 / Cloud Storage",category:"database",description:"Browse S3 buckets, fetch object metadata, and download files from cloud storage.",badge:"Community / Stdio",command:"npx",defaultArgs:["-y","@geunoh/s3-mcp-server"],argsPlaceholder:"-y @geunoh/s3-mcp-server",envFields:[{key:"AWS_ACCESS_KEY_ID",label:"AWS Access Key ID",placeholder:"AKIA...",required:!0},{key:"AWS_SECRET_ACCESS_KEY",label:"AWS Secret Access Key",placeholder:"...",required:!0},{key:"AWS_REGION",label:"AWS Region",placeholder:"us-east-1",required:!1}]},{id:"linear",name:"Linear",category:"productivity",description:"Search, create, and triage Linear issues, cycles, teams, and project roadmaps.",badge:"Productivity / Stdio",command:"npx",defaultArgs:["-y","mcp-linear"],envFields:[{key:"LINEAR_API_KEY",label:"Linear API Key",placeholder:"lin_api_...",required:!0}]},{id:"slack",name:"Slack",category:"productivity",description:"Read channels, post messages, inspect threads, and search team discussions.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-slack"],envFields:[{key:"SLACK_BOT_TOKEN",label:"Slack Bot User Token",placeholder:"xoxb-...",required:!0},{key:"SLACK_TEAM_ID",label:"Slack Team ID",placeholder:"T01234567",required:!0}]},{id:"notion",name:"Notion",category:"productivity",description:"Search Notion workspace pages, read nested blocks, and query database entries.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@notionhq/notion-mcp-server"],envFields:[{key:"NOTION_TOKEN",label:"Notion Internal Integration Token",placeholder:"secret_...",required:!0}]},{id:"jira",name:"Jira / Atlassian",category:"productivity",description:"Manage Jira issues, search JQL, read sprint statuses, and inspect boards.",badge:"uvx / Atlassian",command:"uvx",defaultArgs:["--with","mcp<2","mcp-server-jira","--jira-base-url","https://your-domain.atlassian.net"],argsPlaceholder:"--with mcp<2 mcp-server-jira --jira-base-url https://org.atlassian.net",envFields:[{key:"JIRA_TOKEN",label:"Atlassian API Token",placeholder:"ATATT3...",required:!0}]},{id:"google-drive",name:"Google Drive",category:"productivity",description:"Search, list, and read documents, spreadsheets, and drive files.",badge:"Community / Stdio",command:"npx",defaultArgs:["-y","@piotr-agier/google-drive-mcp"],envFields:[{key:"GOOGLE_APPLICATION_CREDENTIALS",label:"Google Credentials JSON Path",placeholder:"/path/to/credentials.json",required:!0}]},{id:"docker",name:"Docker",category:"cloud",description:"Inspect running containers, tail container logs, list images, and manage compose services.",badge:"uvx / DevOps",command:"uvx",defaultArgs:["mcp-server-docker"],envFields:[]},{id:"kubernetes",name:"Kubernetes (K8s)",category:"cloud",description:"Query cluster pods, services, deployment status, and inspect Kubernetes logs.",badge:"Popular / Stdio",command:"npx",defaultArgs:["-y","@strowk/mcp-k8s"],envFields:[{key:"KUBECONFIG",label:"Kubeconfig File Path (Optional)",placeholder:"~/.kube/config",required:!1}]},{id:"cloudflare",name:"Cloudflare",category:"cloud",description:"Manage Cloudflare Workers, KV namespaces, D1 databases, Vectorize indexes, and DNS.",badge:"Official / Cloudflare",command:"npx",defaultArgs:["-y","@cloudflare/mcp-server-cloudflare","run","dummy_account_id"],argsPlaceholder:"-y @cloudflare/mcp-server-cloudflare run YOUR_ACCOUNT_ID",envFields:[{key:"CLOUDFLARE_API_TOKEN",label:"Cloudflare API Token",placeholder:"...",required:!0},{key:"CLOUDFLARE_ACCOUNT_ID",label:"Cloudflare Account ID",placeholder:"...",required:!0}]},{id:"terraform",name:"Terraform",category:"cloud",description:"Inspect Terraform state files, resource dependency graphs, and plan previews.",badge:"Community / IaC",command:"npx",defaultArgs:["-y","@mseep/terraform-mcp-server"],envFields:[]}];function N(e,t,n){let i=D.find((r)=>r.id.toLowerCase()===e.toLowerCase());if(i)return i;let a=`${t||""} ${(n||[]).join(" ")}`.toLowerCase();return D.find((r)=>{let s=`${r.command} ${r.defaultArgs.join(" ")}`.toLowerCase();if(a.includes(r.id.toLowerCase()))return!0;if(r.command&&a.includes(r.command.toLowerCase())&&r.defaultArgs.some((l)=>a.includes(l.toLowerCase())))return!0;return!1})}function X(){let e=p.getState(),t=e.config.mcpServers||{},n=Object.keys(t),i=e.activeProfile,a=i?e.config.profiles?.[i]:void 0,r=!!a,s=a?.servers||[],l="";if(n.length===0)l=`
      <div style="padding: 40px; text-align: center; color: var(--text-dim); background: var(--surface-card); border-radius: var(--radius-md); border: 1px dashed var(--border);">
        <div style="font-size: 15px; color: var(--text-main); font-weight: 600; margin-bottom: 8px;">No Servers Configured in ${o(e.configPath)}</div>
        <p style="font-size: 12px; margin-bottom: 20px; max-width: 480px; margin-left: auto; margin-right: auto;">
          Warmplane bridges local tools and remote MCP servers into one unified facade. Add your first server or import existing configs from Claude Desktop or Cursor.
        </p>
        <div style="display: flex; gap: 8px; justify-content: center;">
          <button class="btn btn-primary" onclick="window.app.openTemplateCatalog()">✨ Browse Templates</button>
          <button class="btn btn-ghost" onclick="window.app.openAddServerModal()">+ Add Custom</button>
          <button class="btn btn-ghost" onclick="window.app.openImportModal()">Sync from IDEs</button>
        </div>
      </div>
    `;else l=n.map((g)=>{let u=t[g],m=u.command?"stdio":"http / sse",v=u.command?`${u.command} ${(u.args||[]).join(" ")}`:u.url,c=e.serverStatuses[g]||{status:"connected",protocol_version:"2026-07-28"},f=!r||s.includes(g),h=N(g,u.command,u.args),y=Object.keys(u.env||{}),x=(h?.envFields||[]).filter((A)=>A.required&&!y.includes(A.key)),T=u.env?Object.entries(u.env).map(([A,S])=>{if(S.startsWith("keychain://"))return`<span class="brand-badge" style="color: var(--cyan-400); border-color: rgba(34, 211, 238, 0.3);">\uD83D\uDD12 ${o(A)} (Keychain)</span>`;if(S.startsWith("op://"))return`<span class="brand-badge" style="color: var(--cyan-400); border-color: rgba(34, 211, 238, 0.3);">\uD83D\uDD12 ${o(A)} (1Password)</span>`;if(S.startsWith("env://"))return`<span class="brand-badge" style="color: var(--amber-300); border-color: rgba(251, 191, 36, 0.3);">\uD83D\uDD12 ${o(A)} (Env)</span>`;return`<span style="color: var(--text-dim);">${o(A)}=***</span>`}):[];for(let A of x)T.push(`<span class="brand-badge" style="color: var(--red-400); border-color: rgba(248, 113, 113, 0.4); background: rgba(248, 113, 113, 0.1);" title="Required environment variable '${o(A.key)}' is missing">⚠️ Missing ${o(A.key)}</span>`);let I=T.length>0?T.join(" "):"None",E=(e.circuitBreakers||[]).find((A)=>A.server_id===g),w='<span class="brand-badge" style="color: var(--green-400); border-color: rgba(52, 211, 153, 0.25);">Circuit: CLOSED</span>';if(E){if(E.state==="open")w=`<span class="brand-badge" style="color: var(--red-400); border-color: rgba(248, 113, 113, 0.4); background: rgba(248, 113, 113, 0.1);">Circuit: OPEN (${E.consecutive_failures} failures)</span>`;else if(E.state==="half_open")w=`<span class="brand-badge" style="color: var(--amber-300); border-color: rgba(251, 191, 36, 0.4); background: rgba(251, 191, 36, 0.1);">Circuit: HALF-OPEN (${E.consecutive_successes} probe)</span>`}let k=u.resilience||e.config.resilience,C=k?`FT: ${k.failureThreshold||3} · Cooldown: ${(k.cooldownMs||30000)/1000}s · AutoRestart: ${k.autoRestart!==!1?"ON":"OFF"}`:"Default Resilience",_=x.length>0,R=c.status==="degraded"||_,M=c.status==="error"||c.status==="disconnected",z=_?"var(--amber-400)":R?"var(--amber-400)":M?"var(--red-400)":"var(--green-400)",j=_?`Status: ${o(c.status)} (Missing Keys)`:`Status: ${o(c.status)}`,O=(R||M)&&(c.error||_)?`
        <div style="background: rgba(239, 68, 68, 0.08); border-left: 3px solid var(--amber-400); border-radius: var(--radius-xs); padding: 8px 12px; margin-top: 8px; display: flex; justify-content: space-between; align-items: center; gap: 8px;">
          <div style="font-size: 11px; color: var(--amber-300); font-family: var(--ff-mono); overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">
            <span style="font-weight: 700; color: var(--amber-400);">⚠️ Diagnostics:</span> ${o(c.error||`Missing required environment variable(s): ${x.map((A)=>A.key).join(", ")}`)}
          </div>
          <button class="btn btn-ghost" style="padding: 2px 8px; font-size: 10.5px; color: var(--amber-300); border-color: rgba(251, 191, 36, 0.3);" onclick="window.app.openServerDiagnosticsModal('${o(g)}')">Details</button>
        </div>
      `:"",J=r?f?`<span class="brand-badge" style="color: var(--green-400); border-color: rgba(34, 197, 94, 0.3); background: rgba(34, 197, 94, 0.08); display: inline-flex; align-items: center; gap: 6px;">
              ✔ IN CONSTELLATION
              <button style="background: none; border: none; color: var(--amber-400); font-size: 10px; cursor: pointer; padding: 0 2px; text-decoration: underline;" onclick="window.app.toggleServerInProfile('${o(i)}', '${o(g)}', false)">Exclude</button>
            </span>`:`<span class="brand-badge" style="color: var(--amber-400); border-color: rgba(245, 158, 11, 0.35); background: rgba(245, 158, 11, 0.08); display: inline-flex; align-items: center; gap: 6px;">
              \uD83D\uDEAB EXCLUDED FROM PROFILE: ${o(i)}
              <button style="background: none; border: none; color: var(--green-400); font-size: 10px; cursor: pointer; padding: 0 2px; text-decoration: underline; font-weight: 700;" onclick="window.app.toggleServerInProfile('${o(i)}', '${o(g)}', true)">+ Include</button>
            </span>`:"";return`
        <div class="bento-card" style="${r&&!f?"margin-bottom: 12px; opacity: 0.65; border: 1px dashed rgba(245, 158, 11, 0.4); background: rgba(0, 0, 0, 0.2);":`margin-bottom: 12px; border-color: ${R?"rgba(251, 191, 36, 0.3)":M?"rgba(248, 113, 113, 0.3)":"var(--border)"};`}">
          <div style="display: flex; justify-content: space-between; align-items: center; gap: 16px; flex-wrap: wrap;">
            <div style="flex: 1; min-width: 260px;">
              <div style="display: flex; align-items: center; gap: 8px; margin-bottom: 4px; flex-wrap: wrap;">
                <span style="width: 8px; height: 8px; border-radius: 50%; background: ${z}; display: inline-block;"></span>
                <span style="font-size: 15px; font-weight: 700; color: var(--text-main);">${o(g)}</span>
                <span class="brand-badge">${m}</span>
                <span class="brand-badge" style="color: ${z}; border-color: ${_?"rgba(245, 158, 11, 0.5); background: rgba(245, 158, 11, 0.1);":"rgba(245, 158, 11, 0.3);"}">${j}</span>
                <span class="brand-badge" style="color: var(--cyan-400); border-color: rgba(34, 211, 238, 0.25);">Protocol: ${c.protocol_version}</span>
                ${w}
                ${J}
              </div>
              <div style="font-family: var(--ff-mono); font-size: 12px; color: var(--text-muted); margin-top: 4px;">
                ${u.command?"Command: ":"URL: "}<code>${o(v||"")}</code>
              </div>
              <div style="display: flex; gap: 14px; font-family: var(--ff-mono); font-size: 11px; color: var(--text-dim); margin-top: 4px; align-items: center; flex-wrap: wrap;">
                <span>\uD83D\uDEE1️ ${o(C)}</span>
                ${u.env&&Object.keys(u.env).length>0?`<span>Env: ${I}</span>`:""}
              </div>
            </div>
            <div style="display: flex; gap: 8px; align-items: center; flex-shrink: 0;">
              <button class="btn btn-ghost" style="padding: 4px 10px; font-size: 11.5px; color: var(--cyan-400); border-color: rgba(34, 211, 238, 0.3);" onclick="window.app.restartServer('${o(g)}')">⚡ Restart</button>
              <button class="btn btn-ghost" style="padding: 4px 10px; font-size: 11.5px;" onclick="window.app.openServerDiagnosticsModal('${o(g)}')">\uD83D\uDD0D Diagnostics</button>
              <button class="btn btn-ghost" style="padding: 4px 10px; font-size: 11.5px;" onclick="window.app.openEditServerModal('${o(g)}')">✏️ Edit</button>
              <button class="btn btn-danger" style="padding: 4px 10px; font-size: 11.5px;" onclick="window.app.deleteServer('${o(g)}')">${r?"Delete Globally":"Remove"}</button>
            </div>
          </div>
          ${O}
        </div>
      `}).join("");let d=r?`
    <div class="bento-card" style="margin-bottom: 16px; background: rgba(245, 158, 11, 0.05); border: 1px solid rgba(245, 158, 11, 0.3); display: flex; justify-content: space-between; align-items: center; gap: 14px; flex-wrap: wrap;">
      <div style="display: flex; align-items: center; gap: 10px; flex: 1; min-width: 260px;">
        <span style="font-size: 18px; flex-shrink: 0;">\uD83C\uDF0C</span>
        <div>
          <div style="font-size: 13px; font-weight: 700; color: var(--amber-400); display: flex; align-items: center; gap: 8px; flex-wrap: wrap;">
            <span>Active Profile Constellation: <code style="font-size: 13px; color: var(--text-main);">${o(i)}</code></span>
            <span class="brand-badge" style="color: var(--text-main);">${s.length} of ${n.length} servers included</span>
          </div>
          <div style="font-size: 11.5px; color: var(--text-muted); margin-top: 2px;">
            Excluded servers are unavailable to clients connected via this profile. Tools from excluded servers are automatically hidden.
          </div>
        </div>
      </div>
      <div style="display: flex; gap: 8px; flex-shrink: 0;">
        <button class="btn btn-ghost" style="font-size: 11px; padding: 4px 10px;" onclick="window.app.switchTab('profiles')">Manage Profiles</button>
        <button class="btn btn-ghost" style="font-size: 11px; padding: 4px 10px;" onclick="window.app.setActiveProfile(null)">View All Servers</button>
      </div>
    </div>
  `:"";return`
    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 18px;">
      <div>
        <div style="font-size: 16px; font-weight: 700; color: var(--text-main);">Configured MCP Upstream Servers</div>
        <div style="font-size: 11px; color: var(--text-dim);">Active configuration file: <code>${o(e.configPath)}</code></div>
      </div>
      <div style="display: flex; gap: 8px;">
        <button class="btn btn-ghost" onclick="window.app.reloadFromDisk()">⟳ Reload Config</button>
      </div>
    </div>

    ${d}

    ${l}

    ${de()}
  `}function de(){let e=p.getState(),t=e.clients||[],n=Object.keys(e.config.profiles||{}),i=e.clientsCollapsed,a=e.clientFilterCategory||"all",r=(e.clientSearchQuery||"").toLowerCase().trim();if(t.length===0)return"";let s=t.filter((c)=>c.is_attached).length,l=t.filter((c)=>c.config_exists&&!c.is_attached).length,d=t.filter((c)=>c.category.toLowerCase().includes("ide")||c.category.toLowerCase().includes("extension")).length,g=t.filter((c)=>c.category.toLowerCase().includes("agent")||c.category.toLowerCase().includes("cli")||c.category.toLowerCase().includes("platform")).length,u=t.filter((c)=>{if(r){let f=c.name.toLowerCase().includes(r),h=c.id.toLowerCase().includes(r),y=c.category.toLowerCase().includes(r),x=c.config_path.toLowerCase().includes(r);if(!f&&!h&&!y&&!x)return!1}if(a==="connected")return c.is_attached;if(a==="ready")return c.config_exists||c.app_installed;if(a==="ides")return c.category.toLowerCase().includes("ide")||c.category.toLowerCase().includes("extension");if(a==="agents")return c.category.toLowerCase().includes("agent")||c.category.toLowerCase().includes("cli")||c.category.toLowerCase().includes("platform");return!0}),m=[{id:"all",label:`All Ecosystems (${t.length})`},{id:"ready",label:`Ready / Installed (${l+s})`},{id:"connected",label:`⚡ Connected (${s})`},{id:"ides",label:`IDEs & Editors (${d})`},{id:"agents",label:`Agents & CLIs (${g})`}].map((c)=>`
      <button class="btn btn-ghost" style="padding: 3px 10px; font-size: 11px; border-radius: 100px; ${a===c.id?"background: var(--amber-400); color: #000; font-weight: 700; border-color: var(--amber-400);":"background: var(--surface); color: var(--text-muted); border-color: var(--border);"}" onclick="window.app.setClientCategoryFilter('${o(c.id)}')">
        ${o(c.label)}
      </button>
    `).join(""),v=u.length===0?`<div style="padding: 24px; text-align: center; color: var(--text-dim); font-size: 12px;">No AI clients match the filter "${o(r||a)}".</div>`:u.map((c)=>{let{is_attached:f,config_exists:h,app_installed:y}=c,x='<span class="brand-badge" style="color: var(--text-dim); border-color: rgba(255, 255, 255, 0.1);">Not Found</span>';if(f){let w=c.attached_profile?` · ${c.attached_profile}`:"";x=`<span class="brand-badge" style="color: var(--green-400); border-color: rgba(52, 211, 153, 0.3); background: rgba(52, 211, 153, 0.1);">⚡ Connected · ${c.attached_transport==="http"?"HTTP":"stdio"}${o(w)}</span>`}else if(h)x='<span class="brand-badge" style="color: var(--amber-300); border-color: rgba(251, 191, 36, 0.3); background: rgba(251, 191, 36, 0.08);">○ Ready</span>';else if(y)x='<span class="brand-badge" style="color: var(--cyan-400); border-color: rgba(34, 211, 238, 0.25);">○ Installed</span>';let T=e.activeProfile,I=n.map((w)=>`
          <option value="${o(w)}" ${T===w||c.attached_profile===w?"selected":""}>Profile: ${o(w)}</option>
        `).join(""),E=f?`<button class="btn btn-ghost" style="padding: 3px 10px; font-size: 11px; color: var(--red-400); border-color: rgba(248, 113, 113, 0.3);" onclick="window.app.detachClient('${o(c.id)}')">Disconnect</button>`:`<button class="btn btn-primary" style="padding: 3px 10px; font-size: 11px;" onclick="window.app.attachClient('${o(c.id)}')">⚡ Connect</button>`;return`
          <div style="display: grid; grid-template-columns: 200px 130px 1fr 140px 100px; align-items: center; gap: 12px; padding: 8px 12px; background: var(--surface); border: 1px solid var(--border-subtle); border-radius: var(--radius-sm); transition: background 0.15s;" onmouseover="this.style.background='var(--surface-hover)'" onmouseout="this.style.background='var(--surface)'">
            <div>
              <div style="font-weight: 700; font-size: 13px; color: var(--text-main); display: flex; align-items: center; gap: 6px;">
                <span>${o(c.name)}</span>
              </div>
              <div style="font-size: 10.5px; color: var(--text-dim);">${o(c.category)}</div>
            </div>

            <div>
              ${x}
            </div>

            <div style="font-family: var(--ff-mono); font-size: 10.5px; color: var(--text-muted); overflow: hidden; text-overflow: ellipsis; white-space: nowrap;" title="${o(c.config_path)}">
              ${o(c.config_path)}
            </div>

            <div>
              ${n.length>0&&!f?`
                <select id="client-prof-${o(c.id)}" class="form-input" style="font-size: 10.5px; padding: 2px 6px; height: 26px; width: 100%;">
                  <option value="">All Tools (Default)</option>
                  ${I}
                </select>
              `:`<span style="font-size: 11px; color: var(--text-dim);">${c.other_servers_count>0?`${c.other_servers_count} other tools`:"Single facade"}</span>`}
            </div>

            <div style="text-align: right;">
              ${E}
            </div>
          </div>
        `}).join("");return`
    <div class="bento-card" style="margin-top: 28px; padding: 14px 18px; border-color: rgba(245, 158, 11, 0.2); background: rgba(18, 24, 38, 0.4);">
      <div style="display: flex; justify-content: space-between; align-items: center; cursor: pointer; user-select: none;" onclick="window.app.toggleClientsCollapse()">
        <div>
          <div style="font-size: 14px; font-weight: 700; color: var(--text-main); display: flex; align-items: center; gap: 8px;">
            <span>⚡ 1-Click AI Client Integrations</span>
            <span class="brand-badge" style="color: var(--amber-400); border-color: rgba(245, 158, 11, 0.3);">${s>0?`${s} Connected`:`${t.length} Ecosystems Supported`}</span>
          </div>
          <div style="font-size: 11px; color: var(--text-dim); margin-top: 2px;">
            Attach Warmplane's unified facade to desktop IDEs, CLI assistants, and autonomous agent platforms without editing JSON/TOML files.
          </div>
        </div>
        <div style="display: flex; align-items: center; gap: 8px;">
          <button class="btn btn-ghost" style="padding: 3px 8px; font-size: 11px;" onclick="event.stopPropagation(); window.app.refreshClients()">⟳ Scan Ecosystems</button>
          <span style="font-size: 12px; color: var(--text-dim);">${i?"▼ Show":"▲ Hide"}</span>
        </div>
      </div>

      ${!i?`
        <div style="margin-top: 14px; padding-top: 12px; border-top: 1px solid var(--border-subtle);">
          <div style="display: flex; flex-wrap: wrap; justify-content: space-between; align-items: center; gap: 10px; margin-bottom: 12px;">
            <div style="display: flex; flex-wrap: wrap; gap: 6px;">
              ${m}
            </div>
            <div style="position: relative;">
              <input type="text" class="form-input" style="font-size: 11px; padding: 4px 10px; width: 180px; height: 26px; border-radius: 100px;" placeholder="\uD83D\uDD0D Search clients..." value="${o(e.clientSearchQuery||"")}" oninput="window.app.setClientSearchQuery(this.value)">
            </div>
          </div>

          <div style="display: flex; flex-direction: column; gap: 6px;">
            <div style="display: grid; grid-template-columns: 200px 130px 1fr 140px 100px; gap: 12px; padding: 4px 12px; font-family: var(--ff-mono); font-size: 10px; color: var(--text-dim); font-weight: 700; text-transform: uppercase;">
              <span>Application</span>
              <span>Status</span>
              <span>Configuration Path</span>
              <span>Constellation Scope</span>
              <span style="text-align: right;">Action</span>
            </div>
            ${v}
          </div>
        </div>
      `:""}
    </div>
  `}function Z(){let e=p.getState(),t=e.playgroundMode||"tools",n=e.capabilities||[],i=e.resources||[],a=e.prompts||[],r=e.capabilitiesHiddenByPolicy||0,s=e.resourcesHiddenByPolicy||0,l=e.promptsHiddenByPolicy||0,d=e.activeProfile,u=!!(d?e.config.profiles?.[d]:void 0),m=`
    <div style="display: flex; gap: 8px; margin-bottom: 12px; align-items: center; justify-content: space-between; flex-wrap: wrap;">
      <div style="display: inline-flex; padding: 3px; background: rgba(0,0,0,0.3); border: 1px solid var(--border); border-radius: var(--radius-sm); align-items: center;">
        <button 
          class="btn ${t==="tools"?"btn-primary":"btn-ghost"}" 
          style="padding: 4px 12px; font-size: 11.5px; height: 28px; display: inline-flex; align-items: center; gap: 6px;"
          onclick="window.app.setPlaygroundMode('tools')"
        >
          <span>\uD83D\uDEE0️ Tools (${n.length})</span>
          ${r>0?`<span class="badge" style="background: rgba(245, 158, 11, 0.2); color: var(--amber-300); font-size: 9.5px; padding: 1px 5px;" title="${r} tools hidden by constellation/policy">+${r} hidden</span>`:""}
        </button>
        <button 
          class="btn ${t==="resources"?"btn-primary":"btn-ghost"}" 
          style="padding: 4px 12px; font-size: 11.5px; height: 28px; display: inline-flex; align-items: center; gap: 6px;"
          onclick="window.app.setPlaygroundMode('resources')"
        >
          <span>\uD83D\uDCC4 Resources (${i.length})</span>
          ${s>0?`<span class="badge" style="background: rgba(245, 158, 11, 0.2); color: var(--amber-300); font-size: 9.5px; padding: 1px 5px;" title="${s} resources hidden by constellation/policy">+${s} hidden</span>`:""}
        </button>
        <button 
          class="btn ${t==="prompts"?"btn-primary":"btn-ghost"}" 
          style="padding: 4px 12px; font-size: 11.5px; height: 28px; display: inline-flex; align-items: center; gap: 6px;"
          onclick="window.app.setPlaygroundMode('prompts')"
        >
          <span>\uD83D\uDCAC Prompts (${a.length})</span>
          ${l>0?`<span class="badge" style="background: rgba(245, 158, 11, 0.2); color: var(--amber-300); font-size: 9.5px; padding: 1px 5px;" title="${l} prompts hidden by constellation/policy">+${l} hidden</span>`:""}
        </button>
      </div>

      <div style="display: flex; align-items: center; gap: 12px;">
        ${t==="tools"&&r>0?`
          <div style="font-size: 11px; color: var(--amber-300); background: rgba(245, 158, 11, 0.1); border: 1px solid rgba(245, 158, 11, 0.25); padding: 3px 8px; border-radius: var(--radius-sm); display: flex; align-items: center; gap: 6px;">
            <span>\uD83D\uDEE1️ ${r} tool${r>1?"s":""} filtered ${u?`(Profile: ${o(d)})`:"by policy"}</span>
            <a href="javascript:void(0)" onclick="window.app.switchTab('policy')" style="color: var(--amber-400); text-decoration: underline; font-weight: 600;">View Policy</a>
            ${u?`<a href="javascript:void(0)" onclick="window.app.switchTab('servers')" style="color: var(--cyan-400); text-decoration: underline; font-weight: 600; margin-left: 4px;">Server Hub</a>`:""}
          </div>
        `:t==="resources"&&s>0?`
          <div style="font-size: 11px; color: var(--amber-300); background: rgba(245, 158, 11, 0.1); border: 1px solid rgba(245, 158, 11, 0.25); padding: 3px 8px; border-radius: var(--radius-sm); display: flex; align-items: center; gap: 6px;">
            <span>\uD83D\uDEE1️ ${s} resource${s>1?"s":""} filtered ${u?`(Profile: ${o(d)})`:"by policy"}</span>
            <a href="javascript:void(0)" onclick="window.app.switchTab('policy')" style="color: var(--amber-400); text-decoration: underline; font-weight: 600;">View Policy</a>
            ${u?`<a href="javascript:void(0)" onclick="window.app.switchTab('servers')" style="color: var(--cyan-400); text-decoration: underline; font-weight: 600; margin-left: 4px;">Server Hub</a>`:""}
          </div>
        `:t==="prompts"&&l>0?`
          <div style="font-size: 11px; color: var(--amber-300); background: rgba(245, 158, 11, 0.1); border: 1px solid rgba(245, 158, 11, 0.25); padding: 3px 8px; border-radius: var(--radius-sm); display: flex; align-items: center; gap: 6px;">
            <span>\uD83D\uDEE1️ ${l} prompt${l>1?"s":""} filtered ${u?`(Profile: ${o(d)})`:"by policy"}</span>
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
      ${m}
      ${pe(e)}
    `;if(t==="prompts")return`
      ${m}
      ${ue(e)}
    `;return`
    ${m}
    ${ce(e)}
    ${e.isBatchModalOpen?ge(e):""}
  `}function K(e,t=!1){if(!e||!e.properties)return{};let n=e.properties||{},i=Array.isArray(e.required)?e.required:[],a={};for(let[r,s]of Object.entries(n)){let l=i.includes(r);if(t&&!l)continue;if(s.default!==void 0)a[r]=s.default;else if(Array.isArray(s.enum)&&s.enum.length>0)a[r]=s.enum[0];else if(s.examples&&Array.isArray(s.examples)&&s.examples.length>0)a[r]=s.examples[0];else if(s.example!==void 0)a[r]=s.example;else switch(s.type||"string"){case"string":a[r]=l?`sample_${r}`:"";break;case"number":case"integer":a[r]=0;break;case"boolean":a[r]=!0;break;case"array":a[r]=[];break;case"object":a[r]={};break;default:a[r]=`sample_${r}`}}return a}function ce(e){let t=e.capabilities||[],n=e.selectedCapabilityId||(t.length>0?t[0].id:null),i=t.find((T)=>T.id===n),a=e.config.capabilityAliases?.[n],r=typeof a==="object"&&!!a?.reversibility?.testTarget,s=typeof a==="object"?a?.reversibility?.testTarget?.server:void 0,l=e.isExecutingCapability,d=e.capabilitiesHiddenByPolicy||0,g=e.activeProfile,u=!!(g&&e.config.profiles?.[g]),m="";if(t.length===0)m=`
      <div style="padding: 24px 16px; text-align: center; color: var(--text-dim); font-size: 11.5px;">
        No tools or capabilities discovered from connected servers.
      </div>
    `;else m=t.map((T)=>`
        <div class="cap-item ${T.id===n?"active":""}" onclick="window.app.selectCapability('${o(T.id)}')">
          <div style="display: flex; justify-content: space-between; align-items: center;">
            <span style="font-weight: 600; color: var(--text-main); font-family: var(--ff-mono); font-size: 12px;">${o(T.id)}</span>
            <span style="font-size: 10px; color: var(--green-400);">${o(T.mode||"read")}</span>
          </div>
          <div style="font-size: 11px; color: var(--text-dim); margin-top: 2px;">server: ${o(T.server||"local")}</div>
        </div>
      `).join("");let v=i?.input_schema,c=v?.properties||{},f=Array.isArray(v?.required)?v.required:[],h=Object.entries(c),y="";if(h.length>0)y=`
      <div style="display: flex; flex-wrap: wrap; gap: 6px; margin-bottom: 8px; align-items: center;">
        <span style="font-size: 10px; font-weight: 700; color: var(--text-dim); text-transform: uppercase;">Fields:</span>
        ${h.map(([T,I])=>{let E=f.includes(T),w=I.type||(I.enum?"enum":"any"),k=E?"rgba(239, 68, 68, 0.15)":"rgba(148, 163, 184, 0.1)",C=E?"var(--red-400)":"var(--text-muted)",_=E?"rgba(239, 68, 68, 0.3)":"var(--border)",R=I.description?` - ${I.description}`:"";return`
            <button 
              type="button" 
              class="btn" 
              style="padding: 2px 7px; font-size: 10.5px; font-family: var(--ff-mono); background: ${k}; color: ${C}; border: 1px solid ${_}; border-radius: var(--radius-sm);"
              title="Click to insert '${o(T)}' (${o(w)}${o(R)})"
              onclick="window.app.insertPlaygroundArgKey('${o(T)}', '${o(w)}', ${o(JSON.stringify(I.default??null))})"
            >
              + ${o(T)} <span style="font-size: 9px; opacity: 0.7;">(${w}${E?" *":""})</span>
            </button>
          `}).join("")}
      </div>
    `;let x="{}";if(n&&e.playgroundArgs&&e.playgroundArgs[n]!==void 0)x=e.playgroundArgs[n];else{let T=K(v,!1);x=JSON.stringify(T,null,2)}return`
    <div style="display: grid; grid-template-columns: 320px 1fr; gap: 16px; height: calc(100vh - 165px);">
      <!-- Left Sidebar: Capabilities Catalog -->
      <div style="background: var(--surface-card); border: 1px solid var(--border); border-radius: var(--radius-md); display: flex; flex-direction: column; overflow: hidden;">
        <div style="padding: 12px; border-bottom: 1px solid var(--border);">
          <input type="text" class="form-input" placeholder="Search ${t.length} capabilities..." oninput="window.app.filterCapabilities(this.value)">
        </div>
        <div style="flex: 1; overflow-y: auto; padding: 8px;" id="pg-cap-list">
          ${m}
        </div>
        ${d>0?`
          <div style="padding: 8px 12px; background: rgba(245, 158, 11, 0.08); border-top: 1px solid rgba(245, 158, 11, 0.2); font-size: 11px; color: var(--amber-300); display: flex; justify-content: space-between; align-items: center;">
            <span>\uD83D\uDEE1️ ${d} tool${d>1?"s":""} filtered ${u?`(${o(g)})`:""}</span>
            <div style="display: flex; gap: 6px;">
              <a href="javascript:void(0)" onclick="window.app.switchTab('policy')" style="color: var(--amber-400); text-decoration: underline; font-weight: 600; font-size: 10.5px;">Policy</a>
              ${u?`<a href="javascript:void(0)" onclick="window.app.switchTab('servers')" style="color: var(--cyan-400); text-decoration: underline; font-weight: 600; font-size: 10.5px;">Servers</a>`:""}
            </div>
          </div>
        `:""}
      </div>

      <!-- Right Panel: Capability Execution & Envelope Visualizer -->
      <div style="background: var(--surface-card); border: 1px solid var(--border); border-radius: var(--radius-md); display: flex; flex-direction: column; overflow: hidden;">
        <div style="padding: 14px 18px; border-bottom: 1px solid var(--border); display: flex; justify-content: space-between; align-items: center;">
          <div>
            <div style="font-size: 15px; font-weight: 700; color: var(--text-main); font-family: var(--ff-mono);" id="pg-selected-title">
              ${o(i?i.id:"No Capability Selected")}
            </div>
            <div style="font-size: 11.5px; color: var(--text-dim);" id="pg-selected-desc">
              ${o(i?i.summary||i.description:"Connect servers to inspect and execute tools")}
            </div>
          </div>
          
          <div style="display: flex; align-items: center; gap: 10px;">
            ${l?`
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
            ${r?`<button class="btn btn-ghost" data-playground-action="test-undo" data-undo-alias="${o(n)}" ${l?"disabled":""} title="Runs the configured forward and inverse cycle on ${o(s)}">Test undo contract</button>`:""}
          </div>
        </div>

        <div style="flex: 1; display: grid; grid-template-columns: 1fr 1fr; overflow: hidden;">
          <!-- Request Builder -->
          <div style="padding: 16px; border-right: 1px solid var(--border); overflow-y: auto;">
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
              <label class="form-label" style="margin: 0;">Arguments JSON</label>
              <div style="display: flex; gap: 6px;">
                <button type="button" class="btn btn-ghost" style="padding: 2px 7px; font-size: 10.5px;" title="Fill sample payload from schema" onclick="window.app.fillPlaygroundSampleArgs(false)">✨ Sample Template</button>
                ${f.length>0?`
                  <button type="button" class="btn btn-ghost" style="padding: 2px 7px; font-size: 10.5px;" title="Fill only required schema fields" onclick="window.app.fillPlaygroundSampleArgs(true)">\uD83E\uDDF9 Required Only</button>
                `:""}
                <button type="button" class="btn btn-ghost" style="padding: 2px 7px; font-size: 10.5px;" title="Format JSON" onclick="window.app.formatPlaygroundArgs()">\uD83D\uDCCB Format</button>
                <button type="button" class="btn btn-ghost" style="padding: 2px 8px; font-size: 11px;" onclick="window.app.openBatchModal()">⚡ Pipeline Builder</button>
              </div>
            </div>

            ${y}

            <textarea class="form-textarea" rows="7" id="pg-args-input" oninput="window.app.updatePlaygroundArgs(this.value)">${o(x)}</textarea>

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
            ${i&&i.input_schema?`
              <div style="margin-top: 14px;">
                <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 4px;">
                  <label class="form-label" style="margin: 0;">Input JSON Schema</label>
                  <span style="font-size: 10px; color: var(--text-dim); font-family: var(--ff-mono);">${h.length} field${h.length===1?"":"s"} (${f.length} required)</span>
                </div>
                <pre style="background: var(--surface); padding: 10px; border-radius: var(--radius-sm); border: 1px solid var(--border); font-size: 11px; color: var(--text-muted); max-height: 140px; overflow-y: auto;">${o(JSON.stringify(i.input_schema,null,2))}</pre>
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
                      ${o(e.executionResult.data?.task?.status||e.executionResult.data?.status||"TASK_CREATED").toUpperCase()}
                    </span>
                    <span style="font-family: var(--ff-mono); font-size: 12px; font-weight: 700; color: var(--text-main);">${o(e.executionResult.data?.task?.taskId||e.executionResult.data?.taskId||"")}</span>
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

            <pre id="pg-response-json" style="flex: 1; background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 14px; color: var(--amber-300); font-size: 11.5px; overflow-y: auto; margin: 0; white-space: pre-wrap; word-break: break-word;">${e.executionResult?o(JSON.stringify(e.executionResult.data,null,2)):"// Response envelope output will be formatted here"}</pre>
          </div>
        </div>
      </div>
    </div>
  `}function pe(e){let t=e.resources||[],n=e.resourcesHiddenByPolicy||0,i=e.selectedResourceId||(t.length>0?t[0].id:null),a=t.find((l)=>l.id===i),r=e.resourceReadResult,s="";if(t.length===0)s=`
      <div style="padding: 24px 16px; text-align: center; color: var(--text-dim); font-size: 11.5px;">
        No resources exposed by connected MCP servers.
      </div>
    `;else s=t.map((l)=>{let d=l.id===i?"active":"",g=l.uri?l.uri.split(":")[0]:"res";return`
        <div class="cap-item ${d}" onclick="window.app.selectResource('${o(l.id)}')">
          <div style="display: flex; justify-content: space-between; align-items: center;">
            <span style="font-weight: 600; color: var(--text-main); font-family: var(--ff-mono); font-size: 12px;">${o(l.name||l.id)}</span>
            <span class="badge" style="font-size: 9.5px; background: rgba(56, 189, 248, 0.15); color: var(--cyan-400);">${o(g)}</span>
          </div>
          <div style="font-size: 11px; color: var(--text-dim); margin-top: 2px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">${o(l.uri)}</div>
          <div style="display: flex; justify-content: space-between; font-size: 10px; color: var(--text-muted); margin-top: 4px;">
            <span>server: ${o(l.server||"local")}</span>
            <span>${o(l.mime_type||"text/plain")}</span>
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
          ${s}
        </div>
        ${n>0?`
          <div style="padding: 8px 12px; background: rgba(245, 158, 11, 0.08); border-top: 1px solid rgba(245, 158, 11, 0.2); font-size: 11px; color: var(--amber-300); display: flex; justify-content: space-between; align-items: center;">
            <span>\uD83D\uDEE1️ ${n} resource${n>1?"s":""} hidden by policy</span>
            <a href="javascript:void(0)" onclick="window.app.switchTab('policy')" style="color: var(--amber-400); text-decoration: underline; font-weight: 600; font-size: 10.5px;">Edit Policy</a>
          </div>
        `:""}
      </div>

      <!-- Right Panel: Resource Content Reader & Metadata Inspector -->
      <div style="background: var(--surface-card); border: 1px solid var(--border); border-radius: var(--radius-md); display: flex; flex-direction: column; overflow: hidden;">
        <div style="padding: 14px 18px; border-bottom: 1px solid var(--border); display: flex; justify-content: space-between; align-items: center;">
          <div>
            <div style="font-size: 15px; font-weight: 700; color: var(--text-main); font-family: var(--ff-mono);">
              ${o(a?a.name||a.id:"No Resource Selected")}
            </div>
            <div style="font-size: 11.5px; color: var(--cyan-400); font-family: var(--ff-mono);">
              ${o(a?a.uri:"Select a resource from the list to read live content")}
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
                  <div><span style="color: var(--text-muted);">Server:</span> <strong style="color: var(--text-main);">${o(a.server)}</strong></div>
                  <div><span style="color: var(--text-muted);">MIME Type:</span> <strong style="color: var(--text-main);">${o(a.mime_type||"text/plain")}</strong></div>
                </div>
                ${a.description?`
                  <div style="margin-top: 8px; font-size: 11.5px; color: var(--text-dim); border-top: 1px solid rgba(255,255,255,0.05); padding-top: 6px;">
                    ${o(a.description)}
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
            <pre style="flex: 1; background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 14px; color: var(--cyan-400); font-size: 11.5px; overflow-y: auto; margin: 0; white-space: pre-wrap; word-break: break-word;">${r?o(JSON.stringify(r.data,null,2)):'// Click "Read Resource Content" to inspect live payload'}</pre>
          </div>
        </div>
      </div>
    </div>
  `}function ue(e){let t=e.prompts||[],n=e.promptsHiddenByPolicy||0,i=e.selectedPromptId||(t.length>0?t[0].id:null),a=t.find((d)=>d.id===i),r=e.promptGetResult,s="";if(t.length===0)s=`
      <div style="padding: 24px 16px; text-align: center; color: var(--text-dim); font-size: 11.5px;">
        No prompt templates registered by connected MCP servers.
      </div>
    `;else s=t.map((d)=>{let g=d.id===i?"active":"",u=d.arguments?d.arguments.length:0;return`
        <div class="cap-item ${g}" onclick="window.app.selectPrompt('${o(d.id)}')">
          <div style="display: flex; justify-content: space-between; align-items: center;">
            <span style="font-weight: 600; color: var(--text-main); font-family: var(--ff-mono); font-size: 12px;">${o(d.name||d.id)}</span>
            <span class="badge" style="font-size: 9.5px; background: rgba(168, 85, 247, 0.15); color: var(--purple-400);">${u} args</span>
          </div>
          <div style="font-size: 11px; color: var(--text-dim); margin-top: 2px;">${o(d.description||d.title||"Prompt template")}</div>
          <div style="font-size: 10px; color: var(--text-muted); margin-top: 4px;">server: ${o(d.server||"local")}</div>
        </div>
      `}).join("");let l="";if(a&&a.arguments&&a.arguments.length>0)l=a.arguments.map((d)=>`
      <div class="form-group" style="margin-bottom: 12px;">
        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 4px;">
          <label class="form-label" style="margin: 0; font-family: var(--ff-mono);">${o(d.name)}</label>
          ${d.required?'<span class="badge" style="background: rgba(239, 68, 68, 0.15); color: var(--red-400); font-size: 9px;">REQUIRED</span>':'<span style="font-size: 10px; color: var(--text-dim);">optional</span>'}
        </div>
        ${d.description?`<div style="font-size: 11px; color: var(--text-dim); margin-bottom: 4px;">${o(d.description)}</div>`:""}
        <input type="text" class="form-input prompt-arg-input" data-arg-name="${o(d.name)}" placeholder="Enter ${o(d.name)}..." />
      </div>
    `).join("");else if(a)l=`
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
          ${s}
        </div>
        ${n>0?`
          <div style="padding: 8px 12px; background: rgba(245, 158, 11, 0.08); border-top: 1px solid rgba(245, 158, 11, 0.2); font-size: 11px; color: var(--amber-300); display: flex; justify-content: space-between; align-items: center;">
            <span>\uD83D\uDEE1️ ${n} prompt${n>1?"s":""} hidden by policy</span>
            <a href="javascript:void(0)" onclick="window.app.switchTab('policy')" style="color: var(--amber-400); text-decoration: underline; font-weight: 600; font-size: 10.5px;">Edit Policy</a>
          </div>
        `:""}
      </div>

      <!-- Right Panel: Prompt Parameter Binder & Message Envelope Preview -->
      <div style="background: var(--surface-card); border: 1px solid var(--border); border-radius: var(--radius-md); display: flex; flex-direction: column; overflow: hidden;">
        <div style="padding: 14px 18px; border-bottom: 1px solid var(--border); display: flex; justify-content: space-between; align-items: center;">
          <div>
            <div style="font-size: 15px; font-weight: 700; color: var(--text-main); font-family: var(--ff-mono);">
              ${o(a?a.name||a.id:"No Prompt Selected")}
            </div>
            <div style="font-size: 11.5px; color: var(--text-dim);">
              ${o(a?a.description||a.title||"Bind variables and render messages":"Select a prompt from the list to test")}
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
            ${l}
          </div>

          <!-- Rendered Messages Output -->
          <div style="padding: 16px; background: var(--bg-app); display: flex; flex-direction: column; overflow: hidden;">
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
              <span style="font-size: 11px; font-weight: 600; color: var(--text-dim);">RENDERED PROMPT MESSAGES</span>
              <span style="font-size: 11px; font-weight: 600; color: ${r?r.status===200?"var(--green-400)":"var(--red-400)":"var(--text-dim)"}; font-family: var(--ff-mono);">
                ${r?`HTTP ${r.status} · ${r.durationMs.toFixed(1)}ms`:"READY"}
              </span>
            </div>
            <pre style="flex: 1; background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 14px; color: #c084fc; font-size: 11.5px; overflow-y: auto; margin: 0; white-space: pre-wrap; word-break: break-word;">${r?o(JSON.stringify(r.data,null,2)):'// Click "Render Prompt Messages" to view resolved system/user messages'}</pre>
          </div>
        </div>
      </div>
    </div>
  `}function ge(e){let t=e.capabilities||[],n=e.batchSteps||[];return`
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
          ${n.map((a,r)=>{let s=t.find((c)=>c.id===a.capability_id),l=s?.input_schema,d=l?.properties||{},g=Array.isArray(l?.required)?l.required:[],u=Object.entries(d),m=t.map((c)=>`
      <option value="${o(c.id)}" ${c.id===a.capability_id?"selected":""}>
        ${o(c.id)} (${o(c.server||"local")})
      </option>
    `).join(""),v="";if(u.length>0)v=`
        <div style="display: flex; flex-wrap: wrap; gap: 4px; margin-top: 6px; margin-bottom: 6px; align-items: center;">
          <span style="font-size: 9.5px; font-weight: 700; color: var(--text-dim); text-transform: uppercase;">Parameters:</span>
          ${u.map(([c,f])=>{let h=g.includes(c),y=f.type||(f.enum?"enum":"any");return`
              <span style="font-size: 9.5px; font-family: var(--ff-mono); padding: 1px 5px; background: ${h?"rgba(239, 68, 68, 0.15)":"rgba(148, 163, 184, 0.1)"}; color: ${h?"var(--red-400)":"var(--text-muted)"}; border: 1px solid ${h?"rgba(239, 68, 68, 0.3)":"var(--border)"}; border-radius: 3px;" title="${o(f.description||"")}">
                ${o(c)} (${y}${h?" *":""})
              </span>
            `}).join("")}
        </div>
      `;return`
      <div style="background: rgba(0,0,0,0.3); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 14px; margin-bottom: 12px;">
        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 10px;">
          <div style="display: flex; align-items: center; gap: 8px;">
            <span class="badge" style="background: rgba(56, 189, 248, 0.15); color: var(--cyan-400); font-family: var(--ff-mono); font-weight: 700;">STEP ${r+1}</span>
            <span style="font-size: 11px; font-family: var(--ff-mono); color: var(--text-dim);">id: ${o(a.id)}</span>
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
              ${m}
            </select>
          </div>
          <div style="display: flex; align-items: flex-end; padding-bottom: 6px;">
            <label style="display: flex; align-items: center; gap: 6px; font-size: 11.5px; color: var(--text-muted); cursor: pointer;">
              <input type="checkbox" ${a.continue_on_error?"checked":""} onchange="window.app.updateBatchStepContinueOnError(${r}, this.checked)" />
              <span>Continue pipeline on step failure</span>
            </label>
          </div>
        </div>

        ${v}

        <div class="form-group" style="margin: 0;">
          <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 4px;">
            <div style="display: flex; align-items: center; gap: 8px;">
              <label class="form-label" style="margin: 0; font-size: 11px;">Step Arguments JSON</label>
              ${s?`
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
          >${o(a.argsJson)}</textarea>
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
            ${n.length} sequential execution steps configured
          </div>
          <div style="display: flex; gap: 10px;">
            <button class="btn btn-ghost" onclick="window.app.closeBatchModal()">Cancel</button>
            <button class="btn btn-primary" onclick="window.app.executeBatchPipeline()">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polygon points="5 3 19 12 5 21 5 3"></polygon></svg>
              Run Batch Pipeline (${n.length} Steps)
            </button>
          </div>
        </div>
      </div>
    </div>
  `}function ee(e){let t=e.tasks||[],n=e.taskFilterStatus||"all",i=t.filter((c)=>c.status==="input_required"),a=t.filter((c)=>c.status==="working"),r=t.filter((c)=>c.status==="completed"),s=t.filter((c)=>c.status==="cancelled"),l=t.filter((c)=>c.status==="failed"),d=n==="all"?t:t.filter((c)=>c.status===n),g=e.config.policy?.require_approval||e.config.policy?.requireApproval||[],u=i.length===0?`
    <div style="padding: 36px 24px; text-align: center; background: var(--surface-card); border-radius: var(--radius-md); border: 1px dashed var(--border);">
      <div style="width: 44px; height: 44px; border-radius: 50%; background: rgba(52, 211, 153, 0.12); border: 1px solid rgba(52, 211, 153, 0.3); display: flex; align-items: center; justify-content: center; margin: 0 auto 12px; color: var(--green-400); font-size: 18px; font-weight: 700;">
        ✓
      </div>
      <div style="font-size: 14.5px; font-weight: 600; color: var(--text-main); margin-bottom: 5px;">No Tasks Awaiting Input or Approval</div>
      <div style="font-size: 11.5px; color: var(--text-dim); max-width: 520px; margin: 0 auto; line-height: 1.6;">
        Tool calls requiring Human-in-the-Loop approval or returning asynchronous <code style="color: var(--amber-300); font-family: var(--ff-mono);">input_required</code> tasks will suspend here for operator inspection, parameter editing, and response submission.
      </div>
    </div>
  `:i.map((c)=>{let f=c.inputRequests||{},h=Object.keys(f),y=h.length>0,x=Math.floor(Date.now()/1000),T=c.expiresAtEpochSecs?Math.max(0,c.expiresAtEpochSecs-x):c.ttlMs?Math.max(0,Math.floor(c.ttlMs/1000)):c.ttlSeconds||300,I=c.createdAtEpochSecs?new Date(c.createdAtEpochSecs*1000).toLocaleTimeString():c.createdAt?new Date(c.createdAt).toLocaleTimeString():"—";return`
      <div class="bento-card" style="border: 1px solid rgba(245, 158, 11, 0.35); background: var(--surface-card); margin-bottom: 14px; padding: 18px;">
        <div style="display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 12px;">
          <div>
            <div style="display: flex; align-items: center; gap: 8px;">
              <span class="brand-badge" style="background: rgba(245, 158, 11, 0.2); color: var(--amber-300); border-color: rgba(245, 158, 11, 0.5);">
                INPUT REQUIRED
              </span>
              <span style="font-family: var(--ff-mono); font-size: 12px; color: var(--text-muted);">${o(c.taskId)}</span>
            </div>
            <div style="margin-top: 6px; display: flex; align-items: center; gap: 8px;">
              <span style="font-size: 14.5px; font-weight: 700; color: var(--text-main); font-family: var(--ff-mono);">
                ${o(c.capabilityId||"Tool Execution")}
              </span>
              ${c.serverId?`<span style="font-size: 11px; color: var(--text-dim);">via <span style="color: var(--cyan-400); font-family: var(--ff-mono);">${o(c.serverId)}</span></span>`:""}
            </div>
          </div>

          <div style="text-align: right; font-family: var(--ff-mono); font-size: 11px; color: var(--text-dim);">
            <div>Created: <span style="color: var(--text-muted);">${I}</span></div>
            <div style="color: var(--amber-400); margin-top: 2px;">TTL Remaining: ${T}s</div>
          </div>
        </div>

        <!-- Caller Context -->
        ${c.context?`
          <div style="background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 8px 12px; font-family: var(--ff-mono); font-size: 11px; display: flex; flex-wrap: wrap; gap: 16px; margin-bottom: 12px; color: var(--text-muted);">
            ${c.context.actor_id?`<div><span style="color: var(--text-dim);">Actor:</span> <span style="color: var(--cyan-400);">${o(c.context.actor_id)}</span></div>`:""}
            ${c.context.operation_id?`<div><span style="color: var(--text-dim);">Operation:</span> <span style="color: var(--text-main);">${o(c.context.operation_id)}</span></div>`:""}
            ${c.context.grant_id?`<div><span style="color: var(--text-dim);">Grant:</span> <span style="color: var(--text-main);">${o(c.context.grant_id)}</span></div>`:""}
          </div>
        `:""}

        <!-- Dynamic Input Requests Form -->
        <div style="margin-bottom: 14px;">
          <div style="font-size: 11px; font-weight: 600; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.5px; margin-bottom: 8px;">
            ${y?"Required Input Responses (MRTR / HITL)":"Input Responses Payload (JSON)"}
          </div>

          ${y?`
            <div style="display: flex; flex-direction: column; gap: 10px;">
              ${h.map((E)=>{let w=f[E]||{},k=typeof w==="string"?w:w.prompt||w.description||w.title||E,C=w.type||"text",_=w.default!==void 0?JSON.stringify(w.default):w.value!==void 0?JSON.stringify(w.value):w.sanitized_args?JSON.stringify(w.sanitized_args,null,2):"";if(C==="approval_review")return`
                    <div style="background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 10px 12px;">
                      <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 4px;">
                        <label style="font-size: 11.5px; font-weight: 600; color: var(--amber-300); font-family: var(--ff-mono);">${o(E)}</label>
                        <span class="brand-badge" style="font-size: 9.5px; padding: 1px 5px;">APPROVAL GATED</span>
                      </div>
                      <div style="font-size: 11px; color: var(--text-dim); margin-bottom: 6px;">${o(k)}</div>
                      <div style="margin-bottom: 8px;">
                        <label style="font-size: 10.5px; color: var(--text-dim); display: block; margin-bottom: 2px;">Decision:</label>
                        <select id="task-input-${o(c.taskId)}-${o(E)}-decision" class="form-input" style="font-size: 11.5px; font-family: var(--ff-mono); padding: 4px 8px;">
                          <option value="true" selected>Approve &amp; Execute</option>
                          <option value="false">Reject Execution</option>
                        </select>
                      </div>
                      <div>
                        <label style="font-size: 10.5px; color: var(--text-dim); display: block; margin-bottom: 2px;">Parameters (Editable):</label>
                        <textarea id="task-input-${o(c.taskId)}-${o(E)}" class="form-textarea" rows="3" style="color: var(--green-400); font-family: var(--ff-mono); font-size: 11.5px;">${o(_)}</textarea>
                      </div>
                    </div>
                  `;return`
                  <div style="background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 10px 12px;">
                    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 4px;">
                      <label style="font-size: 11.5px; font-weight: 600; color: var(--amber-300); font-family: var(--ff-mono);">${o(E)}</label>
                      <span class="brand-badge" style="font-size: 9.5px; padding: 1px 5px;">${o(C)}</span>
                    </div>
                    <div style="font-size: 11px; color: var(--text-dim); margin-bottom: 6px;">${o(k)}</div>
                    ${C==="confirmation"||C==="boolean"?`
                      <select id="task-input-${o(c.taskId)}-${o(E)}" class="form-input" style="font-size: 11.5px; font-family: var(--ff-mono); padding: 4px 8px;">
                        <option value="true" selected>true (Approve / Confirm)</option>
                        <option value="false">false (Reject / Deny)</option>
                      </select>
                    `:w.sanitized_args||C==="object"||C==="json"?`
                      <textarea id="task-input-${o(c.taskId)}-${o(E)}" class="form-textarea" rows="3" style="color: var(--green-400); font-family: var(--ff-mono); font-size: 11.5px;">${o(_)}</textarea>
                    `:`
                      <input id="task-input-${o(c.taskId)}-${o(E)}" type="text" class="form-input" value="${o(_)}" placeholder="Enter ${o(E)} response..." style="font-size: 11.5px; font-family: var(--ff-mono);">
                    `}
                  </div>
                `}).join("")}
            </div>
          `:`
            <textarea id="task-raw-input-${o(c.taskId)}" class="form-textarea" rows="3" style="color: var(--green-400); font-family: var(--ff-mono); font-size: 11.5px;">{}</textarea>
          `}
        </div>

        <!-- Action Footer -->
        <div style="display: flex; justify-content: space-between; align-items: center; border-top: 1px solid var(--border); padding-top: 12px;">
          <div style="display: flex; align-items: center; gap: 8px;">
            <input id="task-operator-${o(c.taskId)}" type="text" class="form-input" placeholder="Operator ID" value="security-operator" style="width: 180px; padding: 5px 10px; font-size: 11px;">
          </div>

          <div style="display: flex; align-items: center; gap: 8px;">
            <button class="btn btn-danger" onclick="window.app.promptCancelTask('${o(c.taskId)}')">
              ✕ Cancel Task
            </button>
            <button class="btn btn-primary" onclick="window.app.submitTaskInputResponses('${o(c.taskId)}')">
              ✓ Submit &amp; Resume
            </button>
          </div>
        </div>
      </div>
    `}).join(""),m=g.length===0?`
    <div style="color: var(--text-dim); font-size: 11.5px; line-height: 1.5; padding: 8px 0;">
      No explicit <code style="color: var(--amber-400);">require_approval</code> rules active. Gated execution rules convert matching tool calls into tasks in real-time.
    </div>
  `:g.map((c)=>`
    <div style="display: flex; justify-content: space-between; align-items: center; background: var(--surface); padding: 8px 10px; border-radius: var(--radius-sm); border: 1px solid var(--border); margin-bottom: 6px;">
      <span style="font-family: var(--ff-mono); font-size: 11.5px; color: var(--amber-300); font-weight: 500;">\uD83D\uDEE1️ ${o(c)}</span>
      <span class="brand-badge" style="font-size: 9.5px; padding: 1px 5px;">GATED</span>
    </div>
  `).join(""),v=d.length===0?`
    <tr>
      <td colspan="6" style="padding: 24px; text-align: center; color: var(--text-dim); font-size: 12px;">
        No tasks found matching filter "${o(n)}".
      </td>
    </tr>
  `:d.map((c)=>{let f=c.status==="completed"?"background: rgba(52, 211, 153, 0.12); color: var(--green-400); border-color: rgba(52, 211, 153, 0.3);":c.status==="working"?"background: rgba(56, 189, 248, 0.15); color: var(--cyan-400); border-color: rgba(56, 189, 248, 0.4);":c.status==="input_required"?"background: rgba(245, 158, 11, 0.2); color: var(--amber-300); border-color: rgba(245, 158, 11, 0.5);":c.status==="cancelled"?"background: rgba(148, 163, 184, 0.15); color: var(--text-muted); border-color: rgba(148, 163, 184, 0.3);":"background: rgba(248, 113, 113, 0.12); color: var(--red-400); border-color: rgba(248, 113, 113, 0.3);",h=c.progress!==void 0?Math.round(c.progress*100):c.status==="completed"?100:c.status==="working"?50:0,y=c.createdAtEpochSecs?new Date(c.createdAtEpochSecs*1000).toLocaleTimeString():c.createdAt?new Date(c.createdAt).toLocaleTimeString():"—";return`
      <tr style="border-bottom: 1px solid rgba(255,255,255,0.03); transition: background 0.15s;" onmouseover="this.style.background='var(--surface-hover)'" onmouseout="this.style.background='transparent'">
        <td style="padding: 10px 14px;">
          <span class="brand-badge" style="${f}">
            ${c.status.toUpperCase()}
          </span>
        </td>
        <td style="padding: 10px 14px; font-family: var(--ff-mono); font-weight: 600; color: var(--text-main); font-size: 11.5px;">
          ${o(c.capabilityId||"Tool Execution")}
        </td>
        <td style="padding: 10px 14px; font-family: var(--ff-mono); color: var(--text-dim); font-size: 11px;">
          ${o(c.taskId)}
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
          ${y}
        </td>
        <td style="padding: 10px 14px; text-align: right;">
          <div style="display: inline-flex; gap: 6px; align-items: center;">
            <button class="btn btn-ghost" style="padding: 2px 8px; font-size: 10.5px;" onclick="window.app.openTaskInspectorModal('${o(c.taskId)}')">\uD83D\uDD0D Inspect</button>
            ${c.status==="input_required"||c.status==="working"?`
              <button class="btn btn-danger" style="padding: 2px 8px; font-size: 10.5px;" onclick="window.app.promptCancelTask('${o(c.taskId)}')">Cancel</button>
            `:""}
          </div>
        </td>
      </tr>
    `}).join("");return`
    <!-- Sub-header & Actions -->
    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px;">
      <div style="display: flex; align-items: center; gap: 10px;">
        <span class="brand-badge" style="font-size: 11px; padding: 3px 10px; color: ${i.length>0?"var(--amber-300)":"var(--green-400)"}; border-color: ${i.length>0?"rgba(245, 158, 11, 0.4)":"rgba(52, 211, 153, 0.4)"}; background: ${i.length>0?"rgba(245, 158, 11, 0.1)":"rgba(52, 211, 153, 0.1)"};">
          ${i.length} ACTION REQUIRED
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
        <div class="stat-value" style="color: ${i.length>0?"var(--amber-400)":"var(--text-main)"};">${i.length}</div>
        <div class="stat-sub">Awaiting operator decision or response</div>
      </div>
      <div class="bento-card col-3">
        <div class="stat-label">Working / In-Flight</div>
        <div class="stat-value" style="color: var(--cyan-400);">${a.length}</div>
        <div class="stat-sub">Asynchronous active executions</div>
      </div>
      <div class="bento-card col-3">
        <div class="stat-label">Completed Tasks</div>
        <div class="stat-value" style="color: var(--green-400);">${r.length}</div>
        <div class="stat-sub">Finished successfully</div>
      </div>
      <div class="bento-card col-3">
        <div class="stat-label">Cancelled / Failed</div>
        <div class="stat-value" style="color: ${l.length>0?"var(--red-400)":"var(--text-muted)"};">${s.length+l.length}</div>
        <div class="stat-sub">Terminated or errored</div>
      </div>
    </div>

    <!-- Main Content Bento Split (8 cols queue / 4 cols rules) -->
    <div class="bento-grid">
      <!-- Left Column: Input Required Action Queue -->
      <div class="col-8">
        <div style="font-size: 11px; font-weight: 700; color: var(--amber-400); text-transform: uppercase; letter-spacing: 0.5px; margin-bottom: 12px; display: flex; align-items: center; gap: 6px;">
          <span>⚡ Awaiting Operator Action (${i.length})</span>
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
          ${m}
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
          \uD83D\uDCDC Task Registry (${d.length})
        </div>
        <div style="display: flex; gap: 6px; align-items: center;">
          <span style="font-size: 11px; color: var(--text-dim);">Filter Status:</span>
          <select class="form-input" style="padding: 3px 8px; font-size: 11px; width: 140px;" onchange="window.app.filterTasksByStatus(this.value)">
            <option value="all" ${n==="all"?"selected":""}>All Statuses</option>
            <option value="input_required" ${n==="input_required"?"selected":""}>input_required</option>
            <option value="working" ${n==="working"?"selected":""}>working</option>
            <option value="completed" ${n==="completed"?"selected":""}>completed</option>
            <option value="cancelled" ${n==="cancelled"?"selected":""}>cancelled</option>
            <option value="failed" ${n==="failed"?"selected":""}>failed</option>
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
            ${v}
          </tbody>
        </table>
      </div>
    </div>
  `}function te(e){let t=[...e.undoEntries||[]].sort((a,r)=>r.created_at_ms-a.created_at_ms);if(e.undoError)return`<section class="bento-card"><h2>Action feed unavailable</h2><p>${o(e.undoError)}</p><button class="btn btn-ghost" data-undo-action="refresh">Retry</button></section>`;let n=(e.tasks||[]).filter((a)=>a.phase==="pending_emission");if(t.length===0&&n.length===0)return'<section class="bento-card"><h2>Action feed</h2><p>No reversible actions or held sends are currently recorded.</p><p class="text-dim">New armed inverses appear here with actor, profile, capability, and expiry. Snapshot contents and action arguments are never shown.</p><button class="btn btn-ghost" data-undo-action="refresh">Refresh</button></section>';let i=new Map;for(let a of t){let r=a.work_item_id||"";i.set(r,[...i.get(r)||[],a])}return`<section class="bento-card"><header style="display:flex;justify-content:space-between;align-items:center"><div><h2>Action feed</h2><p class="text-dim">Value-free action records. Upstream state is checked by configured contracts before an inverse runs.</p></div><button class="btn btn-ghost" data-undo-action="refresh">Refresh</button></header>
    ${n.map((a)=>{let r=Math.max(0,Math.ceil(((a.releaseAtMs||0)-Date.now())/1000)),s=Math.max(1,Math.ceil(((a.releaseAtMs||0)-(a.createdAtEpochSecs||0)*1000)/1000));return`<article style="margin-top:12px;padding:14px;border:1px solid var(--border);border-radius:var(--radius-md);background:var(--surface)"><div style="display:flex;justify-content:space-between;gap:12px"><div><span class="brand-badge">HELD SEND</span><strong style="margin-left:8px">${o(a.capabilityId||"Delayed action")}</strong><div class="text-dim" style="margin-top:6px">Actor: ${o(a.context?.actor_id||"Unknown")} · Task: ${o(a.taskId)}</div></div><div class="text-dim">${r}s until release</div></div><progress aria-label="Time until send release" value="${Math.min(r,s)}" max="${s}" style="width:100%;margin-top:10px"></progress>${a.cancellable&&r>0?`<button class="btn btn-danger" style="margin-top:12px" data-undo-action="cancel-task" data-undo-target="${o(a.taskId)}">Cancel now</button>`:""}</article>`}).join("")}
    ${Array.from(i.entries()).map(([a,r])=>`<section style="margin-top:18px"><header style="display:flex;justify-content:space-between;align-items:center"><strong>${a?`Work item ${o(a)}`:"Unassigned work item"}</strong>${a&&r.some((s)=>s.status==="armed"&&s.expires_at_ms>Date.now())?`<button class="btn btn-danger" data-undo-action="rollback" data-undo-target="${o(a)}">Rollback turn</button>`:""}</header>
      ${r.map((s)=>{let l=Math.max(0,Math.ceil((s.expires_at_ms-Date.now())/1000)),d=Math.max(1,Math.ceil((s.expires_at_ms-s.created_at_ms)/1000)),g=s.status==="armed"&&l>0;return`<article style="margin-top:10px;padding:14px;border:1px solid var(--border);border-radius:var(--radius-md);background:var(--surface)">
          <div style="display:flex;justify-content:space-between;gap:12px"><div><strong>${o(s.forward_capability)}</strong><div class="text-dim" style="margin-top:6px">Actor: ${o(s.actor_id||"Unknown")} · Profile: ${o(s.profile_id||"Default")}</div><div class="text-dim">Inverse: ${o(s.inverse_capability)}</div></div><div style="text-align:right"><span class="brand-badge">${o(s.status)}</span><div class="text-dim" style="margin-top:6px">${l>0?`${l}s remaining`:"Window expired"}</div></div></div>
          <div class="text-dim" style="margin-top:10px;font-size:11px">Undo ID: ${o(s.undo_id)} · Registered ${o(new Date(s.created_at_ms).toLocaleString())}</div>
          <progress aria-label="Undo window remaining" value="${Math.min(l,d)}" max="${d}" style="width:100%;margin-top:8px"></progress>
          <div class="text-dim" style="margin-top:6px;font-size:11px">Configured inverse fields: ${o((s.change_preview||[]).join(", ")||"withheld")}</div>
          ${g?`<button class="btn btn-primary" style="margin-top:12px" data-undo-action="execute" data-undo-target="${o(s.undo_id)}">Undo action</button>`:""}
        </article>`}).join("")}</section>`).join("")}
  </section>`}function re(){let e=p.getState(),t=e.auditEvents||[],n=e.auditStats||{total_events:0,by_status:{success:0,failed:0,denied:0,intercepted:0}},{auditVerification:i,auditFilters:a}=e,r=e.auditTotal??t.length,s=e.auditSelectedEvent,l=Object.keys(e.config?.mcpServers||{}),d=a.limit||25,g=a.offset||0,u=Math.floor(g/d)+1,m=Math.max(1,Math.ceil(r/d)),v=r===0?0:g+1,c=Math.min(g+d,r),f=b.getAuditExportUrl({actor_id:a.search?void 0:void 0,server_id:a.serverId!=="all"?a.serverId:void 0,event_type:a.eventType!=="all"?a.eventType:void 0,status:a.status!=="all"?a.status:void 0,search:a.search.trim()?a.search.trim():void 0},"csv"),h=b.getAuditExportUrl({server_id:a.serverId!=="all"?a.serverId:void 0,event_type:a.eventType!=="all"?a.eventType:void 0,status:a.status!=="all"?a.status:void 0,search:a.search.trim()?a.search.trim():void 0},"jsonl"),y=i?i.is_valid?`
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
  `,x=l.map((k)=>`<option value="${o(k)}" ${a.serverId===k?"selected":""}>${o(k)}</option>`).join(""),T=`
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
            value="${o(a.search)}"
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
  `,I=`
    <div style="display: flex; justify-content: space-between; align-items: center; padding: 12px 16px; background: rgba(18, 24, 38, 0.5); border-radius: var(--radius-md); border: 1px solid var(--border); margin-top: 16px;">
      <div style="font-size: 12px; color: var(--text-dim); display: flex; align-items: center; gap: 8px;">
        <span>Showing <strong style="color: var(--text-main);">${v}–${c}</strong> of <strong style="color: var(--text-main);">${r}</strong> events</span>
        <span style="color: var(--border);">|</span>
        <span>Page Size:</span>
        <select 
          class="form-input" 
          style="font-size: 11.5px; padding: 2px 24px 2px 8px; height: 28px; width: auto;"
          onchange="window.app.handleAuditPageSize(this.value)"
        >
          <option value="10" ${d===10?"selected":""}>10 / page</option>
          <option value="25" ${d===25?"selected":""}>25 / page</option>
          <option value="50" ${d===50?"selected":""}>50 / page</option>
          <option value="100" ${d===100?"selected":""}>100 / page</option>
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
          Page ${u} of ${m}
        </span>
        <button 
          class="btn btn-ghost" 
          style="padding: 4px 8px; font-size: 11px; height: 28px;"
          ${u>=m?'disabled style="opacity: 0.4; cursor: not-allowed;"':""}
          onclick="window.app.auditNextPage()"
        >
          Next ▶
        </button>
        <button 
          class="btn btn-ghost" 
          style="padding: 4px 8px; font-size: 11px; height: 28px;"
          ${u>=m?'disabled style="opacity: 0.4; cursor: not-allowed;"':""}
          onclick="window.app.auditGoToPage(${m})"
          title="Last Page"
        >
          Last ⏭
        </button>
      </div>
    </div>
  `,E="";if(t.length===0)E=`
      <div style="padding: 48px 24px; text-align: center; color: var(--text-dim); background: var(--surface-card); border-radius: var(--radius-md); border: 1px dashed var(--border);">
        <div style="font-size: 28px; margin-bottom: 8px;">\uD83D\uDD0D</div>
        <div style="font-size: 14px; font-weight: 600; color: var(--text-main); margin-bottom: 4px;">No Matching Audit Events</div>
        <div style="font-size: 12px; max-width: 420px; margin: 0 auto;">No audit records match your currently selected filters. Try broadening your search or resetting filters.</div>
        <button class="btn btn-ghost" style="margin-top: 14px; font-size: 11.5px;" onclick="window.app.clearAuditFilters()">Reset Filters</button>
      </div>
    `;else E=t.map((k)=>{let C=new Date(Math.floor(k.timestamp_ns/1e6)).toLocaleString(),_='<span class="badge" style="background: rgba(34, 197, 94, 0.15); color: var(--green-400); font-weight: 600;">SUCCESS</span>';if(k.status==="denied")_='<span class="badge" style="background: rgba(239, 68, 68, 0.15); color: var(--red-400); font-weight: 600;">DENIED</span>';else if(k.status==="intercepted")_='<span class="badge" style="background: rgba(234, 179, 8, 0.15); color: var(--amber-300); font-weight: 600;">HITL INTERCEPT</span>';else if(k.status==="failed")_='<span class="badge" style="background: rgba(239, 68, 68, 0.15); color: var(--red-400); font-weight: 600;">FAILED</span>';else if(k.status==="cancelled")_='<span class="badge" style="background: rgba(148, 163, 184, 0.15); color: var(--text-muted); font-weight: 600;">CANCELLED</span>';let R=k.sanitized_args?JSON.stringify(k.sanitized_args):"-",M=k.actor_id||k.operator_id||"anonymous",z=k.server_id||"system",j=k.capability_id||k.event_type,O=k.execution_latency_us?`${(k.execution_latency_us/1000).toFixed(1)}ms`:"-";return`
        <div class="bento-card" style="margin-bottom: 12px; padding: 16px; border: 1px solid var(--border); transition: border-color 0.15s ease;">
          <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
            <div style="display: flex; align-items: center; gap: 8px;">
              <span style="font-family: var(--ff-mono); font-size: 11px; font-weight: 700; color: var(--text-dim);">${o(k.id)}</span>
              ${_}
              <span style="font-size: 12px; font-weight: 600; color: var(--text-main);">${o(j)}</span>
            </div>
            <div style="display: flex; align-items: center; gap: 10px;">
              <div style="font-family: var(--ff-mono); font-size: 11px; color: var(--text-muted);">${o(C)}</div>
              <button 
                class="btn btn-ghost" 
                style="padding: 2px 8px; font-size: 11px; height: 24px;" 
                onclick="window.app.selectAuditEvent('${o(k.id)}')"
                title="Inspect event details & cryptographic payload"
              >
                Inspect \uD83D\uDD0D
              </button>
            </div>
          </div>
          
          <div style="display: grid; grid-template-columns: repeat(4, 1fr); gap: 8px; font-size: 11.5px; background: rgba(0,0,0,0.2); padding: 8px 12px; border-radius: var(--radius-sm); margin-bottom: 8px;">
            <div><span style="color: var(--text-muted);">Actor:</span> <strong style="color: var(--text-main);">${o(M)}</strong></div>
            <div><span style="color: var(--text-muted);">Server:</span> <strong style="color: var(--cyan-400);">${o(z)}</strong></div>
            <div><span style="color: var(--text-muted);">Trace:</span> <code style="color: var(--cyan-400); font-size: 10.5px;">${o(k.trace_id)}</code></div>
            <div><span style="color: var(--text-muted);">Latency:</span> <span style="color: var(--amber-300);">${O}</span></div>
          </div>

          <div style="font-family: var(--ff-mono); font-size: 11px; color: var(--text-dim); margin-bottom: 8px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">
            <span style="color: var(--text-muted);">Args:</span> ${o(R)}
          </div>

          <div style="display: flex; justify-content: space-between; align-items: center; border-top: 1px solid var(--border); padding-top: 6px; font-size: 10.5px; font-family: var(--ff-mono); color: var(--text-muted);">
            <div><span style="color: var(--text-dim);">prev_hash:</span> ${o(k.prev_hash.slice(0,16))}...</div>
            <div><span style="color: var(--text-dim);">hash:</span> <span style="color: var(--green-400);">${o(k.hash.slice(0,16))}...</span></div>
          </div>
        </div>
      `}).join("");let w="";if(s){let k=new Date(Math.floor(s.timestamp_ns/1e6)).toISOString();w=`
      <div style="position: fixed; inset: 0; background: rgba(0,0,0,0.7); backdrop-filter: blur(4px); z-index: 9999; display: flex; align-items: center; justify-content: center; padding: 24px;" onclick="if (event.target === this) window.app.selectAuditEvent(null)">
        <div class="bento-card" style="width: 100%; max-width: 720px; max-height: 85vh; display: flex; flex-direction: column; overflow: hidden; background: #0f172a; border: 1px solid var(--border); box-shadow: 0 20px 40px rgba(0,0,0,0.5);">
          <!-- Modal Header -->
          <div style="display: flex; justify-content: space-between; align-items: center; padding: 16px 20px; border-bottom: 1px solid var(--border);">
            <div style="display: flex; align-items: center; gap: 8px;">
              <span style="font-size: 16px;">\uD83D\uDD12</span>
              <h2 style="font-size: 15px; font-weight: 700; color: var(--text-main); margin: 0;">Audit Event Details (${o(s.id)})</h2>
            </div>
            <button class="btn btn-ghost" style="padding: 4px 8px; font-size: 14px;" onclick="window.app.selectAuditEvent(null)">✕</button>
          </div>

          <!-- Modal Body -->
          <div style="padding: 20px; overflow-y: auto; display: flex; flex-direction: column; gap: 14px; font-size: 12px;">
            <div style="display: grid; grid-template-columns: repeat(2, 1fr); gap: 10px; background: rgba(0,0,0,0.25); padding: 12px; border-radius: var(--radius-sm);">
              <div><span style="color: var(--text-muted);">Timestamp:</span> <strong style="color: var(--text-main); font-family: var(--ff-mono); font-size: 11px;">${o(k)}</strong></div>
              <div><span style="color: var(--text-muted);">Status:</span> <strong style="color: var(--text-main);">${o(s.status.toUpperCase())}</strong></div>
              <div><span style="color: var(--text-muted);">Event Type:</span> <strong style="color: var(--text-main);">${o(s.event_type)}</strong></div>
              <div><span style="color: var(--text-muted);">Server:</span> <strong style="color: var(--cyan-400);">${o(s.server_id||"system")}</strong></div>
              <div><span style="color: var(--text-muted);">Capability:</span> <strong style="color: var(--text-main);">${o(s.capability_id||"-")}</strong></div>
              <div><span style="color: var(--text-muted);">Actor / Operator:</span> <strong style="color: var(--text-main);">${o(s.actor_id||s.operator_id||"anonymous")}</strong></div>
              <div><span style="color: var(--text-muted);">Trace ID:</span> <code style="color: var(--cyan-400);">${o(s.trace_id)}</code></div>
              <div><span style="color: var(--text-muted);">Request ID:</span> <code style="color: var(--cyan-400);">${o(s.request_id||"-")}</code></div>
              <div><span style="color: var(--text-muted);">Client IP:</span> <span style="color: var(--text-main);">${o(s.client_ip||"-")}</span></div>
              <div><span style="color: var(--text-muted);">Latency:</span> <span style="color: var(--amber-300);">${s.execution_latency_us?`${(s.execution_latency_us/1000).toFixed(2)} ms`:"-"}</span></div>
            </div>

            ${s.error_message?`
              <div style="background: rgba(239, 68, 68, 0.1); border: 1px solid rgba(239, 68, 68, 0.3); border-radius: var(--radius-sm); padding: 10px 12px; color: var(--red-400);">
                <div style="font-weight: 700; margin-bottom: 2px;">Error (${o(s.error_code||"ERROR")}):</div>
                <div style="font-family: var(--ff-mono); font-size: 11px;">${o(s.error_message)}</div>
              </div>
            `:""}

            <!-- Sanitized Arguments -->
            <div>
              <div style="font-weight: 600; color: var(--text-main); margin-bottom: 4px;">Sanitized Arguments</div>
              <pre style="background: rgba(0,0,0,0.4); padding: 10px; border-radius: var(--radius-sm); border: 1px solid var(--border); font-family: var(--ff-mono); font-size: 11px; max-height: 140px; overflow: auto; margin: 0; color: #cbd5e1;">${o(JSON.stringify(s.sanitized_args||{},null,2))}</pre>
            </div>

            <!-- Sanitized Response -->
            ${s.sanitized_response?`
              <div>
                <div style="font-weight: 600; color: var(--text-main); margin-bottom: 4px;">Sanitized Response</div>
                <pre style="background: rgba(0,0,0,0.4); padding: 10px; border-radius: var(--radius-sm); border: 1px solid var(--border); font-family: var(--ff-mono); font-size: 11px; max-height: 140px; overflow: auto; margin: 0; color: #cbd5e1;">${o(JSON.stringify(s.sanitized_response,null,2))}</pre>
              </div>
            `:""}

            <!-- Cryptographic Hashes -->
            <div style="background: rgba(0,0,0,0.3); padding: 12px; border-radius: var(--radius-sm); border: 1px solid var(--border);">
              <div style="font-weight: 600; color: var(--text-main); margin-bottom: 6px;">Tamper-Evidence Cryptographic Hashes</div>
              <div style="margin-bottom: 6px;">
                <span style="color: var(--text-muted); font-size: 10.5px;">Previous Chain Hash (prev_hash):</span>
                <div style="font-family: var(--ff-mono); font-size: 10.5px; color: var(--text-dim); word-break: break-all;">${o(s.prev_hash)}</div>
              </div>
              <div>
                <span style="color: var(--text-muted); font-size: 10.5px;">Record Hash Signature (hash):</span>
                <div style="font-family: var(--ff-mono); font-size: 10.5px; color: var(--green-400); word-break: break-all;">${o(s.hash)}</div>
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
        ${y}
        <a href="${f}" download class="btn btn-ghost" style="font-size: 11.5px; text-decoration: none;" title="Export current filtered view as CSV">\uD83D\uDCE5 Export CSV</a>
        <a href="${h}" download class="btn btn-ghost" style="font-size: 11.5px; text-decoration: none;" title="Export current filtered view as JSONL">\uD83D\uDCE5 Export JSONL</a>
        <button class="btn btn-primary" style="font-size: 11.5px;" onclick="window.app.refreshAuditEvents()">\uD83D\uDD04 Refresh</button>
      </div>
    </div>

    <!-- Stats summary cards -->
    <div style="display: grid; grid-template-columns: repeat(4, 1fr); gap: 12px; margin-bottom: 20px;">
      <div class="bento-card" style="padding: 14px; text-align: center;">
        <div style="font-size: 11px; color: var(--text-muted); text-transform: uppercase; font-weight: 600;">Total Events</div>
        <div style="font-size: 22px; font-weight: 800; color: var(--text-main); margin-top: 4px;">${n.total_events}</div>
      </div>
      <div class="bento-card" style="padding: 14px; text-align: center;">
        <div style="font-size: 11px; color: var(--text-muted); text-transform: uppercase; font-weight: 600;">Successful Calls</div>
        <div style="font-size: 22px; font-weight: 800; color: var(--green-400); margin-top: 4px;">${n.by_status.success}</div>
      </div>
      <div class="bento-card" style="padding: 14px; text-align: center;">
        <div style="font-size: 11px; color: var(--text-muted); text-transform: uppercase; font-weight: 600;">HITL Intercepts</div>
        <div style="font-size: 22px; font-weight: 800; color: var(--amber-300); margin-top: 4px;">${n.by_status.intercepted}</div>
      </div>
      <div class="bento-card" style="padding: 14px; text-align: center;">
        <div style="font-size: 11px; color: var(--text-muted); text-transform: uppercase; font-weight: 600;">Policy Denials</div>
        <div style="font-size: 22px; font-weight: 800; color: var(--red-400); margin-top: 4px;">${n.by_status.denied}</div>
      </div>
    </div>

    <!-- Search & Filter Toolbar -->
    ${T}

    <!-- Event Timeline List Header -->
    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px;">
      <h2 style="font-size: 14px; font-weight: 600; color: var(--text-main);">Sequential Audit Ledger (SHA-256 Hash Chained)</h2>
      <span style="font-size: 11.5px; color: var(--text-dim);">${t.length} events loaded on this page</span>
    </div>

    <!-- Event Rows -->
    <div>
      ${E}
    </div>

    <!-- Pagination Footer -->
    ${r>0?I:""}

    <!-- Modal Popup for Event Inspection -->
    ${w}
  `}function se(){let e=p.getState(),t=e.activeProfile,n=t?e.config.profiles?.[t]:void 0,i=!!n,a=e.config.policy||{},r=n?.policy,s=i?r||{}:a,l=s.allow||[],d=s.deny||[],g=s.redact_keys||s.redactKeys||[],u=s.require_approval||s.requireApproval||[],m=l.length===0?`
    <div style="color: var(--text-dim); font-size: 12px;">${i?"No profile allow list (inherits global rules)":"No allow list (all non-denied operations permitted)"}</div>
  `:l.map((w,k)=>`
    <div style="display: flex; justify-content: space-between; align-items: center; background: var(--surface); padding: 8px 12px; border-radius: var(--radius-sm); border: 1px solid var(--border);">
      <span style="font-family: var(--ff-mono); font-size: 12px; color: var(--green-400);">✔ ${o(w)}</span>
      <button class="btn btn-ghost" style="padding: 2px 6px; font-size: 11px; color: var(--red-400);" onclick="window.app.removePolicyRule('allow', ${k})">✕</button>
    </div>
  `).join(""),v=d.length===0?`
    <div style="color: var(--text-dim); font-size: 12px;">${i?"No profile deny rules configured":"No deny rules configured"}</div>
  `:d.map((w,k)=>`
    <div style="display: flex; justify-content: space-between; align-items: center; background: var(--surface); padding: 8px 12px; border-radius: var(--radius-sm); border: 1px solid var(--border);">
      <span style="font-family: var(--ff-mono); font-size: 12px; color: var(--red-400);">✖ ${o(w)}</span>
      <button class="btn btn-ghost" style="padding: 2px 6px; font-size: 11px; color: var(--red-400);" onclick="window.app.removePolicyRule('deny', ${k})">✕</button>
    </div>
  `).join(""),c=u.length===0?`
    <div style="color: var(--text-dim); font-size: 12px;">${i?"No profile human-in-the-loop triggers configured":"No human-in-the-loop approval rules configured"}</div>
  `:u.map((w,k)=>`
    <div style="display: flex; justify-content: space-between; align-items: center; background: var(--surface); padding: 8px 12px; border-radius: var(--radius-sm); border: 1px solid var(--border);">
      <span style="font-family: var(--ff-mono); font-size: 12px; color: var(--amber-400);">\uD83D\uDEE1️ ${o(w)}</span>
      <button class="btn btn-ghost" style="padding: 2px 6px; font-size: 11px; color: var(--red-400);" onclick="window.app.removePolicyRule('requireApproval', ${k})">✕</button>
    </div>
  `).join(""),f=g.length===0?`
    <div style="color: var(--text-dim); font-size: 12px;">${i?"No profile key redaction patterns configured":"No key redaction patterns configured"}</div>
  `:g.map((w,k)=>`
    <span class="brand-badge" style="color: var(--amber-300); padding: 5px 10px; font-size: 11px; display: inline-flex; align-items: center; gap: 6px;">
      ${o(w)}
      <span style="cursor: pointer; color: var(--red-400); font-weight: bold;" onclick="window.app.removePolicyRule('redact', ${k})">✕</span>
    </span>
  `).join(""),h=i?`
    <div class="bento-card" style="margin-bottom: 16px; background: rgba(245, 158, 11, 0.05); border: 1px solid rgba(245, 158, 11, 0.3); display: flex; justify-content: space-between; align-items: center;">
      <div style="display: flex; align-items: center; gap: 10px;">
        <span style="font-size: 18px;">\uD83D\uDEE1️</span>
        <div>
          <div style="font-size: 13px; font-weight: 700; color: var(--amber-400);">
            Viewing &amp; Editing Policy for Profile Constellation: <code style="font-size: 13px; color: var(--text-main);">${o(t)}</code>
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
  `,y=Object.keys(e.config.mcpServers||{}),x=n?.servers||[],T=i?y.filter((w)=>!x.includes(w)):[],I=i?`
    <div class="bento-card" style="margin-bottom: 16px; border: 1px solid rgba(245, 158, 11, 0.2); background: rgba(0, 0, 0, 0.2);">
      <div class="stat-header" style="display: flex; justify-content: space-between; align-items: center;">
        <span class="stat-label" style="color: var(--amber-400);">Constellation Server Boundaries (Profile: ${o(t)})</span>
        <span style="font-size: 11px; color: var(--text-dim);">${x.length} of ${y.length} servers active</span>
      </div>
      <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 14px; margin-top: 10px;">
        <div style="background: var(--surface); padding: 10px; border-radius: var(--radius-sm); border: 1px solid var(--border);">
          <div style="font-size: 11px; font-weight: 600; color: var(--green-400); text-transform: uppercase; margin-bottom: 6px;">
            ✔ Included Servers (${x.length})
          </div>
          <div style="display: flex; flex-wrap: wrap; gap: 6px;">
            ${x.length>0?x.map((w)=>`
              <span class="brand-badge" style="color: var(--cyan-400); border-color: rgba(34, 211, 238, 0.25); background: rgba(34, 211, 238, 0.05);">
                ${o(w)}
              </span>
            `).join(""):'<span style="font-size: 11px; color: var(--text-dim);">No servers included</span>'}
          </div>
        </div>

        <div style="background: var(--surface); padding: 10px; border-radius: var(--radius-sm); border: 1px solid var(--border);">
          <div style="font-size: 11px; font-weight: 600; color: var(--amber-400); text-transform: uppercase; margin-bottom: 6px;">
            \uD83D\uDEAB Excluded Servers (${T.length}) · Implicitly Denied
          </div>
          <div style="display: flex; flex-wrap: wrap; gap: 6px; align-items: center;">
            ${T.length>0?T.map((w)=>`
              <span class="brand-badge" style="color: var(--text-muted); border-color: rgba(245, 158, 11, 0.2); background: rgba(245, 158, 11, 0.04); display: inline-flex; align-items: center; gap: 4px;">
                ${o(w)}
                <button style="background: none; border: none; color: var(--amber-400); font-size: 10px; cursor: pointer; padding: 0 2px;" title="Include in profile" onclick="window.app.toggleServerInProfile('${o(t)}', '${o(w)}', true)">+</button>
              </span>
            `).join(""):'<span style="font-size: 11px; color: var(--text-dim);">All servers included in constellation</span>'}
          </div>
        </div>
      </div>
    </div>
  `:"",E=i&&T.length>0?`
    <div style="border-top: 1px dashed var(--border); padding-top: 8px; margin-top: 8px;">
      <div style="font-size: 10.5px; color: var(--text-dim); text-transform: uppercase; font-weight: 600; margin-bottom: 6px;">
        Implicit Boundary Denials (${T.length})
      </div>
      <div style="display: flex; flex-direction: column; gap: 4px;">
        ${T.map((w)=>`
          <div style="display: flex; justify-content: space-between; align-items: center; background: rgba(245, 158, 11, 0.03); padding: 5px 8px; border-radius: var(--radius-xs); border: 1px dashed rgba(245, 158, 11, 0.2);">
            <span style="font-family: var(--ff-mono); font-size: 11px; color: var(--text-dim);">✖ ${o(w)}.*</span>
            <span style="font-size: 9.5px; color: var(--amber-400); font-family: var(--ff-mono);">server excluded</span>
          </div>
        `).join("")}
      </div>
    </div>
  `:"";return`
    ${h}

    ${I}

    <div class="bento-grid">
      <!-- Allow Rules -->
      <div class="bento-card col-4">
        <div class="stat-header">
          <span class="stat-label" style="color: var(--green-400);">Allow List Patterns</span>
        </div>
        <div style="display: flex; flex-direction: column; gap: 8px; margin: 12px 0;">
          ${m}
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
          ${v}
          ${E}
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
          ${f}
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
            <input type="text" class="form-input" id="policy-webhook-url" placeholder="https://hooks.slack.com/services/... or Discord webhook URL" value="${o(typeof s.webhook==="object"&&s.webhook?s.webhook.url||"":"")}">
          </div>
          <div>
            <label class="form-label" style="font-size: 11px;">Payload Layout Format</label>
            <select class="form-input" id="policy-webhook-format">
              <option value="slack" ${typeof s.webhook==="object"&&s.webhook?.format==="slack"?"selected":""}>Slack Block Kit (Interactive)</option>
              <option value="discord" ${typeof s.webhook==="object"&&s.webhook?.format==="discord"?"selected":""}>Discord Embed &amp; Actions</option>
              <option value="teams" ${typeof s.webhook==="object"&&s.webhook?.format==="teams"?"selected":""}>Microsoft Teams Adaptive Cards</option>
              <option value="generic" ${typeof s.webhook==="object"&&s.webhook?.format==="generic"||!s.webhook?"selected":""}>Generic JSON (Standard)</option>
            </select>
          </div>
          <div>
            <label class="form-label" style="font-size: 11px;">HMAC Secret (or Env Var)</label>
            <input type="text" class="form-input" id="policy-webhook-secret" placeholder="e.g. WARMPLANE_WEBHOOK_SECRET" value="${o(typeof s.webhook==="object"&&s.webhook?s.webhook.secret_env||s.webhook.secretEnv||s.webhook.secret||"":"")}">
          </div>
        </div>

        <div style="display: flex; justify-content: space-between; align-items: center; margin-top: 14px; padding-top: 12px; border-top: 1px solid var(--border-subtle);">
          <div style="display: flex; gap: 8px; align-items: center;">
            <button class="btn btn-primary" onclick="window.app.saveWebhookConfig()">Save Webhook Settings</button>
            <button class="btn btn-ghost" onclick="window.app.testWebhook()">⚡ Send Test Event</button>
          </div>
          <div id="policy-webhook-status" style="font-size: 11px; font-family: var(--ff-mono); color: var(--text-dim);">
            ${typeof s.webhook==="object"&&s.webhook?.url?`Active Target: ${o(s.webhook.url)}`:"No webhook configured"}
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
  `}function ae(){let e=p.getState(),t=e.config,n=Object.entries(t.capabilityAliases||{}),i=Object.entries(t.resourceAliases||{}),a=Object.entries(t.promptAliases||{}),r="";if(n.length===0&&i.length===0&&a.length===0)r=`
      <div style="padding: 24px; text-align: center; color: var(--text-dim);">
        No facade aliases configured in ${o(e.configPath)}. Add short names or custom descriptions to prune token payload sizes.
      </div>
    `;else{for(let[s,l]of n){let d=typeof l==="string"?l:l.target,g=typeof l==="object"&&l.summary?l.summary:"",u=typeof l==="object"&&!!l.passthrough,m=u?'<span class="brand-badge" style="color: var(--amber-400); border-color: rgba(245, 158, 11, 0.4); background: rgba(245, 158, 11, 0.12); margin-left: 8px; font-size: 10px; padding: 1px 7px; flex-shrink: 0; white-space: nowrap; display: inline-flex; align-items: center; gap: 3px;">⚡ passthrough</span>':"",v=g?`<div style="font-size: 11px; color: var(--text-dim); margin-top: 2px;">\uD83D\uDCAC ${o(g)}</div>`:"";r+=`
        <div class="feed-row" style="grid-template-columns: 80px 240px 1fr 70px; cursor: pointer; transition: background 0.15s;" onclick="window.app.startEditAlias('tool', '${o(s)}', '${o(d)}', '${o(g)}', ${u})" title="Click to edit alias">
          <span style="color: var(--cyan-400);">Tool</span>
          <div style="display: flex; align-items: center; min-width: 0;">
            <span style="font-weight: 700; color: var(--text-main); font-family: var(--ff-mono); overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">${o(s)}</span>
            ${m}
          </div>
          <div>
            <span style="color: var(--text-muted); font-family: var(--ff-mono); word-break: break-all;">${o(d)}</span>
            ${v}
          </div>
          <div style="text-align: right;" onclick="event.stopPropagation()">
            <button class="btn btn-ghost" style="padding: 2px 6px; color: var(--red-400);" onclick="window.app.deleteAlias('tool', '${o(s)}')">✕</button>
          </div>
        </div>
      `}for(let[s,l]of i){let d=typeof l==="string"?l:l.target,g=typeof l==="object"&&l.summary?l.summary:"",u=g?`<div style="font-size: 11px; color: var(--text-dim); margin-top: 2px;">\uD83D\uDCAC ${o(g)}</div>`:"";r+=`
        <div class="feed-row" style="grid-template-columns: 80px 240px 1fr 70px; cursor: pointer; transition: background 0.15s;" onclick="window.app.startEditAlias('resource', '${o(s)}', '${o(d)}', '${o(g)}', false)" title="Click to edit alias">
          <span style="color: var(--green-400);">Resource</span>
          <span style="font-weight: 700; color: var(--text-main); font-family: var(--ff-mono); overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">${o(s)}</span>
          <div>
            <span style="color: var(--text-muted); font-family: var(--ff-mono); word-break: break-all;">${o(d)}</span>
            ${u}
          </div>
          <div style="text-align: right;" onclick="event.stopPropagation()">
            <button class="btn btn-ghost" style="padding: 2px 6px; color: var(--red-400);" onclick="window.app.deleteAlias('resource', '${o(s)}')">✕</button>
          </div>
        </div>
      `}for(let[s,l]of a){let d=typeof l==="string"?l:l.target,g=typeof l==="object"&&l.summary?l.summary:"",u=g?`<div style="font-size: 11px; color: var(--text-dim); margin-top: 2px;">\uD83D\uDCAC ${o(g)}</div>`:"";r+=`
        <div class="feed-row" style="grid-template-columns: 80px 240px 1fr 70px; cursor: pointer; transition: background 0.15s;" onclick="window.app.startEditAlias('prompt', '${o(s)}', '${o(d)}', '${o(g)}', false)" title="Click to edit alias">
          <span style="color: var(--amber-300);">Prompt</span>
          <span style="font-weight: 700; color: var(--text-main); font-family: var(--ff-mono); overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">${o(s)}</span>
          <div>
            <span style="color: var(--text-muted); font-family: var(--ff-mono); word-break: break-all;">${o(d)}</span>
            ${u}
          </div>
          <div style="text-align: right;" onclick="event.stopPropagation()">
            <button class="btn btn-ghost" style="padding: 2px 6px; color: var(--red-400);" onclick="window.app.deleteAlias('prompt', '${o(s)}')">✕</button>
          </div>
        </div>
      `}}return`
    <!-- Sub-header -->
    <div style="margin-bottom: 16px; font-size: 12px; color: var(--text-dim);">
      Shorten capability IDs and supply custom descriptions to prune prompt tokens. Enable <b>Native Passthrough</b> for tools to expose them directly in <code style="color: var(--amber-300);">tools/list</code> with their native schema. Click any alias row to edit.
    </div>

    <!-- Quick Add / Edit Form -->
    <div class="bento-card" style="margin-bottom: 20px; overflow: visible;">
      <div class="stat-header" style="margin-bottom: 12px; display: flex; justify-content: space-between; align-items: center;">
        <span class="stat-label" id="alias-form-title">Create New Alias</span>
        <button id="alias-cancel-btn" class="btn btn-ghost" style="display: none; padding: 2px 8px; font-size: 11px; color: var(--text-dim);" onclick="window.app.resetAliasForm()">Cancel Edit</button>
      </div>
      <div style="display: grid; grid-template-columns: 140px 1fr 1fr 100px; gap: 10px; align-items: center; position: relative; margin-bottom: 8px;">
        <select class="form-input" id="alias-kind" onchange="const cb = document.getElementById('alias-passthrough-container'); if (cb) cb.style.display = this.value === 'tool' ? 'flex' : 'none';">
          <option value="tool">Tool / Capability</option>
          <option value="resource">Resource</option>
          <option value="prompt">Prompt</option>
        </select>
        <input type="text" class="form-input" id="alias-name" placeholder="Public alias (e.g. search)" onkeydown="if(event.key==='Enter') window.app.createAlias()">
        <div style="position: relative; width: 100%;">
          <input type="text" class="form-input" id="alias-target" autocomplete="off" placeholder="Target ID (e.g. semble.search)" style="width: 100%;" oninput="window.app.handleAliasTargetInput(this.value)" onkeydown="if(event.key==='Enter') window.app.createAlias()" onfocus="window.app.handleAliasTargetInput(this.value)" onblur="setTimeout(() => window.app.hideAliasDropdown(), 200)">
          <div id="alias-suggestions-dropdown" style="display: none; position: absolute; top: calc(100% + 4px); left: 0; right: 0; max-height: 240px; overflow-y: auto; background: var(--surface-elevated); border: 1px solid var(--border); border-radius: var(--radius-sm); box-shadow: 0 8px 24px rgba(0,0,0,0.4); z-index: 1000; font-family: var(--ff-mono); font-size: 11.5px;"></div>
        </div>
        <button id="alias-save-btn" class="btn btn-primary" onclick="window.app.createAlias()">+ Save</button>
      </div>
      <div style="display: grid; grid-template-columns: 1fr auto; gap: 12px; align-items: center;">
        <input type="text" class="form-input" id="alias-summary" placeholder="Optional custom description / prompt instruction (e.g. Fast hybrid code search)" onkeydown="if(event.key==='Enter') window.app.createAlias()" style="font-size: 12px;">
        <label id="alias-passthrough-container" style="display: flex; align-items: center; gap: 6px; font-size: 12px; color: var(--text-main); cursor: pointer; user-select: none; white-space: nowrap;">
          <input type="checkbox" id="alias-passthrough" style="accent-color: var(--amber-400); cursor: pointer;">
          <span>⚡ Native Passthrough Tool</span>
        </label>
      </div>
    </div>

    <!-- Aliases Table -->
    <div style="background: var(--surface-card); border: 1px solid var(--border); border-radius: var(--radius-md); overflow: hidden; font-family: var(--ff-mono); font-size: 12px;">
      <div style="display: grid; grid-template-columns: 80px 240px 1fr 70px; padding: 10px 14px; background: var(--surface-hover); border-bottom: 1px solid var(--border); color: var(--text-muted); font-weight: 600;">
        <span>TYPE</span>
        <span>PUBLIC ALIAS</span>
        <span>CANONICAL TARGET & SUMMARY</span>
        <span style="text-align: right;">ACTION</span>
      </div>
      ${r}
    </div>
  `}function ne(){let e=p.getState(),t=e.config,n=t.profiles||{},i=Object.entries(n),a=t.mcpServers||{},r=e.activeProfile,s="";if(i.length===0)s=`
      <div style="padding: 40px; text-align: center; color: var(--text-dim); background: var(--surface-card); border-radius: var(--radius-md); border: 1px dashed var(--border);">
        <div style="font-size: 15px; color: var(--text-main); font-weight: 600; margin-bottom: 8px;">No Profiles Configured</div>
        <p style="font-size: 12px; margin-bottom: 20px; max-width: 480px; margin-left: auto; margin-right: auto;">
          Profiles allow Warmplane to serve multiple task-relevant server constellations (e.g. <code>coding</code>, <code>support</code>, <code>data</code>) from one running daemon process.
        </p>
        <button class="btn btn-primary" onclick="window.app.openAddProfileModal()">+ Create First Profile</button>
      </div>
    `;else s=i.map(([l,d])=>{let g=r===l,u=d.servers.map((x)=>`<span class="brand-badge" style="${a[x]?"color: var(--cyan-400); border-color: rgba(34, 211, 238, 0.25); background: rgba(34, 211, 238, 0.05);":"color: var(--red-400); border-color: rgba(248, 113, 113, 0.3); background: rgba(248, 113, 113, 0.05);"}">${o(x)}</span>`).join(" "),m=(e.capabilities||[]).filter((x)=>d.servers.includes(x.server)).length,v=!!d.policy,c=d.policy?.allow?.length||0,f=d.policy?.deny?.length||0,h=(d.policy?.require_approval||d.policy?.requireApproval||[]).length,y=(d.policy?.redact_keys||d.policy?.redactKeys||[]).length;return`
        <div class="bento-card" style="margin-bottom: 14px; border-left: ${g?"3px solid var(--amber-400)":"1px solid var(--border)"};">
          <div style="display: flex; justify-content: space-between; align-items: flex-start;">
            <div>
              <div style="display: flex; align-items: center; gap: 8px; margin-bottom: 6px;">
                <span style="font-size: 16px; font-weight: 700; color: var(--text-main); font-family: var(--ff-mono);">${o(l)}</span>
                ${g?'<span class="brand-badge" style="color: var(--amber-400); border-color: rgba(245, 158, 11, 0.4); background: rgba(245, 158, 11, 0.1);">ACTIVE IN UI</span>':""}
                <span class="brand-badge">${d.servers.length} server${d.servers.length===1?"":"s"}</span>
                <span class="brand-badge" style="color: var(--text-dim);">${m} capabilities</span>
                ${v?'<span class="brand-badge" style="color: var(--green-400); border-color: rgba(34, 197, 94, 0.3); background: rgba(34, 197, 94, 0.08);">CUSTOM POLICY</span>':""}
              </div>
              <div style="font-size: 12px; color: var(--text-muted); margin-bottom: 10px;">
                ${o(d.description||"No description provided")}
              </div>
              <div style="display: flex; flex-wrap: wrap; gap: 6px; align-items: center; margin-bottom: ${v?"8px":"0"};">
                <span style="font-size: 11px; color: var(--text-dim); font-weight: 600; text-transform: uppercase;">Servers:</span>
                ${u||'<span style="font-size: 11px; color: var(--text-dim);">None</span>'}
              </div>
              ${v?`
                <div style="display: flex; flex-wrap: wrap; gap: 6px; align-items: center; font-size: 11px;">
                  <span style="color: var(--text-dim); font-weight: 600; text-transform: uppercase;">Policy Overlay:</span>
                  ${c>0?`<span class="brand-badge" style="color: var(--green-400);">Allow: ${c}</span>`:""}
                  ${f>0?`<span class="brand-badge" style="color: var(--red-400);">Deny: ${f}</span>`:""}
                  ${h>0?`<span class="brand-badge" style="color: var(--amber-400);">HITL: ${h}</span>`:""}
                  ${y>0?`<span class="brand-badge" style="color: var(--text-muted);">Redact: ${y}</span>`:""}
                  ${c===0&&f===0&&h===0&&y===0?'<span style="color: var(--text-dim);">Configured</span>':""}
                </div>
              `:""}
            </div>
            
            <div style="display: flex; gap: 8px; align-items: center;">
              ${g?`
                <button class="btn btn-ghost" style="padding: 4px 10px; font-size: 11.5px;" onclick="window.app.setActiveProfile(null)">
                  Deselect
                </button>
              `:`
                <button class="btn btn-primary" style="padding: 4px 10px; font-size: 11.5px;" onclick="window.app.setActiveProfile('${o(l)}')">
                  Activate in UI
                </button>
              `}
              <button class="btn btn-ghost" style="padding: 4px 10px; font-size: 11.5px;" onclick="window.app.openEditProfileModal('${o(l)}')">
                ✏️ Edit
              </button>
              <button class="btn btn-danger" style="padding: 4px 10px; font-size: 11.5px;" onclick="window.app.deleteProfile('${o(l)}')">
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

    ${s}
  `}function oe(){let e=p.getState(),t=e.secrets||[],n=e.config.mcpServers||{},i=[];for(let[u,m]of Object.entries(n)){let v=N(u,m.command,m.args);if(v){let c=Object.keys(m.env||{});for(let f of v.envFields)if(f.required&&!c.includes(f.key))i.push({server:u,key:f.key,uri:"(Not Configured)",is_vault:!1,exists:!1,backend:"Required Variable",display:`Required by ${v.name} template (${f.label})`,is_unconfigured_requirement:!0})}}let a=[...t,...i],r=a.length,s=a.filter((u)=>u.is_vault&&u.exists!==!1).length,l=a.filter((u)=>u.exists===!1).length,d=a.filter((u)=>!u.is_vault&&u.exists!==!1).length,g=a.length===0?`
    <div style="padding: 32px; text-align: center; color: var(--text-dim);">
      No environment variables or secrets configured in active servers.
    </div>
  `:a.map((u)=>{let m='<span class="brand-badge" style="color: var(--red-400); border-color: rgba(248, 113, 113, 0.4); background: rgba(248, 113, 113, 0.1);">Plaintext (Unsecured)</span>',v=u.exists===!1;if(u.is_unconfigured_requirement)m='<span class="brand-badge" style="color: var(--red-400); border-color: rgba(248, 113, 113, 0.4); background: rgba(248, 113, 113, 0.1);">⚠️ Not Configured (Required)</span>';else if(v)m=`<span class="brand-badge" style="color: var(--amber-300); border-color: rgba(245, 158, 11, 0.4); background: rgba(245, 158, 11, 0.1);">⚠️ Missing from ${o(u.backend)}</span>`;else if(u.is_vault)m=`<span class="brand-badge" style="color: var(--green-400); border-color: rgba(52, 211, 153, 0.3); background: rgba(52, 211, 153, 0.1);">\uD83D\uDD12 ${o(u.backend)}</span>`;return`
      <div style="display: grid; grid-template-columns: 140px 180px 1fr 180px auto; padding: 10px 16px; border-bottom: 1px solid var(--border-subtle); align-items: center; font-size: 12px;">
        <span style="font-weight: 700; color: var(--text-main);">${o(u.server)}</span>
        <span style="font-family: var(--ff-mono); color: ${v?"var(--amber-300)":"var(--amber-300)"};">${o(u.key)}</span>
        <span style="font-family: var(--ff-mono); font-size: 11px; color: ${v?"var(--red-400)":"var(--text-muted)"}; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">${o(u.display)}</span>
        <div>${m}</div>
        <div style="display: flex; gap: 6px; justify-content: flex-end;">
          ${u.is_unconfigured_requirement?`
            <button class="btn btn-primary" style="padding: 2px 8px; font-size: 11px;" onclick="window.app.quickVaultEnv('${o(u.server)}', '${o(u.key)}')">➕ Configure in Keychain</button>
          `:v?`
            <button class="btn btn-primary" style="padding: 2px 8px; font-size: 11px;" onclick="window.app.quickVaultEnv('${o(u.server)}', '${o(u.key)}')">➕ Re-add Key</button>
            <button class="btn btn-ghost" style="padding: 2px 8px; font-size: 11px; color: var(--red-400);" onclick="window.app.removeSecretFromConfig('${o(u.server)}', '${o(u.key)}')">Remove from Config</button>
          `:!u.is_vault?`
            <button class="btn btn-primary" style="padding: 2px 8px; font-size: 11px;" onclick="window.app.quickVaultEnv('${o(u.server)}', '${o(u.key)}')">\uD83D\uDD12 Move to Keychain</button>
          `:`
            <button class="btn btn-ghost" style="padding: 2px 8px; font-size: 11px; color: var(--red-400);" onclick="window.app.deleteVaultSecret('${o(u.key)}', '${o(u.server)}')">Delete Key</button>
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
        <div class="stat-label">Total Required &amp; Configured</div>
        <div class="stat-value" style="color: var(--cyan-400);">${r}</div>
        <div class="stat-sub">Across all configured MCP servers</div>
      </div>
      <div class="bento-card col-4">
        <div class="stat-label">Secured via Vault / Keychain</div>
        <div class="stat-value" style="color: var(--green-400);">${s}</div>
        <div class="stat-sub">Zero-disk plaintext exposure</div>
      </div>
      <div class="bento-card col-4">
        <div class="stat-label">Missing or Unsecured</div>
        <div class="stat-value" style="color: ${l+d>0?"var(--red-400)":"var(--green-400)"};">${l+d}</div>
        <div class="stat-sub">${l>0?`${l} missing required key(s)`:d>0?"Recommend migrating to Keychain":"All credentials protected"}</div>
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
        ${g}
      </div>
    </div>
  `}class ie{activeTemplateCategory="all";activeTemplateFilter="";selectedTemplate=null;pendingClientId=null;async init(){let e=window.location.port?`:${window.location.port}`:"",t=document.getElementById("daemon-port-label");if(t)t.textContent=`Daemon ${e}`;await this.refreshData(),this.initSSE(),this.bindUndoFeedActions(),this.render(),p.subscribe(()=>{this.render()}),window.setInterval(()=>{if(p.getState().activeTab==="undo")this.render()},1000)}auditSearchTimeout=null;bindUndoFeedActions(){let e=document.getElementById("app-main");if(!e)return;e.addEventListener("click",(t)=>{let n=t.target;if(!(n instanceof Element))return;let i=n.closest("[data-undo-action], [data-playground-action]");if(!i||!e.contains(i))return;if(i.dataset.playgroundAction==="test-undo"&&i.dataset.undoAlias){this.testUndoContract(i.dataset.undoAlias);return}let a=i.dataset.undoAction,r=i.dataset.undoTarget;if(a==="refresh")this.refreshUndo();else if(a==="cancel-task"&&r)this.cancelHeldTask(r);else if(a==="rollback"&&r)this.rollbackWorkItem(r);else if(a==="execute"&&r)this.executeUndo(r)})}async refreshData(){try{let e=p.getState(),t=e.auditFilters,n=e.activeProfile||void 0,[i,a,r,s,l,d,g,u,m,v,c]=await Promise.all([b.getConfig(),b.listCapabilities(n),b.listResources(n),b.listPrompts(n),b.getCatalogEvents(),b.listApprovals(),b.listTasks(),b.listAuditEvents({server_id:t.serverId!=="all"?t.serverId:void 0,event_type:t.eventType!=="all"?t.eventType:void 0,status:t.status!=="all"?t.status:void 0,search:t.search.trim()?t.search.trim():void 0,limit:t.limit,offset:t.offset}),b.getAuditStats(),b.getClients().catch(()=>({ok:!1,clients:[]})),b.getSecrets().catch(()=>({ok:!1,secrets:[],keychain_service:"warmplane"}))]);if(v&&v.ok&&Array.isArray(v.clients))p.setState({clients:v.clients});if(c&&c.ok&&Array.isArray(c.secrets))p.setState({secrets:c.secrets});if(i.ok)p.setState({configPath:i.config_path,config:i.config,serverStatuses:i.server_statuses||{},circuitBreakers:i.circuit_breakers||[],metrics:{totalCatalogRequests:i.metrics?.total_catalog_requests||0,totalEtagHits:i.metrics?.total_etag_hits||0,totalToolCalls:i.metrics?.total_tool_calls||0,totalToolDurationUs:i.metrics?.total_tool_duration_us||0}});if(a&&Array.isArray(a.capabilities)){let f=p.getState().selectedCapabilityId,y=a.capabilities.some((x)=>x.id===f)?f:a.capabilities.length>0?a.capabilities[0].id:null;p.setState({capabilities:a.capabilities,capabilitiesHiddenByPolicy:a.hidden_by_policy||0,selectedCapabilityId:y})}if(r&&Array.isArray(r.resources)){let f=p.getState().selectedResourceId,y=r.resources.some((x)=>x.uri===f||x.id===f)?f:r.resources.length>0?r.resources[0].uri||r.resources[0].id||null:null;p.setState({resources:r.resources,resourcesHiddenByPolicy:r.hidden_by_policy||0,selectedResourceId:y})}if(s&&Array.isArray(s.prompts)){let f=p.getState().selectedPromptId,y=s.prompts.some((x)=>x.name===f||x.id===f)?f:s.prompts.length>0?s.prompts[0].name||s.prompts[0].id||null:null;p.setState({prompts:s.prompts,promptsHiddenByPolicy:s.hidden_by_policy||0,selectedPromptId:y})}if(l&&Array.isArray(l.events))p.setState({catalogEvents:l.events});if(d&&Array.isArray(d.approvals))p.setState({approvals:d.approvals});if(g&&Array.isArray(g.tasks))p.setState({tasks:g.tasks});if(await this.refreshUndo(),u&&Array.isArray(u.events))p.setState({auditEvents:u.events,auditTotal:u.total??u.events.length});if(m&&m.ok)p.setState({auditStats:m})}catch(e){console.error("Failed to fetch daemon state:",e)}}async refreshAuditEvents(){try{let t=p.getState().auditFilters,[n,i]=await Promise.all([b.listAuditEvents({server_id:t.serverId!=="all"?t.serverId:void 0,event_type:t.eventType!=="all"?t.eventType:void 0,status:t.status!=="all"?t.status:void 0,search:t.search.trim()?t.search.trim():void 0,limit:t.limit,offset:t.offset}),b.getAuditStats()]);if(n&&Array.isArray(n.events))p.setState({auditEvents:n.events,auditTotal:n.total??n.events.length});if(i&&i.ok)p.setState({auditStats:i})}catch(e){console.error("Failed to refresh audit events:",e)}}handleAuditSearchInput(e){let n={...p.getState().auditFilters,search:e,offset:0};p.setState({auditFilters:n}),clearTimeout(this.auditSearchTimeout),this.auditSearchTimeout=setTimeout(()=>{this.refreshAuditEvents()},250)}handleAuditStatusFilter(e){let t=p.getState();p.setState({auditFilters:{...t.auditFilters,status:e,offset:0}}),this.refreshAuditEvents()}handleAuditEventTypeFilter(e){let t=p.getState();p.setState({auditFilters:{...t.auditFilters,eventType:e,offset:0}}),this.refreshAuditEvents()}handleAuditServerFilter(e){let t=p.getState();p.setState({auditFilters:{...t.auditFilters,serverId:e,offset:0}}),this.refreshAuditEvents()}handleAuditPageSize(e){let t=parseInt(e,10)||25,n=p.getState();p.setState({auditFilters:{...n.auditFilters,limit:t,offset:0}}),this.refreshAuditEvents()}clearAuditFilters(){let e=p.getState();p.setState({auditFilters:{search:"",status:"all",eventType:"all",serverId:"all",limit:e.auditFilters.limit||25,offset:0}}),this.refreshAuditEvents()}auditPrevPage(){let e=p.getState(),{limit:t,offset:n}=e.auditFilters,i=Math.max(0,n-t);if(i!==n)p.setState({auditFilters:{...e.auditFilters,offset:i}}),this.refreshAuditEvents()}auditNextPage(){let e=p.getState(),{limit:t,offset:n}=e.auditFilters,i=e.auditTotal;if(n+t<i)p.setState({auditFilters:{...e.auditFilters,offset:n+t}}),this.refreshAuditEvents()}auditGoToPage(e){let t=p.getState(),{limit:n}=t.auditFilters,i=Math.max(0,(e-1)*n);p.setState({auditFilters:{...t.auditFilters,offset:i}}),this.refreshAuditEvents()}selectAuditEvent(e){if(!e){p.setState({auditSelectedEvent:null});return}let n=p.getState().auditEvents.find((i)=>i.id===e)||null;p.setState({auditSelectedEvent:n})}async verifyAuditChain(){try{let e=await b.verifyAuditChain();if(e&&e.report)p.setState({auditVerification:e.report})}catch(e){console.error("Failed to verify audit chain:",e)}}async refreshApprovals(){try{let e=await b.listApprovals();if(e&&Array.isArray(e.approvals))p.setState({approvals:e.approvals})}catch(e){console.error("Failed to refresh approvals:",e)}}initSSE(){try{let e=new EventSource("/v1/resources/updates");e.onmessage=(t)=>{p.addEventLog("SSE","/v1/resources/updates","UPDATED","0.1ms"),this.refreshData()},e.addEventListener("action_capsule",()=>{this.refreshUndo(),this.refreshTasks()}),e.addEventListener("error",()=>{if(p.getState().activeTab==="undo")this.refreshUndo()})}catch(e){console.warn("SSE connection unavailable")}}async refreshUndo(){try{let e=await b.listUndo();if(e.ok&&e.result&&Array.isArray(e.result.entries))p.setState({undoEntries:e.result.entries,undoError:null});else p.setState({undoError:e.error?.message||e.error?.code||"Undo entries could not be loaded."})}catch(e){p.setState({undoError:e?.message||"Undo entries could not be loaded."})}}async executeUndo(e){if(!confirm("Run the configured inverse for this action?"))return;let t=await b.executeUndo(e);if(!t.ok)alert(`Undo failed: ${t.error?.message||t.error?.code||"Request failed"}`);await this.refreshUndo()}async rollbackWorkItem(e){if(!confirm("Run configured inverses for this work item in reverse order?"))return;let t=await b.rollbackWorkItem(e);if(!t.ok)alert(`Rollback incomplete: ${t.error?.message||t.error?.code||"One or more inverses failed"}`);await this.refreshUndo()}async cancelHeldTask(e){if(!confirm("Cancel this send before it is released?"))return;let t=await b.cancelTask(e,"Cancelled from Control Deck Action Feed");if(!t.ok)alert(`Cancellation failed: ${t.error?.message||t.error?.code||"Request failed"}`);await this.refreshTasks()}async testUndoContract(e){if(!confirm(`Run the configured forward and inverse cycle on the test server for '${e}'?`))return;let t=await b.testUndoContract(e),n=t.result;if(t.ok&&n?.observationally_equivalent)alert(`Undo contract passed on ${n.test_server}. The observed state matches its snapshot.`);else alert(`Undo contract failed: ${n?.error_code||t.error?.message||"state did not return to its snapshot"}`)}switchTab(e){p.setState({activeTab:e}),this.refreshData()}render(){let e=p.getState(),t=document.getElementById("app-main");if(!t)return;let n=(e.tasks||[]).filter((d)=>d.status==="input_required").length,i=(e.approvals||[]).filter((d)=>d.status==="pending").length,a=Math.max(n,i),r=document.getElementById("nav-approvals-badge");if(r)r.textContent=a>0?`${a}`:"",r.style.display=a>0?"inline-block":"none";document.querySelectorAll(".nav-item").forEach((d)=>{let g=d.getAttribute("data-tab");if(g===e.activeTab||e.activeTab==="tasks"&&g==="approvals"||e.activeTab==="approvals"&&g==="tasks")d.classList.add("active");else d.classList.remove("active")});let s=document.getElementById("top-title"),l={overview:"Overview Cockpit",servers:"Server Hub & Connections",playground:"MCP Capability Playground",tasks:"SEP-2663 Tasks & HITL Review",undo:"Action Feed & Undo",approvals:"SEP-2663 Tasks & HITL Review",audit:"WORM Audit & Compliance Ledger",policy:"Security Governance & Redaction",secrets:"Native OS Keychain & Secrets Vault",aliases:"Facade & Alias Studio",profiles:"Server Constellation Profiles"};if(s)s.textContent=l[e.activeTab]||"Control Deck";switch(this.renderTopProfileSelector(),e.activeTab){case"overview":t.innerHTML=Y();break;case"servers":t.innerHTML=X();break;case"playground":t.innerHTML=Z();break;case"tasks":case"approvals":t.innerHTML=ee(e);break;case"undo":t.innerHTML=te(e);break;case"audit":t.innerHTML=re();break;case"policy":t.innerHTML=se();break;case"secrets":t.innerHTML=oe();break;case"aliases":t.innerHTML=ae();break;case"profiles":t.innerHTML=ne();break}}toggleClientsCollapse(){let e=p.getState().clientsCollapsed;p.setState({clientsCollapsed:!e}),this.render()}async saveNewVaultSecret(){let e=document.getElementById("vault-new-key"),t=document.getElementById("vault-new-val"),n=document.getElementById("vault-new-service"),i=e?.value.trim(),a=t?.value.trim(),r=n?.value.trim()||"warmplane";if(!i||!a){alert("Key and secret value are required");return}try{let s=await b.saveSecret(i,a,r);if(s.ok){if(alert(`Secret '${i}' saved securely into OS Keychain!
Reference: ${s.uri}`),e)e.value="";if(t)t.value="";await this.refreshData()}else alert(`Failed to save secret: ${s.error}`)}catch(s){alert(`Error saving secret: ${s.message}`)}}async deleteVaultSecret(e,t){let n=t?`Are you sure you want to remove secret '${e}' from OS Keychain and server '${t}'?`:`Are you sure you want to remove secret '${e}' from OS Keychain?`;if(!confirm(n))return;try{let i=await b.deleteSecret(e);if(i.ok||i.error?.includes("not found")){if(t){let s=(p.getState().config.mcpServers||{})[t];if(s&&s.env&&s.env[e]){let l={...s.env};delete l[e];let d={...s,env:l};await b.upsertServer(t,d)}}await this.refreshData()}else alert(`Failed to delete secret: ${i.error}`)}catch(i){alert(`Error deleting secret: ${i.message}`)}}async removeSecretFromConfig(e,t){if(!confirm(`Remove environment variable '${t}' from '${e}' configuration?`))return;try{let a=(p.getState().config.mcpServers||{})[e];if(a&&a.env&&a.env[t]){let r={...a.env};delete r[t];let s={...a,env:r},l=await b.upsertServer(e,s);if(l.ok)await this.refreshData();else alert(`Failed to update server config: ${l.error}`)}}catch(n){alert(`Error removing secret from config: ${n.message}`)}}async quickVaultEnv(e,t){let n=prompt(`Enter secret value to store in OS Keychain for ${e}.${t}:`);if(!n)return;try{let i=await b.saveSecret(t,n,"warmplane");if(!i.ok){alert(`Failed to save to Keychain: ${i.error}`);return}let s=(p.getState().config.mcpServers||{})[e];if(s){let l={...s.env||{},[t]:`keychain://warmplane/${t}`},d={...s,env:l},g=await b.upsertServer(e,d);if(g.ok)await this.refreshData(),alert(`Successfully configured ${e}.${t} in OS Keychain!`);else alert(`Failed to update server config: ${g.error}`)}}catch(i){alert(`Error during migration: ${i.message}`)}}async refreshTasks(){try{let e=await b.listTasks();if(e&&Array.isArray(e.tasks))p.setState({tasks:e.tasks})}catch(e){console.error("Failed to refresh tasks:",e)}}filterTasksByStatus(e){p.setState({taskFilterStatus:e})}togglePlaygroundAsyncTask(e){p.setState({playgroundAsyncTask:e})}async submitTaskInputResponses(e){let n=p.getState().tasks.find((r)=>r.taskId===e)?.inputRequests||{},i=Object.keys(n),a={};if(i.length>0)for(let r of i){let s=n[r];if(s&&s.type==="approval_review"){let l=document.getElementById(`task-input-${e}-${r}-decision`),d=document.getElementById(`task-input-${e}-${r}`),g=l?l.value==="true":!0,u=void 0;if(d&&d.value.trim())try{u=JSON.parse(d.value.trim())}catch{alert("Invalid JSON in parameters editor");return}a[r]={approved:g,modified_args:u,reason:g?void 0:"Operator rejected execution via Tasks review"}}else{let l=document.getElementById(`task-input-${e}-${r}`);if(l){let d=l.value.trim();try{a[r]=JSON.parse(d)}catch{a[r]=d}}}}else{let r=document.getElementById(`task-raw-input-${e}`);if(r&&r.value.trim())try{Object.assign(a,JSON.parse(r.value.trim()))}catch{alert("Invalid JSON in raw input responses");return}}try{let r=await b.updateTask(e,a);if(r.ok)await this.refreshTasks();else alert(`Task update failed: ${r.error?.message||r.error||"Unknown error"}`)}catch(r){alert(`Error updating task: ${r.message}`)}}async promptCancelTask(e){let t=prompt("Reason for cancelling task:");if(t===null)return;try{let n=await b.cancelTask(e,t||void 0);if(n.ok)await this.refreshTasks();else alert(`Task cancellation failed: ${n.error?.message||n.error||"Unknown error"}`)}catch(n){alert(`Error cancelling task: ${n.message}`)}}async openTaskInspectorModal(e){this.closeModals();let t=document.getElementById("modal-task-inspector");if(!t)return;let n=document.getElementById("modal-task-title"),i=document.getElementById("modal-task-body"),a=document.getElementById("modal-task-footer");if(n)n.textContent=`Task Inspector: ${e}`;if(i)i.innerHTML=`
        <div style="padding: 30px; text-align: center; color: var(--text-dim); font-family: var(--ff-mono); font-size: 12px;">
          ⏳ Fetching task execution record...
        </div>
      `;t.classList.add("active");try{let r=await b.getTask(e);if(!r.ok||!r.task){if(i)i.innerHTML=`
            <div style="background: rgba(248, 113, 113, 0.12); border: 1px solid rgba(248, 113, 113, 0.3); border-radius: var(--radius-sm); padding: 16px; color: var(--red-400);">
              <div style="font-weight: 700; margin-bottom: 6px;">Failed to load task snapshot</div>
              <div style="font-family: var(--ff-mono); font-size: 11.5px;">${o(r.error?.message||"Task not found in runtime registry")}</div>
            </div>
          `;return}let s=r.task,l=s.progress!==void 0?Math.round(s.progress*100):s.status==="completed"?100:s.status==="working"?50:0,d=Math.floor(Date.now()/1000),g=s.expiresAtEpochSecs?Math.max(0,s.expiresAtEpochSecs-d):s.ttlSeconds||300,u=s.status==="completed"||s.status==="cancelled"||s.status==="failed",m=s.status==="completed"?"var(--green-400)":s.status==="working"?"var(--cyan-400)":s.status==="input_required"?"var(--amber-300)":s.status==="cancelled"?"var(--text-muted)":"var(--red-400)",v=s.status==="completed"?"rgba(52, 211, 153, 0.15)":s.status==="working"?"rgba(56, 189, 248, 0.15)":s.status==="input_required"?"rgba(245, 158, 11, 0.18)":s.status==="cancelled"?"rgba(148, 163, 184, 0.15)":"rgba(248, 113, 113, 0.15)",c=!!s.error,f=s.result!==void 0&&s.result!==null,h=s.inputRequests&&Object.keys(s.inputRequests).length>0;if(i)i.innerHTML=`
          <!-- Status Banner & Top Metrics -->
          <div style="display: flex; justify-content: space-between; align-items: center; background: var(--surface-card); border: 1px solid var(--border); border-radius: var(--radius-md); padding: 12px 16px; margin-bottom: 14px;">
            <div>
              <div style="display: flex; align-items: center; gap: 8px;">
                <span class="brand-badge" style="background: ${v}; color: ${m}; border-color: ${m}; font-size: 11px;">
                  ${o(s.status.toUpperCase())}
                </span>
                <span style="font-family: var(--ff-mono); font-size: 13px; font-weight: 700; color: var(--text-main);">
                  ${o(s.capabilityId||"Tool Execution")}
                </span>
                ${s.serverId?`<span style="font-size: 11px; color: var(--text-dim);">via <code style="color: var(--cyan-400);">${o(s.serverId)}</code></span>`:""}
              </div>
              <div style="font-family: var(--ff-mono); font-size: 11px; color: var(--text-dim); margin-top: 4px;">
                Task ID: <span style="color: var(--text-muted);">${o(s.taskId)}</span>
              </div>
            </div>
            <div style="text-align: right; font-family: var(--ff-mono); font-size: 11px;">
              <div style="color: var(--text-dim);">Created: <span style="color: var(--text-muted);">${s.createdAtEpochSecs?new Date(s.createdAtEpochSecs*1000).toLocaleString():s.createdAt?new Date(s.createdAt).toLocaleString():"—"}</span></div>
              ${!u?`<div style="color: var(--amber-400); margin-top: 2px;">TTL: ${g}s remaining</div>`:""}
            </div>
          </div>

          <!-- Progress Bar -->
          <div style="margin-bottom: 16px;">
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 4px; font-family: var(--ff-mono); font-size: 11px;">
              <span style="color: var(--text-muted); text-transform: uppercase; font-weight: 600;">Execution Progress</span>
              <span style="color: ${m}; font-weight: 700;">${l}% ${s.total?`(step ${Math.round((s.progress||0)*s.total)} of ${s.total})`:""}</span>
            </div>
            <div style="height: 8px; background: var(--surface-card); border-radius: 4px; overflow: hidden; border: 1px solid var(--border);">
              <div style="height: 100%; width: ${l}%; background: ${m}; transition: width 0.3s;"></div>
            </div>
          </div>

          <!-- Caller Context Envelope -->
          ${s.context?`
            <div style="background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 8px 12px; font-family: var(--ff-mono); font-size: 11px; display: flex; flex-wrap: wrap; gap: 16px; margin-bottom: 14px; color: var(--text-muted);">
              ${s.context.actor_id?`<div><span style="color: var(--text-dim);">Actor:</span> <span style="color: var(--cyan-400);">${o(s.context.actor_id)}</span></div>`:""}
              ${s.context.operation_id?`<div><span style="color: var(--text-dim);">Operation:</span> <span style="color: var(--text-main);">${o(s.context.operation_id)}</span></div>`:""}
              ${s.context.grant_id?`<div><span style="color: var(--text-dim);">Grant:</span> <span style="color: var(--text-main);">${o(s.context.grant_id)}</span></div>`:""}
            </div>
          `:""}

          <!-- Diagnostic / Error Message -->
          ${c?`
            <div style="background: rgba(248, 113, 113, 0.1); border: 1px solid rgba(248, 113, 113, 0.3); border-radius: var(--radius-sm); padding: 12px; margin-bottom: 14px;">
              <div style="font-size: 11px; font-weight: 700; color: var(--red-400); text-transform: uppercase; margin-bottom: 6px;">
                ⚠️ Failure / Cancellation Trace
              </div>
              <pre style="font-family: var(--ff-mono); font-size: 11.5px; color: var(--red-300); white-space: pre-wrap; word-break: break-word; margin: 0;">${o(typeof s.error==="string"?s.error:JSON.stringify(s.error,null,2))}</pre>
            </div>
          `:""}

          <!-- Input Requests (if awaiting input) -->
          ${h?`
            <div style="margin-bottom: 14px;">
              <div style="font-size: 11px; font-weight: 700; color: var(--amber-400); text-transform: uppercase; margin-bottom: 6px;">
                ⚡ Pending Input Requests (MRTR / HITL)
              </div>
              <pre style="background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 12px; font-family: var(--ff-mono); font-size: 11.5px; color: var(--amber-300); white-space: pre-wrap; word-break: break-word; max-height: 180px; overflow-y: auto; margin: 0;">${o(JSON.stringify(s.inputRequests,null,2))}</pre>
            </div>
          `:""}

          <!-- Result Payload -->
          <div style="margin-bottom: 8px;">
            <div style="font-size: 11px; font-weight: 700; color: var(--text-muted); text-transform: uppercase; margin-bottom: 6px;">
              ${f?"\uD83D\uDCE6 Output Result Payload":"State Details"}
            </div>
            <pre style="background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 12px; font-family: var(--ff-mono); font-size: 11.5px; color: var(--text-main); white-space: pre-wrap; word-break: break-word; max-height: 220px; overflow-y: auto; margin: 0;">${o(f?JSON.stringify(s.result,null,2):s.status==="working"?"Task execution is currently in-flight in background worker pool.":"No output payload recorded.")}</pre>
          </div>
        `;if(a)a.innerHTML=`
          <div>
            ${!u?`
              <button class="btn btn-danger" style="font-size: 11.5px;" onclick="window.app.promptCancelTask('${o(s.taskId)}'); window.app.closeModals();">
                ⛔ Cancel Task
              </button>
            `:`
              <span style="font-family: var(--ff-mono); font-size: 11px; color: var(--text-dim);">Task is terminal (${o(s.status)})</span>
            `}
          </div>
          <div style="display: flex; gap: 8px;">
            <button class="btn btn-ghost" style="font-size: 11.5px;" onclick="navigator.clipboard.writeText('${o(s.taskId)}'); alert('Task ID copied to clipboard');">
              \uD83D\uDCCB Copy Task ID
            </button>
            <button class="btn btn-ghost" style="font-size: 11.5px;" onclick="window.app.closeModals()">
              Close
            </button>
          </div>
        `}catch(r){if(i)i.innerHTML=`
          <div style="color: var(--red-400); font-family: var(--ff-mono); font-size: 11.5px;">
            Failed to inspect task: ${o(r.message)}
          </div>
        `}}async submitApproval(e){let t=document.getElementById(`appr-operator-${e}`),n=document.getElementById(`appr-args-${e}`),i=t?.value.trim()||"security-operator",a=void 0;if(n&&n.value.trim())try{a=JSON.parse(n.value.trim())}catch{alert("Invalid JSON in arguments editor");return}let r=await b.approveTicket(e,i,a);if(r.ok)await this.refreshApprovals(),await this.refreshTasks();else alert(`Approval failed: ${r.error||"Unknown error"}`)}async promptReject(e){let t=prompt("Reason for rejection (will be returned to the calling agent):");if(t===null)return;let i=document.getElementById(`appr-operator-${e}`)?.value.trim()||"security-operator",a=await b.rejectTicket(e,i,t);if(a.ok)await this.refreshApprovals(),await this.refreshTasks();else alert(`Rejection failed: ${a.error||"Unknown error"}`)}setPlaygroundMode(e){p.setState({playgroundMode:e})}selectCapability(e){p.setState({selectedCapabilityId:e});let t=p.getState().capabilities.find((i)=>i.id===e),n=document.getElementById("pg-args-input");if(t){let i=K(t.input_schema,!1),a=JSON.stringify(i,null,2);if(n)n.value=a;let r={...p.getState().playgroundArgs||{}};r[e]=a,p.getState().playgroundArgs=r}}selectResource(e){p.setState({selectedResourceId:e})}selectPrompt(e){p.setState({selectedPromptId:e})}filterResources(e){let t=e.toLowerCase().trim(),i=(p.getState().resources||[]).filter((r)=>r.id.toLowerCase().includes(t)||r.name&&r.name.toLowerCase().includes(t)||r.uri&&r.uri.toLowerCase().includes(t)||r.server&&r.server.toLowerCase().includes(t)),a=document.getElementById("pg-res-list");if(a)if(i.length===0)a.innerHTML=`
          <div style="padding: 24px 16px; text-align: center; color: var(--text-dim); font-size: 11.5px;">
            No resources match "${o(e)}"
          </div>
        `;else a.innerHTML=i.map((r)=>{let s=r.id===p.getState().selectedResourceId?"active":"",l=r.uri?r.uri.split(":")[0]:"res";return`
            <div class="cap-item ${s}" onclick="window.app.selectResource('${o(r.id)}')">
              <div style="display: flex; justify-content: space-between; align-items: center;">
                <span style="font-weight: 600; color: var(--text-main); font-family: var(--ff-mono); font-size: 12px;">${o(r.name||r.id)}</span>
                <span class="badge" style="font-size: 9.5px; background: rgba(56, 189, 248, 0.15); color: var(--cyan-400);">${o(l)}</span>
              </div>
              <div style="font-size: 11px; color: var(--text-dim); margin-top: 2px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">${o(r.uri)}</div>
              <div style="display: flex; justify-content: space-between; font-size: 10px; color: var(--text-muted); margin-top: 4px;">
                <span>server: ${o(r.server||"local")}</span>
                <span>${o(r.mime_type||"text/plain")}</span>
              </div>
            </div>
          `}).join("")}filterPrompts(e){let t=e.toLowerCase().trim(),i=(p.getState().prompts||[]).filter((r)=>r.id.toLowerCase().includes(t)||r.name&&r.name.toLowerCase().includes(t)||r.description&&r.description.toLowerCase().includes(t)||r.server&&r.server.toLowerCase().includes(t)),a=document.getElementById("pg-prompt-list");if(a)if(i.length===0)a.innerHTML=`
          <div style="padding: 24px 16px; text-align: center; color: var(--text-dim); font-size: 11.5px;">
            No prompts match "${o(e)}"
          </div>
        `;else a.innerHTML=i.map((r)=>{let s=r.id===p.getState().selectedPromptId?"active":"",l=r.arguments?r.arguments.length:0;return`
            <div class="cap-item ${s}" onclick="window.app.selectPrompt('${o(r.id)}')">
              <div style="display: flex; justify-content: space-between; align-items: center;">
                <span style="font-weight: 600; color: var(--text-main); font-family: var(--ff-mono); font-size: 12px;">${o(r.name||r.id)}</span>
                <span class="badge" style="font-size: 9.5px; background: rgba(168, 85, 247, 0.15); color: var(--purple-400);">${l} args</span>
              </div>
              <div style="font-size: 11px; color: var(--text-dim); margin-top: 2px;">${o(r.description||r.title||"Prompt template")}</div>
              <div style="font-size: 10px; color: var(--text-muted); margin-top: 4px;">server: ${o(r.server||"local")}</div>
            </div>
          `}).join("")}updatePlaygroundArgs(e){let t=p.getState(),n=t.selectedCapabilityId||(t.capabilities[0]?t.capabilities[0].id:null);if(!n)return;let i={...t.playgroundArgs||{}};i[n]=e,t.playgroundArgs=i}fillPlaygroundSampleArgs(e=!1){let t=p.getState(),n=t.selectedCapabilityId||(t.capabilities[0]?t.capabilities[0].id:null),i=t.capabilities.find((l)=>l.id===n),a=document.getElementById("pg-args-input");if(!a)return;if(!i||!i.input_schema){if(a.value="{}",n){let l={...t.playgroundArgs||{}};l[n]="{}",t.playgroundArgs=l}return}let r=K(i.input_schema,e),s=JSON.stringify(r,null,2);if(a.value=s,n){let l={...t.playgroundArgs||{}};l[n]=s,t.playgroundArgs=l}}formatPlaygroundArgs(){let e=p.getState(),t=e.selectedCapabilityId||(e.capabilities[0]?e.capabilities[0].id:null),n=document.getElementById("pg-args-input");if(n)try{let i=JSON.parse(n.value||"{}"),a=JSON.stringify(i,null,2);if(n.value=a,t){let r={...e.playgroundArgs||{}};r[t]=a,e.playgroundArgs=r}}catch(i){alert(`Cannot format JSON: ${i.message}`)}}insertPlaygroundArgKey(e,t,n){let i=p.getState(),a=i.selectedCapabilityId||(i.capabilities[0]?i.capabilities[0].id:null),r=document.getElementById("pg-args-input");if(r){let s={};try{s=JSON.parse(r.value||"{}")}catch{s={}}if(s[e]===void 0)if(n!==null&&n!==void 0)s[e]=n;else switch(t){case"string":s[e]=`sample_${e}`;break;case"number":case"integer":s[e]=0;break;case"boolean":s[e]=!0;break;case"array":s[e]=[];break;case"object":s[e]={};break;default:s[e]=`sample_${e}`}let l=JSON.stringify(s,null,2);if(r.value=l,a){let d={...i.playgroundArgs||{}};d[a]=l,i.playgroundArgs=d}}}fillBatchStepSampleArgs(e){let t=p.getState(),n=[...t.batchSteps||[]],i=n[e];if(!i||!i.capability_id)return;let a=t.capabilities.find((d)=>d.id===i.capability_id);if(!a||!a.input_schema)return;let r=a.input_schema.properties||{},s={};for(let[d,g]of Object.entries(r))if(g.default!==void 0)s[d]=g.default;else if(Array.isArray(g.enum)&&g.enum.length>0)s[d]=g.enum[0];else switch(g.type||"string"){case"string":s[d]=`sample_${d}`;break;case"number":case"integer":s[d]=0;break;case"boolean":s[d]=!0;break;case"array":s[d]=[];break;case"object":s[d]={};break;default:s[d]=`sample_${d}`}let l=JSON.stringify(s,null,2);n[e]={...n[e],argsJson:l},p.setState({batchSteps:n})}filterCapabilities(e){let t=e.toLowerCase().trim(),i=p.getState().capabilities.filter((r)=>r.id.toLowerCase().includes(t)||r.summary&&r.summary.toLowerCase().includes(t)||r.server&&r.server.toLowerCase().includes(t)),a=document.getElementById("pg-cap-list");if(a)if(i.length===0)a.innerHTML=`
          <div style="padding: 24px 16px; text-align: center; color: var(--text-dim); font-size: 11.5px;">
            No capabilities match "${o(e)}"
          </div>
        `;else a.innerHTML=i.map((r)=>`
          <div class="cap-item ${r.id===p.getState().selectedCapabilityId?"active":""}" onclick="window.app.selectCapability('${o(r.id)}')">
            <div style="display: flex; justify-content: space-between; align-items: center;">
              <span style="font-weight: 600; color: var(--text-main); font-family: var(--ff-mono); font-size: 12px;">${o(r.id)}</span>
              <span style="font-size: 10px; color: var(--green-400);">${o(r.mode||"read")}</span>
            </div>
            <div style="font-size: 11px; color: var(--text-dim); margin-top: 2px;">server: ${o(r.server||"local")}</div>
          </div>
        `).join("")}async executePlaygroundTool(){let e=p.getState(),t=e.selectedCapabilityId||(e.capabilities[0]?e.capabilities[0].id:null);if(!t)return;let n=document.getElementById("pg-args-input")?.value||"{}",i=document.getElementById("pg-context-input")?.value||void 0,a=document.getElementById("pg-jsonpath-input")?.value.trim()||void 0,r=document.getElementById("pg-limit-lines-input")?.value.trim()||void 0,s=document.getElementById("pg-truncate-bytes-input")?.value.trim()||void 0,l={};try{l=JSON.parse(n)}catch{alert("Invalid arguments JSON object");return}if(a)l._jsonpath=a;if(r&&!isNaN(Number(r)))l._limit_lines=Number(r);if(s&&!isNaN(Number(s)))l._truncate_bytes=Number(s);let d=`op-${Date.now()}`;p.setState({isExecuting:!0,activeRequestId:d});let g=e.activeProfile||void 0,u=e.playgroundAsyncTask||!1;try{let m=await b.callCapability({capability_id:t,args:l,request_id:d,async_task:u?!0:void 0,context:{operation_id:i||d}},g);if(p.setState({isExecuting:!1,activeRequestId:null,executionResult:{status:m.status,durationMs:m.durationMs,data:m.data}}),m.status===202||m.data?.resultType==="task")this.refreshTasks();p.addEventLog("POST",`/v1/tools/call → ${t}`,m.status===200?"200 OK":`HTTP ${m.status}`,`${m.durationMs.toFixed(1)}ms`),b.getConfig().then((v)=>{if(v.ok&&v.circuit_breakers)p.setState({circuitBreakers:v.circuit_breakers})})}catch(m){p.setState({isExecuting:!1,activeRequestId:null,executionResult:{status:500,durationMs:0,data:{error:m.toString()}}})}}async cancelActiveOperation(){let t=p.getState().activeRequestId;if(t)try{await b.cancelOperation(t)}catch(n){console.warn("Failed to send cancel signal:",n)}p.setState({isExecuting:!1,activeRequestId:null,executionResult:{status:499,durationMs:0,data:{ok:!1,error:{code:"CANCELLED",message:"Operation cancelled by operator"}}}})}openBatchModal(){let e=p.getState(),t=e.batchSteps;if(!t||t.length===0)t=[{id:"step_1",capability_id:e.selectedCapabilityId||(e.capabilities[0]?e.capabilities[0].id:""),argsJson:"{}",continue_on_error:!1},{id:"step_2",capability_id:"",argsJson:"{}",continue_on_error:!0}],p.setState({batchSteps:t});p.setState({isBatchModalOpen:!0})}closeBatchModal(){p.setState({isBatchModalOpen:!1})}addBatchStep(){let t=[...p.getState().batchSteps||[]],n=t.length+1;t.push({id:`step_${n}`,capability_id:"",argsJson:"{}",continue_on_error:!1}),p.setState({batchSteps:t})}removeBatchStep(e){let n=[...p.getState().batchSteps||[]];if(n.length<=1){alert("Pipeline must contain at least one execution step.");return}n.splice(e,1);let i=n.map((a,r)=>({...a,id:`step_${r+1}`}));p.setState({batchSteps:i})}updateBatchStepCapability(e,t){let i=[...p.getState().batchSteps||[]];if(i[e])i[e]={...i[e],capability_id:t},p.setState({batchSteps:i})}updateBatchStepContinueOnError(e,t){let i=[...p.getState().batchSteps||[]];if(i[e])i[e]={...i[e],continue_on_error:t},p.setState({batchSteps:i})}updateBatchStepArgs(e,t){let n=p.getState(),i=[...n.batchSteps||[]];if(i[e])i[e]={...i[e],argsJson:t},n.batchSteps[e].argsJson=t}appendBatchVariable(e,t){let i=[...p.getState().batchSteps||[]],a=document.getElementById(`batch-step-args-${e}`);if(a){let r=a.value,s=a.selectionStart||r.length,l=a.selectionEnd||r.length,d=r.substring(0,s)+t+r.substring(l);if(a.value=d,i[e])i[e]={...i[e],argsJson:d},p.setState({batchSteps:i})}}async executeBatchPipeline(){let e=p.getState(),t=e.batchSteps||[],n=[];for(let a=0;a<t.length;a++){let r=t[a];if(!r.capability_id){alert(`Please select a capability for Step ${a+1}`);return}let s={};try{s=JSON.parse(r.argsJson||"{}")}catch{alert(`Invalid JSON in Step ${a+1} arguments`);return}n.push({id:r.id||`step_${a+1}`,capability_id:r.capability_id,args:s,continue_on_error:r.continue_on_error})}p.setState({isBatchModalOpen:!1});let i=e.activeProfile||void 0;try{let a=await b.batchCallCapabilities(n,i);p.setState({executionResult:{status:a.status,durationMs:a.durationMs,data:a.data}}),p.addEventLog("POST",`/v1/tools/batch_call (${t.length} steps)`,a.status===200?"200 OK":`HTTP ${a.status}`,`${a.durationMs.toFixed(1)}ms`)}catch(a){p.setState({executionResult:{status:500,durationMs:0,data:{error:a.toString()}}})}}async executeReadResource(){let e=p.getState(),t=e.selectedResourceId||(e.resources[0]?e.resources[0].id:null);if(!t)return;let n=document.getElementById("pg-res-jsonpath-input")?.value.trim()||void 0,i=document.getElementById("pg-res-lines-input")?.value.trim()||void 0,a=document.getElementById("pg-res-bytes-input")?.value.trim()||void 0,r={resource_id:t};if(n)r._jsonpath=n;if(i&&!isNaN(Number(i)))r._limit_lines=Number(i);if(a&&!isNaN(Number(a)))r._truncate_bytes=Number(a);let s=e.activeProfile||void 0;try{let l=await b.readResource({resource_id:t,input_responses:r},s);p.setState({resourceReadResult:{status:l.status,durationMs:l.durationMs,data:l.data}}),p.addEventLog("POST",`/v1/resources/read → ${t}`,l.status===200?"200 OK":`HTTP ${l.status}`,`${l.durationMs.toFixed(1)}ms`)}catch(l){p.setState({resourceReadResult:{status:500,durationMs:0,data:{error:l.toString()}}})}}async executeGetPrompt(){let e=p.getState(),t=e.selectedPromptId||(e.prompts[0]?e.prompts[0].id:null);if(!t)return;let n=document.querySelectorAll(".prompt-arg-input"),i={};n.forEach((r)=>{let s=r,l=s.getAttribute("data-arg-name");if(l&&s.value.trim())i[l]=s.value.trim()});let a=e.activeProfile||void 0;try{let r=await b.getPrompt({prompt_id:t,arguments:i},a);p.setState({promptGetResult:{status:r.status,durationMs:r.durationMs,data:r.data}}),p.addEventLog("POST",`/v1/prompts/get → ${t}`,r.status===200?"200 OK":`HTTP ${r.status}`,`${r.durationMs.toFixed(1)}ms`)}catch(r){p.setState({promptGetResult:{status:500,durationMs:0,data:{error:r.toString()}}})}}toggleBatchPlayground(){let e=document.getElementById("pg-args-input");if(!e)return;let t=[{id:"step_1",capability_id:"sqlite.read_query",args:{query:"SELECT * FROM users LIMIT 2"}},{id:"step_2",capability_id:"github.issues.search",args:{query:"label:bug"},continue_on_error:!0}];e.value=JSON.stringify(t,null,2)}async submitPolicyRule(e){let t=e==="allow"?"policy-new-allow":e==="deny"?"policy-new-deny":e==="redact"?"policy-new-redact":"policy-new-requireApproval",n=document.getElementById(t);if(!n)return;let i=n.value.trim();if(!i)return;await this.addPolicyRule(e,i),n.value=""}async addPolicyRule(e,t){let n=(t||"").trim();if(!n)return;let i=p.getState(),a=i.activeProfile,r=a?i.config.profiles?.[a]:void 0;if(r&&a){let s=r.policy||{},l=[...s.allow||[]],d=[...s.deny||[]],g=[...s.redact_keys||s.redactKeys||[]],u=[...s.require_approval||s.requireApproval||[]];if(e==="allow"&&!l.includes(n))l.push(n);if(e==="deny"&&!d.includes(n))d.push(n);if(e==="redact"&&!g.includes(n))g.push(n);if(e==="requireApproval"&&!u.includes(n))u.push(n);let m={...s,allow:l,deny:d,redactKeys:g,requireApproval:u},v=await b.upsertProfile(a,r.servers,r.description,m);if(!v.ok)alert(`Failed to save profile policy rule: ${v.error||"Unknown error"}`)}else{let s=i.config.policy||{},l=[...s.allow||[]],d=[...s.deny||[]],g=[...s.redact_keys||s.redactKeys||[]],u=[...s.require_approval||s.requireApproval||[]];if(e==="allow"&&!l.includes(n))l.push(n);if(e==="deny"&&!d.includes(n))d.push(n);if(e==="redact"&&!g.includes(n))g.push(n);if(e==="requireApproval"&&!u.includes(n))u.push(n);let m=await b.savePolicy({...s,allow:l,deny:d,redact_keys:g,redactKeys:g,require_approval:u,requireApproval:u});if(!m.ok)alert(`Failed to save policy rule: ${m.error||"Unknown error"}`)}await this.refreshData()}async removePolicyRule(e,t){let n=p.getState(),i=n.activeProfile,a=i?n.config.profiles?.[i]:void 0;if(a&&i){let r=a.policy||{},s=[...r.allow||[]],l=[...r.deny||[]],d=[...r.redact_keys||r.redactKeys||[]],g=[...r.require_approval||r.requireApproval||[]];if(e==="allow")s.splice(t,1);if(e==="deny")l.splice(t,1);if(e==="redact")d.splice(t,1);if(e==="requireApproval")g.splice(t,1);let u={...r,allow:s,deny:l,redactKeys:d,requireApproval:g},m=await b.upsertProfile(i,a.servers,a.description,u);if(!m.ok)alert(`Failed to update profile policy: ${m.error||"Unknown error"}`)}else{let r=n.config.policy||{},s=[...r.allow||[]],l=[...r.deny||[]],d=[...r.redact_keys||r.redactKeys||[]],g=[...r.require_approval||r.requireApproval||[]];if(e==="allow")s.splice(t,1);if(e==="deny")l.splice(t,1);if(e==="redact")d.splice(t,1);if(e==="requireApproval")g.splice(t,1);let u=await b.savePolicy({...r,allow:s,deny:l,redact_keys:d,redactKeys:d,require_approval:g,requireApproval:g});if(!u.ok)alert(`Failed to update policy: ${u.error||"Unknown error"}`)}await this.refreshData()}async saveWebhookConfig(){let e=document.getElementById("policy-webhook-url"),t=document.getElementById("policy-webhook-format"),n=document.getElementById("policy-webhook-secret"),i=e?e.value.trim():"",a=t?t.value:"generic",r=n?n.value.trim():"",l=p.getState().config.policy||{},d=i?{url:i,format:a,secret:r&&!r.startsWith("WARMPLANE_")&&!r.includes("_")?r:void 0,secret_env:r&&(r.startsWith("WARMPLANE_")||r.includes("_"))?r:void 0,events:["approval.requested","circuit_breaker.tripped","policy.violation"]}:void 0,g=await b.savePolicy({...l,webhook:d});if(g.ok)alert("Webhook settings saved successfully");else alert(`Failed to save webhook settings: ${g.error||"Unknown error"}`);await this.refreshData()}async testWebhook(){let e=document.getElementById("policy-webhook-url"),t=document.getElementById("policy-webhook-format"),n=e?e.value.trim():void 0,i=t?t.value:void 0,a=p.getState(),r=typeof a.config.policy?.webhook==="object"?a.config.policy.webhook?.url:void 0;if(n&&n!==r)await this.saveWebhookConfig();let s=document.getElementById("policy-webhook-status");if(s)s.textContent="Sending test event...",s.style.color="var(--cyan-400)";try{let l=await b.testWebhook(n,i);if(l.ok){if(alert(`Test webhook sent successfully! (${l.message})`),s)s.textContent=`✔ Test sent (HTTP ${l.status_code||200})`,s.style.color="var(--green-400)"}else if(alert(`Test webhook failed: ${l.error||"Unknown error"}`),s)s.textContent=`✖ Failed: ${l.error}`,s.style.color="var(--red-400)"}catch(l){alert(`Error sending test webhook: ${l.message}`)}}testPolicySandbox(e){let t=document.getElementById("policy-test-verdict");if(!t)return;let n=e.trim();if(!n){t.textContent="ENTER ID",t.style.color="var(--text-dim)";return}let i=p.getState(),a=i.activeProfile,r=a?i.config.profiles?.[a]:void 0,s=i.config.policy||{},l=r?.policy,d=s.deny||[],g=l?.deny||[],u=Array.from(new Set([...d,...g])),m=s.require_approval||s.requireApproval||[],v=l?.require_approval||l?.requireApproval||[],c=Array.from(new Set([...m,...v])),f=l&&l.allow&&l.allow.length>0?l.allow:s.allow||[],h=(y,x)=>{if(y==="*")return!0;if(y.endsWith("*"))return x.startsWith(y.slice(0,-1));return y===x};if(u.some((y)=>h(y,n))){t.textContent="DENIED (Strict Block)",t.style.color="var(--red-400)";return}if(f.length>0&&!f.some((y)=>h(y,n))){t.textContent="DENIED (Not in Allow List)",t.style.color="var(--red-400)";return}if(c.some((y)=>h(y,n))){t.textContent="REQUIRE APPROVAL (HITL Gate)",t.style.color="var(--amber-400)";return}t.textContent="ALLOWED",t.style.color="var(--green-400)"}async deleteServer(e){let t=p.getState().activeProfile,n=t?`Permanently delete server '${e}' globally from warmplane configuration? (This will also unbind it from profile '${t}')`:`Are you sure you want to permanently remove server '${e}' from configuration?`;if(!confirm(n))return;try{let i=await b.deleteServer(e);if(i.ok)await this.refreshData();else alert(`Failed to remove server '${e}': ${i.error||"Unknown error"}`)}catch(i){alert(`Error removing server '${e}': ${i.message}`)}}async restartServer(e){try{let t=await b.restartServer(e);if(t.ok)await this.refreshData();else alert(`Failed to restart server '${e}': ${t.error||"Unknown error"}`)}catch(t){alert(`Error restarting server '${e}': ${t.message}`)}}openServerDiagnosticsModal(e){this.closeModals();let t=p.getState(),n=t.config.mcpServers?.[e],i=t.serverStatuses?.[e],a=(t.circuitBreakers||[]).find((d)=>d.server_id===e),r=document.getElementById("modal-server-diagnostics");if(!r)return;let s=document.getElementById("modal-diag-title"),l=document.getElementById("modal-diag-body");if(s)s.textContent=`Live Diagnostics: ${e}`;if(l){let d=i?.status==="degraded",g=d?"var(--amber-400)":i?.status==="connected"?"var(--green-400)":"var(--red-400)",u=i?.error||"No active crash or error reported. Server is healthy.",m=N(e,n?.command,n?.args),v=n?.env||{},c=Object.keys(v),h=(m?.envFields||[]).filter((x)=>x.required).filter((x)=>!c.includes(x.key)),y="";if(m||c.length>0){let x=(m?.envFields||[]).map((E)=>{let k=v[E.key]!==void 0,C='<span class="brand-badge" style="color: var(--green-400); border-color: rgba(52, 211, 153, 0.3);">Configured</span>';if(!k&&E.required)C='<span class="brand-badge" style="color: var(--red-400); border-color: rgba(248, 113, 113, 0.4); background: rgba(248, 113, 113, 0.1);">Required / Missing</span>';else if(!k)C='<span class="brand-badge" style="color: var(--text-dim); border-color: var(--border);">Optional / Not Set</span>';return`
            <div style="display: flex; justify-content: space-between; align-items: center; padding: 6px 10px; background: var(--surface); border-radius: var(--radius-xs); margin-bottom: 6px; font-size: 11.5px;">
              <div>
                <span style="font-family: var(--ff-mono); font-weight: 700; color: ${!k&&E.required?"var(--amber-300)":"var(--text-main)"};">${o(E.key)}</span>
                ${E.label?`<span style="font-size: 10.5px; color: var(--text-dim); margin-left: 6px;">(${o(E.label)})</span>`:""}
              </div>
              <div style="display: flex; align-items: center; gap: 8px;">
                ${C}
                ${!k?`
                  <button class="btn btn-primary" style="padding: 2px 8px; font-size: 10.5px;" onclick="window.app.quickVaultEnv('${o(e)}', '${o(E.key)}')">➕ Configure in Keychain</button>
                `:""}
              </div>
            </div>
          `}).join(""),T=(m?.envFields||[]).map((E)=>E.key),I=Object.entries(v).filter(([E])=>!T.includes(E)).map(([E,w])=>`
            <div style="display: flex; justify-content: space-between; align-items: center; padding: 6px 10px; background: var(--surface); border-radius: var(--radius-xs); margin-bottom: 6px; font-size: 11.5px;">
              <span style="font-family: var(--ff-mono); font-weight: 700; color: var(--text-main);">${o(E)}</span>
              <span class="brand-badge" style="color: var(--green-400); border-color: rgba(52, 211, 153, 0.3);">Custom Configured</span>
            </div>
          `).join("");y=`
          <div style="background: rgba(0,0,0,0.2); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 12px; margin-bottom: 14px;">
            <div style="font-size: 11px; font-weight: 700; color: ${h.length>0?"var(--amber-400)":"var(--text-main)"}; text-transform: uppercase; margin-bottom: 8px; display: flex; justify-content: space-between; align-items: center;">
              <span>\uD83D\uDD11 Environment Variables &amp; Secrets</span>
              ${h.length>0?`<span style="color: var(--red-400); font-size: 10.5px;">⚠️ ${h.length} required key(s) missing</span>`:""}
            </div>
            ${x}
            ${I}
          </div>
        `}l.innerHTML=`
        <div style="display: flex; gap: 10px; align-items: center; margin-bottom: 16px;">
          <span style="width: 10px; height: 10px; border-radius: 50%; background: ${g};"></span>
          <span style="font-weight: 700; font-size: 14px; color: var(--text-main);">Current Status: <span style="color: ${g}; text-transform: uppercase;">${o(i?.status||"unknown")}</span></span>
          <span class="brand-badge" style="color: var(--cyan-400);">Protocol: ${o(i?.protocol_version||"2026-07-28")}</span>
          ${h.length>0?'<span class="brand-badge" style="color: var(--red-400); border-color: rgba(248, 113, 113, 0.4); background: rgba(248, 113, 113, 0.1);">⚠️ Missing Required Keys</span>':""}
        </div>

        <div style="background: rgba(0,0,0,0.3); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 12px; margin-bottom: 14px;">
          <div style="font-size: 11px; font-weight: 700; color: var(--amber-400); text-transform: uppercase; margin-bottom: 6px;">
            ⚠️ Diagnostic Details / Failure Root Cause
          </div>
          <pre style="font-family: var(--ff-mono); font-size: 11.5px; color: ${d?"var(--red-300)":"var(--text-dim)"}; white-space: pre-wrap; word-break: break-word; margin: 0;">${o(u)}</pre>
        </div>

        ${y}

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
              Auto-Restart: ${n?.resilience?.autoRestart!==!1?"ENABLED":"DISABLED"}
            </div>
          </div>
        </div>

        <div style="background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 10px; margin-bottom: 16px;">
          <div style="font-size: 10.5px; color: var(--text-dim); margin-bottom: 4px;">Configured Execution Target</div>
          <code style="font-family: var(--ff-mono); font-size: 11px; color: var(--cyan-400); display: block; word-break: break-all;">
            ${n?.command?`${o(n.command)} ${o((n.args||[]).join(" "))}`:o(n?.url||"")}
          </code>
        </div>

        <div style="display: flex; justify-content: flex-end; gap: 8px;">
          <button class="btn btn-primary" onclick="window.app.restartServer('${o(e)}'); window.app.closeModals();">⚡ Restart &amp; Probe Now</button>
          <button class="btn btn-ghost" onclick="window.app.closeModals()">Close</button>
        </div>
      `}r.classList.add("active")}openAddServerModal(){this.closeModals();let e=document.getElementById("modal-srv-title"),t=document.getElementById("modal-srv-template-banner"),n=document.getElementById("modal-srv-name"),i=document.getElementById("modal-srv-transport"),a=document.getElementById("modal-srv-command"),r=document.getElementById("modal-srv-url"),s=document.getElementById("modal-srv-ft"),l=document.getElementById("modal-srv-cd"),d=document.getElementById("modal-srv-autorestart"),g=document.getElementById("modal-srv-maxrestarts");if(e)e.textContent="Add Upstream MCP Server";if(t)t.style.display="flex";if(n)n.value="",n.disabled=!1;if(i)i.value="stdio";if(a)a.value="";if(r)r.value="";let u=document.getElementById("modal-group-cmd"),m=document.getElementById("modal-group-url");if(u)u.style.display="block";if(m)m.style.display="none";if(s)s.value="3";if(l)l.value="30000";if(d)d.value="true";if(g)g.value="5";let v=document.getElementById("modal-add-server");if(v)v.classList.add("active")}openEditServerModal(e){this.closeModals();let t=p.getState(),n=t.config.mcpServers?.[e];if(!n){alert(`Server '${e}' not found in configuration.`);return}let i=document.getElementById("modal-srv-title"),a=document.getElementById("modal-srv-template-banner"),r=document.getElementById("modal-srv-name"),s=document.getElementById("modal-srv-transport"),l=document.getElementById("modal-srv-command"),d=document.getElementById("modal-srv-url"),g=document.getElementById("modal-srv-ft"),u=document.getElementById("modal-srv-cd"),m=document.getElementById("modal-srv-autorestart"),v=document.getElementById("modal-srv-maxrestarts");if(i)i.textContent=`Edit Server '${e}'`;if(a)a.style.display="none";if(r)r.value=e,r.disabled=!0;let c=!!n.command;if(s)s.value=c?"stdio":"http";let f=document.getElementById("modal-group-cmd"),h=document.getElementById("modal-group-url");if(f)f.style.display=c?"block":"none";if(h)h.style.display=c?"none":"block";if(l)l.value=c?`${n.command} ${(n.args||[]).join(" ")}`.trim():"";if(d)d.value=n.url||"";let y=n.resilience||t.config.resilience;if(g)g.value=String(y?.failureThreshold??3);if(u)u.value=String(y?.cooldownMs??30000);if(m)m.value=y?.autoRestart===!1?"false":"true";if(v)v.value=String(y?.maxRestarts??5);let x=document.getElementById("modal-add-server");if(x)x.classList.add("active")}async submitAddServer(){let e=document.getElementById("modal-srv-name"),t=e?.value.trim(),n=document.getElementById("modal-srv-transport")?.value;if(!t){alert("Server name is required");return}if(e&&!e.disabled){if((p.getState().config.mcpServers||{})[t]){if(!confirm(`Server '${t}' already exists in configuration. Do you want to overwrite it?`))return}}let i={};if(n==="stdio"){let u=(document.getElementById("modal-srv-command")?.value.trim()).split(/\s+/).filter(Boolean);if(u.length===0){alert("Command is required");return}i.command=u[0],i.args=u.slice(1)}else{let g=document.getElementById("modal-srv-url")?.value.trim();if(!g){alert("URL is required");return}i.url=g}let a=document.getElementById("modal-srv-ft")?.value.trim(),r=document.getElementById("modal-srv-cd")?.value.trim(),s=document.getElementById("modal-srv-autorestart")?.value,l=document.getElementById("modal-srv-maxrestarts")?.value.trim();if(a||r||s||l)i.resilience={failureThreshold:a?Number(a):3,cooldownMs:r?Number(r):30000,autoRestart:s!=="false",maxRestarts:l?Number(l):5};let d=await b.upsertServer(t,i);if(d.ok)this.closeModals(),await this.refreshData();else alert(`Failed to save server: ${d.error}`)}openTemplateCatalog(){this.closeModals();let e=document.getElementById("modal-templates");if(e)e.classList.add("active");this.renderTemplateGrid()}setTemplateCategory(e){this.activeTemplateCategory=e,document.querySelectorAll(".tmpl-cat-btn").forEach((t)=>{if(t.getAttribute("data-category")===e)t.classList.add("active"),t.style.background="var(--surface-elevated)",t.style.color="var(--amber-400)";else t.classList.remove("active"),t.style.background="var(--surface-card)",t.style.color="var(--text-main)"}),this.renderTemplateGrid()}filterTemplates(e){this.activeTemplateFilter=e.toLowerCase().trim(),this.renderTemplateGrid()}renderTemplateGrid(){let e=document.getElementById("tmpl-grid");if(!e)return;let t=D.filter((a)=>{let r=this.activeTemplateCategory==="all"||a.category===this.activeTemplateCategory,s=!this.activeTemplateFilter||a.name.toLowerCase().includes(this.activeTemplateFilter)||a.id.toLowerCase().includes(this.activeTemplateFilter)||a.description.toLowerCase().includes(this.activeTemplateFilter)||a.command.toLowerCase().includes(this.activeTemplateFilter)||a.envFields.some((l)=>l.key.toLowerCase().includes(this.activeTemplateFilter));return r&&s});if(t.length===0){e.innerHTML=`
        <div style="grid-column: span 2; padding: 32px; text-align: center; color: var(--text-dim);">
          No matching MCP server templates found.
        </div>
      `;return}let i=p.getState().config.mcpServers||{};e.innerHTML=t.map((a)=>{let r=!!i[a.id],s=`${a.command} ${a.defaultArgs.join(" ")}`;return`
        <div class="bento-card" style="display: flex; flex-direction: column; justify-content: space-between; padding: 14px; background: var(--surface); border: 1px solid var(--border); min-width: 0; transition: transform 0.15s, border-color 0.15s;">
          <div>
            <div style="display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 6px;">
              <div style="display: flex; align-items: center; gap: 8px; min-width: 0;">
                <span style="font-weight: 700; font-size: 13.5px; color: var(--text-main); white-space: nowrap; overflow: hidden; text-overflow: ellipsis;">${o(a.name)}</span>
                <span class="brand-badge" style="font-size: 9.5px; padding: 1px 6px; flex-shrink: 0;">${o(a.badge)}</span>
              </div>
              ${r?'<span style="font-size: 10px; color: var(--green-400); font-weight: 600; flex-shrink: 0;">CONNECTED</span>':""}
            </div>
            <div style="font-size: 11.5px; color: var(--text-muted); line-height: 1.4; margin-bottom: 8px;">
              ${o(a.description)}
            </div>
            <div style="font-family: var(--ff-mono); font-size: 10.5px; color: var(--text-dim); background: var(--surface-card); padding: 5px 8px; border-radius: var(--radius-sm); border: 1px solid var(--border-subtle); overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">
              <code>${o(s)}</code>
            </div>
            ${a.envFields.length>0?`
              <div style="font-size: 10.5px; color: var(--amber-400); margin-top: 6px; display: flex; align-items: center; gap: 4px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">
                <span>⚡ Needs:</span>
                <code style="overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">${a.envFields.map((l)=>o(l.key)).join(", ")}</code>
              </div>
            `:""}
          </div>

          <div style="display: flex; justify-content: flex-end; margin-top: 12px; gap: 6px;">
            <button class="btn btn-primary" style="font-size: 11.5px; padding: 4px 10px;" onclick="window.app.selectTemplate('${o(a.id)}')">
              ${r?"Configure Another":"✨ 1-Click Setup"}
            </button>
          </div>
        </div>
      `}).join("")}selectTemplate(e){let t=D.find((d)=>d.id===e);if(!t)return;this.selectedTemplate=t,this.closeModals();let n=document.getElementById("modal-configure-template");if(n)n.classList.add("active");let i=document.getElementById("cfg-tmpl-title"),a=document.getElementById("cfg-tmpl-desc"),r=document.getElementById("cfg-tmpl-form");if(i)i.textContent=`Configure ${t.name} Server`;if(a)a.textContent=t.description;let s=p.getState().config.mcpServers||{},l=t.id;if(s[l]){let d=2;while(s[`${t.id}-${d}`])d++;l=`${t.id}-${d}`}if(r){let d="";if(t.envFields.length>0)d=`
          <div style="margin-top: 14px; margin-bottom: 6px; font-weight: 700; font-size: 11px; text-transform: uppercase; color: var(--amber-400); letter-spacing: 0.5px;">
            Environment Variables &amp; API Keys
          </div>
          ${t.envFields.map((g)=>`
            <div class="form-group">
              <label class="form-label">${o(g.label)} ${g.required?'<span style="color: var(--red-400);">*</span>':"(Optional)"}</label>
              <input type="password" class="form-input tmpl-env-input" data-key="${o(g.key)}" placeholder="${o(g.placeholder||"")}">
              ${g.description?`<div style="font-size: 10.5px; color: var(--text-dim); margin-top: 3px;">${o(g.description)}</div>`:""}
            </div>
          `).join("")}
        `;r.innerHTML=`
        <div class="form-group">
          <label class="form-label">Server Identifier (Name)</label>
          <input type="text" class="form-input" id="cfg-srv-id" value="${o(l)}">
          <div style="font-size: 10.5px; color: var(--text-dim); margin-top: 3px;">Must be unique across all configured servers.</div>
        </div>
        <div class="form-group">
          <label class="form-label">Command Line Arguments</label>
          <input type="text" class="form-input" id="cfg-srv-args" value="${o(t.defaultArgs.join(" "))}" placeholder="${o(t.argsPlaceholder||"")}">
          <div style="font-size: 10.5px; color: var(--text-dim); margin-top: 3px;">Executable: <code>${o(t.command)}</code></div>
        </div>
        ${d}
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
      `}}async submitTemplateServer(){if(!this.selectedTemplate)return;let e=this.selectedTemplate,t=document.getElementById("cfg-srv-id")?.value.trim(),n=document.getElementById("cfg-srv-args")?.value.trim();if(!t){alert("Server identifier is required");return}if((p.getState().config.mcpServers||{})[t]){if(!confirm(`Server '${t}' already exists. Do you want to overwrite its configuration?`))return}let r=n?n.split(/\s+/).filter(Boolean):[],s={},l=document.querySelectorAll(".tmpl-env-input");for(let f of Array.from(l)){let h=f.getAttribute("data-key"),y=f.value.trim(),x=e.envFields.find((T)=>T.key===h);if(x?.required&&!y){alert(`Required field '${x.label}' is missing.`);return}if(h&&y)s[h]=y}let d={command:e.command,args:r};if(Object.keys(s).length>0)d.env=s;let g=document.getElementById("cfg-srv-ft")?.value.trim(),u=document.getElementById("cfg-srv-cd")?.value.trim(),m=document.getElementById("cfg-srv-autorestart")?.value,v=document.getElementById("cfg-srv-maxrestarts")?.value.trim();if(g||u||m||v)d.resilience={failureThreshold:g?Number(g):3,cooldownMs:u?Number(u):30000,autoRestart:m!=="false",maxRestarts:v?Number(v):5};let c=await b.upsertServer(t,d);if(c.ok)this.closeModals(),await this.refreshData();else alert(`Failed to save server: ${c.error}`)}async openImportModal(){this.closeModals();let e=document.getElementById("modal-import");if(e)e.classList.add("active");let t=document.getElementById("modal-eco-list");if(!t)return;t.innerHTML='<div style="color: var(--text-dim); padding: 12px; text-align: center;">Scanning IDE configs...</div>';try{let n=await b.getEcosystemSources();if(n.sources&&n.sources.length>0)t.innerHTML=n.sources.map((i)=>`
          <label style="display: flex; align-items: center; gap: 10px; background: var(--surface); padding: 10px 14px; border-radius: var(--radius-sm); border: 1px solid var(--border); cursor: pointer;">
            <input type="checkbox" class="eco-checkbox" value="${i.path}" checked>
            <div>
              <div style="font-weight: 600; color: var(--text-main);">${i.name}</div>
              <div style="font-size: 11px; color: var(--text-dim);">${i.server_count} servers (${i.servers.join(", ")})</div>
            </div>
          </label>
        `).join("");else t.innerHTML='<div style="color: var(--text-dim); padding: 12px; text-align: center;">No external MCP configuration files found on this system.</div>'}catch{t.innerHTML='<div style="color: var(--red-400); padding: 12px; text-align: center;">Failed to scan ecosystem sources.</div>'}}async submitImport(){let e=document.querySelectorAll(".eco-checkbox:checked");if(e.length===0){alert("No sources selected");return}for(let t of Array.from(e))await b.importConfig(t.value,!1);this.closeModals(),await this.refreshData()}async refreshClients(){try{let e=await b.getClients();if(e.ok&&Array.isArray(e.clients))p.setState({clients:e.clients})}catch(e){console.error("Failed to scan clients:",e)}}setClientCategoryFilter(e){p.setState({clientFilterCategory:e})}setClientSearchQuery(e){p.setState({clientSearchQuery:e})}async attachClient(e,t){let n=t;if(!n){let i=document.getElementById(`client-prof-${e}`)||document.getElementById(`overview-client-prof-${e}`);if(i)n=i.value||void 0;else n=p.getState().activeProfile||void 0}this.openClientAttachModal(e,n)}openClientAttachModal(e,t){this.closeModals(),this.pendingClientId=e;let n=p.getState().config,i=n.mcpHttpServer,a=i?`http://${i.bind==="0.0.0.0"||i.bind==="::"?"127.0.0.1":i.bind||"127.0.0.1"}:${i.port||9191}/mcp`:"",r=document.getElementById("modal-client-title"),s=document.getElementById("modal-client-transport"),l=document.getElementById("modal-client-url"),d=document.getElementById("modal-client-profile"),g=document.getElementById("modal-client-profile-wrap");if(r)r.textContent=`Connect ${e}`;if(s)s.value="stdio";if(l)l.value=a;if(d){let u=Object.keys(n.profiles||{});if(d.innerHTML=`<option value="">All Tools (Default)</option>${u.map((m)=>`<option value="${o(m)}">Profile: ${o(m)}</option>`).join("")}`,d.value=t||"",g)g.style.display=u.length>0?"block":"none"}this.updateClientTransportForm(),document.getElementById("modal-client-attach")?.classList.add("active")}updateClientTransportForm(){let t=(document.getElementById("modal-client-transport")?.value||"stdio")==="http",n=document.getElementById("modal-client-http-group"),i=document.getElementById("modal-client-http-status"),a=!!p.getState().config.mcpHttpServer;if(n)n.style.display=t?"block":"none";if(i)i.textContent=a?"Uses the daemon Streamable HTTP endpoint. The daemon must already be running.":"HTTP is unavailable until mcpHttpServer is configured and the daemon is restarted.",i.style.color=a?"var(--text-dim)":"var(--amber-300)"}async submitClientAttach(){let e=this.pendingClientId;if(!e)return;let t=document.getElementById("modal-client-transport")?.value||"stdio",n=document.getElementById("modal-client-profile")?.value||void 0,i=document.getElementById("modal-client-url")?.value.trim()||void 0;if(t==="http"&&!p.getState().config.mcpHttpServer&&!i){alert("Configure mcpHttpServer and restart the daemon, or provide an explicit HTTP endpoint.");return}if(t==="http"&&!i){alert("Enter an HTTP endpoint URL.");return}let a=await b.attachClient(e,n,t,i);if(!a.ok)alert(`Failed to attach client: ${a.error||a.message||"Unknown error"}`);else this.pendingClientId=null,this.closeModals(),await this.refreshData()}async detachClient(e){if(!confirm("Disconnect Warmplane from this client?"))return;let t=await b.detachClient(e);if(!t.ok)alert(`Failed to detach client: ${t.error||t.message||"Unknown error"}`);else await this.refreshData()}handleAliasTargetInput(e){let t=document.getElementById("alias-suggestions-dropdown");if(!t)return;let n=(e||"").trim().toLowerCase();if(n.length<2){t.style.display="none";return}let a=p.getState().capabilities.filter((r)=>r.id.toLowerCase().includes(n)||r.summary&&r.summary.toLowerCase().includes(n)||r.description&&r.description.toLowerCase().includes(n)||r.server&&r.server.toLowerCase().includes(n)).slice(0,8);if(a.length===0){t.style.display="none";return}t.innerHTML=a.map((r)=>`
      <div style="padding: 8px 12px; cursor: pointer; border-bottom: 1px solid var(--border-subtle); display: flex; justify-content: space-between; align-items: center; transition: background 0.1s;"
           onmouseover="this.style.background='var(--surface-hover)'"
           onmouseout="this.style.background='transparent'"
           onmousedown="window.app.selectAliasSuggestion('${o(r.id)}')">
        <div>
          <div style="font-weight: 700; color: var(--text-main);">${o(r.id)}</div>
          <div style="font-size: 10.5px; color: var(--text-dim); margin-top: 2px;">${o(r.summary||r.description||"")}</div>
        </div>
        <span style="font-size: 10px; color: var(--cyan-400);">${o(r.server||"local")}</span>
      </div>
    `).join(""),t.style.display="block"}selectAliasSuggestion(e){let t=document.getElementById("alias-target");if(t)t.value=e;this.hideAliasDropdown()}hideAliasDropdown(){let e=document.getElementById("alias-suggestions-dropdown");if(e)e.style.display="none"}startEditAlias(e,t,n,i="",a=!1){let r=document.getElementById("alias-kind"),s=document.getElementById("alias-name"),l=document.getElementById("alias-target"),d=document.getElementById("alias-summary"),g=document.getElementById("alias-passthrough"),u=document.getElementById("alias-passthrough-container"),m=document.getElementById("alias-form-title"),v=document.getElementById("alias-save-btn"),c=document.getElementById("alias-cancel-btn");if(r)r.value=e;if(s)s.value=t;if(l)l.value=n;if(d)d.value=i;if(g)g.checked=a;if(u)u.style.display=e==="tool"?"flex":"none";if(m)m.innerHTML=`✏️ EDIT ALIAS: <span style="color: var(--amber-400); font-family: var(--ff-mono);">${t}</span>`;if(v)v.textContent="✓ Update";if(c)c.style.display="inline-block";s?.focus(),s?.scrollIntoView({behavior:"smooth",block:"nearest"})}resetAliasForm(){let e=document.getElementById("alias-kind"),t=document.getElementById("alias-name"),n=document.getElementById("alias-target"),i=document.getElementById("alias-summary"),a=document.getElementById("alias-passthrough"),r=document.getElementById("alias-passthrough-container"),s=document.getElementById("alias-form-title"),l=document.getElementById("alias-save-btn"),d=document.getElementById("alias-cancel-btn");if(e)e.value="tool";if(t)t.value="";if(n)n.value="";if(i)i.value="";if(a)a.checked=!1;if(r)r.style.display="flex";if(s)s.textContent="Create New Alias";if(l)l.textContent="+ Save";if(d)d.style.display="none"}async createAlias(){let e=document.getElementById("alias-kind")?.value,t=document.getElementById("alias-name")?.value.trim(),n=document.getElementById("alias-target")?.value.trim(),i=document.getElementById("alias-summary")?.value.trim()||void 0,a=document.getElementById("alias-passthrough")?.checked||!1;if(!t||!n){alert("Please provide both alias name and canonical target");return}if(e==="tool"&&a&&!/^[a-zA-Z0-9_-]{1,64}$/.test(t)){let r=t.replace(/[^a-zA-Z0-9_-]/g,"_").substring(0,64);if(!confirm(`MCP tool names must match ^[a-zA-Z0-9_-]{1,64}$.

'${t}' will be exported to MCP clients as '${r}'.

Do you want to proceed?`))return}await b.updateAlias(e,t,n,i,void 0,a),this.resetAliasForm(),await this.refreshData()}async deleteAlias(e,t){await b.updateAlias(e,t,void 0),this.resetAliasForm(),await this.refreshData()}async reloadFromDisk(){try{let e=await b.reloadConfig();if(e.ok){let t="Hot-reload completed successfully!";if(e.mounted&&e.mounted.length>0)t+=`
Mounted: ${e.mounted.join(", ")}`;if(e.unmounted&&e.unmounted.length>0)t+=`
Unmounted: ${e.unmounted.join(", ")}`;if(e.warnings&&e.warnings.length>0)t+=`
Warnings:
${e.warnings.join(`
`)}`;alert(t)}else alert(`Hot-reload failed: ${e.error||"Unknown error"}`)}catch(e){alert(`Error reaching daemon: ${e.message}`)}await this.refreshData()}renderTopProfileSelector(){let e=document.getElementById("top-profile-selector");if(!e)return;let t=p.getState(),n=t.config.profiles||{},i=Object.keys(n),a=t.activeProfile,r='<option value="">All Servers (Unrestricted)</option>';for(let s of i){let l=a===s?"selected":"";r+=`<option value="${o(s)}" ${l}>Profile: ${o(s)}</option>`}e.innerHTML=r}async setActiveProfile(e){p.setState({activeProfile:e||null}),await this.refreshData()}openAddProfileModal(){let e=document.getElementById("modal-prof-title");if(e)e.textContent="Create Server Constellation Profile";let t=document.getElementById("modal-prof-name"),n=document.getElementById("modal-prof-desc"),i=document.getElementById("modal-prof-mode");if(t)t.value="",t.disabled=!1;if(n)n.value="";if(i)i.value="create";let a=document.getElementById("modal-prof-allow"),r=document.getElementById("modal-prof-deny"),s=document.getElementById("modal-prof-hitl"),l=document.getElementById("modal-prof-redact");if(a)a.value="";if(r)r.value="";if(s)s.value="";if(l)l.value="";this.renderProfileServerCheckboxes([]);let d=document.getElementById("modal-add-profile");if(d)d.classList.add("active")}openEditProfileModal(e){let n=p.getState().config.profiles?.[e];if(!n)return;let i=document.getElementById("modal-prof-title");if(i)i.textContent=`Edit Profile: ${e}`;let a=document.getElementById("modal-prof-name"),r=document.getElementById("modal-prof-desc"),s=document.getElementById("modal-prof-mode");if(a)a.value=e,a.disabled=!0;if(r)r.value=n.description||"";if(s)s.value="edit";let l=document.getElementById("modal-prof-allow"),d=document.getElementById("modal-prof-deny"),g=document.getElementById("modal-prof-hitl"),u=document.getElementById("modal-prof-redact"),m=n.policy;if(l)l.value=(m?.allow||[]).join(", ");if(d)d.value=(m?.deny||[]).join(", ");if(g)g.value=(m?.require_approval||m?.requireApproval||[]).join(", ");if(u)u.value=(m?.redact_keys||m?.redactKeys||[]).join(", ");this.renderProfileServerCheckboxes(n.servers||[]);let v=document.getElementById("modal-add-profile");if(v)v.classList.add("active")}renderProfileServerCheckboxes(e){let t=document.getElementById("modal-prof-servers-list");if(!t)return;let n=p.getState(),i=Object.keys(n.config.mcpServers||{});if(i.length===0){t.innerHTML='<div style="font-size: 11.5px; color: var(--text-dim);">No MCP servers configured yet. Add servers first.</div>';return}t.innerHTML=i.map((a)=>{let r=e.includes(a)?"checked":"";return`
        <label style="display: flex; align-items: center; gap: 8px; font-size: 12px; cursor: pointer; padding: 4px 6px; border-radius: var(--radius-sm); transition: background 0.15s;" onmouseover="this.style.background='var(--surface-hover)'" onmouseout="this.style.background='transparent'">
          <input type="checkbox" class="prof-server-checkbox" value="${o(a)}" ${r} style="accent-color: var(--amber-400);">
          <span style="font-family: var(--ff-mono); font-weight: 600; color: var(--text-main);">${o(a)}</span>
        </label>
      `}).join("")}async saveProfile(){let e=document.getElementById("modal-prof-name"),t=document.getElementById("modal-prof-desc"),n=e?.value.trim(),i=t?.value.trim();if(!n){alert("Please enter a profile name");return}let a=document.querySelectorAll(".prof-server-checkbox:checked"),r=[];if(a.forEach((y)=>{r.push(y.value)}),r.length===0){alert("Please select at least one server to include in this constellation");return}let s=(y)=>{if(!y)return[];return y.split(",").map((x)=>x.trim()).filter((x)=>x.length>0)},l=document.getElementById("modal-prof-allow"),d=document.getElementById("modal-prof-deny"),g=document.getElementById("modal-prof-hitl"),u=document.getElementById("modal-prof-redact"),m=s(l?.value),v=s(d?.value),c=s(g?.value),f=s(u?.value),h=void 0;if(m.length>0||v.length>0||c.length>0||f.length>0)h={allow:m,deny:v,requireApproval:c,redactKeys:f};try{let y=await b.upsertProfile(n,r,i||void 0,h);if(y.ok)this.closeModals(),await this.refreshData();else alert(`Failed to save profile: ${y.error||"Unknown error"}`)}catch(y){alert(`Error saving profile: ${y.message}`)}}async deleteProfile(e){if(!confirm(`Are you sure you want to delete profile '${e}'?`))return;try{let t=await b.deleteProfile(e);if(t.ok){if(p.getState().activeProfile===e)p.setState({activeProfile:null});await this.refreshData()}else alert(`Failed to delete profile: ${t.error||"Unknown error"}`)}catch(t){alert(`Error deleting profile: ${t.message}`)}}async toggleServerInProfile(e,t,n){let a=p.getState().config.profiles?.[e];if(!a)return;let r=[...a.servers||[]];if(n){if(!r.includes(t))r.push(t)}else if(r=r.filter((s)=>s!==t),r.length===0){alert("A profile must contain at least one server. To remove the profile, delete it in the Profiles tab.");return}try{let s=await b.upsertProfile(e,r,a.description,a.policy);if(s.ok)await this.refreshData();else alert(`Failed to update profile constellation: ${s.error||"Unknown error"}`)}catch(s){alert(`Error updating profile constellation: ${s.message}`)}}closeModals(){document.querySelectorAll(".modal-backdrop").forEach((e)=>e.classList.remove("active"))}}var le=new ie;window.app=le;window.addEventListener("DOMContentLoaded",()=>le.init());
