import { Account } from "../entities/account.entity";
import { User } from "../entities/user.entity";
import {
  EmailDispatchKind,
  createAuthApp,
  createVerifiedUser,
  dispatchCount,
  tokenFrom,
  type AuthApp,
  type LogLine,
} from "./auth-app";

const EMAIL = "ana@exemplo.com";
const PASSWORD = "Clinica#2026";
const PINO_ERROR = 50;
const RESPONSE_DEADLINE_IN_MILLISECONDS = 2000;

function isMailFailure(line: LogLine): boolean {
  return line.level === PINO_ERROR && line.email === EMAIL;
}

describe("mail delivery failure", () => {
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

  it("keeps the sign-up and logs the address and the reason when SMTP fails", async () => {
    authApp.transportBehavior = "fail";

    const response = await authApp
      .post("/users")
      .send({ name: "Ana Souza", email: EMAIL, password: PASSWORD })
      .expect(202);

    expect(response.body).toEqual({});
    const user = await authApp.dataSource
      .getRepository(User)
      .findOneOrFail({ where: { email: EMAIL } });
    expect(
      await authApp.dataSource
        .getRepository(Account)
        .count({ where: { userId: user.id } }),
    ).toBe(1);
    expect(
      await dispatchCount(authApp, EMAIL, EmailDispatchKind.Verification),
    ).toBe(1);

    const line = await authApp.nextLog(isMailFailure);
    expect(JSON.stringify(line)).toContain("ECONNREFUSED");
  });

  it("never writes the token or the link into the error log", async () => {
    authApp.transportBehavior = "fail";

    await authApp
      .post("/users")
      .send({ name: "Ana Souza", email: EMAIL, password: PASSWORD })
      .expect(202);
    const token = tokenFrom(authApp.outbox[0], "/verify-email");

    const line = await authApp.nextLog(isMailFailure);
    expect(JSON.stringify(line)).not.toContain(token);
    expect(JSON.stringify(authApp.logLines)).not.toContain(token);
  });

  it("answers the password reset request the same way when SMTP fails", async () => {
    await createVerifiedUser(authApp, EMAIL, PASSWORD);
    authApp.transportBehavior = "fail";

    const response = await authApp
      .post("/password-resets")
      .send({ email: EMAIL })
      .expect(202);

    expect(response.body).toEqual({});
    await authApp.nextLog(isMailFailure);
  });

  it("answers without waiting for a hanging SMTP", async () => {
    authApp.transportBehavior = "hang";

    await authApp
      .post("/users")
      .timeout(RESPONSE_DEADLINE_IN_MILLISECONDS)
      .send({ name: "Ana Souza", email: EMAIL, password: PASSWORD })
      .expect(202);

    expect(authApp.outbox).toHaveLength(1);
  });
});
