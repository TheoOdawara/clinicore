import { type INestApplication } from "@nestjs/common";
import { APP_FILTER, APP_GUARD, APP_PIPE } from "@nestjs/core";
import type { NestExpressApplication } from "@nestjs/platform-express";
import { Test } from "@nestjs/testing";
import type Redis from "ioredis";
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
import { REDIS } from "../../../core/redis/redis";
import { configureApplication } from "../../../configure-application";
import { AuthModule } from "../auth.module";
import { Provider } from "../enums/provider.enum";
import { User } from "../entities/user.entity";

validateEnv(process.env);

export const REVOKED_KEY_PREFIX = "auth:revoked:";

export class AuthApp {
  constructor(
    readonly app: INestApplication,
    readonly dataSource: DataSource,
    readonly redis: Redis,
    readonly origin: string,
  ) {}

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
  const moduleRef = await Test.createTestingModule({
    imports: [CoreModule, AuthModule],
    providers: [
      { provide: APP_FILTER, useClass: BusinessErrorFilter },
      { provide: APP_GUARD, useClass: OriginGuard },
      { provide: APP_GUARD, useClass: JwtAuthGuard },
      { provide: APP_PIPE, useFactory: validationPipe },
    ],
  }).compile();

  const app = moduleRef.createNestApplication<NestExpressApplication>();
  configureApplication(app, app.get(EnvironmentService));
  await app.init();

  const [origin] = app.get(EnvironmentService).get("ALLOWED_ORIGINS");

  if (origin === undefined) {
    throw new Error("ALLOWED_ORIGINS is empty: expected at least one origin");
  }

  return new AuthApp(app, app.get(DataSource), app.get<Redis>(REDIS), origin);
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

export { Provider };
