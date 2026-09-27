import { plainToInstance } from "class-transformer";
import {
  IsEmail,
  IsFQDN,
  IsIn,
  IsInt,
  IsNotEmpty,
  IsString,
  Max,
  Min,
  MinLength,
  registerDecorator,
  type ValidationArguments,
  type ValidatorConstraintInterface,
  validateSync,
} from "class-validator";
import isIPRange from "validator/lib/isIPRange";

const NODE_ENVIRONMENTS = ["development", "production", "test"] as const;

const LOG_LEVELS = [
  "fatal",
  "error",
  "warn",
  "info",
  "debug",
  "trace",
  "silent",
] as const;

function isPostgresUrl(value: unknown): boolean {
  if (typeof value !== "string") {
    return false;
  }
  const parsed = URL.parse(value);
  if (parsed === null) {
    return false;
  }
  return (
    ["postgresql:", "postgres:"].includes(parsed.protocol) &&
    parsed.hostname !== ""
  );
}

function isRedisUrl(value: unknown): boolean {
  if (typeof value !== "string") {
    return false;
  }
  const parsed = URL.parse(value);
  if (parsed === null) {
    return false;
  }
  return parsed.protocol === "redis:" && parsed.hostname !== "";
}

function isAbsoluteOrigin(value: unknown): boolean {
  if (typeof value !== "string") {
    return false;
  }
  const parsed = URL.parse(value);
  if (parsed === null) {
    return false;
  }
  if (!["http:", "https:"].includes(parsed.protocol)) {
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

function isCidrList(value: unknown): boolean {
  if (!Array.isArray(value) || value.length === 0) {
    return false;
  }
  return value.every((block) => typeof block === "string" && isIPRange(block));
}

function isEmailEqualToSmtpUser(
  value: unknown,
  args?: ValidationArguments,
): boolean {
  const siblings = args?.object as Partial<Environment> | undefined;
  if (typeof value !== "string" || value === "") {
    return false;
  }
  return value === siblings?.SMTP_USER;
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
  API_URL!: string;

  @validatedBy("isAbsoluteOrigin", isAbsoluteOrigin)
  APP_ORIGIN!: string;

  @validatedBy("isPostgresUrl", isPostgresUrl)
  DATABASE_URL!: string;

  @IsString()
  @IsNotEmpty()
  GOOGLE_CLIENT_ID!: string;

  @IsString()
  @IsNotEmpty()
  GOOGLE_CLIENT_SECRET!: string;

  @IsString()
  @MinLength(32)
  JWT_SECRET!: string;

  @IsIn(LOG_LEVELS)
  LOG_LEVEL!: (typeof LOG_LEVELS)[number];

  @validatedBy("isEmailEqualToSmtpUser", isEmailEqualToSmtpUser)
  MAIL_FROM!: string;

  @IsIn(NODE_ENVIRONMENTS)
  NODE_ENV!: (typeof NODE_ENVIRONMENTS)[number];

  @IsInt()
  @Min(1)
  @Max(65535)
  PORT!: number;

  @validatedBy("isRedisUrl", isRedisUrl)
  REDIS_URL!: string;

  @IsFQDN()
  SMTP_HOST!: string;

  @IsString()
  @IsNotEmpty()
  SMTP_PASSWORD!: string;

  @IsInt()
  @Min(1)
  @Max(65535)
  SMTP_PORT!: number;

  @IsEmail()
  SMTP_USER!: string;

  @validatedBy("isCidrList", isCidrList)
  TRUSTED_PROXIES!: string[];
}

export function validateEnv(source: Record<string, unknown>): Environment {
  const toList = (raw: unknown): unknown => {
    if (typeof raw !== "string") {
      return raw;
    }
    return raw.split(",").map((item) => item.trim());
  };
  const toPort = (raw: unknown): unknown => {
    if (typeof raw !== "string" || !/^[0-9]+$/.test(raw)) {
      return raw;
    }
    return Number(raw);
  };
  const candidate = plainToInstance(Environment, {
    ALLOWED_ORIGINS: toList(source.ALLOWED_ORIGINS),
    API_URL: source.API_URL,
    APP_ORIGIN: source.APP_ORIGIN,
    DATABASE_URL: source.DATABASE_URL,
    GOOGLE_CLIENT_ID: source.GOOGLE_CLIENT_ID,
    GOOGLE_CLIENT_SECRET: source.GOOGLE_CLIENT_SECRET,
    JWT_SECRET: source.JWT_SECRET,
    LOG_LEVEL: source.LOG_LEVEL,
    MAIL_FROM: source.MAIL_FROM,
    NODE_ENV: source.NODE_ENV,
    PORT: toPort(source.PORT),
    REDIS_URL: source.REDIS_URL,
    SMTP_HOST: source.SMTP_HOST,
    SMTP_PASSWORD: source.SMTP_PASSWORD,
    SMTP_PORT: toPort(source.SMTP_PORT),
    SMTP_USER: source.SMTP_USER,
    TRUSTED_PROXIES: toList(source.TRUSTED_PROXIES),
  });

  const errors = validateSync(candidate, { forbidUnknownValues: true });
  if (errors.length > 0) {
    const expectedFormat = {
      ALLOWED_ORIGINS:
        "expected a comma-separated list of absolute URLs with no trailing slash (https://…)",
      API_URL: "expected an absolute URL with no trailing slash (https://…)",
      APP_ORIGIN: "expected an absolute URL with no trailing slash (https://…)",
      DATABASE_URL: "expected a PostgreSQL connection string (postgresql://…)",
      GOOGLE_CLIENT_ID: "expected a non-empty string",
      GOOGLE_CLIENT_SECRET: "expected a non-empty string",
      JWT_SECRET: "expected a string with at least 32 characters",
      LOG_LEVEL: `expected one of: ${LOG_LEVELS.join(", ")}`,
      MAIL_FROM: "expected an email address equal to SMTP_USER",
      NODE_ENV: `expected one of ${NODE_ENVIRONMENTS.join(", ")}`,
      PORT: "expected an integer between 1 and 65535",
      REDIS_URL: "expected a Redis connection string (redis://…)",
      SMTP_HOST: "expected a hostname",
      SMTP_PASSWORD: "expected a non-empty string",
      SMTP_PORT: "expected an integer between 1 and 65535",
      SMTP_USER: "expected an email address",
      TRUSTED_PROXIES:
        "expected a comma-separated list of CIDR blocks (10.0.0.0/8,…)",
    } as const;
    const rejected = new Set(errors.map((error) => error.property));
    const names = Object.keys(
      expectedFormat,
    ) as (keyof typeof expectedFormat)[];
    const lines = names
      .filter((name) => rejected.has(name))
      .sort()
      .map((name) => `  ${name}: ${expectedFormat[name]}`);
    throw new Error(["Invalid environment:", ...lines].join("\n"));
  }

  return candidate;
}
