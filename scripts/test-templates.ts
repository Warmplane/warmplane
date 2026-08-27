import { spawn } from 'child_process';
import { SERVER_TEMPLATES, type ServerTemplate } from '../ui/src/templates';
import * as fs from 'fs';
import * as path from 'path';
import * as os from 'os';

interface TestResult {
  id: string;
  name: string;
  command: string;
  status: 'PASS' | 'FAIL' | 'TIMEOUT';
  toolCount?: number;
  error?: string;
  durationMs: number;
}

async function smokeTestTemplate(tmpl: ServerTemplate): Promise<TestResult> {
  const start = Date.now();
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), `wp-smoke-${tmpl.id}-`));

  // Prepare dummy args/files for templates needing paths
  const args = [...tmpl.defaultArgs];
  const env: Record<string, string> = {
    ...process.env as Record<string, string>,
    PYTHONUNBUFFERED: '1',
  };

  // Provide mock values for required env fields
  for (const ef of tmpl.envFields) {
    env[ef.key] = `mock_${ef.key.toLowerCase()}_token_12345`;
  }

  // Handle specific template requirements
  if (tmpl.id === 'sqlite') {
    const dbPath = path.join(tempDir, 'test.db');
    fs.writeFileSync(dbPath, '');
    const idx = args.indexOf('./app.db');
    if (idx !== -1) args[idx] = dbPath;
  } else if (tmpl.id === 'filesystem') {
    const idx = args.indexOf('.');
    if (idx !== -1) args[idx] = tempDir;
  } else if (tmpl.id === 'git') {
    const { execSync } = require('child_process');
    try {
      execSync('git init -b main && git config user.email "test@example.com" && git config user.name "Test" && touch README.md && git add README.md && git commit -m "init"', { cwd: tempDir, stdio: 'ignore' });
    } catch {}
    const idx = args.indexOf('.');
    if (idx !== -1) args[idx] = tempDir;
  } else if (tmpl.id === 'kubernetes') {
    // Only pass KUBECONFIG if pointing to a valid mock yaml file
    const kubePath = path.join(tempDir, 'kubeconfig.yaml');
    fs.writeFileSync(kubePath, 'apiVersion: v1\nclusters: []\ncontexts: []\ncurrent-context: ""\nkind: Config\npreferences: {}\nusers: []\n');
    env.KUBECONFIG = kubePath;
  }

  let redisServer: any = null;
  if (tmpl.id === 'redis') {
    const net = require('net');
    redisServer = net.createServer((socket: any) => {
      socket.on('data', (data: Buffer) => {
        const str = data.toString();
        if (str.includes('COMMAND') || str.includes('command')) {
          socket.write('*0\r\n');
        } else if (str.includes('HELLO') || str.includes('hello')) {
          socket.write('%2\r\n$6\r\nserver\r\n$5\r\nredis\r\n$7\r\nversion\r\n$5\r\n7.0.0\r\n');
        } else if (str.includes('PING') || str.includes('ping')) {
          socket.write('+PONG\r\n');
        } else if (str.includes('INFO') || str.includes('info')) {
          socket.write('$17\r\n# Server\r\nredis_ver:7\r\n');
        } else if (str.includes('CLIENT') || str.includes('client')) {
          socket.write('+OK\r\n');
        } else {
          socket.write('+OK\r\n');
        }
      });
    });
    await new Promise<void>((r) => redisServer.listen(6379, '127.0.0.1', () => r()));
  }

  return new Promise<TestResult>((resolve) => {
    let child: any = null;
    let stdoutBuffer = '';
    let stderrBuffer = '';
    let resolved = false;

    const timeout = setTimeout(() => {
      if (!resolved) {
        resolved = true;
        cleanup();
        resolve({
          id: tmpl.id,
          name: tmpl.name,
          command: `${tmpl.command} ${args.join(' ')}`,
          status: 'TIMEOUT',
          error: `Timed out after 15000ms. Stderr: ${stderrBuffer.slice(-300)}`,
          durationMs: Date.now() - start,
        });
      }
    }, 15000);

    const cleanup = () => {
      clearTimeout(timeout);
      if (redisServer) {
        try { redisServer.close(); } catch {}
      }
      if (child) {
        try {
          child.kill('SIGTERM');
          setTimeout(() => {
            try { child.kill('SIGKILL'); } catch {}
          }, 500);
        } catch {}
      }
      try {
        fs.rmSync(tempDir, { recursive: true, force: true });
      } catch {}
    };

    try {
      child = spawn(tmpl.command, args, {
        env,
        cwd: tempDir,
        stdio: ['pipe', 'pipe', 'pipe'],
      });
    } catch (err: any) {
      cleanup();
      return resolve({
        id: tmpl.id,
        name: tmpl.name,
        command: `${tmpl.command} ${args.join(' ')}`,
        status: 'FAIL',
        error: `Failed to spawn: ${err.message}`,
        durationMs: Date.now() - start,
      });
    }

    child.stderr.on('data', (data: Buffer) => {
      stderrBuffer += data.toString();
    });

    child.on('error', (err: any) => {
      if (!resolved) {
        resolved = true;
        cleanup();
        resolve({
          id: tmpl.id,
          name: tmpl.name,
          command: `${tmpl.command} ${args.join(' ')}`,
          status: 'FAIL',
          error: `Process error: ${err.message}\nStderr: ${stderrBuffer.slice(-300)}`,
          durationMs: Date.now() - start,
        });
      }
    });

    child.on('exit', (code: number, signal: string) => {
      if (!resolved) {
        resolved = true;
        cleanup();
        resolve({
          id: tmpl.id,
          name: tmpl.name,
          command: `${tmpl.command} ${args.join(' ')}`,
          status: 'FAIL',
          error: `Exited prematurely with code ${code}, signal ${signal}.\nStderr: ${stderrBuffer.slice(-400)}`,
          durationMs: Date.now() - start,
        });
      }
    });

    // Send MCP initialize request
    const initRequest = JSON.stringify({
      jsonrpc: '2.0',
      id: 1,
      method: 'initialize',
      params: {
        protocolVersion: '2024-11-05',
        capabilities: {},
        clientInfo: { name: 'warmplane-smoke-tester', version: '1.0' },
      },
    }) + '\n';

    child.stdin.write(initRequest);

    child.stdout.on('data', (data: Buffer) => {
      stdoutBuffer += data.toString();
      const lines = stdoutBuffer.split('\n');
      for (const line of lines) {
        if (!line.trim()) continue;
        try {
          const msg = JSON.parse(line.trim());
          if (msg.id === 1 && msg.result) {
            // Handshake successful, request tools/list
            const initNotif = JSON.stringify({
              jsonrpc: '2.0',
              method: 'notifications/initialized',
            }) + '\n';
            const listToolsReq = JSON.stringify({
              jsonrpc: '2.0',
              id: 2,
              method: 'tools/list',
            }) + '\n';
            child.stdin.write(initNotif);
            child.stdin.write(listToolsReq);
          } else if (msg.id === 2 && msg.result) {
            if (!resolved) {
              resolved = true;
              const toolCount = Array.isArray(msg.result.tools) ? msg.result.tools.length : 0;
              cleanup();
              resolve({
                id: tmpl.id,
                name: tmpl.name,
                command: `${tmpl.command} ${args.join(' ')}`,
                status: 'PASS',
                toolCount,
                durationMs: Date.now() - start,
              });
            }
          }
        } catch {
          // not full json yet
        }
      }
    });
  });
}

async function main() {
  console.log(`\n===============================================================`);
  console.log(`  Warmplane MCP Template Library Smoke Test Suite`);
  console.log(`  Testing ${SERVER_TEMPLATES.length} templates...`);
  console.log(`===============================================================\n`);

  const results: TestResult[] = [];

  for (const tmpl of SERVER_TEMPLATES) {
    process.stdout.write(`Testing [${tmpl.id.padEnd(16)}] (${tmpl.name})... `);
    const res = await smokeTestTemplate(tmpl);
    results.push(res);
    if (res.status === 'PASS') {
      console.log(`\x1b[32mPASS\x1b[0m (${res.toolCount} tools, ${res.durationMs}ms)`);
    } else {
      console.log(`\x1b[31m${res.status}\x1b[0m (${res.durationMs}ms)`);
      if (res.error) {
        console.log(`   \x1b[33mError:\x1b[0m ${res.error.trim().replace(/\n/g, '\n   ')}`);
      }
    }
  }

  console.log(`\n===============================================================`);
  console.log(`  Summary:`);
  const passed = results.filter(r => r.status === 'PASS');
  const failed = results.filter(r => r.status !== 'PASS');
  console.log(`  Passed: ${passed.length}/${results.length}`);
  console.log(`  Failed: ${failed.length}/${results.length}`);
  console.log(`===============================================================\n`);

  if (failed.length > 0) {
    console.log(`Failed templates:`);
    for (const f of failed) {
      console.log(`- ${f.id} (${f.command}): ${f.error?.slice(0, 120)}`);
    }
  }
}

main().catch(err => {
  console.error(err);
  process.exit(1);
});
