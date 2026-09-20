import { spawn } from "node:child_process";
import { existsSync } from "node:fs";
import { tmpdir } from "node:os";
import { resolve } from "node:path";

const ENTRY = resolve(__dirname, "../../dist/main.js");

const VALID_ENVIRONMENT = {
  ALLOWED_ORIGINS: "http://localhost:3000",
  APP_ORIGIN: "http://localhost:3000",
  DATABASE_URL: "postgresql://clinicore:local@localhost:5432/clinicore",
  PORT: "3333",
};

interface BootFailure {
  code: number | null;
  stderr: string;
  stdout: string;
}

function bootWithout(
  name: keyof typeof VALID_ENVIRONMENT,
): Promise<BootFailure> {
  const environment = Object.fromEntries(
    Object.entries(VALID_ENVIRONMENT).filter(([variable]) => variable !== name),
  );

  const child = spawn(process.execPath, [ENTRY], {
    cwd: tmpdir(),
    env: { PATH: process.env.PATH ?? "", ...environment },
  });

  let stderr = "";
  let stdout = "";
  child.stderr.on("data", (chunk: Buffer) => (stderr += chunk.toString()));
  child.stdout.on("data", (chunk: Buffer) => (stdout += chunk.toString()));

  return new Promise((settle, fail) => {
    child.on("error", fail);
    child.on("close", (code) => {
      settle({ code, stderr, stdout });
    });
  });
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
