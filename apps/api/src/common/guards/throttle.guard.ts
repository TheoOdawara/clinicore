import { Injectable, type ExecutionContext } from "@nestjs/common";
import { ThrottlerGuard } from "@nestjs/throttler";
import type { Request } from "express";
import { BusinessError } from "../exceptions/business-error";

@Injectable()
export class ThrottleGuard extends ThrottlerGuard {
  override async canActivate(context: ExecutionContext): Promise<boolean> {
    try {
      return await super.canActivate(context);
    } catch (error) {
      if (error instanceof BusinessError) {
        throw error;
      }
      throw BusinessError.unavailable(
        "SERVICE_UNAVAILABLE",
        "Service temporarily unavailable",
      );
    }
  }

  protected override getTracker(request: Request): Promise<string> {
    if (request.ip === undefined) {
      return Promise.resolve("unknown");
    }
    return super.getTracker(request);
  }

  protected override throwThrottlingException(): Promise<void> {
    throw BusinessError.rateLimited("RATE_LIMITED", "Too many requests");
  }
}
