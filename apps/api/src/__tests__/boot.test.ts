import { spawn, type ChildProcess } from "node:child_process";
import { existsSync } from "node:fs";
import { createServer } from "node:net";
import { tmpdir } from "node:os";
import { resolve } from "node:path";
import { setTimeout as delay } from "node:timers/promises";

const ENTRY = resolve(__dirname, "../../dist/main.js");

const VALID_ENVIRONMENT = {
  ALLOWED_ORIGINS: "http://localhost:3000",
  APP_ORIGIN: "http://localhost:3000",
  DATABASE_URL: "postgresql://clinicore:local@localhost:5432/clinicore",
  NODE_ENV: "development",
  PORT: "3333",
};

const BOOT_TIMEOUT_MS = 30_000;
const POLL_INTERVAL_MS = 50;

interface BootFailure {
  code: number | null;
  stderr: string;
  stdout: string;
}

function spawnApi(environment: Record<string, string>): ChildProcess {
  return spawn(process.execPath, [ENTRY], {
    cwd: tmpdir(),
    env: { PATH: process.env.PATH ?? "", ...environment },
  });
}

function bootWithout(
  name: keyof typeof VALID_ENVIRONMENT,
): Promise<BootFailure> {
  const environment = Object.fromEntries(
    Object.entries(VALID_ENVIRONMENT).filter(([variable]) => variable !== name),
  );

  const child = spawnApi(environment);

  let stderr = "";
  let stdout = "";
  child.stderr?.on("data", (chunk: Buffer) => (stderr += chunk.toString()));
  child.stdout?.on("data", (chunk: Buffer) => (stdout += chunk.toString()));

  return new Promise((settle, fail) => {
    child.on("error", fail);
    child.on("close", (code) => {
      settle({ code, stderr, stdout });
    });
  });
}

function freePort(): Promise<number> {
  const probe = createServer();

  return new Promise((settle, fail) => {
    probe.on("error", fail);
    probe.listen(0, "127.0.0.1", () => {
      const address = probe.address();
      if (address === null || typeof address === "string") {
        fail(new Error("the probe server reported no port"));
        return;
      }
      probe.close(() => {
        settle(address.port);
      });
    });
  });
}

async function untilListening(origin: string, child: ChildProcess) {
  const deadline = Date.now() + BOOT_TIMEOUT_MS;

  while (Date.now() < deadline) {
    if (child.exitCode !== null) {
      throw new Error(`the api exited with ${String(child.exitCode)} on boot`);
    }
    const reached = await fetch(`${origin}/health`).catch(() => null);
    if (reached !== null) {
      return;
    }
    await delay(POLL_INTERVAL_MS);
  }

  throw new Error(`the api did not listen on ${origin} in time`);
}

async function documentationStatus(nodeEnvironment: string): Promise<number> {
  const port = await freePort();
  const child = spawnApi({
    ...VALID_ENVIRONMENT,
    NODE_ENV: nodeEnvironment,
    PORT: String(port),
  });
  const origin = `http://127.0.0.1:${String(port)}`;

  try {
    await untilListening(origin, child);
    const response = await fetch(`${origin}/api`);
    return response.status;
  } finally {
    child.kill("SIGKILL");
  }
}

describe("boot with an invalid environment", () => {
  it("has a build to boot", () => {
    expect(existsSync(ENTRY)).toBe(true);
  });

  it("writes only the rejection to stderr and exits 1", async () => {
    const failure = await bootWithout("DATABASE_URL");

    expect(failure.stderr.split("\n")).toEqual([
      "Invalid environment:",
      "  DATABASE_URL: expected a PostgreSQL connection string (postgresql://…)",
      "",
    ]);
    expect(failure.stdout).toBe("");
    expect(failure.code).toBe(1);
  });
});

describe("the api documentation", () => {
  it(
    "is mounted outside production",
    async () => {
      await expect(documentationStatus("development")).resolves.toBe(200);
    },
    BOOT_TIMEOUT_MS,
  );

  it(
    "is absent in production",
    async () => {
      await expect(documentationStatus("production")).resolves.toBe(404);
    },
    BOOT_TIMEOUT_MS,
  );
});
