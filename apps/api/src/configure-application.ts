import cookieParser from "cookie-parser";
import type { NestExpressApplication } from "@nestjs/platform-express";
import type { EnvironmentService } from "./core/config/environment.service";

export function configureApplication(
  app: NestExpressApplication,
  environment: EnvironmentService,
): void {
  app.enableCors({
    origin: environment.get("ALLOWED_ORIGINS"),
    credentials: true,
    methods: ["GET", "POST", "PUT", "PATCH", "DELETE", "OPTIONS"],
    allowedHeaders: ["Content-Type"],
  });
  app.use(cookieParser());
  app.set("trust proxy", environment.get("TRUSTED_PROXIES"));
}
