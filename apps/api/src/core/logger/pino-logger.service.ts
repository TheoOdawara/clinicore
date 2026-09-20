import { Inject, Injectable, type LoggerService } from "@nestjs/common";
import type { Logger } from "pino";
import { LOGGER } from "./logger";

interface Payload {
  context?: string;
}

function toPayload(parameters: unknown[]): Payload {
  const last = parameters.at(-1);
  if (typeof last !== "string") {
    return {};
  }
  return { context: last };
}

@Injectable()
export class PinoLoggerService implements LoggerService {
  constructor(@Inject(LOGGER) private readonly logger: Logger) {}

  log(message: unknown, ...parameters: unknown[]): void {
    this.logger.info(toPayload(parameters), String(message));
  }

  error(message: unknown, ...parameters: unknown[]): void {
    this.logger.error(toPayload(parameters), String(message));
  }

  warn(message: unknown, ...parameters: unknown[]): void {
    this.logger.warn(toPayload(parameters), String(message));
  }

  debug(message: unknown, ...parameters: unknown[]): void {
    this.logger.debug(toPayload(parameters), String(message));
  }

  verbose(message: unknown, ...parameters: unknown[]): void {
    this.logger.trace(toPayload(parameters), String(message));
  }

  fatal(message: unknown, ...parameters: unknown[]): void {
    this.logger.fatal(toPayload(parameters), String(message));
  }
}
