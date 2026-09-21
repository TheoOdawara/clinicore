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

describe("DELETE /sessions/current", () => {
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

  it("drops the session even while the access token is still valid", async () => {
    await createVerifiedUser(authApp, EMAIL, PASSWORD);
    const signIn = await authApp
      .post("/sessions")
      .send({ email: EMAIL, password: PASSWORD })
      .expect(201);
    const access = cookieNamed(signIn, ACCESS_COOKIE) ?? "";
    const session = await authApp.dataSource
      .getRepository(Session)
      .findOneOrFail({ where: {} });

    const response = await authApp
      .delete("/sessions/current")
      .set("Cookie", [access])
      .expect(204);

    expect(cookieNamed(response, ACCESS_COOKIE)).toContain("Max-Age=0");
    expect(cookieNamed(response, REFRESH_COOKIE)).toContain("Max-Age=0");
    expect(cookieNamed(response, REFRESH_COOKIE)).toContain(
      "Path=/sessions/current/tokens",
    );

    expect(
      await authApp.dataSource
        .getRepository(Session)
        .findOne({ where: { id: session.id } }),
    ).toBeNull();
    expect(
      await authApp.redis.exists(`${REVOKED_KEY_PREFIX}${session.id}`),
    ).toBe(1);

    const me = await authApp
      .get("/sessions/current")
      .set("Cookie", [access])
      .expect(401);
    expect(me.body).toMatchObject({
      type: "tag:clinicore.com.br,2026:invalid-session",
    });
  });

  it("refuses the repeated sign-out and changes nothing", async () => {
    await createVerifiedUser(authApp, EMAIL, PASSWORD);
    const signIn = await authApp
      .post("/sessions")
      .send({ email: EMAIL, password: PASSWORD })
      .expect(201);
    const access = cookieNamed(signIn, ACCESS_COOKIE) ?? "";

    await authApp
      .delete("/sessions/current")
      .set("Cookie", [access])
      .expect(204);
    const response = await authApp
      .delete("/sessions/current")
      .set("Cookie", [access])
      .expect(401);

    expect(response.body).toMatchObject({
      type: "tag:clinicore.com.br,2026:invalid-session",
    });
    expect(await authApp.dataSource.getRepository(Session).count()).toBe(0);
  });

  it("refuses a request without the access cookie", async () => {
    const response = await authApp.delete("/sessions/current").expect(401);

    expect(response.body).toMatchObject({
      type: "tag:clinicore.com.br,2026:invalid-session",
    });
  });
});
