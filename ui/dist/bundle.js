function i(e){if(e===null||e===void 0)return"";return String(e).replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;").replace(/'/g,"&#039;")}class G{state={configPath:"mcp_servers.json",config:{mcpServers:{}},serverStatuses:{},circuitBreakers:[],clients:[],clientFilterCategory:"all",clientSearchQuery:"",secrets:[],clientsCollapsed:!1,capabilities:[],capabilitiesHiddenByPolicy:0,resources:[],resourcesHiddenByPolicy:0,prompts:[],promptsHiddenByPolicy:0,catalogEvents:[],tasks:[],selectedTaskId:null,taskFilterStatus:"all",approvals:[],auditEvents:[],auditTotal:0,auditFilters:{search:"",status:"all",eventType:"all",serverId:"all",limit:25,offset:0},auditSelectedEvent:null,auditStats:null,auditVerification:null,selectedCapabilityId:null,selectedResourceId:null,selectedPromptId:null,playgroundMode:"tools",playgroundArgs:{},isExecuting:!1,playgroundAsyncTask:!1,activeRequestId:null,isBatchModalOpen:!1,batchSteps:[{id:"step_1",capability_id:"",argsJson:"{}",continue_on_error:!1},{id:"step_2",capability_id:"",argsJson:"{}",continue_on_error:!0}],activeTab:"overview",activeProfile:null,eventLogs:[],executionResult:null,resourceReadResult:null,promptGetResult:null,metrics:{totalCatalogRequests:0,totalEtagHits:0,totalToolCalls:0,totalToolDurationUs:0}};listeners=[];getState(){return this.state}setState(e){this.state={...this.state,...e},this.listeners.forEach((t)=>t(this.state))}subscribe(e){return this.listeners.push(e),()=>{this.listeners=this.listeners.filter((t)=>t!==e)}}addEventLog(e,t,a,n){let r=[{time:new Date().toLocaleTimeString(),method:e,target:t,status:a,latency:n},...this.state.eventLogs].slice(0,50);this.setState({eventLogs:r})}}var p=new G;class Q{baseUrl;constructor(e=""){this.baseUrl=e}async getConfig(){return(await fetch(`${this.baseUrl}/v1/config`)).json()}async listCapabilities(e){let t={};if(e)t["X-Warmplane-Profile"]=e;return(await fetch(`${this.baseUrl}/v1/capabilities`,{headers:t})).json()}async listResources(e){let t={};if(e)t["X-Warmplane-Profile"]=e;return(await fetch(`${this.baseUrl}/v1/resources`,{headers:t})).json()}async readResource(e,t){let a=performance.now(),n={"Content-Type":"application/json"};if(t)n["X-Warmplane-Profile"]=t;let o=await fetch(`${this.baseUrl}/v1/resources/read`,{method:"POST",headers:n,body:JSON.stringify(e)}),r=performance.now()-a,s=await o.json();return{status:o.status,durationMs:r,data:s}}async listPrompts(e){let t={};if(e)t["X-Warmplane-Profile"]=e;return(await fetch(`${this.baseUrl}/v1/prompts`,{headers:t})).json()}async getPrompt(e,t){let a=performance.now(),n={"Content-Type":"application/json"};if(t)n["X-Warmplane-Profile"]=t;let o=await fetch(`${this.baseUrl}/v1/prompts/get`,{method:"POST",headers:n,body:JSON.stringify(e)}),r=performance.now()-a,s=await o.json();return{status:o.status,durationMs:r,data:s}}async getCatalogEvents(e){let t=e?`?after=${encodeURIComponent(e)}`:"";return(await fetch(`${this.baseUrl}/v1/catalog/events${t}`)).json()}async callCapability(e,t){let a=performance.now(),n={"Content-Type":"application/json"};if(t)n["X-Warmplane-Profile"]=t;let o=await fetch(`${this.baseUrl}/v1/tools/call`,{method:"POST",headers:n,body:JSON.stringify(e)}),r=performance.now()-a,s=await o.json();return{status:o.status,durationMs:r,data:s}}async batchCallCapabilities(e,t){let a=performance.now(),n={"Content-Type":"application/json"};if(t)n["X-Warmplane-Profile"]=t;let o=await fetch(`${this.baseUrl}/v1/tools/batch_call`,{method:"POST",headers:n,body:JSON.stringify({steps:e})}),r=performance.now()-a,s=await o.json();return{status:o.status,durationMs:r,data:s}}async cancelOperation(e){return(await fetch(`${this.baseUrl}/v1/operations/${encodeURIComponent(e)}/cancel`,{method:"POST"})).json()}async completeArgument(e){return(await fetch(`${this.baseUrl}/v1/completion/complete`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify(e)})).json()}async upsertServer(e,t){return(await fetch(`${this.baseUrl}/v1/config/servers`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({name:e,server:t})})).json()}async deleteServer(e){return(await fetch(`${this.baseUrl}/v1/config/servers/${encodeURIComponent(e)}`,{method:"DELETE"})).json()}async restartServer(e){return(await fetch(`${this.baseUrl}/v1/config/servers/${encodeURIComponent(e)}/restart`,{method:"POST"})).json()}async upsertProfile(e,t,a,n){return(await fetch(`${this.baseUrl}/v1/config/profiles`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({name:e,servers:t,description:a,policy:n})})).json()}async deleteProfile(e){return(await fetch(`${this.baseUrl}/v1/config/profiles/${encodeURIComponent(e)}`,{method:"DELETE"})).json()}async getEcosystemSources(){return(await fetch(`${this.baseUrl}/v1/config/ecosystem`)).json()}async importConfig(e,t=!1){return(await fetch(`${this.baseUrl}/v1/config/import`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({source_path:e,overwrite:t})})).json()}async savePolicy(e){let t={allow:e.allow||[],deny:e.deny||[],redactKeys:e.redact_keys||e.redactKeys||[],requireApproval:e.require_approval||e.requireApproval||[],approvalTimeoutSecs:e.approvalTimeoutSecs||e.approval_timeout_secs||300,webhook:e.webhook};return(await fetch(`${this.baseUrl}/v1/config/policy`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify(t)})).json()}async listTasks(){return(await fetch(`${this.baseUrl}/v1/tasks`)).json()}async getTask(e){return(await fetch(`${this.baseUrl}/v1/tasks/${encodeURIComponent(e)}`)).json()}async updateTask(e,t){return(await fetch(`${this.baseUrl}/v1/tasks/${encodeURIComponent(e)}/update`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({inputResponses:t})})).json()}async cancelTask(e,t){return(await fetch(`${this.baseUrl}/v1/tasks/${encodeURIComponent(e)}/cancel`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({reason:t})})).json()}async listApprovals(){return(await fetch(`${this.baseUrl}/v1/approvals`)).json()}async approveTicket(e,t,a){return(await fetch(`${this.baseUrl}/v1/approvals/${encodeURIComponent(e)}/approve`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({operator:t,modified_args:a})})).json()}async rejectTicket(e,t,a){return(await fetch(`${this.baseUrl}/v1/approvals/${encodeURIComponent(e)}/reject`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({operator:t,reason:a})})).json()}async updateAlias(e,t,a,n,o,r){return(await fetch(`${this.baseUrl}/v1/config/alias`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({kind:e,alias:t,target:a,summary:n,description:o,passthrough:r})})).json()}async reloadConfig(){return(await fetch(`${this.baseUrl}/v1/config/reload`,{method:"POST"})).json()}async listAuditEvents(e){let t=new URLSearchParams;if(e?.actor_id)t.set("actor_id",e.actor_id);if(e?.server_id&&e.server_id!=="all")t.set("server_id",e.server_id);if(e?.capability_id)t.set("capability_id",e.capability_id);if(e?.event_type&&e.event_type!=="all")t.set("event_type",e.event_type);if(e?.status&&e.status!=="all")t.set("status",e.status);if(e?.trace_id)t.set("trace_id",e.trace_id);if(e?.request_id)t.set("request_id",e.request_id);if(e?.search)t.set("search",e.search);if(e?.limit)t.set("limit",String(e.limit));if(e?.offset!==void 0)t.set("offset",String(e.offset));let a=t.toString();return(await fetch(`${this.baseUrl}/v1/audit/events${a?`?${a}`:""}`)).json()}getAuditExportUrl(e,t="csv"){let a=new URLSearchParams;if(a.set("format",t),e?.actor_id)a.set("actor_id",e.actor_id);if(e?.server_id&&e.server_id!=="all")a.set("server_id",e.server_id);if(e?.capability_id)a.set("capability_id",e.capability_id);if(e?.event_type&&e.event_type!=="all")a.set("event_type",e.event_type);if(e?.status&&e.status!=="all")a.set("status",e.status);if(e?.trace_id)a.set("trace_id",e.trace_id);if(e?.request_id)a.set("request_id",e.request_id);if(e?.search)a.set("search",e.search);return`${this.baseUrl}/v1/audit/export?${a.toString()}`}async verifyAuditChain(){return(await fetch(`${this.baseUrl}/v1/audit/verify`)).json()}async getAuditStats(){return(await fetch(`${this.baseUrl}/v1/audit/stats`)).json()}async getClients(){return(await fetch(`${this.baseUrl}/v1/clients`)).json()}async attachClient(e,t,a="stdio",n){return(await fetch(`${this.baseUrl}/v1/clients/${encodeURIComponent(e)}/attach`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({profile:t||void 0,transport:a,http_url:n||void 0})})).json()}async detachClient(e){return(await fetch(`${this.baseUrl}/v1/clients/${encodeURIComponent(e)}/detach`,{method:"POST",headers:{"Content-Type":"application/json"}})).json()}async testWebhook(e,t){return(await fetch(`${this.baseUrl}/v1/webhooks/test`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({url:e||void 0,format:t||void 0})})).json()}async getSecrets(){return(await fetch(`${this.baseUrl}/v1/secrets`)).json()}async saveSecret(e,t,a){return(await fetch(`${this.baseUrl}/v1/secrets`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({key:e,value:t,service:a})})).json()}async deleteSecret(e){return(await fetch(`${this.baseUrl}/v1/secrets/${encodeURIComponent(e)}`,{method:"DELETE"})).json()}}var h=new Q;function Y(){let e=p.getState(),t=e.config.mcpServers||{},a=Object.keys(t),n=a.length,o="";if(a.length===0)o=`
      <div style="grid-column: 1 / -1; padding: 32px; text-align: center; color: var(--text-dim); background: var(--surface-card); border-radius: var(--radius-md); border: 1px dashed var(--border);">
        <div style="font-size: 14px; color: var(--text-main); font-weight: 600; margin-bottom: 6px;">No Upstream MCP Servers Connected</div>
        <div style="font-size: 12px; margin-bottom: 16px;">Initialize connections by adding a server or syncing existing IDE configurations.</div>
        <div style="display: flex; gap: 8px; justify-content: center;">
          <button class="btn btn-primary" onclick="window.app.openTemplateCatalog()">✨ Browse Templates</button>
          <button class="btn btn-ghost" onclick="window.app.openAddServerModal()">+ Add Custom</button>
          <button class="btn btn-ghost" onclick="window.app.openImportModal()">Sync from IDEs</button>
        </div>
      </div>
    `;else o=a.map((S)=>{let P=t[S],q=P.command?"stdio":"http / sse",H=P.command?`${P.command} ${(P.args||[]).join(" ")}`:P.url,z=e.serverStatuses[S]||{status:"connected",protocol_version:"2026-07-28"},F=z.status==="degraded",V=z.status==="error"||z.status==="disconnected",U=F?"var(--amber-400)":V?"var(--red-400)":"var(--green-400)";return`
        <div class="bento-card col-4" style="background: var(--surface); border: 1px solid var(--border);">
          <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
            <span style="font-weight: 700; color: var(--text-main); display: flex; align-items: center; gap: 6px;">
              <span style="width: 8px; height: 8px; border-radius: 50%; background: ${U}; display: inline-block;"></span>
              ${i(S)}
            </span>
            <span class="brand-badge">${q}</span>
          </div>
          <div style="font-family: var(--ff-mono); font-size: 11.5px; color: var(--text-dim); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; margin-bottom: 12px;" title="${i(H||"")}">
            ${i(H||"")}
          </div>
          <div style="display: flex; justify-content: space-between; font-size: 11px; color: var(--text-muted); border-top: 1px solid var(--border); padding-top: 8px;">
            <span>Status: <strong style="color: ${U};">${i(z.status)}</strong></span>
            <span>Protocol: ${z.protocol_version}</span>
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
      <span style="color: var(--text-dim);">${i(S.time)}</span>
      <span style="color: var(--cyan-400); font-weight: 600;">${i(S.method)}</span>
      <span style="color: var(--text-main); font-family: var(--ff-mono);">${i(S.target)}</span>
      <span style="color: var(--green-400);">${i(S.status)}</span>
      <span style="color: var(--amber-300); text-align: right;">${i(S.latency)}</span>
    </div>
  `).join(""),s=e.metrics,l=s.totalCatalogRequests,d=s.totalEtagHits,g=l>0?`${(d/l*100).toFixed(1)}%`:"0.0%",u=l>0?`${d} of ${l} requests served via HTTP 304`:"Waiting for client requests",m=s.totalToolCalls,v=m>0?`${(s.totalToolDurationUs/m/1000).toFixed(1)}ms`:"0.0ms",c=m>0?`${m} tool executions processed`:"Local worker task queues warm",y=Object.keys(e.config.capabilityAliases||{}).length+Object.keys(e.config.resourceAliases||{}).length+Object.keys(e.config.promptAliases||{}).length,b=y>0?`${y*18}B / call`:"0B",f=y>0?`${y} active facade aliases pruning prompt size`:"Configure aliases in Studio to reduce prompt size",x=e.tasks||[],T=x.filter((S)=>S.status==="input_required").length,I=x.filter((S)=>S.status==="working"||S.status==="input_required").length,E=e.clients||[],k=E.filter((S)=>S.is_attached).length,w=E.filter((S)=>S.config_exists&&!S.is_attached).length,C=e.clientsCollapsed,A=k>0?`<span class="brand-badge" style="color: var(--green-400); border-color: rgba(52, 211, 153, 0.3); background: rgba(52, 211, 153, 0.1);">⚡ ${k} Connected</span>`:w>0?`<span class="brand-badge" style="color: var(--amber-300); border-color: rgba(251, 191, 36, 0.3); background: rgba(251, 191, 36, 0.1);">○ ${w} Ready to Connect</span>`:'<span class="brand-badge" style="color: var(--text-dim);">No Apps Detected</span>',L=Object.keys(e.config.profiles||{}),M=e.activeProfile,R=e.clientFilterCategory||"all",B=(e.clientSearchQuery||"").toLowerCase().trim(),O=E.filter((S)=>{if(B){let P=S.name.toLowerCase().includes(B),q=S.id.toLowerCase().includes(B),H=S.category.toLowerCase().includes(B);if(!P&&!q&&!H)return!1}if(R==="connected")return S.is_attached;if(R==="ready")return S.config_exists||S.app_installed;if(R==="ides")return S.category.toLowerCase().includes("ide")||S.category.toLowerCase().includes("extension");if(R==="agents")return S.category.toLowerCase().includes("agent")||S.category.toLowerCase().includes("cli")||S.category.toLowerCase().includes("platform");return!0}),J=O.length===0?'<div style="padding: 16px; text-align: center; color: var(--text-dim); font-size: 11.5px; grid-column: 1 / -1;">No AI clients match current filter.</div>':O.map((S)=>{let{is_attached:P,config_exists:q,app_installed:H}=S,z="rgba(255, 255, 255, 0.2)",F="Not Found";if(P){z="var(--green-400)";let j=S.attached_transport==="http"?"HTTP":"stdio";F=S.attached_profile?`Connected · ${j} (${S.attached_profile})`:`Connected · ${j} (All Tools)`}else if(q)z="var(--amber-300)",F="Ready to Attach";else if(H)z="var(--cyan-400)",F="Installed";let V=L.map((j)=>`
      <option value="${i(j)}" ${M===j||S.attached_profile===j?"selected":""}>${i(j)}</option>
    `).join(""),U=P?`<button class="btn btn-ghost" style="padding: 2px 7px; font-size: 10px; color: var(--red-400);" onclick="event.stopPropagation(); window.app.detachClient('${i(S.id)}')">Detach</button>`:q||H?`
        <div style="display: flex; align-items: center; gap: 4px;" onclick="event.stopPropagation();">
          ${L.length>0?`
            <select id="overview-client-prof-${i(S.id)}" class="form-input" style="font-size: 10px; padding: 1px 4px; height: 22px; width: 85px;" title="Select constellation profile">
              <option value="" ${!M?"selected":""}>All Tools</option>
              ${V}
            </select>
          `:""}
          <button class="btn btn-primary" style="padding: 2px 7px; font-size: 10px;" onclick="window.app.attachClient('${i(S.id)}')">⚡ Connect</button>
        </div>
      `:"";return`
      <div style="background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 8px 12px; display: flex; align-items: center; justify-content: space-between; gap: 8px;">
        <div style="display: flex; align-items: center; gap: 8px; overflow: hidden;">
          <span style="width: 7px; height: 7px; border-radius: 50%; background: ${z}; flex-shrink: 0;"></span>
          <div style="overflow: hidden;">
            <div style="font-weight: 600; font-size: 12px; color: var(--text-main); white-space: nowrap; overflow: hidden; text-overflow: ellipsis;">${i(S.name)}</div>
            <div style="font-size: 10px; color: var(--text-dim);">${i(F)}</div>
          </div>
        </div>
        ${U}
      </div>
    `}).join(""),W=[{id:"all",label:"All"},{id:"ready",label:"Ready / Installed"},{id:"connected",label:"Connected"},{id:"ides",label:"IDEs"},{id:"agents",label:"Agents & CLIs"}].map((S)=>`<button class="btn btn-ghost" style="padding: 2px 8px; font-size: 10.5px; border-radius: 100px; ${R===S.id?"background: var(--amber-400); color: #000; font-weight: 700;":"background: var(--surface); color: var(--text-muted);"}" onclick="event.stopPropagation(); window.app.setClientCategoryFilter('${S.id}')">${S.label}</button>`).join(""),_=`
    <div class="bento-card" style="margin-top: 18px; padding: 12px 16px; border-color: rgba(245, 158, 11, 0.25); background: rgba(18, 24, 38, 0.4);">
      <div style="display: flex; justify-content: space-between; align-items: center; cursor: pointer; user-select: none;" onclick="window.app.toggleClientsCollapse()">
        <div style="display: flex; align-items: center; gap: 10px;">
          <span style="font-size: 13.5px; font-weight: 700; color: var(--text-main); display: flex; align-items: center; gap: 6px;">
            <span>⚡ 1-Click AI Client Integrations</span>
          </span>
          ${A}
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
            <input type="text" class="form-input" style="font-size: 10.5px; padding: 2px 8px; width: 140px; height: 22px; border-radius: 100px;" placeholder="\uD83D\uDD0D Search..." value="${i(e.clientSearchQuery||"")}" onclick="event.stopPropagation();" oninput="window.app.setClientSearchQuery(this.value)">
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
        <div class="stat-value" style="color: var(--amber-300);">${b}</div>
        <div class="stat-sub">${f}</div>
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
        <div class="stat-value" style="color: var(--green-400);">${n} Active</div>
        <div class="stat-sub">${n>0?"Persistent worker task channels":"No active upstream servers"}</div>
      </div>
    </div>

    ${_}

    <div style="display: flex; justify-content: space-between; align-items: center; margin: 24px 0 12px;">
      <div style="font-size: 15px; font-weight: 700; color: var(--text-main);">Connected Upstream Servers</div>
      <button class="btn btn-ghost" onclick="window.app.switchTab('servers')">Manage All (${n}) →</button>
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
        ${r}
      </div>
    </div>
  `}var D=[{id:"github",name:"GitHub",category:"devtools",description:"Explore repositories, issues, pull requests, branches, and commit histories.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-github"],envFields:[{key:"GITHUB_PERSONAL_ACCESS_TOKEN",label:"GitHub Personal Access Token",placeholder:"ghp_...",required:!0,description:"Classic or fine-grained token with repo scope."}]},{id:"git",name:"Git (Local)",category:"devtools",description:"Read local Git repository status, diffs, log histories, and commit changes.",badge:"Official / uvx",command:"uvx",defaultArgs:["--with","mcp<2","mcp-server-git","--repository","."],argsPlaceholder:"--with mcp<2 mcp-server-git --repository /path/to/repo",envFields:[]},{id:"filesystem",name:"Filesystem",category:"devtools",description:"Secure, sandboxed access to local files and directories for AI workflows.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-filesystem","."],argsPlaceholder:"-y @modelcontextprotocol/server-filesystem /allowed/dir1 /allowed/dir2",envFields:[]},{id:"memory",name:"Memory Graph",category:"devtools",description:"Persistent knowledge-graph based memory for multi-turn agent learning.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-memory"],envFields:[]},{id:"chrome-devtools",name:"Chrome DevTools",category:"devtools",description:"Inspect live DOM, execute scripts, read console logs, and capture network traces in Chrome.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-puppeteer"],envFields:[]},{id:"sentry",name:"Sentry",category:"devtools",description:"Query production error events, stack traces, and issue frequencies directly from Sentry.",badge:"uvx / Telemetry",command:"uvx",defaultArgs:["--with","mcp<2","--with","httpx","mcp-server-sentry","--auth-token","sntrys_token"],argsPlaceholder:"--with mcp<2 --with httpx mcp-server-sentry --auth-token YOUR_SENTRY_TOKEN",envFields:[{key:"SENTRY_AUTH_TOKEN",label:"Sentry Auth Token",placeholder:"sntrys_...",required:!0}]},{id:"playwright",name:"Playwright Browser",category:"browser",description:"Headless / headed browser automation for scraping, form filling, and UI interaction.",badge:"Popular #1 / npx",command:"npx",defaultArgs:["-y","@executeautomation/playwright-mcp-server"],envFields:[]},{id:"puppeteer",name:"Puppeteer",category:"browser",description:"Official browser automation server for web page scraping and screenshot capture.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-puppeteer"],envFields:[]},{id:"brave-search",name:"Brave Search",category:"browser",description:"Real-time privacy-preserving web search and local point-of-interest query engine.",badge:"Official / Search",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-brave-search"],envFields:[{key:"BRAVE_API_KEY",label:"Brave Search API Key",placeholder:"BSA...",required:!0}]},{id:"tavily",name:"Tavily Search",category:"browser",description:"AI-optimized web search engine structured specifically for LLM context injection.",badge:"Community / Stdio",command:"npx",defaultArgs:["-y","tavily-mcp"],envFields:[{key:"TAVILY_API_KEY",label:"Tavily API Key",placeholder:"tvly-...",required:!0}]},{id:"fetch",name:"Fetch / Web Markdown",category:"browser",description:"Download web pages, strip clutter, and convert raw HTML to clean markdown text.",badge:"Official / uvx",command:"uvx",defaultArgs:["mcp-server-fetch"],envFields:[]},{id:"postgres",name:"PostgreSQL",category:"database",description:"Read schemas, inspect tables, and execute SQL queries against PostgreSQL databases.",badge:"Official / Database",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-postgres","postgresql://user:pass@localhost:5432/mydb"],argsPlaceholder:"-y @modelcontextprotocol/server-postgres postgresql://user:pass@localhost:5432/dbname",envFields:[]},{id:"sqlite",name:"SQLite",category:"database",description:"Local embedded SQLite query runner and schema inspector.",badge:"Official / uvx",command:"uvx",defaultArgs:["--with","mcp<2","mcp-server-sqlite","--db-path","./app.db"],argsPlaceholder:"--with mcp<2 mcp-server-sqlite --db-path /path/to/database.sqlite",envFields:[]},{id:"supabase",name:"Supabase",category:"database",description:"Query database tables, manage auth policies, and inspect storage in Supabase.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@supabase/mcp-server-supabase@latest"],envFields:[{key:"SUPABASE_ACCESS_TOKEN",label:"Supabase Personal Access Token",placeholder:"sbp_...",required:!0},{key:"SUPABASE_PROJECT_REF",label:"Supabase Project Reference ID",placeholder:"abcdefghijklmnop",required:!1}]},{id:"redis",name:"Redis",category:"database",description:"Inspect cached keys, hash sets, lists, TTLs, and pub/sub channels in Redis.",badge:"Official / Key-Value",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-redis","redis://localhost:6379"],argsPlaceholder:"-y @modelcontextprotocol/server-redis redis://localhost:6379",envFields:[]},{id:"s3",name:"AWS S3 / Cloud Storage",category:"database",description:"Browse S3 buckets, fetch object metadata, and download files from cloud storage.",badge:"Community / Stdio",command:"npx",defaultArgs:["-y","@geunoh/s3-mcp-server"],argsPlaceholder:"-y @geunoh/s3-mcp-server",envFields:[{key:"AWS_ACCESS_KEY_ID",label:"AWS Access Key ID",placeholder:"AKIA...",required:!0},{key:"AWS_SECRET_ACCESS_KEY",label:"AWS Secret Access Key",placeholder:"...",required:!0},{key:"AWS_REGION",label:"AWS Region",placeholder:"us-east-1",required:!1}]},{id:"linear",name:"Linear",category:"productivity",description:"Search, create, and triage Linear issues, cycles, teams, and project roadmaps.",badge:"Productivity / Stdio",command:"npx",defaultArgs:["-y","mcp-linear"],envFields:[{key:"LINEAR_API_KEY",label:"Linear API Key",placeholder:"lin_api_...",required:!0}]},{id:"slack",name:"Slack",category:"productivity",description:"Read channels, post messages, inspect threads, and search team discussions.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-slack"],envFields:[{key:"SLACK_BOT_TOKEN",label:"Slack Bot User Token",placeholder:"xoxb-...",required:!0},{key:"SLACK_TEAM_ID",label:"Slack Team ID",placeholder:"T01234567",required:!0}]},{id:"notion",name:"Notion",category:"productivity",description:"Search Notion workspace pages, read nested blocks, and query database entries.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@notionhq/notion-mcp-server"],envFields:[{key:"NOTION_TOKEN",label:"Notion Internal Integration Token",placeholder:"secret_...",required:!0}]},{id:"jira",name:"Jira / Atlassian",category:"productivity",description:"Manage Jira issues, search JQL, read sprint statuses, and inspect boards.",badge:"uvx / Atlassian",command:"uvx",defaultArgs:["--with","mcp<2","mcp-server-jira","--jira-base-url","https://your-domain.atlassian.net"],argsPlaceholder:"--with mcp<2 mcp-server-jira --jira-base-url https://org.atlassian.net",envFields:[{key:"JIRA_TOKEN",label:"Atlassian API Token",placeholder:"ATATT3...",required:!0}]},{id:"google-drive",name:"Google Drive",category:"productivity",description:"Search, list, and read documents, spreadsheets, and drive files.",badge:"Community / Stdio",command:"npx",defaultArgs:["-y","@piotr-agier/google-drive-mcp"],envFields:[{key:"GOOGLE_APPLICATION_CREDENTIALS",label:"Google Credentials JSON Path",placeholder:"/path/to/credentials.json",required:!0}]},{id:"docker",name:"Docker",category:"cloud",description:"Inspect running containers, tail container logs, list images, and manage compose services.",badge:"uvx / DevOps",command:"uvx",defaultArgs:["mcp-server-docker"],envFields:[]},{id:"kubernetes",name:"Kubernetes (K8s)",category:"cloud",description:"Query cluster pods, services, deployment status, and inspect Kubernetes logs.",badge:"Popular / Stdio",command:"npx",defaultArgs:["-y","@strowk/mcp-k8s"],envFields:[{key:"KUBECONFIG",label:"Kubeconfig File Path (Optional)",placeholder:"~/.kube/config",required:!1}]},{id:"cloudflare",name:"Cloudflare",category:"cloud",description:"Manage Cloudflare Workers, KV namespaces, D1 databases, Vectorize indexes, and DNS.",badge:"Official / Cloudflare",command:"npx",defaultArgs:["-y","@cloudflare/mcp-server-cloudflare","run","dummy_account_id"],argsPlaceholder:"-y @cloudflare/mcp-server-cloudflare run YOUR_ACCOUNT_ID",envFields:[{key:"CLOUDFLARE_API_TOKEN",label:"Cloudflare API Token",placeholder:"...",required:!0},{key:"CLOUDFLARE_ACCOUNT_ID",label:"Cloudflare Account ID",placeholder:"...",required:!0}]},{id:"terraform",name:"Terraform",category:"cloud",description:"Inspect Terraform state files, resource dependency graphs, and plan previews.",badge:"Community / IaC",command:"npx",defaultArgs:["-y","@mseep/terraform-mcp-server"],envFields:[]}];function N(e,t,a){let n=D.find((r)=>r.id.toLowerCase()===e.toLowerCase());if(n)return n;let o=`${t||""} ${(a||[]).join(" ")}`.toLowerCase();return D.find((r)=>{let s=`${r.command} ${r.defaultArgs.join(" ")}`.toLowerCase();if(o.includes(r.id.toLowerCase()))return!0;if(r.command&&o.includes(r.command.toLowerCase())&&r.defaultArgs.some((l)=>o.includes(l.toLowerCase())))return!0;return!1})}function X(){let e=p.getState(),t=e.config.mcpServers||{},a=Object.keys(t),n=e.activeProfile,o=n?e.config.profiles?.[n]:void 0,r=!!o,s=o?.servers||[],l="";if(a.length===0)l=`
      <div style="padding: 40px; text-align: center; color: var(--text-dim); background: var(--surface-card); border-radius: var(--radius-md); border: 1px dashed var(--border);">
        <div style="font-size: 15px; color: var(--text-main); font-weight: 600; margin-bottom: 8px;">No Servers Configured in ${i(e.configPath)}</div>
        <p style="font-size: 12px; margin-bottom: 20px; max-width: 480px; margin-left: auto; margin-right: auto;">
          Warmplane bridges local tools and remote MCP servers into one unified facade. Add your first server or import existing configs from Claude Desktop or Cursor.
        </p>
        <div style="display: flex; gap: 8px; justify-content: center;">
          <button class="btn btn-primary" onclick="window.app.openTemplateCatalog()">✨ Browse Templates</button>
          <button class="btn btn-ghost" onclick="window.app.openAddServerModal()">+ Add Custom</button>
          <button class="btn btn-ghost" onclick="window.app.openImportModal()">Sync from IDEs</button>
        </div>
      </div>
    `;else l=a.map((g)=>{let u=t[g],m=u.command?"stdio":"http / sse",v=u.command?`${u.command} ${(u.args||[]).join(" ")}`:u.url,c=e.serverStatuses[g]||{status:"connected",protocol_version:"2026-07-28"},y=!r||s.includes(g),b=N(g,u.command,u.args),f=Object.keys(u.env||{}),x=(b?.envFields||[]).filter((_)=>_.required&&!f.includes(_.key)),T=u.env?Object.entries(u.env).map(([_,S])=>{if(S.startsWith("keychain://"))return`<span class="brand-badge" style="color: var(--cyan-400); border-color: rgba(34, 211, 238, 0.3);">\uD83D\uDD12 ${i(_)} (Keychain)</span>`;if(S.startsWith("op://"))return`<span class="brand-badge" style="color: var(--cyan-400); border-color: rgba(34, 211, 238, 0.3);">\uD83D\uDD12 ${i(_)} (1Password)</span>`;if(S.startsWith("env://"))return`<span class="brand-badge" style="color: var(--amber-300); border-color: rgba(251, 191, 36, 0.3);">\uD83D\uDD12 ${i(_)} (Env)</span>`;return`<span style="color: var(--text-dim);">${i(_)}=***</span>`}):[];for(let _ of x)T.push(`<span class="brand-badge" style="color: var(--red-400); border-color: rgba(248, 113, 113, 0.4); background: rgba(248, 113, 113, 0.1);" title="Required environment variable '${i(_.key)}' is missing">⚠️ Missing ${i(_.key)}</span>`);let I=T.length>0?T.join(" "):"None",E=(e.circuitBreakers||[]).find((_)=>_.server_id===g),k='<span class="brand-badge" style="color: var(--green-400); border-color: rgba(52, 211, 153, 0.25);">Circuit: CLOSED</span>';if(E){if(E.state==="open")k=`<span class="brand-badge" style="color: var(--red-400); border-color: rgba(248, 113, 113, 0.4); background: rgba(248, 113, 113, 0.1);">Circuit: OPEN (${E.consecutive_failures} failures)</span>`;else if(E.state==="half_open")k=`<span class="brand-badge" style="color: var(--amber-300); border-color: rgba(251, 191, 36, 0.4); background: rgba(251, 191, 36, 0.1);">Circuit: HALF-OPEN (${E.consecutive_successes} probe)</span>`}let w=u.resilience||e.config.resilience,C=w?`FT: ${w.failureThreshold||3} · Cooldown: ${(w.cooldownMs||30000)/1000}s · AutoRestart: ${w.autoRestart!==!1?"ON":"OFF"}`:"Default Resilience",A=x.length>0,L=c.status==="degraded"||A,M=c.status==="error"||c.status==="disconnected",R=A?"var(--amber-400)":L?"var(--amber-400)":M?"var(--red-400)":"var(--green-400)",B=A?`Status: ${i(c.status)} (Missing Keys)`:`Status: ${i(c.status)}`,O=(L||M)&&(c.error||A)?`
        <div style="background: rgba(239, 68, 68, 0.08); border-left: 3px solid var(--amber-400); border-radius: var(--radius-xs); padding: 8px 12px; margin-top: 8px; display: flex; justify-content: space-between; align-items: center; gap: 8px;">
          <div style="font-size: 11px; color: var(--amber-300); font-family: var(--ff-mono); overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">
            <span style="font-weight: 700; color: var(--amber-400);">⚠️ Diagnostics:</span> ${i(c.error||`Missing required environment variable(s): ${x.map((_)=>_.key).join(", ")}`)}
          </div>
          <button class="btn btn-ghost" style="padding: 2px 8px; font-size: 10.5px; color: var(--amber-300); border-color: rgba(251, 191, 36, 0.3);" onclick="window.app.openServerDiagnosticsModal('${i(g)}')">Details</button>
        </div>
      `:"",J=r?y?`<span class="brand-badge" style="color: var(--green-400); border-color: rgba(34, 197, 94, 0.3); background: rgba(34, 197, 94, 0.08); display: inline-flex; align-items: center; gap: 6px;">
              ✔ IN CONSTELLATION
              <button style="background: none; border: none; color: var(--amber-400); font-size: 10px; cursor: pointer; padding: 0 2px; text-decoration: underline;" onclick="window.app.toggleServerInProfile('${i(n)}', '${i(g)}', false)">Exclude</button>
            </span>`:`<span class="brand-badge" style="color: var(--amber-400); border-color: rgba(245, 158, 11, 0.35); background: rgba(245, 158, 11, 0.08); display: inline-flex; align-items: center; gap: 6px;">
              \uD83D\uDEAB EXCLUDED FROM PROFILE: ${i(n)}
              <button style="background: none; border: none; color: var(--green-400); font-size: 10px; cursor: pointer; padding: 0 2px; text-decoration: underline; font-weight: 700;" onclick="window.app.toggleServerInProfile('${i(n)}', '${i(g)}', true)">+ Include</button>
            </span>`:"";return`
        <div class="bento-card" style="${r&&!y?"margin-bottom: 12px; opacity: 0.65; border: 1px dashed rgba(245, 158, 11, 0.4); background: rgba(0, 0, 0, 0.2);":`margin-bottom: 12px; border-color: ${L?"rgba(251, 191, 36, 0.3)":M?"rgba(248, 113, 113, 0.3)":"var(--border)"};`}">
          <div style="display: flex; justify-content: space-between; align-items: center; gap: 16px; flex-wrap: wrap;">
            <div style="flex: 1; min-width: 260px;">
              <div style="display: flex; align-items: center; gap: 8px; margin-bottom: 4px; flex-wrap: wrap;">
                <span style="width: 8px; height: 8px; border-radius: 50%; background: ${R}; display: inline-block;"></span>
                <span style="font-size: 15px; font-weight: 700; color: var(--text-main);">${i(g)}</span>
                <span class="brand-badge">${m}</span>
                <span class="brand-badge" style="color: ${R}; border-color: ${A?"rgba(245, 158, 11, 0.5); background: rgba(245, 158, 11, 0.1);":"rgba(245, 158, 11, 0.3);"}">${B}</span>
                <span class="brand-badge" style="color: var(--cyan-400); border-color: rgba(34, 211, 238, 0.25);">Protocol: ${c.protocol_version}</span>
                ${k}
                ${J}
              </div>
              <div style="font-family: var(--ff-mono); font-size: 12px; color: var(--text-muted); margin-top: 4px;">
                ${u.command?"Command: ":"URL: "}<code>${i(v||"")}</code>
              </div>
              <div style="display: flex; gap: 14px; font-family: var(--ff-mono); font-size: 11px; color: var(--text-dim); margin-top: 4px; align-items: center; flex-wrap: wrap;">
                <span>\uD83D\uDEE1️ ${i(C)}</span>
                ${u.env&&Object.keys(u.env).length>0?`<span>Env: ${I}</span>`:""}
              </div>
            </div>
            <div style="display: flex; gap: 8px; align-items: center; flex-shrink: 0;">
              <button class="btn btn-ghost" style="padding: 4px 10px; font-size: 11.5px; color: var(--cyan-400); border-color: rgba(34, 211, 238, 0.3);" onclick="window.app.restartServer('${i(g)}')">⚡ Restart</button>
              <button class="btn btn-ghost" style="padding: 4px 10px; font-size: 11.5px;" onclick="window.app.openServerDiagnosticsModal('${i(g)}')">\uD83D\uDD0D Diagnostics</button>
              <button class="btn btn-ghost" style="padding: 4px 10px; font-size: 11.5px;" onclick="window.app.openEditServerModal('${i(g)}')">✏️ Edit</button>
              <button class="btn btn-danger" style="padding: 4px 10px; font-size: 11.5px;" onclick="window.app.deleteServer('${i(g)}')">${r?"Delete Globally":"Remove"}</button>
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
            <span>Active Profile Constellation: <code style="font-size: 13px; color: var(--text-main);">${i(n)}</code></span>
            <span class="brand-badge" style="color: var(--text-main);">${s.length} of ${a.length} servers included</span>
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
        <div style="font-size: 11px; color: var(--text-dim);">Active configuration file: <code>${i(e.configPath)}</code></div>
      </div>
      <div style="display: flex; gap: 8px;">
        <button class="btn btn-ghost" onclick="window.app.reloadFromDisk()">⟳ Reload Config</button>
      </div>
    </div>

    ${d}

    ${l}

    ${le()}
  `}function le(){let e=p.getState(),t=e.clients||[],a=Object.keys(e.config.profiles||{}),n=e.clientsCollapsed,o=e.clientFilterCategory||"all",r=(e.clientSearchQuery||"").toLowerCase().trim();if(t.length===0)return"";let s=t.filter((c)=>c.is_attached).length,l=t.filter((c)=>c.config_exists&&!c.is_attached).length,d=t.filter((c)=>c.category.toLowerCase().includes("ide")||c.category.toLowerCase().includes("extension")).length,g=t.filter((c)=>c.category.toLowerCase().includes("agent")||c.category.toLowerCase().includes("cli")||c.category.toLowerCase().includes("platform")).length,u=t.filter((c)=>{if(r){let y=c.name.toLowerCase().includes(r),b=c.id.toLowerCase().includes(r),f=c.category.toLowerCase().includes(r),x=c.config_path.toLowerCase().includes(r);if(!y&&!b&&!f&&!x)return!1}if(o==="connected")return c.is_attached;if(o==="ready")return c.config_exists||c.app_installed;if(o==="ides")return c.category.toLowerCase().includes("ide")||c.category.toLowerCase().includes("extension");if(o==="agents")return c.category.toLowerCase().includes("agent")||c.category.toLowerCase().includes("cli")||c.category.toLowerCase().includes("platform");return!0}),m=[{id:"all",label:`All Ecosystems (${t.length})`},{id:"ready",label:`Ready / Installed (${l+s})`},{id:"connected",label:`⚡ Connected (${s})`},{id:"ides",label:`IDEs & Editors (${d})`},{id:"agents",label:`Agents & CLIs (${g})`}].map((c)=>`
      <button class="btn btn-ghost" style="padding: 3px 10px; font-size: 11px; border-radius: 100px; ${o===c.id?"background: var(--amber-400); color: #000; font-weight: 700; border-color: var(--amber-400);":"background: var(--surface); color: var(--text-muted); border-color: var(--border);"}" onclick="window.app.setClientCategoryFilter('${i(c.id)}')">
        ${i(c.label)}
      </button>
    `).join(""),v=u.length===0?`<div style="padding: 24px; text-align: center; color: var(--text-dim); font-size: 12px;">No AI clients match the filter "${i(r||o)}".</div>`:u.map((c)=>{let{is_attached:y,config_exists:b,app_installed:f}=c,x='<span class="brand-badge" style="color: var(--text-dim); border-color: rgba(255, 255, 255, 0.1);">Not Found</span>';if(y){let k=c.attached_profile?` · ${c.attached_profile}`:"";x=`<span class="brand-badge" style="color: var(--green-400); border-color: rgba(52, 211, 153, 0.3); background: rgba(52, 211, 153, 0.1);">⚡ Connected · ${c.attached_transport==="http"?"HTTP":"stdio"}${i(k)}</span>`}else if(b)x='<span class="brand-badge" style="color: var(--amber-300); border-color: rgba(251, 191, 36, 0.3); background: rgba(251, 191, 36, 0.08);">○ Ready</span>';else if(f)x='<span class="brand-badge" style="color: var(--cyan-400); border-color: rgba(34, 211, 238, 0.25);">○ Installed</span>';let T=e.activeProfile,I=a.map((k)=>`
          <option value="${i(k)}" ${T===k||c.attached_profile===k?"selected":""}>Profile: ${i(k)}</option>
        `).join(""),E=y?`<button class="btn btn-ghost" style="padding: 3px 10px; font-size: 11px; color: var(--red-400); border-color: rgba(248, 113, 113, 0.3);" onclick="window.app.detachClient('${i(c.id)}')">Disconnect</button>`:`<button class="btn btn-primary" style="padding: 3px 10px; font-size: 11px;" onclick="window.app.attachClient('${i(c.id)}')">⚡ Connect</button>`;return`
          <div style="display: grid; grid-template-columns: 200px 130px 1fr 140px 100px; align-items: center; gap: 12px; padding: 8px 12px; background: var(--surface); border: 1px solid var(--border-subtle); border-radius: var(--radius-sm); transition: background 0.15s;" onmouseover="this.style.background='var(--surface-hover)'" onmouseout="this.style.background='var(--surface)'">
            <div>
              <div style="font-weight: 700; font-size: 13px; color: var(--text-main); display: flex; align-items: center; gap: 6px;">
                <span>${i(c.name)}</span>
              </div>
              <div style="font-size: 10.5px; color: var(--text-dim);">${i(c.category)}</div>
            </div>

            <div>
              ${x}
            </div>

            <div style="font-family: var(--ff-mono); font-size: 10.5px; color: var(--text-muted); overflow: hidden; text-overflow: ellipsis; white-space: nowrap;" title="${i(c.config_path)}">
              ${i(c.config_path)}
            </div>

            <div>
              ${a.length>0&&!y?`
                <select id="client-prof-${i(c.id)}" class="form-input" style="font-size: 10.5px; padding: 2px 6px; height: 26px; width: 100%;">
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
          <span style="font-size: 12px; color: var(--text-dim);">${n?"▼ Show":"▲ Hide"}</span>
        </div>
      </div>

      ${!n?`
        <div style="margin-top: 14px; padding-top: 12px; border-top: 1px solid var(--border-subtle);">
          <div style="display: flex; flex-wrap: wrap; justify-content: space-between; align-items: center; gap: 10px; margin-bottom: 12px;">
            <div style="display: flex; flex-wrap: wrap; gap: 6px;">
              ${m}
            </div>
            <div style="position: relative;">
              <input type="text" class="form-input" style="font-size: 11px; padding: 4px 10px; width: 180px; height: 26px; border-radius: 100px;" placeholder="\uD83D\uDD0D Search clients..." value="${i(e.clientSearchQuery||"")}" oninput="window.app.setClientSearchQuery(this.value)">
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
  `}function Z(){let e=p.getState(),t=e.playgroundMode||"tools",a=e.capabilities||[],n=e.resources||[],o=e.prompts||[],r=e.capabilitiesHiddenByPolicy||0,s=e.resourcesHiddenByPolicy||0,l=e.promptsHiddenByPolicy||0,d=e.activeProfile,u=!!(d?e.config.profiles?.[d]:void 0),m=`
    <div style="display: flex; gap: 8px; margin-bottom: 12px; align-items: center; justify-content: space-between; flex-wrap: wrap;">
      <div style="display: inline-flex; padding: 3px; background: rgba(0,0,0,0.3); border: 1px solid var(--border); border-radius: var(--radius-sm); align-items: center;">
        <button 
          class="btn ${t==="tools"?"btn-primary":"btn-ghost"}" 
          style="padding: 4px 12px; font-size: 11.5px; height: 28px; display: inline-flex; align-items: center; gap: 6px;"
          onclick="window.app.setPlaygroundMode('tools')"
        >
          <span>\uD83D\uDEE0️ Tools (${a.length})</span>
          ${r>0?`<span class="badge" style="background: rgba(245, 158, 11, 0.2); color: var(--amber-300); font-size: 9.5px; padding: 1px 5px;" title="${r} tools hidden by constellation/policy">+${r} hidden</span>`:""}
        </button>
        <button 
          class="btn ${t==="resources"?"btn-primary":"btn-ghost"}" 
          style="padding: 4px 12px; font-size: 11.5px; height: 28px; display: inline-flex; align-items: center; gap: 6px;"
          onclick="window.app.setPlaygroundMode('resources')"
        >
          <span>\uD83D\uDCC4 Resources (${n.length})</span>
          ${s>0?`<span class="badge" style="background: rgba(245, 158, 11, 0.2); color: var(--amber-300); font-size: 9.5px; padding: 1px 5px;" title="${s} resources hidden by constellation/policy">+${s} hidden</span>`:""}
        </button>
        <button 
          class="btn ${t==="prompts"?"btn-primary":"btn-ghost"}" 
          style="padding: 4px 12px; font-size: 11.5px; height: 28px; display: inline-flex; align-items: center; gap: 6px;"
          onclick="window.app.setPlaygroundMode('prompts')"
        >
          <span>\uD83D\uDCAC Prompts (${o.length})</span>
          ${l>0?`<span class="badge" style="background: rgba(245, 158, 11, 0.2); color: var(--amber-300); font-size: 9.5px; padding: 1px 5px;" title="${l} prompts hidden by constellation/policy">+${l} hidden</span>`:""}
        </button>
      </div>

      <div style="display: flex; align-items: center; gap: 12px;">
        ${t==="tools"&&r>0?`
          <div style="font-size: 11px; color: var(--amber-300); background: rgba(245, 158, 11, 0.1); border: 1px solid rgba(245, 158, 11, 0.25); padding: 3px 8px; border-radius: var(--radius-sm); display: flex; align-items: center; gap: 6px;">
            <span>\uD83D\uDEE1️ ${r} tool${r>1?"s":""} filtered ${u?`(Profile: ${i(d)})`:"by policy"}</span>
            <a href="javascript:void(0)" onclick="window.app.switchTab('policy')" style="color: var(--amber-400); text-decoration: underline; font-weight: 600;">View Policy</a>
            ${u?`<a href="javascript:void(0)" onclick="window.app.switchTab('servers')" style="color: var(--cyan-400); text-decoration: underline; font-weight: 600; margin-left: 4px;">Server Hub</a>`:""}
          </div>
        `:t==="resources"&&s>0?`
          <div style="font-size: 11px; color: var(--amber-300); background: rgba(245, 158, 11, 0.1); border: 1px solid rgba(245, 158, 11, 0.25); padding: 3px 8px; border-radius: var(--radius-sm); display: flex; align-items: center; gap: 6px;">
            <span>\uD83D\uDEE1️ ${s} resource${s>1?"s":""} filtered ${u?`(Profile: ${i(d)})`:"by policy"}</span>
            <a href="javascript:void(0)" onclick="window.app.switchTab('policy')" style="color: var(--amber-400); text-decoration: underline; font-weight: 600;">View Policy</a>
            ${u?`<a href="javascript:void(0)" onclick="window.app.switchTab('servers')" style="color: var(--cyan-400); text-decoration: underline; font-weight: 600; margin-left: 4px;">Server Hub</a>`:""}
          </div>
        `:t==="prompts"&&l>0?`
          <div style="font-size: 11px; color: var(--amber-300); background: rgba(245, 158, 11, 0.1); border: 1px solid rgba(245, 158, 11, 0.25); padding: 3px 8px; border-radius: var(--radius-sm); display: flex; align-items: center; gap: 6px;">
            <span>\uD83D\uDEE1️ ${l} prompt${l>1?"s":""} filtered ${u?`(Profile: ${i(d)})`:"by policy"}</span>
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
      ${ce(e)}
    `;if(t==="prompts")return`
      ${m}
      ${pe(e)}
    `;return`
    ${m}
    ${de(e)}
    ${e.isBatchModalOpen?ue(e):""}
  `}function K(e,t=!1){if(!e||!e.properties)return{};let a=e.properties||{},n=Array.isArray(e.required)?e.required:[],o={};for(let[r,s]of Object.entries(a)){let l=n.includes(r);if(t&&!l)continue;if(s.default!==void 0)o[r]=s.default;else if(Array.isArray(s.enum)&&s.enum.length>0)o[r]=s.enum[0];else if(s.examples&&Array.isArray(s.examples)&&s.examples.length>0)o[r]=s.examples[0];else if(s.example!==void 0)o[r]=s.example;else switch(s.type||"string"){case"string":o[r]=l?`sample_${r}`:"";break;case"number":case"integer":o[r]=0;break;case"boolean":o[r]=!0;break;case"array":o[r]=[];break;case"object":o[r]={};break;default:o[r]=`sample_${r}`}}return o}function de(e){let t=e.capabilities||[],a=e.selectedCapabilityId||(t.length>0?t[0].id:null),n=t.find((b)=>b.id===a),o=e.isExecutingCapability,r=e.capabilitiesHiddenByPolicy||0,s=e.activeProfile,l=!!(s&&e.config.profiles?.[s]),d="";if(t.length===0)d=`
      <div style="padding: 24px 16px; text-align: center; color: var(--text-dim); font-size: 11.5px;">
        No tools or capabilities discovered from connected servers.
      </div>
    `;else d=t.map((b)=>`
        <div class="cap-item ${b.id===a?"active":""}" onclick="window.app.selectCapability('${i(b.id)}')">
          <div style="display: flex; justify-content: space-between; align-items: center;">
            <span style="font-weight: 600; color: var(--text-main); font-family: var(--ff-mono); font-size: 12px;">${i(b.id)}</span>
            <span style="font-size: 10px; color: var(--green-400);">${i(b.mode||"read")}</span>
          </div>
          <div style="font-size: 11px; color: var(--text-dim); margin-top: 2px;">server: ${i(b.server||"local")}</div>
        </div>
      `).join("");let g=n?.input_schema,u=g?.properties||{},m=Array.isArray(g?.required)?g.required:[],v=Object.entries(u),c="";if(v.length>0)c=`
      <div style="display: flex; flex-wrap: wrap; gap: 6px; margin-bottom: 8px; align-items: center;">
        <span style="font-size: 10px; font-weight: 700; color: var(--text-dim); text-transform: uppercase;">Fields:</span>
        ${v.map(([b,f])=>{let x=m.includes(b),T=f.type||(f.enum?"enum":"any"),I=x?"rgba(239, 68, 68, 0.15)":"rgba(148, 163, 184, 0.1)",E=x?"var(--red-400)":"var(--text-muted)",k=x?"rgba(239, 68, 68, 0.3)":"var(--border)",w=f.description?` - ${f.description}`:"";return`
            <button 
              type="button" 
              class="btn" 
              style="padding: 2px 7px; font-size: 10.5px; font-family: var(--ff-mono); background: ${I}; color: ${E}; border: 1px solid ${k}; border-radius: var(--radius-sm);" 
              title="Click to insert '${i(b)}' (${i(T)}${i(w)})" 
              onclick="window.app.insertPlaygroundArgKey('${i(b)}', '${i(T)}', ${i(JSON.stringify(f.default??null))})"
            >
              + ${i(b)} <span style="font-size: 9px; opacity: 0.7;">(${T}${x?" *":""})</span>
            </button>
          `}).join("")}
      </div>
    `;let y="{}";if(a&&e.playgroundArgs&&e.playgroundArgs[a]!==void 0)y=e.playgroundArgs[a];else{let b=K(g,!1);y=JSON.stringify(b,null,2)}return`
    <div style="display: grid; grid-template-columns: 320px 1fr; gap: 16px; height: calc(100vh - 165px);">
      <!-- Left Sidebar: Capabilities Catalog -->
      <div style="background: var(--surface-card); border: 1px solid var(--border); border-radius: var(--radius-md); display: flex; flex-direction: column; overflow: hidden;">
        <div style="padding: 12px; border-bottom: 1px solid var(--border);">
          <input type="text" class="form-input" placeholder="Search ${t.length} capabilities..." oninput="window.app.filterCapabilities(this.value)">
        </div>
        <div style="flex: 1; overflow-y: auto; padding: 8px;" id="pg-cap-list">
          ${d}
        </div>
        ${r>0?`
          <div style="padding: 8px 12px; background: rgba(245, 158, 11, 0.08); border-top: 1px solid rgba(245, 158, 11, 0.2); font-size: 11px; color: var(--amber-300); display: flex; justify-content: space-between; align-items: center;">
            <span>\uD83D\uDEE1️ ${r} tool${r>1?"s":""} filtered ${l?`(${i(s)})`:""}</span>
            <div style="display: flex; gap: 6px;">
              <a href="javascript:void(0)" onclick="window.app.switchTab('policy')" style="color: var(--amber-400); text-decoration: underline; font-weight: 600; font-size: 10.5px;">Policy</a>
              ${l?`<a href="javascript:void(0)" onclick="window.app.switchTab('servers')" style="color: var(--cyan-400); text-decoration: underline; font-weight: 600; font-size: 10.5px;">Servers</a>`:""}
            </div>
          </div>
        `:""}
      </div>

      <!-- Right Panel: Capability Execution & Envelope Visualizer -->
      <div style="background: var(--surface-card); border: 1px solid var(--border); border-radius: var(--radius-md); display: flex; flex-direction: column; overflow: hidden;">
        <div style="padding: 14px 18px; border-bottom: 1px solid var(--border); display: flex; justify-content: space-between; align-items: center;">
          <div>
            <div style="font-size: 15px; font-weight: 700; color: var(--text-main); font-family: var(--ff-mono);" id="pg-selected-title">
              ${i(n?n.id:"No Capability Selected")}
            </div>
            <div style="font-size: 11.5px; color: var(--text-dim);" id="pg-selected-desc">
              ${i(n?n.summary||n.description:"Connect servers to inspect and execute tools")}
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
                ${m.length>0?`
                  <button type="button" class="btn btn-ghost" style="padding: 2px 7px; font-size: 10.5px;" title="Fill only required schema fields" onclick="window.app.fillPlaygroundSampleArgs(true)">\uD83E\uDDF9 Required Only</button>
                `:""}
                <button type="button" class="btn btn-ghost" style="padding: 2px 7px; font-size: 10.5px;" title="Format JSON" onclick="window.app.formatPlaygroundArgs()">\uD83D\uDCCB Format</button>
                <button type="button" class="btn btn-ghost" style="padding: 2px 8px; font-size: 11px;" onclick="window.app.openBatchModal()">⚡ Pipeline Builder</button>
              </div>
            </div>

            ${c}

            <textarea class="form-textarea" rows="7" id="pg-args-input" oninput="window.app.updatePlaygroundArgs(this.value)">${i(y)}</textarea>

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
                  <span style="font-size: 10px; color: var(--text-dim); font-family: var(--ff-mono);">${v.length} field${v.length===1?"":"s"} (${m.length} required)</span>
                </div>
                <pre style="background: var(--surface); padding: 10px; border-radius: var(--radius-sm); border: 1px solid var(--border); font-size: 11px; color: var(--text-muted); max-height: 140px; overflow-y: auto;">${i(JSON.stringify(n.input_schema,null,2))}</pre>
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
                      ${i(e.executionResult.data?.task?.status||e.executionResult.data?.status||"TASK_CREATED").toUpperCase()}
                    </span>
                    <span style="font-family: var(--ff-mono); font-size: 12px; font-weight: 700; color: var(--text-main);">${i(e.executionResult.data?.task?.taskId||e.executionResult.data?.taskId||"")}</span>
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

            <pre id="pg-response-json" style="flex: 1; background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 14px; color: var(--amber-300); font-size: 11.5px; overflow-y: auto; margin: 0; white-space: pre-wrap; word-break: break-word;">${e.executionResult?i(JSON.stringify(e.executionResult.data,null,2)):"// Response envelope output will be formatted here"}</pre>
          </div>
        </div>
      </div>
    </div>
  `}function ce(e){let t=e.resources||[],a=e.resourcesHiddenByPolicy||0,n=e.selectedResourceId||(t.length>0?t[0].id:null),o=t.find((l)=>l.id===n),r=e.resourceReadResult,s="";if(t.length===0)s=`
      <div style="padding: 24px 16px; text-align: center; color: var(--text-dim); font-size: 11.5px;">
        No resources exposed by connected MCP servers.
      </div>
    `;else s=t.map((l)=>{let d=l.id===n?"active":"",g=l.uri?l.uri.split(":")[0]:"res";return`
        <div class="cap-item ${d}" onclick="window.app.selectResource('${i(l.id)}')">
          <div style="display: flex; justify-content: space-between; align-items: center;">
            <span style="font-weight: 600; color: var(--text-main); font-family: var(--ff-mono); font-size: 12px;">${i(l.name||l.id)}</span>
            <span class="badge" style="font-size: 9.5px; background: rgba(56, 189, 248, 0.15); color: var(--cyan-400);">${i(g)}</span>
          </div>
          <div style="font-size: 11px; color: var(--text-dim); margin-top: 2px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">${i(l.uri)}</div>
          <div style="display: flex; justify-content: space-between; font-size: 10px; color: var(--text-muted); margin-top: 4px;">
            <span>server: ${i(l.server||"local")}</span>
            <span>${i(l.mime_type||"text/plain")}</span>
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
              ${i(o?o.name||o.id:"No Resource Selected")}
            </div>
            <div style="font-size: 11.5px; color: var(--cyan-400); font-family: var(--ff-mono);">
              ${i(o?o.uri:"Select a resource from the list to read live content")}
            </div>
          </div>
          <button class="btn btn-primary" onclick="window.app.executeReadResource()" ${o?"":"disabled"}>
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M2 12h20M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"></path></svg>
            Read Resource Content
          </button>
        </div>

        <div style="flex: 1; display: grid; grid-template-columns: 1fr 1fr; overflow: hidden;">
          <!-- Request / Distillation Parameters -->
          <div style="padding: 16px; border-right: 1px solid var(--border); overflow-y: auto;">
            ${o?`
              <div style="background: rgba(0,0,0,0.25); padding: 12px; border-radius: var(--radius-sm); border: 1px solid var(--border); margin-bottom: 14px;">
                <div style="font-size: 11px; font-weight: 600; color: var(--text-muted); text-transform: uppercase; margin-bottom: 6px;">Resource Metadata</div>
                <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 8px; font-size: 11.5px;">
                  <div><span style="color: var(--text-muted);">Server:</span> <strong style="color: var(--text-main);">${i(o.server)}</strong></div>
                  <div><span style="color: var(--text-muted);">MIME Type:</span> <strong style="color: var(--text-main);">${i(o.mime_type||"text/plain")}</strong></div>
                </div>
                ${o.description?`
                  <div style="margin-top: 8px; font-size: 11.5px; color: var(--text-dim); border-top: 1px solid rgba(255,255,255,0.05); padding-top: 6px;">
                    ${i(o.description)}
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
            <pre style="flex: 1; background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 14px; color: var(--cyan-400); font-size: 11.5px; overflow-y: auto; margin: 0; white-space: pre-wrap; word-break: break-word;">${r?i(JSON.stringify(r.data,null,2)):'// Click "Read Resource Content" to inspect live payload'}</pre>
          </div>
        </div>
      </div>
    </div>
  `}function pe(e){let t=e.prompts||[],a=e.promptsHiddenByPolicy||0,n=e.selectedPromptId||(t.length>0?t[0].id:null),o=t.find((d)=>d.id===n),r=e.promptGetResult,s="";if(t.length===0)s=`
      <div style="padding: 24px 16px; text-align: center; color: var(--text-dim); font-size: 11.5px;">
        No prompt templates registered by connected MCP servers.
      </div>
    `;else s=t.map((d)=>{let g=d.id===n?"active":"",u=d.arguments?d.arguments.length:0;return`
        <div class="cap-item ${g}" onclick="window.app.selectPrompt('${i(d.id)}')">
          <div style="display: flex; justify-content: space-between; align-items: center;">
            <span style="font-weight: 600; color: var(--text-main); font-family: var(--ff-mono); font-size: 12px;">${i(d.name||d.id)}</span>
            <span class="badge" style="font-size: 9.5px; background: rgba(168, 85, 247, 0.15); color: var(--purple-400);">${u} args</span>
          </div>
          <div style="font-size: 11px; color: var(--text-dim); margin-top: 2px;">${i(d.description||d.title||"Prompt template")}</div>
          <div style="font-size: 10px; color: var(--text-muted); margin-top: 4px;">server: ${i(d.server||"local")}</div>
        </div>
      `}).join("");let l="";if(o&&o.arguments&&o.arguments.length>0)l=o.arguments.map((d)=>`
      <div class="form-group" style="margin-bottom: 12px;">
        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 4px;">
          <label class="form-label" style="margin: 0; font-family: var(--ff-mono);">${i(d.name)}</label>
          ${d.required?'<span class="badge" style="background: rgba(239, 68, 68, 0.15); color: var(--red-400); font-size: 9px;">REQUIRED</span>':'<span style="font-size: 10px; color: var(--text-dim);">optional</span>'}
        </div>
        ${d.description?`<div style="font-size: 11px; color: var(--text-dim); margin-bottom: 4px;">${i(d.description)}</div>`:""}
        <input type="text" class="form-input prompt-arg-input" data-arg-name="${i(d.name)}" placeholder="Enter ${i(d.name)}..." />
      </div>
    `).join("");else if(o)l=`
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
              ${i(o?o.name||o.id:"No Prompt Selected")}
            </div>
            <div style="font-size: 11.5px; color: var(--text-dim);">
              ${i(o?o.description||o.title||"Bind variables and render messages":"Select a prompt from the list to test")}
            </div>
          </div>
          <button class="btn btn-primary" onclick="window.app.executeGetPrompt()" ${o?"":"disabled"}>
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
            <pre style="flex: 1; background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 14px; color: #c084fc; font-size: 11.5px; overflow-y: auto; margin: 0; white-space: pre-wrap; word-break: break-word;">${r?i(JSON.stringify(r.data,null,2)):'// Click "Render Prompt Messages" to view resolved system/user messages'}</pre>
          </div>
        </div>
      </div>
    </div>
  `}function ue(e){let t=e.capabilities||[],a=e.batchSteps||[];return`
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
          ${a.map((o,r)=>{let s=t.find((c)=>c.id===o.capability_id),l=s?.input_schema,d=l?.properties||{},g=Array.isArray(l?.required)?l.required:[],u=Object.entries(d),m=t.map((c)=>`
      <option value="${i(c.id)}" ${c.id===o.capability_id?"selected":""}>
        ${i(c.id)} (${i(c.server||"local")})
      </option>
    `).join(""),v="";if(u.length>0)v=`
        <div style="display: flex; flex-wrap: wrap; gap: 4px; margin-top: 6px; margin-bottom: 6px; align-items: center;">
          <span style="font-size: 9.5px; font-weight: 700; color: var(--text-dim); text-transform: uppercase;">Parameters:</span>
          ${u.map(([c,y])=>{let b=g.includes(c),f=y.type||(y.enum?"enum":"any");return`
              <span style="font-size: 9.5px; font-family: var(--ff-mono); padding: 1px 5px; background: ${b?"rgba(239, 68, 68, 0.15)":"rgba(148, 163, 184, 0.1)"}; color: ${b?"var(--red-400)":"var(--text-muted)"}; border: 1px solid ${b?"rgba(239, 68, 68, 0.3)":"var(--border)"}; border-radius: 3px;" title="${i(y.description||"")}">
                ${i(c)} (${f}${b?" *":""})
              </span>
            `}).join("")}
        </div>
      `;return`
      <div style="background: rgba(0,0,0,0.3); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 14px; margin-bottom: 12px;">
        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 10px;">
          <div style="display: flex; align-items: center; gap: 8px;">
            <span class="badge" style="background: rgba(56, 189, 248, 0.15); color: var(--cyan-400); font-family: var(--ff-mono); font-weight: 700;">STEP ${r+1}</span>
            <span style="font-size: 11px; font-family: var(--ff-mono); color: var(--text-dim);">id: ${i(o.id)}</span>
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
              <input type="checkbox" ${o.continue_on_error?"checked":""} onchange="window.app.updateBatchStepContinueOnError(${r}, this.checked)" />
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
          >${i(o.argsJson)}</textarea>
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
  `}function ee(e){let t=e.tasks||[],a=e.taskFilterStatus||"all",n=t.filter((c)=>c.status==="input_required"),o=t.filter((c)=>c.status==="working"),r=t.filter((c)=>c.status==="completed"),s=t.filter((c)=>c.status==="cancelled"),l=t.filter((c)=>c.status==="failed"),d=a==="all"?t:t.filter((c)=>c.status===a),g=e.config.policy?.require_approval||e.config.policy?.requireApproval||[],u=n.length===0?`
    <div style="padding: 36px 24px; text-align: center; background: var(--surface-card); border-radius: var(--radius-md); border: 1px dashed var(--border);">
      <div style="width: 44px; height: 44px; border-radius: 50%; background: rgba(52, 211, 153, 0.12); border: 1px solid rgba(52, 211, 153, 0.3); display: flex; align-items: center; justify-content: center; margin: 0 auto 12px; color: var(--green-400); font-size: 18px; font-weight: 700;">
        ✓
      </div>
      <div style="font-size: 14.5px; font-weight: 600; color: var(--text-main); margin-bottom: 5px;">No Tasks Awaiting Input or Approval</div>
      <div style="font-size: 11.5px; color: var(--text-dim); max-width: 520px; margin: 0 auto; line-height: 1.6;">
        Tool calls requiring Human-in-the-Loop approval or returning asynchronous <code style="color: var(--amber-300); font-family: var(--ff-mono);">input_required</code> tasks will suspend here for operator inspection, parameter editing, and response submission.
      </div>
    </div>
  `:n.map((c)=>{let y=c.inputRequests||{},b=Object.keys(y),f=b.length>0,x=Math.floor(Date.now()/1000),T=c.expiresAtEpochSecs?Math.max(0,c.expiresAtEpochSecs-x):c.ttlMs?Math.max(0,Math.floor(c.ttlMs/1000)):c.ttlSeconds||300,I=c.createdAtEpochSecs?new Date(c.createdAtEpochSecs*1000).toLocaleTimeString():c.createdAt?new Date(c.createdAt).toLocaleTimeString():"—";return`
      <div class="bento-card" style="border: 1px solid rgba(245, 158, 11, 0.35); background: var(--surface-card); margin-bottom: 14px; padding: 18px;">
        <div style="display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 12px;">
          <div>
            <div style="display: flex; align-items: center; gap: 8px;">
              <span class="brand-badge" style="background: rgba(245, 158, 11, 0.2); color: var(--amber-300); border-color: rgba(245, 158, 11, 0.5);">
                INPUT REQUIRED
              </span>
              <span style="font-family: var(--ff-mono); font-size: 12px; color: var(--text-muted);">${i(c.taskId)}</span>
            </div>
            <div style="margin-top: 6px; display: flex; align-items: center; gap: 8px;">
              <span style="font-size: 14.5px; font-weight: 700; color: var(--text-main); font-family: var(--ff-mono);">
                ${i(c.capabilityId||"Tool Execution")}
              </span>
              ${c.serverId?`<span style="font-size: 11px; color: var(--text-dim);">via <span style="color: var(--cyan-400); font-family: var(--ff-mono);">${i(c.serverId)}</span></span>`:""}
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
            ${c.context.actor_id?`<div><span style="color: var(--text-dim);">Actor:</span> <span style="color: var(--cyan-400);">${i(c.context.actor_id)}</span></div>`:""}
            ${c.context.operation_id?`<div><span style="color: var(--text-dim);">Operation:</span> <span style="color: var(--text-main);">${i(c.context.operation_id)}</span></div>`:""}
            ${c.context.grant_id?`<div><span style="color: var(--text-dim);">Grant:</span> <span style="color: var(--text-main);">${i(c.context.grant_id)}</span></div>`:""}
          </div>
        `:""}

        <!-- Dynamic Input Requests Form -->
        <div style="margin-bottom: 14px;">
          <div style="font-size: 11px; font-weight: 600; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.5px; margin-bottom: 8px;">
            ${f?"Required Input Responses (MRTR / HITL)":"Input Responses Payload (JSON)"}
          </div>

          ${f?`
            <div style="display: flex; flex-direction: column; gap: 10px;">
              ${b.map((E)=>{let k=y[E]||{},w=typeof k==="string"?k:k.prompt||k.description||k.title||E,C=k.type||"text",A=k.default!==void 0?JSON.stringify(k.default):k.value!==void 0?JSON.stringify(k.value):k.sanitized_args?JSON.stringify(k.sanitized_args,null,2):"";if(C==="approval_review")return`
                    <div style="background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 10px 12px;">
                      <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 4px;">
                        <label style="font-size: 11.5px; font-weight: 600; color: var(--amber-300); font-family: var(--ff-mono);">${i(E)}</label>
                        <span class="brand-badge" style="font-size: 9.5px; padding: 1px 5px;">APPROVAL GATED</span>
                      </div>
                      <div style="font-size: 11px; color: var(--text-dim); margin-bottom: 6px;">${i(w)}</div>
                      <div style="margin-bottom: 8px;">
                        <label style="font-size: 10.5px; color: var(--text-dim); display: block; margin-bottom: 2px;">Decision:</label>
                        <select id="task-input-${i(c.taskId)}-${i(E)}-decision" class="form-input" style="font-size: 11.5px; font-family: var(--ff-mono); padding: 4px 8px;">
                          <option value="true" selected>Approve &amp; Execute</option>
                          <option value="false">Reject Execution</option>
                        </select>
                      </div>
                      <div>
                        <label style="font-size: 10.5px; color: var(--text-dim); display: block; margin-bottom: 2px;">Parameters (Editable):</label>
                        <textarea id="task-input-${i(c.taskId)}-${i(E)}" class="form-textarea" rows="3" style="color: var(--green-400); font-family: var(--ff-mono); font-size: 11.5px;">${i(A)}</textarea>
                      </div>
                    </div>
                  `;return`
                  <div style="background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 10px 12px;">
                    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 4px;">
                      <label style="font-size: 11.5px; font-weight: 600; color: var(--amber-300); font-family: var(--ff-mono);">${i(E)}</label>
                      <span class="brand-badge" style="font-size: 9.5px; padding: 1px 5px;">${i(C)}</span>
                    </div>
                    <div style="font-size: 11px; color: var(--text-dim); margin-bottom: 6px;">${i(w)}</div>
                    ${C==="confirmation"||C==="boolean"?`
                      <select id="task-input-${i(c.taskId)}-${i(E)}" class="form-input" style="font-size: 11.5px; font-family: var(--ff-mono); padding: 4px 8px;">
                        <option value="true" selected>true (Approve / Confirm)</option>
                        <option value="false">false (Reject / Deny)</option>
                      </select>
                    `:k.sanitized_args||C==="object"||C==="json"?`
                      <textarea id="task-input-${i(c.taskId)}-${i(E)}" class="form-textarea" rows="3" style="color: var(--green-400); font-family: var(--ff-mono); font-size: 11.5px;">${i(A)}</textarea>
                    `:`
                      <input id="task-input-${i(c.taskId)}-${i(E)}" type="text" class="form-input" value="${i(A)}" placeholder="Enter ${i(E)} response..." style="font-size: 11.5px; font-family: var(--ff-mono);">
                    `}
                  </div>
                `}).join("")}
            </div>
          `:`
            <textarea id="task-raw-input-${i(c.taskId)}" class="form-textarea" rows="3" style="color: var(--green-400); font-family: var(--ff-mono); font-size: 11.5px;">{}</textarea>
          `}
        </div>

        <!-- Action Footer -->
        <div style="display: flex; justify-content: space-between; align-items: center; border-top: 1px solid var(--border); padding-top: 12px;">
          <div style="display: flex; align-items: center; gap: 8px;">
            <input id="task-operator-${i(c.taskId)}" type="text" class="form-input" placeholder="Operator ID" value="security-operator" style="width: 180px; padding: 5px 10px; font-size: 11px;">
          </div>

          <div style="display: flex; align-items: center; gap: 8px;">
            <button class="btn btn-danger" onclick="window.app.promptCancelTask('${i(c.taskId)}')">
              ✕ Cancel Task
            </button>
            <button class="btn btn-primary" onclick="window.app.submitTaskInputResponses('${i(c.taskId)}')">
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
      <span style="font-family: var(--ff-mono); font-size: 11.5px; color: var(--amber-300); font-weight: 500;">\uD83D\uDEE1️ ${i(c)}</span>
      <span class="brand-badge" style="font-size: 9.5px; padding: 1px 5px;">GATED</span>
    </div>
  `).join(""),v=d.length===0?`
    <tr>
      <td colspan="6" style="padding: 24px; text-align: center; color: var(--text-dim); font-size: 12px;">
        No tasks found matching filter "${i(a)}".
      </td>
    </tr>
  `:d.map((c)=>{let y=c.status==="completed"?"background: rgba(52, 211, 153, 0.12); color: var(--green-400); border-color: rgba(52, 211, 153, 0.3);":c.status==="working"?"background: rgba(56, 189, 248, 0.15); color: var(--cyan-400); border-color: rgba(56, 189, 248, 0.4);":c.status==="input_required"?"background: rgba(245, 158, 11, 0.2); color: var(--amber-300); border-color: rgba(245, 158, 11, 0.5);":c.status==="cancelled"?"background: rgba(148, 163, 184, 0.15); color: var(--text-muted); border-color: rgba(148, 163, 184, 0.3);":"background: rgba(248, 113, 113, 0.12); color: var(--red-400); border-color: rgba(248, 113, 113, 0.3);",b=c.progress!==void 0?Math.round(c.progress*100):c.status==="completed"?100:c.status==="working"?50:0,f=c.createdAtEpochSecs?new Date(c.createdAtEpochSecs*1000).toLocaleTimeString():c.createdAt?new Date(c.createdAt).toLocaleTimeString():"—";return`
      <tr style="border-bottom: 1px solid rgba(255,255,255,0.03); transition: background 0.15s;" onmouseover="this.style.background='var(--surface-hover)'" onmouseout="this.style.background='transparent'">
        <td style="padding: 10px 14px;">
          <span class="brand-badge" style="${y}">
            ${c.status.toUpperCase()}
          </span>
        </td>
        <td style="padding: 10px 14px; font-family: var(--ff-mono); font-weight: 600; color: var(--text-main); font-size: 11.5px;">
          ${i(c.capabilityId||"Tool Execution")}
        </td>
        <td style="padding: 10px 14px; font-family: var(--ff-mono); color: var(--text-dim); font-size: 11px;">
          ${i(c.taskId)}
        </td>
        <td style="padding: 10px 14px; width: 140px;">
          <div style="display: flex; align-items: center; gap: 8px;">
            <div style="flex: 1; height: 6px; background: var(--surface-card); border-radius: 3px; overflow: hidden; border: 1px solid var(--border);">
              <div style="height: 100%; width: ${b}%; background: ${c.status==="completed"?"var(--green-400)":"var(--amber-400)"}; transition: width 0.3s;"></div>
            </div>
            <span style="font-size: 10.5px; font-family: var(--ff-mono); color: var(--text-muted);">${b}%</span>
          </div>
        </td>
        <td style="padding: 10px 14px; color: var(--text-dim); font-size: 11px; text-align: right;">
          ${f}
        </td>
        <td style="padding: 10px 14px; text-align: right;">
          <div style="display: inline-flex; gap: 6px; align-items: center;">
            <button class="btn btn-ghost" style="padding: 2px 8px; font-size: 10.5px;" onclick="window.app.openTaskInspectorModal('${i(c.taskId)}')">\uD83D\uDD0D Inspect</button>
            ${c.status==="input_required"||c.status==="working"?`
              <button class="btn btn-danger" style="padding: 2px 8px; font-size: 10.5px;" onclick="window.app.promptCancelTask('${i(c.taskId)}')">Cancel</button>
            `:""}
          </div>
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
        <div class="stat-value" style="color: var(--cyan-400);">${o.length}</div>
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
            ${v}
          </tbody>
        </table>
      </div>
    </div>
  `}function te(){let e=p.getState(),t=e.auditEvents||[],a=e.auditStats||{total_events:0,by_status:{success:0,failed:0,denied:0,intercepted:0}},n=e.auditVerification,o=e.auditFilters,r=e.auditTotal??t.length,s=e.auditSelectedEvent,l=Object.keys(e.config?.mcpServers||{}),d=o.limit||25,g=o.offset||0,u=Math.floor(g/d)+1,m=Math.max(1,Math.ceil(r/d)),v=r===0?0:g+1,c=Math.min(g+d,r),y=h.getAuditExportUrl({actor_id:o.search?void 0:void 0,server_id:o.serverId!=="all"?o.serverId:void 0,event_type:o.eventType!=="all"?o.eventType:void 0,status:o.status!=="all"?o.status:void 0,search:o.search.trim()?o.search.trim():void 0},"csv"),b=h.getAuditExportUrl({server_id:o.serverId!=="all"?o.serverId:void 0,event_type:o.eventType!=="all"?o.eventType:void 0,status:o.status!=="all"?o.status:void 0,search:o.search.trim()?o.search.trim():void 0},"jsonl"),f=n?n.is_valid?`
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
  `,x=l.map((w)=>`<option value="${i(w)}" ${o.serverId===w?"selected":""}>${i(w)}</option>`).join(""),T=`
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
            value="${i(o.search)}"
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
    `;else E=t.map((w)=>{let C=new Date(Math.floor(w.timestamp_ns/1e6)).toLocaleString(),A='<span class="badge" style="background: rgba(34, 197, 94, 0.15); color: var(--green-400); font-weight: 600;">SUCCESS</span>';if(w.status==="denied")A='<span class="badge" style="background: rgba(239, 68, 68, 0.15); color: var(--red-400); font-weight: 600;">DENIED</span>';else if(w.status==="intercepted")A='<span class="badge" style="background: rgba(234, 179, 8, 0.15); color: var(--amber-300); font-weight: 600;">HITL INTERCEPT</span>';else if(w.status==="failed")A='<span class="badge" style="background: rgba(239, 68, 68, 0.15); color: var(--red-400); font-weight: 600;">FAILED</span>';else if(w.status==="cancelled")A='<span class="badge" style="background: rgba(148, 163, 184, 0.15); color: var(--text-muted); font-weight: 600;">CANCELLED</span>';let L=w.sanitized_args?JSON.stringify(w.sanitized_args):"-",M=w.actor_id||w.operator_id||"anonymous",R=w.server_id||"system",B=w.capability_id||w.event_type,O=w.execution_latency_us?`${(w.execution_latency_us/1000).toFixed(1)}ms`:"-";return`
        <div class="bento-card" style="margin-bottom: 12px; padding: 16px; border: 1px solid var(--border); transition: border-color 0.15s ease;">
          <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
            <div style="display: flex; align-items: center; gap: 8px;">
              <span style="font-family: var(--ff-mono); font-size: 11px; font-weight: 700; color: var(--text-dim);">${i(w.id)}</span>
              ${A}
              <span style="font-size: 12px; font-weight: 600; color: var(--text-main);">${i(B)}</span>
            </div>
            <div style="display: flex; align-items: center; gap: 10px;">
              <div style="font-family: var(--ff-mono); font-size: 11px; color: var(--text-muted);">${i(C)}</div>
              <button 
                class="btn btn-ghost" 
                style="padding: 2px 8px; font-size: 11px; height: 24px;" 
                onclick="window.app.selectAuditEvent('${i(w.id)}')"
                title="Inspect event details & cryptographic payload"
              >
                Inspect \uD83D\uDD0D
              </button>
            </div>
          </div>
          
          <div style="display: grid; grid-template-columns: repeat(4, 1fr); gap: 8px; font-size: 11.5px; background: rgba(0,0,0,0.2); padding: 8px 12px; border-radius: var(--radius-sm); margin-bottom: 8px;">
            <div><span style="color: var(--text-muted);">Actor:</span> <strong style="color: var(--text-main);">${i(M)}</strong></div>
            <div><span style="color: var(--text-muted);">Server:</span> <strong style="color: var(--cyan-400);">${i(R)}</strong></div>
            <div><span style="color: var(--text-muted);">Trace:</span> <code style="color: var(--cyan-400); font-size: 10.5px;">${i(w.trace_id)}</code></div>
            <div><span style="color: var(--text-muted);">Latency:</span> <span style="color: var(--amber-300);">${O}</span></div>
          </div>

          <div style="font-family: var(--ff-mono); font-size: 11px; color: var(--text-dim); margin-bottom: 8px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">
            <span style="color: var(--text-muted);">Args:</span> ${i(L)}
          </div>

          <div style="display: flex; justify-content: space-between; align-items: center; border-top: 1px solid var(--border); padding-top: 6px; font-size: 10.5px; font-family: var(--ff-mono); color: var(--text-muted);">
            <div><span style="color: var(--text-dim);">prev_hash:</span> ${i(w.prev_hash.slice(0,16))}...</div>
            <div><span style="color: var(--text-dim);">hash:</span> <span style="color: var(--green-400);">${i(w.hash.slice(0,16))}...</span></div>
          </div>
        </div>
      `}).join("");let k="";if(s){let w=new Date(Math.floor(s.timestamp_ns/1e6)).toISOString();k=`
      <div style="position: fixed; inset: 0; background: rgba(0,0,0,0.7); backdrop-filter: blur(4px); z-index: 9999; display: flex; align-items: center; justify-content: center; padding: 24px;" onclick="if (event.target === this) window.app.selectAuditEvent(null)">
        <div class="bento-card" style="width: 100%; max-width: 720px; max-height: 85vh; display: flex; flex-direction: column; overflow: hidden; background: #0f172a; border: 1px solid var(--border); box-shadow: 0 20px 40px rgba(0,0,0,0.5);">
          <!-- Modal Header -->
          <div style="display: flex; justify-content: space-between; align-items: center; padding: 16px 20px; border-bottom: 1px solid var(--border);">
            <div style="display: flex; align-items: center; gap: 8px;">
              <span style="font-size: 16px;">\uD83D\uDD12</span>
              <h2 style="font-size: 15px; font-weight: 700; color: var(--text-main); margin: 0;">Audit Event Details (${i(s.id)})</h2>
            </div>
            <button class="btn btn-ghost" style="padding: 4px 8px; font-size: 14px;" onclick="window.app.selectAuditEvent(null)">✕</button>
          </div>

          <!-- Modal Body -->
          <div style="padding: 20px; overflow-y: auto; display: flex; flex-direction: column; gap: 14px; font-size: 12px;">
            <div style="display: grid; grid-template-columns: repeat(2, 1fr); gap: 10px; background: rgba(0,0,0,0.25); padding: 12px; border-radius: var(--radius-sm);">
              <div><span style="color: var(--text-muted);">Timestamp:</span> <strong style="color: var(--text-main); font-family: var(--ff-mono); font-size: 11px;">${i(w)}</strong></div>
              <div><span style="color: var(--text-muted);">Status:</span> <strong style="color: var(--text-main);">${i(s.status.toUpperCase())}</strong></div>
              <div><span style="color: var(--text-muted);">Event Type:</span> <strong style="color: var(--text-main);">${i(s.event_type)}</strong></div>
              <div><span style="color: var(--text-muted);">Server:</span> <strong style="color: var(--cyan-400);">${i(s.server_id||"system")}</strong></div>
              <div><span style="color: var(--text-muted);">Capability:</span> <strong style="color: var(--text-main);">${i(s.capability_id||"-")}</strong></div>
              <div><span style="color: var(--text-muted);">Actor / Operator:</span> <strong style="color: var(--text-main);">${i(s.actor_id||s.operator_id||"anonymous")}</strong></div>
              <div><span style="color: var(--text-muted);">Trace ID:</span> <code style="color: var(--cyan-400);">${i(s.trace_id)}</code></div>
              <div><span style="color: var(--text-muted);">Request ID:</span> <code style="color: var(--cyan-400);">${i(s.request_id||"-")}</code></div>
              <div><span style="color: var(--text-muted);">Client IP:</span> <span style="color: var(--text-main);">${i(s.client_ip||"-")}</span></div>
              <div><span style="color: var(--text-muted);">Latency:</span> <span style="color: var(--amber-300);">${s.execution_latency_us?`${(s.execution_latency_us/1000).toFixed(2)} ms`:"-"}</span></div>
            </div>

            ${s.error_message?`
              <div style="background: rgba(239, 68, 68, 0.1); border: 1px solid rgba(239, 68, 68, 0.3); border-radius: var(--radius-sm); padding: 10px 12px; color: var(--red-400);">
                <div style="font-weight: 700; margin-bottom: 2px;">Error (${i(s.error_code||"ERROR")}):</div>
                <div style="font-family: var(--ff-mono); font-size: 11px;">${i(s.error_message)}</div>
              </div>
            `:""}

            <!-- Sanitized Arguments -->
            <div>
              <div style="font-weight: 600; color: var(--text-main); margin-bottom: 4px;">Sanitized Arguments</div>
              <pre style="background: rgba(0,0,0,0.4); padding: 10px; border-radius: var(--radius-sm); border: 1px solid var(--border); font-family: var(--ff-mono); font-size: 11px; max-height: 140px; overflow: auto; margin: 0; color: #cbd5e1;">${i(JSON.stringify(s.sanitized_args||{},null,2))}</pre>
            </div>

            <!-- Sanitized Response -->
            ${s.sanitized_response?`
              <div>
                <div style="font-weight: 600; color: var(--text-main); margin-bottom: 4px;">Sanitized Response</div>
                <pre style="background: rgba(0,0,0,0.4); padding: 10px; border-radius: var(--radius-sm); border: 1px solid var(--border); font-family: var(--ff-mono); font-size: 11px; max-height: 140px; overflow: auto; margin: 0; color: #cbd5e1;">${i(JSON.stringify(s.sanitized_response,null,2))}</pre>
              </div>
            `:""}

            <!-- Cryptographic Hashes -->
            <div style="background: rgba(0,0,0,0.3); padding: 12px; border-radius: var(--radius-sm); border: 1px solid var(--border);">
              <div style="font-weight: 600; color: var(--text-main); margin-bottom: 6px;">Tamper-Evidence Cryptographic Hashes</div>
              <div style="margin-bottom: 6px;">
                <span style="color: var(--text-muted); font-size: 10.5px;">Previous Chain Hash (prev_hash):</span>
                <div style="font-family: var(--ff-mono); font-size: 10.5px; color: var(--text-dim); word-break: break-all;">${i(s.prev_hash)}</div>
              </div>
              <div>
                <span style="color: var(--text-muted); font-size: 10.5px;">Record Hash Signature (hash):</span>
                <div style="font-family: var(--ff-mono); font-size: 10.5px; color: var(--green-400); word-break: break-all;">${i(s.hash)}</div>
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
        ${f}
        <a href="${y}" download class="btn btn-ghost" style="font-size: 11.5px; text-decoration: none;" title="Export current filtered view as CSV">\uD83D\uDCE5 Export CSV</a>
        <a href="${b}" download class="btn btn-ghost" style="font-size: 11.5px; text-decoration: none;" title="Export current filtered view as JSONL">\uD83D\uDCE5 Export JSONL</a>
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
    ${k}
  `}function re(){let e=p.getState(),t=e.activeProfile,a=t?e.config.profiles?.[t]:void 0,n=!!a,o=e.config.policy||{},r=a?.policy,s=n?r||{}:o,l=s.allow||[],d=s.deny||[],g=s.redact_keys||s.redactKeys||[],u=s.require_approval||s.requireApproval||[],m=l.length===0?`
    <div style="color: var(--text-dim); font-size: 12px;">${n?"No profile allow list (inherits global rules)":"No allow list (all non-denied operations permitted)"}</div>
  `:l.map((k,w)=>`
    <div style="display: flex; justify-content: space-between; align-items: center; background: var(--surface); padding: 8px 12px; border-radius: var(--radius-sm); border: 1px solid var(--border);">
      <span style="font-family: var(--ff-mono); font-size: 12px; color: var(--green-400);">✔ ${i(k)}</span>
      <button class="btn btn-ghost" style="padding: 2px 6px; font-size: 11px; color: var(--red-400);" onclick="window.app.removePolicyRule('allow', ${w})">✕</button>
    </div>
  `).join(""),v=d.length===0?`
    <div style="color: var(--text-dim); font-size: 12px;">${n?"No profile deny rules configured":"No deny rules configured"}</div>
  `:d.map((k,w)=>`
    <div style="display: flex; justify-content: space-between; align-items: center; background: var(--surface); padding: 8px 12px; border-radius: var(--radius-sm); border: 1px solid var(--border);">
      <span style="font-family: var(--ff-mono); font-size: 12px; color: var(--red-400);">✖ ${i(k)}</span>
      <button class="btn btn-ghost" style="padding: 2px 6px; font-size: 11px; color: var(--red-400);" onclick="window.app.removePolicyRule('deny', ${w})">✕</button>
    </div>
  `).join(""),c=u.length===0?`
    <div style="color: var(--text-dim); font-size: 12px;">${n?"No profile human-in-the-loop triggers configured":"No human-in-the-loop approval rules configured"}</div>
  `:u.map((k,w)=>`
    <div style="display: flex; justify-content: space-between; align-items: center; background: var(--surface); padding: 8px 12px; border-radius: var(--radius-sm); border: 1px solid var(--border);">
      <span style="font-family: var(--ff-mono); font-size: 12px; color: var(--amber-400);">\uD83D\uDEE1️ ${i(k)}</span>
      <button class="btn btn-ghost" style="padding: 2px 6px; font-size: 11px; color: var(--red-400);" onclick="window.app.removePolicyRule('requireApproval', ${w})">✕</button>
    </div>
  `).join(""),y=g.length===0?`
    <div style="color: var(--text-dim); font-size: 12px;">${n?"No profile key redaction patterns configured":"No key redaction patterns configured"}</div>
  `:g.map((k,w)=>`
    <span class="brand-badge" style="color: var(--amber-300); padding: 5px 10px; font-size: 11px; display: inline-flex; align-items: center; gap: 6px;">
      ${i(k)}
      <span style="cursor: pointer; color: var(--red-400); font-weight: bold;" onclick="window.app.removePolicyRule('redact', ${w})">✕</span>
    </span>
  `).join(""),b=n?`
    <div class="bento-card" style="margin-bottom: 16px; background: rgba(245, 158, 11, 0.05); border: 1px solid rgba(245, 158, 11, 0.3); display: flex; justify-content: space-between; align-items: center;">
      <div style="display: flex; align-items: center; gap: 10px;">
        <span style="font-size: 18px;">\uD83D\uDEE1️</span>
        <div>
          <div style="font-size: 13px; font-weight: 700; color: var(--amber-400);">
            Viewing &amp; Editing Policy for Profile Constellation: <code style="font-size: 13px; color: var(--text-main);">${i(t)}</code>
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
  `,f=Object.keys(e.config.mcpServers||{}),x=a?.servers||[],T=n?f.filter((k)=>!x.includes(k)):[],I=n?`
    <div class="bento-card" style="margin-bottom: 16px; border: 1px solid rgba(245, 158, 11, 0.2); background: rgba(0, 0, 0, 0.2);">
      <div class="stat-header" style="display: flex; justify-content: space-between; align-items: center;">
        <span class="stat-label" style="color: var(--amber-400);">Constellation Server Boundaries (Profile: ${i(t)})</span>
        <span style="font-size: 11px; color: var(--text-dim);">${x.length} of ${f.length} servers active</span>
      </div>
      <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 14px; margin-top: 10px;">
        <div style="background: var(--surface); padding: 10px; border-radius: var(--radius-sm); border: 1px solid var(--border);">
          <div style="font-size: 11px; font-weight: 600; color: var(--green-400); text-transform: uppercase; margin-bottom: 6px;">
            ✔ Included Servers (${x.length})
          </div>
          <div style="display: flex; flex-wrap: wrap; gap: 6px;">
            ${x.length>0?x.map((k)=>`
              <span class="brand-badge" style="color: var(--cyan-400); border-color: rgba(34, 211, 238, 0.25); background: rgba(34, 211, 238, 0.05);">
                ${i(k)}
              </span>
            `).join(""):'<span style="font-size: 11px; color: var(--text-dim);">No servers included</span>'}
          </div>
        </div>

        <div style="background: var(--surface); padding: 10px; border-radius: var(--radius-sm); border: 1px solid var(--border);">
          <div style="font-size: 11px; font-weight: 600; color: var(--amber-400); text-transform: uppercase; margin-bottom: 6px;">
            \uD83D\uDEAB Excluded Servers (${T.length}) · Implicitly Denied
          </div>
          <div style="display: flex; flex-wrap: wrap; gap: 6px; align-items: center;">
            ${T.length>0?T.map((k)=>`
              <span class="brand-badge" style="color: var(--text-muted); border-color: rgba(245, 158, 11, 0.2); background: rgba(245, 158, 11, 0.04); display: inline-flex; align-items: center; gap: 4px;">
                ${i(k)}
                <button style="background: none; border: none; color: var(--amber-400); font-size: 10px; cursor: pointer; padding: 0 2px;" title="Include in profile" onclick="window.app.toggleServerInProfile('${i(t)}', '${i(k)}', true)">+</button>
              </span>
            `).join(""):'<span style="font-size: 11px; color: var(--text-dim);">All servers included in constellation</span>'}
          </div>
        </div>
      </div>
    </div>
  `:"",E=n&&T.length>0?`
    <div style="border-top: 1px dashed var(--border); padding-top: 8px; margin-top: 8px;">
      <div style="font-size: 10.5px; color: var(--text-dim); text-transform: uppercase; font-weight: 600; margin-bottom: 6px;">
        Implicit Boundary Denials (${T.length})
      </div>
      <div style="display: flex; flex-direction: column; gap: 4px;">
        ${T.map((k)=>`
          <div style="display: flex; justify-content: space-between; align-items: center; background: rgba(245, 158, 11, 0.03); padding: 5px 8px; border-radius: var(--radius-xs); border: 1px dashed rgba(245, 158, 11, 0.2);">
            <span style="font-family: var(--ff-mono); font-size: 11px; color: var(--text-dim);">✖ ${i(k)}.*</span>
            <span style="font-size: 9.5px; color: var(--amber-400); font-family: var(--ff-mono);">server excluded</span>
          </div>
        `).join("")}
      </div>
    </div>
  `:"";return`
    ${b}

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
            <input type="text" class="form-input" id="policy-webhook-url" placeholder="https://hooks.slack.com/services/... or Discord webhook URL" value="${i(typeof s.webhook==="object"&&s.webhook?s.webhook.url||"":"")}">
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
            <input type="text" class="form-input" id="policy-webhook-secret" placeholder="e.g. WARMPLANE_WEBHOOK_SECRET" value="${i(typeof s.webhook==="object"&&s.webhook?s.webhook.secret_env||s.webhook.secretEnv||s.webhook.secret||"":"")}">
          </div>
        </div>

        <div style="display: flex; justify-content: space-between; align-items: center; margin-top: 14px; padding-top: 12px; border-top: 1px solid var(--border-subtle);">
          <div style="display: flex; gap: 8px; align-items: center;">
            <button class="btn btn-primary" onclick="window.app.saveWebhookConfig()">Save Webhook Settings</button>
            <button class="btn btn-ghost" onclick="window.app.testWebhook()">⚡ Send Test Event</button>
          </div>
          <div id="policy-webhook-status" style="font-size: 11px; font-family: var(--ff-mono); color: var(--text-dim);">
            ${typeof s.webhook==="object"&&s.webhook?.url?`Active Target: ${i(s.webhook.url)}`:"No webhook configured"}
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
  `}function se(){let e=p.getState(),t=e.config,a=Object.entries(t.capabilityAliases||{}),n=Object.entries(t.resourceAliases||{}),o=Object.entries(t.promptAliases||{}),r="";if(a.length===0&&n.length===0&&o.length===0)r=`
      <div style="padding: 24px; text-align: center; color: var(--text-dim);">
        No facade aliases configured in ${i(e.configPath)}. Add short names or custom descriptions to prune token payload sizes.
      </div>
    `;else{for(let[s,l]of a){let d=typeof l==="string"?l:l.target,g=typeof l==="object"&&l.summary?l.summary:"",u=typeof l==="object"&&!!l.passthrough,m=u?'<span class="brand-badge" style="color: var(--amber-400); border-color: rgba(245, 158, 11, 0.4); background: rgba(245, 158, 11, 0.12); margin-left: 8px; font-size: 10px; padding: 1px 7px; flex-shrink: 0; white-space: nowrap; display: inline-flex; align-items: center; gap: 3px;">⚡ passthrough</span>':"",v=g?`<div style="font-size: 11px; color: var(--text-dim); margin-top: 2px;">\uD83D\uDCAC ${i(g)}</div>`:"";r+=`
        <div class="feed-row" style="grid-template-columns: 80px 240px 1fr 70px; cursor: pointer; transition: background 0.15s;" onclick="window.app.startEditAlias('tool', '${i(s)}', '${i(d)}', '${i(g)}', ${u})" title="Click to edit alias">
          <span style="color: var(--cyan-400);">Tool</span>
          <div style="display: flex; align-items: center; min-width: 0;">
            <span style="font-weight: 700; color: var(--text-main); font-family: var(--ff-mono); overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">${i(s)}</span>
            ${m}
          </div>
          <div>
            <span style="color: var(--text-muted); font-family: var(--ff-mono); word-break: break-all;">${i(d)}</span>
            ${v}
          </div>
          <div style="text-align: right;" onclick="event.stopPropagation()">
            <button class="btn btn-ghost" style="padding: 2px 6px; color: var(--red-400);" onclick="window.app.deleteAlias('tool', '${i(s)}')">✕</button>
          </div>
        </div>
      `}for(let[s,l]of n){let d=typeof l==="string"?l:l.target,g=typeof l==="object"&&l.summary?l.summary:"",u=g?`<div style="font-size: 11px; color: var(--text-dim); margin-top: 2px;">\uD83D\uDCAC ${i(g)}</div>`:"";r+=`
        <div class="feed-row" style="grid-template-columns: 80px 240px 1fr 70px; cursor: pointer; transition: background 0.15s;" onclick="window.app.startEditAlias('resource', '${i(s)}', '${i(d)}', '${i(g)}', false)" title="Click to edit alias">
          <span style="color: var(--green-400);">Resource</span>
          <span style="font-weight: 700; color: var(--text-main); font-family: var(--ff-mono); overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">${i(s)}</span>
          <div>
            <span style="color: var(--text-muted); font-family: var(--ff-mono); word-break: break-all;">${i(d)}</span>
            ${u}
          </div>
          <div style="text-align: right;" onclick="event.stopPropagation()">
            <button class="btn btn-ghost" style="padding: 2px 6px; color: var(--red-400);" onclick="window.app.deleteAlias('resource', '${i(s)}')">✕</button>
          </div>
        </div>
      `}for(let[s,l]of o){let d=typeof l==="string"?l:l.target,g=typeof l==="object"&&l.summary?l.summary:"",u=g?`<div style="font-size: 11px; color: var(--text-dim); margin-top: 2px;">\uD83D\uDCAC ${i(g)}</div>`:"";r+=`
        <div class="feed-row" style="grid-template-columns: 80px 240px 1fr 70px; cursor: pointer; transition: background 0.15s;" onclick="window.app.startEditAlias('prompt', '${i(s)}', '${i(d)}', '${i(g)}', false)" title="Click to edit alias">
          <span style="color: var(--amber-300);">Prompt</span>
          <span style="font-weight: 700; color: var(--text-main); font-family: var(--ff-mono); overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">${i(s)}</span>
          <div>
            <span style="color: var(--text-muted); font-family: var(--ff-mono); word-break: break-all;">${i(d)}</span>
            ${u}
          </div>
          <div style="text-align: right;" onclick="event.stopPropagation()">
            <button class="btn btn-ghost" style="padding: 2px 6px; color: var(--red-400);" onclick="window.app.deleteAlias('prompt', '${i(s)}')">✕</button>
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
  `}function ae(){let e=p.getState(),t=e.config,a=t.profiles||{},n=Object.entries(a),o=t.mcpServers||{},r=e.activeProfile,s="";if(n.length===0)s=`
      <div style="padding: 40px; text-align: center; color: var(--text-dim); background: var(--surface-card); border-radius: var(--radius-md); border: 1px dashed var(--border);">
        <div style="font-size: 15px; color: var(--text-main); font-weight: 600; margin-bottom: 8px;">No Profiles Configured</div>
        <p style="font-size: 12px; margin-bottom: 20px; max-width: 480px; margin-left: auto; margin-right: auto;">
          Profiles allow Warmplane to serve multiple task-relevant server constellations (e.g. <code>coding</code>, <code>support</code>, <code>data</code>) from one running daemon process.
        </p>
        <button class="btn btn-primary" onclick="window.app.openAddProfileModal()">+ Create First Profile</button>
      </div>
    `;else s=n.map(([l,d])=>{let g=r===l,u=d.servers.map((x)=>`<span class="brand-badge" style="${o[x]?"color: var(--cyan-400); border-color: rgba(34, 211, 238, 0.25); background: rgba(34, 211, 238, 0.05);":"color: var(--red-400); border-color: rgba(248, 113, 113, 0.3); background: rgba(248, 113, 113, 0.05);"}">${i(x)}</span>`).join(" "),m=(e.capabilities||[]).filter((x)=>d.servers.includes(x.server)).length,v=!!d.policy,c=d.policy?.allow?.length||0,y=d.policy?.deny?.length||0,b=(d.policy?.require_approval||d.policy?.requireApproval||[]).length,f=(d.policy?.redact_keys||d.policy?.redactKeys||[]).length;return`
        <div class="bento-card" style="margin-bottom: 14px; border-left: ${g?"3px solid var(--amber-400)":"1px solid var(--border)"};">
          <div style="display: flex; justify-content: space-between; align-items: flex-start;">
            <div>
              <div style="display: flex; align-items: center; gap: 8px; margin-bottom: 6px;">
                <span style="font-size: 16px; font-weight: 700; color: var(--text-main); font-family: var(--ff-mono);">${i(l)}</span>
                ${g?'<span class="brand-badge" style="color: var(--amber-400); border-color: rgba(245, 158, 11, 0.4); background: rgba(245, 158, 11, 0.1);">ACTIVE IN UI</span>':""}
                <span class="brand-badge">${d.servers.length} server${d.servers.length===1?"":"s"}</span>
                <span class="brand-badge" style="color: var(--text-dim);">${m} capabilities</span>
                ${v?'<span class="brand-badge" style="color: var(--green-400); border-color: rgba(34, 197, 94, 0.3); background: rgba(34, 197, 94, 0.08);">CUSTOM POLICY</span>':""}
              </div>
              <div style="font-size: 12px; color: var(--text-muted); margin-bottom: 10px;">
                ${i(d.description||"No description provided")}
              </div>
              <div style="display: flex; flex-wrap: wrap; gap: 6px; align-items: center; margin-bottom: ${v?"8px":"0"};">
                <span style="font-size: 11px; color: var(--text-dim); font-weight: 600; text-transform: uppercase;">Servers:</span>
                ${u||'<span style="font-size: 11px; color: var(--text-dim);">None</span>'}
              </div>
              ${v?`
                <div style="display: flex; flex-wrap: wrap; gap: 6px; align-items: center; font-size: 11px;">
                  <span style="color: var(--text-dim); font-weight: 600; text-transform: uppercase;">Policy Overlay:</span>
                  ${c>0?`<span class="brand-badge" style="color: var(--green-400);">Allow: ${c}</span>`:""}
                  ${y>0?`<span class="brand-badge" style="color: var(--red-400);">Deny: ${y}</span>`:""}
                  ${b>0?`<span class="brand-badge" style="color: var(--amber-400);">HITL: ${b}</span>`:""}
                  ${f>0?`<span class="brand-badge" style="color: var(--text-muted);">Redact: ${f}</span>`:""}
                  ${c===0&&y===0&&b===0&&f===0?'<span style="color: var(--text-dim);">Configured</span>':""}
                </div>
              `:""}
            </div>
            
            <div style="display: flex; gap: 8px; align-items: center;">
              ${g?`
                <button class="btn btn-ghost" style="padding: 4px 10px; font-size: 11.5px;" onclick="window.app.setActiveProfile(null)">
                  Deselect
                </button>
              `:`
                <button class="btn btn-primary" style="padding: 4px 10px; font-size: 11.5px;" onclick="window.app.setActiveProfile('${i(l)}')">
                  Activate in UI
                </button>
              `}
              <button class="btn btn-ghost" style="padding: 4px 10px; font-size: 11.5px;" onclick="window.app.openEditProfileModal('${i(l)}')">
                ✏️ Edit
              </button>
              <button class="btn btn-danger" style="padding: 4px 10px; font-size: 11.5px;" onclick="window.app.deleteProfile('${i(l)}')">
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
  `}function oe(){let e=p.getState(),t=e.secrets||[],a=e.config.mcpServers||{},n=[];for(let[u,m]of Object.entries(a)){let v=N(u,m.command,m.args);if(v){let c=Object.keys(m.env||{});for(let y of v.envFields)if(y.required&&!c.includes(y.key))n.push({server:u,key:y.key,uri:"(Not Configured)",is_vault:!1,exists:!1,backend:"Required Variable",display:`Required by ${v.name} template (${y.label})`,is_unconfigured_requirement:!0})}}let o=[...t,...n],r=o.length,s=o.filter((u)=>u.is_vault&&u.exists!==!1).length,l=o.filter((u)=>u.exists===!1).length,d=o.filter((u)=>!u.is_vault&&u.exists!==!1).length,g=o.length===0?`
    <div style="padding: 32px; text-align: center; color: var(--text-dim);">
      No environment variables or secrets configured in active servers.
    </div>
  `:o.map((u)=>{let m='<span class="brand-badge" style="color: var(--red-400); border-color: rgba(248, 113, 113, 0.4); background: rgba(248, 113, 113, 0.1);">Plaintext (Unsecured)</span>',v=u.exists===!1;if(u.is_unconfigured_requirement)m='<span class="brand-badge" style="color: var(--red-400); border-color: rgba(248, 113, 113, 0.4); background: rgba(248, 113, 113, 0.1);">⚠️ Not Configured (Required)</span>';else if(v)m=`<span class="brand-badge" style="color: var(--amber-300); border-color: rgba(245, 158, 11, 0.4); background: rgba(245, 158, 11, 0.1);">⚠️ Missing from ${i(u.backend)}</span>`;else if(u.is_vault)m=`<span class="brand-badge" style="color: var(--green-400); border-color: rgba(52, 211, 153, 0.3); background: rgba(52, 211, 153, 0.1);">\uD83D\uDD12 ${i(u.backend)}</span>`;return`
      <div style="display: grid; grid-template-columns: 140px 180px 1fr 180px auto; padding: 10px 16px; border-bottom: 1px solid var(--border-subtle); align-items: center; font-size: 12px;">
        <span style="font-weight: 700; color: var(--text-main);">${i(u.server)}</span>
        <span style="font-family: var(--ff-mono); color: ${v?"var(--amber-300)":"var(--amber-300)"};">${i(u.key)}</span>
        <span style="font-family: var(--ff-mono); font-size: 11px; color: ${v?"var(--red-400)":"var(--text-muted)"}; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">${i(u.display)}</span>
        <div>${m}</div>
        <div style="display: flex; gap: 6px; justify-content: flex-end;">
          ${u.is_unconfigured_requirement?`
            <button class="btn btn-primary" style="padding: 2px 8px; font-size: 11px;" onclick="window.app.quickVaultEnv('${i(u.server)}', '${i(u.key)}')">➕ Configure in Keychain</button>
          `:v?`
            <button class="btn btn-primary" style="padding: 2px 8px; font-size: 11px;" onclick="window.app.quickVaultEnv('${i(u.server)}', '${i(u.key)}')">➕ Re-add Key</button>
            <button class="btn btn-ghost" style="padding: 2px 8px; font-size: 11px; color: var(--red-400);" onclick="window.app.removeSecretFromConfig('${i(u.server)}', '${i(u.key)}')">Remove from Config</button>
          `:!u.is_vault?`
            <button class="btn btn-primary" style="padding: 2px 8px; font-size: 11px;" onclick="window.app.quickVaultEnv('${i(u.server)}', '${i(u.key)}')">\uD83D\uDD12 Move to Keychain</button>
          `:`
            <button class="btn btn-ghost" style="padding: 2px 8px; font-size: 11px; color: var(--red-400);" onclick="window.app.deleteVaultSecret('${i(u.key)}', '${i(u.server)}')">Delete Key</button>
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
  `}class ne{activeTemplateCategory="all";activeTemplateFilter="";selectedTemplate=null;pendingClientId=null;async init(){let e=window.location.port?`:${window.location.port}`:"",t=document.getElementById("daemon-port-label");if(t)t.textContent=`Daemon ${e}`;await this.refreshData(),this.initSSE(),this.render(),p.subscribe(()=>{this.render()})}auditSearchTimeout=null;async refreshData(){try{let e=p.getState(),t=e.auditFilters,a=e.activeProfile||void 0,[n,o,r,s,l,d,g,u,m,v,c]=await Promise.all([h.getConfig(),h.listCapabilities(a),h.listResources(a),h.listPrompts(a),h.getCatalogEvents(),h.listApprovals(),h.listTasks(),h.listAuditEvents({server_id:t.serverId!=="all"?t.serverId:void 0,event_type:t.eventType!=="all"?t.eventType:void 0,status:t.status!=="all"?t.status:void 0,search:t.search.trim()?t.search.trim():void 0,limit:t.limit,offset:t.offset}),h.getAuditStats(),h.getClients().catch(()=>({ok:!1,clients:[]})),h.getSecrets().catch(()=>({ok:!1,secrets:[],keychain_service:"warmplane"}))]);if(v&&v.ok&&Array.isArray(v.clients))p.setState({clients:v.clients});if(c&&c.ok&&Array.isArray(c.secrets))p.setState({secrets:c.secrets});if(n.ok)p.setState({configPath:n.config_path,config:n.config,serverStatuses:n.server_statuses||{},circuitBreakers:n.circuit_breakers||[],metrics:{totalCatalogRequests:n.metrics?.total_catalog_requests||0,totalEtagHits:n.metrics?.total_etag_hits||0,totalToolCalls:n.metrics?.total_tool_calls||0,totalToolDurationUs:n.metrics?.total_tool_duration_us||0}});if(o&&Array.isArray(o.capabilities)){let y=p.getState().selectedCapabilityId,f=o.capabilities.some((x)=>x.id===y)?y:o.capabilities.length>0?o.capabilities[0].id:null;p.setState({capabilities:o.capabilities,capabilitiesHiddenByPolicy:o.hidden_by_policy||0,selectedCapabilityId:f})}if(r&&Array.isArray(r.resources)){let y=p.getState().selectedResourceId,f=r.resources.some((x)=>x.uri===y||x.id===y)?y:r.resources.length>0?r.resources[0].uri||r.resources[0].id||null:null;p.setState({resources:r.resources,resourcesHiddenByPolicy:r.hidden_by_policy||0,selectedResourceId:f})}if(s&&Array.isArray(s.prompts)){let y=p.getState().selectedPromptId,f=s.prompts.some((x)=>x.name===y||x.id===y)?y:s.prompts.length>0?s.prompts[0].name||s.prompts[0].id||null:null;p.setState({prompts:s.prompts,promptsHiddenByPolicy:s.hidden_by_policy||0,selectedPromptId:f})}if(l&&Array.isArray(l.events))p.setState({catalogEvents:l.events});if(d&&Array.isArray(d.approvals))p.setState({approvals:d.approvals});if(g&&Array.isArray(g.tasks))p.setState({tasks:g.tasks});if(u&&Array.isArray(u.events))p.setState({auditEvents:u.events,auditTotal:u.total??u.events.length});if(m&&m.ok)p.setState({auditStats:m})}catch(e){console.error("Failed to fetch daemon state:",e)}}async refreshAuditEvents(){try{let t=p.getState().auditFilters,[a,n]=await Promise.all([h.listAuditEvents({server_id:t.serverId!=="all"?t.serverId:void 0,event_type:t.eventType!=="all"?t.eventType:void 0,status:t.status!=="all"?t.status:void 0,search:t.search.trim()?t.search.trim():void 0,limit:t.limit,offset:t.offset}),h.getAuditStats()]);if(a&&Array.isArray(a.events))p.setState({auditEvents:a.events,auditTotal:a.total??a.events.length});if(n&&n.ok)p.setState({auditStats:n})}catch(e){console.error("Failed to refresh audit events:",e)}}handleAuditSearchInput(e){let a={...p.getState().auditFilters,search:e,offset:0};p.setState({auditFilters:a}),clearTimeout(this.auditSearchTimeout),this.auditSearchTimeout=setTimeout(()=>{this.refreshAuditEvents()},250)}handleAuditStatusFilter(e){let t=p.getState();p.setState({auditFilters:{...t.auditFilters,status:e,offset:0}}),this.refreshAuditEvents()}handleAuditEventTypeFilter(e){let t=p.getState();p.setState({auditFilters:{...t.auditFilters,eventType:e,offset:0}}),this.refreshAuditEvents()}handleAuditServerFilter(e){let t=p.getState();p.setState({auditFilters:{...t.auditFilters,serverId:e,offset:0}}),this.refreshAuditEvents()}handleAuditPageSize(e){let t=parseInt(e,10)||25,a=p.getState();p.setState({auditFilters:{...a.auditFilters,limit:t,offset:0}}),this.refreshAuditEvents()}clearAuditFilters(){let e=p.getState();p.setState({auditFilters:{search:"",status:"all",eventType:"all",serverId:"all",limit:e.auditFilters.limit||25,offset:0}}),this.refreshAuditEvents()}auditPrevPage(){let e=p.getState(),{limit:t,offset:a}=e.auditFilters,n=Math.max(0,a-t);if(n!==a)p.setState({auditFilters:{...e.auditFilters,offset:n}}),this.refreshAuditEvents()}auditNextPage(){let e=p.getState(),{limit:t,offset:a}=e.auditFilters,n=e.auditTotal;if(a+t<n)p.setState({auditFilters:{...e.auditFilters,offset:a+t}}),this.refreshAuditEvents()}auditGoToPage(e){let t=p.getState(),{limit:a}=t.auditFilters,n=Math.max(0,(e-1)*a);p.setState({auditFilters:{...t.auditFilters,offset:n}}),this.refreshAuditEvents()}selectAuditEvent(e){if(!e){p.setState({auditSelectedEvent:null});return}let a=p.getState().auditEvents.find((n)=>n.id===e)||null;p.setState({auditSelectedEvent:a})}async verifyAuditChain(){try{let e=await h.verifyAuditChain();if(e&&e.report)p.setState({auditVerification:e.report})}catch(e){console.error("Failed to verify audit chain:",e)}}async refreshApprovals(){try{let e=await h.listApprovals();if(e&&Array.isArray(e.approvals))p.setState({approvals:e.approvals})}catch(e){console.error("Failed to refresh approvals:",e)}}initSSE(){try{let e=new EventSource("/v1/resources/updates");e.onmessage=(t)=>{p.addEventLog("SSE","/v1/resources/updates","UPDATED","0.1ms"),this.refreshData()}}catch(e){console.warn("SSE connection unavailable")}}switchTab(e){p.setState({activeTab:e}),this.refreshData()}render(){let e=p.getState(),t=document.getElementById("app-main");if(!t)return;let a=(e.tasks||[]).filter((d)=>d.status==="input_required").length,n=(e.approvals||[]).filter((d)=>d.status==="pending").length,o=Math.max(a,n),r=document.getElementById("nav-approvals-badge");if(r)r.textContent=o>0?`${o}`:"",r.style.display=o>0?"inline-block":"none";document.querySelectorAll(".nav-item").forEach((d)=>{let g=d.getAttribute("data-tab");if(g===e.activeTab||e.activeTab==="tasks"&&g==="approvals"||e.activeTab==="approvals"&&g==="tasks")d.classList.add("active");else d.classList.remove("active")});let s=document.getElementById("top-title"),l={overview:"Overview Cockpit",servers:"Server Hub & Connections",playground:"MCP Capability Playground",tasks:"SEP-2663 Tasks & HITL Review",approvals:"SEP-2663 Tasks & HITL Review",audit:"WORM Audit & Compliance Ledger",policy:"Security Governance & Redaction",secrets:"Native OS Keychain & Secrets Vault",aliases:"Facade & Alias Studio",profiles:"Server Constellation Profiles"};if(s)s.textContent=l[e.activeTab]||"Control Deck";switch(this.renderTopProfileSelector(),e.activeTab){case"overview":t.innerHTML=Y();break;case"servers":t.innerHTML=X();break;case"playground":t.innerHTML=Z();break;case"tasks":case"approvals":t.innerHTML=ee(e);break;case"audit":t.innerHTML=te();break;case"policy":t.innerHTML=re();break;case"secrets":t.innerHTML=oe();break;case"aliases":t.innerHTML=se();break;case"profiles":t.innerHTML=ae();break}}toggleClientsCollapse(){let e=p.getState().clientsCollapsed;p.setState({clientsCollapsed:!e}),this.render()}async saveNewVaultSecret(){let e=document.getElementById("vault-new-key"),t=document.getElementById("vault-new-val"),a=document.getElementById("vault-new-service"),n=e?.value.trim(),o=t?.value.trim(),r=a?.value.trim()||"warmplane";if(!n||!o){alert("Key and secret value are required");return}try{let s=await h.saveSecret(n,o,r);if(s.ok){if(alert(`Secret '${n}' saved securely into OS Keychain!
Reference: ${s.uri}`),e)e.value="";if(t)t.value="";await this.refreshData()}else alert(`Failed to save secret: ${s.error}`)}catch(s){alert(`Error saving secret: ${s.message}`)}}async deleteVaultSecret(e,t){let a=t?`Are you sure you want to remove secret '${e}' from OS Keychain and server '${t}'?`:`Are you sure you want to remove secret '${e}' from OS Keychain?`;if(!confirm(a))return;try{let n=await h.deleteSecret(e);if(n.ok||n.error?.includes("not found")){if(t){let s=(p.getState().config.mcpServers||{})[t];if(s&&s.env&&s.env[e]){let l={...s.env};delete l[e];let d={...s,env:l};await h.upsertServer(t,d)}}await this.refreshData()}else alert(`Failed to delete secret: ${n.error}`)}catch(n){alert(`Error deleting secret: ${n.message}`)}}async removeSecretFromConfig(e,t){if(!confirm(`Remove environment variable '${t}' from '${e}' configuration?`))return;try{let o=(p.getState().config.mcpServers||{})[e];if(o&&o.env&&o.env[t]){let r={...o.env};delete r[t];let s={...o,env:r},l=await h.upsertServer(e,s);if(l.ok)await this.refreshData();else alert(`Failed to update server config: ${l.error}`)}}catch(a){alert(`Error removing secret from config: ${a.message}`)}}async quickVaultEnv(e,t){let a=prompt(`Enter secret value to store in OS Keychain for ${e}.${t}:`);if(!a)return;try{let n=await h.saveSecret(t,a,"warmplane");if(!n.ok){alert(`Failed to save to Keychain: ${n.error}`);return}let s=(p.getState().config.mcpServers||{})[e];if(s){let l={...s.env||{},[t]:`keychain://warmplane/${t}`},d={...s,env:l},g=await h.upsertServer(e,d);if(g.ok)await this.refreshData(),alert(`Successfully configured ${e}.${t} in OS Keychain!`);else alert(`Failed to update server config: ${g.error}`)}}catch(n){alert(`Error during migration: ${n.message}`)}}async refreshTasks(){try{let e=await h.listTasks();if(e&&Array.isArray(e.tasks))p.setState({tasks:e.tasks})}catch(e){console.error("Failed to refresh tasks:",e)}}filterTasksByStatus(e){p.setState({taskFilterStatus:e})}togglePlaygroundAsyncTask(e){p.setState({playgroundAsyncTask:e})}async submitTaskInputResponses(e){let a=p.getState().tasks.find((r)=>r.taskId===e)?.inputRequests||{},n=Object.keys(a),o={};if(n.length>0)for(let r of n){let s=a[r];if(s&&s.type==="approval_review"){let l=document.getElementById(`task-input-${e}-${r}-decision`),d=document.getElementById(`task-input-${e}-${r}`),g=l?l.value==="true":!0,u=void 0;if(d&&d.value.trim())try{u=JSON.parse(d.value.trim())}catch{alert("Invalid JSON in parameters editor");return}o[r]={approved:g,modified_args:u,reason:g?void 0:"Operator rejected execution via Tasks review"}}else{let l=document.getElementById(`task-input-${e}-${r}`);if(l){let d=l.value.trim();try{o[r]=JSON.parse(d)}catch{o[r]=d}}}}else{let r=document.getElementById(`task-raw-input-${e}`);if(r&&r.value.trim())try{Object.assign(o,JSON.parse(r.value.trim()))}catch{alert("Invalid JSON in raw input responses");return}}try{let r=await h.updateTask(e,o);if(r.ok)await this.refreshTasks();else alert(`Task update failed: ${r.error?.message||r.error||"Unknown error"}`)}catch(r){alert(`Error updating task: ${r.message}`)}}async promptCancelTask(e){let t=prompt("Reason for cancelling task:");if(t===null)return;try{let a=await h.cancelTask(e,t||void 0);if(a.ok)await this.refreshTasks();else alert(`Task cancellation failed: ${a.error?.message||a.error||"Unknown error"}`)}catch(a){alert(`Error cancelling task: ${a.message}`)}}async openTaskInspectorModal(e){this.closeModals();let t=document.getElementById("modal-task-inspector");if(!t)return;let a=document.getElementById("modal-task-title"),n=document.getElementById("modal-task-body"),o=document.getElementById("modal-task-footer");if(a)a.textContent=`Task Inspector: ${e}`;if(n)n.innerHTML=`
        <div style="padding: 30px; text-align: center; color: var(--text-dim); font-family: var(--ff-mono); font-size: 12px;">
          ⏳ Fetching task execution record...
        </div>
      `;t.classList.add("active");try{let r=await h.getTask(e);if(!r.ok||!r.task){if(n)n.innerHTML=`
            <div style="background: rgba(248, 113, 113, 0.12); border: 1px solid rgba(248, 113, 113, 0.3); border-radius: var(--radius-sm); padding: 16px; color: var(--red-400);">
              <div style="font-weight: 700; margin-bottom: 6px;">Failed to load task snapshot</div>
              <div style="font-family: var(--ff-mono); font-size: 11.5px;">${i(r.error?.message||"Task not found in runtime registry")}</div>
            </div>
          `;return}let s=r.task,l=s.progress!==void 0?Math.round(s.progress*100):s.status==="completed"?100:s.status==="working"?50:0,d=Math.floor(Date.now()/1000),g=s.expiresAtEpochSecs?Math.max(0,s.expiresAtEpochSecs-d):s.ttlSeconds||300,u=s.status==="completed"||s.status==="cancelled"||s.status==="failed",m=s.status==="completed"?"var(--green-400)":s.status==="working"?"var(--cyan-400)":s.status==="input_required"?"var(--amber-300)":s.status==="cancelled"?"var(--text-muted)":"var(--red-400)",v=s.status==="completed"?"rgba(52, 211, 153, 0.15)":s.status==="working"?"rgba(56, 189, 248, 0.15)":s.status==="input_required"?"rgba(245, 158, 11, 0.18)":s.status==="cancelled"?"rgba(148, 163, 184, 0.15)":"rgba(248, 113, 113, 0.15)",c=!!s.error,y=s.result!==void 0&&s.result!==null,b=s.inputRequests&&Object.keys(s.inputRequests).length>0;if(n)n.innerHTML=`
          <!-- Status Banner & Top Metrics -->
          <div style="display: flex; justify-content: space-between; align-items: center; background: var(--surface-card); border: 1px solid var(--border); border-radius: var(--radius-md); padding: 12px 16px; margin-bottom: 14px;">
            <div>
              <div style="display: flex; align-items: center; gap: 8px;">
                <span class="brand-badge" style="background: ${v}; color: ${m}; border-color: ${m}; font-size: 11px;">
                  ${i(s.status.toUpperCase())}
                </span>
                <span style="font-family: var(--ff-mono); font-size: 13px; font-weight: 700; color: var(--text-main);">
                  ${i(s.capabilityId||"Tool Execution")}
                </span>
                ${s.serverId?`<span style="font-size: 11px; color: var(--text-dim);">via <code style="color: var(--cyan-400);">${i(s.serverId)}</code></span>`:""}
              </div>
              <div style="font-family: var(--ff-mono); font-size: 11px; color: var(--text-dim); margin-top: 4px;">
                Task ID: <span style="color: var(--text-muted);">${i(s.taskId)}</span>
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
              ${s.context.actor_id?`<div><span style="color: var(--text-dim);">Actor:</span> <span style="color: var(--cyan-400);">${i(s.context.actor_id)}</span></div>`:""}
              ${s.context.operation_id?`<div><span style="color: var(--text-dim);">Operation:</span> <span style="color: var(--text-main);">${i(s.context.operation_id)}</span></div>`:""}
              ${s.context.grant_id?`<div><span style="color: var(--text-dim);">Grant:</span> <span style="color: var(--text-main);">${i(s.context.grant_id)}</span></div>`:""}
            </div>
          `:""}

          <!-- Diagnostic / Error Message -->
          ${c?`
            <div style="background: rgba(248, 113, 113, 0.1); border: 1px solid rgba(248, 113, 113, 0.3); border-radius: var(--radius-sm); padding: 12px; margin-bottom: 14px;">
              <div style="font-size: 11px; font-weight: 700; color: var(--red-400); text-transform: uppercase; margin-bottom: 6px;">
                ⚠️ Failure / Cancellation Trace
              </div>
              <pre style="font-family: var(--ff-mono); font-size: 11.5px; color: var(--red-300); white-space: pre-wrap; word-break: break-word; margin: 0;">${i(typeof s.error==="string"?s.error:JSON.stringify(s.error,null,2))}</pre>
            </div>
          `:""}

          <!-- Input Requests (if awaiting input) -->
          ${b?`
            <div style="margin-bottom: 14px;">
              <div style="font-size: 11px; font-weight: 700; color: var(--amber-400); text-transform: uppercase; margin-bottom: 6px;">
                ⚡ Pending Input Requests (MRTR / HITL)
              </div>
              <pre style="background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 12px; font-family: var(--ff-mono); font-size: 11.5px; color: var(--amber-300); white-space: pre-wrap; word-break: break-word; max-height: 180px; overflow-y: auto; margin: 0;">${i(JSON.stringify(s.inputRequests,null,2))}</pre>
            </div>
          `:""}

          <!-- Result Payload -->
          <div style="margin-bottom: 8px;">
            <div style="font-size: 11px; font-weight: 700; color: var(--text-muted); text-transform: uppercase; margin-bottom: 6px;">
              ${y?"\uD83D\uDCE6 Output Result Payload":"State Details"}
            </div>
            <pre style="background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 12px; font-family: var(--ff-mono); font-size: 11.5px; color: var(--text-main); white-space: pre-wrap; word-break: break-word; max-height: 220px; overflow-y: auto; margin: 0;">${i(y?JSON.stringify(s.result,null,2):s.status==="working"?"Task execution is currently in-flight in background worker pool.":"No output payload recorded.")}</pre>
          </div>
        `;if(o)o.innerHTML=`
          <div>
            ${!u?`
              <button class="btn btn-danger" style="font-size: 11.5px;" onclick="window.app.promptCancelTask('${i(s.taskId)}'); window.app.closeModals();">
                ⛔ Cancel Task
              </button>
            `:`
              <span style="font-family: var(--ff-mono); font-size: 11px; color: var(--text-dim);">Task is terminal (${i(s.status)})</span>
            `}
          </div>
          <div style="display: flex; gap: 8px;">
            <button class="btn btn-ghost" style="font-size: 11.5px;" onclick="navigator.clipboard.writeText('${i(s.taskId)}'); alert('Task ID copied to clipboard');">
              \uD83D\uDCCB Copy Task ID
            </button>
            <button class="btn btn-ghost" style="font-size: 11.5px;" onclick="window.app.closeModals()">
              Close
            </button>
          </div>
        `}catch(r){if(n)n.innerHTML=`
          <div style="color: var(--red-400); font-family: var(--ff-mono); font-size: 11.5px;">
            Failed to inspect task: ${i(r.message)}
          </div>
        `}}async submitApproval(e){let t=document.getElementById(`appr-operator-${e}`),a=document.getElementById(`appr-args-${e}`),n=t?.value.trim()||"security-operator",o=void 0;if(a&&a.value.trim())try{o=JSON.parse(a.value.trim())}catch{alert("Invalid JSON in arguments editor");return}let r=await h.approveTicket(e,n,o);if(r.ok)await this.refreshApprovals(),await this.refreshTasks();else alert(`Approval failed: ${r.error||"Unknown error"}`)}async promptReject(e){let t=prompt("Reason for rejection (will be returned to the calling agent):");if(t===null)return;let n=document.getElementById(`appr-operator-${e}`)?.value.trim()||"security-operator",o=await h.rejectTicket(e,n,t);if(o.ok)await this.refreshApprovals(),await this.refreshTasks();else alert(`Rejection failed: ${o.error||"Unknown error"}`)}setPlaygroundMode(e){p.setState({playgroundMode:e})}selectCapability(e){p.setState({selectedCapabilityId:e});let t=p.getState().capabilities.find((n)=>n.id===e),a=document.getElementById("pg-args-input");if(t){let n=K(t.input_schema,!1),o=JSON.stringify(n,null,2);if(a)a.value=o;let r={...p.getState().playgroundArgs||{}};r[e]=o,p.getState().playgroundArgs=r}}selectResource(e){p.setState({selectedResourceId:e})}selectPrompt(e){p.setState({selectedPromptId:e})}filterResources(e){let t=e.toLowerCase().trim(),n=(p.getState().resources||[]).filter((r)=>r.id.toLowerCase().includes(t)||r.name&&r.name.toLowerCase().includes(t)||r.uri&&r.uri.toLowerCase().includes(t)||r.server&&r.server.toLowerCase().includes(t)),o=document.getElementById("pg-res-list");if(o)if(n.length===0)o.innerHTML=`
          <div style="padding: 24px 16px; text-align: center; color: var(--text-dim); font-size: 11.5px;">
            No resources match "${i(e)}"
          </div>
        `;else o.innerHTML=n.map((r)=>{let s=r.id===p.getState().selectedResourceId?"active":"",l=r.uri?r.uri.split(":")[0]:"res";return`
            <div class="cap-item ${s}" onclick="window.app.selectResource('${i(r.id)}')">
              <div style="display: flex; justify-content: space-between; align-items: center;">
                <span style="font-weight: 600; color: var(--text-main); font-family: var(--ff-mono); font-size: 12px;">${i(r.name||r.id)}</span>
                <span class="badge" style="font-size: 9.5px; background: rgba(56, 189, 248, 0.15); color: var(--cyan-400);">${i(l)}</span>
              </div>
              <div style="font-size: 11px; color: var(--text-dim); margin-top: 2px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">${i(r.uri)}</div>
              <div style="display: flex; justify-content: space-between; font-size: 10px; color: var(--text-muted); margin-top: 4px;">
                <span>server: ${i(r.server||"local")}</span>
                <span>${i(r.mime_type||"text/plain")}</span>
              </div>
            </div>
          `}).join("")}filterPrompts(e){let t=e.toLowerCase().trim(),n=(p.getState().prompts||[]).filter((r)=>r.id.toLowerCase().includes(t)||r.name&&r.name.toLowerCase().includes(t)||r.description&&r.description.toLowerCase().includes(t)||r.server&&r.server.toLowerCase().includes(t)),o=document.getElementById("pg-prompt-list");if(o)if(n.length===0)o.innerHTML=`
          <div style="padding: 24px 16px; text-align: center; color: var(--text-dim); font-size: 11.5px;">
            No prompts match "${i(e)}"
          </div>
        `;else o.innerHTML=n.map((r)=>{let s=r.id===p.getState().selectedPromptId?"active":"",l=r.arguments?r.arguments.length:0;return`
            <div class="cap-item ${s}" onclick="window.app.selectPrompt('${i(r.id)}')">
              <div style="display: flex; justify-content: space-between; align-items: center;">
                <span style="font-weight: 600; color: var(--text-main); font-family: var(--ff-mono); font-size: 12px;">${i(r.name||r.id)}</span>
                <span class="badge" style="font-size: 9.5px; background: rgba(168, 85, 247, 0.15); color: var(--purple-400);">${l} args</span>
              </div>
              <div style="font-size: 11px; color: var(--text-dim); margin-top: 2px;">${i(r.description||r.title||"Prompt template")}</div>
              <div style="font-size: 10px; color: var(--text-muted); margin-top: 4px;">server: ${i(r.server||"local")}</div>
            </div>
          `}).join("")}updatePlaygroundArgs(e){let t=p.getState(),a=t.selectedCapabilityId||(t.capabilities[0]?t.capabilities[0].id:null);if(!a)return;let n={...t.playgroundArgs||{}};n[a]=e,t.playgroundArgs=n}fillPlaygroundSampleArgs(e=!1){let t=p.getState(),a=t.selectedCapabilityId||(t.capabilities[0]?t.capabilities[0].id:null),n=t.capabilities.find((l)=>l.id===a),o=document.getElementById("pg-args-input");if(!o)return;if(!n||!n.input_schema){if(o.value="{}",a){let l={...t.playgroundArgs||{}};l[a]="{}",t.playgroundArgs=l}return}let r=K(n.input_schema,e),s=JSON.stringify(r,null,2);if(o.value=s,a){let l={...t.playgroundArgs||{}};l[a]=s,t.playgroundArgs=l}}formatPlaygroundArgs(){let e=p.getState(),t=e.selectedCapabilityId||(e.capabilities[0]?e.capabilities[0].id:null),a=document.getElementById("pg-args-input");if(a)try{let n=JSON.parse(a.value||"{}"),o=JSON.stringify(n,null,2);if(a.value=o,t){let r={...e.playgroundArgs||{}};r[t]=o,e.playgroundArgs=r}}catch(n){alert(`Cannot format JSON: ${n.message}`)}}insertPlaygroundArgKey(e,t,a){let n=p.getState(),o=n.selectedCapabilityId||(n.capabilities[0]?n.capabilities[0].id:null),r=document.getElementById("pg-args-input");if(r){let s={};try{s=JSON.parse(r.value||"{}")}catch{s={}}if(s[e]===void 0)if(a!==null&&a!==void 0)s[e]=a;else switch(t){case"string":s[e]=`sample_${e}`;break;case"number":case"integer":s[e]=0;break;case"boolean":s[e]=!0;break;case"array":s[e]=[];break;case"object":s[e]={};break;default:s[e]=`sample_${e}`}let l=JSON.stringify(s,null,2);if(r.value=l,o){let d={...n.playgroundArgs||{}};d[o]=l,n.playgroundArgs=d}}}fillBatchStepSampleArgs(e){let t=p.getState(),a=[...t.batchSteps||[]],n=a[e];if(!n||!n.capability_id)return;let o=t.capabilities.find((d)=>d.id===n.capability_id);if(!o||!o.input_schema)return;let r=o.input_schema.properties||{},s={};for(let[d,g]of Object.entries(r))if(g.default!==void 0)s[d]=g.default;else if(Array.isArray(g.enum)&&g.enum.length>0)s[d]=g.enum[0];else switch(g.type||"string"){case"string":s[d]=`sample_${d}`;break;case"number":case"integer":s[d]=0;break;case"boolean":s[d]=!0;break;case"array":s[d]=[];break;case"object":s[d]={};break;default:s[d]=`sample_${d}`}let l=JSON.stringify(s,null,2);a[e]={...a[e],argsJson:l},p.setState({batchSteps:a})}filterCapabilities(e){let t=e.toLowerCase().trim(),n=p.getState().capabilities.filter((r)=>r.id.toLowerCase().includes(t)||r.summary&&r.summary.toLowerCase().includes(t)||r.server&&r.server.toLowerCase().includes(t)),o=document.getElementById("pg-cap-list");if(o)if(n.length===0)o.innerHTML=`
          <div style="padding: 24px 16px; text-align: center; color: var(--text-dim); font-size: 11.5px;">
            No capabilities match "${i(e)}"
          </div>
        `;else o.innerHTML=n.map((r)=>`
          <div class="cap-item ${r.id===p.getState().selectedCapabilityId?"active":""}" onclick="window.app.selectCapability('${i(r.id)}')">
            <div style="display: flex; justify-content: space-between; align-items: center;">
              <span style="font-weight: 600; color: var(--text-main); font-family: var(--ff-mono); font-size: 12px;">${i(r.id)}</span>
              <span style="font-size: 10px; color: var(--green-400);">${i(r.mode||"read")}</span>
            </div>
            <div style="font-size: 11px; color: var(--text-dim); margin-top: 2px;">server: ${i(r.server||"local")}</div>
          </div>
        `).join("")}async executePlaygroundTool(){let e=p.getState(),t=e.selectedCapabilityId||(e.capabilities[0]?e.capabilities[0].id:null);if(!t)return;let a=document.getElementById("pg-args-input")?.value||"{}",n=document.getElementById("pg-context-input")?.value||void 0,o=document.getElementById("pg-jsonpath-input")?.value.trim()||void 0,r=document.getElementById("pg-limit-lines-input")?.value.trim()||void 0,s=document.getElementById("pg-truncate-bytes-input")?.value.trim()||void 0,l={};try{l=JSON.parse(a)}catch{alert("Invalid arguments JSON object");return}if(o)l._jsonpath=o;if(r&&!isNaN(Number(r)))l._limit_lines=Number(r);if(s&&!isNaN(Number(s)))l._truncate_bytes=Number(s);let d=`op-${Date.now()}`;p.setState({isExecuting:!0,activeRequestId:d});let g=e.activeProfile||void 0,u=e.playgroundAsyncTask||!1;try{let m=await h.callCapability({capability_id:t,args:l,request_id:d,async_task:u?!0:void 0,context:{operation_id:n||d}},g);if(p.setState({isExecuting:!1,activeRequestId:null,executionResult:{status:m.status,durationMs:m.durationMs,data:m.data}}),m.status===202||m.data?.resultType==="task")this.refreshTasks();p.addEventLog("POST",`/v1/tools/call → ${t}`,m.status===200?"200 OK":`HTTP ${m.status}`,`${m.durationMs.toFixed(1)}ms`),h.getConfig().then((v)=>{if(v.ok&&v.circuit_breakers)p.setState({circuitBreakers:v.circuit_breakers})})}catch(m){p.setState({isExecuting:!1,activeRequestId:null,executionResult:{status:500,durationMs:0,data:{error:m.toString()}}})}}async cancelActiveOperation(){let t=p.getState().activeRequestId;if(t)try{await h.cancelOperation(t)}catch(a){console.warn("Failed to send cancel signal:",a)}p.setState({isExecuting:!1,activeRequestId:null,executionResult:{status:499,durationMs:0,data:{ok:!1,error:{code:"CANCELLED",message:"Operation cancelled by operator"}}}})}openBatchModal(){let e=p.getState(),t=e.batchSteps;if(!t||t.length===0)t=[{id:"step_1",capability_id:e.selectedCapabilityId||(e.capabilities[0]?e.capabilities[0].id:""),argsJson:"{}",continue_on_error:!1},{id:"step_2",capability_id:"",argsJson:"{}",continue_on_error:!0}],p.setState({batchSteps:t});p.setState({isBatchModalOpen:!0})}closeBatchModal(){p.setState({isBatchModalOpen:!1})}addBatchStep(){let t=[...p.getState().batchSteps||[]],a=t.length+1;t.push({id:`step_${a}`,capability_id:"",argsJson:"{}",continue_on_error:!1}),p.setState({batchSteps:t})}removeBatchStep(e){let a=[...p.getState().batchSteps||[]];if(a.length<=1){alert("Pipeline must contain at least one execution step.");return}a.splice(e,1);let n=a.map((o,r)=>({...o,id:`step_${r+1}`}));p.setState({batchSteps:n})}updateBatchStepCapability(e,t){let n=[...p.getState().batchSteps||[]];if(n[e])n[e]={...n[e],capability_id:t},p.setState({batchSteps:n})}updateBatchStepContinueOnError(e,t){let n=[...p.getState().batchSteps||[]];if(n[e])n[e]={...n[e],continue_on_error:t},p.setState({batchSteps:n})}updateBatchStepArgs(e,t){let a=p.getState(),n=[...a.batchSteps||[]];if(n[e])n[e]={...n[e],argsJson:t},a.batchSteps[e].argsJson=t}appendBatchVariable(e,t){let n=[...p.getState().batchSteps||[]],o=document.getElementById(`batch-step-args-${e}`);if(o){let r=o.value,s=o.selectionStart||r.length,l=o.selectionEnd||r.length,d=r.substring(0,s)+t+r.substring(l);if(o.value=d,n[e])n[e]={...n[e],argsJson:d},p.setState({batchSteps:n})}}async executeBatchPipeline(){let e=p.getState(),t=e.batchSteps||[],a=[];for(let o=0;o<t.length;o++){let r=t[o];if(!r.capability_id){alert(`Please select a capability for Step ${o+1}`);return}let s={};try{s=JSON.parse(r.argsJson||"{}")}catch{alert(`Invalid JSON in Step ${o+1} arguments`);return}a.push({id:r.id||`step_${o+1}`,capability_id:r.capability_id,args:s,continue_on_error:r.continue_on_error})}p.setState({isBatchModalOpen:!1});let n=e.activeProfile||void 0;try{let o=await h.batchCallCapabilities(a,n);p.setState({executionResult:{status:o.status,durationMs:o.durationMs,data:o.data}}),p.addEventLog("POST",`/v1/tools/batch_call (${t.length} steps)`,o.status===200?"200 OK":`HTTP ${o.status}`,`${o.durationMs.toFixed(1)}ms`)}catch(o){p.setState({executionResult:{status:500,durationMs:0,data:{error:o.toString()}}})}}async executeReadResource(){let e=p.getState(),t=e.selectedResourceId||(e.resources[0]?e.resources[0].id:null);if(!t)return;let a=document.getElementById("pg-res-jsonpath-input")?.value.trim()||void 0,n=document.getElementById("pg-res-lines-input")?.value.trim()||void 0,o=document.getElementById("pg-res-bytes-input")?.value.trim()||void 0,r={resource_id:t};if(a)r._jsonpath=a;if(n&&!isNaN(Number(n)))r._limit_lines=Number(n);if(o&&!isNaN(Number(o)))r._truncate_bytes=Number(o);let s=e.activeProfile||void 0;try{let l=await h.readResource({resource_id:t,input_responses:r},s);p.setState({resourceReadResult:{status:l.status,durationMs:l.durationMs,data:l.data}}),p.addEventLog("POST",`/v1/resources/read → ${t}`,l.status===200?"200 OK":`HTTP ${l.status}`,`${l.durationMs.toFixed(1)}ms`)}catch(l){p.setState({resourceReadResult:{status:500,durationMs:0,data:{error:l.toString()}}})}}async executeGetPrompt(){let e=p.getState(),t=e.selectedPromptId||(e.prompts[0]?e.prompts[0].id:null);if(!t)return;let a=document.querySelectorAll(".prompt-arg-input"),n={};a.forEach((r)=>{let s=r,l=s.getAttribute("data-arg-name");if(l&&s.value.trim())n[l]=s.value.trim()});let o=e.activeProfile||void 0;try{let r=await h.getPrompt({prompt_id:t,arguments:n},o);p.setState({promptGetResult:{status:r.status,durationMs:r.durationMs,data:r.data}}),p.addEventLog("POST",`/v1/prompts/get → ${t}`,r.status===200?"200 OK":`HTTP ${r.status}`,`${r.durationMs.toFixed(1)}ms`)}catch(r){p.setState({promptGetResult:{status:500,durationMs:0,data:{error:r.toString()}}})}}toggleBatchPlayground(){let e=document.getElementById("pg-args-input");if(!e)return;let t=[{id:"step_1",capability_id:"sqlite.read_query",args:{query:"SELECT * FROM users LIMIT 2"}},{id:"step_2",capability_id:"github.issues.search",args:{query:"label:bug"},continue_on_error:!0}];e.value=JSON.stringify(t,null,2)}async submitPolicyRule(e){let t=e==="allow"?"policy-new-allow":e==="deny"?"policy-new-deny":e==="redact"?"policy-new-redact":"policy-new-requireApproval",a=document.getElementById(t);if(!a)return;let n=a.value.trim();if(!n)return;await this.addPolicyRule(e,n),a.value=""}async addPolicyRule(e,t){let a=(t||"").trim();if(!a)return;let n=p.getState(),o=n.activeProfile,r=o?n.config.profiles?.[o]:void 0;if(r&&o){let s=r.policy||{},l=[...s.allow||[]],d=[...s.deny||[]],g=[...s.redact_keys||s.redactKeys||[]],u=[...s.require_approval||s.requireApproval||[]];if(e==="allow"&&!l.includes(a))l.push(a);if(e==="deny"&&!d.includes(a))d.push(a);if(e==="redact"&&!g.includes(a))g.push(a);if(e==="requireApproval"&&!u.includes(a))u.push(a);let m={...s,allow:l,deny:d,redactKeys:g,requireApproval:u},v=await h.upsertProfile(o,r.servers,r.description,m);if(!v.ok)alert(`Failed to save profile policy rule: ${v.error||"Unknown error"}`)}else{let s=n.config.policy||{},l=[...s.allow||[]],d=[...s.deny||[]],g=[...s.redact_keys||s.redactKeys||[]],u=[...s.require_approval||s.requireApproval||[]];if(e==="allow"&&!l.includes(a))l.push(a);if(e==="deny"&&!d.includes(a))d.push(a);if(e==="redact"&&!g.includes(a))g.push(a);if(e==="requireApproval"&&!u.includes(a))u.push(a);let m=await h.savePolicy({...s,allow:l,deny:d,redact_keys:g,redactKeys:g,require_approval:u,requireApproval:u});if(!m.ok)alert(`Failed to save policy rule: ${m.error||"Unknown error"}`)}await this.refreshData()}async removePolicyRule(e,t){let a=p.getState(),n=a.activeProfile,o=n?a.config.profiles?.[n]:void 0;if(o&&n){let r=o.policy||{},s=[...r.allow||[]],l=[...r.deny||[]],d=[...r.redact_keys||r.redactKeys||[]],g=[...r.require_approval||r.requireApproval||[]];if(e==="allow")s.splice(t,1);if(e==="deny")l.splice(t,1);if(e==="redact")d.splice(t,1);if(e==="requireApproval")g.splice(t,1);let u={...r,allow:s,deny:l,redactKeys:d,requireApproval:g},m=await h.upsertProfile(n,o.servers,o.description,u);if(!m.ok)alert(`Failed to update profile policy: ${m.error||"Unknown error"}`)}else{let r=a.config.policy||{},s=[...r.allow||[]],l=[...r.deny||[]],d=[...r.redact_keys||r.redactKeys||[]],g=[...r.require_approval||r.requireApproval||[]];if(e==="allow")s.splice(t,1);if(e==="deny")l.splice(t,1);if(e==="redact")d.splice(t,1);if(e==="requireApproval")g.splice(t,1);let u=await h.savePolicy({...r,allow:s,deny:l,redact_keys:d,redactKeys:d,require_approval:g,requireApproval:g});if(!u.ok)alert(`Failed to update policy: ${u.error||"Unknown error"}`)}await this.refreshData()}async saveWebhookConfig(){let e=document.getElementById("policy-webhook-url"),t=document.getElementById("policy-webhook-format"),a=document.getElementById("policy-webhook-secret"),n=e?e.value.trim():"",o=t?t.value:"generic",r=a?a.value.trim():"",l=p.getState().config.policy||{},d=n?{url:n,format:o,secret:r&&!r.startsWith("WARMPLANE_")&&!r.includes("_")?r:void 0,secret_env:r&&(r.startsWith("WARMPLANE_")||r.includes("_"))?r:void 0,events:["approval.requested","circuit_breaker.tripped","policy.violation"]}:void 0,g=await h.savePolicy({...l,webhook:d});if(g.ok)alert("Webhook settings saved successfully");else alert(`Failed to save webhook settings: ${g.error||"Unknown error"}`);await this.refreshData()}async testWebhook(){let e=document.getElementById("policy-webhook-url"),t=document.getElementById("policy-webhook-format"),a=e?e.value.trim():void 0,n=t?t.value:void 0,o=p.getState(),r=typeof o.config.policy?.webhook==="object"?o.config.policy.webhook?.url:void 0;if(a&&a!==r)await this.saveWebhookConfig();let s=document.getElementById("policy-webhook-status");if(s)s.textContent="Sending test event...",s.style.color="var(--cyan-400)";try{let l=await h.testWebhook(a,n);if(l.ok){if(alert(`Test webhook sent successfully! (${l.message})`),s)s.textContent=`✔ Test sent (HTTP ${l.status_code||200})`,s.style.color="var(--green-400)"}else if(alert(`Test webhook failed: ${l.error||"Unknown error"}`),s)s.textContent=`✖ Failed: ${l.error}`,s.style.color="var(--red-400)"}catch(l){alert(`Error sending test webhook: ${l.message}`)}}testPolicySandbox(e){let t=document.getElementById("policy-test-verdict");if(!t)return;let a=e.trim();if(!a){t.textContent="ENTER ID",t.style.color="var(--text-dim)";return}let n=p.getState(),o=n.activeProfile,r=o?n.config.profiles?.[o]:void 0,s=n.config.policy||{},l=r?.policy,d=s.deny||[],g=l?.deny||[],u=Array.from(new Set([...d,...g])),m=s.require_approval||s.requireApproval||[],v=l?.require_approval||l?.requireApproval||[],c=Array.from(new Set([...m,...v])),y=l&&l.allow&&l.allow.length>0?l.allow:s.allow||[],b=(f,x)=>{if(f==="*")return!0;if(f.endsWith("*"))return x.startsWith(f.slice(0,-1));return f===x};if(u.some((f)=>b(f,a))){t.textContent="DENIED (Strict Block)",t.style.color="var(--red-400)";return}if(y.length>0&&!y.some((f)=>b(f,a))){t.textContent="DENIED (Not in Allow List)",t.style.color="var(--red-400)";return}if(c.some((f)=>b(f,a))){t.textContent="REQUIRE APPROVAL (HITL Gate)",t.style.color="var(--amber-400)";return}t.textContent="ALLOWED",t.style.color="var(--green-400)"}async deleteServer(e){let t=p.getState().activeProfile,a=t?`Permanently delete server '${e}' globally from warmplane configuration? (This will also unbind it from profile '${t}')`:`Are you sure you want to permanently remove server '${e}' from configuration?`;if(!confirm(a))return;try{let n=await h.deleteServer(e);if(n.ok)await this.refreshData();else alert(`Failed to remove server '${e}': ${n.error||"Unknown error"}`)}catch(n){alert(`Error removing server '${e}': ${n.message}`)}}async restartServer(e){try{let t=await h.restartServer(e);if(t.ok)await this.refreshData();else alert(`Failed to restart server '${e}': ${t.error||"Unknown error"}`)}catch(t){alert(`Error restarting server '${e}': ${t.message}`)}}openServerDiagnosticsModal(e){this.closeModals();let t=p.getState(),a=t.config.mcpServers?.[e],n=t.serverStatuses?.[e],o=(t.circuitBreakers||[]).find((d)=>d.server_id===e),r=document.getElementById("modal-server-diagnostics");if(!r)return;let s=document.getElementById("modal-diag-title"),l=document.getElementById("modal-diag-body");if(s)s.textContent=`Live Diagnostics: ${e}`;if(l){let d=n?.status==="degraded",g=d?"var(--amber-400)":n?.status==="connected"?"var(--green-400)":"var(--red-400)",u=n?.error||"No active crash or error reported. Server is healthy.",m=N(e,a?.command,a?.args),v=a?.env||{},c=Object.keys(v),b=(m?.envFields||[]).filter((x)=>x.required).filter((x)=>!c.includes(x.key)),f="";if(m||c.length>0){let x=(m?.envFields||[]).map((E)=>{let w=v[E.key]!==void 0,C='<span class="brand-badge" style="color: var(--green-400); border-color: rgba(52, 211, 153, 0.3);">Configured</span>';if(!w&&E.required)C='<span class="brand-badge" style="color: var(--red-400); border-color: rgba(248, 113, 113, 0.4); background: rgba(248, 113, 113, 0.1);">Required / Missing</span>';else if(!w)C='<span class="brand-badge" style="color: var(--text-dim); border-color: var(--border);">Optional / Not Set</span>';return`
            <div style="display: flex; justify-content: space-between; align-items: center; padding: 6px 10px; background: var(--surface); border-radius: var(--radius-xs); margin-bottom: 6px; font-size: 11.5px;">
              <div>
                <span style="font-family: var(--ff-mono); font-weight: 700; color: ${!w&&E.required?"var(--amber-300)":"var(--text-main)"};">${i(E.key)}</span>
                ${E.label?`<span style="font-size: 10.5px; color: var(--text-dim); margin-left: 6px;">(${i(E.label)})</span>`:""}
              </div>
              <div style="display: flex; align-items: center; gap: 8px;">
                ${C}
                ${!w?`
                  <button class="btn btn-primary" style="padding: 2px 8px; font-size: 10.5px;" onclick="window.app.quickVaultEnv('${i(e)}', '${i(E.key)}')">➕ Configure in Keychain</button>
                `:""}
              </div>
            </div>
          `}).join(""),T=(m?.envFields||[]).map((E)=>E.key),I=Object.entries(v).filter(([E])=>!T.includes(E)).map(([E,k])=>`
            <div style="display: flex; justify-content: space-between; align-items: center; padding: 6px 10px; background: var(--surface); border-radius: var(--radius-xs); margin-bottom: 6px; font-size: 11.5px;">
              <span style="font-family: var(--ff-mono); font-weight: 700; color: var(--text-main);">${i(E)}</span>
              <span class="brand-badge" style="color: var(--green-400); border-color: rgba(52, 211, 153, 0.3);">Custom Configured</span>
            </div>
          `).join("");f=`
          <div style="background: rgba(0,0,0,0.2); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 12px; margin-bottom: 14px;">
            <div style="font-size: 11px; font-weight: 700; color: ${b.length>0?"var(--amber-400)":"var(--text-main)"}; text-transform: uppercase; margin-bottom: 8px; display: flex; justify-content: space-between; align-items: center;">
              <span>\uD83D\uDD11 Environment Variables &amp; Secrets</span>
              ${b.length>0?`<span style="color: var(--red-400); font-size: 10.5px;">⚠️ ${b.length} required key(s) missing</span>`:""}
            </div>
            ${x}
            ${I}
          </div>
        `}l.innerHTML=`
        <div style="display: flex; gap: 10px; align-items: center; margin-bottom: 16px;">
          <span style="width: 10px; height: 10px; border-radius: 50%; background: ${g};"></span>
          <span style="font-weight: 700; font-size: 14px; color: var(--text-main);">Current Status: <span style="color: ${g}; text-transform: uppercase;">${i(n?.status||"unknown")}</span></span>
          <span class="brand-badge" style="color: var(--cyan-400);">Protocol: ${i(n?.protocol_version||"2026-07-28")}</span>
          ${b.length>0?'<span class="brand-badge" style="color: var(--red-400); border-color: rgba(248, 113, 113, 0.4); background: rgba(248, 113, 113, 0.1);">⚠️ Missing Required Keys</span>':""}
        </div>

        <div style="background: rgba(0,0,0,0.3); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 12px; margin-bottom: 14px;">
          <div style="font-size: 11px; font-weight: 700; color: var(--amber-400); text-transform: uppercase; margin-bottom: 6px;">
            ⚠️ Diagnostic Details / Failure Root Cause
          </div>
          <pre style="font-family: var(--ff-mono); font-size: 11.5px; color: ${d?"var(--red-300)":"var(--text-dim)"}; white-space: pre-wrap; word-break: break-word; margin: 0;">${i(u)}</pre>
        </div>

        ${f}

        <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 10px; margin-bottom: 14px;">
          <div style="background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 10px;">
            <div style="font-size: 10.5px; color: var(--text-dim);">Circuit Breaker State</div>
            <div style="font-weight: 700; font-size: 13px; color: var(--text-main); margin-top: 2px;">
              ${o?`${o.state.toUpperCase()} (${o.consecutive_failures} failures)`:"CLOSED (Healthy)"}
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
            ${a?.command?`${i(a.command)} ${i((a.args||[]).join(" "))}`:i(a?.url||"")}
          </code>
        </div>

        <div style="display: flex; justify-content: flex-end; gap: 8px;">
          <button class="btn btn-primary" onclick="window.app.restartServer('${i(e)}'); window.app.closeModals();">⚡ Restart &amp; Probe Now</button>
          <button class="btn btn-ghost" onclick="window.app.closeModals()">Close</button>
        </div>
      `}r.classList.add("active")}openAddServerModal(){this.closeModals();let e=document.getElementById("modal-srv-title"),t=document.getElementById("modal-srv-template-banner"),a=document.getElementById("modal-srv-name"),n=document.getElementById("modal-srv-transport"),o=document.getElementById("modal-srv-command"),r=document.getElementById("modal-srv-url"),s=document.getElementById("modal-srv-ft"),l=document.getElementById("modal-srv-cd"),d=document.getElementById("modal-srv-autorestart"),g=document.getElementById("modal-srv-maxrestarts");if(e)e.textContent="Add Upstream MCP Server";if(t)t.style.display="flex";if(a)a.value="",a.disabled=!1;if(n)n.value="stdio";if(o)o.value="";if(r)r.value="";let u=document.getElementById("modal-group-cmd"),m=document.getElementById("modal-group-url");if(u)u.style.display="block";if(m)m.style.display="none";if(s)s.value="3";if(l)l.value="30000";if(d)d.value="true";if(g)g.value="5";let v=document.getElementById("modal-add-server");if(v)v.classList.add("active")}openEditServerModal(e){this.closeModals();let t=p.getState(),a=t.config.mcpServers?.[e];if(!a){alert(`Server '${e}' not found in configuration.`);return}let n=document.getElementById("modal-srv-title"),o=document.getElementById("modal-srv-template-banner"),r=document.getElementById("modal-srv-name"),s=document.getElementById("modal-srv-transport"),l=document.getElementById("modal-srv-command"),d=document.getElementById("modal-srv-url"),g=document.getElementById("modal-srv-ft"),u=document.getElementById("modal-srv-cd"),m=document.getElementById("modal-srv-autorestart"),v=document.getElementById("modal-srv-maxrestarts");if(n)n.textContent=`Edit Server '${e}'`;if(o)o.style.display="none";if(r)r.value=e,r.disabled=!0;let c=!!a.command;if(s)s.value=c?"stdio":"http";let y=document.getElementById("modal-group-cmd"),b=document.getElementById("modal-group-url");if(y)y.style.display=c?"block":"none";if(b)b.style.display=c?"none":"block";if(l)l.value=c?`${a.command} ${(a.args||[]).join(" ")}`.trim():"";if(d)d.value=a.url||"";let f=a.resilience||t.config.resilience;if(g)g.value=String(f?.failureThreshold??3);if(u)u.value=String(f?.cooldownMs??30000);if(m)m.value=f?.autoRestart===!1?"false":"true";if(v)v.value=String(f?.maxRestarts??5);let x=document.getElementById("modal-add-server");if(x)x.classList.add("active")}async submitAddServer(){let e=document.getElementById("modal-srv-name"),t=e?.value.trim(),a=document.getElementById("modal-srv-transport")?.value;if(!t){alert("Server name is required");return}if(e&&!e.disabled){if((p.getState().config.mcpServers||{})[t]){if(!confirm(`Server '${t}' already exists in configuration. Do you want to overwrite it?`))return}}let n={};if(a==="stdio"){let u=(document.getElementById("modal-srv-command")?.value.trim()).split(/\s+/).filter(Boolean);if(u.length===0){alert("Command is required");return}n.command=u[0],n.args=u.slice(1)}else{let g=document.getElementById("modal-srv-url")?.value.trim();if(!g){alert("URL is required");return}n.url=g}let o=document.getElementById("modal-srv-ft")?.value.trim(),r=document.getElementById("modal-srv-cd")?.value.trim(),s=document.getElementById("modal-srv-autorestart")?.value,l=document.getElementById("modal-srv-maxrestarts")?.value.trim();if(o||r||s||l)n.resilience={failureThreshold:o?Number(o):3,cooldownMs:r?Number(r):30000,autoRestart:s!=="false",maxRestarts:l?Number(l):5};let d=await h.upsertServer(t,n);if(d.ok)this.closeModals(),await this.refreshData();else alert(`Failed to save server: ${d.error}`)}openTemplateCatalog(){this.closeModals();let e=document.getElementById("modal-templates");if(e)e.classList.add("active");this.renderTemplateGrid()}setTemplateCategory(e){this.activeTemplateCategory=e,document.querySelectorAll(".tmpl-cat-btn").forEach((t)=>{if(t.getAttribute("data-category")===e)t.classList.add("active"),t.style.background="var(--surface-elevated)",t.style.color="var(--amber-400)";else t.classList.remove("active"),t.style.background="var(--surface-card)",t.style.color="var(--text-main)"}),this.renderTemplateGrid()}filterTemplates(e){this.activeTemplateFilter=e.toLowerCase().trim(),this.renderTemplateGrid()}renderTemplateGrid(){let e=document.getElementById("tmpl-grid");if(!e)return;let t=D.filter((o)=>{let r=this.activeTemplateCategory==="all"||o.category===this.activeTemplateCategory,s=!this.activeTemplateFilter||o.name.toLowerCase().includes(this.activeTemplateFilter)||o.id.toLowerCase().includes(this.activeTemplateFilter)||o.description.toLowerCase().includes(this.activeTemplateFilter)||o.command.toLowerCase().includes(this.activeTemplateFilter)||o.envFields.some((l)=>l.key.toLowerCase().includes(this.activeTemplateFilter));return r&&s});if(t.length===0){e.innerHTML=`
        <div style="grid-column: span 2; padding: 32px; text-align: center; color: var(--text-dim);">
          No matching MCP server templates found.
        </div>
      `;return}let n=p.getState().config.mcpServers||{};e.innerHTML=t.map((o)=>{let r=!!n[o.id],s=`${o.command} ${o.defaultArgs.join(" ")}`;return`
        <div class="bento-card" style="display: flex; flex-direction: column; justify-content: space-between; padding: 14px; background: var(--surface); border: 1px solid var(--border); min-width: 0; transition: transform 0.15s, border-color 0.15s;">
          <div>
            <div style="display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 6px;">
              <div style="display: flex; align-items: center; gap: 8px; min-width: 0;">
                <span style="font-weight: 700; font-size: 13.5px; color: var(--text-main); white-space: nowrap; overflow: hidden; text-overflow: ellipsis;">${i(o.name)}</span>
                <span class="brand-badge" style="font-size: 9.5px; padding: 1px 6px; flex-shrink: 0;">${i(o.badge)}</span>
              </div>
              ${r?'<span style="font-size: 10px; color: var(--green-400); font-weight: 600; flex-shrink: 0;">CONNECTED</span>':""}
            </div>
            <div style="font-size: 11.5px; color: var(--text-muted); line-height: 1.4; margin-bottom: 8px;">
              ${i(o.description)}
            </div>
            <div style="font-family: var(--ff-mono); font-size: 10.5px; color: var(--text-dim); background: var(--surface-card); padding: 5px 8px; border-radius: var(--radius-sm); border: 1px solid var(--border-subtle); overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">
              <code>${i(s)}</code>
            </div>
            ${o.envFields.length>0?`
              <div style="font-size: 10.5px; color: var(--amber-400); margin-top: 6px; display: flex; align-items: center; gap: 4px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">
                <span>⚡ Needs:</span>
                <code style="overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">${o.envFields.map((l)=>i(l.key)).join(", ")}</code>
              </div>
            `:""}
          </div>

          <div style="display: flex; justify-content: flex-end; margin-top: 12px; gap: 6px;">
            <button class="btn btn-primary" style="font-size: 11.5px; padding: 4px 10px;" onclick="window.app.selectTemplate('${i(o.id)}')">
              ${r?"Configure Another":"✨ 1-Click Setup"}
            </button>
          </div>
        </div>
      `}).join("")}selectTemplate(e){let t=D.find((d)=>d.id===e);if(!t)return;this.selectedTemplate=t,this.closeModals();let a=document.getElementById("modal-configure-template");if(a)a.classList.add("active");let n=document.getElementById("cfg-tmpl-title"),o=document.getElementById("cfg-tmpl-desc"),r=document.getElementById("cfg-tmpl-form");if(n)n.textContent=`Configure ${t.name} Server`;if(o)o.textContent=t.description;let s=p.getState().config.mcpServers||{},l=t.id;if(s[l]){let d=2;while(s[`${t.id}-${d}`])d++;l=`${t.id}-${d}`}if(r){let d="";if(t.envFields.length>0)d=`
          <div style="margin-top: 14px; margin-bottom: 6px; font-weight: 700; font-size: 11px; text-transform: uppercase; color: var(--amber-400); letter-spacing: 0.5px;">
            Environment Variables &amp; API Keys
          </div>
          ${t.envFields.map((g)=>`
            <div class="form-group">
              <label class="form-label">${i(g.label)} ${g.required?'<span style="color: var(--red-400);">*</span>':"(Optional)"}</label>
              <input type="password" class="form-input tmpl-env-input" data-key="${i(g.key)}" placeholder="${i(g.placeholder||"")}">
              ${g.description?`<div style="font-size: 10.5px; color: var(--text-dim); margin-top: 3px;">${i(g.description)}</div>`:""}
            </div>
          `).join("")}
        `;r.innerHTML=`
        <div class="form-group">
          <label class="form-label">Server Identifier (Name)</label>
          <input type="text" class="form-input" id="cfg-srv-id" value="${i(l)}">
          <div style="font-size: 10.5px; color: var(--text-dim); margin-top: 3px;">Must be unique across all configured servers.</div>
        </div>
        <div class="form-group">
          <label class="form-label">Command Line Arguments</label>
          <input type="text" class="form-input" id="cfg-srv-args" value="${i(t.defaultArgs.join(" "))}" placeholder="${i(t.argsPlaceholder||"")}">
          <div style="font-size: 10.5px; color: var(--text-dim); margin-top: 3px;">Executable: <code>${i(t.command)}</code></div>
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
      `}}async submitTemplateServer(){if(!this.selectedTemplate)return;let e=this.selectedTemplate,t=document.getElementById("cfg-srv-id")?.value.trim(),a=document.getElementById("cfg-srv-args")?.value.trim();if(!t){alert("Server identifier is required");return}if((p.getState().config.mcpServers||{})[t]){if(!confirm(`Server '${t}' already exists. Do you want to overwrite its configuration?`))return}let r=a?a.split(/\s+/).filter(Boolean):[],s={},l=document.querySelectorAll(".tmpl-env-input");for(let y of Array.from(l)){let b=y.getAttribute("data-key"),f=y.value.trim(),x=e.envFields.find((T)=>T.key===b);if(x?.required&&!f){alert(`Required field '${x.label}' is missing.`);return}if(b&&f)s[b]=f}let d={command:e.command,args:r};if(Object.keys(s).length>0)d.env=s;let g=document.getElementById("cfg-srv-ft")?.value.trim(),u=document.getElementById("cfg-srv-cd")?.value.trim(),m=document.getElementById("cfg-srv-autorestart")?.value,v=document.getElementById("cfg-srv-maxrestarts")?.value.trim();if(g||u||m||v)d.resilience={failureThreshold:g?Number(g):3,cooldownMs:u?Number(u):30000,autoRestart:m!=="false",maxRestarts:v?Number(v):5};let c=await h.upsertServer(t,d);if(c.ok)this.closeModals(),await this.refreshData();else alert(`Failed to save server: ${c.error}`)}async openImportModal(){this.closeModals();let e=document.getElementById("modal-import");if(e)e.classList.add("active");let t=document.getElementById("modal-eco-list");if(!t)return;t.innerHTML='<div style="color: var(--text-dim); padding: 12px; text-align: center;">Scanning IDE configs...</div>';try{let a=await h.getEcosystemSources();if(a.sources&&a.sources.length>0)t.innerHTML=a.sources.map((n)=>`
          <label style="display: flex; align-items: center; gap: 10px; background: var(--surface); padding: 10px 14px; border-radius: var(--radius-sm); border: 1px solid var(--border); cursor: pointer;">
            <input type="checkbox" class="eco-checkbox" value="${n.path}" checked>
            <div>
              <div style="font-weight: 600; color: var(--text-main);">${n.name}</div>
              <div style="font-size: 11px; color: var(--text-dim);">${n.server_count} servers (${n.servers.join(", ")})</div>
            </div>
          </label>
        `).join("");else t.innerHTML='<div style="color: var(--text-dim); padding: 12px; text-align: center;">No external MCP configuration files found on this system.</div>'}catch{t.innerHTML='<div style="color: var(--red-400); padding: 12px; text-align: center;">Failed to scan ecosystem sources.</div>'}}async submitImport(){let e=document.querySelectorAll(".eco-checkbox:checked");if(e.length===0){alert("No sources selected");return}for(let t of Array.from(e))await h.importConfig(t.value,!1);this.closeModals(),await this.refreshData()}async refreshClients(){try{let e=await h.getClients();if(e.ok&&Array.isArray(e.clients))p.setState({clients:e.clients})}catch(e){console.error("Failed to scan clients:",e)}}setClientCategoryFilter(e){p.setState({clientFilterCategory:e})}setClientSearchQuery(e){p.setState({clientSearchQuery:e})}async attachClient(e,t){let a=t;if(!a){let n=document.getElementById(`client-prof-${e}`)||document.getElementById(`overview-client-prof-${e}`);if(n)a=n.value||void 0;else a=p.getState().activeProfile||void 0}this.openClientAttachModal(e,a)}openClientAttachModal(e,t){this.closeModals(),this.pendingClientId=e;let a=p.getState().config,n=a.mcpHttpServer,o=n?`http://${n.bind==="0.0.0.0"||n.bind==="::"?"127.0.0.1":n.bind||"127.0.0.1"}:${n.port||9191}/mcp`:"",r=document.getElementById("modal-client-title"),s=document.getElementById("modal-client-transport"),l=document.getElementById("modal-client-url"),d=document.getElementById("modal-client-profile"),g=document.getElementById("modal-client-profile-wrap");if(r)r.textContent=`Connect ${e}`;if(s)s.value="stdio";if(l)l.value=o;if(d){let u=Object.keys(a.profiles||{});if(d.innerHTML=`<option value="">All Tools (Default)</option>${u.map((m)=>`<option value="${i(m)}">Profile: ${i(m)}</option>`).join("")}`,d.value=t||"",g)g.style.display=u.length>0?"block":"none"}this.updateClientTransportForm(),document.getElementById("modal-client-attach")?.classList.add("active")}updateClientTransportForm(){let t=(document.getElementById("modal-client-transport")?.value||"stdio")==="http",a=document.getElementById("modal-client-http-group"),n=document.getElementById("modal-client-http-status"),o=!!p.getState().config.mcpHttpServer;if(a)a.style.display=t?"block":"none";if(n)n.textContent=o?"Uses the daemon Streamable HTTP endpoint. The daemon must already be running.":"HTTP is unavailable until mcpHttpServer is configured and the daemon is restarted.",n.style.color=o?"var(--text-dim)":"var(--amber-300)"}async submitClientAttach(){let e=this.pendingClientId;if(!e)return;let t=document.getElementById("modal-client-transport")?.value||"stdio",a=document.getElementById("modal-client-profile")?.value||void 0,n=document.getElementById("modal-client-url")?.value.trim()||void 0;if(t==="http"&&!p.getState().config.mcpHttpServer&&!n){alert("Configure mcpHttpServer and restart the daemon, or provide an explicit HTTP endpoint.");return}if(t==="http"&&!n){alert("Enter an HTTP endpoint URL.");return}let o=await h.attachClient(e,a,t,n);if(!o.ok)alert(`Failed to attach client: ${o.error||o.message||"Unknown error"}`);else this.pendingClientId=null,this.closeModals(),await this.refreshData()}async detachClient(e){if(!confirm("Disconnect Warmplane from this client?"))return;let t=await h.detachClient(e);if(!t.ok)alert(`Failed to detach client: ${t.error||t.message||"Unknown error"}`);else await this.refreshData()}handleAliasTargetInput(e){let t=document.getElementById("alias-suggestions-dropdown");if(!t)return;let a=(e||"").trim().toLowerCase();if(a.length<2){t.style.display="none";return}let o=p.getState().capabilities.filter((r)=>r.id.toLowerCase().includes(a)||r.summary&&r.summary.toLowerCase().includes(a)||r.description&&r.description.toLowerCase().includes(a)||r.server&&r.server.toLowerCase().includes(a)).slice(0,8);if(o.length===0){t.style.display="none";return}t.innerHTML=o.map((r)=>`
      <div style="padding: 8px 12px; cursor: pointer; border-bottom: 1px solid var(--border-subtle); display: flex; justify-content: space-between; align-items: center; transition: background 0.1s;"
           onmouseover="this.style.background='var(--surface-hover)'"
           onmouseout="this.style.background='transparent'"
           onmousedown="window.app.selectAliasSuggestion('${i(r.id)}')">
        <div>
          <div style="font-weight: 700; color: var(--text-main);">${i(r.id)}</div>
          <div style="font-size: 10.5px; color: var(--text-dim); margin-top: 2px;">${i(r.summary||r.description||"")}</div>
        </div>
        <span style="font-size: 10px; color: var(--cyan-400);">${i(r.server||"local")}</span>
      </div>
    `).join(""),t.style.display="block"}selectAliasSuggestion(e){let t=document.getElementById("alias-target");if(t)t.value=e;this.hideAliasDropdown()}hideAliasDropdown(){let e=document.getElementById("alias-suggestions-dropdown");if(e)e.style.display="none"}startEditAlias(e,t,a,n="",o=!1){let r=document.getElementById("alias-kind"),s=document.getElementById("alias-name"),l=document.getElementById("alias-target"),d=document.getElementById("alias-summary"),g=document.getElementById("alias-passthrough"),u=document.getElementById("alias-passthrough-container"),m=document.getElementById("alias-form-title"),v=document.getElementById("alias-save-btn"),c=document.getElementById("alias-cancel-btn");if(r)r.value=e;if(s)s.value=t;if(l)l.value=a;if(d)d.value=n;if(g)g.checked=o;if(u)u.style.display=e==="tool"?"flex":"none";if(m)m.innerHTML=`✏️ EDIT ALIAS: <span style="color: var(--amber-400); font-family: var(--ff-mono);">${t}</span>`;if(v)v.textContent="✓ Update";if(c)c.style.display="inline-block";s?.focus(),s?.scrollIntoView({behavior:"smooth",block:"nearest"})}resetAliasForm(){let e=document.getElementById("alias-kind"),t=document.getElementById("alias-name"),a=document.getElementById("alias-target"),n=document.getElementById("alias-summary"),o=document.getElementById("alias-passthrough"),r=document.getElementById("alias-passthrough-container"),s=document.getElementById("alias-form-title"),l=document.getElementById("alias-save-btn"),d=document.getElementById("alias-cancel-btn");if(e)e.value="tool";if(t)t.value="";if(a)a.value="";if(n)n.value="";if(o)o.checked=!1;if(r)r.style.display="flex";if(s)s.textContent="Create New Alias";if(l)l.textContent="+ Save";if(d)d.style.display="none"}async createAlias(){let e=document.getElementById("alias-kind")?.value,t=document.getElementById("alias-name")?.value.trim(),a=document.getElementById("alias-target")?.value.trim(),n=document.getElementById("alias-summary")?.value.trim()||void 0,o=document.getElementById("alias-passthrough")?.checked||!1;if(!t||!a){alert("Please provide both alias name and canonical target");return}if(e==="tool"&&o&&!/^[a-zA-Z0-9_-]{1,64}$/.test(t)){let r=t.replace(/[^a-zA-Z0-9_-]/g,"_").substring(0,64);if(!confirm(`MCP tool names must match ^[a-zA-Z0-9_-]{1,64}$.

'${t}' will be exported to MCP clients as '${r}'.

Do you want to proceed?`))return}await h.updateAlias(e,t,a,n,void 0,o),this.resetAliasForm(),await this.refreshData()}async deleteAlias(e,t){await h.updateAlias(e,t,void 0),this.resetAliasForm(),await this.refreshData()}async reloadFromDisk(){try{let e=await h.reloadConfig();if(e.ok){let t="Hot-reload completed successfully!";if(e.mounted&&e.mounted.length>0)t+=`
Mounted: ${e.mounted.join(", ")}`;if(e.unmounted&&e.unmounted.length>0)t+=`
Unmounted: ${e.unmounted.join(", ")}`;if(e.warnings&&e.warnings.length>0)t+=`
Warnings:
${e.warnings.join(`
`)}`;alert(t)}else alert(`Hot-reload failed: ${e.error||"Unknown error"}`)}catch(e){alert(`Error reaching daemon: ${e.message}`)}await this.refreshData()}renderTopProfileSelector(){let e=document.getElementById("top-profile-selector");if(!e)return;let t=p.getState(),a=t.config.profiles||{},n=Object.keys(a),o=t.activeProfile,r='<option value="">All Servers (Unrestricted)</option>';for(let s of n){let l=o===s?"selected":"";r+=`<option value="${i(s)}" ${l}>Profile: ${i(s)}</option>`}e.innerHTML=r}async setActiveProfile(e){p.setState({activeProfile:e||null}),await this.refreshData()}openAddProfileModal(){let e=document.getElementById("modal-prof-title");if(e)e.textContent="Create Server Constellation Profile";let t=document.getElementById("modal-prof-name"),a=document.getElementById("modal-prof-desc"),n=document.getElementById("modal-prof-mode");if(t)t.value="",t.disabled=!1;if(a)a.value="";if(n)n.value="create";let o=document.getElementById("modal-prof-allow"),r=document.getElementById("modal-prof-deny"),s=document.getElementById("modal-prof-hitl"),l=document.getElementById("modal-prof-redact");if(o)o.value="";if(r)r.value="";if(s)s.value="";if(l)l.value="";this.renderProfileServerCheckboxes([]);let d=document.getElementById("modal-add-profile");if(d)d.classList.add("active")}openEditProfileModal(e){let a=p.getState().config.profiles?.[e];if(!a)return;let n=document.getElementById("modal-prof-title");if(n)n.textContent=`Edit Profile: ${e}`;let o=document.getElementById("modal-prof-name"),r=document.getElementById("modal-prof-desc"),s=document.getElementById("modal-prof-mode");if(o)o.value=e,o.disabled=!0;if(r)r.value=a.description||"";if(s)s.value="edit";let l=document.getElementById("modal-prof-allow"),d=document.getElementById("modal-prof-deny"),g=document.getElementById("modal-prof-hitl"),u=document.getElementById("modal-prof-redact"),m=a.policy;if(l)l.value=(m?.allow||[]).join(", ");if(d)d.value=(m?.deny||[]).join(", ");if(g)g.value=(m?.require_approval||m?.requireApproval||[]).join(", ");if(u)u.value=(m?.redact_keys||m?.redactKeys||[]).join(", ");this.renderProfileServerCheckboxes(a.servers||[]);let v=document.getElementById("modal-add-profile");if(v)v.classList.add("active")}renderProfileServerCheckboxes(e){let t=document.getElementById("modal-prof-servers-list");if(!t)return;let a=p.getState(),n=Object.keys(a.config.mcpServers||{});if(n.length===0){t.innerHTML='<div style="font-size: 11.5px; color: var(--text-dim);">No MCP servers configured yet. Add servers first.</div>';return}t.innerHTML=n.map((o)=>{let r=e.includes(o)?"checked":"";return`
        <label style="display: flex; align-items: center; gap: 8px; font-size: 12px; cursor: pointer; padding: 4px 6px; border-radius: var(--radius-sm); transition: background 0.15s;" onmouseover="this.style.background='var(--surface-hover)'" onmouseout="this.style.background='transparent'">
          <input type="checkbox" class="prof-server-checkbox" value="${i(o)}" ${r} style="accent-color: var(--amber-400);">
          <span style="font-family: var(--ff-mono); font-weight: 600; color: var(--text-main);">${i(o)}</span>
        </label>
      `}).join("")}async saveProfile(){let e=document.getElementById("modal-prof-name"),t=document.getElementById("modal-prof-desc"),a=e?.value.trim(),n=t?.value.trim();if(!a){alert("Please enter a profile name");return}let o=document.querySelectorAll(".prof-server-checkbox:checked"),r=[];if(o.forEach((f)=>{r.push(f.value)}),r.length===0){alert("Please select at least one server to include in this constellation");return}let s=(f)=>{if(!f)return[];return f.split(",").map((x)=>x.trim()).filter((x)=>x.length>0)},l=document.getElementById("modal-prof-allow"),d=document.getElementById("modal-prof-deny"),g=document.getElementById("modal-prof-hitl"),u=document.getElementById("modal-prof-redact"),m=s(l?.value),v=s(d?.value),c=s(g?.value),y=s(u?.value),b=void 0;if(m.length>0||v.length>0||c.length>0||y.length>0)b={allow:m,deny:v,requireApproval:c,redactKeys:y};try{let f=await h.upsertProfile(a,r,n||void 0,b);if(f.ok)this.closeModals(),await this.refreshData();else alert(`Failed to save profile: ${f.error||"Unknown error"}`)}catch(f){alert(`Error saving profile: ${f.message}`)}}async deleteProfile(e){if(!confirm(`Are you sure you want to delete profile '${e}'?`))return;try{let t=await h.deleteProfile(e);if(t.ok){if(p.getState().activeProfile===e)p.setState({activeProfile:null});await this.refreshData()}else alert(`Failed to delete profile: ${t.error||"Unknown error"}`)}catch(t){alert(`Error deleting profile: ${t.message}`)}}async toggleServerInProfile(e,t,a){let o=p.getState().config.profiles?.[e];if(!o)return;let r=[...o.servers||[]];if(a){if(!r.includes(t))r.push(t)}else if(r=r.filter((s)=>s!==t),r.length===0){alert("A profile must contain at least one server. To remove the profile, delete it in the Profiles tab.");return}try{let s=await h.upsertProfile(e,r,o.description,o.policy);if(s.ok)await this.refreshData();else alert(`Failed to update profile constellation: ${s.error||"Unknown error"}`)}catch(s){alert(`Error updating profile constellation: ${s.message}`)}}closeModals(){document.querySelectorAll(".modal-backdrop").forEach((e)=>e.classList.remove("active"))}}var ie=new ne;window.app=ie;window.addEventListener("DOMContentLoaded",()=>ie.init());
