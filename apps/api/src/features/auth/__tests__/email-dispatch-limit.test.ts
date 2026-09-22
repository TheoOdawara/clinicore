import { Verification } from "../entities/verification.entity";
import { EmailDispatchRepository } from "../repository/email-dispatch.repository";
import {
  EmailDispatchKind,
  ageDispatches,
  createAuthApp,
  createVerifiedUser,
  dispatchCount,
  openConnections,
  seedDispatches,
  type AuthApp,
} from "./auth-app";

const EMAIL = "ana@exemplo.com";
const UNKNOWN_EMAIL = "ninguem@exemplo.com";
const PASSWORD = "Clinica#2026";
const RACERS = 5;

describe("per-address email limit", () => {
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

  async function verifiedUser(): Promise<void> {
    await createVerifiedUser(authApp, EMAIL, PASSWORD);
    authApp.outbox.length = 0;
  }

  function requestReset(email: string, clientIp: string) {
    return authApp
      .post("/password-resets")
      .set("X-Forwarded-For", clientIp)
      .send({ email })
      .expect(202);
  }

  function requestVerification(email: string) {
    return authApp.post("/email-verifications").send({ email }).expect(202);
  }

  it("accepts one request per address and kind in 60 seconds, from any IP", async () => {
    await verifiedUser();

    const responses = [];
    for (const clientIp of [
      "203.0.113.1",
      "203.0.113.2",
      "203.0.113.3",
      "203.0.113.4",
    ]) {
      responses.push(await requestReset(EMAIL, clientIp));
    }

    responses.forEach((response) => {
      expect(response.body).toEqual({});
    });
    expect(authApp.outbox).toHaveLength(1);
    expect(
      await authApp.dataSource
        .getRepository(Verification)
        .count({ where: { identifier: EMAIL } }),
    ).toBe(2);
    expect(
      await dispatchCount(authApp, EMAIL, EmailDispatchKind.PasswordReset),
    ).toBe(1);
  });

  it("accepts again once the last request is older than 60 seconds", async () => {
    await verifiedUser();
    await seedDispatches(authApp, EMAIL, EmailDispatchKind.PasswordReset, [90]);

    await requestReset(EMAIL, "203.0.113.1");

    expect(authApp.outbox).toHaveLength(1);
  });

  it("holds the request that would be the sixth in 24 hours", async () => {
    await seedDispatches(
      authApp,
      UNKNOWN_EMAIL,
      EmailDispatchKind.Verification,
      [120, 3600, 7200, 36000, 72000],
    );

    const response = await requestVerification(UNKNOWN_EMAIL);

    expect(response.body).toEqual({});
    expect(
      await dispatchCount(
        authApp,
        UNKNOWN_EMAIL,
        EmailDispatchKind.Verification,
      ),
    ).toBe(5);
  });

  it("holds the sixth verification of a registered address and still accepts its password reset", async () => {
    await authApp
      .post("/users")
      .send({ name: "Ana Souza", email: EMAIL, password: PASSWORD })
      .expect(202);
    await ageDispatches(authApp, EMAIL, 120);
    await seedDispatches(
      authApp,
      EMAIL,
      EmailDispatchKind.Verification,
      [3600, 7200, 36000, 72000],
    );
    authApp.outbox.length = 0;

    await requestVerification(EMAIL);

    expect(authApp.outbox).toHaveLength(0);
    expect(
      await dispatchCount(authApp, EMAIL, EmailDispatchKind.Verification),
    ).toBe(5);

    await requestReset(EMAIL, "203.0.113.1");

    expect(authApp.outbox).toHaveLength(1);
    expect(authApp.outbox[0]?.subject).toBe("Redefinir sua senha do Clinicore");
  });

  it("accepts the fifth request in 24 hours", async () => {
    await verifiedUser();
    await seedDispatches(
      authApp,
      EMAIL,
      EmailDispatchKind.PasswordReset,
      [120, 3600, 36000, 72000],
    );

    await requestReset(EMAIL, "203.0.113.1");

    expect(authApp.outbox).toHaveLength(1);
  });

  it("ignores requests older than 24 hours", async () => {
    await verifiedUser();
    await seedDispatches(authApp, EMAIL, EmailDispatchKind.PasswordReset, [
      120,
      3600,
      36000,
      72000,
      25 * 3600,
    ]);

    await requestReset(EMAIL, "203.0.113.1");

    expect(authApp.outbox).toHaveLength(1);
  });

  it("counts each address separately", async () => {
    await verifiedUser();
    await seedDispatches(
      authApp,
      "outra@exemplo.com",
      EmailDispatchKind.PasswordReset,
      [10, 3600, 7200, 36000, 72000],
    );

    await requestReset(EMAIL, "203.0.113.1");

    expect(authApp.outbox).toHaveLength(1);
  });

  it("counts the address in any letter case", async () => {
    await verifiedUser();
    await seedDispatches(authApp, EMAIL, EmailDispatchKind.PasswordReset, [10]);

    await requestReset("Ana@Exemplo.COM", "203.0.113.1");

    expect(authApp.outbox).toHaveLength(0);
    expect(
      await dispatchCount(authApp, EMAIL, EmailDispatchKind.PasswordReset),
    ).toBe(1);
  });

  it("records the request of an address without an account", async () => {
    const first = await requestReset(UNKNOWN_EMAIL, "203.0.113.1");
    const second = await requestReset(UNKNOWN_EMAIL, "203.0.113.2");

    expect(second.body).toEqual(first.body);
    expect(authApp.outbox).toHaveLength(0);
    expect(
      await dispatchCount(
        authApp,
        UNKNOWN_EMAIL,
        EmailDispatchKind.PasswordReset,
      ),
    ).toBe(1);
  });

  it("lets exactly one of five simultaneous registrations through", async () => {
    const repository = authApp.app.get(EmailDispatchRepository);
    await openConnections(authApp, RACERS);

    const outcomes = await Promise.all(
      Array.from({ length: RACERS }, () =>
        repository.registerDispatch(EMAIL, EmailDispatchKind.PasswordReset),
      ),
    );

    expect(outcomes.filter((accepted) => accepted)).toHaveLength(1);
    expect(
      await dispatchCount(authApp, EMAIL, EmailDispatchKind.PasswordReset),
    ).toBe(1);
  });
});
