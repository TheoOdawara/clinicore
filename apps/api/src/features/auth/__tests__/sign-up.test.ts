import { Account } from "../entities/account.entity";
import { User } from "../entities/user.entity";
import { createAuthApp, type AuthApp } from "./auth-app";

const EMAIL = "ana@exemplo.com";
const PASSWORD = "Clinica#2026";

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
