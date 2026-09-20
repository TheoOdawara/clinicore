import pino, { type DestinationStream } from "pino";
import type { Environment } from "../config/env.validation";

const STDOUT = 1;

export function destinationFor(
  nodeEnvironment: Environment["NODE_ENV"],
): DestinationStream {
  if (nodeEnvironment !== "development") {
    return pino.destination(STDOUT);
  }

  return pino.transport({
    target: "pino-pretty",
    options: {
      colorize: true,
      translateTime: "HH:MM:ss.l",
      ignore: "pid,hostname,context,method,path,statusCode,durationMs",
      messageFormat:
        "{if context}[{context}] {end}{msg}{if method} {method} {path} → {statusCode} ({durationMs}ms){end}",
    },
  });
}
