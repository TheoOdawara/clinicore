import { type INestApplication } from "@nestjs/common";
import { APP_FILTER, APP_GUARD, APP_PIPE } from "@nestjs/core";
import type { NestExpressApplication } from "@nestjs/platform-express";
import { Test } from "@nestjs/testing";
import type Redis from "ioredis";
import type { SendMailOptions } from "nodemailer";
import type { Server } from "node:http";
import request from "supertest";
import { DataSource } from "typeorm";
import { BusinessErrorFilter } from "../../../common/filters/business-error.filter";
import { JwtAuthGuard } from "../../../common/guards/jwt-auth.guard";
import { OriginGuard } from "../../../common/guards/origin.guard";
import { validationPipe } from "../../../common/pipes/validation.pipe";
import { EnvironmentService } from "../../../core/config/environment.service";
import { validateEnv } from "../../../core/config/env.validation";
import { CoreModule } from "../../../core/core.module";
import { LOGGER, createLogger } from "../../../core/logger/logger";
import { MAIL_TRANSPORT, type MailTransport } from "../../../core/mail/mail";
import { REDIS } from "../../../core/redis/redis";
import { configureApplication } from "../../../configure-application";
import { AuthModule } from "../auth.module";
import { Provider } from "../enums/provider.enum";
import { EmailDispatchKind } from "../enums/email-dispatch-kind.enum";
import { EmailDispatch } from "../entities/email-dispatch.entity";
import { User } from "../entities/user.entity";
import { Verification } from "../entities/verification.entity";

validateEnv(process.env);

export const REVOKED_KEY_PREFIX = "auth:revoked:";

const TOKEN_PATTERN = "([A-Za-z0-9_-]{43})";

export interface SentMail {
  to: string;
  subject: string;
  html: string;
  text: string;
}

export type LogLine = Record<string, unknown>;

export type TransportBehavior = "deliver" | "fail" | "hang";

interface LogWaiter {
  matches: (line: LogLine) => boolean;
  resolve: (line: LogLine) => void;
}

function textOf(value: unknown, field: string): string {
  if (typeof value !== "string") {
    throw new Error(`the mail ${field} is not a string`);
  }

  return value;
}

class RecordingTransport implements MailTransport {
  readonly outbox: SentMail[] = [];
  behavior: TransportBehavior = "deliver";

  sendMail(options: SendMailOptions): Promise<void> {
    this.outbox.push({
      to: textOf(options.to, "to"),
      subject: textOf(options.subject, "subject"),
      html: textOf(options.html, "html"),
      text: textOf(options.text, "text"),
    });

    if (this.behavior === "fail") {
      return Promise.reject(
        new Error("connect ECONNREFUSED smtp.gmail.com:587"),
      );
    }

    if (this.behavior === "hang") {
      return new Promise<void>(() => undefined);
    }

    return Promise.resolve();
  }
}

class LogCollector {
  readonly lines: LogLine[] = [];
  private waiters: LogWaiter[] = [];

  write(raw: string): void {
    const line = JSON.parse(raw) as LogLine;
    this.lines.push(line);
    const pending = this.waiters.filter((waiter) => waiter.matches(line));
    this.waiters = this.waiters.filter((waiter) => !waiter.matches(line));
    pending.forEach((waiter) => {
      waiter.resolve(line);
    });
  }

  next(matches: (line: LogLine) => boolean): Promise<LogLine> {
    const found = this.lines.find(matches);
    if (found !== undefined) {
      return Promise.resolve(found);
    }

    return new Promise((resolve) => {
      this.waiters.push({ matches, resolve });
    });
  }

  clear(): void {
    this.lines.length = 0;
    this.waiters = [];
  }
}

export class AuthApp {
  constructor(
    readonly app: INestApplication,
    readonly dataSource: DataSource,
    readonly redis: Redis,
    readonly origin: string,
    private readonly transport: RecordingTransport,
    private readonly logs: LogCollector,
  ) {}

  get appOrigin(): string {
    return this.app.get(EnvironmentService).get("APP_ORIGIN");
  }

  get outbox(): SentMail[] {
    return this.transport.outbox;
  }

  set transportBehavior(behavior: TransportBehavior) {
    this.transport.behavior = behavior;
  }

  get logLines(): LogLine[] {
    return this.logs.lines;
  }

  nextLog(matches: (line: LogLine) => boolean): Promise<LogLine> {
    return this.logs.next(matches);
  }

  get server(): Server {
    return this.app.getHttpServer() as Server;
  }

  post(path: string): request.Test {
    return request(this.server).post(path).set("Origin", this.origin);
  }

  delete(path: string): request.Test {
    return request(this.server).delete(path).set("Origin", this.origin);
  }

  get(path: string): request.Test {
    return request(this.server).get(path);
  }

  async reset(): Promise<void> {
    await this.dataSource.createQueryBuilder().delete().from(User).execute();
    await this.dataSource
      .createQueryBuilder()
      .delete()
      .from(Verification)
      .execute();
    await this.dataSource
      .createQueryBuilder()
      .delete()
      .from(EmailDispatch)
      .execute();
    this.transport.outbox.length = 0;
    this.transport.behavior = "deliver";
    this.logs.clear();
    const keys = await this.redis.keys(`${REVOKED_KEY_PREFIX}*`);
    if (keys.length > 0) {
      await this.redis.del(...keys);
    }
  }

  async close(): Promise<void> {
    await this.reset();
    await this.app.close();
  }
}

export async function createAuthApp(): Promise<AuthApp> {
  const transport = new RecordingTransport();
  const logs = new LogCollector();
  const moduleRef = await Test.createTestingModule({
    imports: [CoreModule, AuthModule],
    providers: [
      { provide: APP_FILTER, useClass: BusinessErrorFilter },
      { provide: APP_GUARD, useClass: OriginGuard },
      { provide: APP_GUARD, useClass: JwtAuthGuard },
      { provide: APP_PIPE, useFactory: validationPipe },
    ],
  })
    .overrideProvider(MAIL_TRANSPORT)
    .useValue(transport)
    .overrideProvider(LOGGER)
    .useFactory({ factory: () => createLogger("error", logs) })
    .compile();

  const app = moduleRef.createNestApplication<NestExpressApplication>();
  configureApplication(app, app.get(EnvironmentService));
  await app.init();

  const [origin] = app.get(EnvironmentService).get("ALLOWED_ORIGINS");

  if (origin === undefined) {
    throw new Error("ALLOWED_ORIGINS is empty: expected at least one origin");
  }

  return new AuthApp(
    app,
    app.get(DataSource),
    app.get<Redis>(REDIS),
    origin,
    transport,
    logs,
  );
}

export async function openConnections(
  authApp: AuthApp,
  connections: number,
): Promise<void> {
  await Promise.all(
    Array.from({ length: connections }, () =>
      authApp.dataSource.transaction((manager) => manager.count(User)),
    ),
  );
}

export function tokenFrom(mail: SentMail | undefined, path: string): string {
  if (mail === undefined) {
    throw new Error(`no mail was sent, expected a link to ${path}`);
  }

  const found = new RegExp(`${path}\\?token=${TOKEN_PATTERN}`).exec(mail.text);
  const token = found?.[1];
  if (token === undefined) {
    throw new Error(`the mail has no link to ${path}`);
  }

  return token;
}

export async function seedDispatches(
  authApp: AuthApp,
  email: string,
  kind: EmailDispatchKind,
  agesInSeconds: number[],
): Promise<void> {
  const rows = agesInSeconds.map((ageInSeconds) => ({
    email,
    kind,
    createdAt: new Date(Date.now() - ageInSeconds * 1000),
  }));
  await authApp.dataSource.getRepository(EmailDispatch).insert(rows);
}

export async function ageDispatches(
  authApp: AuthApp,
  email: string,
  ageInSeconds: number,
): Promise<void> {
  await authApp.dataSource
    .getRepository(EmailDispatch)
    .update(
      { email },
      { createdAt: new Date(Date.now() - ageInSeconds * 1000) },
    );
}

export function dispatchCount(
  authApp: AuthApp,
  email: string,
  kind: EmailDispatchKind,
): Promise<number> {
  return authApp.dataSource
    .getRepository(EmailDispatch)
    .count({ where: { email, kind } });
}

export async function createVerifiedUser(
  authApp: AuthApp,
  email: string,
  password: string,
): Promise<User> {
  await authApp
    .post("/users")
    .send({ name: "Ana Souza", email, password })
    .expect(202);

  const users = authApp.dataSource.getRepository(User);
  const user = await users.findOneOrFail({ where: { email } });
  await users.update(user.id, { emailVerified: true });

  return user;
}

export function cookiesOf(response: request.Response): string[] {
  const header = response.headers["set-cookie"];
  if (Array.isArray(header)) {
    return header;
  }

  if (typeof header === "string") {
    return [header];
  }

  return [];
}

export function cookieNamed(
  response: request.Response,
  name: string,
): string | undefined {
  return cookiesOf(response).find((cookie) => cookie.startsWith(`${name}=`));
}

export function valueOf(cookie: string): string {
  return cookie.slice(cookie.indexOf("=") + 1, cookie.indexOf(";"));
}

export { EmailDispatchKind, Provider };
