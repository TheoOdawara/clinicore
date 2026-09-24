import {
  Injectable,
  type CanActivate,
  type ExecutionContext,
} from "@nestjs/common";
import type { Request } from "express";
import { sessionClientOf } from "../decorators/session-client.decorator";

@Injectable()
export class SessionClientGuard implements CanActivate {
  canActivate(context: ExecutionContext): boolean {
    sessionClientOf(context.switchToHttp().getRequest<Request>());

    return true;
  }
}
