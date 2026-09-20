import { Test, type TestingModule } from "@nestjs/testing";
import { DataSource } from "typeorm";
import { ConfigModule } from "../../config/config.module";
import { validateEnv } from "../../config/env.validation";
import { postgresOptions } from "../data-source.options";
import { DbModule } from "../db.module";

const CLOSED_PORT = 1;

describe("the TypeORM DataSource", () => {
  let moduleRef: TestingModule;

  beforeAll(async () => {
    moduleRef = await Test.createTestingModule({
      imports: [ConfigModule, DbModule],
    }).compile();
  });

  afterAll(async () => {
    await moduleRef.close();
  });

  it("never changes the schema by itself", () => {
    const options = postgresOptions(validateEnv(process.env).DATABASE_URL);

    expect(options.synchronize).toBe(false);
    expect(options.migrationsRun).toBe(false);
  });

  it("opens an empty transaction against the compose Postgres", async () => {
    const dataSource = moduleRef.get(DataSource);

    await expect(
      dataSource.transaction(() => Promise.resolve()),
    ).resolves.toBeUndefined();
  });

  it("fails with ECONNREFUSED when nothing listens on the port", async () => {
    const unreachable = new DataSource(
      postgresOptions(
        `postgresql://clinicore:local@127.0.0.1:${String(CLOSED_PORT)}/clinicore`,
      ),
    );

    await expect(unreachable.initialize()).rejects.toMatchObject({
      code: "ECONNREFUSED",
    });
  });
});
