import { createLogger } from "../logger";

const COOKIE = "session=do-not-leak-cookie";
const AUTHORIZATION = "Bearer do-not-leak-token";
const SET_COOKIE = "session=do-not-leak-set-cookie";
const PASSWORD = "do-not-leak-password";
const REFRESH_TOKEN = "do-not-leak-refresh-token";
const ID_TOKEN = "do-not-leak-id-token";
const WEBAUTHN_RESPONSE = "do-not-leak-webauthn-response";

function capture(
  emit: (logger: ReturnType<typeof createLogger>) => void,
  level: "info" | "silent" = "info",
): string {
  const written: string[] = [];
  const logger = createLogger(level, {
    write(line: string): void {
      written.push(line);
    },
  });

  emit(logger);

  return written.join("");
}

describe("createLogger", () => {
  it("redacts every secret the request and the response carry", () => {
    const written = capture((logger) => {
      logger.info(
        {
          req: {
            method: "POST",
            url: "/sessions?token=do-not-leak-query",
            headers: { cookie: COOKIE, authorization: AUTHORIZATION },
            body: {
              password: PASSWORD,
              newPassword: PASSWORD,
              currentPassword: PASSWORD,
              token: PASSWORD,
              refreshToken: REFRESH_TOKEN,
              idToken: ID_TOKEN,
              response: { signature: WEBAUTHN_RESPONSE },
            },
          },
          res: { statusCode: 200, headers: { "set-cookie": SET_COOKIE } },
        },
        "request",
      );
    });

    expect(written).not.toContain(COOKIE);
    expect(written).not.toContain(AUTHORIZATION);
    expect(written).not.toContain(SET_COOKIE);
    expect(written).not.toContain(PASSWORD);
    expect(written).not.toContain(REFRESH_TOKEN);
    expect(written).not.toContain(ID_TOKEN);
    expect(written).not.toContain(WEBAUTHN_RESPONSE);
  });

  it("keeps the method, the path and the status", () => {
    const written = capture((logger) => {
      logger.info(
        {
          req: { method: "POST", url: "/sessions?token=secret" },
          res: { statusCode: 200 },
        },
        "request",
      );
    });

    const line = JSON.parse(written) as {
      req: { method: string; path: string };
      res: { statusCode: number };
    };

    expect(line.req).toEqual({ method: "POST", path: "/sessions" });
    expect(line.res).toEqual({ statusCode: 200 });
  });

  it("writes nothing when the level is silent", () => {
    const written = capture((logger) => {
      logger.info({ path: "/probe" }, "request");
    }, "silent");

    expect(written).toBe("");
  });
});
