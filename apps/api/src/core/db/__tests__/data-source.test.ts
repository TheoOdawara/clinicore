import { Test } from "@nestjs/testing";
import { DataSource } from "typeorm";
import { freePort } from "../../../__tests__/free-port";
import { ConfigModule } from "../../config/config.module";
import { postgresOptions } from "../data-source.options";
import { DbModule } from "../db.module";

describe("the TypeORM DataSource", () => {
  it("never changes the schema by itself", () => {
    const options = postgresOptions(
      "postgresql://clinicore:local@127.0.0.1:5432/clinicore",
    );

    expect(options.synchronize).toBe(false);
    expect(options.migrationsRun).toBe(false);
  });

  it("opens an empty transaction against the compose Postgres", async () => {
    const moduleRef = await Test.createTestingModule({
      imports: [ConfigModule, DbModule],
    }).compile();
    const dataSource = moduleRef.get(DataSource);

    await expect(
      dataSource.transaction(() => Promise.resolve()),
    ).resolves.toBeUndefined();

    await moduleRef.close();
  });

  it("fails with ECONNREFUSED when nothing listens on the port", async () => {
    const port = await freePort();
    const unreachable = new DataSource(
      postgresOptions(
        `postgresql://clinicore:local@127.0.0.1:${String(port)}/clinicore`,
      ),
    );

    await expect(unreachable.initialize()).rejects.toMatchObject({
      code: "ECONNREFUSED",
    });
  });
});
