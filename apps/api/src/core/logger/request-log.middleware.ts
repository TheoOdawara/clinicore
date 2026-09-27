import { Inject, Injectable, type NestMiddleware } from "@nestjs/common";
import type { NextFunction, Request, Response } from "express";
import type { Logger } from "pino";
import { LOGGER } from "./logger";

@Injectable()
export class RequestLogMiddleware implements NestMiddleware {
  constructor(@Inject(LOGGER) private readonly logger: Logger) {}

  use(request: Request, response: Response, next: NextFunction): void {
    const nanosecondsPerMillisecond = 1e6;
    const millisecondsSince = (startedAt: bigint): number => {
      const elapsed = process.hrtime.bigint() - startedAt;
      return Number((Number(elapsed) / nanosecondsPerMillisecond).toFixed(3));
    };
    const path = request.originalUrl.split("?")[0] ?? request.originalUrl;
    if (request.method === "GET" && path === "/health") {
      next();
      return;
    }

    const startedAt = process.hrtime.bigint();
    response.on("close", () => {
      const line = {
        method: request.method,
        path,
        statusCode: response.statusCode,
        durationMs: millisecondsSince(startedAt),
      };
      if (!response.writableFinished) {
        this.logger.info({ ...line, aborted: true }, "request");
        return;
      }
      this.logger.info(line, "request");
    });

    next();
  }
}
