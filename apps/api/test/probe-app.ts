import {
  Body,
  Controller,
  Get,
  Module,
  Post,
  Res,
  type INestApplication,
} from "@nestjs/common";
import { Test } from "@nestjs/testing";
import type { Server } from "node:http";
import { IsEmail, MinLength } from "class-validator";
import type { Response } from "express";
import { AppModule } from "../src/app.module";
import { EnvironmentService } from "../src/core/config/environment.service";
import { LOGGER, createLogger } from "../src/core/logger/logger";

export const PROBE_COOKIE = "session=probe-cookie-value";
export const PROBE_SET_COOKIE = "session=probe-set-cookie-value";

class SignInProbeDto {
  @IsEmail()
  email!: string;

  @MinLength(8)
  password!: string;
}

@Controller("probe")
class ProbeController {
  @Post("sign-in")
  signIn(
    @Body() body: SignInProbeDto,
    @Res({ passthrough: true }) response: Response,
  ): { email: string } {
    response.setHeader("set-cookie", PROBE_SET_COOKIE);
    return { email: body.email };
  }

  @Get("session")
  session(): { status: string } {
    return { status: "ok" };
  }

  @Get("boom")
  boom(): never {
    throw new Error("probe failure with a stack");
  }
}

@Module({ controllers: [ProbeController] })
class ProbeModule {}

type LogLine = Record<string, unknown>;

export class Probe {
  constructor(
    readonly app: INestApplication,
    private readonly lines: LogLine[],
  ) {}

  get server(): Server {
    return this.app.getHttpServer() as Server;
  }

  linesFor(path: string): LogLine[] {
    return this.lines.filter((line) => line.path === path);
  }

  written(): string {
    return JSON.stringify(this.lines);
  }
}

export async function createProbeApp(): Promise<Probe> {
  const lines: LogLine[] = [];
  const destination = {
    write(line: string): void {
      lines.push(JSON.parse(line) as LogLine);
    },
  };

  const moduleRef = await Test.createTestingModule({
    imports: [AppModule, ProbeModule],
  })
    .overrideProvider(LOGGER)
    .useFactory({
      inject: [EnvironmentService],
      factory: (environment: EnvironmentService) =>
        createLogger(environment.get("LOG_LEVEL"), destination),
    })
    .compile();

  const app = moduleRef.createNestApplication();
  await app.init();

  return new Probe(app, lines);
}
