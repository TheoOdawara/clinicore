import { Account } from "../entities/account.entity";
import { User } from "../entities/user.entity";
import { Verification } from "../entities/verification.entity";
import { VerificationPurpose } from "../enums/verification-purpose.enum";
import { hashSecret } from "../utils/session-token";
import {
  EmailDispatchKind,
  createAuthApp,
  dispatchCount,
  seedDispatches,
  tokenFrom,
  type AuthApp,
} from "./auth-app";

const EMAIL = "ana@exemplo.com";
const PASSWORD = "Clinica#2026";
const HOUR_IN_MILLISECONDS = 60 * 60 * 1000;
const TOLERANCE_IN_MILLISECONDS = 60 * 1000;

describe("POST /users", () => {
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

  it("creates the user and the credential account", async () => {
    const response = await authApp
      .post("/users")
      .send({ name: "Ana Souza", email: EMAIL, password: PASSWORD })
      .expect(202);

    expect(response.body).toEqual({});

    const user = await authApp.dataSource
      .getRepository(User)
      .findOneOrFail({ where: { email: EMAIL } });
    expect(user.emailVerified).toBe(false);

    const accounts = await authApp.dataSource
      .getRepository(Account)
      .find({ where: { userId: user.id } });
    expect(accounts).toHaveLength(1);
    expect(accounts[0]?.passwordHash).not.toContain(PASSWORD);
    expect(response.headers["set-cookie"]).toBeUndefined();
  });

  it("sends the verification link and records the dispatch", async () => {
    await authApp
      .post("/users")
      .send({ name: "Ana Souza", email: EMAIL, password: PASSWORD })
      .expect(202);

    expect(authApp.outbox).toHaveLength(1);
    const [mail] = authApp.outbox;
    expect(mail?.to).toBe(EMAIL);
    expect(mail?.subject).toBe("Confirme seu e-mail no Clinicore");
    expect(mail?.text).toContain(`${authApp.appOrigin}/verify-email?token=`);
    const token = tokenFrom(mail, "/verify-email");

    const verifications = await authApp.dataSource
      .getRepository(Verification)
      .find({ where: { identifier: EMAIL } });
    expect(verifications).toHaveLength(1);
    const [verification] = verifications;
    expect(verification?.purpose).toBe(VerificationPurpose.EmailVerification);
    expect(verification?.consumedAt).toBeNull();
    expect(verification?.tokenHash).toBe(hashSecret(token));
    const lifetime = (verification?.expiresAt.getTime() ?? 0) - Date.now();
    expect(Math.abs(lifetime - HOUR_IN_MILLISECONDS)).toBeLessThanOrEqual(
      TOLERANCE_IN_MILLISECONDS,
    );

    expect(
      await dispatchCount(authApp, EMAIL, EmailDispatchKind.Verification),
    ).toBe(1);
  });

  it("creates the user without a token or a mail when the address used its daily cap", async () => {
    await seedDispatches(
      authApp,
      EMAIL,
      EmailDispatchKind.Verification,
      [120, 3600, 7200, 36000, 72000],
    );

    await authApp
      .post("/users")
      .send({ name: "Ana Souza", email: EMAIL, password: PASSWORD })
      .expect(202);

    expect(await authApp.dataSource.getRepository(User).count()).toBe(1);
    expect(await authApp.dataSource.getRepository(Verification).count()).toBe(
      0,
    );
    expect(authApp.outbox).toHaveLength(0);
    expect(
      await dispatchCount(authApp, EMAIL, EmailDispatchKind.Verification),
    ).toBe(5);
  });

  it("stores the email in lowercase", async () => {
    await authApp
      .post("/users")
      .send({ name: "Ana Souza", email: "Ana@Exemplo.COM", password: PASSWORD })
      .expect(202);

    const users = await authApp.dataSource.getRepository(User).find();
    expect(users.map((user) => user.email)).toEqual([EMAIL]);
  });

  it("answers the repeated sign-up exactly like the first one", async () => {
    const body = { name: "Ana Souza", email: EMAIL, password: PASSWORD };
    const first = await authApp.post("/users").send(body).expect(202);
    const second = await authApp.post("/users").send(body).expect(202);
    const third = await authApp.post("/users").send(body).expect(202);

    expect(second.body).toEqual(first.body);
    expect(third.body).toEqual(first.body);

    const users = await authApp.dataSource.getRepository(User).find();
    expect(users).toHaveLength(1);
    const accounts = await authApp.dataSource.getRepository(Account).find();
    expect(accounts).toHaveLength(1);
    expect(authApp.outbox).toHaveLength(1);
    expect(
      await dispatchCount(authApp, EMAIL, EmailDispatchKind.Verification),
    ).toBe(1);
  });

  it("keeps a single user when two sign-ups race with the same email", async () => {
    const body = { name: "Ana Souza", email: EMAIL, password: PASSWORD };
    const [first, second] = await Promise.all([
      authApp.post("/users").send(body),
      authApp.post("/users").send(body),
    ]);

    expect(first.status).toBe(202);
    expect(second.status).toBe(202);

    const users = await authApp.dataSource.getRepository(User).find();
    expect(users).toHaveLength(1);
    expect(authApp.outbox).toHaveLength(1);
    expect(
      await dispatchCount(authApp, EMAIL, EmailDispatchKind.Verification),
    ).toBe(1);
  });

  it.each([["sem_maiuscula#1"], ["SEM_DIGITO#a"], ["SemEspecial1"], ["Aa#1"]])(
    "refuses the weak password %s",
    async (password) => {
      const response = await authApp
        .post("/users")
        .send({ name: "Ana Souza", email: EMAIL, password })
        .expect(400);

      expect(response.body).toEqual({
        type: "tag:clinicore.com.br,2026:validation-failed",
        title: "Validation failed",
        status: 400,
        errors: [{ pointer: "#/password", code: "WEAK_PASSWORD" }],
      });

      const users = await authApp.dataSource.getRepository(User).find();
      expect(users).toHaveLength(0);
    },
  );

  it("refuses a field the DTO does not declare", async () => {
    const response = await authApp
      .post("/users")
      .send({
        name: "Ana Souza",
        email: EMAIL,
        password: PASSWORD,
        callbackURL: "http://evil.example/x",
      })
      .expect(400);

    expect(response.body).toMatchObject({
      type: "tag:clinicore.com.br,2026:validation-failed",
    });

    const users = await authApp.dataSource.getRepository(User).find();
    expect(users).toHaveLength(0);
  });
});
