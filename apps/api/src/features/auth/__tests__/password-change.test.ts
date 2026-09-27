import request from "supertest";
import { Account } from "../entities/account.entity";
import { Session } from "../entities/session.entity";
import {
  Provider,
  REVOKED_KEY_PREFIX,
  cookieNamed,
  createAuthApp,
  createVerifiedUser,
  openConnections,
  type AuthApp,
} from "./auth-app";

const EMAIL = "ana@exemplo.com";
const PASSWORD = "Clinica#2026";
const NEW_PASSWORD = "Outra#Senha9";
const ACCESS_COOKIE = "clinicore_access";
const PATH = "/users/me/password";

const INVALID_PASSWORD = {
  type: "tag:clinicore.com.br,2026:invalid-password",
  title: "Invalid password",
  status: 400,
};

describe("PUT /users/me/password", () => {
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

  async function signIn(password: string): Promise<string> {
    const response = await authApp
      .post("/sessions")
      .send({ email: EMAIL, password })
      .expect(201);

    const access = cookieNamed(response, ACCESS_COOKIE);
    if (access === undefined) {
      throw new Error("the sign-in set no access cookie");
    }

    return access;
  }

  function changePassword(
    access: string,
    currentPassword: string,
    newPassword: string,
  ): request.Test {
    return authApp
      .put(PATH)
      .set("Cookie", [access])
      .send({ currentPassword, newPassword });
  }

  function sessionOf(access: string): request.Test {
    return authApp.get("/sessions/current").set("Cookie", [access]);
  }

  async function storedHash(userId: string): Promise<string | null> {
    const account = await authApp.dataSource
      .getRepository(Account)
      .findOneOrFail({ where: { userId, provider: Provider.Credential } });

    return account.passwordHash;
  }

  it("keeps the current session and drops the others at once", async () => {
    await createVerifiedUser(authApp, EMAIL, PASSWORD);
    const first = await signIn(PASSWORD);
    const second = await signIn(PASSWORD);
    const sessions = authApp.dataSource.getRepository(Session);
    const before = await sessions.find();
    expect(before).toHaveLength(2);

    const response = await changePassword(first, PASSWORD, NEW_PASSWORD).expect(
      204,
    );

    expect(cookieNamed(response, ACCESS_COOKIE)).toBeUndefined();
    const remaining = await sessions.find();
    expect(remaining).toHaveLength(1);
    const dropped = before.find((session) => session.id !== remaining[0]?.id);
    expect(
      await authApp.redis.exists(`${REVOKED_KEY_PREFIX}${dropped?.id ?? ""}`),
    ).toBe(1);
    await sessionOf(first).expect(200);
    const refused = await sessionOf(second).expect(401);
    expect(refused.body).toMatchObject({
      type: "tag:clinicore.com.br,2026:invalid-session",
    });

    const repeated = await changePassword(first, PASSWORD, NEW_PASSWORD).expect(
      400,
    );

    expect(repeated.body).toEqual(INVALID_PASSWORD);
    await authApp
      .post("/sessions")
      .send({ email: EMAIL, password: NEW_PASSWORD })
      .expect(201);
    await authApp
      .post("/sessions")
      .send({ email: EMAIL, password: PASSWORD })
      .expect(401);
  });

  it("refuses a wrong current password and changes nothing", async () => {
    const user = await createVerifiedUser(authApp, EMAIL, PASSWORD);
    const first = await signIn(PASSWORD);
    const second = await signIn(PASSWORD);
    const hashBefore = await storedHash(user.id);

    const response = await changePassword(
      first,
      "Errada#Senha1",
      NEW_PASSWORD,
    ).expect(400);

    expect(response.body).toEqual(INVALID_PASSWORD);
    expect(await storedHash(user.id)).toBe(hashBefore);
    expect(await authApp.dataSource.getRepository(Session).count()).toBe(2);
    await sessionOf(second).expect(200);
  });

  it("refuses a weak newPassword and changes nothing", async () => {
    const user = await createVerifiedUser(authApp, EMAIL, PASSWORD);
    const access = await signIn(PASSWORD);
    const hashBefore = await storedHash(user.id);

    const response = await changePassword(access, PASSWORD, "fraca").expect(
      400,
    );

    expect(response.body).toEqual({
      type: "tag:clinicore.com.br,2026:validation-failed",
      title: "Validation failed",
      status: 400,
      errors: [{ pointer: "#/newPassword", code: "WEAK_PASSWORD" }],
    });
    expect(await storedHash(user.id)).toBe(hashBefore);
  });

  it("refuses a Google-only user without creating a credential account", async () => {
    const user = await createVerifiedUser(authApp, EMAIL, PASSWORD);
    const access = await signIn(PASSWORD);
    const accounts = authApp.dataSource.getRepository(Account);
    await accounts.delete({ userId: user.id, provider: Provider.Credential });
    await accounts.save({
      userId: user.id,
      provider: Provider.Google,
      providerAccountId: "google-subject-1",
    });

    const response = await changePassword(
      access,
      PASSWORD,
      NEW_PASSWORD,
    ).expect(400);

    expect(response.body).toEqual(INVALID_PASSWORD);
    const providers = await accounts.find({ where: { userId: user.id } });
    expect(providers.map((account) => account.provider)).toEqual([
      Provider.Google,
    ]);
  });

  it("lets only one of two simultaneous changes through", async () => {
    await createVerifiedUser(authApp, EMAIL, PASSWORD);
    const access = await signIn(PASSWORD);
    await openConnections(authApp, 2);

    const responses = await Promise.all([
      changePassword(access, PASSWORD, NEW_PASSWORD),
      changePassword(access, PASSWORD, "Terceira#Senha1"),
    ]);

    expect(responses.map((response) => response.status).sort()).toEqual([
      204, 400,
    ]);
  });

  it("keeps the mobile session that made the change", async () => {
    await createVerifiedUser(authApp, EMAIL, PASSWORD);
    const web = await signIn(PASSWORD);
    const signedIn = await request(authApp.server)
      .post("/sessions")
      .set("Clinicore-Client", "mobile")
      .send({ email: EMAIL, password: PASSWORD })
      .expect(201);
    const { accessToken } = (
      signedIn.body as { tokens: { accessToken: string } }
    ).tokens;
    const bearer = `Bearer ${accessToken}`;

    await request(authApp.server)
      .put(PATH)
      .set("Clinicore-Client", "mobile")
      .set("Authorization", bearer)
      .send({ currentPassword: PASSWORD, newPassword: NEW_PASSWORD })
      .expect(204);

    await request(authApp.server)
      .get("/sessions/current")
      .set("Clinicore-Client", "mobile")
      .set("Authorization", bearer)
      .expect(200);
    await sessionOf(web).expect(401);
  });
});
