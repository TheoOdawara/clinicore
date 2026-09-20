import { Inject, Injectable, type NestMiddleware } from "@nestjs/common";
import type { NextFunction, Request, Response } from "express";
import type { Logger } from "pino";
import { LOGGER } from "./logger";

const UNLOGGED_METHOD = "GET";
const UNLOGGED_PATH = "/health";
const NANOSECONDS_PER_MILLISECOND = 1e6;

function millisecondsSince(startedAt: bigint): number {
  const elapsed = process.hrtime.bigint() - startedAt;
  return Number((Number(elapsed) / NANOSECONDS_PER_MILLISECOND).toFixed(3));
}

function pathOf(request: Request): string {
  const [path] = request.originalUrl.split("?");
  return path ?? request.originalUrl;
}

@Injectable()
export class RequestLogMiddleware implements NestMiddleware {
  constructor(@Inject(LOGGER) private readonly logger: Logger) {}

  use(request: Request, response: Response, next: NextFunction): void {
    const path = pathOf(request);
    if (request.method === UNLOGGED_METHOD && path === UNLOGGED_PATH) {
      next();
      return;
    }

    const startedAt = process.hrtime.bigint();
    response.on("finish", () => {
      this.logger.info(
        {
          method: request.method,
          path,
          statusCode: response.statusCode,
          durationMs: millisecondsSince(startedAt),
        },
        "request",
      );
    });

    next();
  }
}
