import { validateEnv } from "../env.validation";

const VALID = {
  ALLOWED_ORIGINS: "http://localhost:3000",
  APP_ORIGIN: "http://localhost:3000",
  DATABASE_URL: "postgresql://clinicore:local@localhost:5432/clinicore",
  PORT: "3333",
};

function without(...names: string[]): Record<string, string> {
  return Object.fromEntries(
    Object.entries(VALID).filter(([name]) => !names.includes(name)),
  );
}

function rejectionOf(source: Record<string, unknown>): string {
  try {
    validateEnv(source);
  } catch (error) {
    if (error instanceof Error) {
      return error.message;
    }
    throw error;
  }
  throw new Error(
    "validateEnv accepted an environment it should have rejected",
  );
}

describe("validateEnv", () => {
  it("returns PORT as a number and ALLOWED_ORIGINS as a list", () => {
    const environment = validateEnv({
      ...VALID,
      ALLOWED_ORIGINS: "http://localhost:3000,https://clinicore.com.br",
    });

    expect(environment.PORT).toBe(3333);
    expect(environment.DATABASE_URL).toBe(VALID.DATABASE_URL);
    expect(environment.APP_ORIGIN).toBe(VALID.APP_ORIGIN);
    expect(environment.ALLOWED_ORIGINS).toEqual([
      "http://localhost:3000",
      "https://clinicore.com.br",
    ]);
  });

  it("rejects a missing DATABASE_URL naming only the expected format", () => {
    expect(rejectionOf(without("DATABASE_URL")).split("\n")).toEqual([
      "Invalid environment:",
      "  DATABASE_URL: expected a PostgreSQL connection string (postgresql://…)",
    ]);
  });

  it("lists every failing variable in alphabetical order", () => {
    expect(
      rejectionOf(
        without("APP_ORIGIN", "DATABASE_URL", "ALLOWED_ORIGINS"),
      ).split("\n"),
    ).toEqual([
      "Invalid environment:",
      "  ALLOWED_ORIGINS: expected a comma-separated list of absolute URLs with no trailing slash (https://…)",
      "  APP_ORIGIN: expected an absolute URL with no trailing slash (https://…)",
      "  DATABASE_URL: expected a PostgreSQL connection string (postgresql://…)",
    ]);
  });

  it("never prints the value it received", () => {
    const secret = "b4dc0ffee-do-not-leak";

    const message = rejectionOf({
      ...VALID,
      DATABASE_URL: `mysql://root:${secret}@localhost:3306/db`,
    });

    expect(message).not.toContain(secret);
    expect(message).toContain(
      "  DATABASE_URL: expected a PostgreSQL connection string (postgresql://…)",
    );
  });

  it.each([
    "postgres://clinicore@localhost:5432/clinicore",
    "postgresql://user:pass@db.internal:5432/clinicore?schema=public",
  ])("accepts DATABASE_URL %p", (url) => {
    expect(validateEnv({ ...VALID, DATABASE_URL: url }).DATABASE_URL).toBe(url);
  });

  it.each([
    "mysql://localhost:5432/db",
    "localhost:5432",
    "",
    "postgresql",
    "postgresql://",
    "postgresql:///clinicore",
  ])("rejects DATABASE_URL %p", (url) => {
    expect(rejectionOf({ ...VALID, DATABASE_URL: url })).toContain(
      "  DATABASE_URL: expected a PostgreSQL connection string (postgresql://…)",
    );
  });

  it.each([
    "http://localhost:3000",
    "https://app.clinicore.com.br",
    "http://127.0.0.1:5173",
  ])("accepts APP_ORIGIN %p", (origin) => {
    expect(validateEnv({ ...VALID, APP_ORIGIN: origin }).APP_ORIGIN).toBe(
      origin,
    );
  });

  it.each([
    "http://localhost:3000/",
    "localhost:3000",
    "ftp://localhost:3000",
    "http://localhost:3000/api",
    "http://localhost:3000#",
    "http://localhost:3000/#",
    "http://localhost:3000?",
    "http://user:pass@localhost:3000",
    "HTTP://LOCALHOST:3000",
    "https://example.com:443",
    "",
  ])("rejects APP_ORIGIN %p", (origin) => {
    expect(rejectionOf({ ...VALID, APP_ORIGIN: origin })).toContain(
      "  APP_ORIGIN: expected an absolute URL with no trailing slash (https://…)",
    );
  });

  it.each([
    "http://localhost:3000,https://clinicore.com.br,https://app.clinicore.com.br",
    "https://app.clinicore.com.br",
  ])("accepts ALLOWED_ORIGINS %p", (origins) => {
    expect(
      validateEnv({ ...VALID, ALLOWED_ORIGINS: origins }).ALLOWED_ORIGINS,
    ).toEqual(origins.split(","));
  });

  it("accepts the spaces a human types around each comma", () => {
    expect(
      validateEnv({
        ...VALID,
        ALLOWED_ORIGINS: "http://localhost:3000, http://localhost:4000",
      }).ALLOWED_ORIGINS,
    ).toEqual(["http://localhost:3000", "http://localhost:4000"]);
  });

  it.each([
    "",
    "http://localhost:3000,",
    "http://localhost:3000,https://clinicore.com.br/",
    "http://localhost:3000;https://clinicore.com.br",
    ",",
  ])("rejects ALLOWED_ORIGINS %p", (origins) => {
    expect(rejectionOf({ ...VALID, ALLOWED_ORIGINS: origins })).toContain(
      "  ALLOWED_ORIGINS: expected a comma-separated list of absolute URLs with no trailing slash (https://…)",
    );
  });

  it.each([
    "0",
    "65536",
    "abc",
    "3333.5",
    "0x10",
    "1e3",
    " 3333 ",
    "+3333",
    "-3333",
    "",
  ])("rejects PORT %p", (port) => {
    expect(rejectionOf({ ...VALID, PORT: port })).toContain(
      "  PORT: expected an integer between 1 and 65535",
    );
  });
});
