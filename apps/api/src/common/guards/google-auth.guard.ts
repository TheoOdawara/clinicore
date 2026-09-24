import { Inject, Injectable, type ExecutionContext } from "@nestjs/common";
import { AuthGuard } from "@nestjs/passport";
import type { Request } from "express";
import type { Logger } from "pino";
import type { Observable } from "rxjs";
import { LOGGER } from "../../core/logger/logger";
import { BusinessError } from "../exceptions/business-error";

@Injectable()
export class GoogleAuthGuard extends AuthGuard("google") {
  override getAuthenticateOptions(): { prompt: string } {
    return { prompt: "select_account" };
  }
}

@Injectable()
export class GoogleCallbackGuard extends GoogleAuthGuard {
  constructor(@Inject(LOGGER) private readonly logger: Logger) {
    super();
  }

  override canActivate(
    context: ExecutionContext,
  ): boolean | Promise<boolean> | Observable<boolean> {
    const { code } = context.switchToHttp().getRequest<Request>().query;

    if (typeof code !== "string" || code === "") {
      throw BusinessError.invalid("INVALID_STATE", "Invalid OAuth state");
    }

    return super.canActivate(context);
  }

  override handleRequest<TUser>(error: unknown, user: TUser | false): TUser {
    if (error !== null) {
      this.logger.warn({ err: error }, "google sign-in failed");
    }

    if (error !== null || user === false) {
      throw BusinessError.invalid("INVALID_STATE", "Invalid OAuth state");
    }

    return user;
  }
}
