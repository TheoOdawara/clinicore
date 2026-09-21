import { Injectable, type ExecutionContext } from "@nestjs/common";
import { Reflector } from "@nestjs/core";
import { AuthGuard } from "@nestjs/passport";
import type { Observable } from "rxjs";
import { IS_PUBLIC } from "../decorators/public.decorator";
import { BusinessError } from "../exceptions/business-error";

@Injectable()
export class JwtAuthGuard extends AuthGuard("jwt") {
  constructor(private readonly reflector: Reflector) {
    super();
  }

  override canActivate(
    context: ExecutionContext,
  ): boolean | Promise<boolean> | Observable<boolean> {
    const isPublic = this.reflector.getAllAndOverride<boolean>(IS_PUBLIC, [
      context.getHandler(),
      context.getClass(),
    ]);

    if (isPublic) {
      return true;
    }

    return super.canActivate(context);
  }

  override handleRequest<TUser>(error: unknown, user: TUser | false): TUser {
    if (error instanceof BusinessError) {
      throw error;
    }

    if (error !== null || user === false) {
      throw BusinessError.unauthorized("INVALID_SESSION", "Invalid session");
    }

    return user;
  }
}
