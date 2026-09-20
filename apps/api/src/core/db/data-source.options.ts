import { join } from "node:path";
import type { DataSourceOptions } from "typeorm";

export function postgresOptions(url: string): DataSourceOptions {
  return {
    type: "postgres",
    url,
    synchronize: false,
    migrationsRun: false,
    entities: [join(__dirname, "../../features/**/entities/*.entity.{ts,js}")],
    migrations: [join(__dirname, "migrations/*.{ts,js}")],
  };
}
