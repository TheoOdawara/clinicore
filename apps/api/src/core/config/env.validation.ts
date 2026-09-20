import { plainToInstance } from "class-transformer";
import {
  IsInt,
  Max,
  Min,
  registerDecorator,
  type ValidatorConstraintInterface,
  validateSync,
} from "class-validator";

const EXPECTED_FORMAT = {
  ALLOWED_ORIGINS:
    "expected a comma-separated list of absolute URLs with no trailing slash (https://…)",
  APP_ORIGIN: "expected an absolute URL with no trailing slash (https://…)",
  DATABASE_URL: "expected a PostgreSQL connection string (postgresql://…)",
  PORT: "expected an integer between 1 and 65535",
} as const;

type VariableName = keyof typeof EXPECTED_FORMAT;

const POSTGRES_PROTOCOLS = ["postgresql:", "postgres:"];
const ORIGIN_PROTOCOLS = ["http:", "https:"];
const DIGITS_ONLY = /^[0-9]+$/;

function isPostgresUrl(value: unknown): boolean {
  if (typeof value !== "string") {
    return false;
  }
  const parsed = URL.parse(value);
  if (parsed === null) {
    return false;
  }
  return POSTGRES_PROTOCOLS.includes(parsed.protocol) && parsed.hostname !== "";
}

function isAbsoluteOrigin(value: unknown): boolean {
  if (typeof value !== "string") {
    return false;
  }
  const parsed = URL.parse(value);
  if (parsed === null) {
    return false;
  }
  if (!ORIGIN_PROTOCOLS.includes(parsed.protocol)) {
    return false;
  }
  return parsed.origin === value;
}

function isOriginList(value: unknown): boolean {
  if (!Array.isArray(value) || value.length === 0) {
    return false;
  }
  return value.every(isAbsoluteOrigin);
}

function validatedBy(
  name: string,
  validate: ValidatorConstraintInterface["validate"],
) {
  return function decorate(target: object, propertyName: string): void {
    registerDecorator({
      name,
      target: target.constructor,
      propertyName,
      validator: { validate },
    });
  };
}

export class Environment {
  @validatedBy("isAbsoluteOriginList", isOriginList)
  ALLOWED_ORIGINS!: string[];

  @validatedBy("isAbsoluteOrigin", isAbsoluteOrigin)
  APP_ORIGIN!: string;

  @validatedBy("isPostgresUrl", isPostgresUrl)
  DATABASE_URL!: string;

  @IsInt()
  @Min(1)
  @Max(65535)
  PORT!: number;
}

function toOriginList(raw: unknown): unknown {
  if (typeof raw !== "string") {
    return raw;
  }
  return raw.split(",").map((origin) => origin.trim());
}

function toPort(raw: unknown): unknown {
  if (typeof raw !== "string" || !DIGITS_ONLY.test(raw)) {
    return raw;
  }
  return Number(raw);
}

function describeRejection(rejected: Set<string>): string {
  const names = Object.keys(EXPECTED_FORMAT) as VariableName[];
  const lines = names
    .filter((name) => rejected.has(name))
    .sort()
    .map((name) => `  ${name}: ${EXPECTED_FORMAT[name]}`);
  return ["Invalid environment:", ...lines].join("\n");
}

export function validateEnv(source: Record<string, unknown>): Environment {
  const candidate = plainToInstance(Environment, {
    ALLOWED_ORIGINS: toOriginList(source.ALLOWED_ORIGINS),
    APP_ORIGIN: source.APP_ORIGIN,
    DATABASE_URL: source.DATABASE_URL,
    PORT: toPort(source.PORT),
  });

  const errors = validateSync(candidate, { forbidUnknownValues: true });
  if (errors.length > 0) {
    throw new Error(
      describeRejection(new Set(errors.map((error) => error.property))),
    );
  }

  return candidate;
}
