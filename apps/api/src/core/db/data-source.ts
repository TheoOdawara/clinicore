import { DataSource } from "typeorm";
import { postgresOptions } from "./data-source.options";

const url = process.env.DATABASE_URL;

if (url === undefined) {
  throw new Error(
    "DATABASE_URL is required: expected a PostgreSQL connection string (postgresql://…)",
  );
}

export default new DataSource(postgresOptions(url));
