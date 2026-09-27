import request from "supertest";
import { Session } from "../entities/session.entity";
import { User } from "../entities/user.entity";
import {
  cookieNamed,
  cookiesOf,
  createAuthApp,
  createVerifiedUser,
  type AuthApp,
} from "./auth-app";

const EMAIL = "ana@exemplo.com";
const PASSWORD = "Clinica#2026";
const CLIENT_HEADER = "Clinicore-Client";
const ACCESS_COOKIE = "clinicore_access";
const REFRESH_COOKIE = "clinicore_refresh";
const MOBILE_LIFETIME_IN_MILLISECONDS = 604800 * 1000;
const WEB_LIFETIME_IN_MILLISECONDS = 86400 * 1000;

interface Tokens {
  accessToken: string;
  refreshToken: string;
  accessTokenExpiresIn: number;
}

function tokensOf(response: request.Response): Tokens {
  const body = response.body as { tokens?: Tokens };

  if (body.tokens === undefined) {
    throw new Error("the response has no tokens");
  }

  return body.tokens;
}

function expectExpiryAhead(expiresAt: Date, lifetime: number): void {
  const toleranceInMilliseconds = 60 * 1000;

  expect(Math.abs(expiresAt.getTime() - (Date.now() + lifetime))).toBeLessThan(
    toleranceInMilliseconds,
  );
}

describe("session tokens for the mobile app", () => {
  let authApp: AuthApp;

  beforeAll(async () => {
    authApp = await createAuthApp();
  });

  beforeEach(async () => {
    await authApp.reset();
  });

  afterAll(async () => {
    await authApp.close();
  });

  function mobile(
    method: "get" | "post" | "delete",
    path: string,
  ): request.Test {
    return request(authApp.server)[method](path).set(CLIENT_HEADER, "mobile");
  }

  async function signInOnMobile(): Promise<Tokens> {
    await createVerifiedUser(authApp, EMAIL, PASSWORD);
    const response = await mobile("post", "/sessions")
      .send({ email: EMAIL, password: PASSWORD })
      .expect(201);

    return tokensOf(response);
  }

  function onlySession(): Promise<Session> {
    return authApp.dataSource
      .getRepository(Session)
      .findOneOrFail({ where: {} });
  }

  it("issues the tokens in the body and no cookie", async () => {
    const user = await createVerifiedUser(authApp, EMAIL, PASSWORD);

    const response = await mobile("post", "/sessions")
      .send({ email: EMAIL, password: PASSWORD })
      .expect(201);

    expect(response.body).toMatchObject({
      user: { id: user.id, email: EMAIL, emailVerified: true },
    });
    const tokens = tokensOf(response);
    expect(tokens.accessTokenExpiresIn).toBe(900);
    expect(tokens.refreshToken).toMatch(/^[0-9a-f-]{36}\.[A-Za-z0-9_-]{43}$/);
    expect(cookiesOf(response)).toEqual([]);

    const session = await onlySession();
    expect(session.client).toBe("mobile");
    expectExpiryAhead(session.expiresAt, MOBILE_LIFETIME_IN_MILLISECONDS);

    await mobile("get", "/sessions/current")
      .set("Authorization", `Bearer ${tokens.accessToken}`)
      .expect(200);
  });

  it("refuses an unknown client before creating anything", async () => {
    await createVerifiedUser(authApp, EMAIL, PASSWORD);

    const response = await request(authApp.server)
      .post("/sessions")
      .set(CLIENT_HEADER, "desktop")
      .send({ email: EMAIL, password: PASSWORD })
      .expect(400);

    expect(response.body).toEqual({
      type: "tag:clinicore.com.br,2026:invalid-client",
      title: "Invalid client",
      status: 400,
    });
    expect(await authApp.dataSource.getRepository(Session).count()).toBe(0);
  });

  it("refuses an unknown client on a route that never reads it", async () => {
    await request(authApp.server)
      .post("/users")
      .set(CLIENT_HEADER, "desktop")
      .send({ name: "Ana Souza", email: EMAIL, password: PASSWORD })
      .expect(400);

    expect(await authApp.dataSource.getRepository(User).count()).toBe(0);
  });

  it("refuses an unknown client on an authenticated route", async () => {
    const tokens = await signInOnMobile();

    const response = await request(authApp.server)
      .get("/sessions/current")
      .set(CLIENT_HEADER, "desktop")
      .set("Authorization", `Bearer ${tokens.accessToken}`)
      .expect(400);

    expect(response.body).toMatchObject({
      type: "tag:clinicore.com.br,2026:invalid-client",
    });
  });

  it("never authenticates the app by the cookie, nor the web by the header", async () => {
    const tokens = await signInOnMobile();

    const byCookie = await request(authApp.server)
      .get("/sessions/current")
      .set("Cookie", [`${ACCESS_COOKIE}=${tokens.accessToken}`])
      .expect(401);
    expect(byCookie.body).toMatchObject({
      type: "tag:clinicore.com.br,2026:invalid-session",
    });

    const byHeaderWithoutClient = await request(authApp.server)
      .get("/sessions/current")
      .set("Authorization", `Bearer ${tokens.accessToken}`)
      .expect(401);
    expect(byHeaderWithoutClient.body).toMatchObject({
      type: "tag:clinicore.com.br,2026:invalid-session",
    });

    const byCookieWithClient = await mobile("get", "/sessions/current")
      .set("Cookie", [`${ACCESS_COOKIE}=${tokens.accessToken}`])
      .expect(401);
    expect(byCookieWithClient.body).toMatchObject({
      type: "tag:clinicore.com.br,2026:invalid-session",
    });
  });

  it("never authenticates a web session by the bearer of the app", async () => {
    await createVerifiedUser(authApp, EMAIL, PASSWORD);
    const signIn = await authApp
      .post("/sessions")
      .send({ email: EMAIL, password: PASSWORD })
      .expect(201);
    const accessCookie = cookieNamed(signIn, ACCESS_COOKIE) ?? "";
    const accessToken = accessCookie.slice(
      accessCookie.indexOf("=") + 1,
      accessCookie.indexOf(";"),
    );

    const withClient = await mobile("get", "/sessions/current")
      .set("Authorization", `Bearer ${accessToken}`)
      .expect(401);
    expect(withClient.body).toMatchObject({
      type: "tag:clinicore.com.br,2026:invalid-session",
    });

    const withoutClient = await request(authApp.server)
      .get("/sessions/current")
      .set("Authorization", `Bearer ${accessToken}`)
      .expect(401);
    expect(withoutClient.body).toMatchObject({
      type: "tag:clinicore.com.br,2026:invalid-session",
    });
  });

  it("rotates the refresh token from the body and pushes seven days", async () => {
    const tokens = await signInOnMobile();
    const before = await onlySession();

    const response = await mobile("post", "/sessions/current/tokens")
      .send({ refreshToken: tokens.refreshToken })
      .expect(200);

    const rotated = tokensOf(response);
    expect(rotated.refreshToken).not.toEqual(tokens.refreshToken);
    expect(rotated.accessTokenExpiresIn).toBe(900);
    expect(cookiesOf(response)).toEqual([]);

    const after = await onlySession();
    expect(after.refreshTokenHash).not.toEqual(before.refreshTokenHash);
    expectExpiryAhead(after.expiresAt, MOBILE_LIFETIME_IN_MILLISECONDS);

    await mobile("get", "/sessions/current")
      .set("Authorization", `Bearer ${rotated.accessToken}`)
      .expect(200);

    const reused = await mobile("post", "/sessions/current/tokens")
      .send({ refreshToken: tokens.refreshToken })
      .expect(401);
    expect(reused.body).toMatchObject({
      type: "tag:clinicore.com.br,2026:session-reused",
    });
    expect(await authApp.dataSource.getRepository(Session).count()).toBe(0);

    await mobile("get", "/sessions/current")
      .set("Authorization", `Bearer ${rotated.accessToken}`)
      .expect(401);
  });

  it("keeps the lifetime of the stored client, whatever the refresh transport", async () => {
    await createVerifiedUser(authApp, EMAIL, PASSWORD);
    const signIn = await authApp
      .post("/sessions")
      .send({ email: EMAIL, password: PASSWORD })
      .expect(201);
    const refreshCookie = cookieNamed(signIn, REFRESH_COOKIE) ?? "";
    const refreshToken = refreshCookie.slice(
      refreshCookie.indexOf("=") + 1,
      refreshCookie.indexOf(";"),
    );

    await mobile("post", "/sessions/current/tokens")
      .send({ refreshToken })
      .expect(200);

    const session = await onlySession();
    expect(session.client).toBe("web");
    expectExpiryAhead(session.expiresAt, WEB_LIFETIME_IN_MILLISECONDS);
  });

  it("requires a well-formed refresh token in the body of the app", async () => {
    await signInOnMobile();

    const missing = await mobile("post", "/sessions/current/tokens")
      .send({})
      .expect(400);
    expect(missing.body).toMatchObject({
      type: "tag:clinicore.com.br,2026:validation-failed",
      errors: [{ pointer: "#/refreshToken", code: "IS_DEFINED" }],
    });

    const malformed = await mobile("post", "/sessions/current/tokens")
      .send({ refreshToken: "abc.xyz" })
      .expect(400);
    expect(malformed.body).toMatchObject({
      type: "tag:clinicore.com.br,2026:validation-failed",
      errors: [{ pointer: "#/refreshToken", code: "MATCHES" }],
    });
  });

  it("signs the app out at once, without touching cookies", async () => {
    const tokens = await signInOnMobile();

    const response = await mobile("delete", "/sessions/current")
      .set("Authorization", `Bearer ${tokens.accessToken}`)
      .expect(204);

    expect(cookiesOf(response)).toEqual([]);
    expect(await authApp.dataSource.getRepository(Session).count()).toBe(0);

    await mobile("get", "/sessions/current")
      .set("Authorization", `Bearer ${tokens.accessToken}`)
      .expect(401);
  });
});
