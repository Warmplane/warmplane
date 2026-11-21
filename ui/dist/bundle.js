function i(e){if(e===null||e===void 0)return"";return String(e).replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;").replace(/'/g,"&#039;")}class le{state={configPath:"mcp_servers.json",config:{mcpServers:{}},configReady:!1,configError:null,serverStatuses:{},circuitBreakers:[],clients:[],clientFilterCategory:"all",clientSearchQuery:"",secrets:[],clientsCollapsed:!1,capabilities:[],capabilitiesHiddenByPolicy:0,resources:[],resourcesHiddenByPolicy:0,prompts:[],promptsHiddenByPolicy:0,catalogEvents:[],tasks:[],taskError:null,undoEntries:[],undoError:null,undoOffset:0,undoTotal:0,undoLoading:!1,selectedTaskId:null,taskFilterStatus:"all",approvals:[],approvalError:null,auditEvents:[],auditTotal:0,auditFilters:{search:"",status:"all",eventType:"all",serverId:"all",limit:25,offset:0},auditSelectedEvent:null,auditStats:null,auditVerification:null,selectedCapabilityId:null,selectedResourceId:null,selectedPromptId:null,playgroundMode:"tools",playgroundArgs:{},isExecuting:!1,playgroundAsyncTask:!1,activeRequestId:null,isBatchModalOpen:!1,batchSteps:[{id:"step_1",capability_id:"",argsJson:"{}",continue_on_error:!1},{id:"step_2",capability_id:"",argsJson:"{}",continue_on_error:!0}],activeTab:"overview",activeProfile:null,eventLogs:[],executionResult:null,resourceReadResult:null,promptGetResult:null,metrics:{totalCatalogRequests:0,totalEtagHits:0,totalToolCalls:0,totalToolDurationUs:0}};listeners=[];getState(){return this.state}setState(e){this.state={...this.state,...e},this.listeners.forEach((t)=>t(this.state))}subscribe(e){return this.listeners.push(e),()=>{this.listeners=this.listeners.filter((t)=>t!==e)}}addEventLog(e,t,s,r){let o=[{time:new Date().toLocaleTimeString(),method:e,target:t,status:s,latency:r},...this.state.eventLogs].slice(0,50);this.setState({eventLogs:o})}}var p=new le;class de{baseUrl;approvalToken="";trustToken="";constructor(e=""){this.baseUrl=e}setApprovalToken(e){this.approvalToken=e}setTrustToken(e){this.trustToken=e}hasTrustToken(){return!!this.trustToken}headers(e,t=!1,s){let r=new Headers;if(e)r.set("Authorization",`Bearer ${e}`);if(t)r.set("Content-Type","application/json");if(s)r.set("X-Warmplane-Profile",s);return r}approvalHeaders(e=!1,t){return this.headers(this.approvalToken||this.trustToken,e,t)}async listToolPins(e=this.trustToken){return this.requestToolPins(e)}async approveToolPin(e){return this.requestToolPins(this.trustToken,e)}async requestToolPins(e,t){let s=await fetch(`${this.baseUrl}/v1/config/tool-pins${t?"/approve":""}`,{method:t?"POST":"GET",headers:this.headers(e,!!t),body:t?JSON.stringify(t):void 0}),r=await s.json();if(s.ok&&r.ok)return{pins:r.pins};return{error:z(r,"Could not load or approve tool definitions.")}}async getConfig(){return(await fetch(`${this.baseUrl}/v1/config`,{headers:this.headers(this.trustToken||this.approvalToken)})).json()}async listCapabilities(e){let t=this.approvalHeaders(!1,e);return(await fetch(`${this.baseUrl}/v1/capabilities`,{headers:t})).json()}async listResources(e){let t=this.approvalHeaders(!1,e);return(await fetch(`${this.baseUrl}/v1/resources`,{headers:t})).json()}async readResource(e,t){return this.timedPost("/v1/resources/read",e,t)}async timedPost(e,t,s){let r=performance.now(),a=this.approvalHeaders(!0,s),o=await fetch(`${this.baseUrl}${e}`,{method:"POST",headers:a,body:JSON.stringify(t)}),n=performance.now()-r,l=await o.json();return{status:o.status,durationMs:n,data:l}}async listPrompts(e){let t=this.approvalHeaders(!1,e);return(await fetch(`${this.baseUrl}/v1/prompts`,{headers:t})).json()}async getPrompt(e,t){return this.timedPost("/v1/prompts/get",e,t)}async getCatalogEvents(e){let t=e?`?after=${encodeURIComponent(e)}`:"";return(await fetch(`${this.baseUrl}/v1/catalog/events${t}`,{headers:this.approvalHeaders()})).json()}async callCapability(e,t){return this.timedPost("/v1/tools/call",e,t)}async batchCallCapabilities(e,t){return this.timedPost("/v1/tools/batch_call",{steps:e},t)}async cancelOperation(e){return(await fetch(`${this.baseUrl}/v1/operations/${encodeURIComponent(e)}/cancel`,{method:"POST"})).json()}async completeArgument(e){return(await fetch(`${this.baseUrl}/v1/completion/complete`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify(e)})).json()}async upsertServer(e,t,s=!1){return(await fetch(`${this.baseUrl}/v1/config/servers`,{method:"POST",headers:this.headers(this.trustToken,!0),body:JSON.stringify({name:e,server:t,preserve_unedited:s})})).json()}async deleteServer(e){return(await fetch(`${this.baseUrl}/v1/config/servers/${encodeURIComponent(e)}`,{method:"DELETE",headers:this.headers(this.trustToken)})).json()}async restartServer(e){return(await fetch(`${this.baseUrl}/v1/config/servers/${encodeURIComponent(e)}/restart`,{method:"POST",headers:this.headers(this.trustToken)})).json()}async upsertProfile(e,t,s,r){return(await fetch(`${this.baseUrl}/v1/config/profiles`,{method:"POST",headers:this.headers(this.trustToken,!0),body:JSON.stringify({name:e,servers:t,description:s,policy:r})})).json()}async deleteProfile(e){return(await fetch(`${this.baseUrl}/v1/config/profiles/${encodeURIComponent(e)}`,{method:"DELETE",headers:this.headers(this.trustToken)})).json()}async getEcosystemSources(){return(await fetch(`${this.baseUrl}/v1/config/ecosystem`,{headers:this.headers(this.trustToken)})).json()}async importConfig(e,t=!1){return(await fetch(`${this.baseUrl}/v1/config/import`,{method:"POST",headers:this.headers(this.trustToken,!0),body:JSON.stringify({source_path:e,overwrite:t})})).json()}async savePolicy(e){let t={allow:e.allow||[],deny:e.deny||[],redactKeys:e.redact_keys||e.redactKeys||[],resultSanitization:e.resultSanitization,requireApproval:e.require_approval||e.requireApproval||[],approvalTimeoutSecs:e.approvalTimeoutSecs||e.approval_timeout_secs||300,holdWindowSecs:e.holdWindowSecs,webhook:e.webhook};return(await fetch(`${this.baseUrl}/v1/config/policy`,{method:"POST",headers:this.headers(this.trustToken,!0),body:JSON.stringify(t)})).json()}async listTasks(e){return(await fetch(`${this.baseUrl}/v1/tasks`,{headers:this.approvalHeaders(!1,e)})).json()}async listUndo(e=0,t=50){return(await fetch(`${this.baseUrl}/v1/undo?offset=${e}&limit=${t}`)).json()}async testUndoContract(e){return(await fetch(`${this.baseUrl}/v1/undo/test`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({alias:e})})).json()}async executeUndo(e){return(await fetch(`${this.baseUrl}/v1/undo/${encodeURIComponent(e)}/execute`,{method:"POST",headers:{"Content-Type":"application/json"},body:"{}"})).json()}async rollbackWorkItem(e){return(await fetch(`${this.baseUrl}/v1/undo/work-items/${encodeURIComponent(e)}/rollback`,{method:"POST",headers:{"Content-Type":"application/json"},body:"{}"})).json()}async getTask(e,t){return(await fetch(`${this.baseUrl}/v1/tasks/${encodeURIComponent(e)}`,{headers:this.approvalHeaders(!1,t)})).json()}async updateTask(e,t,s){return(await fetch(`${this.baseUrl}/v1/tasks/${encodeURIComponent(e)}/update`,{method:"POST",headers:this.approvalHeaders(!0,s),body:JSON.stringify({inputResponses:t})})).json()}async cancelTask(e,t,s){return(await fetch(`${this.baseUrl}/v1/tasks/${encodeURIComponent(e)}/cancel`,{method:"POST",headers:this.approvalHeaders(!0,s),body:JSON.stringify({reason:t})})).json()}async listApprovals(){return(await fetch(`${this.baseUrl}/v1/approvals`,{headers:this.approvalHeaders()})).json()}async approveTicket(e,t){return(await fetch(`${this.baseUrl}/v1/approvals/${encodeURIComponent(e)}/approve`,{method:"POST",headers:this.approvalHeaders(!0),body:JSON.stringify({modified_args:t})})).json()}async rejectTicket(e,t){return(await fetch(`${this.baseUrl}/v1/approvals/${encodeURIComponent(e)}/reject`,{method:"POST",headers:this.approvalHeaders(!0),body:JSON.stringify({reason:t})})).json()}async updateAlias(e,t,s,r,a,o,n){return(await fetch(`${this.baseUrl}/v1/config/alias`,{method:"POST",headers:this.headers(this.trustToken,!0),body:JSON.stringify({kind:e,alias:t,target:s,summary:r,description:a,passthrough:o,reversibility:n})})).json()}async reloadConfig(){return(await fetch(`${this.baseUrl}/v1/config/reload`,{method:"POST",headers:this.headers(this.trustToken)})).json()}async listAuditEvents(e){let t=new URLSearchParams;if(e?.actor_id)t.set("actor_id",e.actor_id);if(e?.server_id&&e.server_id!=="all")t.set("server_id",e.server_id);if(e?.capability_id)t.set("capability_id",e.capability_id);if(e?.event_type&&e.event_type!=="all")t.set("event_type",e.event_type);if(e?.status&&e.status!=="all")t.set("status",e.status);if(e?.trace_id)t.set("trace_id",e.trace_id);if(e?.request_id)t.set("request_id",e.request_id);if(e?.search)t.set("search",e.search);if(e?.limit)t.set("limit",String(e.limit));if(e?.offset!==void 0)t.set("offset",String(e.offset));let s=t.toString();return(await fetch(`${this.baseUrl}/v1/audit/events${s?`?${s}`:""}`)).json()}getAuditExportUrl(e,t="csv"){let s=new URLSearchParams;if(s.set("format",t),e?.actor_id)s.set("actor_id",e.actor_id);if(e?.server_id&&e.server_id!=="all")s.set("server_id",e.server_id);if(e?.capability_id)s.set("capability_id",e.capability_id);if(e?.event_type&&e.event_type!=="all")s.set("event_type",e.event_type);if(e?.status&&e.status!=="all")s.set("status",e.status);if(e?.trace_id)s.set("trace_id",e.trace_id);if(e?.request_id)s.set("request_id",e.request_id);if(e?.search)s.set("search",e.search);return`${this.baseUrl}/v1/audit/export?${s.toString()}`}async verifyAuditChain(){return(await fetch(`${this.baseUrl}/v1/audit/verify`)).json()}async getAuditStats(){return(await fetch(`${this.baseUrl}/v1/audit/stats`)).json()}async getClients(){return(await fetch(`${this.baseUrl}/v1/clients`)).json()}async attachClient(e,t,s="stdio",r){return(await fetch(`${this.baseUrl}/v1/clients/${encodeURIComponent(e)}/attach`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({profile:t||void 0,transport:s,http_url:r||void 0})})).json()}async detachClient(e){return(await fetch(`${this.baseUrl}/v1/clients/${encodeURIComponent(e)}/detach`,{method:"POST",headers:{"Content-Type":"application/json"}})).json()}async testWebhook(e,t){return(await fetch(`${this.baseUrl}/v1/webhooks/test`,{method:"POST",headers:this.headers(this.trustToken,!0),body:JSON.stringify({url:e||void 0,format:t||void 0})})).json()}async getSecrets(){return(await fetch(`${this.baseUrl}/v1/secrets`)).json()}async saveSecret(e,t,s){return(await fetch(`${this.baseUrl}/v1/secrets`,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({key:e,value:t,service:s})})).json()}async deleteSecret(e){return(await fetch(`${this.baseUrl}/v1/secrets/${encodeURIComponent(e)}`,{method:"DELETE"})).json()}}function z(e,t){return e.message||(typeof e.error==="string"?e.error:e.error?.message)||t}var y=new de;var V="approval.requested",Q="tool_pin.quarantined";function ce(e){return`
    <form id="result-sanitization-form" data-policy-key="${i(JSON.stringify(e??null))}" class="bento-card col-12 trust-settings" onsubmit="window.app.saveResultSanitization(event)">
      <h2 class="stat-label">Result sanitization</h2>
      <label class="trust-checkbox"><input name="enabled" type="checkbox" ${e?"checked":""}> Inspect upstream results before delivery</label>
      <div class="trust-fields">
        <div class="form-group">
          <label class="form-label" for="result-fields">Structured field names · one per line</label>
          <textarea class="form-textarea" id="result-fields" name="fields" rows="3">${i((e?.fieldNames??[]).join(`
`))}</textarea>
        </div>
        <div class="form-group">
          <label class="form-label" for="result-patterns">Regular expressions · JSON array</label>
          <textarea class="form-textarea" id="result-patterns" name="patterns" rows="3">${i(JSON.stringify(e?.patterns??[],null,2))}</textarea>
        </div>
      </div>
      <div class="form-group">
        <label class="form-label" for="result-exceptions">Opaque content exceptions · JSON array</label>
        <textarea class="form-textarea" id="result-exceptions" name="exceptions" rows="3" spellcheck="false"
          placeholder='[{"server":"reports","kind":"image","mimeType":"image/png"}]'>${i(JSON.stringify(e?.opaqueExceptions??[],null,2))}</textarea>
        <span class="tool-pin-feedback">Opaque content is withheld unless its server, kind, and MIME type match an exception.</span>
      </div>
      <div class="trust-actions">
        <p class="tool-pin-feedback" role="status"></p>
        <button type="submit" class="btn btn-primary">Save result policy</button>
      </div>
    </form>
  `}function O(e,t,s,r="text"){return`
    <div class="form-group">
      <label class="form-label" for="webhook-${e}">${t}</label>
      <input class="form-input" id="webhook-${e}" name="${e}" type="${r}" value="${i(s??"")}" ${r==="password"?'autocomplete="new-password"':""}>
    </div>
  `}function pe(e,t){let s=typeof e==="string"?{url:e}:e,r=[["generic","Generic JSON"],["slack","Slack"],["teams","Microsoft Teams"],["discord","Discord"]];return`
    <form id="chatops-settings-form" data-policy-key="${i(JSON.stringify([e??null,t]))}" class="bento-card col-12 trust-settings" onsubmit="window.app.saveWebhookConfig(event)">
      <h2 class="stat-label">ChatOps and outbound webhooks</h2>
      <div class="trust-fields">
        ${O("url","Destination URL",s?.url,"url")}
        <div class="form-group">
          <label class="form-label" for="webhook-format">Format</label>
          <select class="form-input" id="webhook-format" name="format">
            ${r.map(([a,o])=>`<option value="${a}" ${(s?.format??"generic")===a?"selected":""}>${o}</option>`).join("")}
          </select>
        </div>
        ${O("secret","Webhook signing secret",s?.secret,"password")}
        ${O("secretEnv","Webhook signing secret environment variable",s?.secretEnv??s?.secret_env)}
      </div>
      <fieldset>
        <legend>Signed actions</legend>
        <label class="trust-checkbox"><input name="approvals" type="checkbox" ${s?.events?.includes(V)?"checked":""}> Send signed approval cards</label>
        <label class="trust-checkbox"><input name="quarantine" type="checkbox" ${s?.events?.includes(Q)?"checked":""}> Send tool quarantine notifications</label>
        <div class="trust-fields">
          ${O("callbackUrl","Callback URL ending in /v1/webhooks/actions",s?.callbackUrl??s?.callback_url,"url")}
          ${O("actionRole","Delegated action role",s?.actionRole)}
          ${O("approvalTenant",t?"Approval tenant · required with RBAC":"Approval tenant",s?.approvalTenant)}
          ${O("slackSigningSecret","Slack app signing secret",s?.slackSigningSecret,"password")}
          ${O("slackSigningSecretEnv","Slack signing secret environment variable",s?.slackSigningSecretEnv)}
        </div>
      </fieldset>
      <div class="trust-actions">
        <p class="tool-pin-feedback" role="status"></p>
        <button type="button" class="btn btn-ghost" onclick="window.app.testWebhook()" ${s?.url?"":"disabled"}>Test saved destination</button>
        <button type="submit" class="btn btn-primary">Save webhook settings</button>
      </div>
    </form>
  `}function K(e,t){return String(e.get(t)??"").trim()}function ue(e){if(!e.has("enabled"))return;let t=K(e,"fields").split(`
`).map((a)=>a.trim()).filter(Boolean),s=JSON.parse(K(e,"patterns")||"[]");if(!Array.isArray(s)||!s.every((a)=>typeof a==="string"))throw Error("Enter regular expressions as a JSON array of strings.");let r=JSON.parse(K(e,"exceptions")||"[]");if(!Array.isArray(r))throw Error("Enter opaque content exceptions as a JSON array.");return{fieldNames:t,patterns:s,opaqueExceptions:r}}function ge(e,t,s){let r=K(e,"url");if(!r){if(e.has("approvals")||e.has("quarantine"))throw Error("Enter a destination URL for the enabled notifications.");return}let a=typeof t==="object"?t:void 0,o=(a?.events??[]).filter((c)=>c!==V&&c!==Q);if(e.has("approvals"))o.push(V);if(e.has("quarantine"))o.push(Q);let n=(c)=>K(e,c)||void 0,l={...a,url:r,events:o,format:n("format")??"generic",callbackUrl:n("callbackUrl"),actionRole:n("actionRole"),approvalTenant:n("approvalTenant"),secret:n("secret"),secretEnv:n("secretEnv"),slackSigningSecret:n("slackSigningSecret"),slackSigningSecretEnv:n("slackSigningSecretEnv"),callback_url:void 0,secret_env:void 0};return Be(l,s),l}function Be(e,t){if(e.events?.includes(V)){if(!e.callbackUrl||new URL(e.callbackUrl).pathname!=="/v1/webhooks/actions")throw Error("Use a callback URL ending in /v1/webhooks/actions.");if(!e.actionRole)throw Error("Enter the delegated action role for signed approvals.");if(t&&!e.approvalTenant)throw Error("Enter the approval tenant for signed approvals under RBAC.");if(e.format==="slack"&&!e.slackSigningSecret&&!e.slackSigningSecretEnv)throw Error("Enter the Slack app signing secret or its environment variable.")}if(e.events?.some((r)=>r===V||r===Q)&&!e.secret&&!e.secretEnv)throw Error("Enter the webhook signing secret or its environment variable.")}function me(){let e=p.getState(),t=e.config.mcpServers||{},s=Object.keys(t),r=s.length,a="";if(s.length===0)a=`
      <div style="grid-column: 1 / -1; padding: 32px; text-align: center; color: var(--text-dim); background: var(--surface-card); border-radius: var(--radius-md); border: 1px dashed var(--border);">
        <div style="font-size: 14px; color: var(--text-main); font-weight: 600; margin-bottom: 6px;">No Upstream MCP Servers Connected</div>
        <div style="font-size: 12px; margin-bottom: 16px;">Initialize connections by adding a server or syncing existing IDE configurations.</div>
        <div style="display: flex; gap: 8px; justify-content: center;">
          <button class="btn btn-primary" onclick="window.app.openTemplateCatalog()">✨ Browse Templates</button>
          <button class="btn btn-ghost" onclick="window.app.openAddServerModal()">+ Add Custom</button>
          <button class="btn btn-ghost" onclick="window.app.openImportModal()">Sync from IDEs</button>
        </div>
      </div>
    `;else a=s.map((S)=>{let P=t[S],D=P.command?"stdio":"http / sse",B=P.command?`${P.command} ${(P.args||[]).join(" ")}`:P.url,M=e.serverStatuses[S]||{status:"connected",protocol_version:"2026-07-28"},U=M.status==="degraded",ee=M.status==="error"||M.status==="disconnected",G=U?"var(--amber-400)":ee?"var(--red-400)":"var(--green-400)";return`
        <div class="bento-card col-4" style="background: var(--surface); border: 1px solid var(--border);">
          <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
            <span style="font-weight: 700; color: var(--text-main); display: flex; align-items: center; gap: 6px;">
              <span style="width: 8px; height: 8px; border-radius: 50%; background: ${G}; display: inline-block;"></span>
              ${i(S)}
            </span>
            <span class="brand-badge">${D}</span>
          </div>
          <div style="font-family: var(--ff-mono); font-size: 11.5px; color: var(--text-dim); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; margin-bottom: 12px;" title="${i(B||"")}">
            ${i(B||"")}
          </div>
          <div style="display: flex; justify-content: space-between; font-size: 11px; color: var(--text-muted); border-top: 1px solid var(--border); padding-top: 8px;">
            <span>Status: <strong style="color: ${G};">${i(M.status)}</strong></span>
            <span>Protocol: ${M.protocol_version}</span>
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
  `:e.eventLogs.map((S)=>`
    <div class="feed-row" style="grid-template-columns: 80px 100px 1fr 100px 80px;">
      <span style="color: var(--text-dim);">${i(S.time)}</span>
      <span style="color: var(--cyan-400); font-weight: 600;">${i(S.method)}</span>
      <span style="color: var(--text-main); font-family: var(--ff-mono);">${i(S.target)}</span>
      <span style="color: var(--green-400);">${i(S.status)}</span>
      <span style="color: var(--amber-300); text-align: right;">${i(S.latency)}</span>
    </div>
  `).join(""),n=e.metrics,{totalCatalogRequests:l,totalEtagHits:c}=n,g=l>0?`${(c/l*100).toFixed(1)}%`:"0.0%",u=l>0?`${c} of ${l} requests served via HTTP 304`:"Waiting for client requests",m=n.totalToolCalls,v=m>0?`${(n.totalToolDurationUs/m/1000).toFixed(1)}ms`:"0.0ms",d=m>0?`${m} tool executions processed`:"Local worker task queues warm",f=Object.keys(e.config.capabilityAliases||{}).length+Object.keys(e.config.resourceAliases||{}).length+Object.keys(e.config.promptAliases||{}).length,x=f>0?`${f*18}B / call`:"0B",b=f>0?`${f} active facade aliases pruning prompt size`:"Configure aliases in Studio to reduce prompt size",h=e.tasks||[],T=h.filter((S)=>S.status==="input_required").length,C=h.filter((S)=>S.status==="working"||S.status==="input_required").length,E=e.clients||[],w=E.filter((S)=>S.is_attached).length,k=E.filter((S)=>S.config_exists&&!S.is_attached).length,A=e.clientsCollapsed,I=w>0?`<span class="brand-badge" style="color: var(--green-400); border-color: rgba(52, 211, 153, 0.3); background: rgba(52, 211, 153, 0.1);">⚡ ${w} Connected</span>`:k>0?`<span class="brand-badge" style="color: var(--amber-300); border-color: rgba(251, 191, 36, 0.3); background: rgba(251, 191, 36, 0.1);">○ ${k} Ready to Connect</span>`:'<span class="brand-badge" style="color: var(--text-dim);">No Apps Detected</span>',R=Object.keys(e.config.profiles||{}),q=e.activeProfile,L=e.clientFilterCategory||"all",j=(e.clientSearchQuery||"").toLowerCase().trim(),N=E.filter((S)=>{if(j){let P=S.name.toLowerCase().includes(j),D=S.id.toLowerCase().includes(j),B=S.category.toLowerCase().includes(j);if(!P&&!D&&!B)return!1}if(L==="connected")return S.is_attached;if(L==="ready")return S.config_exists||S.app_installed;if(L==="ides")return S.category.toLowerCase().includes("ide")||S.category.toLowerCase().includes("extension");if(L==="agents")return S.category.toLowerCase().includes("agent")||S.category.toLowerCase().includes("cli")||S.category.toLowerCase().includes("platform");return!0}),Z=N.length===0?'<div style="padding: 16px; text-align: center; color: var(--text-dim); font-size: 11.5px; grid-column: 1 / -1;">No AI clients match current filter.</div>':N.map((S)=>{let{is_attached:P,config_exists:D,app_installed:B}=S,M="rgba(255, 255, 255, 0.2)",U="Not Found";if(P){M="var(--green-400)";let F=S.attached_transport==="http"?"HTTP":"stdio";U=S.attached_profile?`Connected · ${F} (${S.attached_profile})`:`Connected · ${F} (All Tools)`}else if(D)M="var(--amber-300)",U="Ready to Attach";else if(B)M="var(--cyan-400)",U="Installed";let ee=R.map((F)=>`
      <option value="${i(F)}" ${q===F||S.attached_profile===F?"selected":""}>${i(F)}</option>
    `).join(""),G=P?`<button class="btn btn-ghost" style="padding: 2px 7px; font-size: 10px; color: var(--red-400);" onclick="event.stopPropagation(); window.app.detachClient('${i(S.id)}')">Detach</button>`:D||B?`
        <div style="display: flex; align-items: center; gap: 4px;" onclick="event.stopPropagation();">
          ${R.length>0?`
            <select id="overview-client-prof-${i(S.id)}" class="form-input" style="font-size: 10px; padding: 1px 4px; height: 22px; width: 85px;" title="Select constellation profile">
              <option value="" ${!q?"selected":""}>All Tools</option>
              ${ee}
            </select>
          `:""}
          <button class="btn btn-primary" style="padding: 2px 7px; font-size: 10px;" onclick="window.app.attachClient('${i(S.id)}')">⚡ Connect</button>
        </div>
      `:"";return`
      <div style="background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 8px 12px; display: flex; align-items: center; justify-content: space-between; gap: 8px;">
        <div style="display: flex; align-items: center; gap: 8px; overflow: hidden;">
          <span style="width: 7px; height: 7px; border-radius: 50%; background: ${M}; flex-shrink: 0;"></span>
          <div style="overflow: hidden;">
            <div style="font-weight: 600; font-size: 12px; color: var(--text-main); white-space: nowrap; overflow: hidden; text-overflow: ellipsis;">${i(S.name)}</div>
            <div style="font-size: 10px; color: var(--text-dim);">${i(U)}</div>
          </div>
        </div>
        ${G}
      </div>
    `}).join(""),oe=[{id:"all",label:"All"},{id:"ready",label:"Ready / Installed"},{id:"connected",label:"Connected"},{id:"ides",label:"IDEs"},{id:"agents",label:"Agents & CLIs"}].map((S)=>`<button class="btn btn-ghost" style="padding: 2px 8px; font-size: 10.5px; border-radius: 100px; ${L===S.id?"background: var(--amber-400); color: #000; font-weight: 700;":"background: var(--surface); color: var(--text-muted);"}" onclick="event.stopPropagation(); window.app.setClientCategoryFilter('${S.id}')">${S.label}</button>`).join(""),_=`
    <div class="bento-card" style="margin-top: 18px; padding: 12px 16px; border-color: rgba(245, 158, 11, 0.25); background: rgba(18, 24, 38, 0.4);">
      <div style="display: flex; justify-content: space-between; align-items: center; cursor: pointer; user-select: none;" onclick="window.app.toggleClientsCollapse()">
        <div style="display: flex; align-items: center; gap: 10px;">
          <span style="font-size: 13.5px; font-weight: 700; color: var(--text-main); display: flex; align-items: center; gap: 6px;">
            <span>⚡ 1-Click AI Client Integrations</span>
          </span>
          ${I}
        </div>
        <div style="display: flex; align-items: center; gap: 8px;">
          <button class="btn btn-ghost" style="padding: 2px 8px; font-size: 11px;" onclick="event.stopPropagation(); window.app.refreshClients()">⟳ Scan</button>
          <span style="font-size: 12px; color: var(--text-dim);">${A?"▼ Show":"▲ Hide"}</span>
        </div>
      </div>

      ${!A?`
        <div style="margin-top: 10px; padding-top: 8px; border-top: 1px solid var(--border-subtle);">
          <div style="display: flex; justify-content: space-between; align-items: center; gap: 8px; margin-bottom: 8px;">
            <div style="display: flex; gap: 4px; flex-wrap: wrap;">
              ${oe}
            </div>
            <input type="text" class="form-input" style="font-size: 10.5px; padding: 2px 8px; width: 140px; height: 22px; border-radius: 100px;" placeholder="\uD83D\uDD0D Search..." value="${i(e.clientSearchQuery||"")}" onclick="event.stopPropagation();" oninput="window.app.setClientSearchQuery(this.value)">
          </div>
          <div style="display: grid; grid-template-columns: repeat(auto-fill, minmax(270px, 1fr)); gap: 8px;">
            ${Z}
          </div>
        </div>
      `:""}
    </div>
  `;return`
    <div class="bento-grid">
      <div class="bento-card col-3">
        <div class="stat-label">Token Savings Rate</div>
        <div class="stat-value" style="color: var(--amber-300);">${x}</div>
        <div class="stat-sub">${b}</div>
      </div>
      <div class="bento-card col-3">
        <div class="stat-label">ETag Cache Hit Rate</div>
        <div class="stat-value" style="color: var(--cyan-400);">${g}</div>
        <div class="stat-sub">${u}</div>
      </div>
      <div class="bento-card col-3">
        <div class="stat-label">Tasks &amp; HITL State</div>
        <div class="stat-value" style="color: ${T>0?"var(--amber-400)":"var(--green-400)"};">${T>0?`${T} Action Req`:`${C} Active`}</div>
        <div class="stat-sub">${T>0?"Awaiting Human-in-the-Loop decision":`${h.length} total registered tasks`}</div>
      </div>
      <div class="bento-card col-3">
        <div class="stat-label">Connected Upstreams</div>
        <div class="stat-value" style="color: var(--green-400);">${r} Active</div>
        <div class="stat-sub">${r>0?"Persistent worker task channels":"No active upstream servers"}</div>
      </div>
    </div>

    ${_}

    <div style="display: flex; justify-content: space-between; align-items: center; margin: 24px 0 12px;">
      <div style="font-size: 15px; font-weight: 700; color: var(--text-main);">Connected Upstream Servers</div>
      <button class="btn btn-ghost" onclick="window.app.switchTab('servers')">Manage All (${r}) →</button>
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
        ${o}
      </div>
    </div>
  `}var W=[{id:"github",name:"GitHub",category:"devtools",description:"Explore repositories, issues, pull requests, branches, and commit histories.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-github"],envFields:[{key:"GITHUB_PERSONAL_ACCESS_TOKEN",label:"GitHub Personal Access Token",placeholder:"ghp_...",required:!0,description:"Classic or fine-grained token with repo scope."}]},{id:"git",name:"Git (Local)",category:"devtools",description:"Read local Git repository status, diffs, log histories, and commit changes.",badge:"Official / uvx",command:"uvx",defaultArgs:["--with","mcp<2","mcp-server-git","--repository","."],argsPlaceholder:"--with mcp<2 mcp-server-git --repository /path/to/repo",envFields:[]},{id:"filesystem",name:"Filesystem",category:"devtools",description:"Secure, sandboxed access to local files and directories for AI workflows.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-filesystem","."],argsPlaceholder:"-y @modelcontextprotocol/server-filesystem /allowed/dir1 /allowed/dir2",envFields:[]},{id:"memory",name:"Memory Graph",category:"devtools",description:"Persistent knowledge-graph based memory for multi-turn agent learning.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-memory"],envFields:[]},{id:"chrome-devtools",name:"Chrome DevTools",category:"devtools",description:"Inspect live DOM, execute scripts, read console logs, and capture network traces in Chrome.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-puppeteer"],envFields:[]},{id:"sentry",name:"Sentry",category:"devtools",description:"Query production error events, stack traces, and issue frequencies directly from Sentry.",badge:"uvx / Telemetry",command:"uvx",defaultArgs:["--with","mcp<2","--with","httpx","mcp-server-sentry","--auth-token","sntrys_token"],argsPlaceholder:"--with mcp<2 --with httpx mcp-server-sentry --auth-token YOUR_SENTRY_TOKEN",envFields:[{key:"SENTRY_AUTH_TOKEN",label:"Sentry Auth Token",placeholder:"sntrys_...",required:!0}]},{id:"playwright",name:"Playwright Browser",category:"browser",description:"Headless / headed browser automation for scraping, form filling, and UI interaction.",badge:"Popular #1 / npx",command:"npx",defaultArgs:["-y","@executeautomation/playwright-mcp-server"],envFields:[]},{id:"puppeteer",name:"Puppeteer",category:"browser",description:"Official browser automation server for web page scraping and screenshot capture.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-puppeteer"],envFields:[]},{id:"brave-search",name:"Brave Search",category:"browser",description:"Real-time privacy-preserving web search and local point-of-interest query engine.",badge:"Official / Search",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-brave-search"],envFields:[{key:"BRAVE_API_KEY",label:"Brave Search API Key",placeholder:"BSA...",required:!0}]},{id:"tavily",name:"Tavily Search",category:"browser",description:"AI-optimized web search engine structured specifically for LLM context injection.",badge:"Community / Stdio",command:"npx",defaultArgs:["-y","tavily-mcp"],envFields:[{key:"TAVILY_API_KEY",label:"Tavily API Key",placeholder:"tvly-...",required:!0}]},{id:"fetch",name:"Fetch / Web Markdown",category:"browser",description:"Download web pages, strip clutter, and convert raw HTML to clean markdown text.",badge:"Official / uvx",command:"uvx",defaultArgs:["mcp-server-fetch"],envFields:[]},{id:"postgres",name:"PostgreSQL",category:"database",description:"Read schemas, inspect tables, and execute SQL queries against PostgreSQL databases.",badge:"Official / Database",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-postgres","postgresql://user:pass@localhost:5432/mydb"],argsPlaceholder:"-y @modelcontextprotocol/server-postgres postgresql://user:pass@localhost:5432/dbname",envFields:[]},{id:"sqlite",name:"SQLite",category:"database",description:"Local embedded SQLite query runner and schema inspector.",badge:"Official / uvx",command:"uvx",defaultArgs:["--with","mcp<2","mcp-server-sqlite","--db-path","./app.db"],argsPlaceholder:"--with mcp<2 mcp-server-sqlite --db-path /path/to/database.sqlite",envFields:[]},{id:"supabase",name:"Supabase",category:"database",description:"Query database tables, manage auth policies, and inspect storage in Supabase.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@supabase/mcp-server-supabase@latest"],envFields:[{key:"SUPABASE_ACCESS_TOKEN",label:"Supabase Personal Access Token",placeholder:"sbp_...",required:!0},{key:"SUPABASE_PROJECT_REF",label:"Supabase Project Reference ID",placeholder:"abcdefghijklmnop",required:!1}]},{id:"redis",name:"Redis",category:"database",description:"Inspect cached keys, hash sets, lists, TTLs, and pub/sub channels in Redis.",badge:"Official / Key-Value",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-redis","redis://localhost:6379"],argsPlaceholder:"-y @modelcontextprotocol/server-redis redis://localhost:6379",envFields:[]},{id:"s3",name:"AWS S3 / Cloud Storage",category:"database",description:"Browse S3 buckets, fetch object metadata, and download files from cloud storage.",badge:"Community / Stdio",command:"npx",defaultArgs:["-y","@geunoh/s3-mcp-server"],argsPlaceholder:"-y @geunoh/s3-mcp-server",envFields:[{key:"AWS_ACCESS_KEY_ID",label:"AWS Access Key ID",placeholder:"AKIA...",required:!0},{key:"AWS_SECRET_ACCESS_KEY",label:"AWS Secret Access Key",placeholder:"...",required:!0},{key:"AWS_REGION",label:"AWS Region",placeholder:"us-east-1",required:!1}]},{id:"linear",name:"Linear",category:"productivity",description:"Search, create, and triage Linear issues, cycles, teams, and project roadmaps.",badge:"Productivity / Stdio",command:"npx",defaultArgs:["-y","mcp-linear"],envFields:[{key:"LINEAR_API_KEY",label:"Linear API Key",placeholder:"lin_api_...",required:!0}]},{id:"slack",name:"Slack",category:"productivity",description:"Read channels, post messages, inspect threads, and search team discussions.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@modelcontextprotocol/server-slack"],envFields:[{key:"SLACK_BOT_TOKEN",label:"Slack Bot User Token",placeholder:"xoxb-...",required:!0},{key:"SLACK_TEAM_ID",label:"Slack Team ID",placeholder:"T01234567",required:!0}]},{id:"notion",name:"Notion",category:"productivity",description:"Search Notion workspace pages, read nested blocks, and query database entries.",badge:"Official / Stdio",command:"npx",defaultArgs:["-y","@notionhq/notion-mcp-server"],envFields:[{key:"NOTION_TOKEN",label:"Notion Internal Integration Token",placeholder:"secret_...",required:!0}]},{id:"jira",name:"Jira / Atlassian",category:"productivity",description:"Manage Jira issues, search JQL, read sprint statuses, and inspect boards.",badge:"uvx / Atlassian",command:"uvx",defaultArgs:["--with","mcp<2","mcp-server-jira","--jira-base-url","https://your-domain.atlassian.net"],argsPlaceholder:"--with mcp<2 mcp-server-jira --jira-base-url https://org.atlassian.net",envFields:[{key:"JIRA_TOKEN",label:"Atlassian API Token",placeholder:"ATATT3...",required:!0}]},{id:"google-drive",name:"Google Drive",category:"productivity",description:"Search, list, and read documents, spreadsheets, and drive files.",badge:"Community / Stdio",command:"npx",defaultArgs:["-y","@piotr-agier/google-drive-mcp"],envFields:[{key:"GOOGLE_APPLICATION_CREDENTIALS",label:"Google Credentials JSON Path",placeholder:"/path/to/credentials.json",required:!0}]},{id:"docker",name:"Docker",category:"cloud",description:"Inspect running containers, tail container logs, list images, and manage compose services.",badge:"uvx / DevOps",command:"uvx",defaultArgs:["mcp-server-docker"],envFields:[]},{id:"kubernetes",name:"Kubernetes (K8s)",category:"cloud",description:"Query cluster pods, services, deployment status, and inspect Kubernetes logs.",badge:"Popular / Stdio",command:"npx",defaultArgs:["-y","@strowk/mcp-k8s"],envFields:[{key:"KUBECONFIG",label:"Kubeconfig File Path (Optional)",placeholder:"~/.kube/config",required:!1}]},{id:"cloudflare",name:"Cloudflare",category:"cloud",description:"Manage Cloudflare Workers, KV namespaces, D1 databases, Vectorize indexes, and DNS.",badge:"Official / Cloudflare",command:"npx",defaultArgs:["-y","@cloudflare/mcp-server-cloudflare","run","dummy_account_id"],argsPlaceholder:"-y @cloudflare/mcp-server-cloudflare run YOUR_ACCOUNT_ID",envFields:[{key:"CLOUDFLARE_API_TOKEN",label:"Cloudflare API Token",placeholder:"...",required:!0},{key:"CLOUDFLARE_ACCOUNT_ID",label:"Cloudflare Account ID",placeholder:"...",required:!0}]},{id:"terraform",name:"Terraform",category:"cloud",description:"Inspect Terraform state files, resource dependency graphs, and plan previews.",badge:"Community / IaC",command:"npx",defaultArgs:["-y","@mseep/terraform-mcp-server"],envFields:[]}];function J(e,t,s){let r=W.find((o)=>o.id.toLowerCase()===e.toLowerCase());if(r)return r;let a=`${t||""} ${(s||[]).join(" ")}`.toLowerCase();return W.find((o)=>{let n=`${o.command} ${o.defaultArgs.join(" ")}`.toLowerCase();if(a.includes(o.id.toLowerCase()))return!0;if(o.command&&a.includes(o.command.toLowerCase())&&o.defaultArgs.some((l)=>a.includes(l.toLowerCase())))return!0;return!1})}function ve(){let e=p.getState(),t=e.config.mcpServers||{},s=Object.keys(t),r=e.activeProfile,a=r?e.config.profiles?.[r]:void 0,o=!!a,n=a?.servers||[],l="";if(s.length===0)l=`
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
    `;else l=s.map((g)=>{let u=t[g],m=u.command?"stdio":"http / sse",v=u.command?`${u.command} ${(u.args||[]).join(" ")}`:u.url,d=e.serverStatuses[g]||{status:"connected",protocol_version:"2026-07-28"},f=!o||n.includes(g),x=J(g,u.command,u.args),b=Object.keys(u.env||{}),h=(x?.envFields||[]).filter((_)=>_.required&&!b.includes(_.key)),T=u.env?Object.entries(u.env).map(([_,S])=>{if(S.startsWith("keychain://"))return`<span class="brand-badge" style="color: var(--cyan-400); border-color: rgba(34, 211, 238, 0.3);">\uD83D\uDD12 ${i(_)} (Keychain)</span>`;if(S.startsWith("op://"))return`<span class="brand-badge" style="color: var(--cyan-400); border-color: rgba(34, 211, 238, 0.3);">\uD83D\uDD12 ${i(_)} (1Password)</span>`;if(S.startsWith("env://"))return`<span class="brand-badge" style="color: var(--amber-300); border-color: rgba(251, 191, 36, 0.3);">\uD83D\uDD12 ${i(_)} (Env)</span>`;return`<span style="color: var(--text-dim);">${i(_)}=***</span>`}):[];for(let _ of h)T.push(`<span class="brand-badge" style="color: var(--red-400); border-color: rgba(248, 113, 113, 0.4); background: rgba(248, 113, 113, 0.1);" title="Required environment variable '${i(_.key)}' is missing">⚠️ Missing ${i(_.key)}</span>`);let C=T.length>0?T.join(" "):"None",E=(e.circuitBreakers||[]).find((_)=>_.server_id===g),w='<span class="brand-badge" style="color: var(--green-400); border-color: rgba(52, 211, 153, 0.25);">Circuit: CLOSED</span>';if(E){if(E.state==="open")w=`<span class="brand-badge" style="color: var(--red-400); border-color: rgba(248, 113, 113, 0.4); background: rgba(248, 113, 113, 0.1);">Circuit: OPEN (${E.consecutive_failures} failures)</span>`;else if(E.state==="half_open")w=`<span class="brand-badge" style="color: var(--amber-300); border-color: rgba(251, 191, 36, 0.4); background: rgba(251, 191, 36, 0.1);">Circuit: HALF-OPEN (${E.consecutive_successes} probe)</span>`}let k=u.resilience||e.config.resilience,A=k?`FT: ${k.failureThreshold||3} · Cooldown: ${(k.cooldownMs||30000)/1000}s · AutoRestart: ${k.autoRestart!==!1?"ON":"OFF"}`:"Default Resilience",I=h.length>0,R=d.status==="degraded"||I,q=d.status==="error"||d.status==="disconnected",L=I?"var(--amber-400)":R?"var(--amber-400)":q?"var(--red-400)":"var(--green-400)",j=I?`Status: ${i(d.status)} (Missing Keys)`:`Status: ${i(d.status)}`,N=(R||q)&&(d.error||I)?`
        <div style="background: rgba(239, 68, 68, 0.08); border-left: 3px solid var(--amber-400); border-radius: var(--radius-xs); padding: 8px 12px; margin-top: 8px; display: flex; justify-content: space-between; align-items: center; gap: 8px;">
          <div style="font-size: 11px; color: var(--amber-300); font-family: var(--ff-mono); overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">
            <span style="font-weight: 700; color: var(--amber-400);">⚠️ Diagnostics:</span> ${i(d.error||`Missing required environment variable(s): ${h.map((_)=>_.key).join(", ")}`)}
          </div>
          <button class="btn btn-ghost" style="padding: 2px 8px; font-size: 10.5px; color: var(--amber-300); border-color: rgba(251, 191, 36, 0.3);" onclick="window.app.openServerDiagnosticsModal('${i(g)}')">Details</button>
        </div>
      `:"",Z=o?f?`<span class="brand-badge" style="color: var(--green-400); border-color: rgba(34, 197, 94, 0.3); background: rgba(34, 197, 94, 0.08); display: inline-flex; align-items: center; gap: 6px;">
              ✔ IN CONSTELLATION
              <button style="background: none; border: none; color: var(--amber-400); font-size: 10px; cursor: pointer; padding: 0 2px; text-decoration: underline;" onclick="window.app.toggleServerInProfile('${i(r)}', '${i(g)}', false)">Exclude</button>
            </span>`:`<span class="brand-badge" style="color: var(--amber-400); border-color: rgba(245, 158, 11, 0.35); background: rgba(245, 158, 11, 0.08); display: inline-flex; align-items: center; gap: 6px;">
              \uD83D\uDEAB EXCLUDED FROM PROFILE: ${i(r)}
              <button style="background: none; border: none; color: var(--green-400); font-size: 10px; cursor: pointer; padding: 0 2px; text-decoration: underline; font-weight: 700;" onclick="window.app.toggleServerInProfile('${i(r)}', '${i(g)}', true)">+ Include</button>
            </span>`:"";return`
        <div class="bento-card" style="${o&&!f?"margin-bottom: 12px; opacity: 0.65; border: 1px dashed rgba(245, 158, 11, 0.4); background: rgba(0, 0, 0, 0.2);":`margin-bottom: 12px; border-color: ${R?"rgba(251, 191, 36, 0.3)":q?"rgba(248, 113, 113, 0.3)":"var(--border)"};`}">
          <div style="display: flex; justify-content: space-between; align-items: center; gap: 16px; flex-wrap: wrap;">
            <div style="flex: 1; min-width: 260px;">
              <div style="display: flex; align-items: center; gap: 8px; margin-bottom: 4px; flex-wrap: wrap;">
                <span style="width: 8px; height: 8px; border-radius: 50%; background: ${L}; display: inline-block;"></span>
                <span style="font-size: 15px; font-weight: 700; color: var(--text-main);">${i(g)}</span>
                <span class="brand-badge">${m}</span>
                <span class="brand-badge" style="color: ${L}; border-color: ${I?"rgba(245, 158, 11, 0.5); background: rgba(245, 158, 11, 0.1);":"rgba(245, 158, 11, 0.3);"}">${j}</span>
                <span class="brand-badge" style="color: var(--cyan-400); border-color: rgba(34, 211, 238, 0.25);">Protocol: ${d.protocol_version}</span>
                ${w}
                ${Z}
              </div>
              <div style="font-family: var(--ff-mono); font-size: 12px; color: var(--text-muted); margin-top: 4px;">
                ${u.command?"Command: ":"URL: "}<code>${i(v||"")}</code>
              </div>
              <div style="display: flex; gap: 14px; font-family: var(--ff-mono); font-size: 11px; color: var(--text-dim); margin-top: 4px; align-items: center; flex-wrap: wrap;">
                <span>\uD83D\uDEE1️ ${i(A)}</span>
                ${u.env&&Object.keys(u.env).length>0?`<span>Env: ${C}</span>`:""}
              </div>
            </div>
            <div style="display: flex; gap: 8px; align-items: center; flex-shrink: 0;">
              <button class="btn btn-ghost" style="padding: 4px 10px; font-size: 11.5px; color: var(--cyan-400); border-color: rgba(34, 211, 238, 0.3);" onclick="window.app.restartServer('${i(g)}')">⚡ Restart</button>
              <button class="btn btn-ghost" style="padding: 4px 10px; font-size: 11.5px;" onclick="window.app.openServerDiagnosticsModal('${i(g)}')">\uD83D\uDD0D Diagnostics</button>
              <button class="btn btn-ghost" style="padding: 4px 10px; font-size: 11.5px;" onclick="window.app.openEditServerModal('${i(g)}')">✏️ Edit</button>
              <button class="btn btn-danger" style="padding: 4px 10px; font-size: 11.5px;" onclick="window.app.deleteServer('${i(g)}')">${o?"Delete Globally":"Remove"}</button>
            </div>
          </div>
          ${N}
        </div>
      `}).join("");let c=o?`
    <div class="bento-card" style="margin-bottom: 16px; background: rgba(245, 158, 11, 0.05); border: 1px solid rgba(245, 158, 11, 0.3); display: flex; justify-content: space-between; align-items: center; gap: 14px; flex-wrap: wrap;">
      <div style="display: flex; align-items: center; gap: 10px; flex: 1; min-width: 260px;">
        <span style="font-size: 18px; flex-shrink: 0;">\uD83C\uDF0C</span>
        <div>
          <div style="font-size: 13px; font-weight: 700; color: var(--amber-400); display: flex; align-items: center; gap: 8px; flex-wrap: wrap;">
            <span>Active Profile Constellation: <code style="font-size: 13px; color: var(--text-main);">${i(r)}</code></span>
            <span class="brand-badge" style="color: var(--text-main);">${n.length} of ${s.length} servers included</span>
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

    ${c}

    ${l}

    ${Fe()}
  `}function Fe(){let e=p.getState(),t=e.clients||[],s=Object.keys(e.config.profiles||{}),r=e.clientsCollapsed,a=e.clientFilterCategory||"all",o=(e.clientSearchQuery||"").toLowerCase().trim();if(t.length===0)return"";let n=t.filter((d)=>d.is_attached).length,l=t.filter((d)=>d.config_exists&&!d.is_attached).length,c=t.filter((d)=>d.category.toLowerCase().includes("ide")||d.category.toLowerCase().includes("extension")).length,g=t.filter((d)=>d.category.toLowerCase().includes("agent")||d.category.toLowerCase().includes("cli")||d.category.toLowerCase().includes("platform")).length,u=t.filter((d)=>{if(o){let f=d.name.toLowerCase().includes(o),x=d.id.toLowerCase().includes(o),b=d.category.toLowerCase().includes(o),h=d.config_path.toLowerCase().includes(o);if(!f&&!x&&!b&&!h)return!1}if(a==="connected")return d.is_attached;if(a==="ready")return d.config_exists||d.app_installed;if(a==="ides")return d.category.toLowerCase().includes("ide")||d.category.toLowerCase().includes("extension");if(a==="agents")return d.category.toLowerCase().includes("agent")||d.category.toLowerCase().includes("cli")||d.category.toLowerCase().includes("platform");return!0}),m=[{id:"all",label:`All Ecosystems (${t.length})`},{id:"ready",label:`Ready / Installed (${l+n})`},{id:"connected",label:`⚡ Connected (${n})`},{id:"ides",label:`IDEs & Editors (${c})`},{id:"agents",label:`Agents & CLIs (${g})`}].map((d)=>`
      <button class="btn btn-ghost" style="padding: 3px 10px; font-size: 11px; border-radius: 100px; ${a===d.id?"background: var(--amber-400); color: #000; font-weight: 700; border-color: var(--amber-400);":"background: var(--surface); color: var(--text-muted); border-color: var(--border);"}" onclick="window.app.setClientCategoryFilter('${i(d.id)}')">
        ${i(d.label)}
      </button>
    `).join(""),v=u.length===0?`<div style="padding: 24px; text-align: center; color: var(--text-dim); font-size: 12px;">No AI clients match the filter "${i(o||a)}".</div>`:u.map((d)=>{let{is_attached:f,config_exists:x,app_installed:b}=d,h='<span class="brand-badge" style="color: var(--text-dim); border-color: rgba(255, 255, 255, 0.1);">Not Found</span>';if(f){let w=d.attached_profile?` · ${d.attached_profile}`:"";h=`<span class="brand-badge" style="color: var(--green-400); border-color: rgba(52, 211, 153, 0.3); background: rgba(52, 211, 153, 0.1);">⚡ Connected · ${d.attached_transport==="http"?"HTTP":"stdio"}${i(w)}</span>`}else if(x)h='<span class="brand-badge" style="color: var(--amber-300); border-color: rgba(251, 191, 36, 0.3); background: rgba(251, 191, 36, 0.08);">○ Ready</span>';else if(b)h='<span class="brand-badge" style="color: var(--cyan-400); border-color: rgba(34, 211, 238, 0.25);">○ Installed</span>';let T=e.activeProfile,C=s.map((w)=>`
          <option value="${i(w)}" ${T===w||d.attached_profile===w?"selected":""}>Profile: ${i(w)}</option>
        `).join(""),E=f?`<button class="btn btn-ghost" style="padding: 3px 10px; font-size: 11px; color: var(--red-400); border-color: rgba(248, 113, 113, 0.3);" onclick="window.app.detachClient('${i(d.id)}')">Disconnect</button>`:`<button class="btn btn-primary" style="padding: 3px 10px; font-size: 11px;" onclick="window.app.attachClient('${i(d.id)}')">⚡ Connect</button>`;return`
          <div style="display: grid; grid-template-columns: 200px 130px 1fr 140px 100px; align-items: center; gap: 12px; padding: 8px 12px; background: var(--surface); border: 1px solid var(--border-subtle); border-radius: var(--radius-sm); transition: background 0.15s;" onmouseover="this.style.background='var(--surface-hover)'" onmouseout="this.style.background='var(--surface)'">
            <div>
              <div style="font-weight: 700; font-size: 13px; color: var(--text-main); display: flex; align-items: center; gap: 6px;">
                <span>${i(d.name)}</span>
              </div>
              <div style="font-size: 10.5px; color: var(--text-dim);">${i(d.category)}</div>
            </div>

            <div>
              ${h}
            </div>

            <div style="font-family: var(--ff-mono); font-size: 10.5px; color: var(--text-muted); overflow: hidden; text-overflow: ellipsis; white-space: nowrap;" title="${i(d.config_path)}">
              ${i(d.config_path)}
            </div>

            <div>
              ${s.length>0&&!f?`
                <select id="client-prof-${i(d.id)}" class="form-input" style="font-size: 10.5px; padding: 2px 6px; height: 26px; width: 100%;">
                  <option value="">All Tools (Default)</option>
                  ${C}
                </select>
              `:`<span style="font-size: 11px; color: var(--text-dim);">${d.other_servers_count>0?`${d.other_servers_count} other tools`:"Single facade"}</span>`}
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
            <span class="brand-badge" style="color: var(--amber-400); border-color: rgba(245, 158, 11, 0.3);">${n>0?`${n} Connected`:`${t.length} Ecosystems Supported`}</span>
          </div>
          <div style="font-size: 11px; color: var(--text-dim); margin-top: 2px;">
            Attach Warmplane's unified facade to desktop IDEs, CLI assistants, and autonomous agent platforms without editing JSON/TOML files.
          </div>
        </div>
        <div style="display: flex; align-items: center; gap: 8px;">
          <button class="btn btn-ghost" style="padding: 3px 8px; font-size: 11px;" onclick="event.stopPropagation(); window.app.refreshClients()">⟳ Scan Ecosystems</button>
          <span style="font-size: 12px; color: var(--text-dim);">${r?"▼ Show":"▲ Hide"}</span>
        </div>
      </div>

      ${!r?`
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
  `}function fe(){let e=p.getState(),t=e.playgroundMode||"tools",s=e.capabilities||[],r=e.resources||[],a=e.prompts||[],o=e.capabilitiesHiddenByPolicy||0,n=e.resourcesHiddenByPolicy||0,l=e.promptsHiddenByPolicy||0,c=e.activeProfile,u=!!(c?e.config.profiles?.[c]:void 0),m=`
    <div style="display: flex; gap: 8px; margin-bottom: 12px; align-items: center; justify-content: space-between; flex-wrap: wrap;">
      <div style="display: inline-flex; padding: 3px; background: rgba(0,0,0,0.3); border: 1px solid var(--border); border-radius: var(--radius-sm); align-items: center;">
        <button 
          class="btn ${t==="tools"?"btn-primary":"btn-ghost"}" 
          style="padding: 4px 12px; font-size: 11.5px; height: 28px; display: inline-flex; align-items: center; gap: 6px;"
          onclick="window.app.setPlaygroundMode('tools')"
        >
          <span>\uD83D\uDEE0️ Tools (${s.length})</span>
          ${o>0?`<span class="badge" style="background: rgba(245, 158, 11, 0.2); color: var(--amber-300); font-size: 9.5px; padding: 1px 5px;" title="${o} tools hidden by constellation/policy">+${o} hidden</span>`:""}
        </button>
        <button 
          class="btn ${t==="resources"?"btn-primary":"btn-ghost"}" 
          style="padding: 4px 12px; font-size: 11.5px; height: 28px; display: inline-flex; align-items: center; gap: 6px;"
          onclick="window.app.setPlaygroundMode('resources')"
        >
          <span>\uD83D\uDCC4 Resources (${r.length})</span>
          ${n>0?`<span class="badge" style="background: rgba(245, 158, 11, 0.2); color: var(--amber-300); font-size: 9.5px; padding: 1px 5px;" title="${n} resources hidden by constellation/policy">+${n} hidden</span>`:""}
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
        ${t==="tools"&&o>0?`
          <div style="font-size: 11px; color: var(--amber-300); background: rgba(245, 158, 11, 0.1); border: 1px solid rgba(245, 158, 11, 0.25); padding: 3px 8px; border-radius: var(--radius-sm); display: flex; align-items: center; gap: 6px;">
            <span>\uD83D\uDEE1️ ${o} tool${o>1?"s":""} filtered ${u?`(Profile: ${i(c)})`:"by policy"}</span>
            <a href="javascript:void(0)" onclick="window.app.switchTab('policy')" style="color: var(--amber-400); text-decoration: underline; font-weight: 600;">View Policy</a>
            ${u?`<a href="javascript:void(0)" onclick="window.app.switchTab('servers')" style="color: var(--cyan-400); text-decoration: underline; font-weight: 600; margin-left: 4px;">Server Hub</a>`:""}
          </div>
        `:t==="resources"&&n>0?`
          <div style="font-size: 11px; color: var(--amber-300); background: rgba(245, 158, 11, 0.1); border: 1px solid rgba(245, 158, 11, 0.25); padding: 3px 8px; border-radius: var(--radius-sm); display: flex; align-items: center; gap: 6px;">
            <span>\uD83D\uDEE1️ ${n} resource${n>1?"s":""} filtered ${u?`(Profile: ${i(c)})`:"by policy"}</span>
            <a href="javascript:void(0)" onclick="window.app.switchTab('policy')" style="color: var(--amber-400); text-decoration: underline; font-weight: 600;">View Policy</a>
            ${u?`<a href="javascript:void(0)" onclick="window.app.switchTab('servers')" style="color: var(--cyan-400); text-decoration: underline; font-weight: 600; margin-left: 4px;">Server Hub</a>`:""}
          </div>
        `:t==="prompts"&&l>0?`
          <div style="font-size: 11px; color: var(--amber-300); background: rgba(245, 158, 11, 0.1); border: 1px solid rgba(245, 158, 11, 0.25); padding: 3px 8px; border-radius: var(--radius-sm); display: flex; align-items: center; gap: 6px;">
            <span>\uD83D\uDEE1️ ${l} prompt${l>1?"s":""} filtered ${u?`(Profile: ${i(c)})`:"by policy"}</span>
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
      ${De(e)}
    `;if(t==="prompts")return`
      ${m}
      ${Ue(e)}
    `;return`
    ${m}
    ${Ne(e)}
    ${e.isBatchModalOpen?Je(e):""}
  `}function Y(e,t=!1){if(!e||!e.properties)return{};let s=e.properties||{},r=Array.isArray(e.required)?e.required:[],a={};for(let[o,n]of Object.entries(s)){let l=r.includes(o);if(t&&!l)continue;if(n.default!==void 0)a[o]=n.default;else if(Array.isArray(n.enum)&&n.enum.length>0)a[o]=n.enum[0];else if(n.examples&&Array.isArray(n.examples)&&n.examples.length>0)a[o]=n.examples[0];else if(n.example!==void 0)a[o]=n.example;else switch(n.type||"string"){case"string":a[o]=l?`sample_${o}`:"";break;case"number":case"integer":a[o]=0;break;case"boolean":a[o]=!0;break;case"array":a[o]=[];break;case"object":a[o]={};break;default:a[o]=`sample_${o}`}}return a}function Ne(e){let t=e.capabilities||[],s=e.selectedCapabilityId||(t.length>0?t[0].id:null),r=t.find((T)=>T.id===s),a=e.config.capabilityAliases?.[s],o=typeof a==="object"&&!!a?.reversibility?.testTarget,n=typeof a==="object"?a?.reversibility?.testTarget?.server:void 0,l=e.isExecutingCapability,c=e.capabilitiesHiddenByPolicy||0,g=e.activeProfile,u=!!(g&&e.config.profiles?.[g]),m="";if(t.length===0)m=`
      <div style="padding: 24px 16px; text-align: center; color: var(--text-dim); font-size: 11.5px;">
        No tools or capabilities discovered from connected servers.
      </div>
    `;else m=t.map((T)=>`
        <div class="cap-item ${T.id===s?"active":""}" onclick="window.app.selectCapability('${i(T.id)}')">
          <div style="display: flex; justify-content: space-between; align-items: center;">
            <span style="font-weight: 600; color: var(--text-main); font-family: var(--ff-mono); font-size: 12px;">${i(T.id)}</span>
            <span style="font-size: 10px; color: var(--green-400);">${i(T.mode||"read")}</span>
          </div>
          <div style="font-size: 11px; color: var(--text-dim); margin-top: 2px;">server: ${i(T.server||"local")}</div>
        </div>
      `).join("");let v=r?.input_schema,d=v?.properties||{},f=Array.isArray(v?.required)?v.required:[],x=Object.entries(d),b="";if(x.length>0)b=`
      <div style="display: flex; flex-wrap: wrap; gap: 6px; margin-bottom: 8px; align-items: center;">
        <span style="font-size: 10px; font-weight: 700; color: var(--text-dim); text-transform: uppercase;">Fields:</span>
        ${x.map(([T,C])=>{let E=f.includes(T),w=C.type||(C.enum?"enum":"any"),k=E?"rgba(239, 68, 68, 0.15)":"rgba(148, 163, 184, 0.1)",A=E?"var(--red-400)":"var(--text-muted)",I=E?"rgba(239, 68, 68, 0.3)":"var(--border)",R=C.description?` - ${C.description}`:"";return`
            <button 
              type="button" 
              class="btn" 
              style="padding: 2px 7px; font-size: 10.5px; font-family: var(--ff-mono); background: ${k}; color: ${A}; border: 1px solid ${I}; border-radius: var(--radius-sm);"
              title="Click to insert '${i(T)}' (${i(w)}${i(R)})"
              onclick="window.app.insertPlaygroundArgKey('${i(T)}', '${i(w)}', ${i(JSON.stringify(C.default??null))})"
            >
              + ${i(T)} <span style="font-size: 9px; opacity: 0.7;">(${w}${E?" *":""})</span>
            </button>
          `}).join("")}
      </div>
    `;let h="{}";if(s&&e.playgroundArgs&&e.playgroundArgs[s]!==void 0)h=e.playgroundArgs[s];else{let T=Y(v,!1);h=JSON.stringify(T,null,2)}return`
    <div style="display: grid; grid-template-columns: 320px 1fr; gap: 16px; height: calc(100vh - 165px);">
      <!-- Left Sidebar: Capabilities Catalog -->
      <div style="background: var(--surface-card); border: 1px solid var(--border); border-radius: var(--radius-md); display: flex; flex-direction: column; overflow: hidden;">
        <div style="padding: 12px; border-bottom: 1px solid var(--border);">
          <input type="text" class="form-input" placeholder="Search ${t.length} capabilities..." oninput="window.app.filterCapabilities(this.value)">
        </div>
        <div style="flex: 1; overflow-y: auto; padding: 8px;" id="pg-cap-list">
          ${m}
        </div>
        ${c>0?`
          <div style="padding: 8px 12px; background: rgba(245, 158, 11, 0.08); border-top: 1px solid rgba(245, 158, 11, 0.2); font-size: 11px; color: var(--amber-300); display: flex; justify-content: space-between; align-items: center;">
            <span>\uD83D\uDEE1️ ${c} tool${c>1?"s":""} filtered ${u?`(${i(g)})`:""}</span>
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
              ${i(r?r.id:"No Capability Selected")}
            </div>
            <div style="font-size: 11.5px; color: var(--text-dim);" id="pg-selected-desc">
              ${i(r?r.summary||r.description:"Connect servers to inspect and execute tools")}
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
              <button class="btn btn-primary" onclick="window.app.executePlaygroundTool()" ${r?"":"disabled"}>
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polygon points="5 3 19 12 5 21 5 3"></polygon></svg>
                Execute Capability
              </button>
            `}
            ${o?`<button class="btn btn-ghost" data-playground-action="test-undo" data-undo-alias="${i(s)}" ${l?"disabled":""} title="Runs the configured forward and inverse cycle on ${i(n)}">Test undo contract</button>`:""}
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

            ${b}

            <textarea class="form-textarea" rows="7" id="pg-args-input" oninput="window.app.updatePlaygroundArgs(this.value)">${i(h)}</textarea>

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
                  <span style="font-size: 10px; color: var(--text-dim); font-family: var(--ff-mono);">${x.length} field${x.length===1?"":"s"} (${f.length} required)</span>
                </div>
                <pre style="background: var(--surface); padding: 10px; border-radius: var(--radius-sm); border: 1px solid var(--border); font-size: 11px; color: var(--text-muted); max-height: 140px; overflow-y: auto;">${i(JSON.stringify(r.input_schema,null,2))}</pre>
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
  `}function De(e){let t=e.resources||[],s=e.resourcesHiddenByPolicy||0,r=e.selectedResourceId||(t.length>0?t[0].id:null),a=t.find((l)=>l.id===r),o=e.resourceReadResult,n=te(t,r,"No resources exposed by connected MCP servers.");return`
    <div style="display: grid; grid-template-columns: 340px 1fr; gap: 16px; height: calc(100vh - 165px);">
      ${be(t.length,s,n,"pg-res-list","filterResources","resource")}

      <!-- Right Panel: Resource Content Reader & Metadata Inspector -->
      <div style="background: var(--surface-card); border: 1px solid var(--border); border-radius: var(--radius-md); display: flex; flex-direction: column; overflow: hidden;">
        <div style="padding: 14px 18px; border-bottom: 1px solid var(--border); display: flex; justify-content: space-between; align-items: center;">
          <div>
            <div style="font-size: 15px; font-weight: 700; color: var(--text-main); font-family: var(--ff-mono);">
              ${i(a?a.name||a.id:"No Resource Selected")}
            </div>
            <div style="font-size: 11.5px; color: var(--cyan-400); font-family: var(--ff-mono);">
              ${i(a?a.uri:"Select a resource from the list to read live content")}
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
                  <div><span style="color: var(--text-muted);">Server:</span> <strong style="color: var(--text-main);">${i(a.server)}</strong></div>
                  <div><span style="color: var(--text-muted);">MIME Type:</span> <strong style="color: var(--text-main);">${i(a.mime_type||"text/plain")}</strong></div>
                </div>
                ${a.description?`
                  <div style="margin-top: 8px; font-size: 11.5px; color: var(--text-dim); border-top: 1px solid rgba(255,255,255,0.05); padding-top: 6px;">
                    ${i(a.description)}
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

          ${xe(o,"RESOURCE CONTENT ENVELOPE","var(--cyan-400)",'// Click "Read Resource Content" to inspect live payload')}
        </div>
      </div>
    </div>
  `}function Ue(e){let t=e.prompts||[],s=e.promptsHiddenByPolicy||0,r=e.selectedPromptId||(t.length>0?t[0].id:null),a=t.find((c)=>c.id===r),o=e.promptGetResult,n=se(t,r,"No prompt templates registered by connected MCP servers."),l="";if(a&&a.arguments&&a.arguments.length>0)l=a.arguments.map((c)=>`
      <div class="form-group" style="margin-bottom: 12px;">
        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 4px;">
          <label class="form-label" style="margin: 0; font-family: var(--ff-mono);">${i(c.name)}</label>
          ${c.required?'<span class="badge" style="background: rgba(239, 68, 68, 0.15); color: var(--red-400); font-size: 9px;">REQUIRED</span>':'<span style="font-size: 10px; color: var(--text-dim);">optional</span>'}
        </div>
        ${c.description?`<div style="font-size: 11px; color: var(--text-dim); margin-bottom: 4px;">${i(c.description)}</div>`:""}
        <input type="text" class="form-input prompt-arg-input" data-arg-name="${i(c.name)}" placeholder="Enter ${i(c.name)}..." />
      </div>
    `).join("");else if(a)l=`
      <div style="padding: 12px; background: rgba(0,0,0,0.2); border-radius: var(--radius-sm); border: 1px solid var(--border); font-size: 11.5px; color: var(--text-dim);">
        This prompt template does not require any input arguments.
      </div>
    `;return`
    <div style="display: grid; grid-template-columns: 340px 1fr; gap: 16px; height: calc(100vh - 165px);">
      ${be(t.length,s,n,"pg-prompt-list","filterPrompts","prompt")}

      <!-- Right Panel: Prompt Parameter Binder & Message Envelope Preview -->
      <div style="background: var(--surface-card); border: 1px solid var(--border); border-radius: var(--radius-md); display: flex; flex-direction: column; overflow: hidden;">
        <div style="padding: 14px 18px; border-bottom: 1px solid var(--border); display: flex; justify-content: space-between; align-items: center;">
          <div>
            <div style="font-size: 15px; font-weight: 700; color: var(--text-main); font-family: var(--ff-mono);">
              ${i(a?a.name||a.id:"No Prompt Selected")}
            </div>
            <div style="font-size: 11.5px; color: var(--text-dim);">
              ${i(a?a.description||a.title||"Bind variables and render messages":"Select a prompt from the list to test")}
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

          ${xe(o,"RENDERED PROMPT MESSAGES","#c084fc",'// Click "Render Prompt Messages" to view resolved system/user messages')}
        </div>
      </div>
    </div>
  `}function Je(e){let t=e.capabilities||[],s=e.batchSteps||[];return`
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
          ${s.map((a,o)=>{let n=t.find((d)=>d.id===a.capability_id),l=n?.input_schema,c=l?.properties||{},g=Array.isArray(l?.required)?l.required:[],u=Object.entries(c),m=t.map((d)=>`
      <option value="${i(d.id)}" ${d.id===a.capability_id?"selected":""}>
        ${i(d.id)} (${i(d.server||"local")})
      </option>
    `).join(""),v="";if(u.length>0)v=`
        <div style="display: flex; flex-wrap: wrap; gap: 4px; margin-top: 6px; margin-bottom: 6px; align-items: center;">
          <span style="font-size: 9.5px; font-weight: 700; color: var(--text-dim); text-transform: uppercase;">Parameters:</span>
          ${u.map(([d,f])=>{let x=g.includes(d),b=f.type||(f.enum?"enum":"any");return`
              <span style="font-size: 9.5px; font-family: var(--ff-mono); padding: 1px 5px; background: ${x?"rgba(239, 68, 68, 0.15)":"rgba(148, 163, 184, 0.1)"}; color: ${x?"var(--red-400)":"var(--text-muted)"}; border: 1px solid ${x?"rgba(239, 68, 68, 0.3)":"var(--border)"}; border-radius: 3px;" title="${i(f.description||"")}">
                ${i(d)} (${b}${x?" *":""})
              </span>
            `}).join("")}
        </div>
      `;return`
      <div style="background: rgba(0,0,0,0.3); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 14px; margin-bottom: 12px;">
        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 10px;">
          <div style="display: flex; align-items: center; gap: 8px;">
            <span class="badge" style="background: rgba(56, 189, 248, 0.15); color: var(--cyan-400); font-family: var(--ff-mono); font-weight: 700;">STEP ${o+1}</span>
            <span style="font-size: 11px; font-family: var(--ff-mono); color: var(--text-dim);">id: ${i(a.id)}</span>
          </div>
          <button class="btn btn-ghost" style="padding: 2px 8px; font-size: 11px; color: var(--red-400);" onclick="window.app.removeBatchStep(${o})">
            ✕ Remove
          </button>
        </div>

        <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 12px; margin-bottom: 6px;">
          <div class="form-group" style="margin: 0;">
            <label class="form-label" style="font-size: 11px;">Target Capability</label>
            <select class="form-input" style="font-size: 11.5px;" onchange="window.app.updateBatchStepCapability(${o}, this.value)">
              <option value="">-- Select Capability --</option>
              ${m}
            </select>
          </div>
          <div style="display: flex; align-items: flex-end; padding-bottom: 6px;">
            <label style="display: flex; align-items: center; gap: 6px; font-size: 11.5px; color: var(--text-muted); cursor: pointer;">
              <input type="checkbox" ${a.continue_on_error?"checked":""} onchange="window.app.updateBatchStepContinueOnError(${o}, this.checked)" />
              <span>Continue pipeline on step failure</span>
            </label>
          </div>
        </div>

        ${v}

        <div class="form-group" style="margin: 0;">
          <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 4px;">
            <div style="display: flex; align-items: center; gap: 8px;">
              <label class="form-label" style="margin: 0; font-size: 11px;">Step Arguments JSON</label>
              ${n?`
                <button type="button" class="btn btn-ghost" style="padding: 1px 6px; font-size: 9.5px;" onclick="window.app.fillBatchStepSampleArgs(${o})">✨ Sample Args</button>
              `:""}
            </div>
            <div style="display: flex; gap: 6px; font-size: 10px; color: var(--cyan-400); font-family: var(--ff-mono);">
              <span>Helpers:</span>
              <code style="cursor: pointer; background: rgba(0,0,0,0.3); padding: 1px 4px; border-radius: 2px;" onclick="window.app.appendBatchVariable(${o}, '\${steps[0].result.id}')">\${steps[0].result.id}</code>
              <code style="cursor: pointer; background: rgba(0,0,0,0.3); padding: 1px 4px; border-radius: 2px;" onclick="window.app.appendBatchVariable(${o}, '\${steps[0].result.data}')">\${steps[0].result.data}</code>
            </div>
          </div>
          <textarea 
            id="batch-step-args-${o}"
            class="form-textarea" 
            rows="3" 
            style="font-size: 11px; font-family: var(--ff-mono);" 
            oninput="window.app.updateBatchStepArgs(${o}, this.value)"
          >${i(a.argsJson)}</textarea>
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
            ${s.length} sequential execution steps configured
          </div>
          <div style="display: flex; gap: 10px;">
            <button class="btn btn-ghost" onclick="window.app.closeBatchModal()">Cancel</button>
            <button class="btn btn-primary" onclick="window.app.executeBatchPipeline()">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polygon points="5 3 19 12 5 21 5 3"></polygon></svg>
              Run Batch Pipeline (${s.length} Steps)
            </button>
          </div>
        </div>
      </div>
    </div>
  `}function ye(e){return`<div style="padding: 24px 16px; text-align: center; color: var(--text-dim); font-size: 11.5px;">${i(e)}</div>`}function be(e,t,s,r,a,o){return`<div style="background: var(--surface-card); border: 1px solid var(--border); border-radius: var(--radius-md); display: flex; flex-direction: column; overflow: hidden;">
    <div style="padding: 12px; border-bottom: 1px solid var(--border);">
      <input type="text" class="form-input" placeholder="Search ${e} ${o}s..." oninput="window.app.${a}(this.value)">
    </div>
    <div style="flex: 1; overflow-y: auto; padding: 8px;" id="${r}">${s}</div>
    ${t>0?`<div style="padding: 8px 12px; background: rgba(245, 158, 11, 0.08); border-top: 1px solid rgba(245, 158, 11, 0.2); font-size: 11px; color: var(--amber-300); display: flex; justify-content: space-between; align-items: center;">
      <span>\uD83D\uDEE1️ ${t} ${o}${t>1?"s":""} hidden by policy</span>
      <a href="javascript:void(0)" onclick="window.app.switchTab('policy')" style="color: var(--amber-400); text-decoration: underline; font-weight: 600; font-size: 10.5px;">Edit Policy</a>
    </div>`:""}
  </div>`}function xe(e,t,s,r){return`<div style="padding: 16px; background: var(--bg-app); display: flex; flex-direction: column; overflow: hidden;">
    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
      <span style="font-size: 11px; font-weight: 600; color: var(--text-dim);">${t}</span>
      <span style="font-size: 11px; font-weight: 600; color: ${e?e.status===200?"var(--green-400)":"var(--red-400)":"var(--text-dim)"}; font-family: var(--ff-mono);">
        ${e?`HTTP ${e.status} · ${e.durationMs.toFixed(1)}ms`:"READY"}
      </span>
    </div>
    <pre style="flex: 1; background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 14px; color: ${s}; font-size: 11.5px; overflow-y: auto; margin: 0; white-space: pre-wrap; word-break: break-word;">${e?i(JSON.stringify(e.data,null,2)):r}</pre>
  </div>`}function te(e,t,s){let r="";if(e.length===0)r=ye(s);else r=e.map((a)=>{let o=a.id===t?"active":"",n=a.uri?a.uri.split(":")[0]:"res";return`
        <div class="cap-item ${o}" onclick="window.app.selectResource('${i(a.id)}')">
          <div style="display: flex; justify-content: space-between; align-items: center;">
            <span style="font-weight: 600; color: var(--text-main); font-family: var(--ff-mono); font-size: 12px;">${i(a.name||a.id)}</span>
            <span class="badge" style="font-size: 9.5px; background: rgba(56, 189, 248, 0.15); color: var(--cyan-400);">${i(n)}</span>
          </div>
          <div style="font-size: 11px; color: var(--text-dim); margin-top: 2px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">${i(a.uri)}</div>
          <div style="display: flex; justify-content: space-between; font-size: 10px; color: var(--text-muted); margin-top: 4px;">
            <span>server: ${i(a.server||"local")}</span>
            <span>${i(a.mime_type||"text/plain")}</span>
          </div>
        </div>
      `}).join("");return r}function se(e,t,s){let r="";if(e.length===0)r=ye(s);else r=e.map((a)=>{let o=a.id===t?"active":"",n=a.arguments?a.arguments.length:0;return`
        <div class="cap-item ${o}" onclick="window.app.selectPrompt('${i(a.id)}')">
          <div style="display: flex; justify-content: space-between; align-items: center;">
            <span style="font-weight: 600; color: var(--text-main); font-family: var(--ff-mono); font-size: 12px;">${i(a.name||a.id)}</span>
            <span class="badge" style="font-size: 9.5px; background: rgba(168, 85, 247, 0.15); color: var(--purple-400);">${n} args</span>
          </div>
          <div style="font-size: 11px; color: var(--text-dim); margin-top: 2px;">${i(a.description||a.title||"Prompt template")}</div>
          <div style="font-size: 10px; color: var(--text-muted); margin-top: 4px;">server: ${i(a.server||"local")}</div>
        </div>
      `}).join("");return r}function Ke(e){let t=e.decision_receipt,s=e.status==="pending",r=e.receipt_delivery==="delivered"?"Delivered":e.receipt_delivery==="failed"?"Delivery failed":"No delivery recorded";return`
    <details class="tool-pin-review">
      <summary><code>${i(e.capability_id)}</code>
        <span class="tool-pin-status">${i(e.status)}</span>
      </summary>
      <div class="tool-pin-comparison">
        <div class="tool-pin-definition">
          <strong>Reviewed arguments</strong>
          <pre tabindex="0">${i(JSON.stringify(e.sanitized_args??{},null,2))}</pre>
        </div>
        <dl class="approval-details">
          <dt>Ticket</dt><dd><code>${i(e.id)}</code></dd>
          <dt>Decided by</dt><dd>${i(t?.principal??"Awaiting decision")}</dd>
          <dt>Execution</dt><dd>${i(e.upstream_execution_state??e.execution_status??"No execution status recorded")}</dd>
          <dt>ChatOps receipt</dt><dd>${r}</dd>
        </dl>
      </div>
      ${s?`
        <button type="button" class="btn btn-primary" data-approval-choice="approve" data-approval-target="${i(e.id)}">Approve invocation</button>
        <button type="button" class="btn btn-ghost" data-approval-choice="reject" data-approval-target="${i(e.id)}">Reject</button>
      `:""}
    </details>
  `}function Ve(e){return`
    <section class="bento-card" style="margin-bottom: 16px;">
      <h2 class="stat-label">Approval activity</h2>
      <form class="trust-auth-form" id="approval-auth-form">
        <div class="form-group">
          <label class="form-label" for="approval-operator-token">Approval operator token</label>
          <input class="form-input" type="password" autocomplete="off" id="approval-operator-token">
        </div>
        <button class="btn btn-ghost" type="submit">Load approvals</button>
      </form>
      <p class="tool-pin-feedback" role="status">${i(e.approvalError??(e.approvals.length?"":"No approval tickets are visible."))}</p>
      ${e.approvals.map(Ke).join("")}
    </section>
    ${e.taskError?`<p class="tool-pin-feedback" role="status">${i(e.taskError)}</p>`:""}
  `}function he(e){let t=e.tasks||[],s=e.taskFilterStatus||"all",r=t.filter((d)=>d.status==="input_required"),a=t.filter((d)=>d.status==="working"),o=t.filter((d)=>d.status==="completed"),n=t.filter((d)=>d.status==="cancelled"),l=t.filter((d)=>d.status==="failed"),c=s==="all"?t:t.filter((d)=>d.status===s),g=e.config.policy?.require_approval||e.config.policy?.requireApproval||[],u=r.length===0?`
    <div style="padding: 36px 24px; text-align: center; background: var(--surface-card); border-radius: var(--radius-md); border: 1px dashed var(--border);">
      <div style="width: 44px; height: 44px; border-radius: 50%; background: rgba(52, 211, 153, 0.12); border: 1px solid rgba(52, 211, 153, 0.3); display: flex; align-items: center; justify-content: center; margin: 0 auto 12px; color: var(--green-400); font-size: 18px; font-weight: 700;">
        ✓
      </div>
      <div style="font-size: 14.5px; font-weight: 600; color: var(--text-main); margin-bottom: 5px;">No Tasks Awaiting Input or Approval</div>
      <div style="font-size: 11.5px; color: var(--text-dim); max-width: 520px; margin: 0 auto; line-height: 1.6;">
        Tool calls requiring Human-in-the-Loop approval or returning asynchronous <code style="color: var(--amber-300); font-family: var(--ff-mono);">input_required</code> tasks will suspend here for operator inspection, parameter editing, and response submission.
      </div>
    </div>
  `:r.map((d)=>{let f=d.inputRequests||{},x=Object.keys(f),b=x.length>0,h=Math.floor(Date.now()/1000),T=d.expiresAtEpochSecs?Math.max(0,d.expiresAtEpochSecs-h):d.ttlMs?Math.max(0,Math.floor(d.ttlMs/1000)):d.ttlSeconds||300,C=d.createdAtEpochSecs?new Date(d.createdAtEpochSecs*1000).toLocaleTimeString():d.createdAt?new Date(d.createdAt).toLocaleTimeString():"—";return`
      <div class="bento-card" style="border: 1px solid rgba(245, 158, 11, 0.35); background: var(--surface-card); margin-bottom: 14px; padding: 18px;">
        <div style="display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 12px;">
          <div>
            <div style="display: flex; align-items: center; gap: 8px;">
              <span class="brand-badge" style="background: rgba(245, 158, 11, 0.2); color: var(--amber-300); border-color: rgba(245, 158, 11, 0.5);">
                INPUT REQUIRED
              </span>
              <span style="font-family: var(--ff-mono); font-size: 12px; color: var(--text-muted);">${i(d.taskId)}</span>
            </div>
            <div style="margin-top: 6px; display: flex; align-items: center; gap: 8px;">
              <span style="font-size: 14.5px; font-weight: 700; color: var(--text-main); font-family: var(--ff-mono);">
                ${i(d.capabilityId||"Tool Execution")}
              </span>
              ${d.serverId?`<span style="font-size: 11px; color: var(--text-dim);">via <span style="color: var(--cyan-400); font-family: var(--ff-mono);">${i(d.serverId)}</span></span>`:""}
            </div>
          </div>

          <div style="text-align: right; font-family: var(--ff-mono); font-size: 11px; color: var(--text-dim);">
            <div>Created: <span style="color: var(--text-muted);">${C}</span></div>
            <div style="color: var(--amber-400); margin-top: 2px;">TTL Remaining: ${T}s</div>
          </div>
        </div>

        <!-- Caller Context -->
        ${d.context?`
          <div style="background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 8px 12px; font-family: var(--ff-mono); font-size: 11px; display: flex; flex-wrap: wrap; gap: 16px; margin-bottom: 12px; color: var(--text-muted);">
            ${d.context.actor_id?`<div><span style="color: var(--text-dim);">Actor:</span> <span style="color: var(--cyan-400);">${i(d.context.actor_id)}</span></div>`:""}
            ${d.context.operation_id?`<div><span style="color: var(--text-dim);">Operation:</span> <span style="color: var(--text-main);">${i(d.context.operation_id)}</span></div>`:""}
            ${d.context.grant_id?`<div><span style="color: var(--text-dim);">Grant:</span> <span style="color: var(--text-main);">${i(d.context.grant_id)}</span></div>`:""}
          </div>
        `:""}

        <!-- Dynamic Input Requests Form -->
        <div style="margin-bottom: 14px;">
          <div style="font-size: 11px; font-weight: 600; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.5px; margin-bottom: 8px;">
            ${b?"Required Input Responses (MRTR / HITL)":"Input Responses Payload (JSON)"}
          </div>

          ${b?`
            <div style="display: flex; flex-direction: column; gap: 10px;">
              ${x.map((E)=>{let w=f[E]||{},k=typeof w==="string"?w:w.prompt||w.description||w.title||E,A=w.type||"text",I=w.default!==void 0?JSON.stringify(w.default):w.value!==void 0?JSON.stringify(w.value):w.sanitized_args?JSON.stringify(w.sanitized_args,null,2):"";if(A==="approval_review")return`
                    <div style="background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 10px 12px;">
                      <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 4px;">
                        <label style="font-size: 11.5px; font-weight: 600; color: var(--amber-300); font-family: var(--ff-mono);">${i(E)}</label>
                        <span class="brand-badge" style="font-size: 9.5px; padding: 1px 5px;">APPROVAL GATED</span>
                      </div>
                      <div style="font-size: 11px; color: var(--text-dim); margin-bottom: 6px;">${i(k)}</div>
                      <div style="margin-bottom: 8px;">
                        <label style="font-size: 10.5px; color: var(--text-dim); display: block; margin-bottom: 2px;">Decision:</label>
                        <select id="task-input-${i(d.taskId)}-${i(E)}-decision" class="form-input" style="font-size: 11.5px; font-family: var(--ff-mono); padding: 4px 8px;">
                          <option value="" selected>Choose a decision</option>
                          <option value="true">Approve &amp; Execute</option>
                          <option value="false">Reject Execution</option>
                        </select>
                      </div>
                      <div>
                        <div class="tool-pin-definition"><strong>Reviewed arguments</strong><pre tabindex="0">${i(I)}</pre></div>
                        <label class="form-label" for="task-input-${i(d.taskId)}-${i(E)}">Replacement arguments (optional)</label>
                        <textarea id="task-input-${i(d.taskId)}-${i(E)}" class="form-textarea" rows="3"></textarea>
                      </div>
                    </div>
                  `;return`
                  <div style="background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 10px 12px;">
                    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 4px;">
                      <label style="font-size: 11.5px; font-weight: 600; color: var(--amber-300); font-family: var(--ff-mono);">${i(E)}</label>
                      <span class="brand-badge" style="font-size: 9.5px; padding: 1px 5px;">${i(A)}</span>
                    </div>
                    <div style="font-size: 11px; color: var(--text-dim); margin-bottom: 6px;">${i(k)}</div>
                    ${A==="confirmation"||A==="boolean"?`
                      <select id="task-input-${i(d.taskId)}-${i(E)}" class="form-input" style="font-size: 11.5px; font-family: var(--ff-mono); padding: 4px 8px;">
                        <option value="true" selected>true (Approve / Confirm)</option>
                        <option value="false">false (Reject / Deny)</option>
                      </select>
                    `:w.sanitized_args||A==="object"||A==="json"?`
                      <textarea id="task-input-${i(d.taskId)}-${i(E)}" class="form-textarea" rows="3" style="color: var(--green-400); font-family: var(--ff-mono); font-size: 11.5px;">${i(I)}</textarea>
                    `:`
                      <input id="task-input-${i(d.taskId)}-${i(E)}" type="text" class="form-input" value="${i(I)}" placeholder="Enter ${i(E)} response..." style="font-size: 11.5px; font-family: var(--ff-mono);">
                    `}
                  </div>
                `}).join("")}
            </div>
          `:`
            <textarea id="task-raw-input-${i(d.taskId)}" class="form-textarea" rows="3" style="color: var(--green-400); font-family: var(--ff-mono); font-size: 11.5px;">{}</textarea>
          `}
        </div>

        <!-- Action Footer -->
        <div style="display: flex; justify-content: space-between; align-items: center; border-top: 1px solid var(--border); padding-top: 12px;">
          <div style="display: flex; align-items: center; gap: 8px;">
            <input id="task-operator-${i(d.taskId)}" type="text" class="form-input" placeholder="Operator ID" value="security-operator" style="width: 180px; padding: 5px 10px; font-size: 11px;">
          </div>

          <div style="display: flex; align-items: center; gap: 8px;">
            <button class="btn btn-danger" onclick="window.app.promptCancelTask('${i(d.taskId)}')">
              ✕ Cancel Task
            </button>
            <button class="btn btn-primary" onclick="window.app.submitTaskInputResponses('${i(d.taskId)}')">
              ✓ Submit &amp; Resume
            </button>
          </div>
        </div>
      </div>
    `}).join(""),m=g.length===0?`
    <div style="color: var(--text-dim); font-size: 11.5px; line-height: 1.5; padding: 8px 0;">
      No explicit <code style="color: var(--amber-400);">require_approval</code> rules active. Gated execution rules convert matching tool calls into tasks in real-time.
    </div>
  `:g.map((d)=>`
    <div style="display: flex; justify-content: space-between; align-items: center; background: var(--surface); padding: 8px 10px; border-radius: var(--radius-sm); border: 1px solid var(--border); margin-bottom: 6px;">
      <span style="font-family: var(--ff-mono); font-size: 11.5px; color: var(--amber-300); font-weight: 500;">\uD83D\uDEE1️ ${i(d)}</span>
      <span class="brand-badge" style="font-size: 9.5px; padding: 1px 5px;">GATED</span>
    </div>
  `).join(""),v=c.length===0?`
    <tr>
      <td colspan="6" style="padding: 24px; text-align: center; color: var(--text-dim); font-size: 12px;">
        No tasks found matching filter "${i(s)}".
      </td>
    </tr>
  `:c.map((d)=>{let f=d.status==="completed"?"background: rgba(52, 211, 153, 0.12); color: var(--green-400); border-color: rgba(52, 211, 153, 0.3);":d.status==="working"?"background: rgba(56, 189, 248, 0.15); color: var(--cyan-400); border-color: rgba(56, 189, 248, 0.4);":d.status==="input_required"?"background: rgba(245, 158, 11, 0.2); color: var(--amber-300); border-color: rgba(245, 158, 11, 0.5);":d.status==="cancelled"?"background: rgba(148, 163, 184, 0.15); color: var(--text-muted); border-color: rgba(148, 163, 184, 0.3);":"background: rgba(248, 113, 113, 0.12); color: var(--red-400); border-color: rgba(248, 113, 113, 0.3);",x=d.progress!==void 0?Math.round(d.progress*100):d.status==="completed"?100:d.status==="working"?50:0,b=d.createdAtEpochSecs?new Date(d.createdAtEpochSecs*1000).toLocaleTimeString():d.createdAt?new Date(d.createdAt).toLocaleTimeString():"—";return`
      <tr style="border-bottom: 1px solid rgba(255,255,255,0.03); transition: background 0.15s;" onmouseover="this.style.background='var(--surface-hover)'" onmouseout="this.style.background='transparent'">
        <td style="padding: 10px 14px;">
          <span class="brand-badge" style="${f}">
            ${d.status.toUpperCase()}
          </span>
        </td>
        <td style="padding: 10px 14px; font-family: var(--ff-mono); font-weight: 600; color: var(--text-main); font-size: 11.5px;">
          ${i(d.capabilityId||"Tool Execution")}
        </td>
        <td style="padding: 10px 14px; font-family: var(--ff-mono); color: var(--text-dim); font-size: 11px;">
          ${i(d.taskId)}
        </td>
        <td style="padding: 10px 14px; width: 140px;">
          <div style="display: flex; align-items: center; gap: 8px;">
            <div style="flex: 1; height: 6px; background: var(--surface-card); border-radius: 3px; overflow: hidden; border: 1px solid var(--border);">
              <div style="height: 100%; width: ${x}%; background: ${d.status==="completed"?"var(--green-400)":"var(--amber-400)"}; transition: width 0.3s;"></div>
            </div>
            <span style="font-size: 10.5px; font-family: var(--ff-mono); color: var(--text-muted);">${x}%</span>
          </div>
        </td>
        <td style="padding: 10px 14px; color: var(--text-dim); font-size: 11px; text-align: right;">
          ${b}
        </td>
        <td style="padding: 10px 14px; text-align: right;">
          <div style="display: inline-flex; gap: 6px; align-items: center;">
            <button class="btn btn-ghost" style="padding: 2px 8px; font-size: 10.5px;" onclick="window.app.openTaskInspectorModal('${i(d.taskId)}')">\uD83D\uDD0D Inspect</button>
            ${d.status==="input_required"||d.status==="working"?`
              <button class="btn btn-danger" style="padding: 2px 8px; font-size: 10.5px;" onclick="window.app.promptCancelTask('${i(d.taskId)}')">Cancel</button>
            `:""}
          </div>
        </td>
      </tr>
    `}).join("");return`
    ${Ve(e)}
    <!-- Sub-header & Actions -->
    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px;">
      <div style="display: flex; align-items: center; gap: 10px;">
        <span class="brand-badge" style="font-size: 11px; padding: 3px 10px; color: ${r.length>0?"var(--amber-300)":"var(--green-400)"}; border-color: ${r.length>0?"rgba(245, 158, 11, 0.4)":"rgba(52, 211, 153, 0.4)"}; background: ${r.length>0?"rgba(245, 158, 11, 0.1)":"rgba(52, 211, 153, 0.1)"};">
          ${r.length} ACTION REQUIRED
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
        <div class="stat-value" style="color: ${r.length>0?"var(--amber-400)":"var(--text-main)"};">${r.length}</div>
        <div class="stat-sub">Awaiting operator decision or response</div>
      </div>
      <div class="bento-card col-3">
        <div class="stat-label">Working / In-Flight</div>
        <div class="stat-value" style="color: var(--cyan-400);">${a.length}</div>
        <div class="stat-sub">Asynchronous active executions</div>
      </div>
      <div class="bento-card col-3">
        <div class="stat-label">Completed Tasks</div>
        <div class="stat-value" style="color: var(--green-400);">${o.length}</div>
        <div class="stat-sub">Finished successfully</div>
      </div>
      <div class="bento-card col-3">
        <div class="stat-label">Cancelled / Failed</div>
        <div class="stat-value" style="color: ${l.length>0?"var(--red-400)":"var(--text-muted)"};">${n.length+l.length}</div>
        <div class="stat-sub">Terminated or errored</div>
      </div>
    </div>

    <!-- Main Content Bento Split (8 cols queue / 4 cols rules) -->
    <div class="bento-grid">
      <!-- Left Column: Input Required Action Queue -->
      <div class="col-8">
        <div style="font-size: 11px; font-weight: 700; color: var(--amber-400); text-transform: uppercase; letter-spacing: 0.5px; margin-bottom: 12px; display: flex; align-items: center; gap: 6px;">
          <span>⚡ Awaiting Operator Action (${r.length})</span>
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
          \uD83D\uDCDC Task Registry (${c.length})
        </div>
        <div style="display: flex; gap: 6px; align-items: center;">
          <span style="font-size: 11px; color: var(--text-dim);">Filter Status:</span>
          <select class="form-input" style="padding: 3px 8px; font-size: 11px; width: 140px;" onchange="window.app.filterTasksByStatus(this.value)">
            <option value="all" ${s==="all"?"selected":""}>All Statuses</option>
            <option value="input_required" ${s==="input_required"?"selected":""}>input_required</option>
            <option value="working" ${s==="working"?"selected":""}>working</option>
            <option value="completed" ${s==="completed"?"selected":""}>completed</option>
            <option value="cancelled" ${s==="cancelled"?"selected":""}>cancelled</option>
            <option value="failed" ${s==="failed"?"selected":""}>failed</option>
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
  `}function we(e){let t=[...e.undoEntries||[]].sort((a,o)=>o.created_at_ms-a.created_at_ms);if(e.undoError)return`<section class="bento-card"><h2>Action feed unavailable</h2><p>${i(e.undoError)}</p><button class="btn btn-ghost" data-undo-action="refresh" ${e.undoLoading?"disabled":""}>Retry</button></section>`;let s=(e.tasks||[]).filter((a)=>a.phase==="pending_emission");if(t.length===0&&s.length===0&&e.undoTotal===0)return`<section class="bento-card"><h2>Action feed</h2><p>No reversible actions or held sends are currently recorded.</p><p class="text-dim">New armed inverses appear here with actor, profile, capability, and expiry. Snapshot contents and action arguments are never shown.</p><button class="btn btn-ghost" data-undo-action="refresh" ${e.undoLoading?"disabled":""}>Refresh</button></section>`;let r=new Map;for(let a of t){let o=a.work_item_id||"";r.set(o,[...r.get(o)||[],a])}return`<section class="bento-card"><header style="display:flex;justify-content:space-between;align-items:center"><div><h2>Action feed</h2><p class="text-dim">Value-free action records. Upstream state is checked by configured contracts before an inverse runs.</p></div><button class="btn btn-ghost" data-undo-action="refresh" ${e.undoLoading?"disabled":""}>Refresh</button></header>
    ${s.map((a)=>{let o=Math.max(0,Math.ceil(((a.releaseAtMs||0)-Date.now())/1000)),n=Math.max(1,Math.ceil(((a.releaseAtMs||0)-(a.createdAtEpochSecs||0)*1000)/1000));return`<article style="margin-top:12px;padding:14px;border:1px solid var(--border);border-radius:var(--radius-md);background:var(--surface)"><div style="display:flex;justify-content:space-between;gap:12px"><div><span class="brand-badge">HELD SEND</span><strong style="margin-left:8px">${i(a.capabilityId||"Delayed action")}</strong><div class="text-dim" style="margin-top:6px">Actor: ${i(a.context?.actor_id||"Unknown")} · Task: ${i(a.taskId)}</div></div><div class="text-dim">${o}s until release</div></div><progress aria-label="Time until send release" value="${Math.min(o,n)}" max="${n}" style="width:100%;margin-top:10px"></progress>${a.cancellable&&o>0?`<button class="btn btn-danger" style="margin-top:12px" data-undo-action="cancel-task" data-undo-target="${i(a.taskId)}">Cancel now</button>`:""}</article>`}).join("")}
    ${Array.from(r.entries()).map(([a,o])=>`<section style="margin-top:18px"><header style="display:flex;justify-content:space-between;align-items:center"><strong>${a?`Work item ${i(a)}`:"Unassigned work item"}</strong>${a&&o.some((n)=>n.status==="armed"&&n.expires_at_ms>Date.now())?`<button class="btn btn-danger" data-undo-action="rollback" data-undo-target="${i(a)}">Rollback entire work item</button>`:""}</header>
      ${o.map((n)=>{let l=Math.max(0,Math.ceil((n.expires_at_ms-Date.now())/1000)),c=Math.max(1,Math.ceil((n.expires_at_ms-n.created_at_ms)/1000)),g=n.status==="armed"&&l>0;return`<article style="margin-top:10px;padding:14px;border:1px solid var(--border);border-radius:var(--radius-md);background:var(--surface)">
          <div style="display:flex;justify-content:space-between;gap:12px"><div><strong>${i(n.forward_capability)}</strong><div class="text-dim" style="margin-top:6px">Actor: ${i(n.actor_id||"Unknown")} · Profile: ${i(n.profile_id||"Default")}</div><div class="text-dim">Inverse: ${i(n.inverse_capability)}</div></div><div style="text-align:right"><span class="brand-badge">${i(n.status)}</span><div class="text-dim" style="margin-top:6px">${l>0?`${l}s remaining`:"Window expired"}</div></div></div>
          <div class="text-dim" style="margin-top:10px;font-size:11px">Undo ID: ${i(n.undo_id)} · Registered ${i(new Date(n.created_at_ms).toLocaleString())}</div>
          <progress aria-label="Undo window remaining" value="${Math.min(l,c)}" max="${c}" style="width:100%;margin-top:8px"></progress>
          <div class="text-dim" style="margin-top:6px;font-size:11px">Configured inverse fields: ${i((n.change_preview||[]).join(", ")||"withheld")}</div>
          ${g?`<button class="btn btn-primary" style="margin-top:12px" data-undo-action="execute" data-undo-target="${i(n.undo_id)}">Undo action</button>`:""}
        </article>`}).join("")}</section>`).join("")}
    <nav aria-label="Undo history pages" style="display:flex;align-items:center;justify-content:space-between;gap:12px;margin-top:16px;">
      <button class="btn btn-ghost" data-undo-action="previous" ${e.undoLoading||e.undoOffset===0?"disabled":""}>Previous</button>
      <span role="status">${e.undoLoading?"Loading…":`${e.undoTotal?e.undoOffset+1:0}–${e.undoOffset+t.length} of ${e.undoTotal}`}</span>
      <button class="btn btn-ghost" data-undo-action="next" ${e.undoLoading||e.undoOffset+50>=e.undoTotal?"disabled":""}>Next</button>
    </nav>
  </section>`}function We(e){if(!e.is_valid)return e.corrupted_at_index===void 0?"Verification failed":`Verification failed at record #${e.corrupted_at_index}`;if(e.first_retained_index===void 0)return`Chain verified (${e.total_records} events)`;return`Retained records verified (${e.total_records-e.first_retained_index} of ${e.total_records} events)`}function Ge(e){if(!e)return`
      <button class="btn btn-ghost" style="padding: 4px 10px; font-size: 11.5px;" onclick="window.app.verifyAuditChain()">
        \uD83D\uDEE1️ Verify Cryptographic Hash Chain
      </button>
    `;let t=e.is_valid?"34, 197, 94":"239, 68, 68",s=e.is_valid?"var(--green-400)":"var(--red-400)",r=e.is_valid?"\uD83D\uDEE1️":"⚠️";return`
    <div style="display: inline-flex; align-items: center; gap: 6px; padding: 4px 10px; background: rgba(${t}, 0.1); border: 1px solid rgba(${t}, 0.3); border-radius: var(--radius-sm); font-size: 11.5px; color: ${s};">
      <span>${r}</span>
      <span style="font-weight: 600;">${We(e)}</span>
    </div>
  `}function $e(){let e=p.getState(),t=e.auditEvents||[],s=e.auditStats||{total_events:0,by_status:{success:0,failed:0,denied:0,intercepted:0}},{auditVerification:r,auditFilters:a}=e,o=e.auditTotal??t.length,n=e.auditSelectedEvent,l=Object.keys(e.config?.mcpServers||{}),c=a.limit||25,g=a.offset||0,u=Math.floor(g/c)+1,m=Math.max(1,Math.ceil(o/c)),v=o===0?0:g+1,d=Math.min(g+c,o),f=y.getAuditExportUrl({actor_id:a.search?void 0:void 0,server_id:a.serverId!=="all"?a.serverId:void 0,event_type:a.eventType!=="all"?a.eventType:void 0,status:a.status!=="all"?a.status:void 0,search:a.search.trim()?a.search.trim():void 0},"csv"),x=y.getAuditExportUrl({server_id:a.serverId!=="all"?a.serverId:void 0,event_type:a.eventType!=="all"?a.eventType:void 0,status:a.status!=="all"?a.status:void 0,search:a.search.trim()?a.search.trim():void 0},"jsonl"),b=Ge(r),h=l.map((k)=>`<option value="${i(k)}" ${a.serverId===k?"selected":""}>${i(k)}</option>`).join(""),T=`
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
            value="${i(a.search)}"
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
            ${h}
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
  `,C=`
    <div style="display: flex; justify-content: space-between; align-items: center; padding: 12px 16px; background: rgba(18, 24, 38, 0.5); border-radius: var(--radius-md); border: 1px solid var(--border); margin-top: 16px;">
      <div style="font-size: 12px; color: var(--text-dim); display: flex; align-items: center; gap: 8px;">
        <span>Showing <strong style="color: var(--text-main);">${v}–${d}</strong> of <strong style="color: var(--text-main);">${o}</strong> events</span>
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
    `;else E=t.map((k)=>{let A=new Date(Math.floor(k.timestamp_ns/1e6)).toLocaleString(),I='<span class="badge" style="background: rgba(34, 197, 94, 0.15); color: var(--green-400); font-weight: 600;">SUCCESS</span>';if(k.status==="denied")I='<span class="badge" style="background: rgba(239, 68, 68, 0.15); color: var(--red-400); font-weight: 600;">DENIED</span>';else if(k.status==="intercepted")I='<span class="badge" style="background: rgba(234, 179, 8, 0.15); color: var(--amber-300); font-weight: 600;">HITL INTERCEPT</span>';else if(k.status==="failed")I='<span class="badge" style="background: rgba(239, 68, 68, 0.15); color: var(--red-400); font-weight: 600;">FAILED</span>';else if(k.status==="cancelled")I='<span class="badge" style="background: rgba(148, 163, 184, 0.15); color: var(--text-muted); font-weight: 600;">CANCELLED</span>';let R=k.sanitized_args?JSON.stringify(k.sanitized_args):"-",q=k.actor_id||k.operator_id||"anonymous",L=k.server_id||"system",j=k.capability_id||k.event_type,N=k.execution_latency_us?`${(k.execution_latency_us/1000).toFixed(1)}ms`:"-";return`
        <div class="bento-card" style="margin-bottom: 12px; padding: 16px; border: 1px solid var(--border); transition: border-color 0.15s ease;">
          <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
            <div style="display: flex; align-items: center; gap: 8px;">
              <span style="font-family: var(--ff-mono); font-size: 11px; font-weight: 700; color: var(--text-dim);">${i(k.id)}</span>
              ${I}
              <span style="font-size: 12px; font-weight: 600; color: var(--text-main);">${i(j)}</span>
            </div>
            <div style="display: flex; align-items: center; gap: 10px;">
              <div style="font-family: var(--ff-mono); font-size: 11px; color: var(--text-muted);">${i(A)}</div>
              <button 
                class="btn btn-ghost" 
                style="padding: 2px 8px; font-size: 11px; height: 24px;" 
                onclick="window.app.selectAuditEvent('${i(k.id)}')"
                title="Inspect event details & cryptographic payload"
              >
                Inspect \uD83D\uDD0D
              </button>
            </div>
          </div>
          
          <div style="display: grid; grid-template-columns: repeat(4, 1fr); gap: 8px; font-size: 11.5px; background: rgba(0,0,0,0.2); padding: 8px 12px; border-radius: var(--radius-sm); margin-bottom: 8px;">
            <div><span style="color: var(--text-muted);">Actor:</span> <strong style="color: var(--text-main);">${i(q)}</strong></div>
            <div><span style="color: var(--text-muted);">Server:</span> <strong style="color: var(--cyan-400);">${i(L)}</strong></div>
            <div><span style="color: var(--text-muted);">Trace:</span> <code style="color: var(--cyan-400); font-size: 10.5px;">${i(k.trace_id)}</code></div>
            <div><span style="color: var(--text-muted);">Latency:</span> <span style="color: var(--amber-300);">${N}</span></div>
          </div>

          <div style="font-family: var(--ff-mono); font-size: 11px; color: var(--text-dim); margin-bottom: 8px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">
            <span style="color: var(--text-muted);">Args:</span> ${i(R)}
          </div>

          <div style="display: flex; justify-content: space-between; align-items: center; border-top: 1px solid var(--border); padding-top: 6px; font-size: 10.5px; font-family: var(--ff-mono); color: var(--text-muted);">
            <div><span style="color: var(--text-dim);">prev_hash:</span> ${i(k.prev_hash.slice(0,16))}...</div>
            <div><span style="color: var(--text-dim);">hash:</span> <span style="color: var(--green-400);">${i(k.hash.slice(0,16))}...</span></div>
          </div>
        </div>
      `}).join("");let w="";if(n){let k=new Date(Math.floor(n.timestamp_ns/1e6)).toISOString();w=`
      <div style="position: fixed; inset: 0; background: rgba(0,0,0,0.7); backdrop-filter: blur(4px); z-index: 9999; display: flex; align-items: center; justify-content: center; padding: 24px;" onclick="if (event.target === this) window.app.selectAuditEvent(null)">
        <div class="bento-card" style="width: 100%; max-width: 720px; max-height: 85vh; display: flex; flex-direction: column; overflow: hidden; background: #0f172a; border: 1px solid var(--border); box-shadow: 0 20px 40px rgba(0,0,0,0.5);">
          <!-- Modal Header -->
          <div style="display: flex; justify-content: space-between; align-items: center; padding: 16px 20px; border-bottom: 1px solid var(--border);">
            <div style="display: flex; align-items: center; gap: 8px;">
              <span style="font-size: 16px;">\uD83D\uDD12</span>
              <h2 style="font-size: 15px; font-weight: 700; color: var(--text-main); margin: 0;">Audit Event Details (${i(n.id)})</h2>
            </div>
            <button class="btn btn-ghost" style="padding: 4px 8px; font-size: 14px;" onclick="window.app.selectAuditEvent(null)">✕</button>
          </div>

          <!-- Modal Body -->
          <div style="padding: 20px; overflow-y: auto; display: flex; flex-direction: column; gap: 14px; font-size: 12px;">
            <div style="display: grid; grid-template-columns: repeat(2, 1fr); gap: 10px; background: rgba(0,0,0,0.25); padding: 12px; border-radius: var(--radius-sm);">
              <div><span style="color: var(--text-muted);">Timestamp:</span> <strong style="color: var(--text-main); font-family: var(--ff-mono); font-size: 11px;">${i(k)}</strong></div>
              <div><span style="color: var(--text-muted);">Status:</span> <strong style="color: var(--text-main);">${i(n.status.toUpperCase())}</strong></div>
              <div><span style="color: var(--text-muted);">Event Type:</span> <strong style="color: var(--text-main);">${i(n.event_type)}</strong></div>
              <div><span style="color: var(--text-muted);">Server:</span> <strong style="color: var(--cyan-400);">${i(n.server_id||"system")}</strong></div>
              <div><span style="color: var(--text-muted);">Capability:</span> <strong style="color: var(--text-main);">${i(n.capability_id||"-")}</strong></div>
              <div><span style="color: var(--text-muted);">Actor / Operator:</span> <strong style="color: var(--text-main);">${i(n.actor_id||n.operator_id||"anonymous")}</strong></div>
              <div><span style="color: var(--text-muted);">Trace ID:</span> <code style="color: var(--cyan-400);">${i(n.trace_id)}</code></div>
              <div><span style="color: var(--text-muted);">Request ID:</span> <code style="color: var(--cyan-400);">${i(n.request_id||"-")}</code></div>
              <div><span style="color: var(--text-muted);">Client IP:</span> <span style="color: var(--text-main);">${i(n.client_ip||"-")}</span></div>
              <div><span style="color: var(--text-muted);">Latency:</span> <span style="color: var(--amber-300);">${n.execution_latency_us?`${(n.execution_latency_us/1000).toFixed(2)} ms`:"-"}</span></div>
            </div>

            ${n.error_message?`
              <div style="background: rgba(239, 68, 68, 0.1); border: 1px solid rgba(239, 68, 68, 0.3); border-radius: var(--radius-sm); padding: 10px 12px; color: var(--red-400);">
                <div style="font-weight: 700; margin-bottom: 2px;">Error (${i(n.error_code||"ERROR")}):</div>
                <div style="font-family: var(--ff-mono); font-size: 11px;">${i(n.error_message)}</div>
              </div>
            `:""}

            <!-- Sanitized Arguments -->
            <div>
              <div style="font-weight: 600; color: var(--text-main); margin-bottom: 4px;">Sanitized Arguments</div>
              <pre style="background: rgba(0,0,0,0.4); padding: 10px; border-radius: var(--radius-sm); border: 1px solid var(--border); font-family: var(--ff-mono); font-size: 11px; max-height: 140px; overflow: auto; margin: 0; color: #cbd5e1;">${i(JSON.stringify(n.sanitized_args||{},null,2))}</pre>
            </div>

            <!-- Sanitized Response -->
            ${n.sanitized_response?`
              <div>
                <div style="font-weight: 600; color: var(--text-main); margin-bottom: 4px;">Sanitized Response</div>
                <pre style="background: rgba(0,0,0,0.4); padding: 10px; border-radius: var(--radius-sm); border: 1px solid var(--border); font-family: var(--ff-mono); font-size: 11px; max-height: 140px; overflow: auto; margin: 0; color: #cbd5e1;">${i(JSON.stringify(n.sanitized_response,null,2))}</pre>
              </div>
            `:""}

            <!-- Cryptographic Hashes -->
            <div style="background: rgba(0,0,0,0.3); padding: 12px; border-radius: var(--radius-sm); border: 1px solid var(--border);">
              <div style="font-weight: 600; color: var(--text-main); margin-bottom: 6px;">Tamper-Evidence Cryptographic Hashes</div>
              <div style="margin-bottom: 6px;">
                <span style="color: var(--text-muted); font-size: 10.5px;">Previous Chain Hash (prev_hash):</span>
                <div style="font-family: var(--ff-mono); font-size: 10.5px; color: var(--text-dim); word-break: break-all;">${i(n.prev_hash)}</div>
              </div>
              <div>
                <span style="color: var(--text-muted); font-size: 10.5px;">Record Hash Signature (hash):</span>
                <div style="font-family: var(--ff-mono); font-size: 10.5px; color: var(--green-400); word-break: break-all;">${i(n.hash)}</div>
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
        ${b}
        <a href="${f}" download class="btn btn-ghost" style="font-size: 11.5px; text-decoration: none;" title="Export current filtered view as CSV">\uD83D\uDCE5 Export CSV</a>
        <a href="${x}" download class="btn btn-ghost" style="font-size: 11.5px; text-decoration: none;" title="Export current filtered view as JSONL">\uD83D\uDCE5 Export JSONL</a>
        <button class="btn btn-primary" style="font-size: 11.5px;" onclick="window.app.refreshAuditEvents()">\uD83D\uDD04 Refresh</button>
      </div>
    </div>

    <!-- Stats summary cards -->
    <div style="display: grid; grid-template-columns: repeat(4, 1fr); gap: 12px; margin-bottom: 20px;">
      <div class="bento-card" style="padding: 14px; text-align: center;">
        <div style="font-size: 11px; color: var(--text-muted); text-transform: uppercase; font-weight: 600;">Total Events</div>
        <div style="font-size: 22px; font-weight: 800; color: var(--text-main); margin-top: 4px;">${s.total_events}</div>
      </div>
      <div class="bento-card" style="padding: 14px; text-align: center;">
        <div style="font-size: 11px; color: var(--text-muted); text-transform: uppercase; font-weight: 600;">Successful Calls</div>
        <div style="font-size: 22px; font-weight: 800; color: var(--green-400); margin-top: 4px;">${s.by_status.success}</div>
      </div>
      <div class="bento-card" style="padding: 14px; text-align: center;">
        <div style="font-size: 11px; color: var(--text-muted); text-transform: uppercase; font-weight: 600;">HITL Intercepts</div>
        <div style="font-size: 22px; font-weight: 800; color: var(--amber-300); margin-top: 4px;">${s.by_status.intercepted}</div>
      </div>
      <div class="bento-card" style="padding: 14px; text-align: center;">
        <div style="font-size: 11px; color: var(--text-muted); text-transform: uppercase; font-weight: 600;">Policy Denials</div>
        <div style="font-size: 22px; font-weight: 800; color: var(--red-400); margin-top: 4px;">${s.by_status.denied}</div>
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
    ${o>0?C:""}

    <!-- Modal Popup for Event Inspection -->
    ${w}
  `}function ke(){let e=p.getState();if(!e.configReady)return`
    <section class="bento-card">
      <h2 class="stat-label">Configuration</h2>
      <p class="tool-pin-feedback" role="status">${i(e.configError??"Loading configuration…")}</p>
      <button type="button" class="btn btn-ghost" onclick="window.app.openTrustAccess()">Connect as trust administrator</button>
    </section>
  `;let t=e.activeProfile,s=t?e.config.profiles?.[t]:void 0,r=!!s,a=e.config.policy||{},o=s?.policy,n=r?o||{}:a,l=n.allow||[],c=n.deny||[],g=n.redact_keys||n.redactKeys||[],u=n.require_approval||n.requireApproval||[],m=l.length===0?`
    <div style="color: var(--text-dim); font-size: 12px;">${r?"No profile allow list (inherits global rules)":"No allow list (all non-denied operations permitted)"}</div>
  `:l.map((w,k)=>`
    <div style="display: flex; justify-content: space-between; align-items: center; background: var(--surface); padding: 8px 12px; border-radius: var(--radius-sm); border: 1px solid var(--border);">
      <span style="font-family: var(--ff-mono); font-size: 12px; color: var(--green-400);">✔ ${i(w)}</span>
      <button class="btn btn-ghost" style="padding: 2px 6px; font-size: 11px; color: var(--red-400);" onclick="window.app.removePolicyRule('allow', ${k})">✕</button>
    </div>
  `).join(""),v=c.length===0?`
    <div style="color: var(--text-dim); font-size: 12px;">${r?"No profile deny rules configured":"No deny rules configured"}</div>
  `:c.map((w,k)=>`
    <div style="display: flex; justify-content: space-between; align-items: center; background: var(--surface); padding: 8px 12px; border-radius: var(--radius-sm); border: 1px solid var(--border);">
      <span style="font-family: var(--ff-mono); font-size: 12px; color: var(--red-400);">✖ ${i(w)}</span>
      <button class="btn btn-ghost" style="padding: 2px 6px; font-size: 11px; color: var(--red-400);" onclick="window.app.removePolicyRule('deny', ${k})">✕</button>
    </div>
  `).join(""),d=u.length===0?`
    <div style="color: var(--text-dim); font-size: 12px;">${r?"No profile human-in-the-loop triggers configured":"No human-in-the-loop approval rules configured"}</div>
  `:u.map((w,k)=>`
    <div style="display: flex; justify-content: space-between; align-items: center; background: var(--surface); padding: 8px 12px; border-radius: var(--radius-sm); border: 1px solid var(--border);">
      <span style="font-family: var(--ff-mono); font-size: 12px; color: var(--amber-400);">\uD83D\uDEE1️ ${i(w)}</span>
      <button class="btn btn-ghost" style="padding: 2px 6px; font-size: 11px; color: var(--red-400);" onclick="window.app.removePolicyRule('requireApproval', ${k})">✕</button>
    </div>
  `).join(""),f=g.length===0?`
    <div style="color: var(--text-dim); font-size: 12px;">${r?"No profile key redaction patterns configured":"No key redaction patterns configured"}</div>
  `:g.map((w,k)=>`
    <span class="brand-badge" style="color: var(--amber-300); padding: 5px 10px; font-size: 11px; display: inline-flex; align-items: center; gap: 6px;">
      ${i(w)}
      <span style="cursor: pointer; color: var(--red-400); font-weight: bold;" onclick="window.app.removePolicyRule('redact', ${k})">✕</span>
    </span>
  `).join(""),x=r?`
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
  `,b=Object.keys(e.config.mcpServers||{}),h=s?.servers||[],T=r?b.filter((w)=>!h.includes(w)):[],C=r?`
    <div class="bento-card" style="margin-bottom: 16px; border: 1px solid rgba(245, 158, 11, 0.2); background: rgba(0, 0, 0, 0.2);">
      <div class="stat-header" style="display: flex; justify-content: space-between; align-items: center;">
        <span class="stat-label" style="color: var(--amber-400);">Constellation Server Boundaries (Profile: ${i(t)})</span>
        <span style="font-size: 11px; color: var(--text-dim);">${h.length} of ${b.length} servers active</span>
      </div>
      <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 14px; margin-top: 10px;">
        <div style="background: var(--surface); padding: 10px; border-radius: var(--radius-sm); border: 1px solid var(--border);">
          <div style="font-size: 11px; font-weight: 600; color: var(--green-400); text-transform: uppercase; margin-bottom: 6px;">
            ✔ Included Servers (${h.length})
          </div>
          <div style="display: flex; flex-wrap: wrap; gap: 6px;">
            ${h.length>0?h.map((w)=>`
              <span class="brand-badge" style="color: var(--cyan-400); border-color: rgba(34, 211, 238, 0.25); background: rgba(34, 211, 238, 0.05);">
                ${i(w)}
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
                ${i(w)}
                <button style="background: none; border: none; color: var(--amber-400); font-size: 10px; cursor: pointer; padding: 0 2px;" title="Include in profile" onclick="window.app.toggleServerInProfile('${i(t)}', '${i(w)}', true)">+</button>
              </span>
            `).join(""):'<span style="font-size: 11px; color: var(--text-dim);">All servers included in constellation</span>'}
          </div>
        </div>
      </div>
    </div>
  `:"",E=r&&T.length>0?`
    <div style="border-top: 1px dashed var(--border); padding-top: 8px; margin-top: 8px;">
      <div style="font-size: 10.5px; color: var(--text-dim); text-transform: uppercase; font-weight: 600; margin-bottom: 6px;">
        Implicit Boundary Denials (${T.length})
      </div>
      <div style="display: flex; flex-direction: column; gap: 4px;">
        ${T.map((w)=>`
          <div style="display: flex; justify-content: space-between; align-items: center; background: rgba(245, 158, 11, 0.03); padding: 5px 8px; border-radius: var(--radius-xs); border: 1px dashed rgba(245, 158, 11, 0.2);">
            <span style="font-family: var(--ff-mono); font-size: 11px; color: var(--text-dim);">✖ ${i(w)}.*</span>
            <span style="font-size: 9.5px; color: var(--amber-400); font-family: var(--ff-mono);">server excluded</span>
          </div>
        `).join("")}
      </div>
    </div>
  `:"";return`
    ${x}

    ${C}

    <div class="bento-grid">
      ${!r?'<section id="tool-pins-panel" class="bento-card col-12"></section>':""}
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
          ${d}
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
          <span style="font-size: 11px; color: var(--text-dim); margin-left: 8px;">Keys masked in logs and audit records</span>
        </div>
        <div style="display: flex; gap: 8px; flex-wrap: wrap; margin: 12px 0;">
          ${f}
        </div>
        <div style="display: flex; gap: 8px; margin-top: 14px; max-width: 420px;">
          <input type="text" class="form-input" id="policy-new-redact" placeholder="e.g. token, api_key, password, secret" onkeydown="if(event.key==='Enter') window.app.submitPolicyRule('redact')">
          <button class="btn btn-ghost" onclick="window.app.submitPolicyRule('redact')">Add Key</button>
        </div>
      </div>

      ${!r?ce(a.resultSanitization):""}
      ${!r?pe(a.webhook,e.config.rbac?.enabled===!0):""}

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
  `}var Qe={missing:"Approval needed",matching:"Approved",changed:"Definition changed",unavailable:"Unavailable"};function Se(e,t){let s=t?JSON.stringify(t.value,null,2):"No definition available";return`
    <div class="tool-pin-definition">
      <strong>${e}</strong>
      ${t?"<code>"+i(t.digest)+"</code>":""}
      <pre tabindex="0">${i(s)}</pre>
    </div>
  `}function Ye(e,t){let s=e.tool&&e.current&&e.status!=="matching";return`
    <details class="tool-pin-review">
      <summary><code>${i(e.server)}${e.tool?" / "+i(e.tool):""}</code>
        <span class="tool-pin-status" data-status="${i(e.status)}">${Qe[e.status]}</span>
      </summary>
      <div class="tool-pin-comparison">
        ${Se("Observed definition",e.current)}
        ${Se("Approved definition",e.approved)}
      </div>
      ${s?'<button type="button" class="btn btn-primary" data-approve-pin="'+t+'">Approve this definition</button>':""}
    </details>
  `}function Ee(e){e.innerHTML=`
    <h2 class="stat-label">Tool definition pins</h2>
    <form class="trust-auth-form">
      <button class="btn btn-ghost" type="submit">Load definitions</button>
    </form>
    <p class="tool-pin-feedback" role="status" aria-live="polite">Load definitions for servers with tool pinning enabled.</p>
    <div class="tool-pin-list"></div>
  `;let t=e.querySelector("form"),s=e.querySelector('[role="status"]'),r=e.querySelector(".tool-pin-list"),a=[],o=!1;async function n(c){let g=await y.listToolPins();a=g.pins??[],r.innerHTML=a.map(Ye).join(""),s.textContent=g.error??c??(a.length?`${a.length} definitions and server statuses loaded.`:"No tool definitions are awaiting review.")}async function l(c){if(o)return;o=!0,e.setAttribute("aria-busy","true");try{await c()}catch{s.textContent="Could not reach Warmplane. Load definitions again when the daemon is available."}finally{e.removeAttribute("aria-busy"),o=!1}}if(t.addEventListener("submit",(c)=>{c.preventDefault(),r.replaceChildren(),s.textContent="Loading definitions…",l(()=>n())}),r.addEventListener("click",(c)=>{if(!(c.target instanceof Element))return;let g=c.target.closest("[data-approve-pin]");if(!g)return;let u=a[Number(g.dataset.approvePin)];if(!u?.tool||!u.current)return;let m={server:u.server,tool:u.tool,digest:u.current.digest};l(async()=>{s.textContent="Saving approval…";let v=await y.approveToolPin(m);await n(v.error??"Definition approved.")})}),y.hasTrustToken())l(()=>n())}function Ce(){return`
    <section id="reversibility-editor" data-alias-form-section="tool" style="margin-top:16px;padding:16px;border:1px solid var(--border);border-radius:var(--radius-md);background:var(--surface-hover);">
      <div style="display:flex;justify-content:space-between;gap:16px;align-items:flex-start;margin-bottom:12px;">
        <div>
          <h3 style="margin:0 0 4px;font-size:14px;">Execution reversibility</h3>
          <p class="text-dim" style="margin:0;font-size:11px;">Define how Warmplane handles this capability after it is called.</p>
        </div>
        <label for="reversibility-class" style="min-width:220px;font-size:11px;color:var(--text-dim);">Contract class
          <select class="form-input" id="reversibility-class" onchange="window.app.updateReversibilityEditor()" style="display:block;margin-top:4px;">
            <option value="">Not configured</option>
            <option value="delayable">Delayable · hold for cancellation</option>
            <option value="reversible">Reversible · restore prior state</option>
            <option value="compensable">Compensable · apply a compensating action</option>
            <option value="irreversible">Irreversible · explicitly no inverse</option>
          </select>
        </label>
      </div>
      <p id="reversibility-class-help" class="text-dim" style="font-size:11px;margin:0 0 12px;">No reversibility contract is currently configured.</p>

      <div data-reversibility-class="delayable" hidden>
        ${re("hold-window-secs","Hold window (seconds)","Duration the call waits before dispatch; must be positive.")}
      </div>

      <div data-reversibility-class="compensable" hidden>
        ${re("daily-limit","Daily limit","Maximum compensable calls in the rolling daily window; must be positive.")}
      </div>

      <div data-reversibility-class="reversible compensable" hidden>
        <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:12px;margin-bottom:14px;">
          ${re("ttl-secs","Undo window (seconds)","Optional lifetime for the registered inverse; must be positive when set.")}
          ${Ie("inverse-capability","Inverse capability","Canonical capability used to reverse or compensate for the call.")}
        </div>
        ${ae("inverse-args","Inverse arguments template","JSON object supporting argument, result, and snapshot template references.",`{
  "id": "\${args.id}",
  "value": "\${snapshot}"
}`)}
        ${Te("pre-snapshot","Capture pre-call state","Read state before the forward call. Its captured value can be used through ${snapshot} in the inverse arguments.")}
        ${Te("pre-undo-check","Check for state drift before undo","Read current state immediately before undo; a mismatch blocks the inverse.")}
        <section style="padding:12px 0 0;border-top:1px solid var(--border);margin-top:12px;">
          <label style="display:flex;gap:8px;align-items:center;font-size:12px;color:var(--text-main);">
            <input type="checkbox" id="test-target-enabled" onchange="window.app.updateReversibilityEditor()">
            Enable isolated undo contract testing
          </label>
          <div data-optional-field="test-target" hidden style="margin-top:10px;">
            ${Ae("test-target-server","Dedicated test server","Server used only for the isolated forward/snapshot/inverse test.")}
            ${ae("test-target-args","Fixed test arguments","JSON object sent to the forward call on the dedicated test server.",`{
  "id": "UNDO-TEST-1"
}`)}
          </div>
          <p class="text-dim" style="font-size:10px;margin:8px 0 0;">Contract testing requires a pre-call snapshot and a dedicated non-production server.</p>
        </section>
      </div>
    </section>
  `}function re(e,t,s){return`<label for="${e}" style="display:block;font-size:11px;color:var(--text-dim);margin-bottom:12px;">${t}
    <input class="form-input" id="${e}" type="number" min="1" step="1" style="display:block;margin-top:4px;">
    <span class="text-dim" style="display:block;margin-top:3px;font-size:10px;">${s}</span>
  </label>`}function Ae(e,t,s){return`<label for="${e}" style="display:block;font-size:11px;color:var(--text-dim);margin-bottom:12px;">${t}
    <input class="form-input" id="${e}" type="text" style="display:block;margin-top:4px;">
    <span class="text-dim" style="display:block;margin-top:3px;font-size:10px;">${s}</span>
  </label>`}function ne(e,t,s=""){return`<div class="capability-autocomplete">
    <input class="form-input" id="${e}" type="text" role="combobox" aria-label="${t}" aria-autocomplete="list" aria-expanded="false" aria-controls="${e}-suggestions" autocomplete="off" placeholder="${s}">
    <div id="${e}-suggestions" role="listbox" aria-label="${t} suggestions" class="capability-suggestions" hidden></div>
  </div>`}function Ie(e,t,s){return`<div class="form-group">
    <label class="form-label" for="${e}">${t}</label>
    ${ne(e,t)}
    <span class="text-dim" style="font-size:10px;">${s}</span>
  </div>`}function ae(e,t,s,r){return`<label for="${e}" style="display:block;font-size:11px;color:var(--text-dim);margin:0 0 12px;">${t}
    <textarea class="form-input" id="${e}" rows="4" spellcheck="false" placeholder='${r}' style="display:block;margin-top:4px;width:100%;font-family:var(--ff-mono);font-size:11px;resize:vertical;"></textarea>
    <span class="text-dim" style="display:block;margin-top:3px;font-size:10px;">${s} Must be a JSON object.</span>
  </label>`}function Te(e,t,s){let r=e;return`<section style="padding:12px 0 0;border-top:1px solid var(--border);margin-top:12px;">
    <label style="display:flex;gap:8px;align-items:center;font-size:12px;color:var(--text-main);">
      <input type="checkbox" id="${e}-enabled" onchange="window.app.updateReversibilityEditor()"> ${t}
    </label>
    <div data-optional-field="${e}" hidden style="margin-top:10px;">
      ${Ie(`${r}-capability`,"Read capability","Canonical read capability to invoke.")}
      ${ae(`${r}-args`,"Read arguments template",s,`{
  "id": "\${args.id}"
}`)}
      ${Ae(`${r}-capture-json-path`,"Captured JSONPath (optional)","Optional JSONPath to select the value to snapshot or hash.")}
    </div>
  </section>`}var _e=[{kind:"tool",label:"Tool",color:"cyan-400",key:"capabilityAliases"},{kind:"resource",label:"Resource",color:"green-400",key:"resourceAliases"},{kind:"prompt",label:"Prompt",color:"amber-300",key:"promptAliases"}];function X(e,t){let s=_e.find((a)=>a.kind===e),r=s&&p.getState().config[s.key];return r&&Object.hasOwn(r,t)?r[t]:void 0}function Pe(e){let{config:t,configPath:s}=p.getState(),r=_e.flatMap(({kind:a,label:o,color:n,key:l})=>Object.entries(t[l]||{}).map(([c,g])=>{let u=typeof g==="object"?g:void 0,m=a==="tool"?u?.reversibility:void 0,v=e.get(c),d=v?.running||v?.savedAlias===JSON.stringify(g)?v:void 0,f=`data-alias-kind="${a}" data-alias-name="${i(c)}"`;return`
        <div class="feed-row alias-row">
          <span style="color:var(--${n});">${o}</span>
          <div class="alias-row-name">
            <strong>${i(c)}</strong>
            ${m?`<span class="brand-badge">${i(m.class)}</span>`:""}
            ${u?.passthrough?'<span class="brand-badge">Passthrough</span>':""}
          </div>
          <div class="alias-row-target">
            <div>${i(u?.target??g)}</div>
            ${u?.summary?`<div class="text-dim">${i(u.summary)}</div>`:""}
          </div>
          <div class="alias-row-actions">
            <button type="button" class="btn btn-ghost" data-alias-action="edit" ${f}>Edit</button>
            ${m?.testTarget?`<button type="button" class="btn btn-ghost" data-alias-action="test" ${f} aria-disabled="${!!d?.running}">${d?.running?"Testing…":"Test saved contract"}</button>`:""}
            <button type="button" class="btn btn-ghost" data-alias-action="delete" ${f} aria-label="Delete ${i(c)}">Delete</button>
          </div>
          ${m?.testTarget?`<div role="status" class="alias-test-status${d?.failed?" alias-test-failed":""}">${i(d?.message)}</div>`:""}
        </div>`}));if(!r.length)return`<div class="bento-card text-dim">No aliases configured in ${i(s)}. Use the form above to add one.</div>`;return`<div class="alias-list">${r.join("")}</div>`}function Re(){return`
    <!-- Sub-header -->
    <div style="margin-bottom: 16px; font-size: 12px; color: var(--text-dim);">
      Shorten capability IDs and supply custom descriptions to prune prompt tokens. Enable <b>Native Passthrough</b> for tools to expose them directly in <code style="color: var(--amber-300);">tools/list</code> with their native schema.
    </div>

    <!-- Quick Add / Edit Form -->
    <form id="alias-form" class="bento-card" novalidate style="margin-bottom: 20px; overflow: visible;">
      <div class="stat-header" style="margin-bottom: 12px; display: flex; justify-content: space-between; align-items: center;">
        <span class="stat-label" id="alias-form-title">Create New Alias</span>
        <button type="button" id="alias-cancel-btn" class="btn btn-ghost" style="padding: 2px 8px; font-size: 11px; color: var(--text-dim);" onclick="window.app.resetAliasForm()">Cancel</button>
      </div>
      <div class="alias-form-fields">
        <label for="alias-kind">Alias kind
        <select class="form-input" id="alias-kind" onchange="window.app.updateAliasKind(this.value)">
          <option value="tool">Tool / Capability</option>
          <option value="resource">Resource</option>
          <option value="prompt">Prompt</option>
        </select>
        </label>
        <label for="alias-name">Public alias
        <input type="text" class="form-input" id="alias-name" placeholder="e.g. search">
        </label>
        <div>
          <label for="alias-target">Canonical target</label>
          ${ne("alias-target","Canonical target","e.g. semble.search")}
        </div>
      </div>
      <div style="display: grid; grid-template-columns: 1fr auto; gap: 12px; align-items: center;">
        <input type="text" class="form-input" id="alias-summary" aria-label="Alias summary" placeholder="Optional custom description / prompt instruction (e.g. Fast hybrid code search)" style="font-size: 12px;">
        <label id="alias-passthrough-container" style="display: flex; align-items: center; gap: 6px; font-size: 12px; color: var(--text-main); cursor: pointer; user-select: none; white-space: nowrap;">
          <input type="checkbox" id="alias-passthrough" style="accent-color: var(--amber-400); cursor: pointer;">
          <span>⚡ Native Passthrough Tool</span>
        </label>
      </div>
      ${Ce()}
      <div id="alias-form-error" role="alert" aria-live="polite" hidden style="margin-top:12px;padding:10px 12px;border:1px solid rgba(248,113,113,.35);border-radius:var(--radius-sm);background:rgba(248,113,113,.1);color:var(--red-400);font-size:12px;"></div>
      <button type="submit" id="alias-save-btn" class="btn btn-primary" style="margin-top:12px;">Save</button>
    </form>
    <div id="alias-list"></div>
  `}function ie(e){return document.getElementById(e.getAttribute("aria-controls")||"")}function H(e){let t=ie(e);if(t)t.hidden=!0;e.setAttribute("aria-expanded","false"),e.removeAttribute("aria-activedescendant")}function Le(e){H(e);let t=ie(e),s=e.value.trim().toLowerCase();if(!t||s.length<2)return;let r=p.getState().capabilities.filter((a)=>[a.id,a.summary,a.description,a.server].some((o)=>o?.toLowerCase().includes(s))).slice(0,8);if(!r.length)return;t.innerHTML=r.map((a,o)=>`
    <div id="${e.id}-option-${o}" role="option" aria-selected="false"
         data-capability-id="${i(a.id)}" class="capability-option">
      <span><strong>${i(a.id)}</strong>
        <small>${i(a.summary||a.description||"")}</small></span>
      <span class="capability-option-server">${i(a.server||"local")}</span>
    </div>`).join(""),t.hidden=!1,e.setAttribute("aria-expanded","true")}function Me(e,t){e.value=t.dataset.capabilityId??e.value,H(e),e.dispatchEvent(new Event("change",{bubbles:!0}))}function ze(e){let t=(s)=>s instanceof HTMLInputElement&&s.getAttribute("role")==="combobox"?s:null;for(let s of["input","focusin","focusout"])e.addEventListener(s,(r)=>{let a=t(r.target);if(!a)return;if(s==="focusout")H(a);else Le(a)});e.addEventListener("mousedown",(s)=>{if(s.target instanceof Element&&s.target.closest("[data-capability-id]"))s.preventDefault()}),e.addEventListener("click",(s)=>{if(!(s.target instanceof Element))return;let r=s.target.closest("[data-capability-id]"),a=t(r?.parentElement?.previousElementSibling??null);if(r&&a)Me(a,r)}),e.addEventListener("keydown",(s)=>{let r=t(s.target);if(!r||s.isComposing)return;let a=ie(r);if(!a)return;let o=s.key==="ArrowDown"||s.key==="ArrowUp";if(o&&a.hidden)Le(r);if(a.hidden)return;let n=r.getAttribute("aria-activedescendant"),l=[...a.querySelectorAll('[role="option"]')],c=l.findIndex((g)=>g.id===n);if(o&&l.length){s.preventDefault();let g=s.key==="ArrowDown"?(c+1)%l.length:c<=0?l.length-1:c-1;l[c]?.setAttribute("aria-selected","false"),l[g].setAttribute("aria-selected","true"),r.setAttribute("aria-activedescendant",l[g].id),l[g].scrollIntoView({block:"nearest"})}else if(s.key==="Enter"){if(s.preventDefault(),c>=0)Me(r,l[c])}else if(s.key==="Escape")s.preventDefault(),H(r);else if(s.key==="Tab")H(r)})}function He(){let e=p.getState(),t=e.config,s=t.profiles||{},r=Object.entries(s),a=t.mcpServers||{},o=e.activeProfile,n="";if(r.length===0)n=`
      <div style="padding: 40px; text-align: center; color: var(--text-dim); background: var(--surface-card); border-radius: var(--radius-md); border: 1px dashed var(--border);">
        <div style="font-size: 15px; color: var(--text-main); font-weight: 600; margin-bottom: 8px;">No Profiles Configured</div>
        <p style="font-size: 12px; margin-bottom: 20px; max-width: 480px; margin-left: auto; margin-right: auto;">
          Profiles allow Warmplane to serve multiple task-relevant server constellations (e.g. <code>coding</code>, <code>support</code>, <code>data</code>) from one running daemon process.
        </p>
        <button class="btn btn-primary" onclick="window.app.openAddProfileModal()">+ Create First Profile</button>
      </div>
    `;else n=r.map(([l,c])=>{let g=o===l,u=c.servers.map((h)=>`<span class="brand-badge" style="${a[h]?"color: var(--cyan-400); border-color: rgba(34, 211, 238, 0.25); background: rgba(34, 211, 238, 0.05);":"color: var(--red-400); border-color: rgba(248, 113, 113, 0.3); background: rgba(248, 113, 113, 0.05);"}">${i(h)}</span>`).join(" "),m=(e.capabilities||[]).filter((h)=>c.servers.includes(h.server)).length,v=!!c.policy,d=c.policy?.allow?.length||0,f=c.policy?.deny?.length||0,x=(c.policy?.require_approval||c.policy?.requireApproval||[]).length,b=(c.policy?.redact_keys||c.policy?.redactKeys||[]).length;return`
        <div class="bento-card" style="margin-bottom: 14px; border-left: ${g?"3px solid var(--amber-400)":"1px solid var(--border)"};">
          <div style="display: flex; justify-content: space-between; align-items: flex-start;">
            <div>
              <div style="display: flex; align-items: center; gap: 8px; margin-bottom: 6px;">
                <span style="font-size: 16px; font-weight: 700; color: var(--text-main); font-family: var(--ff-mono);">${i(l)}</span>
                ${g?'<span class="brand-badge" style="color: var(--amber-400); border-color: rgba(245, 158, 11, 0.4); background: rgba(245, 158, 11, 0.1);">ACTIVE IN UI</span>':""}
                <span class="brand-badge">${c.servers.length} server${c.servers.length===1?"":"s"}</span>
                <span class="brand-badge" style="color: var(--text-dim);">${m} capabilities</span>
                ${v?'<span class="brand-badge" style="color: var(--green-400); border-color: rgba(34, 197, 94, 0.3); background: rgba(34, 197, 94, 0.08);">CUSTOM POLICY</span>':""}
              </div>
              <div style="font-size: 12px; color: var(--text-muted); margin-bottom: 10px;">
                ${i(c.description||"No description provided")}
              </div>
              <div style="display: flex; flex-wrap: wrap; gap: 6px; align-items: center; margin-bottom: ${v?"8px":"0"};">
                <span style="font-size: 11px; color: var(--text-dim); font-weight: 600; text-transform: uppercase;">Servers:</span>
                ${u||'<span style="font-size: 11px; color: var(--text-dim);">None</span>'}
              </div>
              ${v?`
                <div style="display: flex; flex-wrap: wrap; gap: 6px; align-items: center; font-size: 11px;">
                  <span style="color: var(--text-dim); font-weight: 600; text-transform: uppercase;">Policy Overlay:</span>
                  ${d>0?`<span class="brand-badge" style="color: var(--green-400);">Allow: ${d}</span>`:""}
                  ${f>0?`<span class="brand-badge" style="color: var(--red-400);">Deny: ${f}</span>`:""}
                  ${x>0?`<span class="brand-badge" style="color: var(--amber-400);">HITL: ${x}</span>`:""}
                  ${b>0?`<span class="brand-badge" style="color: var(--text-muted);">Redact: ${b}</span>`:""}
                  ${d===0&&f===0&&x===0&&b===0?'<span style="color: var(--text-dim);">Configured</span>':""}
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

    ${n}
  `}function qe(){let e=p.getState(),t=e.secrets||[],s=e.config.mcpServers||{},r=[];for(let[u,m]of Object.entries(s)){let v=J(u,m.command,m.args);if(v){let d=Object.keys(m.env||{});for(let f of v.envFields)if(f.required&&!d.includes(f.key))r.push({server:u,key:f.key,uri:"(Not Configured)",is_vault:!1,exists:!1,backend:"Required Variable",display:`Required by ${v.name} template (${f.label})`,is_unconfigured_requirement:!0})}}let a=[...t,...r],o=a.length,n=a.filter((u)=>u.is_vault&&u.exists!==!1).length,l=a.filter((u)=>u.exists===!1).length,c=a.filter((u)=>!u.is_vault&&u.exists!==!1).length,g=a.length===0?`
    <div style="padding: 32px; text-align: center; color: var(--text-dim);">
      No environment variables or secrets configured in active servers.
    </div>
  `:a.map((u)=>{let m='<span class="brand-badge" style="color: var(--red-400); border-color: rgba(248, 113, 113, 0.4); background: rgba(248, 113, 113, 0.1);">Plaintext (Unsecured)</span>',v=u.exists===!1;if(u.is_unconfigured_requirement)m='<span class="brand-badge" style="color: var(--red-400); border-color: rgba(248, 113, 113, 0.4); background: rgba(248, 113, 113, 0.1);">⚠️ Not Configured (Required)</span>';else if(v)m=`<span class="brand-badge" style="color: var(--amber-300); border-color: rgba(245, 158, 11, 0.4); background: rgba(245, 158, 11, 0.1);">⚠️ Missing from ${i(u.backend)}</span>`;else if(u.is_vault)m=`<span class="brand-badge" style="color: var(--green-400); border-color: rgba(52, 211, 153, 0.3); background: rgba(52, 211, 153, 0.1);">\uD83D\uDD12 ${i(u.backend)}</span>`;return`
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
        <div class="stat-value" style="color: var(--cyan-400);">${o}</div>
        <div class="stat-sub">Across all configured MCP servers</div>
      </div>
      <div class="bento-card col-4">
        <div class="stat-label">Secured via Vault / Keychain</div>
        <div class="stat-value" style="color: var(--green-400);">${n}</div>
        <div class="stat-sub">Zero-disk plaintext exposure</div>
      </div>
      <div class="bento-card col-4">
        <div class="stat-label">Missing or Unsecured</div>
        <div class="stat-value" style="color: ${l+c>0?"var(--red-400)":"var(--green-400)"};">${l+c}</div>
        <div class="stat-sub">${l>0?`${l} missing required key(s)`:c>0?"Recommend migrating to Keychain":"All credentials protected"}</div>
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
  `}class je{activeTemplateCategory="all";activeTemplateFilter="";selectedTemplate=null;pendingClientId=null;taskRefresh=0;approvalRefresh=0;configIdentity=0;savingTrustPolicy=!1;async init(){let e=window.location.port?`:${window.location.port}`:"",t=document.getElementById("daemon-port-label");if(t)t.textContent=`Daemon ${e}`;await this.refreshData(),this.initSSE(),this.bindUndoFeedActions(),this.bindApprovalActions(),this.render(),p.subscribe(()=>{this.render()}),window.setInterval(()=>{if(p.getState().activeTab==="undo")this.render()},1000)}auditSearchTimeout=null;bindUndoFeedActions(){let e=document.getElementById("app-main");if(!e)return;e.addEventListener("click",(t)=>{let s=t.target;if(!(s instanceof Element))return;let r=s.closest("[data-undo-action], [data-playground-action]");if(!r||!e.contains(r))return;if(r.dataset.playgroundAction==="test-undo"&&r.dataset.undoAlias){this.testUndoContract(r.dataset.undoAlias);return}let a=r.dataset.undoAction,o=r.dataset.undoTarget;if(a==="previous"||a==="next"){let n=p.getState();if(!n.undoLoading)this.refreshUndo(Math.max(0,n.undoOffset+(a==="next"?50:-50)))}else if(a==="refresh")this.refreshUndo();else if(a==="cancel-task"&&o)this.cancelHeldTask(o);else if(a==="rollback"&&o)this.rollbackWorkItem(o);else if(a==="execute"&&o)this.executeUndo(o)})}bindApprovalActions(){let e=document.getElementById("app-main");e?.addEventListener("submit",(t)=>{if(!(t.target instanceof HTMLFormElement)||t.target.id!=="approval-auth-form")return;t.preventDefault();let s=t.target.querySelector("#approval-operator-token");if(s?.value.trim())this.configIdentity++,y.setApprovalToken(s.value.trim()),s.value="",p.setState({approvals:[],tasks:[],approvalError:null,taskError:null}),this.refreshData();else Promise.all([this.refreshApprovals(),this.refreshTasks()])}),e?.addEventListener("click",(t)=>{if(!(t.target instanceof Element))return;let s=t.target.closest("[data-approval-target]"),r=s?.dataset.approvalTarget;if(!r)return;if(s.dataset.approvalChoice==="approve")this.submitApproval(r);if(s.dataset.approvalChoice==="reject")this.promptReject(r)})}async refreshData(){try{let e=p.getState(),t=this.configIdentity,s=e.auditFilters,r=e.activeProfile||void 0,[a,o,n,l,c,g,u,m,v]=await Promise.all([y.getConfig(),y.listCapabilities(r),y.listResources(r),y.listPrompts(r),y.getCatalogEvents(),y.listAuditEvents({server_id:s.serverId!=="all"?s.serverId:void 0,event_type:s.eventType!=="all"?s.eventType:void 0,status:s.status!=="all"?s.status:void 0,search:s.search.trim()?s.search.trim():void 0,limit:s.limit,offset:s.offset}),y.getAuditStats(),y.getClients().catch(()=>({ok:!1,clients:[]})),y.getSecrets().catch(()=>({ok:!1,secrets:[],keychain_service:"warmplane"})),this.refreshApprovals(),this.refreshTasks()]);if(t!==this.configIdentity||e.activeProfile!==p.getState().activeProfile)return;if(m&&m.ok&&Array.isArray(m.clients))p.setState({clients:m.clients});if(v&&v.ok&&Array.isArray(v.secrets))p.setState({secrets:v.secrets});if(a.ok)p.setState({configReady:!0,configError:null,configPath:a.config_path,config:a.config,serverStatuses:a.server_statuses||{},circuitBreakers:a.circuit_breakers||[],metrics:{totalCatalogRequests:a.metrics?.total_catalog_requests||0,totalEtagHits:a.metrics?.total_etag_hits||0,totalToolCalls:a.metrics?.total_tool_calls||0,totalToolDurationUs:a.metrics?.total_tool_duration_us||0}});else p.setState({configReady:!1,configError:z(a,"Configuration could not be loaded.")});if(o&&Array.isArray(o.capabilities)){let d=p.getState().selectedCapabilityId,x=o.capabilities.some((b)=>b.id===d)?d:o.capabilities.length>0?o.capabilities[0].id:null;p.setState({capabilities:o.capabilities,capabilitiesHiddenByPolicy:o.hidden_by_policy||0,selectedCapabilityId:x})}if(n&&Array.isArray(n.resources)){let d=p.getState().selectedResourceId,x=n.resources.some((b)=>b.uri===d||b.id===d)?d:n.resources.length>0?n.resources[0].uri||n.resources[0].id||null:null;p.setState({resources:n.resources,resourcesHiddenByPolicy:n.hidden_by_policy||0,selectedResourceId:x})}if(l&&Array.isArray(l.prompts)){let d=p.getState().selectedPromptId,x=l.prompts.some((b)=>b.name===d||b.id===d)?d:l.prompts.length>0?l.prompts[0].name||l.prompts[0].id||null:null;p.setState({prompts:l.prompts,promptsHiddenByPolicy:l.hidden_by_policy||0,selectedPromptId:x})}if(c&&Array.isArray(c.events))p.setState({catalogEvents:c.events});if(await this.refreshUndo(),g&&Array.isArray(g.events))p.setState({auditEvents:g.events,auditTotal:g.total??g.events.length});if(u&&u.ok)p.setState({auditStats:u})}catch(e){console.error("Failed to fetch daemon state:",e)}}async refreshAuditEvents(){try{let t=p.getState().auditFilters,[s,r]=await Promise.all([y.listAuditEvents({server_id:t.serverId!=="all"?t.serverId:void 0,event_type:t.eventType!=="all"?t.eventType:void 0,status:t.status!=="all"?t.status:void 0,search:t.search.trim()?t.search.trim():void 0,limit:t.limit,offset:t.offset}),y.getAuditStats()]);if(s&&Array.isArray(s.events))p.setState({auditEvents:s.events,auditTotal:s.total??s.events.length});if(r&&r.ok)p.setState({auditStats:r})}catch(e){console.error("Failed to refresh audit events:",e)}}handleAuditSearchInput(e){let s={...p.getState().auditFilters,search:e,offset:0};p.setState({auditFilters:s}),clearTimeout(this.auditSearchTimeout),this.auditSearchTimeout=setTimeout(()=>{this.refreshAuditEvents()},250)}handleAuditStatusFilter(e){let t=p.getState();p.setState({auditFilters:{...t.auditFilters,status:e,offset:0}}),this.refreshAuditEvents()}handleAuditEventTypeFilter(e){let t=p.getState();p.setState({auditFilters:{...t.auditFilters,eventType:e,offset:0}}),this.refreshAuditEvents()}handleAuditServerFilter(e){let t=p.getState();p.setState({auditFilters:{...t.auditFilters,serverId:e,offset:0}}),this.refreshAuditEvents()}handleAuditPageSize(e){let t=parseInt(e,10)||25,s=p.getState();p.setState({auditFilters:{...s.auditFilters,limit:t,offset:0}}),this.refreshAuditEvents()}clearAuditFilters(){let e=p.getState();p.setState({auditFilters:{search:"",status:"all",eventType:"all",serverId:"all",limit:e.auditFilters.limit||25,offset:0}}),this.refreshAuditEvents()}auditPrevPage(){let e=p.getState(),{limit:t,offset:s}=e.auditFilters,r=Math.max(0,s-t);if(r!==s)p.setState({auditFilters:{...e.auditFilters,offset:r}}),this.refreshAuditEvents()}auditNextPage(){let e=p.getState(),{limit:t,offset:s}=e.auditFilters,r=e.auditTotal;if(s+t<r)p.setState({auditFilters:{...e.auditFilters,offset:s+t}}),this.refreshAuditEvents()}auditGoToPage(e){let t=p.getState(),{limit:s}=t.auditFilters,r=Math.max(0,(e-1)*s);p.setState({auditFilters:{...t.auditFilters,offset:r}}),this.refreshAuditEvents()}selectAuditEvent(e){if(!e){p.setState({auditSelectedEvent:null});return}let s=p.getState().auditEvents.find((r)=>r.id===e)||null;p.setState({auditSelectedEvent:s})}async verifyAuditChain(){try{let e=await y.verifyAuditChain();if(e&&e.report)p.setState({auditVerification:e.report})}catch(e){console.error("Failed to verify audit chain:",e)}}async refreshCollection(e,t,s){try{let r=t(await e());if(r)p.setState(r)}catch(r){console.error(s,r)}}async refreshApprovals(){let e=++this.approvalRefresh,t="Could not load approval tickets.",s=await y.listApprovals().catch(()=>({ok:!1,approvals:[],error:"Could not load approval tickets."}));if(e===this.approvalRefresh)p.setState({approvals:s.approvals??[],approvalError:s.ok?null:z(s,"Could not load approval tickets.")})}openTrustAccess(){this.closeModals();let e=document.getElementById("modal-trust-access"),t=document.querySelector("#trust-access-token"),s=document.querySelector("#trust-disconnect"),r=document.getElementById("trust-access-status");if(s)s.hidden=!y.hasTrustToken();if(r)r.textContent=y.hasTrustToken()?"Connected for this tab.":"Use a trust administrator credential to load and edit protected configuration.";if(t)t.value="";e?.classList.add("active"),t?.focus()}async connectTrustAdministrator(e){e.preventDefault();let t=e.currentTarget,s=t.querySelector("#trust-access-token"),r=t.querySelector("#trust-access-status"),a=t.querySelector("button[type=submit]"),o=s.value.trim();if(!o||a.disabled)return;a.disabled=!0,s.readOnly=!0,r.textContent="Checking administrator access…";try{let n=await y.listToolPins(o);if(n.error){r.textContent=n.error;return}s.value="",await this.setTrustAccess(o)}catch{r.textContent="Could not verify administrator access. Check the daemon connection."}finally{a.disabled=!1,s.readOnly=!1}}async disconnectTrustAdministrator(){await this.setTrustAccess("")}async setTrustAccess(e){this.configIdentity++,y.setTrustToken(e),document.getElementById("tool-pins-panel")?.remove(),p.setState({config:{mcpServers:{}},configReady:!1,configError:null,tasks:[],approvals:[]}),this.closeModals();let t=document.getElementById("trust-access-button");if(t)t.textContent=e?"Trust access · Connected":"Trust access";await this.refreshData()}initSSE(){try{let e=new EventSource("/v1/resources/updates");e.onmessage=(t)=>{p.addEventLog("SSE","/v1/resources/updates","UPDATED","0.1ms"),this.refreshData()},e.addEventListener("action_capsule",()=>{this.refreshUndo(),this.refreshTasks()}),e.addEventListener("error",()=>{if(p.getState().activeTab==="undo")this.refreshUndo()})}catch(e){console.warn("SSE connection unavailable")}}undoRequestGeneration=0;async refreshUndo(e=p.getState().undoOffset){let t=++this.undoRequestGeneration,s=e!==p.getState().undoOffset;p.setState({undoLoading:!0,undoOffset:e,...s?{undoEntries:[]}:{}});try{let r=await y.listUndo(e,50);if(t!==this.undoRequestGeneration)return;if(r.ok&&r.result&&e>0&&e>=r.result.total){if(e=Math.max(0,Math.floor((r.result.total-1)/50)*50),r=await y.listUndo(e,50),t!==this.undoRequestGeneration)return}if(r.ok&&r.result&&Array.isArray(r.result.entries))p.setState({undoEntries:r.result.entries,undoOffset:e,undoTotal:r.result.total,undoLoading:!1,undoError:null});else p.setState({undoLoading:!1,undoError:r.error?.message||r.error?.code||"Undo entries could not be loaded."})}catch(r){if(t===this.undoRequestGeneration)p.setState({undoLoading:!1,undoError:r?.message||"Undo entries could not be loaded."})}}async executeUndo(e){if(!confirm("Run the configured inverse for this action?"))return;let t=await y.executeUndo(e);if(!t.ok)alert(`Undo failed: ${t.error?.message||t.error?.code||"Request failed"}`);await this.refreshUndo()}async rollbackWorkItem(e){if(!confirm("Run configured inverses for this work item in reverse order?"))return;let t=await y.rollbackWorkItem(e);if(!t.ok)alert(`Rollback incomplete: ${t.error?.message||t.error?.code||"One or more inverses failed"}`);await this.refreshUndo()}async cancelHeldTask(e){if(!confirm("Cancel this send before it is released?"))return;let t=await y.cancelTask(e,"Cancelled from Control Deck Action Feed",p.getState().activeProfile??void 0);if(!t.ok)alert(`Cancellation failed: ${t.error?.message||t.error?.code||"Request failed"}`);await this.refreshTasks()}async testUndoContract(e){if(!confirm(`Run the configured forward and inverse cycle on the test server for '${e}'?`))return;let t=await y.testUndoContract(e),s=t.result;if(t.ok&&s?.observationally_equivalent)alert(`Undo contract passed on ${s.test_server}. The observed state matches its snapshot.`);else alert(`Undo contract failed: ${s?.error_code||t.error?.message||"state did not return to its snapshot"}`)}switchTab(e){p.setState({activeTab:e}),this.refreshData()}render(){let e=p.getState(),t=document.getElementById("app-main");if(!t)return;let s=(e.tasks||[]).filter((c)=>c.status==="input_required").length,r=(e.approvals||[]).filter((c)=>c.status==="pending").length,a=Math.max(s,r),o=document.getElementById("nav-approvals-badge");if(o)o.textContent=a>0?`${a}`:"",o.style.display=a>0?"inline-block":"none";document.querySelectorAll(".nav-item").forEach((c)=>{let g=c.getAttribute("data-tab");if(g===e.activeTab||e.activeTab==="tasks"&&g==="approvals"||e.activeTab==="approvals"&&g==="tasks")c.classList.add("active");else c.classList.remove("active")});let n=document.getElementById("top-title"),l={overview:"Overview Cockpit",servers:"Server Hub & Connections",playground:"MCP Capability Playground",tasks:"SEP-2663 Tasks & HITL Review",undo:"Action Feed & Undo",approvals:"SEP-2663 Tasks & HITL Review",audit:"WORM Audit & Compliance Ledger",policy:"Security Governance & Redaction",secrets:"Native OS Keychain & Secrets Vault",aliases:"Facade & Alias Studio",profiles:"Server Constellation Profiles"};if(n)n.textContent=l[e.activeTab]||"Control Deck";switch(this.renderTopProfileSelector(),e.activeTab){case"overview":t.innerHTML=me();break;case"servers":t.innerHTML=ve();break;case"playground":t.innerHTML=fe();break;case"tasks":case"approvals":t.innerHTML=he(e);break;case"undo":t.innerHTML=we(e);break;case"audit":t.innerHTML=$e();break;case"policy":this.renderPolicyView(t);break;case"secrets":t.innerHTML=qe();break;case"aliases":this.renderAliasStudio(t);break;case"profiles":t.innerHTML=He();break}}toggleClientsCollapse(){let e=p.getState().clientsCollapsed;p.setState({clientsCollapsed:!e}),this.render()}async saveNewVaultSecret(){let e=document.getElementById("vault-new-key"),t=document.getElementById("vault-new-val"),s=document.getElementById("vault-new-service"),r=e?.value.trim(),a=t?.value.trim(),o=s?.value.trim()||"warmplane";if(!r||!a){alert("Key and secret value are required");return}try{let n=await y.saveSecret(r,a,o);if(n.ok){if(alert(`Secret '${r}' saved securely into OS Keychain!
Reference: ${n.uri}`),e)e.value="";if(t)t.value="";await this.refreshData()}else alert(`Failed to save secret: ${n.error}`)}catch(n){alert(`Error saving secret: ${n.message}`)}}async deleteVaultSecret(e,t){let s=t?`Are you sure you want to remove secret '${e}' from OS Keychain and server '${t}'?`:`Are you sure you want to remove secret '${e}' from OS Keychain?`;if(!confirm(s))return;try{let r=await y.deleteSecret(e);if(r.ok||r.error?.includes("not found")){if(t){let n=(p.getState().config.mcpServers||{})[t];if(n&&n.env&&n.env[e]){let l={...n.env};delete l[e];let c={...n,env:l};await y.upsertServer(t,c)}}await this.refreshData()}else alert(`Failed to delete secret: ${r.error}`)}catch(r){alert(`Error deleting secret: ${r.message}`)}}async removeSecretFromConfig(e,t){if(!confirm(`Remove environment variable '${t}' from '${e}' configuration?`))return;try{let a=(p.getState().config.mcpServers||{})[e];if(a&&a.env&&a.env[t]){let o={...a.env};delete o[t];let n={...a,env:o},l=await y.upsertServer(e,n);if(l.ok)await this.refreshData();else alert(`Failed to update server config: ${l.error}`)}}catch(s){alert(`Error removing secret from config: ${s.message}`)}}async quickVaultEnv(e,t){let s=prompt(`Enter secret value to store in OS Keychain for ${e}.${t}:`);if(!s)return;try{let r=await y.saveSecret(t,s,"warmplane");if(!r.ok){alert(`Failed to save to Keychain: ${r.error}`);return}let n=(p.getState().config.mcpServers||{})[e];if(n){let l={...n.env||{},[t]:`keychain://warmplane/${t}`},c={...n,env:l},g=await y.upsertServer(e,c);if(g.ok)await this.refreshData(),alert(`Successfully configured ${e}.${t} in OS Keychain!`);else alert(`Failed to update server config: ${g.error}`)}}catch(r){alert(`Error during migration: ${r.message}`)}}async refreshTasks(){let e=++this.taskRefresh,t=p.getState().activeProfile??void 0,s="Could not load tasks for this profile.",r=await y.listTasks(t).catch(()=>({ok:!1,tasks:[],error:"Could not load tasks for this profile."}));if(e===this.taskRefresh)p.setState({tasks:r.tasks??[],taskError:r.ok?null:z(r,"Could not load tasks for this profile.")})}filterTasksByStatus(e){p.setState({taskFilterStatus:e})}togglePlaygroundAsyncTask(e){p.setState({playgroundAsyncTask:e})}async submitTaskInputResponses(e){let s=p.getState().tasks.find((o)=>o.taskId===e)?.inputRequests||{},r=Object.keys(s),a={};if(r.length>0)for(let o of r){let n=s[o];if(n&&n.type==="approval_review"){let l=this.readTaskApproval(e,o);if(!l)return;a[o]=l}else{let l=document.getElementById(`task-input-${e}-${o}`);if(l){let c=l.value.trim();try{a[o]=JSON.parse(c)}catch{a[o]=c}}}}else{let o=document.getElementById(`task-raw-input-${e}`);if(o&&o.value.trim())try{Object.assign(a,JSON.parse(o.value.trim()))}catch{alert("Invalid JSON in raw input responses");return}}try{let o=await y.updateTask(e,a,p.getState().activeProfile??void 0);if(o.ok)await Promise.all([this.refreshTasks(),this.refreshApprovals()]);else alert(`Task update failed: ${o.error?.message||o.error||"Unknown error"}`)}catch(o){alert(`Error updating task: ${o.message}`)}}readTaskApproval(e,t){let s=document.getElementById(`task-input-${e}-${t}-decision`),r=document.getElementById(`task-input-${e}-${t}`);if(!s?.value){alert("Choose whether to approve or reject this invocation."),s?.focus();return}let a=s.value==="true",o=void 0;if(a&&r&&r.value.trim())try{o=JSON.parse(r.value.trim())}catch{alert("Invalid JSON in parameters editor");return}return{approved:a,modified_args:o,reason:a?void 0:"Operator rejected execution via Tasks review"}}async promptCancelTask(e){let t=prompt("Reason for cancelling task:");if(t===null)return;try{let s=await y.cancelTask(e,t||void 0,p.getState().activeProfile??void 0);if(s.ok)await this.refreshTasks();else alert(`Task cancellation failed: ${s.error?.message||s.error||"Unknown error"}`)}catch(s){alert(`Error cancelling task: ${s.message}`)}}async openTaskInspectorModal(e){this.closeModals();let t=document.getElementById("modal-task-inspector");if(!t)return;let s=document.getElementById("modal-task-title"),r=document.getElementById("modal-task-body"),a=document.getElementById("modal-task-footer");if(s)s.textContent=`Task Inspector: ${e}`;if(r)r.innerHTML=`
        <div style="padding: 30px; text-align: center; color: var(--text-dim); font-family: var(--ff-mono); font-size: 12px;">
          ⏳ Fetching task execution record...
        </div>
      `;t.classList.add("active");try{let o=await y.getTask(e,p.getState().activeProfile??void 0);if(!o.ok||!o.task){if(r)r.innerHTML=`
            <div style="background: rgba(248, 113, 113, 0.12); border: 1px solid rgba(248, 113, 113, 0.3); border-radius: var(--radius-sm); padding: 16px; color: var(--red-400);">
              <div style="font-weight: 700; margin-bottom: 6px;">Failed to load task snapshot</div>
              <div style="font-family: var(--ff-mono); font-size: 11.5px;">${i(o.error?.message||"Task not found in runtime registry")}</div>
            </div>
          `;return}let n=o.task,l=n.progress!==void 0?Math.round(n.progress*100):n.status==="completed"?100:n.status==="working"?50:0,c=Math.floor(Date.now()/1000),g=n.expiresAtEpochSecs?Math.max(0,n.expiresAtEpochSecs-c):n.ttlSeconds||300,u=n.status==="completed"||n.status==="cancelled"||n.status==="failed",m=n.status==="completed"?"var(--green-400)":n.status==="working"?"var(--cyan-400)":n.status==="input_required"?"var(--amber-300)":n.status==="cancelled"?"var(--text-muted)":"var(--red-400)",v=n.status==="completed"?"rgba(52, 211, 153, 0.15)":n.status==="working"?"rgba(56, 189, 248, 0.15)":n.status==="input_required"?"rgba(245, 158, 11, 0.18)":n.status==="cancelled"?"rgba(148, 163, 184, 0.15)":"rgba(248, 113, 113, 0.15)",d=!!n.error,f=n.result!==void 0&&n.result!==null,x=n.inputRequests&&Object.keys(n.inputRequests).length>0;if(r)r.innerHTML=`
          <!-- Status Banner & Top Metrics -->
          <div style="display: flex; justify-content: space-between; align-items: center; background: var(--surface-card); border: 1px solid var(--border); border-radius: var(--radius-md); padding: 12px 16px; margin-bottom: 14px;">
            <div>
              <div style="display: flex; align-items: center; gap: 8px;">
                <span class="brand-badge" style="background: ${v}; color: ${m}; border-color: ${m}; font-size: 11px;">
                  ${i(n.status.toUpperCase())}
                </span>
                <span style="font-family: var(--ff-mono); font-size: 13px; font-weight: 700; color: var(--text-main);">
                  ${i(n.capabilityId||"Tool Execution")}
                </span>
                ${n.serverId?`<span style="font-size: 11px; color: var(--text-dim);">via <code style="color: var(--cyan-400);">${i(n.serverId)}</code></span>`:""}
              </div>
              <div style="font-family: var(--ff-mono); font-size: 11px; color: var(--text-dim); margin-top: 4px;">
                Task ID: <span style="color: var(--text-muted);">${i(n.taskId)}</span>
              </div>
            </div>
            <div style="text-align: right; font-family: var(--ff-mono); font-size: 11px;">
              <div style="color: var(--text-dim);">Created: <span style="color: var(--text-muted);">${n.createdAtEpochSecs?new Date(n.createdAtEpochSecs*1000).toLocaleString():n.createdAt?new Date(n.createdAt).toLocaleString():"—"}</span></div>
              ${!u?`<div style="color: var(--amber-400); margin-top: 2px;">TTL: ${g}s remaining</div>`:""}
            </div>
          </div>

          <!-- Progress Bar -->
          <div style="margin-bottom: 16px;">
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 4px; font-family: var(--ff-mono); font-size: 11px;">
              <span style="color: var(--text-muted); text-transform: uppercase; font-weight: 600;">Execution Progress</span>
              <span style="color: ${m}; font-weight: 700;">${l}% ${n.total?`(step ${Math.round((n.progress||0)*n.total)} of ${n.total})`:""}</span>
            </div>
            <div style="height: 8px; background: var(--surface-card); border-radius: 4px; overflow: hidden; border: 1px solid var(--border);">
              <div style="height: 100%; width: ${l}%; background: ${m}; transition: width 0.3s;"></div>
            </div>
          </div>

          <!-- Caller Context Envelope -->
          ${n.context?`
            <div style="background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 8px 12px; font-family: var(--ff-mono); font-size: 11px; display: flex; flex-wrap: wrap; gap: 16px; margin-bottom: 14px; color: var(--text-muted);">
              ${n.context.actor_id?`<div><span style="color: var(--text-dim);">Actor:</span> <span style="color: var(--cyan-400);">${i(n.context.actor_id)}</span></div>`:""}
              ${n.context.operation_id?`<div><span style="color: var(--text-dim);">Operation:</span> <span style="color: var(--text-main);">${i(n.context.operation_id)}</span></div>`:""}
              ${n.context.grant_id?`<div><span style="color: var(--text-dim);">Grant:</span> <span style="color: var(--text-main);">${i(n.context.grant_id)}</span></div>`:""}
            </div>
          `:""}

          <!-- Diagnostic / Error Message -->
          ${d?`
            <div style="background: rgba(248, 113, 113, 0.1); border: 1px solid rgba(248, 113, 113, 0.3); border-radius: var(--radius-sm); padding: 12px; margin-bottom: 14px;">
              <div style="font-size: 11px; font-weight: 700; color: var(--red-400); text-transform: uppercase; margin-bottom: 6px;">
                ⚠️ Failure / Cancellation Trace
              </div>
              <pre style="font-family: var(--ff-mono); font-size: 11.5px; color: var(--red-300); white-space: pre-wrap; word-break: break-word; margin: 0;">${i(typeof n.error==="string"?n.error:JSON.stringify(n.error,null,2))}</pre>
            </div>
          `:""}

          <!-- Input Requests (if awaiting input) -->
          ${x?`
            <div style="margin-bottom: 14px;">
              <div style="font-size: 11px; font-weight: 700; color: var(--amber-400); text-transform: uppercase; margin-bottom: 6px;">
                ⚡ Pending Input Requests (MRTR / HITL)
              </div>
              <pre style="background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 12px; font-family: var(--ff-mono); font-size: 11.5px; color: var(--amber-300); white-space: pre-wrap; word-break: break-word; max-height: 180px; overflow-y: auto; margin: 0;">${i(JSON.stringify(n.inputRequests,null,2))}</pre>
            </div>
          `:""}

          <!-- Result Payload -->
          <div style="margin-bottom: 8px;">
            <div style="font-size: 11px; font-weight: 700; color: var(--text-muted); text-transform: uppercase; margin-bottom: 6px;">
              ${f?"\uD83D\uDCE6 Output Result Payload":"State Details"}
            </div>
            <pre style="background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 12px; font-family: var(--ff-mono); font-size: 11.5px; color: var(--text-main); white-space: pre-wrap; word-break: break-word; max-height: 220px; overflow-y: auto; margin: 0;">${i(f?JSON.stringify(n.result,null,2):n.status==="working"?"Task execution is currently in-flight in background worker pool.":"No output payload recorded.")}</pre>
          </div>
        `;if(a)a.innerHTML=`
          <div>
            ${!u?`
              <button class="btn btn-danger" style="font-size: 11.5px;" onclick="window.app.promptCancelTask('${i(n.taskId)}'); window.app.closeModals();">
                ⛔ Cancel Task
              </button>
            `:`
              <span style="font-family: var(--ff-mono); font-size: 11px; color: var(--text-dim);">Task is terminal (${i(n.status)})</span>
            `}
          </div>
          <div style="display: flex; gap: 8px;">
            <button class="btn btn-ghost" style="font-size: 11.5px;" onclick="navigator.clipboard.writeText('${i(n.taskId)}'); alert('Task ID copied to clipboard');">
              \uD83D\uDCCB Copy Task ID
            </button>
            <button class="btn btn-ghost" style="font-size: 11.5px;" onclick="window.app.closeModals()">
              Close
            </button>
          </div>
        `}catch(o){if(r)r.innerHTML=`
          <div style="color: var(--red-400); font-family: var(--ff-mono); font-size: 11.5px;">
            Failed to inspect task: ${i(o.message)}
          </div>
        `}}async submitApproval(e){let t=document.getElementById(`appr-args-${e}`),s=void 0;if(t&&t.value.trim())try{s=JSON.parse(t.value.trim())}catch{alert("Invalid JSON in arguments editor");return}let r=await y.approveTicket(e,s);if(r.ok)await this.refreshApprovals(),await this.refreshTasks();else alert(`Approval failed: ${r.error?.message||"Unknown error"}`)}async promptReject(e){let t=prompt("Reason for rejection (will be returned to the calling agent):");if(t===null)return;let s=await y.rejectTicket(e,t);if(s.ok)await this.refreshApprovals(),await this.refreshTasks();else alert(`Rejection failed: ${s.error?.message||"Unknown error"}`)}setPlaygroundMode(e){p.setState({playgroundMode:e})}selectCapability(e){p.setState({selectedCapabilityId:e});let t=p.getState().capabilities.find((r)=>r.id===e),s=document.getElementById("pg-args-input");if(t){let r=Y(t.input_schema,!1),a=JSON.stringify(r,null,2);if(s)s.value=a;let o={...p.getState().playgroundArgs||{}};o[e]=a,p.getState().playgroundArgs=o}}selectResource(e){p.setState({selectedResourceId:e})}selectPrompt(e){p.setState({selectedPromptId:e})}filterCatalog(e,t,s,r,a){let o=e.toLowerCase().trim(),n=t.filter((c)=>[c.id,c.name,c[s],c.server].some((g)=>g&&g.toLowerCase().includes(o))),l=document.getElementById(r);if(l)l.innerHTML=a(n)}filterResources(e){let t=p.getState();this.filterCatalog(e,t.resources||[],"uri","pg-res-list",(s)=>te(s,t.selectedResourceId,`No resources match "${e}"`))}filterPrompts(e){let t=p.getState();this.filterCatalog(e,t.prompts||[],"description","pg-prompt-list",(s)=>se(s,t.selectedPromptId,`No prompts match "${e}"`))}updatePlaygroundArgs(e){let t=p.getState(),s=t.selectedCapabilityId||(t.capabilities[0]?t.capabilities[0].id:null);if(!s)return;let r={...t.playgroundArgs||{}};r[s]=e,t.playgroundArgs=r}fillPlaygroundSampleArgs(e=!1){let t=p.getState(),s=t.selectedCapabilityId||(t.capabilities[0]?t.capabilities[0].id:null),r=t.capabilities.find((l)=>l.id===s),a=document.getElementById("pg-args-input");if(!a)return;if(!r||!r.input_schema){if(a.value="{}",s){let l={...t.playgroundArgs||{}};l[s]="{}",t.playgroundArgs=l}return}let o=Y(r.input_schema,e),n=JSON.stringify(o,null,2);if(a.value=n,s){let l={...t.playgroundArgs||{}};l[s]=n,t.playgroundArgs=l}}formatPlaygroundArgs(){let e=p.getState(),t=e.selectedCapabilityId||(e.capabilities[0]?e.capabilities[0].id:null),s=document.getElementById("pg-args-input");if(s)try{let r=JSON.parse(s.value||"{}"),a=JSON.stringify(r,null,2);if(s.value=a,t){let o={...e.playgroundArgs||{}};o[t]=a,e.playgroundArgs=o}}catch(r){alert(`Cannot format JSON: ${r.message}`)}}insertPlaygroundArgKey(e,t,s){let r=p.getState(),a=r.selectedCapabilityId||(r.capabilities[0]?r.capabilities[0].id:null),o=document.getElementById("pg-args-input");if(o){let n={};try{n=JSON.parse(o.value||"{}")}catch{n={}}if(n[e]===void 0)if(s!==null&&s!==void 0)n[e]=s;else switch(t){case"string":n[e]=`sample_${e}`;break;case"number":case"integer":n[e]=0;break;case"boolean":n[e]=!0;break;case"array":n[e]=[];break;case"object":n[e]={};break;default:n[e]=`sample_${e}`}let l=JSON.stringify(n,null,2);if(o.value=l,a){let c={...r.playgroundArgs||{}};c[a]=l,r.playgroundArgs=c}}}fillBatchStepSampleArgs(e){let t=p.getState(),s=[...t.batchSteps||[]],r=s[e];if(!r||!r.capability_id)return;let a=t.capabilities.find((c)=>c.id===r.capability_id);if(!a||!a.input_schema)return;let o=a.input_schema.properties||{},n={};for(let[c,g]of Object.entries(o))if(g.default!==void 0)n[c]=g.default;else if(Array.isArray(g.enum)&&g.enum.length>0)n[c]=g.enum[0];else switch(g.type||"string"){case"string":n[c]=`sample_${c}`;break;case"number":case"integer":n[c]=0;break;case"boolean":n[c]=!0;break;case"array":n[c]=[];break;case"object":n[c]={};break;default:n[c]=`sample_${c}`}let l=JSON.stringify(n,null,2);s[e]={...s[e],argsJson:l},p.setState({batchSteps:s})}filterCapabilities(e){let t=e.toLowerCase().trim(),r=p.getState().capabilities.filter((o)=>o.id.toLowerCase().includes(t)||o.summary&&o.summary.toLowerCase().includes(t)||o.server&&o.server.toLowerCase().includes(t)),a=document.getElementById("pg-cap-list");if(a)if(r.length===0)a.innerHTML=`
          <div style="padding: 24px 16px; text-align: center; color: var(--text-dim); font-size: 11.5px;">
            No capabilities match "${i(e)}"
          </div>
        `;else a.innerHTML=r.map((o)=>`
          <div class="cap-item ${o.id===p.getState().selectedCapabilityId?"active":""}" onclick="window.app.selectCapability('${i(o.id)}')">
            <div style="display: flex; justify-content: space-between; align-items: center;">
              <span style="font-weight: 600; color: var(--text-main); font-family: var(--ff-mono); font-size: 12px;">${i(o.id)}</span>
              <span style="font-size: 10px; color: var(--green-400);">${i(o.mode||"read")}</span>
            </div>
            <div style="font-size: 11px; color: var(--text-dim); margin-top: 2px;">server: ${i(o.server||"local")}</div>
          </div>
        `).join("")}async executePlaygroundTool(){let e=p.getState(),t=e.selectedCapabilityId||(e.capabilities[0]?e.capabilities[0].id:null);if(!t)return;let s=document.getElementById("pg-args-input")?.value||"{}",r=document.getElementById("pg-context-input")?.value||void 0,a=document.getElementById("pg-jsonpath-input")?.value.trim()||void 0,o=document.getElementById("pg-limit-lines-input")?.value.trim()||void 0,n=document.getElementById("pg-truncate-bytes-input")?.value.trim()||void 0,l={};try{l=JSON.parse(s)}catch{alert("Invalid arguments JSON object");return}if(a)l._jsonpath=a;if(o&&!isNaN(Number(o)))l._limit_lines=Number(o);if(n&&!isNaN(Number(n)))l._truncate_bytes=Number(n);let c=`op-${Date.now()}`;p.setState({isExecuting:!0,activeRequestId:c});let g=e.activeProfile||void 0,u=e.playgroundAsyncTask||!1;try{let m=await y.callCapability({capability_id:t,args:l,request_id:c,async_task:u?!0:void 0,context:{operation_id:r||c}},g);if(p.setState({isExecuting:!1,activeRequestId:null,executionResult:{status:m.status,durationMs:m.durationMs,data:m.data}}),m.status===202||m.data?.resultType==="task")this.refreshTasks();p.addEventLog("POST",`/v1/tools/call → ${t}`,m.status===200?"200 OK":`HTTP ${m.status}`,`${m.durationMs.toFixed(1)}ms`),y.getConfig().then((v)=>{if(v.ok&&v.circuit_breakers)p.setState({circuitBreakers:v.circuit_breakers})})}catch(m){p.setState({isExecuting:!1,activeRequestId:null,executionResult:{status:500,durationMs:0,data:{error:m.toString()}}})}}async cancelActiveOperation(){let t=p.getState().activeRequestId;if(t)try{await y.cancelOperation(t)}catch(s){console.warn("Failed to send cancel signal:",s)}p.setState({isExecuting:!1,activeRequestId:null,executionResult:{status:499,durationMs:0,data:{ok:!1,error:{code:"CANCELLED",message:"Operation cancelled by operator"}}}})}openBatchModal(){let e=p.getState(),t=e.batchSteps;if(!t||t.length===0)t=[{id:"step_1",capability_id:e.selectedCapabilityId||(e.capabilities[0]?e.capabilities[0].id:""),argsJson:"{}",continue_on_error:!1},{id:"step_2",capability_id:"",argsJson:"{}",continue_on_error:!0}],p.setState({batchSteps:t});p.setState({isBatchModalOpen:!0})}closeBatchModal(){p.setState({isBatchModalOpen:!1})}addBatchStep(){let t=[...p.getState().batchSteps||[]],s=t.length+1;t.push({id:`step_${s}`,capability_id:"",argsJson:"{}",continue_on_error:!1}),p.setState({batchSteps:t})}removeBatchStep(e){let s=[...p.getState().batchSteps||[]];if(s.length<=1){alert("Pipeline must contain at least one execution step.");return}s.splice(e,1);let r=s.map((a,o)=>({...a,id:`step_${o+1}`}));p.setState({batchSteps:r})}updateBatchStepCapability(e,t){this.updateBatchStep(e,{capability_id:t})}updateBatchStepContinueOnError(e,t){this.updateBatchStep(e,{continue_on_error:t})}updateBatchStep(e,t){let s=[...p.getState().batchSteps];if(s[e])s[e]={...s[e],...t},p.setState({batchSteps:s})}updateBatchStepArgs(e,t){let s=p.getState(),r=[...s.batchSteps||[]];if(r[e])r[e]={...r[e],argsJson:t},s.batchSteps[e].argsJson=t}appendBatchVariable(e,t){let r=[...p.getState().batchSteps||[]],a=document.getElementById(`batch-step-args-${e}`);if(a){let o=a.value,n=a.selectionStart||o.length,l=a.selectionEnd||o.length,c=o.substring(0,n)+t+o.substring(l);if(a.value=c,r[e])r[e]={...r[e],argsJson:c},p.setState({batchSteps:r})}}async executeBatchPipeline(){let e=p.getState(),t=e.batchSteps||[],s=[];for(let a=0;a<t.length;a++){let o=t[a];if(!o.capability_id){alert(`Please select a capability for Step ${a+1}`);return}let n={};try{n=JSON.parse(o.argsJson||"{}")}catch{alert(`Invalid JSON in Step ${a+1} arguments`);return}s.push({id:o.id||`step_${a+1}`,capability_id:o.capability_id,args:n,continue_on_error:o.continue_on_error})}p.setState({isBatchModalOpen:!1});let r=e.activeProfile||void 0;try{let a=await y.batchCallCapabilities(s,r);p.setState({executionResult:{status:a.status,durationMs:a.durationMs,data:a.data}}),p.addEventLog("POST",`/v1/tools/batch_call (${t.length} steps)`,a.status===200?"200 OK":`HTTP ${a.status}`,`${a.durationMs.toFixed(1)}ms`)}catch(a){p.setState({executionResult:{status:500,durationMs:0,data:{error:a.toString()}}})}}async executeCatalogRequest(e,t,s){try{let r=await s();p.setState({[e]:r}),p.addEventLog("POST",t,r.status===200?"200 OK":`HTTP ${r.status}`,`${r.durationMs.toFixed(1)}ms`)}catch(r){p.setState({[e]:{status:500,durationMs:0,data:{error:r.toString()}}})}}async executeReadResource(){let e=p.getState(),t=e.selectedResourceId||(e.resources[0]?e.resources[0].id:null);if(!t)return;let s=document.getElementById("pg-res-jsonpath-input")?.value.trim()||void 0,r=document.getElementById("pg-res-lines-input")?.value.trim()||void 0,a=document.getElementById("pg-res-bytes-input")?.value.trim()||void 0,o={resource_id:t};if(s)o._jsonpath=s;if(r&&!isNaN(Number(r)))o._limit_lines=Number(r);if(a&&!isNaN(Number(a)))o._truncate_bytes=Number(a);let n=e.activeProfile||void 0;await this.executeCatalogRequest("resourceReadResult",`/v1/resources/read → ${t}`,()=>y.readResource({resource_id:t,input_responses:o},n))}async executeGetPrompt(){let e=p.getState(),t=e.selectedPromptId||(e.prompts[0]?e.prompts[0].id:null);if(!t)return;let s=document.querySelectorAll(".prompt-arg-input"),r={};s.forEach((o)=>{let n=o,l=n.getAttribute("data-arg-name");if(l&&n.value.trim())r[l]=n.value.trim()});let a=e.activeProfile||void 0;await this.executeCatalogRequest("promptGetResult",`/v1/prompts/get → ${t}`,()=>y.getPrompt({prompt_id:t,arguments:r},a))}toggleBatchPlayground(){let e=document.getElementById("pg-args-input");if(!e)return;let t=[{id:"step_1",capability_id:"sqlite.read_query",args:{query:"SELECT * FROM users LIMIT 2"}},{id:"step_2",capability_id:"github.issues.search",args:{query:"label:bug"},continue_on_error:!0}];e.value=JSON.stringify(t,null,2)}async submitPolicyRule(e){let t=e==="allow"?"policy-new-allow":e==="deny"?"policy-new-deny":e==="redact"?"policy-new-redact":"policy-new-requireApproval",s=document.getElementById(t);if(!s)return;let r=s.value.trim();if(!r)return;await this.addPolicyRule(e,r),s.value=""}async addPolicyRule(e,t){let s=(t||"").trim();if(s)await this.mutatePolicyRules(e,(r)=>r.includes(s)?r:[...r,s],!0)}async removePolicyRule(e,t){await this.mutatePolicyRules(e,(s)=>(s.splice(t,1),s),!1)}async mutatePolicyRules(e,t,s){let r=p.getState(),a=r.activeProfile,o=a?r.config.profiles?.[a]:void 0,n=(o?o.policy:r.config.policy)||{},l={allow:[...n.allow||[]],deny:[...n.deny||[]],redact:[...n.redact_keys||n.redactKeys||[]],requireApproval:[...n.require_approval||n.requireApproval||[]]};l[e]=t(l[e]);let c={...n,allow:l.allow,deny:l.deny,redactKeys:l.redact,requireApproval:l.requireApproval},g=o&&a?await y.upsertProfile(a,o.servers,o.description,c):await y.savePolicy({...c,redact_keys:l.redact,require_approval:l.requireApproval});if(!g.ok)alert(`Failed to ${s?o?"save profile policy rule":"save policy rule":o?"update profile policy":"update policy"}: ${g.error||"Unknown error"}`);await this.refreshData()}renderPolicyView(e){let t=e.querySelector("#tool-pins-panel"),s=[...e.querySelectorAll("form[data-policy-key]")],r=document.activeElement;e.innerHTML=ke();for(let o of s){let n=document.getElementById(o.id);if(n&&n.dataset.policyKey===o.dataset.policyKey){if(n.replaceWith(o),r instanceof HTMLElement&&o.contains(r))r.focus({preventScroll:!0})}}let a=e.querySelector("#tool-pins-panel");if(a&&t)a.replaceWith(t);else if(a)Ee(a)}async saveResultSanitization(e){await this.saveTrustPolicy(e,(t,s)=>({...t,resultSanitization:ue(s)}))}async saveWebhookConfig(e){await this.saveTrustPolicy(e,(t,s,r)=>({...t,webhook:ge(s,t.webhook,r)}))}async saveTrustPolicy(e,t){if(e.preventDefault(),this.savingTrustPolicy)return;let s=e.currentTarget,r=s.querySelector("[role=status]"),a=s.querySelector("button[type=submit]"),o=new FormData(s),n=this.configIdentity;this.savingTrustPolicy=!0,a.disabled=!0,r.textContent="Saving…";try{let l=await y.getConfig();if(n!==this.configIdentity)return;if(!l.ok)throw Error(z(l,"Configuration could not be loaded."));let c=t(l.config.policy??{},o,l.config.rbac?.enabled===!0),g=await y.savePolicy(c);if(!g.ok)throw Error(z(g,"Settings could not be saved."));s.remove(),await this.refreshData();let u=document.getElementById(s.id)?.querySelector("[role=status]");if(u)u.textContent="Settings saved."}catch(l){r.textContent=l instanceof Error?l.message:"Settings could not be saved."}finally{this.savingTrustPolicy=!1,a.disabled=!1}}async testWebhook(){let e=document.querySelector("#chatops-settings-form [role=status]");if(!e)return;e.textContent="Sending a test event to the saved destination…";let t=await y.testWebhook().catch(()=>({ok:!1,error:"Could not reach Warmplane."}));e.textContent=t.ok?"Test event sent.":z(t,"Test event could not be sent.")}testPolicySandbox(e){let t=document.getElementById("policy-test-verdict");if(!t)return;let s=e.trim();if(!s){t.textContent="ENTER ID",t.style.color="var(--text-dim)";return}let r=p.getState(),a=r.activeProfile,o=a?r.config.profiles?.[a]:void 0,n=r.config.policy||{},l=o?.policy,c=n.deny||[],g=l?.deny||[],u=Array.from(new Set([...c,...g])),m=n.require_approval||n.requireApproval||[],v=l?.require_approval||l?.requireApproval||[],d=Array.from(new Set([...m,...v])),f=l&&l.allow&&l.allow.length>0?l.allow:n.allow||[],x=(b,h)=>{if(b==="*")return!0;if(b.endsWith("*"))return h.startsWith(b.slice(0,-1));return b===h};if(u.some((b)=>x(b,s))){t.textContent="DENIED (Strict Block)",t.style.color="var(--red-400)";return}if(f.length>0&&!f.some((b)=>x(b,s))){t.textContent="DENIED (Not in Allow List)",t.style.color="var(--red-400)";return}if(d.some((b)=>x(b,s))){t.textContent="REQUIRE APPROVAL (HITL Gate)",t.style.color="var(--amber-400)";return}t.textContent="ALLOWED",t.style.color="var(--green-400)"}async deleteServer(e){let t=p.getState().activeProfile,s=t?`Permanently delete server '${e}' globally from warmplane configuration? (This will also unbind it from profile '${t}')`:`Are you sure you want to permanently remove server '${e}' from configuration?`;if(!confirm(s))return;try{let r=await y.deleteServer(e);if(r.ok)await this.refreshData();else alert(`Failed to remove server '${e}': ${r.error||"Unknown error"}`)}catch(r){alert(`Error removing server '${e}': ${r.message}`)}}async restartServer(e){try{let t=await y.restartServer(e);if(t.ok)await this.refreshData();else alert(`Failed to restart server '${e}': ${t.error||"Unknown error"}`)}catch(t){alert(`Error restarting server '${e}': ${t.message}`)}}openServerDiagnosticsModal(e){this.closeModals();let t=p.getState(),s=t.config.mcpServers?.[e],r=t.serverStatuses?.[e],a=(t.circuitBreakers||[]).find((c)=>c.server_id===e),o=document.getElementById("modal-server-diagnostics");if(!o)return;let n=document.getElementById("modal-diag-title"),l=document.getElementById("modal-diag-body");if(n)n.textContent=`Live Diagnostics: ${e}`;if(l){let c=r?.status==="degraded",g=c?"var(--amber-400)":r?.status==="connected"?"var(--green-400)":"var(--red-400)",u=r?.error||"No active crash or error reported. Server is healthy.",m=J(e,s?.command,s?.args),v=s?.env||{},d=Object.keys(v),x=(m?.envFields||[]).filter((h)=>h.required).filter((h)=>!d.includes(h.key)),b="";if(m||d.length>0){let h=(m?.envFields||[]).map((E)=>{let k=v[E.key]!==void 0,A='<span class="brand-badge" style="color: var(--green-400); border-color: rgba(52, 211, 153, 0.3);">Configured</span>';if(!k&&E.required)A='<span class="brand-badge" style="color: var(--red-400); border-color: rgba(248, 113, 113, 0.4); background: rgba(248, 113, 113, 0.1);">Required / Missing</span>';else if(!k)A='<span class="brand-badge" style="color: var(--text-dim); border-color: var(--border);">Optional / Not Set</span>';return`
            <div style="display: flex; justify-content: space-between; align-items: center; padding: 6px 10px; background: var(--surface); border-radius: var(--radius-xs); margin-bottom: 6px; font-size: 11.5px;">
              <div>
                <span style="font-family: var(--ff-mono); font-weight: 700; color: ${!k&&E.required?"var(--amber-300)":"var(--text-main)"};">${i(E.key)}</span>
                ${E.label?`<span style="font-size: 10.5px; color: var(--text-dim); margin-left: 6px;">(${i(E.label)})</span>`:""}
              </div>
              <div style="display: flex; align-items: center; gap: 8px;">
                ${A}
                ${!k?`
                  <button class="btn btn-primary" style="padding: 2px 8px; font-size: 10.5px;" onclick="window.app.quickVaultEnv('${i(e)}', '${i(E.key)}')">➕ Configure in Keychain</button>
                `:""}
              </div>
            </div>
          `}).join(""),T=(m?.envFields||[]).map((E)=>E.key),C=Object.entries(v).filter(([E])=>!T.includes(E)).map(([E,w])=>`
            <div style="display: flex; justify-content: space-between; align-items: center; padding: 6px 10px; background: var(--surface); border-radius: var(--radius-xs); margin-bottom: 6px; font-size: 11.5px;">
              <span style="font-family: var(--ff-mono); font-weight: 700; color: var(--text-main);">${i(E)}</span>
              <span class="brand-badge" style="color: var(--green-400); border-color: rgba(52, 211, 153, 0.3);">Custom Configured</span>
            </div>
          `).join("");b=`
          <div style="background: rgba(0,0,0,0.2); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 12px; margin-bottom: 14px;">
            <div style="font-size: 11px; font-weight: 700; color: ${x.length>0?"var(--amber-400)":"var(--text-main)"}; text-transform: uppercase; margin-bottom: 8px; display: flex; justify-content: space-between; align-items: center;">
              <span>\uD83D\uDD11 Environment Variables &amp; Secrets</span>
              ${x.length>0?`<span style="color: var(--red-400); font-size: 10.5px;">⚠️ ${x.length} required key(s) missing</span>`:""}
            </div>
            ${h}
            ${C}
          </div>
        `}l.innerHTML=`
        <div style="display: flex; gap: 10px; align-items: center; margin-bottom: 16px;">
          <span style="width: 10px; height: 10px; border-radius: 50%; background: ${g};"></span>
          <span style="font-weight: 700; font-size: 14px; color: var(--text-main);">Current Status: <span style="color: ${g}; text-transform: uppercase;">${i(r?.status||"unknown")}</span></span>
          <span class="brand-badge" style="color: var(--cyan-400);">Protocol: ${i(r?.protocol_version||"2026-07-28")}</span>
          ${x.length>0?'<span class="brand-badge" style="color: var(--red-400); border-color: rgba(248, 113, 113, 0.4); background: rgba(248, 113, 113, 0.1);">⚠️ Missing Required Keys</span>':""}
        </div>

        <div style="background: rgba(0,0,0,0.3); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 12px; margin-bottom: 14px;">
          <div style="font-size: 11px; font-weight: 700; color: var(--amber-400); text-transform: uppercase; margin-bottom: 6px;">
            ⚠️ Diagnostic Details / Failure Root Cause
          </div>
          <pre style="font-family: var(--ff-mono); font-size: 11.5px; color: ${c?"var(--red-300)":"var(--text-dim)"}; white-space: pre-wrap; word-break: break-word; margin: 0;">${i(u)}</pre>
        </div>

        ${b}

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
              Auto-Restart: ${s?.resilience?.autoRestart!==!1?"ENABLED":"DISABLED"}
            </div>
          </div>
        </div>

        <div style="background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-sm); padding: 10px; margin-bottom: 16px;">
          <div style="font-size: 10.5px; color: var(--text-dim); margin-bottom: 4px;">Configured Execution Target</div>
          <code style="font-family: var(--ff-mono); font-size: 11px; color: var(--cyan-400); display: block; word-break: break-all;">
            ${s?.command?`${i(s.command)} ${i((s.args||[]).join(" "))}`:i(s?.url||"")}
          </code>
        </div>

        <div style="display: flex; justify-content: flex-end; gap: 8px;">
          <button class="btn btn-primary" onclick="window.app.restartServer('${i(e)}'); window.app.closeModals();">⚡ Restart &amp; Probe Now</button>
          <button class="btn btn-ghost" onclick="window.app.closeModals()">Close</button>
        </div>
      `}o.classList.add("active")}openAddServerModal(){this.closeModals(),this.populateServerModal()}openEditServerModal(e){this.closeModals();let s=p.getState().config.mcpServers?.[e];if(!s){alert(`Server '${e}' not found in configuration.`);return}this.populateServerModal(e,s)}populateServerModal(e,t){let s=e!==void 0,r=document.querySelector("#modal-srv-require-tool-pins");if(r)r.checked=!!t?.requireToolPins;let a=document.getElementById("modal-srv-title"),o=document.getElementById("modal-srv-template-banner");if(a)a.textContent=s?`Edit Server '${e}'`:"Add Upstream MCP Server";if(o)o.style.display=s?"none":"flex";let n=document.getElementById("modal-srv-name");if(n)n.value=e||"",n.disabled=s;let l=!s||!!t?.command,c=s?t.resilience||p.getState().config.resilience:void 0,g={transport:l?"stdio":"http",command:s&&l?`${t.command} ${(t.args||[]).join(" ")}`.trim():"",url:t?.url||"",ft:String(c?.failureThreshold??3),cd:String(c?.cooldownMs??30000),autorestart:c?.autoRestart===!1?"false":"true",maxrestarts:String(c?.maxRestarts??5)};for(let[v,d]of Object.entries(g)){let f=document.getElementById(`modal-srv-${v}`);if(f)f.value=d}let u=document.getElementById("modal-group-cmd"),m=document.getElementById("modal-group-url");if(u)u.style.display=l?"block":"none";if(m)m.style.display=l?"none":"block";document.getElementById("modal-add-server")?.classList.add("active")}async submitAddServer(){let e=document.getElementById("modal-srv-name"),t=e?.value.trim(),s=document.getElementById("modal-srv-transport")?.value;if(!e||!t){alert("Server name is required");return}let r=p.getState();if(!r.configReady){alert("Load the current configuration through Trust access before saving a server.");return}let a=e.disabled;if(!a&&r.config.mcpServers?.[t]){if(confirm(`Server '${t}' already exists. Open its settings?`))this.openEditServerModal(t);return}let o={requireToolPins:document.querySelector("#modal-srv-require-tool-pins")?.checked??!1};if(s==="stdio"){let l=document.getElementById("modal-srv-command")?.value.trim(),c=a?r.config.mcpServers?.[t]:void 0,g=c?.command?[c.command,...c.args??[]].join(" ").trim():"",u=c?.command&&l===g?[c.command,...c.args??[]]:l.split(/\s+/).filter(Boolean);if(u.length===0){alert("Command is required");return}o.command=u[0],o.args=u.slice(1)}else{let l=document.getElementById("modal-srv-url")?.value.trim();if(!l){alert("URL is required");return}o.url=l}o.resilience=this.readServerResilience("modal");let n=await y.upsertServer(t,o,a);if(n.ok)this.closeModals(),await this.refreshData();else alert(z(n,"Server settings could not be saved."))}openTemplateCatalog(){this.closeModals();let e=document.getElementById("modal-templates");if(e)e.classList.add("active");this.renderTemplateGrid()}setTemplateCategory(e){this.activeTemplateCategory=e,document.querySelectorAll(".tmpl-cat-btn").forEach((t)=>{if(t.getAttribute("data-category")===e)t.classList.add("active"),t.style.background="var(--surface-elevated)",t.style.color="var(--amber-400)";else t.classList.remove("active"),t.style.background="var(--surface-card)",t.style.color="var(--text-main)"}),this.renderTemplateGrid()}filterTemplates(e){this.activeTemplateFilter=e.toLowerCase().trim(),this.renderTemplateGrid()}renderTemplateGrid(){let e=document.getElementById("tmpl-grid");if(!e)return;let t=W.filter((a)=>{let o=this.activeTemplateCategory==="all"||a.category===this.activeTemplateCategory,n=!this.activeTemplateFilter||a.name.toLowerCase().includes(this.activeTemplateFilter)||a.id.toLowerCase().includes(this.activeTemplateFilter)||a.description.toLowerCase().includes(this.activeTemplateFilter)||a.command.toLowerCase().includes(this.activeTemplateFilter)||a.envFields.some((l)=>l.key.toLowerCase().includes(this.activeTemplateFilter));return o&&n});if(t.length===0){e.innerHTML=`
        <div style="grid-column: span 2; padding: 32px; text-align: center; color: var(--text-dim);">
          No matching MCP server templates found.
        </div>
      `;return}let r=p.getState().config.mcpServers||{};e.innerHTML=t.map((a)=>{let o=!!r[a.id],n=`${a.command} ${a.defaultArgs.join(" ")}`;return`
        <div class="bento-card" style="display: flex; flex-direction: column; justify-content: space-between; padding: 14px; background: var(--surface); border: 1px solid var(--border); min-width: 0; transition: transform 0.15s, border-color 0.15s;">
          <div>
            <div style="display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 6px;">
              <div style="display: flex; align-items: center; gap: 8px; min-width: 0;">
                <span style="font-weight: 700; font-size: 13.5px; color: var(--text-main); white-space: nowrap; overflow: hidden; text-overflow: ellipsis;">${i(a.name)}</span>
                <span class="brand-badge" style="font-size: 9.5px; padding: 1px 6px; flex-shrink: 0;">${i(a.badge)}</span>
              </div>
              ${o?'<span style="font-size: 10px; color: var(--green-400); font-weight: 600; flex-shrink: 0;">CONNECTED</span>':""}
            </div>
            <div style="font-size: 11.5px; color: var(--text-muted); line-height: 1.4; margin-bottom: 8px;">
              ${i(a.description)}
            </div>
            <div style="font-family: var(--ff-mono); font-size: 10.5px; color: var(--text-dim); background: var(--surface-card); padding: 5px 8px; border-radius: var(--radius-sm); border: 1px solid var(--border-subtle); overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">
              <code>${i(n)}</code>
            </div>
            ${a.envFields.length>0?`
              <div style="font-size: 10.5px; color: var(--amber-400); margin-top: 6px; display: flex; align-items: center; gap: 4px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">
                <span>⚡ Needs:</span>
                <code style="overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">${a.envFields.map((l)=>i(l.key)).join(", ")}</code>
              </div>
            `:""}
          </div>

          <div style="display: flex; justify-content: flex-end; margin-top: 12px; gap: 6px;">
            <button class="btn btn-primary" style="font-size: 11.5px; padding: 4px 10px;" onclick="window.app.selectTemplate('${i(a.id)}')">
              ${o?"Configure Another":"✨ 1-Click Setup"}
            </button>
          </div>
        </div>
      `}).join("")}selectTemplate(e){let t=W.find((c)=>c.id===e);if(!t)return;this.selectedTemplate=t,this.closeModals();let s=document.getElementById("modal-configure-template");if(s)s.classList.add("active");let r=document.getElementById("cfg-tmpl-title"),a=document.getElementById("cfg-tmpl-desc"),o=document.getElementById("cfg-tmpl-form");if(r)r.textContent=`Configure ${t.name} Server`;if(a)a.textContent=t.description;let n=p.getState().config.mcpServers||{},l=t.id;if(n[l]){let c=2;while(n[`${t.id}-${c}`])c++;l=`${t.id}-${c}`}if(o){let c="";if(t.envFields.length>0)c=`
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
        `;o.innerHTML=`
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
      `}}async submitTemplateServer(){if(!this.selectedTemplate)return;let e=this.selectedTemplate,t=document.getElementById("cfg-srv-id")?.value.trim(),s=document.getElementById("cfg-srv-args")?.value.trim();if(!t){alert("Server identifier is required");return}let a=p.getState().config.mcpServers||{};if(a[t]){if(!confirm(`Server '${t}' already exists. Do you want to overwrite its configuration?`))return}let o=s?s.split(/\s+/).filter(Boolean):[],n={},l=document.querySelectorAll(".tmpl-env-input");for(let u of Array.from(l)){let m=u.getAttribute("data-key"),v=u.value.trim(),d=e.envFields.find((f)=>f.key===m);if(d?.required&&!v){alert(`Required field '${d.label}' is missing.`);return}if(m&&v)n[m]=v}let c={command:e.command,args:o,requireToolPins:a[t]?.requireToolPins};if(Object.keys(n).length>0)c.env=n;c.resilience=this.readServerResilience("cfg");let g=await y.upsertServer(t,c);if(g.ok)this.closeModals(),await this.refreshData();else alert(`Failed to save server: ${g.error}`)}readServerResilience(e){let t=(n)=>document.querySelector(`#${e}-srv-${n}`)?.value.trim(),s=t("ft"),r=t("cd"),a=t("autorestart"),o=t("maxrestarts");if(!s&&!r&&!a&&!o)return;return{failureThreshold:s?Number(s):3,cooldownMs:r?Number(r):30000,autoRestart:a!=="false",maxRestarts:o?Number(o):5}}async openImportModal(){this.closeModals();let e=document.getElementById("modal-import");if(e)e.classList.add("active");let t=document.getElementById("modal-eco-list");if(!t)return;t.innerHTML='<div style="color: var(--text-dim); padding: 12px; text-align: center;">Scanning IDE configs...</div>';try{let s=await y.getEcosystemSources();if(s.sources&&s.sources.length>0)t.innerHTML=s.sources.map((r)=>`
          <label style="display: flex; align-items: center; gap: 10px; background: var(--surface); padding: 10px 14px; border-radius: var(--radius-sm); border: 1px solid var(--border); cursor: pointer;">
            <input type="checkbox" class="eco-checkbox" value="${r.path}" checked>
            <div>
              <div style="font-weight: 600; color: var(--text-main);">${r.name}</div>
              <div style="font-size: 11px; color: var(--text-dim);">${r.server_count} servers (${r.servers.join(", ")})</div>
            </div>
          </label>
        `).join("");else t.innerHTML='<div style="color: var(--text-dim); padding: 12px; text-align: center;">No external MCP configuration files found on this system.</div>'}catch{t.innerHTML='<div style="color: var(--red-400); padding: 12px; text-align: center;">Failed to scan ecosystem sources.</div>'}}async submitImport(){let e=document.querySelectorAll(".eco-checkbox:checked");if(e.length===0){alert("No sources selected");return}for(let t of Array.from(e))await y.importConfig(t.value,!1);this.closeModals(),await this.refreshData()}async refreshClients(){await this.refreshCollection(()=>y.getClients(),(e)=>e.ok&&Array.isArray(e.clients)?{clients:e.clients}:null,"Failed to scan clients:")}setClientCategoryFilter(e){p.setState({clientFilterCategory:e})}setClientSearchQuery(e){p.setState({clientSearchQuery:e})}async attachClient(e,t){let s=t;if(!s){let r=document.getElementById(`client-prof-${e}`)||document.getElementById(`overview-client-prof-${e}`);if(r)s=r.value||void 0;else s=p.getState().activeProfile||void 0}this.openClientAttachModal(e,s)}openClientAttachModal(e,t){this.closeModals(),this.pendingClientId=e;let s=p.getState().config,r=s.mcpHttpServer,a=r?`http://${r.bind==="0.0.0.0"||r.bind==="::"?"127.0.0.1":r.bind||"127.0.0.1"}:${r.port||9191}/mcp`:"",o=document.getElementById("modal-client-title"),n=document.getElementById("modal-client-transport"),l=document.getElementById("modal-client-url"),c=document.getElementById("modal-client-profile"),g=document.getElementById("modal-client-profile-wrap");if(o)o.textContent=`Connect ${e}`;if(n)n.value="stdio";if(l)l.value=a;if(c){let u=Object.keys(s.profiles||{});if(c.innerHTML=`<option value="">All Tools (Default)</option>${u.map((m)=>`<option value="${i(m)}">Profile: ${i(m)}</option>`).join("")}`,c.value=t||"",g)g.style.display=u.length>0?"block":"none"}this.updateClientTransportForm(),document.getElementById("modal-client-attach")?.classList.add("active")}updateClientTransportForm(){let t=(document.getElementById("modal-client-transport")?.value||"stdio")==="http",s=document.getElementById("modal-client-http-group"),r=document.getElementById("modal-client-http-status"),a=!!p.getState().config.mcpHttpServer;if(s)s.style.display=t?"block":"none";if(r)r.textContent=a?"Uses the daemon Streamable HTTP endpoint. The daemon must already be running.":"HTTP is unavailable until mcpHttpServer is configured and the daemon is restarted.",r.style.color=a?"var(--text-dim)":"var(--amber-300)"}async submitClientAttach(){let e=this.pendingClientId;if(!e)return;let t=document.getElementById("modal-client-transport")?.value||"stdio",s=document.getElementById("modal-client-profile")?.value||void 0,r=document.getElementById("modal-client-url")?.value.trim()||void 0;if(t==="http"&&!p.getState().config.mcpHttpServer&&!r){alert("Configure mcpHttpServer and restart the daemon, or provide an explicit HTTP endpoint.");return}if(t==="http"&&!r){alert("Enter an HTTP endpoint URL.");return}let a=await y.attachClient(e,s,t,r);if(!a.ok)alert(`Failed to attach client: ${a.error||a.message||"Unknown error"}`);else this.pendingClientId=null,this.closeModals(),await this.refreshData()}async detachClient(e){if(!confirm("Disconnect Warmplane from this client?"))return;let t=await y.detachClient(e);if(!t.ok)alert(`Failed to detach client: ${t.error||t.message||"Unknown error"}`);else await this.refreshData()}aliasStudio;aliasListMarkup="";aliasSaving=!1;aliasTests=new Map;renderAliasStudio(e){if(!this.aliasStudio){let t=document.createElement("div");t.innerHTML=Re();let s=t.querySelector("#alias-form");if(!s)return;ze(s),s.addEventListener("submit",(r)=>{r.preventDefault(),this.createAlias()}),s.addEventListener("focusout",(r)=>this.validateAliasField(r)),s.addEventListener("input",(r)=>this.validateAliasField(r,!0)),s.addEventListener("change",(r)=>this.validateAliasField(r,!0)),t.addEventListener("click",(r)=>this.handleAliasRowAction(r)),this.aliasStudio=t}if(this.aliasStudio.parentElement!==e)e.replaceChildren(this.aliasStudio),this.aliasStudio.querySelectorAll('[role="combobox"]').forEach(H);this.refreshAliasList()}refreshAliasList(){let e=this.aliasStudio?.querySelector("#alias-list");if(!e)return;let t=Pe(this.aliasTests);if(t===this.aliasListMarkup&&e.hasChildNodes())return;this.aliasListMarkup=t;let s=document.activeElement,r=s instanceof HTMLButtonElement&&e.contains(s)?s.dataset:void 0;if(e.innerHTML=t,r)[...e.querySelectorAll("[data-alias-action]")].find((a)=>a.dataset.aliasAction===r.aliasAction&&a.dataset.aliasKind===r.aliasKind&&a.dataset.aliasName===r.aliasName)?.focus({preventScroll:!0})}handleAliasRowAction(e){if(!(e.target instanceof Element))return;let t=e.target.closest("[data-alias-action]");if(!t)return;let{aliasAction:s,aliasKind:r,aliasName:a}=t.dataset;if(a===void 0||r!=="tool"&&r!=="resource"&&r!=="prompt")return;if(s==="test")this.testSavedAlias(a);else if(!this.aliasSaving&&s==="edit")this.startEditAlias(r,a);else if(!this.aliasSaving&&s==="delete")this.deleteAlias(r,a)}async testSavedAlias(e){if(this.aliasTests.get(e)?.running)return;let t=X("tool",e),s=typeof t==="object"?t.reversibility?.testTarget?.server:void 0;if(!s||!confirm(`Run the saved contract for "${e}" on test server "${s}"?`))return;let r=JSON.stringify(t);this.aliasTests.set(e,{savedAlias:r,running:!0,message:`Running on ${s}…`}),this.refreshAliasList();try{let{ok:a,result:o,error:n}=await y.testUndoContract(e),l=a&&o?.observationally_equivalent,c=l?`Passed on ${o.test_server}. The final state matched the snapshot.`:`Test failed. ${n?.message||o?.error_code||"The final state did not match the snapshot."}`;this.aliasTests.set(e,{savedAlias:r,running:!1,message:c,failed:!l})}catch{this.aliasTests.set(e,{savedAlias:r,running:!1,message:"Could not run the test. Check the connection and try again.",failed:!0})}this.refreshAliasList()}startEditAlias(e,t){let s=X(e,t);if(s===void 0)return;let r=typeof s==="object"?s:void 0;this.resetAliasForm();let a=document.getElementById("alias-kind"),o=document.getElementById("alias-name"),n=document.getElementById("alias-target"),l=document.getElementById("alias-summary"),c=document.getElementById("alias-passthrough"),g=document.getElementById("alias-passthrough-container"),u=document.getElementById("alias-form-title"),m=document.getElementById("alias-save-btn");if(a)a.value=e;if(o)o.value=t;if(n)n.value=typeof s==="string"?s:s.target;if(l)l.value=r?.summary??"";if(c)c.checked=!!r?.passthrough;if(g)g.style.display=e==="tool"?"flex":"none";let v=document.getElementById("reversibility-editor");if(v)v.hidden=e!=="tool";if(this.loadReversibilityEditor(e==="tool"?r?.reversibility:void 0),this.showAliasFormError(""),u)u.textContent=`Edit alias ${t}`;if(m)m.textContent="Update";o?.focus(),o?.scrollIntoView({behavior:"smooth",block:"nearest"})}resetAliasForm(){if(this.aliasSaving)return;let e=document.getElementById("alias-kind"),t=document.getElementById("alias-name"),s=document.getElementById("alias-target"),r=document.getElementById("alias-summary"),a=document.getElementById("alias-passthrough"),o=document.getElementById("alias-passthrough-container"),n=document.getElementById("alias-form-title"),l=document.getElementById("alias-save-btn");if(e)e.value="tool";if(t)t.value="";if(s)s.value="";if(r)r.value="";if(a)a.checked=!1;if(o)o.style.display="flex";if(n)n.textContent="Create New Alias";if(l)l.textContent="Save";let c=document.getElementById("reversibility-editor");if(c)c.hidden=!1;this.loadReversibilityEditor(void 0),this.clearAliasFieldErrors(),document.querySelectorAll('#alias-form [role="combobox"]').forEach(H),this.showAliasFormError("")}updateAliasKind(e){let t=document.getElementById("alias-passthrough-container");if(t)t.style.display=e==="tool"?"flex":"none";let s=document.getElementById("reversibility-editor");if(s)s.hidden=e!=="tool";this.clearAliasFieldErrors(!0)}updateReversibilityEditor(){let e=document.getElementById("reversibility-class");if(!e)return;let t=e.dataset.previousClass||"",s=e.value;if(t!==s){if(t==="delayable"&&s!=="delayable")this.setInputValue("hold-window-secs","");if(t==="compensable"&&s!=="compensable")this.setInputValue("daily-limit","");if((t==="reversible"||t==="compensable")&&!(s==="reversible"||s==="compensable"))this.clearUndoContractFields();e.dataset.previousClass=s}document.querySelectorAll("[data-reversibility-class]").forEach((a)=>{let o=(a.dataset.reversibilityClass||"").split(" ");a.hidden=!o.includes(s)}),this.setOptionalSectionVisibility("pre-snapshot"),this.setOptionalSectionVisibility("pre-undo-check"),this.setOptionalSectionVisibility("test-target"),this.clearAliasFieldErrors(!0),this.refreshAliasDependencyError();let r=document.getElementById("reversibility-class-help");if(r){let a={"":"No reversibility contract is configured for this capability.",delayable:"Hold the forward call before dispatch so it can be cancelled.",reversible:"Capture prior state and register an inverse that restores it.",compensable:"Register a compensating action and limit its daily use.",irreversible:"Mark this capability as having no configured inverse."};r.textContent=a[s]||a[""]}}loadReversibilityEditor(e){let t=document.getElementById("reversibility-class");if(!t)return;t.value=e?.class||"",t.dataset.previousClass=t.value,this.setInputValue("hold-window-secs",e?.holdWindowSecs?.toString()||""),this.setInputValue("ttl-secs",e?.ttlSecs?.toString()||""),this.setInputValue("daily-limit",e?.dailyLimit?.toString()||""),this.setInputValue("inverse-capability",e?.inverse?.capability||""),this.setJsonInput("inverse-args",e?.inverse?.args),this.setCheckbox("pre-snapshot-enabled",!!e?.preSnapshot),this.setInputValue("pre-snapshot-capability",e?.preSnapshot?.capability||""),this.setJsonInput("pre-snapshot-args",e?.preSnapshot?.args),this.setInputValue("pre-snapshot-capture-json-path",e?.preSnapshot?.captureJsonPath||""),this.setCheckbox("pre-undo-check-enabled",!!e?.preUndoCheck),this.setInputValue("pre-undo-check-capability",e?.preUndoCheck?.capability||""),this.setJsonInput("pre-undo-check-args",e?.preUndoCheck?.args),this.setInputValue("pre-undo-check-capture-json-path",e?.preUndoCheck?.captureJsonPath||""),this.setCheckbox("test-target-enabled",!!e?.testTarget),this.setInputValue("test-target-server",e?.testTarget?.server||""),this.setJsonInput("test-target-args",e?.testTarget?.args),this.updateReversibilityEditor()}clearUndoContractFields(){["ttl-secs","inverse-capability","inverse-args","pre-snapshot-capability","pre-snapshot-args","pre-snapshot-capture-json-path","pre-undo-check-capability","pre-undo-check-args","pre-undo-check-capture-json-path","test-target-server","test-target-args"].forEach((e)=>this.setInputValue(e,"")),["pre-snapshot-enabled","pre-undo-check-enabled","test-target-enabled"].forEach((e)=>this.setCheckbox(e,!1))}setOptionalSectionVisibility(e){let t=document.getElementById(`${e}-enabled`),s=document.querySelector(`[data-optional-field="${e}"]`);if(s)s.hidden=!t?.checked}setInputValue(e,t){let s=document.getElementById(e);if(s)s.value=t}setCheckbox(e,t){let s=document.getElementById(e);if(s)s.checked=t}setJsonInput(e,t){this.setInputValue(e,t?JSON.stringify(t,null,2):"")}showAliasFormError(e){let t=this.aliasStudio?.querySelector("#alias-form-error");if(!t)return;t.textContent=e,t.hidden=!e}updateAliasFieldError(e,t){let s=`${e.id}-error`,r=e.closest(".capability-autocomplete")??(e.id==="test-target-enabled"?e.closest("label"):null)??e,a=r.nextElementSibling;if(!t){if(a?.id===s)a.remove();e.removeAttribute("aria-invalid"),this.setAliasErrorDescription(e,s,!1);return}if(a?.id!==s)a=document.createElement("span"),a.id=s,a.className="alias-field-error",r.after(a);a.textContent=t,e.setAttribute("aria-invalid","true"),this.setAliasErrorDescription(e,s,!0)}setAliasErrorDescription(e,t,s){let r=(e.getAttribute("aria-describedby")||"").split(/\s+/).filter((a)=>a&&a!==t);if(s)r.push(t);if(r.length)e.setAttribute("aria-describedby",r.join(" "));else e.removeAttribute("aria-describedby")}clearAliasFieldErrors(e=!1){this.aliasStudio?.querySelectorAll("#alias-form [aria-invalid]").forEach((t)=>{if(!e||t.closest("[hidden]"))this.updateAliasFieldError(t)})}validateAliasField(e,t=!1){let s=e.target;if(this.aliasSaving||!(s instanceof HTMLElement)||!s.id||s.closest("[hidden]"))return;if(t&&!s.hasAttribute("aria-invalid"))return;this.updateAliasFieldError(s,this.readAliasFields().errors.get(s.id))}refreshAliasDependencyError(){let e=document.getElementById("test-target-enabled");if(e?.hasAttribute("aria-invalid"))this.updateAliasFieldError(e,this.readAliasFields().errors.get(e.id))}readAliasFields(){let e=new Map,t=(d,f)=>{let x=document.getElementById(d)?.value.trim();if(!x)e.set(d,`${f} is required.`);return x||void 0};if(t("alias-name","Public alias"),t("alias-target","Canonical target"),document.getElementById("alias-kind").value!=="tool")return{contract:void 0,errors:e};let s=document.getElementById("reversibility-class")?.value||"";if(!s)return{contract:null,errors:e};let r=(d,f,x)=>{let b=document.getElementById(d),h=b?.value.trim()||"";if(b?.validity.badInput){e.set(d,`${f} must be a positive whole number.`);return}if(!h){if(x)e.set(d,`${f} is required.`);return}let T=Number(h);if(!Number.isSafeInteger(T)||T<1){e.set(d,`${f} must be a positive whole number.`);return}return T},a=(d,f)=>{let x=document.getElementById(d)?.value.trim()||"";if(!x){e.set(d,`${f} is required and must be a JSON object.`);return}try{let b=JSON.parse(x);if(!b||typeof b!=="object"||Array.isArray(b)){e.set(d,`${f} must be a JSON object.`);return}return b}catch{e.set(d,`${f} contains invalid JSON.`);return}};if(s==="delayable"){let d=r("hold-window-secs","Hold window",!1);return{contract:{class:s,holdWindowSecs:d},errors:e}}if(s==="irreversible")return{contract:{class:s},errors:e};let o=t("inverse-capability","Inverse capability"),n=a("inverse-args","Inverse arguments"),l=r("ttl-secs","Undo window",!1),c=s==="compensable"?r("daily-limit","Daily limit",!0):void 0,g={class:s,...l===void 0?{}:{ttlSecs:l},...c===void 0?{}:{dailyLimit:c},...o&&n?{inverse:{capability:o,args:n}}:{}},u=(d)=>{if(!document.getElementById(`${d}-enabled`)?.checked)return;let x=t(`${d}-capability`,d==="pre-snapshot"?"Snapshot capability":"Drift-check capability"),b=a(`${d}-args`,d==="pre-snapshot"?"Snapshot arguments":"Drift-check arguments"),h=document.getElementById(`${d}-capture-json-path`)?.value.trim()||void 0;return x&&b?{capability:x,args:b,...h?{captureJsonPath:h}:{}}:void 0},m=u("pre-snapshot"),v=u("pre-undo-check");if(m)g.preSnapshot=m;if(v)g.preUndoCheck=v;if(document.getElementById("test-target-enabled")?.checked){let d=t("test-target-server","Test server"),f=a("test-target-args","Test arguments");if(!document.getElementById("pre-snapshot-enabled").checked)e.set("test-target-enabled","Enable and configure a pre-call snapshot to test this contract.");if(d&&f)g.testTarget={server:d,args:f}}return{contract:g,errors:e}}async createAlias(){let e=document.getElementById("alias-form");if(!e||this.aliasSaving)return;let t=document.getElementById("alias-kind")?.value,s=document.getElementById("alias-name")?.value.trim(),r=document.getElementById("alias-target")?.value.trim(),a=document.getElementById("alias-summary")?.value.trim()||void 0,o=document.getElementById("alias-passthrough")?.checked||!1;this.showAliasFormError("");let{contract:n,errors:l}=this.readAliasFields();if(this.clearAliasFieldErrors(),l.size){let v=[...e.elements].filter((d)=>d instanceof HTMLElement&&l.has(d.id));v.forEach((d)=>this.updateAliasFieldError(d,l.get(d.id))),v[0]?.focus();return}if(t==="tool"&&o&&!/^[a-zA-Z0-9_-]{1,64}$/.test(s)){let v=s.replace(/[^a-zA-Z0-9_-]/g,"_").substring(0,64);if(!confirm(`MCP tool names must match ^[a-zA-Z0-9_-]{1,64}$.

'${s}' will be exported to MCP clients as '${v}'.

Do you want to proceed?`))return}let c=X(t,s),g=typeof c==="object"?c.description:void 0,u=e.querySelector("#alias-save-btn"),m=u?.textContent??"Save";if(e.querySelectorAll('[role="combobox"]').forEach(H),this.aliasSaving=!0,e.inert=!0,e.setAttribute("aria-busy","true"),u)u.textContent="Saving…";try{let v=await y.updateAlias(t,s,r,a,g,o,n);if(!v.ok){this.showAliasFormError(v.error||"Alias configuration could not be saved.");return}}catch(v){this.showAliasFormError(v?.message||"Alias configuration could not be saved.");return}finally{if(this.aliasSaving=!1,e.inert=!1,e.removeAttribute("aria-busy"),u)u.textContent=m}if(e.isConnected)this.resetAliasForm();else this.aliasStudio=void 0;await this.refreshData()}async deleteAlias(e,t){try{let s=await y.updateAlias(e,t);if(!s.ok){this.showAliasFormError(s.error||"Alias could not be deleted.");return}await this.refreshData()}catch{this.showAliasFormError("Alias could not be deleted. Check the connection and try again.")}}async reloadFromDisk(){try{let e=await y.reloadConfig();if(e.ok){let t="Hot-reload completed successfully!";if(e.mounted&&e.mounted.length>0)t+=`
Mounted: ${e.mounted.join(", ")}`;if(e.unmounted&&e.unmounted.length>0)t+=`
Unmounted: ${e.unmounted.join(", ")}`;if(e.warnings&&e.warnings.length>0)t+=`
Warnings:
${e.warnings.join(`
`)}`;alert(t)}else alert(`Hot-reload failed: ${e.error||"Unknown error"}`)}catch(e){alert(`Error reaching daemon: ${e.message}`)}await this.refreshData()}renderTopProfileSelector(){let e=document.getElementById("top-profile-selector");if(!e)return;let t=p.getState(),s=t.config.profiles||{},r=Object.keys(s),a=t.activeProfile,o='<option value="">All Servers (Unrestricted)</option>';for(let n of r){let l=a===n?"selected":"";o+=`<option value="${i(n)}" ${l}>Profile: ${i(n)}</option>`}e.innerHTML=o}async setActiveProfile(e){p.setState({activeProfile:e||null,tasks:[],taskError:null}),await this.refreshData()}openAddProfileModal(){let e=document.getElementById("modal-prof-title");if(e)e.textContent="Create Server Constellation Profile";let t=document.getElementById("modal-prof-name"),s=document.getElementById("modal-prof-desc"),r=document.getElementById("modal-prof-mode");if(t)t.value="",t.disabled=!1;if(s)s.value="";if(r)r.value="create";let a=document.getElementById("modal-prof-allow"),o=document.getElementById("modal-prof-deny"),n=document.getElementById("modal-prof-hitl"),l=document.getElementById("modal-prof-redact");if(a)a.value="";if(o)o.value="";if(n)n.value="";if(l)l.value="";this.renderProfileServerCheckboxes([]);let c=document.getElementById("modal-add-profile");if(c)c.classList.add("active")}openEditProfileModal(e){let s=p.getState().config.profiles?.[e];if(!s)return;let r=document.getElementById("modal-prof-title");if(r)r.textContent=`Edit Profile: ${e}`;let a=document.getElementById("modal-prof-name"),o=document.getElementById("modal-prof-desc"),n=document.getElementById("modal-prof-mode");if(a)a.value=e,a.disabled=!0;if(o)o.value=s.description||"";if(n)n.value="edit";let l=document.getElementById("modal-prof-allow"),c=document.getElementById("modal-prof-deny"),g=document.getElementById("modal-prof-hitl"),u=document.getElementById("modal-prof-redact"),m=s.policy;if(l)l.value=(m?.allow||[]).join(", ");if(c)c.value=(m?.deny||[]).join(", ");if(g)g.value=(m?.require_approval||m?.requireApproval||[]).join(", ");if(u)u.value=(m?.redact_keys||m?.redactKeys||[]).join(", ");this.renderProfileServerCheckboxes(s.servers||[]);let v=document.getElementById("modal-add-profile");if(v)v.classList.add("active")}renderProfileServerCheckboxes(e){let t=document.getElementById("modal-prof-servers-list");if(!t)return;let s=p.getState(),r=Object.keys(s.config.mcpServers||{});if(r.length===0){t.innerHTML='<div style="font-size: 11.5px; color: var(--text-dim);">No MCP servers configured yet. Add servers first.</div>';return}t.innerHTML=r.map((a)=>{let o=e.includes(a)?"checked":"";return`
        <label style="display: flex; align-items: center; gap: 8px; font-size: 12px; cursor: pointer; padding: 4px 6px; border-radius: var(--radius-sm); transition: background 0.15s;" onmouseover="this.style.background='var(--surface-hover)'" onmouseout="this.style.background='transparent'">
          <input type="checkbox" class="prof-server-checkbox" value="${i(a)}" ${o} style="accent-color: var(--amber-400);">
          <span style="font-family: var(--ff-mono); font-weight: 600; color: var(--text-main);">${i(a)}</span>
        </label>
      `}).join("")}async saveProfile(){let e=document.getElementById("modal-prof-name"),t=document.getElementById("modal-prof-desc"),s=e?.value.trim(),r=t?.value.trim();if(!s){alert("Please enter a profile name");return}let a=document.querySelectorAll(".prof-server-checkbox:checked"),o=[];if(a.forEach((b)=>{o.push(b.value)}),o.length===0){alert("Please select at least one server to include in this constellation");return}let n=(b)=>{if(!b)return[];return b.split(",").map((h)=>h.trim()).filter((h)=>h.length>0)},l=document.getElementById("modal-prof-allow"),c=document.getElementById("modal-prof-deny"),g=document.getElementById("modal-prof-hitl"),u=document.getElementById("modal-prof-redact"),m=n(l?.value),v=n(c?.value),d=n(g?.value),f=n(u?.value),x=void 0;if(m.length>0||v.length>0||d.length>0||f.length>0)x={allow:m,deny:v,requireApproval:d,redactKeys:f};try{let b=await y.upsertProfile(s,o,r||void 0,x);if(b.ok)this.closeModals(),await this.refreshData();else alert(`Failed to save profile: ${b.error||"Unknown error"}`)}catch(b){alert(`Error saving profile: ${b.message}`)}}async deleteProfile(e){if(!confirm(`Are you sure you want to delete profile '${e}'?`))return;try{let t=await y.deleteProfile(e);if(t.ok){if(p.getState().activeProfile===e)p.setState({activeProfile:null});await this.refreshData()}else alert(`Failed to delete profile: ${t.error||"Unknown error"}`)}catch(t){alert(`Error deleting profile: ${t.message}`)}}async toggleServerInProfile(e,t,s){let a=p.getState().config.profiles?.[e];if(!a)return;let o=[...a.servers||[]];if(s){if(!o.includes(t))o.push(t)}else if(o=o.filter((n)=>n!==t),o.length===0){alert("A profile must contain at least one server. To remove the profile, delete it in the Profiles tab.");return}try{let n=await y.upsertProfile(e,o,a.description,a.policy);if(n.ok)await this.refreshData();else alert(`Failed to update profile constellation: ${n.error||"Unknown error"}`)}catch(n){alert(`Error updating profile constellation: ${n.message}`)}}closeModals(){document.querySelectorAll(".modal-backdrop").forEach((e)=>e.classList.remove("active"))}}var Oe=new je;window.app=Oe;window.addEventListener("DOMContentLoaded",()=>Oe.init());
