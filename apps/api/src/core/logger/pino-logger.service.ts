import { Inject, Injectable, type LoggerService } from "@nestjs/common";
import type { Logger } from "pino";
import { LOGGER } from "./logger";

const STACK_FORMAT = /^(.)+\n\s+at .+:\d+:\d+/;

interface Payload {
  context?: string;
  stack?: string;
  err?: Error;
}

function toPayload(parameters: unknown[]): Payload {
  const last = parameters.at(-1);
  if (typeof last !== "string") {
    return {};
  }
  return { context: last };
}

function toErrorPayload(parameters: unknown[]): Payload {
  const [first] = parameters;
  if (typeof first !== "string") {
    return toPayload(parameters);
  }
  if (parameters.length === 1 && STACK_FORMAT.test(first)) {
    return { stack: first };
  }
  if (parameters.length === 1) {
    return { context: first };
  }
  return { ...toPayload(parameters), stack: first };
}

@Injectable()
export class PinoLoggerService implements LoggerService {
  constructor(@Inject(LOGGER) private readonly logger: Logger) {}

  log(message: unknown, ...parameters: unknown[]): void {
    this.logger.info(toPayload(parameters), String(message));
  }

  error(message: unknown, ...parameters: unknown[]): void {
    const payload = toErrorPayload(parameters);
    if (message instanceof Error) {
      this.logger.error({ ...payload, err: message }, message.message);
      return;
    }
    this.logger.error(payload, String(message));
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
