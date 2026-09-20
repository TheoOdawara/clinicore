import pino, { type DestinationStream, type Logger } from "pino";
import type { Environment } from "../config/env.validation";

export const LOGGER = Symbol("LOGGER");

export function createLogger(
  level: Environment["LOG_LEVEL"],
  destination: DestinationStream,
): Logger {
  return pino(
    {
      level,
      redact: [
        "req.headers.cookie",
        "req.headers.authorization",
        'res.headers["set-cookie"]',
        "req.body.password",
        "req.body.newPassword",
        "req.body.currentPassword",
        "req.body.token",
      ],
      serializers: {
        req: (request: { method: string; url: string }) => ({
          method: request.method,
          path: request.url.split("?")[0],
        }),
        res: (response: { statusCode: number }) => ({
          statusCode: response.statusCode,
        }),
      },
    },
    destination,
  );
}
