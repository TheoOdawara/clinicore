import { Global, Module } from "@nestjs/common";
import { EnvironmentService } from "../config/environment.service";
import { MAIL_TRANSPORT, createMailTransport } from "./mail";
import { MailService } from "./mail.service";

@Global()
@Module({
  providers: [
    {
      provide: MAIL_TRANSPORT,
      inject: [EnvironmentService],
      useFactory: (environment: EnvironmentService) =>
        createMailTransport({
          SMTP_HOST: environment.get("SMTP_HOST"),
          SMTP_PORT: environment.get("SMTP_PORT"),
          SMTP_USER: environment.get("SMTP_USER"),
          SMTP_PASSWORD: environment.get("SMTP_PASSWORD"),
        }),
    },
    MailService,
  ],
  exports: [MailService],
})
export class MailModule {}
