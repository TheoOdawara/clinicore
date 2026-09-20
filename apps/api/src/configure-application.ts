import type { NestExpressApplication } from "@nestjs/platform-express";
import type { EnvironmentService } from "./core/config/environment.service";

export function configureApplication(
  app: NestExpressApplication,
  environment: EnvironmentService,
): void {
  app.enableCors({
    origin: environment.get("ALLOWED_ORIGINS"),
    credentials: true,
    methods: ["GET", "POST", "OPTIONS"],
    allowedHeaders: ["Content-Type"],
  });
  app.set("trust proxy", environment.get("TRUSTED_PROXIES"));
}
