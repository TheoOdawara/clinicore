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
const SESSION_CAP = 5;

describe("the cap of active sessions", () => {
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

  async function signIn(): Promise<string> {
    const response = await authApp
      .post("/sessions")
      .send({ email: EMAIL, password: PASSWORD })
      .expect(201);
    return cookieNamed(response, ACCESS_COOKIE) ?? "";
  }

  it("drops the oldest session on the sixth sign-in", async () => {
    await createVerifiedUser(authApp, EMAIL, PASSWORD);
    const sessions = authApp.dataSource.getRepository(Session);

    const oldestAccess = await signIn();
    const oldest = await sessions.findOneOrFail({
      where: {},
      order: { createdAt: "ASC" },
    });

    for (let attempt = 0; attempt < SESSION_CAP - 1; attempt += 1) {
      await signIn();
    }
    expect(await sessions.count()).toBe(SESSION_CAP);

    await signIn();

    expect(await sessions.count()).toBe(SESSION_CAP);
    expect(await sessions.findOne({ where: { id: oldest.id } })).toBeNull();
    expect(
      await authApp.redis.exists(`${REVOKED_KEY_PREFIX}${oldest.id}`),
    ).toBe(1);

    const me = await authApp
      .get("/sessions/current")
      .set("Cookie", [oldestAccess])
      .expect(401);
    expect(me.body).toMatchObject({
      type: "tag:clinicore.com.br,2026:invalid-session",
    });
  });

  it("opens no session when the eviction cannot be revoked", async () => {
    await createVerifiedUser(authApp, EMAIL, PASSWORD);
    const sessions = authApp.dataSource.getRepository(Session);
    for (let attempt = 0; attempt < SESSION_CAP; attempt += 1) {
      await signIn();
    }
    const before = await sessions.find({ select: { id: true } });
    jest
      .spyOn(authApp.redis, "set")
      .mockRejectedValueOnce(new Error("OOM command not allowed"));

    const response = await authApp
      .post("/sessions")
      .send({ email: EMAIL, password: PASSWORD })
      .expect(503);

    expect(response.body).toMatchObject({
      type: "tag:clinicore.com.br,2026:service-unavailable",
    });
    const after = await sessions.find({ select: { id: true } });
    expect(after.map((session) => session.id).sort()).toEqual(
      before.map((session) => session.id).sort(),
    );
  });

  it("keeps the cap when two sign-ins race", async () => {
    await createVerifiedUser(authApp, EMAIL, PASSWORD);

    for (let attempt = 0; attempt < SESSION_CAP; attempt += 1) {
      await signIn();
    }
    await Promise.all([signIn(), signIn()]);

    expect(await authApp.dataSource.getRepository(Session).count()).toBe(
      SESSION_CAP,
    );
  });
});
