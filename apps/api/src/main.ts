import "reflect-metadata";
import { writeSync } from "node:fs";
import { NestFactory } from "@nestjs/core";
import type { NestExpressApplication } from "@nestjs/platform-express";
import { DocumentBuilder, SwaggerModule } from "@nestjs/swagger";
import { AppModule } from "./app.module";
import { configureApplication } from "./configure-application";
import { EnvironmentService } from "./core/config/environment.service";
import { PinoLoggerService } from "./core/logger/pino-logger.service";

const INVALID_ENVIRONMENT = "Invalid environment:";

async function createApplication(): Promise<NestExpressApplication> {
  try {
    return await NestFactory.create<NestExpressApplication>(AppModule, {
      abortOnError: false,
      bufferLogs: true,
      autoFlushLogs: false,
    });
  } catch (error) {
    if (
      error instanceof Error &&
      error.message.startsWith(INVALID_ENVIRONMENT)
    ) {
      writeSync(2, `${error.message}\n`);
      process.exit(1);
    }
    throw error;
  }
}

function mountDocumentation(app: NestExpressApplication): void {
  const document = SwaggerModule.createDocument(
    app,
    new DocumentBuilder().setTitle("Clinicore API").setVersion("0.1.0").build(),
  );
  SwaggerModule.setup("api", app, document);
}

async function bootstrap(): Promise<void> {
  const app = await createApplication();
  app.useLogger(app.get(PinoLoggerService));
  app.flushLogs();
  app.enableShutdownHooks();

  const environment = app.get(EnvironmentService);
  if (environment.get("NODE_ENV") !== "production") {
    mountDocumentation(app);
  }

  configureApplication(app, environment);

  await app.listen(environment.get("PORT"));
}

void bootstrap();
