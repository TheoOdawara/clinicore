import { Account } from "../entities/account.entity";
import { Session } from "../entities/session.entity";
import { User } from "../entities/user.entity";
import { Verification } from "../entities/verification.entity";
import { VerificationPurpose } from "../enums/verification-purpose.enum";
import {
  Provider,
  REVOKED_KEY_PREFIX,
  cookieNamed,
  createAuthApp,
  createVerifiedUser,
  openConnections,
  tokenFrom,
  type AuthApp,
} from "./auth-app";

const EMAIL = "ana@exemplo.com";
const PASSWORD = "Clinica#2026";
const NEW_PASSWORD = "Outra#Senha9";
const ACCESS_COOKIE = "clinicore_access";
const DENYLIST_TTL_IN_SECONDS = 900;

const INVALID_TOKEN = {
  type: "tag:clinicore.com.br,2026:invalid-token",
  title: "Invalid token",
  status: 400,
};

describe("POST /password-resets", () => {
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

  async function requestReset(email: string): Promise<string> {
    const response = await authApp
      .post("/password-resets")
      .send({ email })
      .expect(202);

    expect(response.body).toEqual({});

    return tokenFrom(authApp.outbox.at(-1), "/reset-password");
  }

  function confirm(token: string, newPassword: string) {
    return authApp
      .post("/password-resets/confirmation")
      .send({ token, newPassword });
  }

  async function signIn(password: string): Promise<string> {
    const response = await authApp
      .post("/sessions")
      .send({ email: EMAIL, password })
      .expect(201);

    return cookieNamed(response, ACCESS_COOKIE) ?? "";
  }

  it("mails the reset link only to an existing address", async () => {
    await createVerifiedUser(authApp, EMAIL, PASSWORD);
    authApp.outbox.length = 0;

    const existing = await authApp
      .post("/password-resets")
      .send({ email: EMAIL })
      .expect(202);
    const unknown = await authApp
      .post("/password-resets")
      .send({ email: "ninguem@exemplo.com" })
      .expect(202);

    expect(unknown.body).toEqual(existing.body);
    expect(unknown.body).toEqual({});
    expect(authApp.outbox).toHaveLength(1);
    const [mail] = authApp.outbox;
    expect(mail?.to).toBe(EMAIL);
    expect(mail?.subject).toBe("Redefinir sua senha do Clinicore");
    expect(mail?.text).toContain(`${authApp.appOrigin}/reset-password?token=`);
    expect(
      await authApp.dataSource
        .getRepository(Verification)
        .count({ where: { purpose: VerificationPurpose.PasswordReset } }),
    ).toBe(1);
  });

  describe("confirmation", () => {
    it("drops every session at once and revokes each one", async () => {
      await createVerifiedUser(authApp, EMAIL, PASSWORD);
      const accessCookies = [await signIn(PASSWORD), await signIn(PASSWORD)];
      const sessions = await authApp.dataSource.getRepository(Session).find();
      expect(sessions).toHaveLength(2);
      const token = await requestReset(EMAIL);

      await confirm(token, NEW_PASSWORD).expect(204);

      expect(await authApp.dataSource.getRepository(Session).count()).toBe(0);
      for (const session of sessions) {
        const ttl = await authApp.redis.ttl(
          `${REVOKED_KEY_PREFIX}${session.id}`,
        );
        expect(ttl).toBeGreaterThan(0);
        expect(ttl).toBeLessThanOrEqual(DENYLIST_TTL_IN_SECONDS);
      }
      for (const access of accessCookies) {
        const response = await authApp
          .get("/sessions/current")
          .set("Cookie", [access])
          .expect(401);
        expect(response.body).toMatchObject({
          type: "tag:clinicore.com.br,2026:invalid-session",
        });
      }
    });

    it("signs in with the new password and refuses the old one", async () => {
      await createVerifiedUser(authApp, EMAIL, PASSWORD);
      const token = await requestReset(EMAIL);

      await confirm(token, NEW_PASSWORD).expect(204);

      await authApp
        .post("/sessions")
        .send({ email: EMAIL, password: NEW_PASSWORD })
        .expect(201);
      await authApp
        .post("/sessions")
        .send({ email: EMAIL, password: PASSWORD })
        .expect(401);
    });

    it("refuses the same token a second time and keeps the first password", async () => {
      await createVerifiedUser(authApp, EMAIL, PASSWORD);
      const token = await requestReset(EMAIL);
      await confirm(token, NEW_PASSWORD).expect(204);

      const response = await confirm(token, "Terceira#Senha1").expect(400);

      expect(response.body).toEqual(INVALID_TOKEN);
      await authApp
        .post("/sessions")
        .send({ email: EMAIL, password: NEW_PASSWORD })
        .expect(201);
    });

    it("lets only one of two simultaneous confirmations through", async () => {
      await createVerifiedUser(authApp, EMAIL, PASSWORD);
      const token = await requestReset(EMAIL);
      await openConnections(authApp, 2);

      const responses = await Promise.all([
        confirm(token, NEW_PASSWORD),
        confirm(token, "Terceira#Senha1"),
      ]);

      expect(responses.map((response) => response.status).sort()).toEqual([
        204, 400,
      ]);
    });

    it("refuses an expired reset token with INVALID_TOKEN", async () => {
      await createVerifiedUser(authApp, EMAIL, PASSWORD);
      const token = await requestReset(EMAIL);
      await authApp.dataSource
        .getRepository(Verification)
        .update(
          { purpose: VerificationPurpose.PasswordReset },
          { expiresAt: new Date(Date.now() - 60_000) },
        );

      const response = await confirm(token, NEW_PASSWORD).expect(400);

      expect(response.body).toEqual(INVALID_TOKEN);
      await authApp
        .post("/sessions")
        .send({ email: EMAIL, password: PASSWORD })
        .expect(201);
    });

    it("refuses a verification token", async () => {
      await authApp
        .post("/users")
        .send({ name: "Ana Souza", email: EMAIL, password: PASSWORD })
        .expect(202);
      const verificationToken = tokenFrom(authApp.outbox[0], "/verify-email");

      const response = await confirm(verificationToken, NEW_PASSWORD).expect(
        400,
      );

      expect(response.body).toEqual(INVALID_TOKEN);
      const verification = await authApp.dataSource
        .getRepository(Verification)
        .findOneOrFail({ where: { identifier: EMAIL } });
      expect(verification.consumedAt).toBeNull();
    });

    it("refuses a weak newPassword and leaves the token usable", async () => {
      await createVerifiedUser(authApp, EMAIL, PASSWORD);
      const token = await requestReset(EMAIL);

      const response = await confirm(token, "fraca").expect(400);

      expect(response.body).toEqual({
        type: "tag:clinicore.com.br,2026:validation-failed",
        title: "Validation failed",
        status: 400,
        errors: [{ pointer: "#/newPassword", code: "WEAK_PASSWORD" }],
      });
      await confirm(token, NEW_PASSWORD).expect(204);
    });

    it("gives a Google-only user a credential account", async () => {
      const user = await authApp.dataSource.getRepository(User).save({
        name: "Ana Souza",
        email: EMAIL,
        emailVerified: true,
      });
      await authApp.dataSource.getRepository(Account).save({
        userId: user.id,
        provider: Provider.Google,
        providerAccountId: "google-subject-1",
      });
      const token = await requestReset(EMAIL);

      await confirm(token, PASSWORD).expect(204);

      const providers = await authApp.dataSource
        .getRepository(Account)
        .find({ where: { userId: user.id } });
      expect(providers.map((account) => account.provider).sort()).toEqual([
        Provider.Credential,
        Provider.Google,
      ]);
      await authApp
        .post("/sessions")
        .send({ email: EMAIL, password: PASSWORD })
        .expect(201);
    });
  });
});
