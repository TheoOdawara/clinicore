import { Session } from "../entities/session.entity";
import { User } from "../entities/user.entity";
import { Verification } from "../entities/verification.entity";
import { VerificationPurpose } from "../enums/verification-purpose.enum";
import {
  EmailDispatchKind,
  ageDispatches,
  cookieNamed,
  createAuthApp,
  createVerifiedUser,
  dispatchCount,
  tokenFrom,
  type AuthApp,
} from "./auth-app";

const EMAIL = "ana@exemplo.com";
const PASSWORD = "Clinica#2026";
const ACCESS_COOKIE = "clinicore_access";
const REFRESH_COOKIE = "clinicore_refresh";
const UNKNOWN_TOKEN = "A".repeat(43);

const INVALID_TOKEN = {
  type: "tag:clinicore.com.br,2026:invalid-token",
  title: "Invalid token",
  status: 400,
};

describe("POST /email-verifications", () => {
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

  async function signUp(): Promise<string> {
    await authApp
      .post("/users")
      .send({ name: "Ana Souza", email: EMAIL, password: PASSWORD })
      .expect(202);

    return tokenFrom(authApp.outbox.at(-1), "/verify-email");
  }

  async function resend(email: string): Promise<void> {
    const response = await authApp
      .post("/email-verifications")
      .send({ email })
      .expect(202);

    expect(response.body).toEqual({});
  }

  it("sends a new link to an unverified address outside the window", async () => {
    await signUp();
    await ageDispatches(authApp, EMAIL, 90);

    await resend(EMAIL);

    expect(authApp.outbox).toHaveLength(2);
    expect(authApp.outbox[1]?.subject).toBe("Confirme seu e-mail no Clinicore");
    tokenFrom(authApp.outbox[1], "/verify-email");
  });

  it("answers 202 to an unknown address and sends nothing", async () => {
    await resend("ninguem@exemplo.com");

    expect(authApp.outbox).toHaveLength(0);
    expect(
      await dispatchCount(
        authApp,
        "ninguem@exemplo.com",
        EmailDispatchKind.Verification,
      ),
    ).toBe(1);
  });

  it("sends nothing to an already verified address", async () => {
    await createVerifiedUser(authApp, EMAIL, PASSWORD);
    await ageDispatches(authApp, EMAIL, 90);

    await resend(EMAIL);

    expect(authApp.outbox).toHaveLength(1);
    expect(
      await authApp.dataSource
        .getRepository(Verification)
        .count({ where: { identifier: EMAIL } }),
    ).toBe(1);
  });

  describe("confirmation", () => {
    it("opens the session and consumes every pending verification token of the address", async () => {
      const first = await signUp();
      await ageDispatches(authApp, EMAIL, 90);
      await resend(EMAIL);
      const second = tokenFrom(authApp.outbox.at(-1), "/verify-email");
      expect(second).not.toBe(first);

      const response = await authApp
        .post("/email-verifications/confirmation")
        .send({ token: first })
        .expect(204);

      expect(cookieNamed(response, ACCESS_COOKIE)).toContain("HttpOnly");
      expect(cookieNamed(response, REFRESH_COOKIE)).toContain("HttpOnly");

      const user = await authApp.dataSource
        .getRepository(User)
        .findOneOrFail({ where: { email: EMAIL } });
      expect(user.emailVerified).toBe(true);
      expect(
        await authApp.dataSource
          .getRepository(Session)
          .count({ where: { userId: user.id } }),
      ).toBe(1);

      const verifications = await authApp.dataSource
        .getRepository(Verification)
        .find({ where: { identifier: EMAIL } });
      expect(verifications).toHaveLength(2);
      verifications.forEach((verification) => {
        expect(verification.consumedAt).toBeInstanceOf(Date);
      });
    });

    it("refuses either token again without opening a second session", async () => {
      const first = await signUp();
      await ageDispatches(authApp, EMAIL, 90);
      await resend(EMAIL);
      const second = tokenFrom(authApp.outbox.at(-1), "/verify-email");
      await authApp
        .post("/email-verifications/confirmation")
        .send({ token: first })
        .expect(204);

      for (const token of [first, second]) {
        const response = await authApp
          .post("/email-verifications/confirmation")
          .send({ token })
          .expect(400);

        expect(response.body).toEqual(INVALID_TOKEN);
        expect(response.headers["set-cookie"]).toBeUndefined();
      }

      expect(await authApp.dataSource.getRepository(Session).count()).toBe(1);
    });

    it("leaves a pending password reset token of the same address usable", async () => {
      const verificationToken = await signUp();
      await authApp.post("/password-resets").send({ email: EMAIL }).expect(202);
      const resetToken = tokenFrom(authApp.outbox.at(-1), "/reset-password");

      await authApp
        .post("/email-verifications/confirmation")
        .send({ token: verificationToken })
        .expect(204);

      await authApp
        .post("/password-resets/confirmation")
        .send({ token: resetToken, newPassword: "Outra#Senha9" })
        .expect(204);
    });

    it("refuses an unknown token with INVALID_TOKEN and no cookie", async () => {
      await signUp();

      const response = await authApp
        .post("/email-verifications/confirmation")
        .send({ token: UNKNOWN_TOKEN })
        .expect(400);

      expect(response.body).toEqual(INVALID_TOKEN);
      expect(response.headers["set-cookie"]).toBeUndefined();
      const user = await authApp.dataSource
        .getRepository(User)
        .findOneOrFail({ where: { email: EMAIL } });
      expect(user.emailVerified).toBe(false);
      expect(await authApp.dataSource.getRepository(Session).count()).toBe(0);
    });

    it("refuses an expired token with TOKEN_EXPIRED and leaves the user unverified", async () => {
      const token = await signUp();
      await authApp.dataSource
        .getRepository(Verification)
        .update(
          { identifier: EMAIL },
          { expiresAt: new Date(Date.now() - 60_000) },
        );

      const response = await authApp
        .post("/email-verifications/confirmation")
        .send({ token })
        .expect(400);

      expect(response.body).toEqual({
        type: "tag:clinicore.com.br,2026:token-expired",
        title: "Token expired",
        status: 400,
      });
      expect(response.headers["set-cookie"]).toBeUndefined();
      const user = await authApp.dataSource
        .getRepository(User)
        .findOneOrFail({ where: { email: EMAIL } });
      expect(user.emailVerified).toBe(false);
      expect(await authApp.dataSource.getRepository(Session).count()).toBe(0);
    });

    it("refuses a password reset token with INVALID_TOKEN", async () => {
      await signUp();
      await authApp.post("/password-resets").send({ email: EMAIL }).expect(202);
      const resetToken = tokenFrom(authApp.outbox.at(-1), "/reset-password");

      const response = await authApp
        .post("/email-verifications/confirmation")
        .send({ token: resetToken })
        .expect(400);

      expect(response.body).toEqual(INVALID_TOKEN);
      const reset = await authApp.dataSource
        .getRepository(Verification)
        .findOneOrFail({
          where: {
            identifier: EMAIL,
            purpose: VerificationPurpose.PasswordReset,
          },
        });
      expect(reset.consumedAt).toBeNull();
    });

    it("refuses a token that is not 43 base64url characters", async () => {
      const response = await authApp
        .post("/email-verifications/confirmation")
        .send({ token: "curto" })
        .expect(400);

      expect(response.body).toMatchObject({
        type: "tag:clinicore.com.br,2026:validation-failed",
        errors: [expect.objectContaining({ pointer: "#/token" })],
      });
    });
  });
});
