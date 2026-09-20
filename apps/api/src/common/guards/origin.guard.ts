import {
  Injectable,
  type CanActivate,
  type ExecutionContext,
} from "@nestjs/common";
import type { Request } from "express";
import { EnvironmentService } from "../../core/config/environment.service";
import { BusinessError } from "../exceptions/business-error";

const GUARDED_METHOD = "POST";

@Injectable()
export class OriginGuard implements CanActivate {
  constructor(private readonly environment: EnvironmentService) {}

  canActivate(context: ExecutionContext): boolean {
    const request = context.switchToHttp().getRequest<Request>();
    if (request.method !== GUARDED_METHOD) {
      return true;
    }

    const origin = request.headers.origin;
    const allowed = this.environment.get("ALLOWED_ORIGINS");
    if (origin === undefined || !allowed.includes(origin)) {
      throw BusinessError.forbidden("INVALID_ORIGIN", "Invalid origin");
    }

    return true;
  }
}
