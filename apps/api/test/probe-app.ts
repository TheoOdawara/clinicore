import {
  All,
  Body,
  Controller,
  Get,
  InternalServerErrorException,
  Module,
  Post,
  Res,
  ServiceUnavailableException,
  type INestApplication,
} from "@nestjs/common";
import { Test } from "@nestjs/testing";
import type { Server } from "node:http";
import { Type } from "class-transformer";
import { IsEmail, IsString, MinLength, ValidateNested } from "class-validator";
import type { NestExpressApplication } from "@nestjs/platform-express";
import type { Response } from "express";
import { AppModule } from "../src/app.module";
import { Public } from "../src/common/decorators/public.decorator";
import { BusinessError } from "../src/common/exceptions/business-error";
import { configureApplication } from "../src/configure-application";
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

class AddressProbeDto {
  @IsString()
  zip!: string;
}

class ProfileProbeDto {
  @ValidateNested()
  @Type(() => AddressProbeDto)
  address!: AddressProbeDto;
}

@Public()
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

  @All("resource")
  resource(): { status: string } {
    return { status: "ok" };
  }

  @Post("profile")
  profile(@Body() body: ProfileProbeDto): ProfileProbeDto {
    return body;
  }

  @Get("internal")
  internal(): never {
    throw new InternalServerErrorException(
      "connection to db-prod:5432 refused",
    );
  }

  @Get("unavailable")
  unavailable(): never {
    throw new ServiceUnavailableException("redis at cache-prod:6379 is down");
  }

  @Get("business-unavailable")
  businessUnavailable(): never {
    throw BusinessError.unavailable(
      "SERVICE_UNAVAILABLE",
      "Service temporarily unavailable",
    );
  }

  @Get("hang")
  hang(): Promise<never> {
    return new Promise<never>(() => undefined);
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

  const app = moduleRef.createNestApplication<NestExpressApplication>();
  configureApplication(app, app.get(EnvironmentService));
  await app.init();

  return new Probe(app, lines);
}
