import { Catch, Inject, type ArgumentsHost } from "@nestjs/common";
import type { Response } from "express";
import type { Logger } from "pino";
import { EnvironmentService } from "../../core/config/environment.service";
import { LOGGER } from "../../core/logger/logger";
import { BusinessError, type ErrorCode } from "../exceptions/business-error";
import { BusinessErrorFilter } from "./business-error.filter";

@Catch(BusinessError)
export class GoogleRedirectFilter extends BusinessErrorFilter {
  constructor(
    @Inject(LOGGER) logger: Logger,
    private readonly environment: EnvironmentService,
  ) {
    super(logger);
  }

  override catch(exception: BusinessError, host: ArgumentsHost): void {
    const redirectedCodes = new Set<ErrorCode>([
      "INVALID_STATE",
      "UNVERIFIED_PROVIDER_EMAIL",
    ]);
    if (!redirectedCodes.has(exception.code)) {
      super.catch(exception, host);
      return;
    }

    const login = new URL("/login", this.environment.get("APP_ORIGIN"));
    login.searchParams.set("error", exception.code);
    host.switchToHttp().getResponse<Response>().redirect(login.toString());
  }
}
