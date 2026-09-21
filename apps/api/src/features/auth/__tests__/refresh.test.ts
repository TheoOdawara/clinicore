import { Session } from "../entities/session.entity";
import {
  cookieNamed,
  createAuthApp,
  createVerifiedUser,
  REVOKED_KEY_PREFIX,
  type AuthApp,
} from "./auth-app";

const EMAIL = "ana@exemplo.com";
const PASSWORD = "Clinica#2026";
const ACCESS_COOKIE = "clinicore_access";
const REFRESH_COOKIE = "clinicore_refresh";
const REVOCATION_TTL_IN_SECONDS = 900;

describe("POST /sessions/current/tokens", () => {
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

  async function signIn(): Promise<{
    access: string;
    refresh: string;
    sessionId: string;
  }> {
    await createVerifiedUser(authApp, EMAIL, PASSWORD);
    const response = await authApp
      .post("/sessions")
      .send({ email: EMAIL, password: PASSWORD })
      .expect(201);

    const session = await authApp.dataSource
      .getRepository(Session)
      .findOneOrFail({ where: {} });

    return {
      access: cookieNamed(response, ACCESS_COOKIE) ?? "",
      refresh: cookieNamed(response, REFRESH_COOKIE) ?? "",
      sessionId: session.id,
    };
  }

  it("rotates the refresh token and issues new cookies", async () => {
    const { refresh, sessionId } = await signIn();
    const sessions = authApp.dataSource.getRepository(Session);
    const before = await sessions.findOneOrFail({ where: { id: sessionId } });

    const response = await authApp
      .post("/sessions/current/tokens")
      .set("Cookie", [refresh])
      .expect(204);

    const rotatedRefresh = cookieNamed(response, REFRESH_COOKIE) ?? "";
    const rotatedAccess = cookieNamed(response, ACCESS_COOKIE) ?? "";
    expect(rotatedRefresh).not.toEqual(refresh);

    const after = await sessions.findOneOrFail({ where: { id: sessionId } });
    expect(after.refreshTokenHash).not.toEqual(before.refreshTokenHash);
    expect(after.expiresAt.getTime()).toBeGreaterThan(
      before.expiresAt.getTime(),
    );

    await authApp
      .get("/sessions/current")
      .set("Cookie", [rotatedAccess])
      .expect(200);
  });

  it("drops the whole session when the old refresh token comes back", async () => {
    const { refresh, sessionId } = await signIn();
    const rotation = await authApp
      .post("/sessions/current/tokens")
      .set("Cookie", [refresh])
      .expect(204);

    const response = await authApp
      .post("/sessions/current/tokens")
      .set("Cookie", [refresh])
      .expect(401);

    expect(response.body).toEqual({
      type: "tag:clinicore.com.br,2026:session-reused",
      title: "Refresh token reuse detected",
      status: 401,
    });

    const session = await authApp.dataSource
      .getRepository(Session)
      .findOne({ where: { id: sessionId } });
    expect(session).toBeNull();

    const ttl = await authApp.redis.ttl(`${REVOKED_KEY_PREFIX}${sessionId}`);
    expect(ttl).toBeGreaterThan(0);
    expect(ttl).toBeLessThanOrEqual(REVOCATION_TTL_IN_SECONDS);

    const rotatedAccess = cookieNamed(rotation, ACCESS_COOKIE) ?? "";
    const invalidSession = await authApp
      .get("/sessions/current")
      .set("Cookie", [rotatedAccess])
      .expect(401);
    expect(invalidSession.body).toMatchObject({
      type: "tag:clinicore.com.br,2026:invalid-session",
    });

    const rotatedRefresh = cookieNamed(rotation, REFRESH_COOKIE) ?? "";
    const refusedRefresh = await authApp
      .post("/sessions/current/tokens")
      .set("Cookie", [rotatedRefresh])
      .expect(401);
    expect(refusedRefresh.body).toMatchObject({
      type: "tag:clinicore.com.br,2026:invalid-session",
    });
  });

  it("refuses a refresh token of a session that does not exist", async () => {
    const { refresh, sessionId } = await signIn();
    await authApp.dataSource.getRepository(Session).delete({ id: sessionId });

    const response = await authApp
      .post("/sessions/current/tokens")
      .set("Cookie", [refresh])
      .expect(401);

    expect(response.body).toMatchObject({
      type: "tag:clinicore.com.br,2026:invalid-session",
    });
  });

  it("refuses a malformed refresh token", async () => {
    await signIn();

    const response = await authApp
      .post("/sessions/current/tokens")
      .set("Cookie", [`${REFRESH_COOKIE}=not-a-token`])
      .expect(401);

    expect(response.body).toMatchObject({
      type: "tag:clinicore.com.br,2026:invalid-session",
    });
  });
});
