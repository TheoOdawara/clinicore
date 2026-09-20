import "reflect-metadata";
import { writeSync } from "node:fs";
import type { INestApplication } from "@nestjs/common";
import { NestFactory } from "@nestjs/core";
import { DocumentBuilder, SwaggerModule } from "@nestjs/swagger";
import { AppModule } from "./app.module";
import { EnvironmentService } from "./core/config/environment.service";

const INVALID_ENVIRONMENT = "Invalid environment:";

async function createApplication(): Promise<INestApplication> {
  try {
    return await NestFactory.create(AppModule, {
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

function mountDocumentation(app: INestApplication): void {
  const document = SwaggerModule.createDocument(
    app,
    new DocumentBuilder().setTitle("Clinicore API").setVersion("0.1.0").build(),
  );
  SwaggerModule.setup("api", app, document);
}

async function bootstrap(): Promise<void> {
  const app = await createApplication();
  app.flushLogs();
  app.enableShutdownHooks();

  const environment = app.get(EnvironmentService);
  if (environment.get("NODE_ENV") !== "production") {
    mountDocumentation(app);
  }

  await app.listen(environment.get("PORT"));
}

void bootstrap();
