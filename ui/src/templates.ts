export interface EnvFieldDef {
  key: string;
  label: string;
  placeholder: string;
  required: boolean;
  description?: string;
}

export interface ServerTemplate {
  id: string;
  name: string;
  category: 'devtools' | 'browser' | 'database' | 'productivity' | 'cloud';
  description: string;
  badge: string;
  command: string;
  defaultArgs: string[];
  argsPlaceholder?: string;
  envFields: EnvFieldDef[];
  docsUrl?: string;
}

export const SERVER_TEMPLATES: ServerTemplate[] = [
  // 1. Developer Tools
  {
    id: 'github',
    name: 'GitHub',
    category: 'devtools',
    description: 'Explore repositories, issues, pull requests, branches, and commit histories.',
    badge: 'Official / Stdio',
    command: 'npx',
    defaultArgs: ['-y', '@modelcontextprotocol/server-github'],
    envFields: [
      { key: 'GITHUB_PERSONAL_ACCESS_TOKEN', label: 'GitHub Personal Access Token', placeholder: 'ghp_...', required: true, description: 'Classic or fine-grained token with repo scope.' }
    ]
  },
  {
    id: 'git',
    name: 'Git (Local)',
    category: 'devtools',
    description: 'Read local Git repository status, diffs, log histories, and commit changes.',
    badge: 'Official / uvx',
    command: 'uvx',
    defaultArgs: ['mcp-server-git', '--repository', '.'],
    argsPlaceholder: 'mcp-server-git --repository /path/to/repo',
    envFields: []
  },
  {
    id: 'filesystem',
    name: 'Filesystem',
    category: 'devtools',
    description: 'Secure, sandboxed access to local files and directories for AI workflows.',
    badge: 'Official / Stdio',
    command: 'npx',
    defaultArgs: ['-y', '@modelcontextprotocol/server-filesystem', '.'],
    argsPlaceholder: '-y @modelcontextprotocol/server-filesystem /allowed/dir1 /allowed/dir2',
    envFields: []
  },
  {
    id: 'memory',
    name: 'Memory Graph',
    category: 'devtools',
    description: 'Persistent knowledge-graph based memory for multi-turn agent learning.',
    badge: 'Official / Stdio',
    command: 'npx',
    defaultArgs: ['-y', '@modelcontextprotocol/server-memory'],
    envFields: []
  },
  {
    id: 'chrome-devtools',
    name: 'Chrome DevTools',
    category: 'devtools',
    description: 'Inspect live DOM, execute scripts, read console logs, and capture network traces in Chrome.',
    badge: 'Official / Stdio',
    command: 'npx',
    defaultArgs: ['-y', '@modelcontextprotocol/server-puppeteer'],
    envFields: []
  },
  {
    id: 'sentry',
    name: 'Sentry',
    category: 'devtools',
    description: 'Query production error events, stack traces, and issue frequencies directly from Sentry.',
    badge: 'uvx / Telemetry',
    command: 'uvx',
    defaultArgs: ['mcp-server-sentry'],
    envFields: [
      { key: 'SENTRY_AUTH_TOKEN', label: 'Sentry Auth Token', placeholder: 'sntrys_...', required: true }
    ]
  },

  // 2. Browser & Search
  {
    id: 'playwright',
    name: 'Playwright Browser',
    category: 'browser',
    description: 'Headless / headed browser automation for scraping, form filling, and UI interaction.',
    badge: 'Popular #1 / npx',
    command: 'npx',
    defaultArgs: ['-y', '@executeautomation/playwright-mcp-server'],
    envFields: []
  },
  {
    id: 'puppeteer',
    name: 'Puppeteer',
    category: 'browser',
    description: 'Official browser automation server for web page scraping and screenshot capture.',
    badge: 'Official / Stdio',
    command: 'npx',
    defaultArgs: ['-y', '@modelcontextprotocol/server-puppeteer'],
    envFields: []
  },
  {
    id: 'brave-search',
    name: 'Brave Search',
    category: 'browser',
    description: 'Real-time privacy-preserving web search and local point-of-interest query engine.',
    badge: 'Official / Search',
    command: 'npx',
    defaultArgs: ['-y', '@modelcontextprotocol/server-brave-search'],
    envFields: [
      { key: 'BRAVE_API_KEY', label: 'Brave Search API Key', placeholder: 'BSA...', required: true }
    ]
  },
  {
    id: 'tavily',
    name: 'Tavily Search',
    category: 'browser',
    description: 'AI-optimized web search engine structured specifically for LLM context injection.',
    badge: 'Community / Stdio',
    command: 'npx',
    defaultArgs: ['-y', '@tavily/mcp-server'],
    envFields: [
      { key: 'TAVILY_API_KEY', label: 'Tavily API Key', placeholder: 'tvly-...', required: true }
    ]
  },
  {
    id: 'fetch',
    name: 'Fetch / Web Markdown',
    category: 'browser',
    description: 'Download web pages, strip clutter, and convert raw HTML to clean markdown text.',
    badge: 'Official / uvx',
    command: 'uvx',
    defaultArgs: ['mcp-server-fetch'],
    envFields: []
  },

  // 3. Databases & Storage
  {
    id: 'postgres',
    name: 'PostgreSQL',
    category: 'database',
    description: 'Read schemas, inspect tables, and execute SQL queries against PostgreSQL databases.',
    badge: 'Official / Database',
    command: 'npx',
    defaultArgs: ['-y', '@modelcontextprotocol/server-postgres', 'postgresql://user:pass@localhost:5432/mydb'],
    argsPlaceholder: '-y @modelcontextprotocol/server-postgres postgresql://user:pass@localhost:5432/dbname',
    envFields: []
  },
  {
    id: 'sqlite',
    name: 'SQLite',
    category: 'database',
    description: 'Local embedded SQLite query runner and schema inspector.',
    badge: 'Official / uvx',
    command: 'uvx',
    defaultArgs: ['mcp-server-sqlite', '--db-path', './app.db'],
    argsPlaceholder: 'mcp-server-sqlite --db-path /path/to/database.sqlite',
    envFields: []
  },
  {
    id: 'supabase',
    name: 'Supabase',
    category: 'database',
    description: 'Query database tables, manage auth policies, and inspect storage in Supabase.',
    badge: 'Official / Stdio',
    command: 'npx',
    defaultArgs: ['-y', '@supabase/mcp-server'],
    envFields: [
      { key: 'SUPABASE_ACCESS_TOKEN', label: 'Supabase Personal Access Token', placeholder: 'sbp_...', required: true },
      { key: 'SUPABASE_PROJECT_REF', label: 'Supabase Project Reference ID', placeholder: 'abcdefghijklmnop', required: false }
    ]
  },
  {
    id: 'redis',
    name: 'Redis',
    category: 'database',
    description: 'Inspect cached keys, hash sets, lists, TTLs, and pub/sub channels in Redis.',
    badge: 'uvx / Key-Value',
    command: 'uvx',
    defaultArgs: ['mcp-server-redis', '--url', 'redis://localhost:6379'],
    argsPlaceholder: 'mcp-server-redis --url redis://localhost:6379',
    envFields: []
  },
  {
    id: 's3',
    name: 'AWS S3 / Cloud Storage',
    category: 'database',
    description: 'Browse S3 buckets, fetch object metadata, and download files from cloud storage.',
    badge: 'uvx / Cloud Storage',
    command: 'uvx',
    defaultArgs: ['mcp-server-s3', '--bucket', 'my-bucket-name'],
    argsPlaceholder: 'mcp-server-s3 --bucket bucket-name --region us-east-1',
    envFields: [
      { key: 'AWS_ACCESS_KEY_ID', label: 'AWS Access Key ID', placeholder: 'AKIA...', required: true },
      { key: 'AWS_SECRET_ACCESS_KEY', label: 'AWS Secret Access Key', placeholder: '...', required: true },
      { key: 'AWS_REGION', label: 'AWS Region', placeholder: 'us-east-1', required: false }
    ]
  },

  // 4. Productivity & Workspace
  {
    id: 'linear',
    name: 'Linear',
    category: 'productivity',
    description: 'Search, create, and triage Linear issues, cycles, teams, and project roadmaps.',
    badge: 'Productivity / Stdio',
    command: 'npx',
    defaultArgs: ['-y', 'mcp-linear'],
    envFields: [
      { key: 'LINEAR_API_KEY', label: 'Linear API Key', placeholder: 'lin_api_...', required: true }
    ]
  },
  {
    id: 'slack',
    name: 'Slack',
    category: 'productivity',
    description: 'Read channels, post messages, inspect threads, and search team discussions.',
    badge: 'Official / Stdio',
    command: 'npx',
    defaultArgs: ['-y', '@modelcontextprotocol/server-slack'],
    envFields: [
      { key: 'SLACK_BOT_TOKEN', label: 'Slack Bot User Token', placeholder: 'xoxb-...', required: true },
      { key: 'SLACK_TEAM_ID', label: 'Slack Team ID', placeholder: 'T01234567', required: true }
    ]
  },
  {
    id: 'notion',
    name: 'Notion',
    category: 'productivity',
    description: 'Search Notion workspace pages, read nested blocks, and query database entries.',
    badge: 'Official / Stdio',
    command: 'npx',
    defaultArgs: ['-y', '@modelcontextprotocol/server-notion'],
    envFields: [
      { key: 'NOTION_API_KEY', label: 'Notion Internal Integration Token', placeholder: 'secret_...', required: true }
    ]
  },
  {
    id: 'jira',
    name: 'Jira / Atlassian',
    category: 'productivity',
    description: 'Manage Jira issues, search JQL, read sprint statuses, and inspect boards.',
    badge: 'uvx / Atlassian',
    command: 'uvx',
    defaultArgs: ['mcp-server-jira', '--url', 'https://your-domain.atlassian.net', '--email', 'user@example.com'],
    argsPlaceholder: 'mcp-server-jira --url https://org.atlassian.net --email me@org.com',
    envFields: [
      { key: 'JIRA_API_TOKEN', label: 'Atlassian API Token', placeholder: 'ATATT3...', required: true }
    ]
  },
  {
    id: 'google-drive',
    name: 'Google Drive',
    category: 'productivity',
    description: 'Search, list, and read documents, spreadsheets, and drive files.',
    badge: 'Official / Stdio',
    command: 'npx',
    defaultArgs: ['-y', '@modelcontextprotocol/server-gdrive'],
    envFields: [
      { key: 'GOOGLE_APPLICATION_CREDENTIALS', label: 'Google Credentials JSON Path', placeholder: '/path/to/credentials.json', required: true }
    ]
  },

  // 5. Cloud & Infrastructure
  {
    id: 'docker',
    name: 'Docker',
    category: 'cloud',
    description: 'Inspect running containers, tail container logs, list images, and manage compose services.',
    badge: 'uvx / DevOps',
    command: 'uvx',
    defaultArgs: ['mcp-server-docker'],
    envFields: []
  },
  {
    id: 'kubernetes',
    name: 'Kubernetes (K8s)',
    category: 'cloud',
    description: 'Query cluster pods, services, deployment status, and inspect Kubernetes logs.',
    badge: 'Official / Stdio',
    command: 'npx',
    defaultArgs: ['-y', '@modelcontextprotocol/server-kubernetes'],
    envFields: [
      { key: 'KUBECONFIG', label: 'Kubeconfig File Path (Optional)', placeholder: '~/.kube/config', required: false }
    ]
  },
  {
    id: 'cloudflare',
    name: 'Cloudflare',
    category: 'cloud',
    description: 'Manage Cloudflare Workers, KV namespaces, D1 databases, Vectorize indexes, and DNS.',
    badge: 'Official / Cloudflare',
    command: 'npx',
    defaultArgs: ['-y', '@cloudflare/mcp-server-cloudflare'],
    envFields: [
      { key: 'CLOUDFLARE_API_TOKEN', label: 'Cloudflare API Token', placeholder: '...', required: true },
      { key: 'CLOUDFLARE_ACCOUNT_ID', label: 'Cloudflare Account ID', placeholder: '...', required: true }
    ]
  },
  {
    id: 'terraform',
    name: 'Terraform',
    category: 'cloud',
    description: 'Inspect Terraform state files, resource dependency graphs, and plan previews.',
    badge: 'uvx / IaC',
    command: 'uvx',
    defaultArgs: ['mcp-server-terraform'],
    envFields: []
  }
];
