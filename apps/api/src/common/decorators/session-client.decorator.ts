import { createParamDecorator, type ExecutionContext } from "@nestjs/common";
import type { Request } from "express";
import { BusinessError } from "../exceptions/business-error";

export const SESSION_CLIENT_HEADER = "clinicore-client";

export type SessionClientKind = "web" | "mobile";

export function sessionClientOf(request: Request): SessionClientKind {
  const declared = request.headers[SESSION_CLIENT_HEADER];

  if (declared === undefined) {
    return "web";
  }

  if (declared !== "mobile") {
    throw BusinessError.invalid("INVALID_CLIENT", "Invalid client");
  }

  return "mobile";
}

export const SessionClient = createParamDecorator(
  (_data: unknown, context: ExecutionContext): SessionClientKind =>
    sessionClientOf(context.switchToHttp().getRequest<Request>()),
);
