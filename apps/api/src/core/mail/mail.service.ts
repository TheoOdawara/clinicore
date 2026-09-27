import { Inject, Injectable } from "@nestjs/common";
import type { Logger } from "pino";
import { EnvironmentService } from "../config/environment.service";
import { LOGGER } from "../logger/logger";
import { MAIL_TRANSPORT, type MailTransport } from "./mail";
import type { MailMessage } from "./templates";

@Injectable()
export class MailService {
  constructor(
    @Inject(MAIL_TRANSPORT) private readonly transport: MailTransport,
    @Inject(LOGGER) private readonly logger: Logger,
    private readonly environment: EnvironmentService,
  ) {}

  send(to: string, message: MailMessage): void {
    const reasonOf = (error: unknown): string => {
      if (error instanceof Error) {
        return error.message;
      }

      return String(error);
    };

    this.transport
      .sendMail({
        from: { name: "Clinicore", address: this.environment.get("MAIL_FROM") },
        to,
        subject: message.subject,
        html: message.html,
        text: message.text,
      })
      .catch((error: unknown) => {
        this.logger.error(
          { email: to, reason: reasonOf(error) },
          "Mail delivery failed",
        );
      });
  }
}
