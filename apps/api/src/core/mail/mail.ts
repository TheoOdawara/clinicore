import { createTransport, type SendMailOptions } from "nodemailer";
import type { Environment } from "../config/env.validation";

export const MAIL_TRANSPORT = Symbol("MAIL_TRANSPORT");

export interface MailTransport {
  sendMail(options: SendMailOptions): Promise<unknown>;
}

export function createMailTransport(
  environment: Pick<
    Environment,
    "SMTP_HOST" | "SMTP_PORT" | "SMTP_USER" | "SMTP_PASSWORD"
  >,
): MailTransport {
  return createTransport({
    host: environment.SMTP_HOST,
    port: environment.SMTP_PORT,
    secure: false,
    requireTLS: true,
    auth: { user: environment.SMTP_USER, pass: environment.SMTP_PASSWORD },
  });
}
