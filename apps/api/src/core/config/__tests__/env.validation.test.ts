import { validateEnv } from "../env.validation";

const VALID = {
  ALLOWED_ORIGINS: "http://localhost:3000",
  API_URL: "http://localhost:3333",
  APP_ORIGIN: "http://localhost:3000",
  DATABASE_URL: "postgresql://clinicore:local@localhost:5432/clinicore",
  GOOGLE_CLIENT_ID: "client-id.apps.googleusercontent.com",
  GOOGLE_CLIENT_SECRET: "google-client-secret",
  JWT_SECRET: "a-jwt-secret-with-at-least-32-characters",
  LOG_LEVEL: "silent",
  MAIL_FROM: "clinicore@gmail.com",
  NODE_ENV: "test",
  PORT: "3333",
  REDIS_URL: "redis://localhost:6379",
  SMTP_HOST: "smtp.gmail.com",
  SMTP_PASSWORD: "smtp-password",
  SMTP_PORT: "587",
  SMTP_USER: "clinicore@gmail.com",
  TRUSTED_PROXIES: "10.0.0.0/8",
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
        without("APP_ORIGIN", "DATABASE_URL", "ALLOWED_ORIGINS", "NODE_ENV"),
      ).split("\n"),
    ).toEqual([
      "Invalid environment:",
      "  ALLOWED_ORIGINS: expected a comma-separated list of absolute URLs with no trailing slash (https://…)",
      "  APP_ORIGIN: expected an absolute URL with no trailing slash (https://…)",
      "  DATABASE_URL: expected a PostgreSQL connection string (postgresql://…)",
      "  NODE_ENV: expected one of development, production, test",
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

  it.each(["development", "production", "test"])(
    "accepts NODE_ENV %p",
    (nodeEnvironment) => {
      expect(
        validateEnv({ ...VALID, NODE_ENV: nodeEnvironment }).NODE_ENV,
      ).toBe(nodeEnvironment);
    },
  );

  it.each(["", "prod", "Production", "staging", "development,test"])(
    "rejects NODE_ENV %p",
    (nodeEnvironment) => {
      expect(rejectionOf({ ...VALID, NODE_ENV: nodeEnvironment })).toContain(
        "  NODE_ENV: expected one of development, production, test",
      );
    },
  );

  it("rejects a missing NODE_ENV instead of assuming one", () => {
    expect(rejectionOf(without("NODE_ENV")).split("\n")).toEqual([
      "Invalid environment:",
      "  NODE_ENV: expected one of development, production, test",
    ]);
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
  it("rejects a missing JWT_SECRET and TRUSTED_PROXIES naming both", () => {
    expect(
      rejectionOf(without("JWT_SECRET", "TRUSTED_PROXIES")).split("\n"),
    ).toEqual([
      "Invalid environment:",
      "  JWT_SECRET: expected a string with at least 32 characters",
      "  TRUSTED_PROXIES: expected a comma-separated list of CIDR blocks (10.0.0.0/8,…)",
    ]);
  });

  it("never prints a secret that was valid while another variable failed", () => {
    const secret = "b4dc0ffee-do-not-leak";

    const message = rejectionOf({
      ...without("NODE_ENV"),
      GOOGLE_CLIENT_SECRET: secret,
      JWT_SECRET: `${secret}-${secret}`,
      SMTP_PASSWORD: secret,
    });

    expect(message).not.toContain(secret);
  });

  it("never prints a JWT_SECRET that was itself rejected", () => {
    const secret = "b4dc0ffee-do-not-leak";

    expect(rejectionOf({ ...VALID, JWT_SECRET: secret })).not.toContain(secret);
  });

  it("returns SMTP_PORT as a number and TRUSTED_PROXIES as a list", () => {
    const environment = validateEnv({
      ...VALID,
      TRUSTED_PROXIES: "10.0.0.0/8, 172.16.0.0/12",
    });

    expect(environment.SMTP_PORT).toBe(587);
    expect(environment.TRUSTED_PROXIES).toEqual([
      "10.0.0.0/8",
      "172.16.0.0/12",
    ]);
  });

  it.each(["fatal", "error", "warn", "info", "debug", "trace", "silent"])(
    "accepts LOG_LEVEL %p",
    (level) => {
      expect(validateEnv({ ...VALID, LOG_LEVEL: level }).LOG_LEVEL).toBe(level);
    },
  );

  it.each(["", "verbose", "INFO", "warning", "info,debug"])(
    "rejects LOG_LEVEL %p",
    (level) => {
      expect(rejectionOf({ ...VALID, LOG_LEVEL: level })).toContain(
        "  LOG_LEVEL: expected one of: fatal, error, warn, info, debug, trace, silent",
      );
    },
  );

  it.each(["10.0.0.0/8", "10.0.0.0/8,172.16.0.0/12", "fd00::/8"])(
    "accepts TRUSTED_PROXIES %p",
    (proxies) => {
      expect(
        validateEnv({ ...VALID, TRUSTED_PROXIES: proxies }).TRUSTED_PROXIES,
      ).toEqual(proxies.split(",").map((block) => block.trim()));
    },
  );

  it.each([
    "",
    "10.0.0.0",
    "10.0.0.0/8,",
    "10.0.0.0/33",
    "loopback",
    "10.0.0.0/8;172.16.0.0/12",
  ])("rejects TRUSTED_PROXIES %p", (proxies) => {
    expect(rejectionOf({ ...VALID, TRUSTED_PROXIES: proxies })).toContain(
      "  TRUSTED_PROXIES: expected a comma-separated list of CIDR blocks (10.0.0.0/8,…)",
    );
  });

  it("accepts MAIL_FROM when it is the SMTP_USER", () => {
    const environment = validateEnv({
      ...VALID,
      MAIL_FROM: "contato@clinicore.com.br",
      SMTP_USER: "contato@clinicore.com.br",
    });

    expect(environment.MAIL_FROM).toBe("contato@clinicore.com.br");
  });

  it.each(["outro@clinicore.com.br", "", "clinicore@gmail.com.br"])(
    "rejects MAIL_FROM %p when SMTP_USER is another address",
    (from) => {
      expect(rejectionOf({ ...VALID, MAIL_FROM: from })).toContain(
        "  MAIL_FROM: expected an email address equal to SMTP_USER",
      );
    },
  );

  it.each(["", "clinicore", "clinicore@", "@gmail.com"])(
    "rejects SMTP_USER %p",
    (user) => {
      expect(rejectionOf({ ...VALID, SMTP_USER: user })).toContain(
        "  SMTP_USER: expected an email address",
      );
    },
  );

  it.each(["", "smtp", "smtp.gmail.com/", "http://smtp.gmail.com"])(
    "rejects SMTP_HOST %p",
    (host) => {
      expect(rejectionOf({ ...VALID, SMTP_HOST: host })).toContain(
        "  SMTP_HOST: expected a hostname",
      );
    },
  );

  it.each(["0", "65536", "587.5", "", "abc"])(
    "rejects SMTP_PORT %p",
    (port) => {
      expect(rejectionOf({ ...VALID, SMTP_PORT: port })).toContain(
        "  SMTP_PORT: expected an integer between 1 and 65535",
      );
    },
  );

  it.each([
    "redis://localhost:6379",
    "redis://cache.internal:6379",
    "redis://user:pass@cache.internal:6379/0",
  ])("accepts REDIS_URL %p", (url) => {
    expect(validateEnv({ ...VALID, REDIS_URL: url }).REDIS_URL).toBe(url);
  });

  it.each(["", "localhost:6379", "rediss://localhost:6379", "redis://"])(
    "rejects REDIS_URL %p",
    (url) => {
      expect(rejectionOf({ ...VALID, REDIS_URL: url })).toContain(
        "  REDIS_URL: expected a Redis connection string (redis://…)",
      );
    },
  );

  it.each(["http://localhost:3333", "https://api.clinicore.com.br"])(
    "accepts API_URL %p",
    (url) => {
      expect(validateEnv({ ...VALID, API_URL: url }).API_URL).toBe(url);
    },
  );

  it.each(["", "api.clinicore.com.br", "https://api.clinicore.com.br/"])(
    "rejects API_URL %p",
    (url) => {
      expect(rejectionOf({ ...VALID, API_URL: url })).toContain(
        "  API_URL: expected an absolute URL with no trailing slash (https://…)",
      );
    },
  );

  it.each(["GOOGLE_CLIENT_ID", "GOOGLE_CLIENT_SECRET", "SMTP_PASSWORD"])(
    "rejects an empty %s",
    (name) => {
      expect(rejectionOf({ ...VALID, [name]: "" })).toContain(
        `  ${name}: expected a non-empty string`,
      );
    },
  );

  it("rejects a JWT_SECRET shorter than 32 characters", () => {
    expect(rejectionOf({ ...VALID, JWT_SECRET: "a".repeat(31) })).toContain(
      "  JWT_SECRET: expected a string with at least 32 characters",
    );
  });
});
