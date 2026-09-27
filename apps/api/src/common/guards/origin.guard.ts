import {
  Injectable,
  type CanActivate,
  type ExecutionContext,
} from "@nestjs/common";
import type { Request } from "express";
import { EnvironmentService } from "../../core/config/environment.service";
import { BusinessError } from "../exceptions/business-error";

@Injectable()
export class OriginGuard implements CanActivate {
  constructor(private readonly environment: EnvironmentService) {}

  canActivate(context: ExecutionContext): boolean {
    const safeMethods = new Set(["GET", "HEAD", "OPTIONS"]);
    const request = context.switchToHttp().getRequest<Request>();
    if (safeMethods.has(request.method)) {
      return true;
    }

    const origin = request.headers.origin;
    if (origin === undefined && request.headers.cookie === undefined) {
      return true;
    }

    const allowed = this.environment.get("ALLOWED_ORIGINS");
    if (origin === undefined || !allowed.includes(origin)) {
      throw BusinessError.forbidden("INVALID_ORIGIN", "Invalid origin");
    }

    return true;
  }
}
